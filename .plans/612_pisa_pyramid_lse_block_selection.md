# Plan 612: PISA — Pyramid Top-K + LSE Block Selection (opt-in `pyramid_topk`)
**Status:** Active — Phase 1 COMPLETE (T1.1–T1.4); Phase 2 not started

**Date:** 2026-09-29
**Research:** [katgpt-rs/.research/595_PISA_Pyramid_Sparse_Attention.md](../.research/595_PISA_Pyramid_Sparse_Attention.md)
**Source paper:** [arXiv:2609.31093](https://arxiv.org/abs/2609.31093) — Block Sparse Attention with Log-Linear Complexity (PISA), Tang et al., Sep 2026
**Target:** `crates/katgpt-attn/src/dash_attn/pyramid_topk.rs` (new module) + Cargo feature `pyramid_topk`
**Slot:** sparse-attention routing (contested — DashAttention + PFlash default-on; MSA R225 + HGA R379 negative)

---

## Goal

Ship the modelless PISA extraction — a mean-pooled coarse-to-fine key pyramid with LSE block scoring and root-seeded bounded expansion — behind the opt-in `pyramid_topk` feature, consuming kernels that already ship (`argtopk_with_scratch`, `simd::logsumexp_parts`, `simd_dot_f32`). GOAT gate: head-to-head against single-level selectors on REAL captured checkpoint tensors at iso-budget — per-family mass retention (never pooled, Issue-750-T3 lossy law) + log-log selection-latency slope + the assertable candidate bound. **If pyramid-LSE does not beat single-level LSE at iso-quality on real tensors, the result is a documented negative and the feature stays opt-in** (the MSA/HGA precedent).

**Binding constraint (HGA lesson):** the random-key NIAH harness is BANNED for this gate. HGA's G2-proxy FAIL (2/12) was measured on random keys; its own root-cause note says the dilution is a random-key artifact. All selection-quality claims replay REAL pretrained tensors.

## Phase 1 — Primitive (CORE)

### Tasks

- [x] **T1.1** `PyramidKeyHierarchy` — build from per-KV-head `[N, d]` keys: level-1 leaf means over C=64 blocks, then g=2 pairwise pooling to ⌈log₂(N/C)⌉ levels. Storage ≤ 2N/C·d f32. Zero-alloc build API (caller-provided scratch). Unit: level-j node == exact subtree mean (integral-image identity, bit-tolerance pin). **DONE** — pooling is weighted by covered TOKEN counts (`min(2^(t-1)·C, N − c·2^(t-1)·C)`), which is what keeps the exact-subtree-mean identity at the ragged tail (leaf-count weights would misweight a short last leaf); storage unit-pinned ≤ 2·(⌈N/C⌉+1)·d.
- [x] **T1.2** `coarse_to_fine_select` — root-seeded loop ℓ = L→1: score ≤ gK candidates via `simd_dot_f32` child logits + LSE (`logsumexp_parts` for leaves over ≤C logits; inline 2-element LSE for internal levels); `argtopk_with_scratch` per level; expand children of retained blocks. Forced-block union (first/previous/current leaf + ancestor paths). GQA group-sum (u = Σ over query heads sharing the KV head) before Top-K. **Assert per call: scored candidates ≤ 1 + gK·⌈log₂(N/C)⌉.** Taylor-order knob: `Mean` / `MeanPlusHalfVar` / `ExactLse` (sweep at the gate; the paper predicts exact wins on recall). **DONE, with one measured refinement — the constant-keys canary caught a walk-death defect:** if a level's every candidate is forced (shallow level, constant keys), a Top-K-only expansion set empties and the walk scores NOTHING below it. Fix: forced ancestors EXPAND (their non-forced descendants stay Top-K-eligible) while still never being scored — which widens the honest per-call bound to `1 + (gK + g·FORCED)·⌈log₂(n_leaves)⌉` (the paper's pure no-forced form `1 + gK·⌈log₂(N/C)⌉` is the special case; same O(gK log N) shape; asserted per call on the honest form). Forced leaves join the leaf output directly, deduped, ascending. API: `PyramidScorer { mode, scale }` + output lands in `scratch.out` (both fns exactly 7 args, no `too_many_arguments` allow).
- [x] **T1.3** `PyramidDecodeCache` — per-KV-head pyramid with rank-1 leaf update `(c·k̄+k)/(c+1)` + ancestor-path recompute on append; level capacities grow by a fixed fraction (amortized O(1) expansions per doubling, O((N/C)d) steady state). Unit: path-update == full recompute (ulp tolerance). **DONE** — same token-count weighting as the static build (both reduce to the exact subtree mean); path==rebuild pinned at 1e-4 across checkpoints spanning every depth-growth boundary (n=1..3000, d=8); the cache drives the SAME selection core via the shared `PyramidLevels` trait.
- [x] **T1.4** Feature wiring — `pyramid_topk` feature row in `crates/katgpt-attn/Cargo.toml`; module + re-exports from `dash_attn/mod.rs` (verify the `dash_attn` gating shape at implementation and keep `pyramid_topk` independently selectable); N ≤ C degenerates to single-level behavior (envelope assert). README sparse-slot table row (opt-in). **DONE** — `pyramid_topk = ["dash_attn"]` (the `vortex_flow` sub-feature pattern: implies only the parent, not the VortexFlow/MSA cluster); gating shape verified (`#[cfg(feature = "pyramid_topk")]` inside the `dash_attn`-gated module tree); N ≤ C envelope assert + empty-pyramid no-op test; README Opt-In table row + `.docs/09_feature_catalog` row + the five 661→662 flag-count claim sites. Validated: tests 8/8, clippy `-D warnings` at default AND `--features pyramid_topk` AND `--all-features` AND wasm32, docs_gate 35/35.

