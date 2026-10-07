# Plan 620: katgpt-assign — Spec-Driven Constrained Assignment Solver (Rebalancer distillation)

> **Status:** Active — Phase 1 not started (filed 2026-10-07, post Claude-verdict AGREE; Research 607)
> **Source:** Research 607 (OSDI'24 Rebalancer, facebook/rebalancer @ e4c35517980d893849275f71b3cb90f81164b6d5)
> **Consumers:** riir-chain Issue 164 (primary — shard_assignment solver); riir-rethink Issue 029 (candidate)

## Phase 1 — the crate + core solver (this plan's whole scope)

New leaf member crate **`katgpt-assign`** — std-only, alloc allowed, **zero external deps, zero workspace deps** (sits upstream of `katgpt-core`; core takes it as an optional path dep and re-exports `katgpt_core::assign` behind the opt-in `assignment` feature — the `katgpt-dec` → `katgpt_core::dec` precedent). Run the `boundary-guard` skill BEFORE the crate lands. Integer determinism is a G1 requirement from the first commit — the API never carries float accumulation, so it can't be retrofitted.

- [ ] Run `boundary-guard` (allowlist + domain read for a new member crate; confirm zero-dep posture)
- [ ] Scaffold `crates/katgpt-assign` (workspace member; `types.rs` for decoupled structs per repo convention)
- [ ] Types: `Problem` (objects/containers as dense `u32` indices; `Dimension` values `i64`; flat scopes; `Group` reserved for Phase 2), `Assignment` (dense `Vec<u32>` object→container), `Seed` (u64)
- [ ] Expression DAG: `Lookup` leaves (shared per (dimension, container)) + `Sum` + `Max` internal nodes; per-node cached `i64` value; leaf-affectance maps `M_o` (object→leaves), `M_b` (container→leaves); evaluate-const / apply-mutate split
- [ ] Specs (constraint-or-goal duality; broken constraint → fix-it goal + never-worse guard, paper defaults 100/10000): `Capacity` (scope-item util ≤ limit) · `Balance` (minimize util spread across scope items) · `MinimizeMovement` (penalize moved objects vs initial)
- [ ] Fold all per-container constraint rows into one root via `Max` (linear graph size)
- [ ] Local search: strict improvement (`obj_δ ≤ 0` signed integer folding violation + objective delta), **hot-container ordering** from node potentials (the paper's measured win — keep), move types `Single` + `Swap`, deterministic seeded tie-breaks, time + move limits
- [ ] Determinism test (G1): same input + same seed ⇒ byte-identical `Assignment` across runs AND across `Vec` iteration-order perturbations; no HashMap probe-order dependence anywhere in move generation
- [ ] G1 correctness fixtures: feasibility under Capacity; objective ≤ initial on every fixture; improvement-vs-greedy-init sanity; constraint-violation fallback behavior
- [ ] G2 outside-baseline bench (`--release`, box-state provenance line per the latency-claim rule):
  - [ ] vs greedy / first-fit-decreasing on the same instances (quality + wall time) — workspace precedent `riir-rag/src/packer.rs`
  - [ ] optimality gap vs brute force, n ≤ 12 (exhaustive)
  - [ ] mid-size lower-bound gap, n ≈ 100–1000 (capacity-relaxation bound, or Hungarian on the pure-assignment subset)
  - [ ] NO-GO clause: greedy ≈ local search at target sizes (≤ ~50k objects) within tolerance + latency ⇒ record NEGATIVE, primitive stays opt-in forever / dropped; delta-vs-full-recompute recorded as diagnostic only, never a gate
- [ ] G4: hot loop alloc-free under the repo's counting allocator (`debug_assertions`-gated per Issue 856)
- [ ] `cargo clippy` clean (healer-first for mechanical findings); feature-gated re-export compiles at BOTH postures (`assignment` on/off)
- [ ] README block in the crate + one line in the workspace members list

## Phase 2 — only after G1/G2 pass vs outside baselines

- [ ] Specs: `GroupCount` (spread), `AvoidMoving`, `AssignmentAffinities`
- [ ] Moves: triple-loop, KL-search; sampled/greedy variants
- [ ] Equivalence-class collapsing (paper Alg 2)
- [ ] Parallel candidate evaluation (rayon is core-external — keep katgpt-assign dep-free; thread via core's existing pattern or leave single-threaded)
- [ ] `explain` surface: why-was-this-move-rejected + assignment diff (the Explorer role)
- [ ] Constraint-policy variants beyond DEFAULT

## Phase 3 — consumer wiring (each is its own consumer-repo issue, not this plan)

- [ ] riir-chain Issue 164: shard→node assignment for `shard_assignment.rs` (solver-run location choice recorded there: leader-computes-commits-raw leaning; byte-equality test REQUIRED before any consensus-replayable use)
- [ ] Game runtime follow-ups per Research 607 §3 (spawn placement, migration balancing) — respect brain+FSM law
- [ ] riir-rethink Issue 029 candidate tracker: revisit only if escalate rows return to serve

## GOAT gate verdict protocol

Feature stays `assignment` opt-in. Promotion to default requires: G1+G2 pass vs outside baselines AND a real consumer wired (riir-chain Issue 164 minimum). Otherwise demote/keep opt-in and record the verdict here. NO-GO outcome ⇒ NEGATIVE record in Research 607 + this plan closed.
