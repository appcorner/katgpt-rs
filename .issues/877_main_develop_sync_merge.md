# 877 — main↔develop sync: merge origin/main into develop, conflict-by-conflict adjudication

**Status:** OPEN — plan filed, Claude verdict pending. Merge origin/main (`5e2b730f2`) into develop; resolution target: **ours (develop) for every conflict**, yielding a merge whose tree is **byte-identical to develop HEAD** — pure ancestry sync, zero content delta.

## Situation

main is NOT an ancestor of develop. Merge base `37bb9cbf8`. Three main-only commits:

| sha | subject | relationship to develop |
|---|---|---|
| `569daf98e` | feat: lthash primitive (Issue 807) | TWIN of develop `8da938961` (+ develop's lint fix `b85bda6e6` on top) |
| `3c844aebb` | fix: complete the lthash wiring (569 committed only auto-staged files) | TWIN of develop's wiring (already inside `8da938961`/evolved) |
| `5e2b730f2` | feat: exact_sigmoid/exact_sigmoid_f64/dot_f32_ordered (Bench 844) | TWIN of develop `5458dd69b` + evolved by `da89c386b`, `a36895e32` (861), `9b09783d9` (870) |

`git cherry develop origin/main` → all three `+` (semantically twins, not patch-identical).
main's own HISTORY record says it outright: *"Cherry-picked onto main from develop `8da93896`"*.
`git merge-tree --write-tree` → **11 conflicted files**, 2 clean auto-merges, 5 byte-identical adds.

## Conflict-by-conflict adjudication (develop = evolved canonical, wins every row)

| # | File | main side | develop side | Verdict |
|---|---|---|---|---|
| 1 | `.benchmarks/.highwater` | 844 | 883 | ours |
| 2 | `.benchmarks/844_exact_sigmoid_ordered_dot_substrate.md` | original doc | + Issue-861 follow-up section | ours |
| 3 | `.docs/09_feature_catalog/opt_in_features.md` | `## 106` lthash, pre-consumer wording | `## 112` lthash, evolved (Agave-mined consumers) | ours |
| 4 | `.issues/.highwater` | 807 | 876 | ours |
| 5 | `HISTORY.md` | +15-line mainline-transplant narration | canonical Issue-807 record (line ~3004) + full later history | ours (transplant narration lives in git history via the merge commit) |
| 6 | `README.md` | 594→595 feature-count bumps | 641/204 evolved counts | ours (main's bump subsumed) |
| 7 | `crates/katgpt-core/Cargo.toml` | `lthash = []` + bench_844 + bench_lthash rows | same rows at evolved positions | ours |
| 8 | `crates/katgpt-core/src/lib.rs` | `exact_sigmoid`/`_f64` fns + `#[cfg(feature="lthash")] pub mod lthash` | same at evolved positions (7/3 mentions) | ours |
| 9 | `crates/katgpt-core/src/lthash.rs` (add/add) | original | + let-chains lint fix (`b85bda6e6`) | ours |
| 10 | `crates/katgpt-types/src/simd/dot.rs` | len-4 anti-dedup pin | len-16 pin (x86_64 execution-matrix fix, 2026-09-21) | ours |
| 11 | `examples/README.md` | 594→595 count | evolved count | ours |

Clean auto-merges, verified equal to develop in the merge-tree result (`36505546ff`):
`crates/katgpt-types/src/simd/activations.rs` (develop carries the avx2_exp_sum n-clamp,
riir-train Issue 549) · `simd/mod.rs` (develop carries plasma_dispatch + bitcos).
Byte-identical adds (no action): `bench_lthash.rs`, `bench_844_exact_sigmoid_ordered_dot.rs`,
`.benchmarks/771_lthash_goat.md`.

**Duplicate-symbol trap checked:** the one real risk (both sides adding `exact_sigmoid` to
`activations.rs` at different hunks → textual auto-merge with two definitions) is REFUTED —
the merge-tree result has exactly one `exact_sigmoid`/`exact_sigmoid_f64`, at develop's lines.

## Execution plan

- [ ] File this issue + bump `.issues/.highwater` to 877 (dual_allocation_gate: 0 collisions, run)
- [ ] Claude verdict on the resolution + push posture
- [ ] `git merge --no-ff --no-commit origin/main` on develop
- [ ] `git checkout --ours` all 11 conflicted files, `git add`
- [ ] Validate A: `git diff develop HEAD --stat` **EMPTY** (byte-identity proof)
- [ ] Validate B: `git merge-base --is-ancestor origin/main HEAD`
- [ ] Validate C: scoped `cargo check -p katgpt-core --lib` (isolated target dir) — belt-and-suspenders; tree is byte-identical to the green origin/develop HEAD, so this is expected to pass trivially
- [ ] Merge commit (references this issue, Session marker in body)
- [ ] Fast-forward local `main` (279 stale) to the merge commit
- [ ] ONE push: `git push origin develop main` (main CI fires once, on the final state — owner-accepted)
- [ ] Close: HISTORY.md row citing the merge sha, remove this file, `docs:` commit (same push or follow-up push of develop only)

## Risk notes

- Sibling agents own this repo: develop was clean and == origin/develop at plan time; re-check `git status -sb` before the merge and before the push.
- Highwaters: ours (876/883) keeps the monotonic ledgers; main's 807/844 bumps are already consumed in develop's numbering history (Bench 844 exists on develop, Issue 807 closed at line 3004 of HISTORY).
- No content is lost: every main-side hunk exists on develop in evolved form (table above); the only main-unique text (the transplant narration) is preserved permanently in the merge commit's second parent.
