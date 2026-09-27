# Issue 903: `row_logit_floor` promotion lane — quality side UNBLOCKED by riir-infer Bench 003 (T3c PASS at 64K)

**Status:** Open — filed 2026-09-27 from riir-infer Issue 011's closure (its gate conditions "T2 + T3 passing" are now met; the promotion lane itself is this repo's). Feature stays opt-in until this closes.
**Primitive:** katgpt-core `row_logit_floor` (Bench 888, Issue 882 P2) — sink-exempt floor `l̃ = max(l, m_r − w)` + symmetric b-bit code + `2^b`-entry exp-table softmax; fused `floored_coded_exp_inplace`.
**Consumer evidence:** riir-infer [Bench 003](../riir-infer/.benchmarks/003_row_logit_floor_ppl_needle.md) — the model-bound G1, now COMPLETE.

## What closed the gate (riir-infer Bench 003, final standing)

- **T2** (gemma-2-2b-it f16, 4096 tok): PASS at 8-bit (+0.033% ppl / 0.17% flips) and 6-bit (+0.066% / 0.90%). b4 flips 4.32% — 6-bit is the admissibility floor.
- **T3** (needle, long context): PASS at 64K for every arm — T3a gemma 64K-width proxy (6/6, 0 flips), T3b MiniCPM5 16K (3/3, 0 flips, floored 8.9%), **T3c MiniCPM5 64K (3/3, 0 flips, floored 8.4% — the `ln(n/ε)` width saturated the floor term; the envelope held through a 4× dilution)**.
- **T4** (sink exemption): load-bearing — without it the floor floors 1.58× the keys, |ΔNLL| 1.35×, flips 1.49×, while aggregate ppl reads *better* (sign cancellation). The shipped design includes the exemption; a ppl-only gate would have promoted the defect.

## What promotion still requires (this repo owns each)

- [ ] **Per-family retention walk** — the lossy-surface promotion rule (riir-ai Issue 750 T3; Orthrus confirm arXiv 2609.15504): aggregate flip rates (0.17% b8 / 0.90% b6 at 4K) can hide family-conditional behavior flips. Under BF16, Orthrus exact-matched only 43–45% of prompts while aggregates stayed flat. Run the per-family conditional walk on the coded softmax before any default-on claim.
- [ ] **Full-forward G2 paired A/B** — Bench 888 measured the codec in isolation (−14.8% vs the exp softmax at N=4096, M3 CPU). Promotion needs the whole decode step (floor + code + path) vs the plain path, interleaved-paired (the `tests/common/ab_timing.rs` protocol), with box-state PROVENANCE.
- [ ] **Width decision** — 8-bit (conservative default) vs 6-bit (admissible per T2/T3). Record the choice with its flip table; demote the loser.
- [ ] **G3/G4** — no-regression across the existing looped/attention gates with the feature on, alloc floor on the probe path.

## Non-goals

- No change to `ForwardContext.logit_floor: None` default (bit-identical off) until every box above is checked and the GOAT holds.
- The contingent BO arm of riir-infer 011's cousin (katgpt-rs 898) is unrelated and stays blocked on the BO-trained checkpoint.

Session: issue011-t3c-harvest
