//! Continuous E2LSH per-axis addressing (Plan 619 T1.2).
//!
//! The REJECTED form is recorded in the plan: integer-split SimHash (sign-bit
//! split → two integers) — a 1-bit high-order change moves |Δx| ≫ 1 (lattice
//! neighbors ≠ embedding neighbors) and yields no fractional part, killing
//! the bump. The adopted form: per axis ℓ, the coordinate of embedding `x`
//! is `(a_ℓ·x + b_ℓ) / w_ℓ` with a seeded p-stable (Gaussian) `a_ℓ`, uniform
//! `b_ℓ ∈ [0, w_ℓ)` — the integer part is the cell coordinate, the
//! **fractional part is the bump input** (real, non-zero — the property the
//! rejected form could not give).
//!
//! Seeds follow the shared `crate::lsh_seed::config_seed` discipline (the
//! Issue 809 law, moved down from `katgpt-pruners/lsh_cache.rs`): same
//! configuration → same projection stream, every process.

use crate::lsh_seed::config_seed;

/// The lattice axis count (a 2D lattice: 3×3 neighborhoods, per the plan).
pub const AXES: usize = 2;

/// Seeded per-axis continuous-LSH projector.
///
/// One Gaussian direction `a_ℓ ∈ R^d`, one offset `b_ℓ ∈ [0, w_ℓ)`, one width
/// per axis. Draws come from a single `fastrand::Rng` seeded by FNV-1a over
/// the configuration scalars; construction is O(AXES · d) and happens once.
#[derive(Clone, Debug)]
pub struct LshAxes {
    /// Per-embedding-dim projection entries, both axes at once (row-major:
    /// one row per `x` element, `[axis0, axis1]` per row) — one pass over
    /// `x` computes both coordinates.
    a: Vec<[f32; AXES]>,
    b: [f32; AXES],
    w: [f32; AXES],
    d: usize,
}

impl LshAxes {
    /// Build the seeded axes for a `d`-dim embedding and per-axis widths.
    ///
    /// `parts` extends the seed feed — callers fold THEIR distinguishing
    /// scalars in (the lattice folds grid + widths; a second consumer folds
    /// its own) so two configs never share a projection stream by accident.
    #[must_use]
    pub fn new(d: usize, w: [f32; AXES], seed: u64, parts: &[u64]) -> Self {
        let mut feed = Vec::with_capacity(parts.len() + 2 + AXES);
        feed.push(d as u64);
        feed.push(seed);
        for axis_w in w {
            feed.push(u64::from(axis_w.to_bits()));
        }
        feed.extend_from_slice(parts);
        let mut rng = fastrand::Rng::with_seed(config_seed(&feed));
        // Gaussian directions via Box–Muller from the seeded uniform stream —
        // deterministic given the seed, zero new deps (fastrand is already a
        // katgpt-core dep).
        let mut a = Vec::with_capacity(d);
        for _ in 0..d {
            let u1 = rng.f32().max(f32::EPSILON);
            let u2 = rng.f32();
            let mag = (-2.0 * u1.ln()).sqrt();
            a.push([
                mag * (core::f32::consts::TAU * u2).cos(),
                mag * (core::f32::consts::TAU * u2).sin(),
            ]);
        }
        let b = [rng.f32() * w[0], rng.f32() * w[1]];
        Self { a, b, w, d }
    }

    /// Embedding dimension.
    #[must_use]
    pub const fn d(&self) -> usize {
        self.d
    }

    /// Both axis coordinates for `x` — ONE pass over the projection rows
    /// computes both axes (the hot path never walks `x` twice).
    ///
    /// The inner loop is branch-free and 2-wide per row (LLVM pairs the two
    /// accumulators); `x` shorter than `d` is treated as zero-padded, longer
    /// is truncated — the caller's config `d` is the contract.
    #[must_use]
    pub fn coordinates(&self, x: &[f32]) -> [f32; AXES] {
        let n = self.d.min(x.len());
        let mut d0 = 0.0f32;
        let mut d1 = 0.0f32;
        for (i, row) in self.a[..n].iter().enumerate() {
            let x_i = x[i];
            d0 += row[0] * x_i;
            d1 += row[1] * x_i;
        }
        [
            (d0 + self.b[0]) / self.w[0],
            (d1 + self.b[1]) / self.w[1],
        ]
    }
}

/// Integer + fractional split of a continuous coordinate.
///
/// `floor` is the cell coordinate (clamped into the grid by the caller),
/// `frac ∈ [0, 1)` is the bump input — the real fractional part the rejected
/// SimHash form could not produce.
#[must_use]
pub fn split_coordinate(c: f32) -> (i64, f32) {
    let floored = c.floor();
    let frac = c - floored;
    (floored as i64, frac)
}

