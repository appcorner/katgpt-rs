# Issue 923: escalation_guard — the shared serving-escalation guard primitive (Plan 202 R1)

**Status:** Open — R1 primitive half (katgpt-core). The consumer halves land separately: riir-refine's escalation manifest (Plan 202 R1 refine side, ships ALL-EMPTY) and the instinct/rethink migration onto the primitive (follow-up, not this issue's scope).
**Date:** 2026-10-07
**Plan:** [riir-refine/.plans/202_decision_stack_arsenal_alignment.md](../../riir-refine/.plans/202_decision_stack_arsenal_alignment.md) (R1, round-3 AGREE baseline)
**Prior art:** riir-rethink `src/escalating_backend.rs` (`EscRateGuard`, `kill_switch_decode`, `RATE_GUARD_WINDOW=200`, the `ESC:think|cheap|demoted` receipt tiers — Issue 017 T5's grammar-locked bounds); riir-instinct `src/arsenal.rs` `EscalateSpec` (`0 < min_rate < max_rate < 1` grammar, the manifest-side vocabulary).

## The one-definition problem (why katgpt-core)

Plan 202 R1: every escalation lane in the stack needs the same three runtime
pieces, and they currently exist as PRIVATE per-module copies spelled
differently in each repo — the substrate-first vocabulary-translation grep
found **three** existing spellings of the kill-switch decode alone:

| Piece | riir-rethink (public there) | katgpt-core private copies | riir-refine (needs it) |
|---|---|---|---|
| Rolling-rate latch (N-decision window, latched demotion) | `EscRateGuard` (ring, `observe`/`rate`/`latched`) | — none (verified: `rate_control` is a dual-EWLS effect-size CONTROLLER, a different concept) | Plan 202 R1 manifest's cost-ceiling guard |
| Exact-literal demote-only kill-switch decode | `kill_switch_decode` (`v != Some("0")`, absence-armed) | `tpr::parse_kill` (`matches!(v, Some("0"))` — same truth table, inverted polarity, private + OnceLock'd); `ugc_schedule`'s `UGC_DEBUG` ("the tpr::kill_switch pattern") | the manifest row's demote-only switch |
| Receipt tier (one spelling per serve outcome) | `TIER_THINK="ESC:think"`, `tier_cheap(arm)`, `tier_demoted(arm)` | — none | arm-H receipts (Plan 202 R6) |

A refine-side copy (the plan's original shape) would be a FOURTH spelling
breaking the one-definition law the plan itself cites. The extraction lands
ONCE here — katgpt-core is the lowest dep both sides share (refine's only
default-build dep; instinct depends on katgpt-core) — and consumers keep
their own grammar/validation/migration on top.

## Scope

Behind the opt-in `escalation_guard` feature (Feature Flag Discipline —
promotion to default needs its own GOAT; this is a fresh primitive with no
incumbent in this repo, so it ships with its own GOAT gate):

1. **`RollingRateLatch`** — the rolling-window rate latch: ring of N bools,
   `observe(escalated)` O(1) alloc-free, `rate() -> Option<f64>` (None until
   the window fills — a partial window is no measurement), latched demotion
   `rate_above_max` / `rate_below_min` (sticky, terminal in-process;
   re-arming is a manifest edit + re-construction, never an API call).
   `DEFAULT_WINDOW = 200` (the grammar-locked ESC window). TWO constructor
   shapes: both-bounds (`0 < min < max < 1`, rethink's lane) and
   **cost-ceiling-only** (`max_rate` alone — refine's shape, where the
   escalation rate is the L0–L3 miss rate: low is good, never a demotion
   trigger; `rate_below_min` cannot exist there).
2. **`demote_only_decode(v: Option<&str>) -> bool`** — the pure decode,
   absence-armed polarity (`v != Some("0")`): ONLY the exact literal `"0"`
   demotes; unset / `"1"` / `"true"` / junk all leave the lane armed (the
   manifest is the only ARMING surface, so the env may only DEMOTE). Plus
   `demote_only_env_armed(env_name)` — the uncached env read helper;
   construction-time only, never on the decide path. Callers cache
   (rethink reads once at construction; `tpr` OnceLock's — both compose).
3. **`receipt` formatters** — `tier_think(ns)`, `tier_cheap(ns, arm)`,
   `tier_demoted(ns, arm)`: the three receipt roles every escalation lane
   spells the same way, namespace-parameterized (`ns = "ESC"` reproduces
   rethink's bytes exactly — migration is a pure move, zero byte drift).
   Not hot-path (one String per served answer).
4. **`rate_bounds_valid(min, max)`** — the shared grammar predicate
   (`0 < min < max < 1`, finite) so instinct's `EscalateSpec::validate` and
   refine's manifest validation consume ONE definition of the bounds
   grammar (consumers keep their field-scoped refusal text).

### Non-goals (recorded, not silently skipped)

- **Migrating `tpr::parse_kill` / `ugc_schedule` onto the decode** — tpr is
  UNGATED default-on; consuming an opt-in feature from a default-on surface
  would either break default builds or force promoting this primitive
  prematurely. The copies are 3-line private fns with per-module caching;
  migration is a deliberate follow-up IF this primitive ever promotes.
  The truth tables are cross-pinned by tests on both sides instead.
- **The manifest grammar itself** — instinct's `EscalateSpec` fields
  (think_file/think_digest/margin/lcb_floor) are encoder-lane-specific and
  stay in instinct; refine's manifest rows are refine-side (Plan 202 R1's
  second half). Only the shared subset is here.
- **Any wiring into refine/reflex/instinct/rethink** — consumer landings
  are their own commits/repos (G3: this repo is upstream).

## GOAT

| Gate | Instrument | Bar |
|---|---|---|
| G1 | in-module `#[cfg(test)]` suite (pinned semantics: rate-None-until-full, latch reasons, stickiness, window roll, kill-switch truth table, receipt bytes, bounds grammar, constructor validation) | all pass, `--features escalation_guard --lib` |
| G2 | `bench_923_escalation_guard_goat` (bench_875 shape: best-of-3, black_box sink, absolute ceiling) | ns/observe under ceiling |
| G3 | default-feature surface unchanged (opt-in feature; default build compiles nothing of it) + clippy clean at both postures | `cargo check` (default) + `cargo clippy --features escalation_guard --all-targets` |
| G4 | alloc-free `observe` (post-construction) via the lib test binary's `TrackingAllocator` (`crate::alloc`) | 0 allocs / 1000 observes |

## Tasks

- [x] T1 `crates/katgpt-core/src/escalation_guard.rs` + feature + lib.rs gate + `[[bench]]` row (required-features)
- [x] T2 G1 test suite in-module (incl. the cross-repo byte pins: `ESC:` receipt bytes, `"0"` truth table) — 17 tests
- [x] T3 G2 bench + measured record `.benchmarks/923_escalation_guard_goat.md` — **1.74 ns/observe** (ceiling 20)
- [x] T4 G4 alloc-free observe test — 0 allocs / 1000 observes
- [x] T5 test_gate.sh row `katgpt-core:2158:escalation_guard` (2158 = 2141 default + 17) + README/examples/README feature count 673→674
- [x] T6 `docs_gate.sh` green 35/35 (the transient repo_set/cross-repo reds were the sibling session's `riir-ai.wprobe` probe dir appearing+disappearing mid-run — not this change)
- [x] T7 commit + push; note the landing hash in riir-refine Plan 202 R1 (the consumer-half handoff)
