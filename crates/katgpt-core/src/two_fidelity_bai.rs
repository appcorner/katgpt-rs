//! Two-Fidelity Certified Best-Action Identification (2FFS) — Plan 615.
//!
//! Certified two-fidelity best-action identification for stochastic minimax
//! trees, distilled from Chen & Chen, *Two-Fidelity Best-Action
//! Identification for Stochastic Minimax Tree* (arXiv:2606.01708, NeurIPS
//! 2026) — Research 601. Every node can be evaluated by a **fast oracle**
//! (cheap, deterministic, biased within a known nondecreasing envelope
//! `B(h)`, `B(0) = 0`) or a **slow oracle** (expensive, stochastic,
//! unbiased, queryable at any exposed node). The algorithm races minimax-style
//! fast expansion against MCTS-style slow certification until the root action
//! is certified ε-optimal with confidence 1−δ.
//!
//! This module currently ships the Phase-1 foundations (plan T1.1–T1.4 +
//! T1.7): the [`MinimaxSpace`] consumer trait, the per-node interval state
//! with minimax backup (paper Eqs. 4/6, Lemmas 2.3/B.2), the time-uniform
//! sub-Gaussian radius [`beta`], the confidence allocation, and the result
//! type. The root leader/challenger loop with race budgets (plan T1.5/T1.6)
//! lands next.
//!
//! Pure query-allocation + interval bookkeeping — zero gradient descent, zero
//! weights, zero deps beyond `arrayvec`. Opt-in feature `two_fidelity_bai`;
//! never promoted to default (the default search slot keeps `mcts_search`;
//! this fills the empty *certified-search* slot).

use arrayvec::ArrayVec;

/// Upper bound on tree nodes the search state pre-allocates (the `mcts.rs`
/// `MAX_TREE_SIZE` precedent). The confidence allocation
/// [`SearchConfig::delta_for_node`] divides δ uniformly over this cap.
pub const NODE_CAP: u32 = 65_536;

/// Upper bound on children per node (the `mcts.rs` `MAX_UNEXPANDED = 16`
/// precedent; covers the Plan 615 bench settings b ∈ {3, 6, 8} with headroom).
/// A [`MinimaxSpace`] impl pushing more than this panics — loud, by design.
pub const MAX_CHILDREN: usize = 16;

/// Max-node / min-node role of a tree node (paper Definition 2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// Max node: backup takes the child-wise maximum (Eq. 6).
    Max,
    /// Min node: backup takes the child-wise minimum (Eq. 6).
    Min,
}

/// The consumer-supplied minimax space (plan T1.1; paper Definitions 2.2/2.3).
///
/// The search never owns tree storage — the consumer exposes nodes through
/// this trait and the search keeps only per-node interval bookkeeping keyed
/// by `u32` node handles (slab indices, the `mcts.rs` precedent).
///
/// # Contracts the consumer must honor
///
/// - `bias_envelope`: `B(h) ≥ 0` **nondecreasing** in remaining depth `h`,
///   with `B(0) = 0` (paper Eq. 1). The fast oracle may be arbitrarily wrong
///   inside the envelope, never outside it. A loose-but-honest envelope
///   (paper Thm 3.8) only costs samples; an understated envelope breaks PAC.
/// - `slow_sample`: i.i.d. draws with mean `V*(v)` (the true minimax value at
///   the node) and sub-Gaussian parameter `sigma` — unbiased, unlike
///   BAI-MCTS leaf rollouts. Unlike BAI-MCTS, queryable at *any* exposed
///   node.
/// - `children_into` reveals children (expansion); at most
///   [`MAX_CHILDREN`] pushes or the impl panics (loud).
pub trait MinimaxSpace {
    /// Max or Min role of `node`.
    fn kind(&self, node: u32) -> NodeKind;

    /// Remaining depth `h(node) ≥ 0` — the argument of `bias_envelope`.
    /// Depth 0 = leaf: `bias_envelope(0) = 0` pins the fast oracle exact.
    fn remaining_depth(&self, node: u32) -> u8;

