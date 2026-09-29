# Issue 907 — KVarN V-row bit arms are not a pure bit ladder on real rows (riir-infer T1 finding; mining intake)

**Status:** RESOLVED 2026-09-29 — [Bench 903](../.benchmarks/903_kvarn_vrow_real_ladder_attribution.md) reproduced the inversion at the cache level on REAL captured rows (riir-infer `vrow_capture`, the Bench-011 model/stream posture) and ran the issue's step-2 crossed-config sweep: plain b2 24.1 < b4 28.8 < b3 132.5 rel-MSE (b3 cosine 0.9416 vs b2 0.9886 — the inversion is the quantizer's own, no model needed); the crossed arms are ALL worse than every plain arm (b3-on-b2-machinery ≈ 1351, b2-on-varn ≈ 828 — the machinery classes don't compose), which attributes by elimination: within the var-norm machinery, 3 bits cost 4.6× 4 bits, and NO config in the family makes 3-bit competitive → **the 3-bit var-norm arm itself is the defect site** (the scale-field handling at 8 code values); the fix, if b3 is ever wanted, is local to it. Consumer rule LANDED in `kvarn/mod.rs`: never interpolate V-row quality across bit arms — each arm is a distinct quantizer. Substrate: `KVarNKVCache::set_quant_mode` (feature `quant_mode_override`, measurement-only) with the load-bearing RTN-scratch resize (the naive override panics — `with_config` sizes the scratch for the derived group count). Fixture + bench BLAKE3-pinned; loud-skip without the fixture.

Evidence: riir-infer `.benchmarks/011_t1_fitted_v_p1_gate.md` + `011_run_report.md`/`011_run.log` (riir-infer commits `bfc923d` instrument, `facbab8` record). Modelless mining intake — a measured quantizer-behavior anomaly.

## The finding

On gemma-2-2b-it f16 natural-chat decode (26 layers, kv_dim 1024, tile 128, KVarN instantiation recorded in the protocol: hadamard OFF, var-norm ON at b > 2, skip-varn + grouped-4 RTN at b2), the V-row quantizer's bit arms measured **not monotone in bits** — both at the cache level and at the model level:

- **Top-1 flip rate (plain KVarN V quant, vs the f16 base):** b2 **1.04%** < b4 **2.63%** < b3 **5.72%** — b3 flips ~5.5× more than b2 and ~2.2× more than b4.
- **PPL (plain arms):** p-b3 read **−0.24% BELOW f16** (6.0761 vs 6.0907) while p-b2 (+0.083%) and p-b4 (+0.130%) read above.
- The mean-removed arms (P1 decorator) inherited the same non-monotone shape (mr-b3 +1.474% vs mr-b4 +0.621%).

## Why this matters (the mining-intake angle)

1. **The b2/b3/b4 arms are DIFFERENT QUANTIZERS, not one quantizer at three widths** — the `with_config` derivation switches the whole per-tile machinery (skip-varn + grouped-4 RTN at b2 vs per-row var-norm at b3/b4). Whatever makes b3 worst is plausibly the var-norm path at 3 bits (scale-field rounding, or the RTN step vs the var-norm rescale interacting), but **that is a hypothesis, not a measurement** — nothing in this issue asserts the mechanism.
2. **Any consumer that treats "bits" as a monotone quality dial on V rows is mis-served** — riir-infer's own T1 protocol assumed the ladder shape when pre-registering; the measured inversion is the recorded correction. The pattern-leaf docs and any GOAT gate that interpolates quality across bits should carry the same caveat until this is explained or fixed.
3. The **synthetic** calibration dashboards (Bench 895's 1−ρ law) read monotone — the inversion only shows on REAL rows under decode (the T1 instrument replays the real trajectory), which is the second instance of the synthetic-vs-real divergence class this lane has recorded (the first: Bench 011 Gate 1's absmax overshoot).

## Suggested next steps (owner-ordered, not started)

- Reproduce at the primitive level: a katgpt-kv bench replaying REAL V rows (the T1 captures are riir-infer-side) through KVarN at b2/b3/b4, decomposing the MSE by the same tile machinery the config derivation selects. If b3's var-norm path is the defect, the fix is local; if the inversion is real but benign (b3 wins elsewhere), the outcome is a documented caveat.
- Sweep the grouped-4/b2 vs var-norm/b3 config split explicitly (force b3 down the b2 machinery, and vice versa) to attribute the divergence to the machinery vs the width.
- Whichever way it resolves, the consumer-facing rule lands in the katgpt-kv docs: **never interpolate V-row quality across KVarN bit arms** — each arm is a distinct quantizer.

## Traps

- The T1 fixture carries the Bench-004 caveats (288-tensor no-qk-norm conversion) — a standard-conversion fixture may behave differently; reproduce before generalizing.
- The riir-infer run was single-corpus (chat_probe natural text); a per-family conditional walk (the Orthrus law) is unmeasured here.
- Box state of the evidence run: 4090 workstation, CPU lane, 13 h scheduled run, AC power — recorded in the bench doc.