## Phase 2 — GOAT gate (`tests/bench_612_pyramid_goat.rs`)

### Tasks

- [ ] **T2.1** Real-tensor replay fixture — capture per-layer Q/K from the FULL-ATTENTION layers of a real pretrained hybrid at 4K/16K/32K/64K. Primary: **Bonsai-27B PQ2** (the league-preferred model; GDN+FA hybrid — FA layers only, interval read from its config at capture time; secondary qwen3.8-27B whose 48 GDN + 16 FA interval-4 layout is pinned in `riir-infer-gpu/src/qwen38_dense_cudarc.rs`). Capture on the 4090 via the GPU-resident forward lanes (riir-infer-gpu `qwen38_dense_cudarc` whole-model composition, riir-ai riir-gpu prefill FA lane — Issue 742 T9.2 lineage; lineage: riir-ai Issue 599, closed at `2dd461085`). **Kimi-K3 0.4B is DEMOTED to canaries + the G2 latency-slope measurement only** — a test-arch model carries no selection-quality claim. Fixture size: commit a BLAKE3 manifest + a SAMPLED subset (e.g. 2 layers × 2 heads per length, f32 bins); full captures stay in gitignored storage with provenance (path, sha, capture command, box state). **No synthetic random-key fixtures.**
- [ ] **T2.2** **G1 selection quality** — selectors: single-level mean-dot (BSA-class), single-level exact-LSE, pyramid-LSE. Metrics vs full-attention reference (paper D.2 template): Recall@K + attention-mass ratio, reported PER FAMILY (per prompt × position × layer × head), never pooled (Issue-750-T3). Jensen pins, SPLIT by provability: (a) per-candidate SCORE ordering is a theorem and is ASSERTED per fixture (normalized mean score ≤ LSE score for the same candidate block); (b) selection-OUTCOME captured mass: the UPPER direction is assertable per fixture (exact-mass Top-K over all blocks maximizes captured mass ⇒ pyramid captured ≤ leaf-exact captured), the LOWER direction (mean-dot vs pyramid-LSE) is NOT a theorem — a mean selector can out-capture on a given fixture by chance — so it is a MEASURED per-family aggregate ordering, never a per-fixture assert. Recall ordering is likewise empirical and pinned by replay.
- [ ] **T2.3** **G2 (LOAD-BEARING) quality-vs-complexity head-to-head** — log-log selection-latency slope: pyramid ≈ 1 vs single-level ≈ 2; crossover N\* measured + pinned as a calibration constant; **iso-quality at 32K+: pyramid-LSE ≥ single-level-LSE on Recall@K within noise.** Fail ⇒ negative result documented in `.benchmarks/612_pyramid_goat.md` (MSA/HGA format), feature stays opt-in, no promotion.
- [ ] **T2.4** **G4** — zero-alloc selection (scratch reuse, the `argtopk_with_scratch` pattern); allocation count asserted on the gate path.
- [ ] **T2.5** Canary — degenerate sequence (constant keys): forced policy retains the current-token block and the selector does not collapse; N ≤ C degenerates to single-level (byte-identical index set).

## Phase 3 — Verdict + docs

### Tasks

- [ ] **T3.1** Promote/demote decision per G2: pass ⇒ promote-to-default only if a long-context consumer exists, else keep opt-in with the slot-ledger row; fail ⇒ negative-result doc + README row update. No default-on promotion without the head-to-head win AND a consumer.
- [ ] **T3.2** Note + HISTORY cross-references; T\* calibration documented for `meta_router` dispatch (opt-in until armed).
- [ ] **T3.3** Cross-repo pointers recorded: riir-ai Issue 1017 (limelight Wave-2 zone-salience LSE pyramid), riir-train Issue 586 (training proof lane). DenseEmbedIndex/AnyRAG follow-ups remain note-level (Research 595 §2.2 discards).

## Constraints

- Contested slot: no default-on promotion without the G2 win + consumer; loser discipline per the §1.6 ledger.
- Never quote the paper's 90.95%/99.46% as our floor — MGATE is re-measured on captured tensors (Research 595 §3).
- All quality metrics per-family, never pooled (the lossy-surface promotion rule).
- Overflowing note vocabulary: `pyramid`, `coarse_to_fine`, `PyramidKeyHierarchy` — grep-vocabulary for the substrate-first check at implementation time is `hierarch|level|pool|argtopk|logsumexp`.