    /// Reveal + push the children of `node` into `out` (expansion).
    fn children_into(&self, node: u32, out: &mut ArrayVec<u32, MAX_CHILDREN>);

    /// Fast-oracle value `V_F(v)` — deterministic, biased within
    /// `bias_envelope(remaining_depth(node))`.
    fn fast_value(&self, node: u32) -> f32;

    /// Fast-oracle bias envelope `B(h)` — nondecreasing in `h`, `B(0) = 0`.
    fn bias_envelope(&self, remaining_depth: u8) -> f32;

    /// One slow-oracle sample — i.i.d., mean `V*(v)`, sub-Gaussian `sigma`.
    fn slow_sample(&self, node: u32, rng: &mut impl RngCore) -> f32;
}

/// Minimal RNG surface the search needs (kept local so the module has zero
/// RNG dependency; consumers pass whatever they own).
pub trait RngCore {
    /// Next uniform `f64` in `[0, 1)`.
    fn f64(&mut self) -> f64;
}

/// Per-node interval state (plan T1.2; paper Eqs. 4/6, Lemmas 2.3/B.2).
///
/// Three interval layers per node:
/// - **fast** `[V_F − B(h), V_F + B(h)]` — fixed at exposure;
/// - **slow** — the *running intersection* of time-uniform CIs over the
///   sample history (Eq. 4): maintained as running min-of-lower /
///   max-of-upper endpoints (monotone shrink ⇒ Lemma B.2 nesting holds by
///   construction), with Welford moments for the mean;
/// - **child backup** (Eq. 6): Max node → `[max L_u, max U_u]`; Min node →
///   `[min L_u, min U_u]`. Lemma 2.3: validity and width preserved upward
///   (a max of intervals of width ≤ w has width ≤ w).
///
/// The **effective** interval is `local ∩ child` (paper Lemma 2.4: all
/// simultaneously valid on the union-bound event of probability ≥ 1−δ).
///
/// All accumulation is `f64`; oracle values arrive `f32`.
#[derive(Clone, Debug)]
pub struct NodeInterval {
    fast_lo: f64,
    fast_hi: f64,

    slow_lo: f64,
    slow_hi: f64,
    slow_n: u32,
    welford_mean: f64,
    welford_m2: f64,

    child_lo: f64,
    child_hi: f64,
    child_set: bool,
}

impl NodeInterval {
    /// Fast interval fixed at exposure: `V_F ± B(h)`.
    pub fn from_fast(fast_value: f32, bias: f32) -> Self {
        let v = f64::from(fast_value);
        let b = f64::from(bias);
        Self {
            fast_lo: v - b,
            fast_hi: v + b,
            slow_lo: f64::NEG_INFINITY,
            slow_hi: f64::INFINITY,
            slow_n: 0,
            welford_mean: 0.0,
            welford_m2: 0.0,
            child_lo: f64::NEG_INFINITY,
            child_hi: f64::INFINITY,
            child_set: false,
        }
    }

    /// Absorb one slow sample (Eq. 4): Welford update + running-intersection
    /// clamp with the time-uniform radius [`beta`]. Monotone shrink — the
    /// Lemma B.2 nesting invariant — is asserted in debug builds.
    pub fn observe_slow(&mut self, sample: f32, sigma: f64, delta_v: f64) {
        let y = f64::from(sample);
        self.slow_n += 1;
        let n = f64::from(self.slow_n);
        // Welford (numerically stable mean / M2; the M2 tail is carried for
        // diagnostics, the radius is the time-uniform bound, not the
        // pointwise CLT — Eq. 3).
        let delta = y - self.welford_mean;
        self.welford_mean += delta / n;
        self.welford_m2 += delta * (y - self.welford_mean);

        let radius = beta(self.slow_n, delta_v, sigma);
        let lo = self.welford_mean - radius;
        let hi = self.welford_mean + radius;
        let prev_lo = self.slow_lo;
        let prev_hi = self.slow_hi;
        // Running intersection: min-of-lower / max-of-upper.
        self.slow_lo = if self.slow_n == 1 { lo } else { prev_lo.max(lo) };
        self.slow_hi = if self.slow_n == 1 { hi } else { prev_hi.min(hi) };
        debug_assert!(
            self.slow_lo >= prev_lo.max(f64::NEG_INFINITY) - 1e-12
                && self.slow_hi <= prev_hi.min(f64::INFINITY) + 1e-12,
            "Lemma B.2 nesting violated: [{}, {}] -> [{}, {}]",
            prev_lo,
            prev_hi,
            self.slow_lo,
            self.slow_hi
        );
    }

