# Plan 615: Two-Fidelity Certified Best-Action Identification (2FFS)

**Status:** Active — Phase 1 IN FLIGHT (T1.1–T1.4 + T1.7 landed 2026-10-01: `two_fidelity_bai.rs` foundations — `MinimaxSpace` trait, `NodeInterval` fast/slow/child machinery, stitched time-uniform `beta` (geometric stages n_k = ⌈1.5^(k−1)⌉, δ_k = δ·2^−k; union-bound valid per stage, stage-monotone for every δ ≤ 2/3), `race_scale`, `SearchConfig` + uniform δ allocation, `Cost`/`TwoFidelityResult`; 10 module tests green at `--features two_fidelity_bai`, clippy clean, default lib 2074/0 unchanged). Remaining: T1.5 (root leader/challenger loop + Resolve races + latching), T1.6 (tie-breaking + empty-intersection guard), T1.8 (property-test sweep; interval nesting + Lemma 2.3 arithmetic already covered by unit tests). Phase 0 (filed; no code yet; T2 baselines corrected per verdict round 1: `mcts_search` is single-player UCT and cannot be the minimax baseline)

**Date:** 2026-10-01
**Research:** [katgpt-rs/.research/601_Two_Fidelity_BAI_Minimax_Tree.md](../.research/601_Two_Fidelity_BAI_Minimax_Tree.md)
**Source paper:** [arXiv:2606.01708](https://arxiv.org/abs/2606.01708) — Chen & Chen, *Two-Fidelity Best-Action Identification for Stochastic Minimax Tree*, NeurIPS 2026
**Target:** `crates/katgpt-core/src/two_fidelity_bai.rs` (new module) + Cargo feature `two_fidelity_bai`

---

## Goal

Ship the 2FFS primitive as a public, opt-in katgpt-core module: **certified two-fidelity best-action identification for minimax trees** — interval minimax backup (fast-envelope ∩ time-uniform slow CI) + leader/challenger ε-stopping + a budgeted local-vs-recursive race with latched dyadic side-certificates. GOAT gate: strict sample/ops win on a **unified cost model** (`cost = n_fast + c·n_slow`, c fixed per setting) over proper adversarial baselines — **BAI-MCTS** (the paper's own baseline) and a **negamax-UCT adapter** — at matched accuracy, PAC empirical error judged by a one-sided Clopper–Pearson bound, zero-alloc steady-state loop. `mcts_search` is single-player UCT (no Min nodes) and appears only as a labelled context row. Stays **opt-in** (certified-search tool, not a default path). Consumer map (recorded, not wired here): riir-ai `MultiHypothesisBoMMinimaxPlanner` certified stopping (Phase 3 seam), riir-instinct critic-guided search (watch), flat-LCB cousins unaffected.

Feature flag: `two_fidelity_bai = []` in katgpt-core. Promotion to default is NOT expected — the default MCTS slot keeps `mcts_search`; this fills the empty *certified-search* slot.

## Phase 1 — Core module (algorithm + invariants)

### Tasks

- [x] **T1.1** `two_fidelity_bai.rs` skeleton: `MinimaxSpace` trait — `kind(&self, node) -> NodeKind{Max,Min}`, `remaining_depth(&self, node) -> u8`, `children_into(&self, node, &mut ArrayVec)` (expansion reveals + caller fast-queries children), `fast_value(&self, node) -> f32`, `bias_envelope(&self, remaining_depth) -> f32`, `slow_sample(&self, node, &mut Rng) -> f32`. Doc-comment cites Research 601 + the paper's Definitions 2.2/2.3 and the B(h) monotonicity contract (Eq. 1, `B(0)=0`). (Landed 2026-10-01; `RngCore` is a module-local 1-method trait — zero RNG deps.)
- [x] **T1.2** Interval state per node: fast interval (fixed at exposure), slow running-intersection (Eq. 4: maintain `n_v`, `Σy`, `Σy²` or a Welford pair + current-radius clamp — the interval is `[mean_j − β(j)]` intersected over j; implement as running min-of-lower / max-of-upper endpoints, zero-alloc), local = fast ∩ slow, child-backup (Eq. 6), effective = local ∩ child. Monotone nesting (Lemma B.2) asserted in debug. (Landed 2026-10-01 as `NodeInterval`; Welford moments carried for diagnostics — the radius is the time-uniform bound, never the pointwise CLT.)
- [x] **T1.3** Time-uniform sub-Gaussian radius via stitching (Howard et al. 2021 / Kaufmann-Koolen 2021 class): `beta(n: u32, delta_v: f64, sigma: f64) -> f64`, non-increasing in n, documented as satisfying the paper's Eq. 3. Unit test: monotone in n, → 0 as n → ∞, and empirical coverage ≥ 1−δ_v over 10k sampled trajectories on a fixed-mean Gaussian (the G1 pre-arm). (Landed 2026-10-01: geometric stages n_k = ⌈1.5^(k−1)⌉ with δ_k = δ·2^−k and stage-constant radius σ·sqrt(2·ln(2·n_k/δ_k)/n_k) — per-stage union bound ≤ n_k counts ⇒ P(in-stage violation) ≤ δ_k, Σδ_k = δ; stage radii provably decreasing for every δ ≤ 2/3 (ratio test L ≤ c'). Coverage test: 2000 trajectories × 2000 steps, U[0,1] as 0.5-sub-Gaussian, seeded xorshift only (no global RNG), violations ≤ 2·δ_v·N_TRAJ.)
- [x] **T1.4** Feasible confidence allocation: `δ_v = δ / node_cap` uniform over an a-priori node cap (the `mcts.rs` MAX_TREE_SIZE precedent), doc-comment carrying the dynamic-tree tradeoff and the dyadic-rank alternative. `SearchConfig { epsilon, delta, slow_cost c, alpha_h schedule (h+1)², node_cap }`. (Landed 2026-10-01: `NODE_CAP = 65536`, `SearchConfig::delta_for_node`, `race_scale(remaining_depth) = (h+1)²`.)
- [ ] **T1.5** Root loop: leader/challenger + ε-stop (`L_â ≥ max U_{≠â} − ε`); `Resolve(v, side, scale)` with the race budget `α_h·Γ_v^(k)` (Γ from Eq. 7: 0 if `B(h) ≤ ρ_k/4` else `c·m_v(ρ_k)`), local reversibility, lazy comparison-margin child discharge (ρ_k/2), selector-case endpoint witnesses. `Done_s(v,k)` latching bitflags + capped unresolved-scale selection (`K_s^{≤k}`).
- [ ] **T1.6** Deterministic tie-breaking + the empty-intersection guard (terminate + default action, the paper's B.3 convention) + the ρ_0 = 0 early exit.
- [x] **T1.7** Result type: `TwoFidelityResult { best_action, cost { fast, slow }, certified: bool, root_intervals }` — the certificate is first-class output (the point of the primitive). (Landed 2026-10-01 with `Cost { fast, slow } + Cost::unified(c)`.)
- [ ] **T1.8** Property tests (paper Lemmas as invariants): Lemma 2.3 (backup preserves validity + width), Lemma B.2 (interval nesting over time), Lemma B.5 (no same-scale rework — latched scales never re-spend).

## Phase 2 — GOAT bench (the gate)

### Tasks

- [ ] **T2.1** Fixture generator: balanced b-ary stochastic minimax trees, leaf means ~ U[0,1], slow = N(μ_ℓ, σ), fast = V* + bias within B(h) (bias schedule `b̄·(h/D)`-scaled, B(h) = envelope). **Adversarial-bias arm**: a per-tree bias sign chosen to FLIP the fast-only minimax answer at the root — a fixed-sign bias can let fast-only win for free and never forces the two-oracle race. Exact ground truth by full minimax. Settings (D,b) ∈ {(5,8),(7,6),(10,3)}, seeded, 100 trees/setting (paper §4 parity).
- [ ] **T2.2** Baselines in-bench, all scored on the unified cost model `cost = n_fast + c·n_slow` (c fixed per setting; ops reported separately): (a) **BAI-MCTS** — UGapE-MCTS / LUCB-MCTS per Kaufmann & Koolen 2017 (arXiv:1706.02986), the paper's own baseline (parity row for the 163–1458× figure); (b) **negamax-UCT adapter** over the same `MinimaxSpace` (two-player UCT, sign-flipped backups); (c) fast-only (minimax on fast evals); (d) slow-only (fixed-depth + slow sampling). `mcts_search` appears ONLY as a labelled "not adversarial — context only" row (single-player UCT: fixed `player_id`, max-only backups — it estimates max-max values on these trees). Never mix `advance()`-budget units with oracle-query units.
- [ ] **T2.2b** Negamax-UCT adapter sanity arm (verdict-round-2 note): on a small tree (D=3, b=3) with zero slow noise, the adapter converges to the exact minimax value as budget grows — a weak adapter would inflate the G2 win; this arm pins it before any comparison row is believed.
- [ ] **T2.3** `benches/bench_615_two_fidelity_bai_goat.rs`: **G1** empirical PAC error with δ = 0.05 named, judged by a one-sided Clopper–Pearson 95% upper bound on the error count over the seeded suite (a zero-error run reads as consistent-with-δ, never pass-by-fiat) + T1.3 coverage arm + property arms (Lemma 2.3 / B.2 / B.5); **G2** strict paired win (LB95 > 0, the `grouped_evidence` Bench 905 paired protocol) on the unified cost model vs **BAI-MCTS and negamax-UCT** at matched accuracy (paper's 163–1458× vs BAI-MCTS is the target class; the bar is a strict win, not parity); **G3** `mcts.rs`/`chance_puct.rs` tests untouched and green (new module only); **G4** zero allocations in the steady-state search loop (counting allocator over 3 searches after warmup — the chance_puct G4 precedent). Canary-first (impossible floors fire).
- [ ] **T2.4** `.benchmarks/615_two_fidelity_bai_goat.md` record with the paper-parity table (163–1458× / 1.9–4.6× as context rows, our measured numbers on the unified cost model as the claim) + box-state provenance line.
- [ ] **T2.5** GOAT verdict recorded in the plan Status; if G2 fails against either adversarial baseline on any setting → honest record + stay opt-in + issue filed with the measured numbers (no silent demote; `mcts_search` was never a contender).

## Phase 3 — Consumer seams (recorded; wiring gated on GOAT pass)

### Tasks

- [ ] **T3.1** Example `examples/two_fidelity_bai_basic.rs` (the `mcts_state_action_cache_basic.rs` shape): synthetic tree, print cost/certificate/intervals.
- [ ] **T3.2** riir-ai seam note (NO code in this repo): `MultiHypothesisBoMMinimaxPlanner` (riir-ai `crates/riir-engine/src/bom_arena/multi_hypothesis_planner.rs`) certified stopping — fast = mean-belief worst-case score (envelope = BoM dispersion, shrinks as belief concentrates), slow = sampled-hypothesis minimax; the wiring reference planner is katgpt-micro-belief `BoMMinimaxPlanner` (`crates/katgpt-micro-belief/src/bom_arena.rs`, re-exported via katgpt-core's `micro_belief`). File the riir-ai issue only when the primitive is green (cites Research 601 + Bench 615). The strategic-tier-only constraint is recorded there (never per-tick; the op-count evidence). The per-step (flat) fusion is published prior art (Poiani 2022/2024) — the riir-ai issue must scope to the multi-step/tree shape or to plain cost savings, not to flat MF-BAI novelty.
- [ ] **T3.3** Watch entries (no files): riir-instinct critic-guided search (T7 round-4 budget re-derivation under certified early-stop); riir-infer kernel-trial early-stop (effective-gap shape). Both recorded in Research 601 §2.2 — nothing owed here.

## Phase 4 — Docs + gate hygiene

### Tasks

- [ ] **T4.1** Feature-catalog opt-in row (`.docs/09_feature_catalog/`) + README feature table entry.
- [ ] **T4.2** `full_gate.sh` residue check: the new module is feature-gated; confirm Layer 3 (`--all-features`) compiles it and the named bench target carries its `required-features` row (the cfg-gated-target green-zero rule).
- [ ] **T4.3** Doc-sync: Research 601 Status → Done; this plan → Phase states; HISTORY.md entry on GOAT pass.
