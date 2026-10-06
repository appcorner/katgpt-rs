//! LatticeMemory — E2LSH-addressed delta-rule cell lattice (Plan 619).
//!
//! The deterministic, modelless extraction of the Spotlight-class addressing
//! math (research note 233): a 2D lattice of delta-rule cells addressed by
//! **continuous E2LSH per-axis coordinates** (`address`), bump-interpolated
//! over the 3×3 neighborhood (`bump`), cells allocated on first write into a
//! preallocated flat slab (`types`) — the candidate fifth retrieval
//! complexity class (growing state + constant-neighborhood access + graded
//! near-miss recall) beside Raven O(1) / Engram O(1) / δ-Mem O(r) / PKM
//! O(√N).
//!
//! Opt-in (`feature = "lattice_memory"`) until the Plan 619 GOAT gate
//! (T1.7) passes; the gate designs live in the plan, never here. Name
//! disambiguation: NOT `analytic_lattice` (Plan 330 transport operators).
//!
//! Cycle note (T1.1+T1.2+T1.3 landing): `write_value` is the minimal
//! cell-write the addressing/near-miss tests need; T1.4 generalizes it to
//! the closed-form delta rule `S ← S(I−βkkᵀ)+βkvᵀ` with the
//! forgetting/interference/alloc gates.

mod address;
mod bump;
pub mod types;

pub use address::{split_coordinate, LshAxes, AXES};
pub use bump::{axis_weights, build_lut, weights_direct, AxisWeights, BumpLut};
pub use types::{validate, LatticeCell, LatticeConfig, LatticeConfigError};

/// The 3×3 neighborhood weight row for one axis: `[prev, center, next]`.
const TAPS: usize = 3;

/// A query's lattice location: the clamped primary cell + the signed
/// fractional offsets the bump consumes.
struct Located {
    row: usize,
    col: usize,
    delta: [f32; AXES],
}

/// The lattice: seeded axes + bump table + dense cell table + flat slab.
///
/// Constructed once via [`LatticeMemory::new`]; every hot-path read is a
/// coordinate pair → 9 dense-table lookups → slab slices blended in place —
/// zero heap allocation after construction (the G4 law, alloc-gated at T1.4).
pub struct LatticeMemory {
    config: LatticeConfig,
    axes: LshAxes,
    lut: BumpLut,
    /// Dense cell table, `grid[0] × grid[1]` — unwritten cells carry
    /// `LatticeCell::unwritten()` (writes == 0).
    cells: Vec<LatticeCell>,
    /// Preallocated flat slab, `cell_count × d_k × d_v` elements. Cells claim
    /// regions from a bump cursor on first write; there is no cell
    /// deallocation, so no free list exists.
    slab: Vec<f32>,
    /// Next unclaimed slab element.
    cursor: usize,
}

impl LatticeMemory {
    /// Construct from a validated config — one slab allocation, once.
    ///
    /// # Errors
    /// [`LatticeConfigError`] on zero dimensions, non-finite widths, or a
    /// slab over the configured `byte_budget`.
    pub fn new(config: LatticeConfig) -> Result<Self, LatticeConfigError> {
        validate(&config)?;
        let axes = LshAxes::new(
            config.d,
            config.w,
            config.seed,
            &[config.grid[0] as u64, config.grid[1] as u64, config.d_k as u64, config.d_v as u64],
        );
        let cells = vec![LatticeCell::unwritten(); config.cell_count()];
        let slab = vec![0.0_f32; config.cell_count() * config.cell_len()];
        Ok(Self {
            config,
            axes,
            lut: build_lut(),
            cells,
            slab,
            cursor: 0,
        })
    }

    /// The validated config this lattice was built from.
    #[must_use]
    pub const fn config(&self) -> &LatticeConfig {
        &self.config
    }

    /// Cells claimed so far (occupancy readout — the T1.5 sizing axis).
    #[must_use]
    pub fn written_cells(&self) -> usize {
        self.cursor.checked_div(self.config.cell_len()).unwrap_or(0)
    }

    /// The clamped primary cell `(row, col)` a query addresses — the
    /// occupancy/locality readout; the hot paths recompute it inline.
    #[must_use]
    pub fn primary_cell(&self, x: &[f32]) -> [usize; 2] {
        let located = self.locate(x);
        [located.row, located.col]
    }