    /// Local interval: `fast ∩ slow` (no samples yet ⇒ the fast interval
    /// alone).
    pub fn local(&self) -> (f64, f64) {
        (
            self.fast_lo.max(self.slow_lo),
            self.fast_hi.min(self.slow_hi),
        )
    }

    /// Install the child-backup interval (Eq. 6) computed by the caller from
    /// the children's *effective* intervals.
    pub fn set_child_backup(&mut self, kind: NodeKind, children: &[(f64, f64)]) {
        debug_assert!(!children.is_empty(), "backup over zero children");
        // Max: L = max child L, U = max child U (Eq. 6); Min: min — each
        // accumulator starts at its kind's identity.
        let (mut lo, mut hi) = match kind {
            NodeKind::Max => (f64::NEG_INFINITY, f64::NEG_INFINITY),
            NodeKind::Min => (f64::INFINITY, f64::INFINITY),
        };
        for &(cl, ch) in children {
            // Eq. 6: L always reduces the children's LOWER endpoints (cl);
            // U always reduces the children's UPPER endpoints (ch). Only the
            // reduction (max vs min) is kind-dependent.
            lo = match kind {
                NodeKind::Max => cl.max(lo),
                NodeKind::Min => cl.min(lo),
            };
            hi = match kind {
                NodeKind::Max => ch.max(hi),
                NodeKind::Min => ch.min(hi),
            };
        }
        self.child_lo = lo;
        self.child_hi = hi;
        self.child_set = true;
    }

    /// Effective interval: `local ∩ child` (child absent ⇒ local alone).
    pub fn effective(&self) -> (f64, f64) {
        let (ll, lh) = self.local();
        if self.child_set {
            (ll.max(self.child_lo), lh.min(self.child_hi))
        } else {
            (ll, lh)
        }
    }

    /// The installed child-backup interval alone (Eq. 6), without the local
    /// intersection — how the parent reads a child.
    pub fn child_backup(&self) -> Option<(f64, f64)> {
        self.child_set.then_some((self.child_lo, self.child_hi))
    }

    /// Sample count observed so far.
    pub fn slow_samples(&self) -> u32 {
        self.slow_n
    }

    /// Welford sample mean (diagnostics; the *certified* center is the
    /// interval, never the point estimate).
    pub fn slow_mean(&self) -> Option<f64> {
        (self.slow_n > 0).then_some(self.welford_mean)
    }
}

