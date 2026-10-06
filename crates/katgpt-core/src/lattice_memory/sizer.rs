//! Grid sizer — Plan 619 T1.5 (support-radius law).
//!
//! `LatticeMemory::sized` turns (expected items, near-miss radius) into a
//! grid + a fitted occupancy warp. The law, and why it is shaped this way:
//!
//! **The support radius drives the grid.** The class claim (graded
//! near-miss recall) lives in the 3×3 bump neighborhood — a query perturbed
//! by less than the support radius must recover through neighbor weight.
//! Under the CDF warp the coordinate slope at the median is
//! `0.399·L/σ_fit` cells per raw unit, so the ±1.5-cell support spans
//! `3.76·σ_fit/L` raw units: the grid side is `L = ⌈3.76/R⌉` for a desired
//! radius `R` (in fitted-σ units). Sizing from OCCUPANCY instead (the
//! first cut, `items/λ`) decoupled the two — at 10⁶ items it chose
//! L ≈ 700 and a 0.005σ support, the G5 gate measured the cliff (recall
//! 0.997 → 0.06 at ε = 0.02σ), and the law below is the repair.
//!
//! **Occupancy floats; the delta rule carries the crowding.** With L fixed
//! by the support, the per-cell mean is `λ = items/L²` — allowed to exceed
//! 1 because a cell is a `d_k × d_v` associative memory, not one slot.
//! The capacity gate: same-cell keys are ~N(0, 1/d_k) correlated, so each
//! write erodes older associations by `E[cos²] = 1/d_k`; survival ≥ ½
//! after λ overwrites gives `λ ≤ ln2·d_k`, halved for the cross-term noise
//! floor → the first-order capacity `0.35·d_k` (gated, not assumed — the
//! GOAT gates measure the real recall at the chosen operating point).
//!
//! **The CDF warp stays load-bearing.** It equalizes per-axis marginals
//! (uniform occupancy predictions hold — the histogram gate below) AND
//! decouples the grid from the projected range: tails clamp into the
//! boundary cells, so `L` is free of the ±4σ coverage arithmetic entirely.

use super::address::LshAxes;
use super::cdf::CdfWarp;
use super::types::{BumpKernel, LatticeConfig, LatticeConfigError};
use super::LatticeMemory;

/// Cells per support radius: the median CDF slope `0.399·L/σ` inverted —
/// a ±1.5-cell support spans `3.76σ/L` raw units. The sizer's one constant.
const CELLS_PER_SUPPORT: f32 = 3.76;

/// First-order per-cell capacity: `0.35·d_k` associations (see module docs
/// for the erosion derivation; the GOAT gates verify it empirically).
#[must_use]
pub fn capacity_per_cell(d_k: usize) -> f32 {
    0.35 * d_k as f32
}

/// The support radius a built lattice's grid actually delivers, in
/// fitted-σ units (the inverse of the sizing law — the honest readout for
/// callers that let the budget clamp their grid).
#[must_use]
pub fn support_radius(grid_side: u32) -> f32 {
    CELLS_PER_SUPPORT / grid_side as f32
}

