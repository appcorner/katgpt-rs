# Issue 916 — two_fidelity_bai G2 cost debt: δ/node_cap dilution + a non-δ-correct BAI baseline

**Status:** OPEN — cause (a) LANDED 2026-10-03 (commit `3aa27c149`, rebased over the 617 lane); cause (b) MEASURED 2026-10-03 (the per-node trace attribution landed — instrument `SearchConfig::trace_nodes` / `B615_2FFS_TRACE`, see §Cause (b); the lever it names is OPEN, opt-in-knob discipline applies); G1 certificate arms PASS (the 915 defect is gone); G2(a) reads a massive PASS against the corrected baseline (shakedown: the δ-correct LUCB-MCTS caps all 8 trees at every setting — 30M samples = 120M unified — LB95 ≈ +120M/setting vs 2FFS ≤ 1M).

**Parent:** Issue 915 (closed: the certificate invalidity was a BENCH fixture bug — internal slow samples centered at 0.0 — not a module defect; mechanism + fix in git history and `.benchmarks/615_two_fidelity_bai_goat.md`).

## Evidence (re-arm shakedown, `B615_TREES=8`, post-915 fixture fix, 2026-10-03)

| setting | 2FFS errs / cert | 2FFS cost (fast / slow) | BAI errs / capped | BAI cost | UCT B* |
|---|---|---|---|---|---|
| (5,8)  | 0/8, **8/8 certified** | 225 386 (616 / 56 192) | **4/8 errs**, 0 capped | 10 817 | 33.5M (no match) |
| (7,6)  | 0/8, **8/8 certified** | 987 702 (1 647 / 246 514) | **4/8 errs**, 0 capped | 5 342 | 33.5M (no match) |
| (10,3) | 0/8, **8/8 certified** | 574 838 (520 / 143 579) | **3/8 errs**, 0 capped | 3 840 | 33.5M (no match) |

