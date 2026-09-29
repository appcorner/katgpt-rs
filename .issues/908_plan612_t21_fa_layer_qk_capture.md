# Issue 908 — Plan 612 T2.1: FA-layer Q/K capture lane (4090 long task)

**Status:** OPEN — unblocks Plan 612 Phase 2 (`goat gate needs REAL tensors; the random-key NIAH harness is BANNED — the HGA lesson, Research 595 §2.1)

**Plan:** [`.plans/612_pisa_pyramid_lse_block_selection.md`](../.plans/612_pisa_pyramid_lse_block_selection.md) T2.1
**Executor:** the 4090 session (this is a CUDA/cudarc task; the M3 box is load-disciplined and the capture is GPU-resident)

## What exists (recon done 2026-09-29, M3)

- `riir-infer/crates/riir-infer-gpu/src/qwen38_dense_cudarc.rs` — the whole-model
  qwen3.8-27B cudarc composition. Layout pinned: **48 GDN + 16 FA, interval-4**
  (file header). It has `forward_token_capture` (line ~4461) but that taps the
  **post-MLP residual stream** — NOT attention Q/K. T2.1 needs the attention
  tensors themselves.
- No capture lane exists for Bonsai arch in riir-infer-gpu.
- Model files (M3 + 4090 both carry them; VERIFY at execution):
  - PRIMARY `riir-train/data/Ternary-Bonsai-2-27B-PQ2_0.gguf` (the
    league-preferred model; GDN+FA hybrid — FA layers only, interval read from
    its GGUF config at capture time)
  - SECONDARY the qwen3.8-27B GGUF (its lane exists; 16 FA layers interval-4)

## What to build

1. **riir-infer side (the capture bin):** a new `riir-infer-gpu` example/bin
   (`qwen38_pyramid_capture`) that runs the dense lane forward over a prompt of
   length L and, inside the FA attention arm, downloads per-layer **K rows (per
   KV head, f32)** for ALL positions and **Q rows (per query head, f32)** for a
   SAMPLED position set. Reuse the `forward_token_capture` dtoh discipline (one
   sync per tap, diagnostic-only). GQA head grouping follows the config.
2. **Capture matrix:** L ∈ {4096, 16384, 32768, 65536}; per length: 2 layers ×
   2 heads (Q heads + their KV group) MINIMUM, sampled positions spread across
   the sequence (first / quartiles / current-decode positions — the forced
   policy needs a real `query_pos` per case).
3. **Fixture contract (lands in katgpt-rs):**
   `crates/katgpt-attn/tests/fixtures/pyramid_612/` — the SAMPLED subset only,
   f32 bins, one BLAKE3 `manifest.json` (per-file sha, model sha, layer/head/
   position provenance, L, capture command, box state: GPU name + driver +
   exclusivity note per the riir-ai GPU-exclusivity rule). Full captures stay in
   gitignored storage (`/tmp` or SDXC) with the manifest recording their path+sha.
4. **Provenance discipline:** run `scripts/bench_preflight.sh`-style box-state
   notes are an M3 lane — on the 4090 record GPU name, VRAM free, and whether any
   compute consumer was co-resident. Never quote the paper's 90.95%/99.46% —
   MGATE is re-measured on these tensors (Research 595 §3).

## Constraints

- Sync riir-infer + katgpt-rs + riir-train on BOTH boxes first (the both-boxes
  sync rule; prevent diverge).
- The capture bin is diagnostic tooling in riir-infer (inference-substrate
  diagnostics — inside its boundary); the FIXTURES + the gate belong to
  katgpt-rs Plan 612. If the riir-infer session prefers its own issue there,
  mirror it and cross-ref.
- Q/K dtype: dump f32 (post-scale, PRE-softmax logits are re-derivable from
  Q·K — the gate replays selection, not softmax).
- **No synthetic random-key fixtures** — real pretrained tensors only.

## Acceptance

- [ ] Capture bin lands in riir-infer-gpu (commit + sha cross-referenced here)
- [ ] Fixtures + BLAKE3 manifest land in katgpt-rs
- [ ] At least the SECONDARY model captured at all 4 lengths (PRIMARY attempted;
      a Bonsai-arch loader gap is a documented blocker, not a silent skip)
- [ ] Plan 612 T2.1 checkbox ticked with the fixture commit sha