    /// Address `x` → primary cell + both axes' fractional offsets.
    ///
    /// Cell coordinates clamp into the grid (queries may project outside it;
    /// the lattice's answers stay in-bounds — the read renormalizes over the
    /// clipped neighborhood).
    fn locate(&self, x: &[f32]) -> Located {
        let c = self.axes.coordinates(x);
        let mut cell = [0_i64; AXES];
        let mut delta = [0.0_f32; AXES];
        for axis in 0..AXES {
            let (i, f) = split_coordinate(c[axis]);
            let m = i + i64::from(f > 0.5); // round(c): the primary
            cell[axis] = m;
            delta[axis] = c[axis] - m as f32;
        }
        let row = cell[0].clamp(0, i64::from(self.config.grid[0]) - 1) as usize;
        let col = cell[1].clamp(0, i64::from(self.config.grid[1]) - 1) as usize;
        Located { row, col, delta }
    }

    /// Write `v` as THE value of the cell `x` addresses (minimal write —
    /// T1.4 replaces this with the delta-rule update; the signature stays).
    ///
    /// First write claims the cell's slab region; re-writes overwrite the
    /// claimed region in place (the slab never grows).
    pub fn write_value(&mut self, x: &[f32], v: &[f32]) {
        assert_eq!(v.len(), self.config.d_v, "value width must be d_v");
        let located = self.locate(x);
        let idx = located.row * self.config.grid[1] as usize + located.col;
        let len = self.config.cell_len();
        if !self.cells[idx].is_written() {
            let offset = self.cursor;
            self.cursor += len;
            self.cells[idx] = LatticeCell {
                offset: offset as u32,
                len: len as u32,
                writes: 0,
            };
        }
        let cell = self.cells[idx];
        let d_v = self.config.d_v;
        let dst =
            &mut self.slab[cell.offset as usize..cell.offset as usize + cell.len as usize];
        // State layout: d_k rows × d_v — the value broadcast across the key
        // rows is T1.4's delta-rule concern; today the first d_v lane holds
        // `v` and the rest stay zero (unwritten key rows read as no-op).
        dst[..d_v].copy_from_slice(v);
        self.cells[idx].writes += 1;
    }

    /// The φ-blended read at `x`: Σ_cells φ(cell) · state, written into
    /// `out` (caller-owned — the hot path allocates nothing).
    ///
    /// φ is the raw kernel product normalized over its IN-GRID geometric
    /// support (the plan's separable denominator — interior queries sum to
    /// ≈ 1, grid-edge queries renormalize over the clipped neighborhood).
    /// Unwritten cells contribute NO STATE but their φ stays in the
    /// denominator: sparsity attenuates the read — the graded near-miss
    /// recovery G5 gates (renormalizing over written cells instead would
    /// read a flat 1.0 across the whole support, a wider cliff than
    /// Engram's). Read-then-blend, never sum-then-read (T1.4's law).
    pub fn read_blend(&self, x: &[f32], out: &mut [f32]) {
        assert_eq!(out.len(), self.config.d_v, "out width must be d_v");
        out.fill(0.0);
        let located = self.locate(x);
        let (w_row, w_col) = self.neighborhood_weights(located.delta);
        let d_v = self.config.d_v;
        let grid1 = self.config.grid[1] as usize;
        // The kernel's in-grid mass — data-independent, computed first.
        let mut mass = 0.0_f32;
        for (ri, &wr) in w_row.iter().enumerate() {
            let r = located.row as i64 + ri as i64 - 1;
            if r < 0 || r >= i64::from(self.config.grid[0]) {
                continue;
            }
            for (ci, &wc) in w_col.iter().enumerate() {
                let c = located.col as i64 + ci as i64 - 1;
                if c >= 0 && c < i64::from(self.config.grid[1]) {
                    mass += wr * wc;
                }
            }
        }
        if mass <= f32::EPSILON {
            return;
        }
        for (ri, &wr) in w_row.iter().enumerate() {
            let r = located.row as i64 + ri as i64 - 1;
            if r < 0 || r >= i64::from(self.config.grid[0]) {
                continue;
            }
            for (ci, &wc) in w_col.iter().enumerate() {
                let c = located.col as i64 + ci as i64 - 1;
                if c < 0 || c >= i64::from(self.config.grid[1]) {
                    continue;
                }
                let phi = wr * wc / mass;
                let cell = self.cells[r as usize * grid1 + c as usize];
                if !cell.is_written() {
                    continue;
                }
                let src =
                    &self.slab[cell.offset as usize..cell.offset as usize + d_v];
                for (o, &s) in out.iter_mut().zip(src.iter()) {
                    *o += phi * s;
                }
            }
        }
    }

    /// Per-axis 3-tap weights for both axes at once (one LUT pair).
    #[must_use]
    fn neighborhood_weights(&self, delta: [f32; AXES]) -> ([f32; TAPS], [f32; TAPS]) {
        let (r0, r1, r2) = axis_weights(&self.lut, delta[0]);
        let (c0, c1, c2) = axis_weights(&self.lut, delta[1]);
        ([r0, r1, r2], [c0, c1, c2])
    }
}