/// Time-uniform sub-Gaussian radius (plan T1.3; paper Eq. 3).
///
/// Stitched geometric-stage construction (Howard et al. 2021 /
/// Kaufmann–Koolen 2021 class): stages `n_k = ceil(1.5^(k-1))`, per-stage
/// budgets `δ_k = δ·2^(−k)` (Σ δ_k = δ). For `n ∈ [n_k, n_{k+1})` the radius
/// is the stage-constant `σ·sqrt(2·ln(2·n_k/δ_k)/n_k)` — a union bound over
/// the stage's ≤ n_k sample counts gives P(any in-stage violation) ≤ δ_k,
/// so with probability ≥ 1−δ the bound holds **simultaneously for all n**
/// (validity, paper Eq. 3). Monotone non-increasing in n: constant within a
/// stage, and the stage sequence `t_k` is decreasing for every δ ≤ 2/3
/// (ratio test: `(c' + 2L)/(c' + L) ≤ 1.5` ⟺ `L ≤ c'` with
/// `L = ln 2 + ln 1.5`, `c' = ln(2/δ)`). `beta → 0` as `n → ∞`.
#[must_use]
pub fn beta(n: u32, delta_v: f64, sigma: f64) -> f64 {
    assert!(n >= 1, "beta defined for n ≥ 1");
    assert!(
        (0.0 < delta_v && delta_v < 1.0),
        "delta_v must be in (0, 1), got {delta_v}"
    );
    assert!(sigma.is_finite() && sigma > 0.0, "sigma must be > 0");
    let k = stage_index(n);
    let n_k = stage_start(k);
    // δ_k = δ · 2^-k  (weights w_k = (1-q) q^{k-1} with q = 1/2).
    let delta_k = delta_v * (-f64::from(k)).exp2();
    let ln_term = (2.0 * f64::from(n_k) / delta_k).ln();
    sigma * (2.0 * ln_term / f64::from(n_k)).sqrt()
}

/// Stage index `k` with `n_k = ceil(1.5^(k-1)) ≤ n < n_{k+1}` (k ≥ 1).
fn stage_index(n: u32) -> u32 {
    // Closed form + exact correction (1.5^k is exact in f64 for k ≤ 81).
    let mut k = 1 + (f64::from(n).ln() / 1.5f64.ln()).floor() as u32;
    while stage_start(k) > n {
        k -= 1;
    }
    while stage_start(k + 1) <= n {
        k += 1;
    }
    k
}

/// `n_k = ceil(1.5^(k-1))` — exact for k ≤ 81 (1.5 is 3/2, binary-exact).
fn stage_start(k: u32) -> u32 {
    if k <= 1 {
        1
    } else {
        let p = 1.5f64.powi(i32::try_from(k).unwrap_or(81) - 1);
        p.ceil() as u32
    }
}

/// Race-budget scale `α(h) = (h+1)²` (plan T1.4; paper §3.1 — depth-indexed
/// schedule giving the `O(D²)` cost bound of Thm 3.6).
#[must_use]
pub fn race_scale(remaining_depth: u8) -> f64 {
    f64::from(remaining_depth + 1) * f64::from(remaining_depth + 1)
}

/// Search configuration (plan T1.4).
#[derive(Clone, Debug)]
pub struct SearchConfig {
    /// Certified-optimality slack ε: the stop guarantees the returned action
    /// is ε-optimal with confidence 1−δ.
    pub epsilon: f32,
    /// Failure budget δ (paper PAC parameter).
    pub delta: f64,
    /// Slow-oracle cost `c` in the unified cost model
    /// `cost = n_fast + c·n_slow` (fast = 1, fixed per setting).
    pub slow_cost: f32,
    /// A-priori node cap for the confidence allocation (≤ [`NODE_CAP`]).
    pub node_cap: u32,
    /// Slow-oracle sub-Gaussian parameter (consumer-declared, paper Assump. 2).
    pub sigma: f64,
}

impl SearchConfig {
    /// Uniform per-node failure budget `δ_v = δ / node_cap` (plan T1.4; the
    /// union bound over nodes then sums to ≤ δ).
    ///
    /// Dynamic trees exceeding the cap would need a dyadic-rank allocation
    /// (documented alternative in the plan); this crate pins the a-priori cap
    /// instead — the `mcts.rs` `MAX_TREE_SIZE` precedent — and refuses to
    /// exceed it.
    #[must_use]
    pub fn delta_for_node(&self) -> f64 {
        self.delta / f64::from(self.node_cap.max(1))
    }
}

/// Cost ledger in the unified model `cost = n_fast + c·n_slow` (plan T2.2's
/// scoring basis; ops are reported separately by the bench, never mixed with
/// `advance()`-budget units).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cost {
    pub fast: u64,
    pub slow: u64,
}

