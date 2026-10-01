# Issue 914: Pseudo-Head Mixing Runtime — deterministic cross-head interaction op (IHA distill)

**Status:** OPEN — **classified TRAINING-DEPENDENT (Feature-Flag step 5: opt-in, riir-train dependency, no promotion to default)**. BLOCKED-ON [riir-train Issue 606](../../riir-train/.issues/606_iha_relation_composition_micro_rung.md) arms 4+5 evidence. Filed 2026-10-01 from [Research 600](../.research/600_Interleaved_Head_Attention.md) (arXiv:2602.21371); revised at verdict round 1.

> **Source:** the paper's own THEOREMS are deterministic constructions — only its benchmarks need training. This issue ships the constructions as an OPT-IN op; the learned variant stays in the paper. Per Thm 2, mixing changes the function class: a model must be trained with (or tolerant of) the mixing for the op to be useful — HLA was checked and does NOT qualify as a fixed-projection consumer (`riir-ai/crates/riir-engine/src/hla/forward.rs` consumes learned-style `attn_wq/wk/wo`).

## What

A feature-gated (`pseudo_head_mix`, default-off, **opt-in ceiling forever absent new evidence**) cross-head mixing op for multi-head latent states, host `crates/katgpt-attn/src/` (beside `funcattn_compose/`):

1. `apply_pseudo_mix(q, k, v, α_q, α_k, α_v) -> interleaved [H, N·P, d]` — the einsum `mhp,nmd→hpnd` + `merge_pseudo` layout (no allocation in the hot path; scratch-reuse, G4).
2. `collapse_pseudo(o, r) -> [H, N, d]`.
3. Deterministic α constructors:
   - `replication()` — α = 1(m=i) for every pseudo j + R selecting block 1; the superset-theorem INCLUSION witness (equals MHA under exact arithmetic in the algebraic formulation — the paper's one-hot ROUTERS are cross-head mixing, NOT this).
   - `sign_flip_pair()` — ±1 pseudo-pairs; the Thm-2 separation witness (difference-of-softmax nonlinearity on repeated-token inputs where MHA is linear). Note: the softmax domain grows P× — this is an expressivity witness, not an identity.
   - `hadamard(h)` — DENSE ±1 Hadamard (Sylvester for power-of-two H, Paley otherwise; not a signed permutation), `α[m,h,j] = Had[(h+j) mod H][m] · (1/√H)` — orthogonal over the right axes; the 1/√H normalization preserves the q·k scale (without it scores inflate ~H×). `bonsai2_hadamard` (riir-infer) is the shipped dense-Hadamard precedent.

## Gates (GOAT, feature-flagged, opt-in)

- **G1**: replication ≈ plain MHA at f32 TOLERANCE in the algebraic (no-interleave, no-RoPE) formulation — a mismatch is a construction-premise investigation, not auto-harness-bug; sign-flip nonlinearity test (repeated-token subspace: mixing output nonlinear where per-head MHA is linear — Thm B.1 as a unit test); Hadamard orthogonality `αᵀα = I` over the (m × (h,j)) axes + scale-preservation test.
- **G2**: mixing cost vs plain forward (SIMD; H²P MACs/token — expected noise-level at small H).
- **G3**: feature-off bit-identical (bit-identity is reserved for THIS posture — the only place it is achievable).
- **G4**: zero-alloc scratch reuse across heads.

## Consumer wiring (why this is not decorative — and its honest limit)

`katgpt-core::causal_head_importance` (Research 362) ships a causal head-necessity scorer whose own docs say it is "ready for any future head-mixing runtime (currently unused — Plan 182 is layer-wise)". This op IS that runtime shape: the scorer ranks WHICH heads matter for a capability; the mixing constructor then composes exactly those heads' Q/K/V. HLA (`katgpt-hla`) and the katgpt-attn variants are integration surfaces — all of which carry trained/learned projections, hence the step-5 classification above.

## GQA caveat (for any future consumer wiring)

The H×H mixing tensor presumes H query heads = H key/value heads. riir-ai's HLA forward projects K/V at `kvd` (grouped-query width — fewer KV heads than query heads); a grouped-query consumer needs the mixing reshaped (mix query heads only, or expand KV) before this op applies. One-line check at wiring time.

## Block condition

Do NOT implement before Issue 606's verdicts: arm 4 (trained-Hadamard) tells whether a fixed layout helps training; arm 5 (post-hoc insertion on trained MHA, incl. the random-orthogonal control) tells whether inference-time insertion is even benign. If arm 5 shows catastrophic degradation and no trained-with-mixing consumer exists in our stack, this issue CLOSES negative (constructions remain recorded in Research 600). Substrate-first grep: DONE in Research 600 §4 (heads are isolated everywhere; no cross-head mixing ships).

## Tasks

- [ ] T1 BLOCKED: await Issue 606 arm-4 + arm-5 verdicts (promote/demote/close this issue in the same commit)
- [ ] T2 `pseudo_head_mix` feature + the three α constructors + layout ops
- [ ] T3 G1–G4 gates + bench (`benches/`, feature-gated)
- [ ] T4 consumer example: causal_head_importance ranking → mixing selection
