# Issue 916 — two_fidelity_bai G2 cost debt: δ/node_cap dilution + a non-δ-correct BAI baseline

**Status:** OPEN — GOAT G2(a) failure decomposition after the Issue-915 fixture fix. G1 certificate arms PASS (the 915 defect is gone); the remaining GOAT blocker is cost vs BAI-MCTS, with two independent causes that need separating before any G2 verdict is trustable.

**Parent:** Issue 915 (closed: the certificate invalidity was a BENCH fixture bug — internal slow samples centered at 0.0 — not a module defect; mechanism + fix in git history and `.benchmarks/615_two_fidelity_bai_goat.md`).

## Evidence (re-arm shakedown, `B615_TREES=8`, post-915 fixture fix, 2026-10-03)

| setting | 2FFS errs / cert | 2FFS cost (fast / slow) | BAI errs / capped | BAI cost | UCT B* |
|---|---|---|---|---|---|
| (5,8)  | 0/8, **8/8 certified** | 225 386 (616 / 56 192) | **4/8 errs**, 0 capped | 10 817 | 33.5M (no match) |
| (7,6)  | 0/8, **8/8 certified** | 987 702 (1 647 / 246 514) | **4/8 errs**, 0 capped | 5 342 | 33.5M (no match) |
| (10,3) | 0/8, **8/8 certified** | 574 838 (520 / 143 579) | **3/8 errs**, 0 capped | 3 840 | 33.5M (no match) |

G1: certificate-validity 0 invalid (was 42+), guard-fires 0/24 (was 24/40), picks 0/24 errors (CP arm needs n ≥ 59 to certify δ at zero errors — the full n = 300 run passes by construction; the 8-tree shakedown cannot, so the record's "shakedown: G1 all arms ✓" expectation was arithmetically wrong for that one arm).

## Cause (a) — the BAI-MCTS baseline is not δ-correct as implemented

The in-bench BAI arm = LUCB stop at the root over the SHARED UCT engine's sample-max/min point backups. It stops (0 capped) while erring 4/8, 4/8, 3/8 — three orders above δ = 0.05. The root-arm radii only cover arm-level sampling noise; the extreme-backup bias (`E[max of noisy means]` optimistic at Max nodes, disclosed in the bench header) is NOT covered, so the stop fires on miscalibrated intervals. The paper's own BAI-MCTS (the δ-correct bound-propagation class, Kaufmann–Koolen 2017) spends 8.8e5 samples at (5,8) — 80× ours — precisely because correct radii must cover the deep backup. **A cost win against a baseline that stops wrong is not the plan's "strict win at matched accuracy"; G2(a) is unadjudicable until the baseline delivers its own δ.** Fix direction: interval-propagating LUCB — per-leaf CIs from sample counts, internal CI = max-of-uppers / min-of-lowers (Eq. 6 over bounds, the same algebra the 2FFS module already implements), targeted leader/challenger descents, δ allocated a-priori. The shared `descend_and_sample` stays for the UCT context rows; BAI gets its own bound-propagation engine.

## Cause (b) — the primitive's own cost: uniform δ over NODE_CAP = 65 536

`SearchConfig::delta_for_node = δ / node_cap` (plan T1.4's a-priori cap precedent) gives δ_v ≈ 7.6e-7. The stitched radius needs ~1 500–2 000 slow samples per certified node to reach width 0.02 (β(n, 7.6e-7, 0.05) ≤ 0.01 first happens around n_k ≈ 2000 because δ_k = δ_v·2^−k keeps shrinking with the stage index). The (5,8) tree spends 56 192 slow samples over ~28 certified nodes ≈ 2 000 each — the arithmetic is exact. The paper's 2FFS uses 5.39e3 total samples on this setting; ours is ~10×.

This is the documented T1.4 tradeoff (uniform-over-cap vs the dyadic-rank alternative). Sound tightening options, in order of safety:

1. **Depth-budgeted a-priori allocation** — δ split over depth levels (the recursion provably samples at most one blocking path per level per root side), each level's mass uniform over that level's node count bound. A-priori, union-bound-safe, no adaptivity leak.
2. **The paper's own allocation** (Eq. set around Thm 3.6) — re-read Research 601 §allocation for the exact scheme; if it allocates over the RESOLVED set, port it verbatim with its own union proof, never ad hoc.
3. NOT sound: allocating over the runtime-revealed node count (post-hoc δ division breaks the union bound — the sampled set is outcome-dependent).

## What this issue owns

- (a) Rebuild the in-bench BAI baseline as the δ-correct bound-propagation class; re-judge G2(a) only against that.
- (b) Tighten the δ allocation behind a **new opt-in config knob** (never a silent change to the promoted default), with the union proof in the doc comment and the coverage pre-arm extended to the new allocation.
- Re-arm: `bench_615_two_fidelity_bai_goat` full protocol (100 trees/setting) — G1 must stay green (it is the certificate's gate) and G2(a) is then a real verdict. Phase 3/4 of plan 615 stay gated on that GOAT PASS.

## What this is NOT

- NOT a correctness finding: every certified interval in the shakedown contained V* (the 915 defect is gone); 0/24 pick errors.
- NOT a promotion candidate either way: the module is opt-in by design (certified-search slot).
