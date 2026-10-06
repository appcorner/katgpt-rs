# Issue 920: HyperThink modelless delta-overlay PoC (defend-wrong, pre-registered against the §31 failure class)

**Status:** Open — PoC defined, not started
**Date:** 2026-10-06
**Research:** [katgpt-rs/.research/606_HyperThink_Text_to_Parameter_VQ_Bias_Amortization.md](../.research/606_HyperThink_Text_to_Parameter_VQ_Bias_Amortization.md)
**Source:** arXiv:2610.03039 (HyperThink, COLM 2026) — modelless track only
**Related:** negative_results §31 (factorized/VQ systematic failure class); R062 SHINE; riir-train Plan 445 (the trained arm); katgpt-pruners CNA (the starting substrate)

## Hypothesis

The paper's ablation ladder (identity 58.82 < **static delta 60.88** > per-instance-continuous 59.74 < VQ-routed 61.06, GSM8K) says discrete route-conditioned deltas generalize where continuous per-instance deltas overfit. §31's modelless VQ failure quantized **content** (action-effect patches) and died on the gate (L2-norm at parity) and transfer (k-means overfits source). This PoC tests whether quantizing the **conditioning signal** (which delta to apply, HyperThink's regime) escapes that failure — i.e., whether a modellessly-constructed, k-means-routed bias-delta table beats a single static delta on an OOD read.

**Where the additive overlay attaches:** the overlay is "add vector b_l to the OUTPUT of projection p in block l" — attachable at projection-output accumulation sites in the forward regardless of whether the checkpoint stores bias tensors (same math as a bias; zero weight mutation). For a GGUF carrying native bias tensors, those are used directly.

## Experimental spec (pre-registered before any run)

- **Target model:** first cell = `riir-train/data/gemma-2-2b-it-f16.gguf` (fp16, cheapest iteration cell; hook surface = projection-output accumulation sites via the riir-infer Issue-014 tap stack `forward_gemma2_f16_act_tapped`, the seam issue 919 already used for its gemma calibration cell). **Bonsai-27B PQ2_0 is NOT excluded for format reasons** — the overlay adds to projection OUTPUTS, so the weight format is irrelevant, and `TernaryMatvecHook` (riir-infer: `ternary_forward.rs`, `deltanet/profiling.rs`, `bin/act_diagonal_calibration.rs`; forward proven bit-identical-to-unhooked on Bonsai-27B by issue 919's calibration cell) is the proven seam. Bonsai is sequenced SECOND (T6) for iteration cost only and to keep the league lane unperturbed while the discriminator is unproven — it is the standing-priority confirmation cell, not an afterthought.
- **Probe set + prompt `c`:** a fixed 3-shot worked-exemplar prompt as the static "thinking register" `c`; probes = 2,000 queries sampled once, seed-pinned, never re-drawn.
- **Splits:** in-domain = GSM8K 500-query subset (seed-pinned); OOD = MATH-500-style held-out subject mix, 500-query subset (seed-pinned). Both frozen reads.
- **Power rule (pre-registered, McNemar discordant-pair form):** MDE_δ = (z_{α/2} + z_{power})·√(p_d / n), α = 0.05 two-sided, power 80% (z sum ≈ 2.80), n = 500 per read, p_d = the OBSERVED discordant-pair rate on the actual read. Planning values: δ ≈ 3.0 pts at p_d ≈ 5%; δ ≈ 4–5 pts at p_d 10–15%. Power is adjudicated AFTER p_d is observed — never from the planning value alone. A read whose measured delta sits below its observed-p_d MDE with CI crossing zero is **INCONCLUSIVE (underpowered)** — a distinct verdict from FAIL. Only a powered FAIL/FLAT may be recorded in `negative_results.md`. This is the Issue-825 rule: a falsification must not be written from an underpowered run.
- **Retry policy (pre-registered, no gate-shopping):** if G2 is INCONCLUSIVE, exactly ONE widening is allowed (n = 500 → 2,000, same frozen queries extended seed-pinned) and at most TWO embedding classes may ever be tried, from this named set: (a) hashed-bag unigram+bigram (the reflex `span_embed`-class default), (b) DFT-based `ModellessEmbedder` embedding. Every attempt is counted and reported in the verdict regardless of outcome — a pass on attempt 3 is reported as attempt-3-of-3, never as a clean pass.
- **Layer window:** ρ = ‖E[Δout]‖²/E[‖Δout‖²] capture-ratio ranking reproduces the late-block-concentration law on the target model BEFORE any table spend; if the law does not reproduce, the PoC stops and reports that instead.

## Tasks

- [ ] **T1** Delta-content construction — **start from the shipped substrate, do not build a parallel one:** extend CNA's contrastive-pair machinery (`katgpt-pruners/src/cna.rs` — circuits from with-prompt/without-prompt pairs, runtime modulation; GOAT `tests/bench_cna_steering_goat.rs`) from neuron-granularity modulation to **bias-space persistent deltas**: `Δb_window = E_probe[out_with_c − out_plain]` per projection window (forward passes only). If bias-space needs its own path (persistent per-position constant vs per-token modulation), record WHY in the PoC notes.
- [ ] **T2** Codebook construction: offline k-means (fixed seed, k-means++ init) over modelless query embeddings (hashed-bag class); K sweep bounded (too small underfits, too large re-enters the per-instance trap); per-cluster TF-IDF labels emitted as a static artifact beside the table.
- [ ] **T3** Serving lane: frozen table (BLAKE3-pinned), nearest-code lookup (zero-alloc, µs-class), unarmed = bit-identical decode; per-code kill switches; usage-entropy health monitor (H vs ln K + per-code count floor → demote-to-identity).
- [ ] **T4** Gates (ALL must pass or the modelless track closes) and report-only arms:
  - **GATES:**
    - **(G1) static-vs-identity, in-domain, paired** — a SANITY arm that **blocks only on a powered NEGATIVE** (the static floor must not be worse than identity). Its positive expectation (+2.06 pts planning) is BELOW the MDE, so a non-negative INCONCLUSIVE does not block — by design, since this arm exists to catch harm, not to prove lift.
    - **(G2) routed-vs-static on the OOD read**, paired, power adjudicated by the pre-registered McNemar rule at observed p_d — THE discriminator gate. Expected effect **UNKNOWN**: the paper's +5.4 OOD figure is trained-VQ vs continuous-no-VQ, NOT routed-vs-static (the paper reports no routed-vs-static OOD effect, and this arm is modelless k-means). G2 fails on a powered negative, or on a powered **FLAT** — FLAT is pre-registered as an EQUIVALENCE margin, not decided after the data: FLAT ⇔ the 95% CI upper bound on the routed−static delta is below **+2.0 pts**. An underpowered read is INCONCLUSIVE → widen n or try a pre-named alternative embedding class (retry policy below) BEFORE any verdict is written.
    - **(G3) unarmed decode bit-identical; (G4) route cost p99 µs-class, 0 alloc; (G5) usage entropy ≥ floor under adversarial single-cluster traffic with graceful identity fallback.**
  - **REPORT-ONLY (never in the pass set):**
    - **(R1) routed-vs-static in-domain** — planning expectation +0.18 pts, UNDERPOWERED-BY-DESIGN at n=500; reported as INCONCLUSIVE, never FAIL, never promoted on.
    - **(R2) static-vs-identity positive magnitude** — floor-arm sizing data for future cells.
- [ ] **T5** Verdict + negative-result hygiene: a powered FAIL/FLAT on G2 → record the negative here + in `.docs/09_feature_catalog/negative_results.md` (extending §31's class note with the conditioning-vs-content discriminator). An INCONCLUSIVE on G2 → widen n or change embedding class BEFORE any negative is written. PoC stays as permanent regression check if it passes.
- [ ] **T6** Bonsai-27B confirmation cell (runs after any G2 pass): re-run the winning arm through `TernaryMatvecHook` (the seam issue 919's Bonsai calibration cell proved bit-identical-to-unhooked) on the standing-priority league model — Bonsai gets its own measured confirmation, never an extrapolation from the gemma-2 cell.

## Discipline

- PoC harness lives in `riir-ai/crates/riir-poc/` per the defend-wrong convention (research skill §3.6: "Where the PoC lives: riir-ai/crates/riir-poc/"); the primitive surface (codebook routing + delta overlay) is katgpt-rs vocabulary, so the tracking issue lives HERE by that same documented convention — cross-repo split is the convention, not drift.
- Three arms minimum: identity / static / routed. `CARGO_TARGET_DIR=/tmp/...`, cleaned up after.
- This issue does NOT authorize any serving-lane wiring; consumers are owner-gated (R178 posture, A9 lane admission).
- The trained arm of the same mechanism is riir-train Plan 445 — this PoC failing does NOT kill 445 (per-track separation), and vice versa.