G1: certificate-validity 0 invalid (was 42+), guard-fires 0/24 (was 24/40), picks 0/24 errors (CP arm needs n ≥ 59 to certify δ at zero errors — the full n = 300 run passes by construction; the 8-tree shakedown cannot, so the record's "shakedown: G1 all arms ✓" expectation was arithmetically wrong for that one arm).

## Cause (a) — RESOLVED: the BAI-MCTS baseline was not δ-correct as implemented

The in-bench BAI arm = LUCB stop at the root over the SHARED UCT engine's sample-max/min point backups. It stopped (0 capped) while erring 4/8, 4/8, 3/8 — three orders above δ = 0.05. The root-arm radii only cover arm-level sampling noise; the extreme-backup bias (`E[max of noisy means]` optimistic at Max nodes, disclosed in the bench header) is NOT covered, so the stop fires on miscalibrated intervals. **Fixed** (commit `3aa27c149`): the arm is now the LUCB-MCTS/UGapE-MCTS bound-propagation class — per-leaf time-uniform slow CIs through the module's own `NodeInterval` machinery (same β/δ_v rule as `two_fidelity_search`, fairness: the baseline shares 2FFS's confidence machinery, isolating the two-fidelity mechanism), Eq. 6 bound propagation (internal (L, U) = max|min child L/U, unrevealed children at the neutral sentinel), optimistic binding-child descent, root LUCB ε-stop, deterministic leader-L/challenger-U alternation. Module support: `NodeInterval::slow_only()` (the oracle-less carrier). Divergences from KKC documented at the site.

Shakedown verdict vs the corrected baseline (`B615_TREES=8`): **the δ-correct LUCB-MCTS CAPS all 8 trees at every setting** — 30M samples = 120M unified cost — because the leader's L at a Min arm needs the full b-ary fan-out (8·8 Max@1 nodes at (5,8), each with a CI-converged leaf chain) before it rises off −∞, and the U-side fans out at Min nodes symmetrically; ε = 0.02 tightness then needs ~2 000 diluted-δ samples per binding leaf. G2(a) reads **LB95 ≈ +120M per setting** (cap = the conservative understatement, the same convention as the UCT no-checkpoint-matched row); 2FFS rows bit-identical to the pre-fix run (determinism across builds confirmed). Disclosure for any claim built on this: the paper's own BAI instance stops at 8.8e5–1.91e7 samples where ours caps at 3e7 — our instance is the conservative member of the class (diluted uniform per-leaf δ + deterministic alternation), so a G2(a) win measured against it is an UPPER bound on the ratio vs a better-tuned instance; the paper's own numbers stay the cross-implementation context rows.

## Cause (b) — the primitive's own cost: ~10× the paper's sample count — ATTRIBUTED 2026-10-03 (trace measurement landed; lever next)

The (5,8) tree spends 56 192 slow samples; the paper's 2FFS uses 5.39e3 on this setting — ~10×. **Do not build on the first attribution written here**: the initial draft blamed the uniform `δ / node_cap` dilution; the closed-form arithmetic refutes that as the DOMINANT term. At an ε-driven width target w = 0.02 (σ = 0.05), n ≈ 8σ²·ln(2n_k/δ_k)/w² ≈ 50·L: the diluted δ_v = δ/65536 (δ_k ≈ 1e-12 at n ≈ 2000) gives L ≈ 35 ⇒ n ≈ 1800, while an undiluted δ_v ≈ 0.01 gives L ≈ 25 ⇒ n ≈ 1300 — a **~1.4× factor**, real but secondary.

**Measured attribution (2026-10-03, the `B615_2FFS_TRACE=1` per-node ledger — `SearchConfig::trace_nodes` + `TwoFidelityResult::node_stats`, 8-tree shakedown, all three settings):**

| finding | number | reading |
|---|---|---|
| sampled nodes per tree | ~6.5 (52 rows / 8 trees, d5b8) | concentrated: ~2 depth-1 + 1 depth-3 nodes eat ~85% of the slow budget |
| top depth-1 node's own slow CI | n ≈ 24.6k → width 2β ≈ 0.0071 | the ladder drove the side through rung k≈5 (target ρ_0/2⁶·2 ≈ 0.007) — **~350× finer than the ε = 0.02 stop can consume** |
| depth-3 share | 39–45% of total samples (8 nodes) | the RECURSIVE route certifies subtree interiors to the same fineness with zero root-side demand |
| final effective widths | ~1e-5 (vs targets 0.1125) | backups (Lemma 2.3) collapse widths past the node's own CI — the certificate body ends far finer than any decision needs |
| stage-δ halving at depth | δ_k = δ_v·2^-k; at n ≈ 16.8k (stage 25), δ_k ≈ 2.3e-14, ln ≈ 42 vs ~20 undiluted | **~2× n multiplier** from the stitched-stage construction alone |

So the ~7× residual decomposes ≈ **ladder rungs past the ε-necessary width (~2×) × stage-δ halving (~2×) × a-priori dilution 65 536-vs-8-arms (~1.5–2×)** — the draft's single-cause story was wrong in the expected direction: the dominant waste is STRUCTURAL (the rung ladder + stage weights), not the uniform cap dilution.

**The lever the measurement names (Issue lever 2/3 class — the paper's allocation over the resolved set, ε-referenced):** a (node, side) side's ladder may STOP at the first scale whose target ≤ the width the consuming decision can use — propagate the ROOT's required resolution down the recursion (a child certifies no finer than its parent's target; Lemma 2.3 makes that sufficient for the backup) and cap the ladder at the ε-referenced width at the root. Soundness requirements before any landing: the union δ is UNCHANGED (same δ_v per node — only fewer stages consumed per side, and latching a side early is the same latch the current code performs once width ≤ target); the ε-stop rule is untouched; the coverage pre-arm (`beta_holds_simultaneously_over_trajectories` + the G1 empirical PAC suite) must be extended to the new stopping rule and re-run at n = 300. **This changes the confidence allocation ⇒ behind a NEW opt-in config knob (never silent), per the standing rule.**

## What this issue owns

- [x] (a) Rebuild the in-bench BAI baseline as the δ-correct bound-propagation class — LANDED `3aa27c149`; G2(a) re-judged against it (shakedown PASS; the full-protocol run is the recording run).
- [x] (b-measure) Per-node sample accounting — **LANDED 2026-10-03**: `SearchConfig::trace_nodes` (pure observation, never affects the search) + `TwoFidelityResult::node_stats` + the unconditional `slow_n` counter + the bench `B615_2FFS_TRACE=1` dump (samples-by-depth, width-target histogram, top consumers). Attribution in §Cause (b).
- [ ] (b-lever) The ε-referenced / propagated-target allocation the measurement names — behind a NEW opt-in config knob (never a silent change), with the union proof in the doc comment and the coverage pre-arm extended (see §Cause (b)'s soundness requirements). Re-read Research 601 §allocation first (the paper's Thm 3.6 scheme may already be this exact shape — port verbatim if so).
- [ ] Re-arm: `bench_615_two_fidelity_bai_goat` full protocol (100 trees/setting) under the corrected baseline — DETACHED RUN IN FLIGHT (2026-10-03 ~06:1x, output `katgpt-rs/.runs/b615_full_out.txt`; the earlier foreground attempt was killed at the 60-min tool timeout mid-setting-2 with setting 1 complete: 2FFS 0/100 errs, 100/100 certified, cost mean 196,435; BAI 62/100 errs all capped). Its ALL-GATES-PASS is the GOAT verdict for plan 615 Phase 2 and un-gates Phase 3/4.

## What this is NOT

- NOT a correctness finding: every certified interval in the shakedown contained V* (the 915 defect is gone); 0/24 pick errors.
- NOT a promotion candidate either way: the module is opt-in by design (certified-search slot).
