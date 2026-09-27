# Issue 903: `row_logit_floor` promotion lane — quality side UNBLOCKED by riir-infer Bench 003 (T3c PASS at 64K)

**Status:** Boxes COMPLETE 2026-09-27 — all four measured and PASS (Bench 902); GOAT holds at 8-bit, 6-bit demoted from default candidacy. The `ForwardContext.logit_floor` default flip is the owner act (non-goals below); everything it needs is on record. Feature stays opt-in until that flip.
**Primitive:** katgpt-core `row_logit_floor` (Bench 888, Issue 882 P2) — sink-exempt floor `l̃ = max(l, m_r − w)` + symmetric b-bit code + `2^b`-entry exp-table softmax; fused `floored_coded_exp_inplace`.
**Consumer evidence:** riir-infer [Bench 003](../riir-infer/.benchmarks/003_row_logit_floor_ppl_needle.md) — the model-bound G1, now COMPLETE.
**Lane record:** katgpt-rs [Bench 902](.benchmarks/902_row_logit_floor_promotion.md) — the four boxes below, all PASS.

## What closed the gate (riir-infer Bench 003, final standing)

- **T2** (gemma-2-2b-it f16, 4096 tok): PASS at 8-bit (+0.033% ppl / 0.17% flips) and 6-bit (+0.066% / 0.90%). b4 flips 4.32% — 6-bit is the admissibility floor.
- **T3** (needle, long context): PASS at 64K for every arm — T3a gemma 64K-width proxy (6/6, 0 flips), T3b MiniCPM5 16K (3/3, 0 flips, floored 8.9%), **T3c MiniCPM5 64K (3/3, 0 flips, floored 8.4% — the `ln(n/ε)` width saturated the floor term; the envelope held through a 4× dilution)**.
- **T4** (sink exemption): load-bearing — without it the floor floors 1.58× the keys, |ΔNLL| 1.35×, flips 1.49×, while aggregate ppl reads *better* (sign cancellation). The shipped design includes the exemption; a ppl-only gate would have promoted the defect.

## What promotion still requires (this repo owns each)

- [x] **Per-family retention walk** — DONE (Bench 902): riir-infer `row_logit_floor_ppl --families true` (`ae3cd1e`/`d4b39c5`), gemma-2 4K × 4 chunks, arms base/b8/b6. Aggregates reproduce Bench 003 T2 exactly; per-chunk flips 0.10–0.20% (b8) / 0.59–1.17% (b6) — no family concentration; **every flip at both widths sits in the near-tie bucket [0, 0.5)** and the confident-flip class [8, ∞) is EMPTY (n=52, 0 flips). Prompt-family axis: Bench 003's T3a/b/c seq-exact 6/6 · 3/3 · 3/3 at every arm.
- [x] **Full-forward G2 paired A/B** — DONE (Bench 902): the whole decode step (per-call width policy + floor+code+LUT exp + normalize + P·V + envelope tally) vs plain, interleaved-paired, both widths, N=4096/16384, TWO reproducing runs: medians 0.989–1.000, all ≤ 1.01 — latency-neutral-to-positive at both widths; width is perf-irrelevant at the step level. G1-echo envelope cells + G4 alloc-free (0 allocs incl. tally) in the same bench (`bench_903_row_logit_floor_promotion_goat`).
- [x] **Width decision** — RECORDED (Bench 902): **8-bit is the width of record** (0.17% flips, all near-tie); **6-bit demoted from default candidacy** — admissible (T2/T3/family walk all pass) but 5.3× the flip rate for no measured perf return; stays selectable via `RowLogitFloorPolicy.bits = 6`. 4-bit out (Bench 003 T2).
- [x] **G3/G4** — DONE (Bench 902): `cargo test -p katgpt-core --features row_logit_floor --lib` 2075 passed / 0 failed; Bench 888's full GOAT re-run green on the same tree (its G2 head rows −0.02%/+0.65% ≤ 1.01); G4 0 allocations over full-step calls including the envelope tally.

## Non-goals

- The **default flip** (`ForwardContext.logit_floor: None` → the 8-bit policy) is the owner act: every precondition it names is now met and recorded (Bench 902), and the flip itself is one `Some(RowLogitFloorPolicy { n_sink: 4, bits: 8, tv: 1e-3, width_ctx: None })` in riir-infer — deliberately NOT made this session (riir-infer carries sibling WIP; a production-numerics default change belongs to its own owner-greenlit commit).
- The contingent BO arm of riir-infer 011's cousin (katgpt-rs 898) is unrelated and stays blocked on the BO-trained checkpoint.

Session: issue011-t3c-harvest