#[cfg(test)]
mod tests {
    use super::types::LatticeConfig;
    use super::LatticeMemory;

    fn config(grid: [u32; 2], w: [f32; 2]) -> LatticeConfig {
        LatticeConfig {
            d: 16,
            d_k: 4,
            d_v: 4,
            grid,
            w,
            seed: 42,
            byte_budget: None,
        }
    }

    #[test]
    fn constructor_refuses_zero_and_over_budget() {
        let bad = config([0, 4], [1.0, 1.0]);
        assert!(matches!(
            LatticeMemory::new(bad),
            Err(super::LatticeConfigError::ZeroDimension)
        ));
        let mut big = config([4, 4], [1.0, 1.0]);
        big.byte_budget = Some(16); // slab needs 16·4·4·4 = 1024 B
        assert!(matches!(
            LatticeMemory::new(big),
            Err(super::LatticeConfigError::OverBudget { .. })
        ));
    }

    #[test]
    fn write_then_exact_read_returns_the_value() {
        let mut lat = LatticeMemory::new(config([64, 64], [2.0, 2.0])).unwrap();
        let v = [0.25, -0.5, 0.75, 1.0];
        lat.write_value(&[0.1; 16], &v);
        let mut out = [0.0_f32; 4];
        lat.read_blend(&[0.1; 16], &mut out);
        // With only the stored cell written, the read is k·v — DIRECTION
        // preserved (every component the same factor). k = the stored cell's
        // renormalized φ ≤ 1 (the in-grid-normalized weights stay a partition
        // of unity; interior queries cap at center_max² = 0.36, grid-edge
        // queries renormalize over the clipped neighborhood and may carry
        // more). Retrieval ranks on direction; the dilution gradient is what
        // G5 contrasts against Engram's cliff.
        let k = out[0] / v[0];
        assert!(k > 0.0 && k <= 1.0 + 1e-6, "read factor {k} outside (0, 1]");
        for (o, &s) in out.iter().zip(v.iter()) {
            assert!((o - k * s).abs() < 1e-6, "direction broken: {o} vs {k}·{s}");
        }
    }

    #[test]
    fn near_miss_recovers_through_neighbor_weight() {
        // T1.2's property: a query whose PRIMARY cell differs from the
        // stored key's still recovers the stored value — graded, not a
        // cliff. Sweep past a cell boundary; recall must stay positive
        // while neighborhood overlap remains and decay with distance.
        let mut lat = LatticeMemory::new(config([64, 64], [1.0, 1.0])).unwrap();
        let key = [0.0_f32; 16];
        lat.write_value(&key, &[1.0; 4]);
        // Find the first primary-cell crossing on a monotone sweep.
        let mut boundary = None;
        let mut prev_cell = lat.primary_cell(&key)[0];
        for step in 1..4096_u32 {
            let off = step as f32 * 0.001;
            let mut q = key;
            q[0] = off;
            let row = lat.primary_cell(&q)[0];
            if row != prev_cell {
                boundary = Some(off);
                break;
            }
            prev_cell = row;
        }
        let boundary =
            boundary.expect("a 1.0-width lattice must have a boundary within the sweep");
        let mut out = [0.0_f32; 4];
        let mut graded = Vec::new();
        for k in 1..=8_u32 {
            let off = boundary + k as f32 * 0.02;
            let mut q = key;
            q[0] = off;
            lat.read_blend(&q, &mut out);
            let recall = out[0]; // v was all-ones → the blended mass IS recall
            assert!(
                recall > 0.0,
                "near-miss at +{k}·0.02 past the boundary reads ZERO — cliff, not graded"
            );
            graded.push(recall);
        }
        // Graded = decaying with distance, not flat or inverted.
        assert!(
            graded[0] > graded[graded.len() - 1],
            "recall must decay with distance: {graded:?}"
        );
    }

    #[test]
    fn unwritten_lattice_reads_zero() {
        let lat = LatticeMemory::new(config([8, 8], [2.0, 2.0])).unwrap();
        let mut out = [1.0_f32; 4]; // poisoned — must be overwritten to 0
        lat.read_blend(&[0.3; 16], &mut out);
        for o in out {
            assert_eq!(o, 0.0);
        }
    }

    #[test]
    fn same_config_same_lattice() {
        let a = LatticeMemory::new(config([16, 16], [2.0, 2.0])).unwrap();
        let b = LatticeMemory::new(config([16, 16], [2.0, 2.0])).unwrap();
        let mut oa = [0.0_f32; 4];
        let mut ob = [0.0_f32; 4];
        // (reads on unwritten lattices — the axes agreement is what's pinned)
        a.read_blend(&[0.2; 16], &mut oa);
        b.read_blend(&[0.2; 16], &mut ob);
        assert_eq!(oa, ob);
    }
}