#[cfg(test)]
mod tests {
    use super::{split_coordinate, AXES, LshAxes};

    fn axes_for(d: usize, w: [f32; AXES], seed: u64) -> LshAxes {
        LshAxes::new(d, w, seed, &[7, 15])
    }

    #[test]
    fn same_config_same_axes() {
        // The Issue 809 law at the lattice: identical configs → identical
        // projection streams → identical coordinates, bit-for-bit.
        let a = axes_for(16, [4.0, 4.0], 42);
        let b = axes_for(16, [4.0, 4.0], 42);
        let x: Vec<f32> = (0..16).map(|i| (i as f32) * 0.25 - 1.5).collect();
        assert_eq!(a.coordinates(&x), b.coordinates(&x));
    }

    #[test]
    fn different_seed_different_axes() {
        let a = axes_for(16, [4.0, 4.0], 42);
        let b = axes_for(16, [4.0, 4.0], 43);
        let x: Vec<f32> = (0..16).map(|i| (i as f32) * 0.25 - 1.5).collect();
        assert_ne!(a.coordinates(&x), b.coordinates(&x));
    }

    #[test]
    fn locality_small_perturbation_stays_close() {
        // T1.2's load-bearing property: a small embedding perturbation lands
        // in the same or an adjacent cell with high probability — per axis,
        // across seeds. Measured over 200 trials × 8 seeds: |Δcell| ≤ 1 must
        // dominate (≥ 95%); a failure here is the plan's STOP gate.
        let d = 32;
        let trials = 200;
        let eps = 0.05_f32;
        let mut close = 0_u32;
        let mut total = 0_u32;
        for seed in 0..8_u64 {
            let axes = axes_for(d, [2.0, 2.0], seed);
            let mut rng = fastrand::Rng::with_seed(seed ^ 0x9e37_79b9);
            for _ in 0..trials {
                let x: Vec<f32> = (0..d).map(|_| rng.f32() * 2.0 - 1.0).collect();
                let mut x2 = x.clone();
                for v in &mut x2 {
                    *v += eps * (rng.f32() * 2.0 - 1.0);
                }
                let (c0, c1) = { (axes.coordinates(&x), axes.coordinates(&x2)) };
                for axis in 0..AXES {
                    let delta = (c0[axis] - c1[axis]).abs();
                    if delta <= 1.0 + f32::EPSILON {
                        close += 1;
                    }
                    total += 1;
                }
            }
        }
        let rate = f64::from(close) / f64::from(total);
        assert!(
            rate >= 0.95,
            "locality FAILED the plan's premise: same/adjacent-cell rate {rate:.4} < 0.95 \
             — per Plan 619 T1.2 this STOPS the lattice claim (restate as Engram+delta-cells)"
        );
    }

    #[test]
    fn fractional_part_not_degenerate() {
        // The bump's input must be a real, spread distribution — not
        // pinned at 0/½, not collapsed. Chi-square-lite over 16 bins.
        let d = 32;
        let axes = axes_for(d, [2.0, 2.0], 7);
        let mut rng = fastrand::Rng::with_seed(0xDECA_FBAD);
        let bins = 16;
        let mut hist = [0_u32; 16];
        let n = 4000;
        for _ in 0..n {
            let x: Vec<f32> = (0..d).map(|_| rng.f32() * 2.0 - 1.0).collect();
            let c = axes.coordinates(&x);
            let (_, frac) = split_coordinate(c[0]);
            let bin = ((frac * bins as f32) as usize).min(bins - 1);
            hist[bin] += 1;
        }
        let expected = n / bins as u32;
        for (i, &count) in hist.iter().enumerate() {
            let share = f64::from(count) / f64::from(expected);
            // A degenerate distribution pins ≥ 2× expected in one bin; a
            // healthy uniform-ish spread stays within ±25% per bin.
            assert!(
                (0.75..=1.25).contains(&share),
                "fractional-part bin {i} at {share:.2}× expected — distribution degenerate"
            );
        }
    }

    #[test]
    fn split_is_exact_at_integers_and_fraction_in_range() {
        assert_eq!(split_coordinate(3.0), (3, 0.0));
        assert_eq!(split_coordinate(-0.25), (-1, 0.75));
        let (i, f) = split_coordinate(-2.5);
        assert_eq!((i, f), (-3, 0.5));
        let (i, f) = split_coordinate(17.125);
        assert_eq!((i, f), (17, 0.125));
    }
}