impl LatticeMemory {
    /// Construct a lattice sized so that a query perturbed by less than
    /// `near_miss_sigma` (in fitted-σ units of the projected corpus) stays
    /// inside the graded 3×3 support. `samples` is the fit corpus
    /// (`d`-wide, ≥ 1 required).
    ///
    /// Refusals are the honest answers: [`LatticeConfigError::OverCapacity`]
    /// names the `d_k` the requested (items, radius) pair needs — widen the
    /// key width, shrink the corpus, or widen the radius — and the
    /// constructor's `byte_budget` still binds on the full slab.
    ///
    /// # Errors
    /// [`LatticeConfigError::BadNearMissRadius`] outside (0, 10],
    /// [`LatticeConfigError::EmptySample`] on no samples,
    /// [`LatticeConfigError::OverCapacity`] when the requested operating
    /// point exceeds `capacity_per_cell(d_k)`, then everything `new` refuses.
    // The plan (619 T1.5) names this constructor's signature; a params
    // struct would obscure the one-shot sizing call. The scalars are the
    // config, the sample slice is the fit corpus.
    #[allow(clippy::too_many_arguments)]
    pub fn sized(
        d: usize,
        d_k: usize,
        d_v: usize,
        seed: u64,
        samples: &[&[f32]],
        expected_items: usize,
        near_miss_sigma: f32,
        byte_budget: Option<usize>,
    ) -> Result<Self, LatticeConfigError> {
        if !(near_miss_sigma > 0.0 && near_miss_sigma <= 10.0) {
            return Err(LatticeConfigError::BadNearMissRadius(near_miss_sigma));
        }
        if samples.is_empty() {
            return Err(LatticeConfigError::EmptySample);
        }
        let side = (CELLS_PER_SUPPORT / near_miss_sigma).ceil().max(1.0) as u32;
        let lambda = expected_items as f32 / (side as usize * side as usize) as f32;
        let capacity = capacity_per_cell(d_k);
        if lambda > capacity {
            // The remedy is arithmetic: λ ≤ 0.35·d_k needs
            // d_k ≥ λ/0.35 — named, not gestured at.
            let required_d_k =
                (lambda / 0.35).ceil().max(1.0) as usize;
            return Err(LatticeConfigError::OverCapacity {
                items: expected_items,
                cells: side as usize * side as usize,
                capacity_per_cell: capacity,
                required_d_k,
            });
        }
        let axes = LshAxes::new(
            d,
            [1.0, 1.0],
            seed,
            &[side as u64, side as u64, d_k as u64, d_v as u64],
        );
        // Construction-time projection of the fit corpus (never the hot path).
        let mut proj: [Vec<f32>; 2] =
            [Vec::with_capacity(samples.len()), Vec::with_capacity(samples.len())];
        for s in samples {
            let c = axes.coordinates(s);
            proj[0].push(c[0]);
            proj[1].push(c[1]);
        }
        let warp = [CdfWarp::fit(&proj[0], side), CdfWarp::fit(&proj[1], side)];
        let config = LatticeConfig {
            d,
            d_k,
            d_v,
            grid: [side, side],
            w: [1.0, 1.0],
            seed,
            byte_budget,
            warp: Some(warp),
            bump_kernel: BumpKernel::default(),
        };
        LatticeMemory::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{BumpKernel, LatticeConfigError};
    use super::super::LatticeMemory;
    use super::{capacity_per_cell, support_radius};

    /// Deterministic standard Gaussian embeddings (Box–Muller over a seeded
    /// fastrand stream — the `address.rs` draw, test-side).
    fn gaussian_samples(d: usize, n: usize, seed: u64) -> Vec<Vec<f32>> {
        let mut rng = fastrand::Rng::with_seed(seed);
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let mut v = Vec::with_capacity(d);
            for _ in 0..d {
                let u1 = rng.f32().max(f32::EPSILON);
                let u2 = rng.f32();
                let mag = (-2.0 * u1.ln()).sqrt();
                v.push(mag * (core::f32::consts::TAU * u2).cos());
            }
            out.push(v);
        }
        out
    }

    #[test]
    fn sizer_refuses_bad_radius_empty_samples_and_over_capacity() {
        let samples = gaussian_samples(8, 16, 1);
        let refs: Vec<&[f32]> = samples.iter().map(|s| s.as_slice()).collect();
        for bad in [0.0_f32, -0.5, f32::NAN, f32::INFINITY, 11.0] {
            assert!(
                matches!(
                    LatticeMemory::sized(8, 4, 4, 7, &refs, 100, bad, None),
                    Err(LatticeConfigError::BadNearMissRadius(_))
                ),
                "radius {bad} must be refused"
            );
        }
        assert!(matches!(
            LatticeMemory::sized(8, 4, 4, 7, &[], 100, 0.1, None),
            Err(LatticeConfigError::EmptySample)
        ));
        // Over capacity: 1000 items into a coarse 0.5σ-radius grid (8×8 =
        // 64 cells → λ = 15.6) against d_k = 4 (capacity 1.4) — refused,
        // naming the d_k the point needs.
        let big = gaussian_samples(8, 1000, 2);
        let big_refs: Vec<&[f32]> = big.iter().map(|s| s.as_slice()).collect();
        match LatticeMemory::sized(8, 4, 4, 7, &big_refs, 1000, 0.5, None) {
            Err(LatticeConfigError::OverCapacity { required_d_k, .. }) => {
                assert!(required_d_k >= 40, "remedy d_k {required_d_k} too small");
            }
            Err(other) => panic!("expected OverCapacity, got {other}"),
            Ok(_) => panic!("expected OverCapacity, but the point was accepted"),
        }
    }

