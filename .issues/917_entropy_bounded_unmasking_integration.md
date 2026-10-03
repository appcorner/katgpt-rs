# Issue 917: EB-Sampler-Class Entropy-Bounded Commitment for D2F/DDTree/Drafter Commit Paths

**Status:** Open — Gain-tier integration PoC (execution against a named published lineage; zero novelty claims)

**Source lineage (the controlling prior art):**
- Ben-Hamu, Gat, Severo, Nolte, Karrer (FAIR/Meta), ["Accelerated Sampling from Masked Diffusion Models via Entropy Bounded Unmasking"](https://arxiv.org/abs/2505.24857) (NeurIPS 2025) — the EB-Sampler: sort masked positions by an error proxy (entropy/confidence/margin), unmask the largest prefix whose `Σ H − max H ≤ γ` — a bound on the joint-dependence error (their Eq. 8). Measured 2–3× NFE reduction on LLaDa 8B / Dream 7B at quality parity.
- Same family: PC-Sampler (cumulative-entropy error tolerance), APD (adaptive per-step count), and the DiffusionGemma Technical Report's own algorithm (`Require: Entropy budget b = 0.1`).
- Composition into drafters is AUTHOR-SUGGESTED: EB-Sampler §7 — "Our EB-Sampler approach is complementary, and could be simply applied to speed up the draft model" (their spec-decode related work cites Christopher et al. arXiv:2408.05636, the DFlash precursor).
- Trigger source: the Srivastava essay ["Diffusion will be everywhere"](https://varunneal.github.io/essays/diffusion) (2026-09-27), whose appendix `entropy_bound` presents a prefix-sum variant (`Σ_{j<i} H_j ≤ τ`); full distill in `riir-train/.research/465_AR_to_Diffusion_Conversion_Essay.md`.

**Why this is an issue and not a novelty claim:** the verdict round-1 claim ("cumulative-entropy commitment is novel") was FALSIFIED by the reviewer's independent search under the correct vocabulary — recorded as the wrong-vocabulary lesson in Research 465 §4. What remains is execution: our shipped commit paths use fixed-k / per-position-threshold policies; the EB-class adaptive policy is absent workspace-wide.

## What we would build (modelless, zero training)

- [ ] **T1** `entropy_bounded_commit` in katgpt-core (suggested home: `canvas/` or `speculative/`): given per-position entropies `H`, an error-proxy order, and threshold γ, return the commit set via the EB residual bound `Σ H − max H ≤ γ` (implement the EB form, not the essay's prefix form — the EB form carries the Eq-8 joint-dependence bound; the prefix form is the simpler, weaker heuristic). Zero-alloc, fixed-size, sigmoid-free (this is argmax/cumsum territory).
- [ ] **T2** Wire into the D2F decode commit (riir-ai `gemma2_d2f` lane's τ_conf site) behind a feature flag — A/B vs the incumbent per-position threshold.
- [ ] **T3** Wire into DDTree expansion order (best-first search already sorts by a proxy; EB's adaptive count replaces the fixed width-k) and the DFlash block commit (`dflash_predict`) — the drafter-composition the EB paper itself suggests.
- [ ] **T4** Code-level prior-art check at PoC time: the ComfyUI-DiffusionGemma GitHub project surfaced during review — verify whether it implements the entropy-budget sampler before claiming any in-repo first (verdict reviewer's note; unverified as of filing).

## GOAT gate plan (for any future promotion)

- **G1 (guarantee correctness):** the no-stall property holds under adversarial entropy distributions (all-equal, one-dominant, degenerate-zero) — note this property is EB-Sampler's BY CONSTRUCTION (singleton U: `sum − max = 0 ≤ γ`, their Algorithm 1), so G1 asserts our implementation preserves it, it does not claim it as ours.
- **G2 (perf):** NFE / forward-pass count at equal task quality vs the incumbent fixed-k / per-position-threshold commit, at DDTree and D2F scales — the EB paper's own 2–3× is the external reference, our bar is a measured win on OUR lanes, not a reproduction of theirs.
- **G3 (no regression):** task quality at the matched-compute point ≥ incumbent (their Pareto claim, re-measured here).
- **G4 (alloc):** zero-alloc hot path preserved.

## Cousins (signal-diffs, kept separate by axis)

| Cousin | Axis | Diff |
|---|---|---|
| EB-Sampler (2505.24857) | MDM commit policy — the class source | We compose into drafter/DDTree paths; EB form adopted over the essay's prefix form |
| D2F τ_conf (Research 034, shipped) | Per-position remask threshold | Fixed per-position threshold vs adaptive cumulative budget |
| Research 243 Bebop | Step-level spec-decode acceptance forecast (γ adaptation) | Different mechanism axis: forecasts acceptance to size the verify budget; not a canvas commit policy |
| EntropyBifurcatedPruner (Research 164) | Entropy-routed screening strictness | Routes pruner strictness; not a commit-count policy |

**Filed:** 2026-10-03 · verdict path: spawn_agent fallback (primary reviewer rate-limited), 2 rounds REVISE→AGREE · related: riir-train Research 465, Plan 437