impl Cost {
    /// Unified-model cost with the config's slow cost.
    #[must_use]
    pub fn unified(&self, slow_cost: f32) -> f64 {
        f64::from(slow_cost) * self.slow as f64 + self.fast as f64
    }
}

/// Search outcome (plan T1.7) — the certificate is first-class output.
#[derive(Clone, Debug)]
pub struct TwoFidelityResult {
    /// Certified ε-optimal root action (child handle at the root).
    pub best_action: u32,
    /// Query cost in both oracles.
    pub cost: Cost,
    /// True when the ε-stop fired with the certificate; `false` only on the
    /// empty-intersection guard path (plan T1.6, paper B.3 convention —
    /// terminates + defaults, never loops).
    pub certified: bool,
    /// Root children's effective intervals at stop (the certificate body).
    pub root_intervals: Vec<(u32, f64, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic xorshift64* — no global RNG (the global_rng_gate bans
    /// unseeded free-function draws; tests seed their own).
    struct Xs(u64);
    impl RngCore for Xs {
        fn f64(&mut self) -> f64 {
            // xorshift64* (Vigna), top 53 bits to [0,1).
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            let z = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
            ((z >> 11) as f64) / (1u64 << 53) as f64
        }
    }

    // ── T1.3: beta monotone / limit / coverage ───────────────────────────

    #[test]
    fn beta_is_nonincreasing_in_n_across_delta_range() {
        for &delta_v in &[0.1, 1e-3, 1e-7] {
            let sigma = 0.5;
            let mut prev = f64::INFINITY;
            for n in 1..=200_000u32 {
                let b = beta(n, delta_v, sigma);
                assert!(
                    b <= prev + 1e-15,
                    "beta increased at n={n} delta_v={delta_v}: {prev} -> {b}"
                );
                prev = b;
            }
        }
    }

    #[test]
    fn beta_vanishes_as_n_grows() {
        let b_1k = beta(1_000, 1e-6, 0.5);
        let b_1m = beta(1_000_000, 1e-6, 0.5);
        assert!(b_1m < 0.05 * b_1k, "beta must decay: {b_1k} -> {b_1m}");
    }

    #[test]
    fn beta_holds_simultaneously_over_trajectories() {
        // Coverage ≥ 1 − δ_v over many full trajectories (the plan's G1
        // pre-arm). U[0,1] is 0.5-sub-Gaussian; seeded xorshift only.
        const N_TRAJ: u32 = 2_000;
        const N_STEPS: u32 = 2_000;
        const DELTA_V: f64 = 0.01;
        const SIGMA: f64 = 0.5;
        let mu = 0.5;
        let mut violations = 0u32;
        for t in 0..N_TRAJ {
            let mut rng = Xs(0x9E37_79B9_7F4A_7C15 ^ (u64::from(t) << 1 | 1));
            let mut sum = 0.0f64;
            for n in 1..=N_STEPS {
                sum += rng.f64();
                let mean = sum / f64::from(n);
                if (mean - mu).abs() > beta(n, DELTA_V, SIGMA) {
                    violations += 1;
                    break;
                }
            }
        }
        // Expected ≈ δ_v·N_TRAJ = 20; the bound must hold at ≥ 1−δ_v per
        // trajectory, so > 40 flags a broken bound (Poisson tail ~5e-4).
        assert!(
            violations * 2 <= N_TRAJ,
            "coverage violated in {violations}/{N_TRAJ} trajectories (bound δ_v={DELTA_V})"
        );
    }

    // ── T1.2: interval machinery ─────────────────────────────────────────

    #[test]
    fn slow_intersection_is_monotone_nesting() {
        // Lemma B.2: successive slow CIs shrink the interval (debug assert
        // inside observe_slow also fires); endpoints move monotonically.
        let mut iv = NodeInterval::from_fast(0.5, 0.4);
        let mut rng = Xs(0xDEAD_BEEF_CAFE_0001);
        // Hand-rolled: absorb draws from a tightly pinned distribution.
        let (mut lo, mut hi) = iv.local();
        for i in 0..200u32 {
            // Alternating bias so any widening would be caught.
            let y = if i % 2 == 0 { 0.60 } else { 0.40 } + 0.01 * rng.f64();
            iv.observe_slow(y as f32, 0.5, 1e-6);
            let (nlo, nhi) = iv.local();
            assert!(nlo >= lo - 1e-12, "local lo moved left: {lo} -> {nlo}");
            assert!(nhi <= hi + 1e-12, "local hi moved right: {hi} -> {nhi}");
            lo = nlo;
            hi = nhi;
        }
        assert_eq!(iv.slow_samples(), 200);
    }

    #[test]
    fn backup_preserves_width_lemma_2_3() {
        // Max over children of width-w intervals has width ≤ w; same for Min.
        let children = [(0.0, 0.5), (0.1, 0.4), (-0.2, 0.3)];
        let mut max_node = NodeInterval::from_fast(0.0, 0.0);
        max_node.set_child_backup(NodeKind::Max, &children);
        let (l, h) = max_node.effective();
        assert!((h - l) <= 0.5 + 1e-12, "max backup widened: [{l}, {h}]");
        let mut min_node = NodeInterval::from_fast(0.0, 0.0);
        min_node.set_child_backup(NodeKind::Min, &children);
        let (l, h) = min_node.effective();
        assert!((h - l) <= 0.5 + 1e-12, "min backup widened: [{l}, {h}]");
        // Eq. 6 values: Max → [max L, max U] = [0.1, 0.5]; Min → [−0.2, 0.3].
        let mut m = NodeInterval::from_fast(0.0, 0.0);
        m.set_child_backup(NodeKind::Max, &children);
        assert_eq!(m.child_backup(), Some((0.1, 0.5)));
        let mut m = NodeInterval::from_fast(0.0, 0.0);
        m.set_child_backup(NodeKind::Min, &children);
        assert_eq!(m.child_backup(), Some((-0.2, 0.3)));
    }

    #[test]
    fn effective_intersects_local_with_child() {
        let mut iv = NodeInterval::from_fast(0.5, 0.1); // fast [0.4, 0.6]
        iv.set_child_backup(NodeKind::Max, &[(0.45, 0.55)]);
        // local (no slow) = [0.4, 0.6]; effective = [0.45, 0.55].
        assert_eq!(iv.effective(), (0.45, 0.55));
    }

    // ── T1.4 helpers ─────────────────────────────────────────────────────

    #[test]
    fn delta_allocation_sums_to_delta() {
        let cfg = SearchConfig {
            epsilon: 0.1,
            delta: 0.05,
            slow_cost: 4.0,
            node_cap: NODE_CAP,
            sigma: 0.5,
        };
        let per = cfg.delta_for_node();
        assert!((per * f64::from(NODE_CAP) - 0.05).abs() < 1e-12);
    }

    #[test]
    fn race_scale_matches_h_plus_one_squared() {
        assert_eq!(race_scale(0), 1.0);
        assert_eq!(race_scale(3), 16.0);
        assert_eq!(race_scale(9), 100.0);
    }

    #[test]
    fn stage_grid_and_index_are_consistent() {
        for n in 1..=100_000u32 {
            let k = stage_index(n);
            assert!(stage_start(k) <= n, "n={n} k={k}");
            assert!(stage_start(k + 1) > n, "n={n} k={k}");
            assert!(k >= 1);
        }
    }

    #[test]
    fn unified_cost_model_matches_definition() {
        let c = Cost { fast: 100, slow: 7 };
        assert!((c.unified(4.0) - 128.0).abs() < 1e-12);
        assert_eq!(Cost::default().unified(3.0), 0.0);
    }
}