    #[test]
    fn occupancy_histogram_matches_the_cdf_corrected_prediction() {
        // The T1.5 gate: write the expected items, then the measured
        // per-cell histogram must match the (CDF-corrected) Poisson
        // prediction at the SUPPORT-driven grid's λ — uniform marginals are
        // the warp's load-bearing property.
        let n = 4000;
        let samples = gaussian_samples(16, n, 0x619_5101);
        let refs: Vec<&[f32]> = samples.iter().map(|s| s.as_slice()).collect();
        let mut lat = LatticeMemory::sized(16, 4, 4, 42, &refs, n, 0.05, None).unwrap();
        assert_eq!(lat.config().bump_kernel, BumpKernel::Tent);
        assert!(lat.config().warp.is_some());
        // The sizing law, both directions: ⌈3.76/0.05⌉ = 76, and the
        // delivered radius is never overpromised (ceil can only tighten).
        let side = lat.config().grid[0];
        assert_eq!(side, 76); // ⌈3.76/0.05⌉
        let delivered = support_radius(side);
        assert!(delivered <= 0.05 && delivered > 0.05 * 0.98, "delivered {delivered}");
        for s in &samples {
            lat.write_value(s, &[1.0; 4]);
        }
        // Per-cell occupancy over the written grid.
        let s_usize = side as usize;
        let cells = lat.config().cell_count();
        let mut occupied = vec![0_u32; cells];
        for s in &samples {
            let [r, c] = lat.primary_cell(s);
            occupied[r * s_usize + c] += 1;
        }
        let lambda = n as f32 / cells as f32;
        let predicted_ge2 = 1.0 - (-lambda).exp() * (1.0 + lambda);
        let ge2 = occupied.iter().filter(|&&o| o >= 2).count();
        let share = ge2 as f32 / cells as f32;
        assert!(
            (share - predicted_ge2).abs() <= 0.04,
            "P(≥2 items/cell) = {share:.4} vs predicted {predicted_ge2:.4} (λ = {lambda:.3})"
        );
        let max = *occupied.iter().max().unwrap();
        assert!(
            (max as f32) <= lambda + 6.0 * lambda.max(1.0).sqrt(),
            "a wrapped cell holds {max} items — past the Poisson tail at λ = {lambda:.3}"
        );
        // Everything claimed exactly once per written cell; occupancy count
        // matches the histogram's nonzero cells.
        let nonzero = occupied.iter().filter(|&&o| o > 0).count();
        assert_eq!(lat.written_cells(), nonzero);
        // And the read stays graded on the sized lattice (warp didn't break
        // the near-miss property) — at the support radius the lattice was
        // SIZED for, not an arbitrary one.
        let mut q = samples[0].clone();
        let radius = support_radius(side) * 0.5;
        let mut rng = fastrand::Rng::with_seed(0x619_0619);
        for v in q.iter_mut() {
            *v += radius * (rng.f32() * 2.0 - 1.0);
        }
        let mut out = [0.0_f32; 4];
        lat.read_blend(&q, &mut out);
        assert!(out.iter().sum::<f32>() > 0.0, "sized-lattice near-miss read a cliff");
    }

    #[test]
    fn unwarped_grid_overloads_the_center() {
        // The contrast that proves the correction does something: the SAME
        // embeddings on the SAME grid without the warp pile into the central
        // columns — the overload the Gaussian-projection geometry predicts.
        let n = 4000;
        let samples = gaussian_samples(16, n, 0x619_5101);
        let refs: Vec<&[f32]> = samples.iter().map(|s| s.as_slice()).collect();
        let warped = LatticeMemory::sized(16, 4, 4, 42, &refs, n, 0.05, None).unwrap();
        let mut cfg = warped.config().clone();
        cfg.warp = None;
        let mut raw = LatticeMemory::new(cfg).unwrap();
        for s in &samples {
            raw.write_value(s, &[1.0; 4]);
        }
        let count = |lat: &LatticeMemory| {
            let s = lat.config().grid[0] as usize;
            let mut occ = vec![0_u32; lat.config().cell_count()];
            for sample in &samples {
                let [r, c] = lat.primary_cell(sample);
                occ[r * s + c] += 1;
            }
            *occ.iter().max().unwrap()
        };
        let (raw_max, wrapped_max) = (count(&raw), count(&warped));
        assert!(
            raw_max >= 3 * wrapped_max.max(1),
            "unwrapped peak {raw_max} must dwarf the wrapped peak {wrapped_max}"
        );
        assert!(raw_max >= 12, "unwrapped peak {raw_max} below the overload floor");
    }

    #[test]
    fn capacity_law_matches_the_erosion_derivation() {
        // The pinned first-order estimate: d_k = 32 → 11 associations per
        // cell (ln2·32 = 22, halved for the cross-term noise floor).
        assert!((capacity_per_cell(32) - 11.2).abs() < 1e-6);
        assert!((capacity_per_cell(4) - 1.4).abs() < 1e-6);
    }
}
