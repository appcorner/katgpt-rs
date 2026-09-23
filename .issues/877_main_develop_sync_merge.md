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

**Not-a-loss note (verdict round 1 finding):** 10 files (`.issues/747_…`–`756_…`) exist on
main and not on develop — all present at merge-base, deleted on develop under the
noise-reduction rule (their records live in HISTORY.md + git history). The 3 main-only
commits touch nothing under `.issues/` but `.highwater`. Nothing is lost by keeping them
deleted; recorded here so a future main↔develop diff is not misread.

## Execution plan (amended per Claude verdict round 1 — REVISE adopted in full)

- [x] File this issue + bump `.issues/.highwater` to 877 (dual_allocation_gate: 0 collisions, run)
- [x] Claude verdict round 1 — REVISE: instrument changed to `-s ours`, validation changed to tree-hash equality, cargo check dropped, atomic push, 747–756 close-out row added. D1/D2/D3 otherwise confirmed.
- [ ] Amend issue commit with the revisions
- [ ] `git merge -s ours --no-ff origin/main` on develop — zero working-tree churn in the shared worktree; the ours-everywhere target is made true by construction (the per-file adjudication above was measured beforehand via `git merge-tree --write-tree`: conflict set = exactly the 11 files, `git diff --name-only` develop↔merge-tree = those 11, the 2 auto-merges resolving to develop's content. merge-tree is deterministic per (ref spelling, commit pair) — its conflict markers label the refs passed, so a merge-tree TREE HASH is not a pin; quote the file set, never the hash)
- [ ] Validate (a TRIPWIRE against mis-invocation — NOT a content proof): `git rev-parse HEAD^{tree}` == `git rev-parse HEAD^1^{tree}`. First parent is develop by construction, so the pin is self-referential and cannot go stale (a literal develop-tree hash DID go stale here — measured `4ad737a9…` pre-amendment, invalidated by the very commit carrying it). Under `-s ours` equality is true by construction, so this proves nothing about content — it reds only if the merge was mis-invoked (forgot `-s ours`, or `-X ours`, which does NOT guarantee ours-everywhere). The content adjudication is the pre-measured file-set evidence above.
- [ ] Validate: `git merge-base --is-ancestor origin/main HEAD`
- [ ] ~~Scoped `cargo check`~~ DROPPED per verdict: the tree is byte-identical to origin/develop's; a build re-derives a fact the tree hash settles
- [ ] Merge commit (references this issue, names twin commits `569daf98e`/`3c844aebb`/`5e2b730f2` + the file-set adjudication in body, Session marker)
- [ ] ONE push: `git push --atomic origin develop HEAD:main` — both-or-neither; moves origin/main `5e2b730f2` → merge commit (fast-forward). Cost accepted per owner ("dont mind about main ci"): the 279-commit push range matches every path filter, so docs_gate / full_gate (macos-latest, 10× minute multiplier) / required_features_touched / lean_proofs / wasm32_gate fire once — the only automatic whole-repo lane this content has ever had (nothing fires on develop). A red there is a true finding about develop, not a merge defect.
- [ ] Fast-forward local `main` (a0ef5d68, 279 stale, confirmed ancestor of origin/main) to the merge commit
- [ ] Close: HISTORY.md row citing the merge sha + the **10 recovered-by-history issue files (`.issues/747–756`)** — present on main, deleted on develop under the noise-reduction rule, present at merge-base; a future reader diffing main against develop would see them as "missing on develop" and the record pre-empts that. Remove this file, `docs:` commit.

## Risk notes

- Sibling agents own this repo: develop was clean and == origin/develop at plan time; re-check `git status -sb` before the merge and before the push.
- Highwaters: ours (876/883) keeps the monotonic ledgers; main's 807/844 bumps are already consumed in develop's numbering history (Bench 844 exists on develop, Issue 807 closed at line 3004 of HISTORY).
- No content is lost: every main-side hunk exists on develop in evolved form (table above); the only main-unique text (the transplant narration) is preserved permanently in the merge commit's second parent.
