# Plan 616 — promote `minmax_partition` to katgpt-core as an open primitive

**Status:** ALLOCATED 2026-10-01 — from riir-infer Issue 022 T6.2 (the PoC's zero-training floor PASSED, narrowly: the plan files the promotion; the lane's measured window and its one open comparison debt are recorded in `../riir-infer/.benchmarks/022_t55_pareto.md`).

## Why now

The TWT PoC closed 2026-10-01 with the partition machinery battle-proven on
three fronts: it priced a real 64-layer ternary checkpoint end to end
(11 ε points, m(ε) machined-monotone, 8 measured known answers), it drove a
typed variant (`minmax_partition_typed`) whose tier-(i) rule became the
audition's structural gate, and it fed a validated collapsed checkpoint
(agreement 0.9486 at the 4.7% cut — the PoC's only passing point). The
primitive is generic — min-max partition of an ordered sequence under a
pairwise-distance oracle with ε-free (k-free) granularity — and Research
594's own table (row 2) pre-committed this promotion: *"open-primitive
promotion to katgpt-core after the PoC (single implementation home)"*.

## The DRY question the plan must settle FIRST (substrate-first, T0)

katgpt-core already ships a contiguous K-block DP:
`crates/katgpt-core/src/ugc_schedule.rs:692 dp_partition(profile, k)` —
sum-of-costs oracle, **fixed k**, min-SUM objective. The TWT DP is
**min-MAX worst-case, ε-constrained, k free**. Research 594's recorded
rule: *"generalize dp_partition or land beside it with a cross-link; two
partition DPs that don't know about each other is the DRY violation."*

- [ ] **T0.1** Read both DPs side by side; decide GENERALIZE (one
  implementation with an objective parameter) vs NEIGHBOR (two
  functions, one module, cross-linked docs + a shared type). The
  decision criterion: can `dp_partition`'s existing callers (UGC
  schedule) consume the generalized form byte-identically at their
  current call sites? If any behavior changes, NEIGHBOR wins — a
  promotion must not move a shipped consumer.
- [ ] **T0.2** Whatever the decision, ONE module home
  (`katgpt-core/src/partition.rs` proposed), one vocabulary, and the
  min-max DP keeps its riir-infer name and semantics verbatim —
  promote by MOVE, not copy (riir-infer `twt::partition` re-exports the
  katgpt-core items; its own copy dies in the same commit).

## Scope

- [ ] **T1** Move `minmax_partition` + `minmax_partition_typed` +
  `brute_force_optimal_typed` + the `SMatrix` pair-distance accumulator
  + `partition_worst` into the chosen home, `#[repr]`/allocation shape
  unchanged (the 64×64 artifact path must stay byte-deterministic —
  the emit known-answer pins depend on it).
- [ ] **T2** Feature flag `minmax_partition` (opt-in), katgpt-core
  default build unchanged; riir-infer forwards the feature
  (`twt_profile = ["katgpt-core/minmax_partition", …]` shape).
- [ ] **T3** Cross-repo determinism pin: the 11-point m table
  (61,57,49,35,25,14,8,5,3,2,1 over the bonsai profile artifact) becomes
  a katgpt-core-side known-answer test keyed on a committed S-matrix
  FIXTURE (the artifact itself stays gitignored in riir-infer — commit
  the 2016-entry upper triangle as a small fixture file, BLAKE3-pinned).
- [ ] **T4** Consumers inventory (the four-oracle reuse list from
  Research 594 §6.4): riir-infer twt (existing), ternary group-scale
  boundaries under an activation-range ε, KV-cache segment tiers,
  tick-series regime segmentation. Each future consumer files its own
  issue; this plan only guarantees the primitive + one home.
- [ ] **T5** Bench + GOAT gate: DP cost is O(L²·m) worst case —
  microbench at L ∈ {16, 64, 256, 1024} over a synthetic oracle;
  GOAT gates G1 (partition optimality vs brute force at n ≤ 16 —
  the property the brute_force twin already asserts), G2 (DP runtime
  floor/ceiling vs the L² model), G3 (no regression: the two riir-infer
  call sites re-run their existing gates unchanged), G4 (alloc shape:
  zero alloc per call at fixed L — the artifact path is
  allocation-counted today). Promote to default ONLY on a GOAT pass;
  the flag-off posture stays byte-identical.
- [ ] **T6** Docs: katgpt-core module doc + README feature-gate row +
  Research 594 row 2's promotion note flipped to SHIPPED (with commit).

## Non-goals

- No behavior change in the UGC schedule path (T0 decides, never edits
  its semantics).
- No new oracle implementations (consumers file their own).
- No promotion of the collapse WRITER/loader — that stays riir-infer
  (weights-domain substrate); this plan moves the pure partition
  primitive only.

## Provenance

- PoC record: `../riir-infer/.issues/022` (T5/T6), benches
  `022_t5_collapsed_goat_agreement.md` + `022_t55_pareto.md`.
- Research: `.research/594` §"MOAT gate" + §7 (the close-out), row 2 of
  the component table.
- Existing consumer: `../riir-infer/src/twt/partition.rs` (+ `s_matrix.rs`).
