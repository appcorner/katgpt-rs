# 921 — citation oracle cannot resolve two-lane self-references (riir-infer 9 IN-LOCAL-RANGE rows)

**Status:** PARTIAL — Option 3 (corpus repair) LANDED 2026-10-07 (riir-infer `71c8eed`; sweep green on riir-infer, ILR 9→0, novel 3→0, pin 0 holds); **Arm B DEFERRED** (design below, `- [-]`); Arm A CLOSED-token variant + Arm C rejected by verdict (round 2)

- [x] Qualify bare `Issue 980` → `riir-ai Issue 980` (riir-infer `f0191e3`)
- [x] Option 3: rewrite the six self-allocation headings (004–009) into the readable date-led grammar + pin the grammar in riir-infer AGENTS.md (riir-infer `71c8eed`)
- [-] Arm B — two-lane gap boundary: DEFERRED until the next real touch of this instrument. Design of record: for a repo whose kind dir carries `.highwater_local`, citations in the GAP (`highwater_local` < n < smallest file-backed allocation ABOVE the local counter) classify CROSS/ORPHAN instead of IN-LOCAL-RANGE. Floor derived from git-log file ADDITIONS (the `removed_by_number` record source — NOT the worktree: riir-infer 998/1004 exist only in history), heading-oracle allocations EXCLUDED from the floor (else Arm-A-style additions shrink the gap — order-dependent), the derived floor PRINTED on the per-repo line (or declared in the repo beside `.highwater_local`), contract-name keyed per round-1 Hole 1, `.highwater_local` added to the head-provenance patterns, and a two-sided `--prove-fires` at riir-infer `f0191e3^` (bare `Issue 980` must classify CROSS → riir-ai) vs `f0191e3` (zero).

## The finding

`citation_drift_sweep.py` reds on riir-infer: `IN-LOCAL-RANGE 10 > pinned 0` (measured 2026-10-07).
One row was a genuine missing qualification — riir-infer `HISTORY.md` cited `Issue 980` bare for
riir-ai's Bonsai-2 hadamard loader issue; repaired to `riir-ai Issue 980` (M3-side fix commit of
this session, riir-infer develop). The remaining **9 rows are the real finding**: they cannot be
repaired at the document level and cannot be resolved by the current oracle.

## The 9 rows, precisely

riir-infer's `HISTORY.md` self-narrates six closed local-lane issues — 004, 005, 006, 007, 008,
009 (the early CUDA-kernel rungs: "the narrow reg4 rung", "CUDA graphs — NEGATIVE", "the sgemm
tile ladder", ...) — across 9 citation sites (6 headings + 3 prose cites). Measured on BOTH boxes:

- **The files never existed.** `git log --all --diff-filter=A -- .issues/` on the M3 and on the
  4090 agree: `.issues/001..003` and `.issues/010..035` (+998/1003/1004) were added; **004–009
  were never committed on any ref either box has**. The numbers were consumed by the early
  4090-side kernel lane and narrated in HISTORY only; `.issues/.highwater_local` = 035 covers them.
- The headings are **self-qualified by name**: `## 2026-09-25 — riir-infer Issue 008 CLOSED: ...`
  — the repo name is written directly on the citation. Qualification still fails because
  `is_qualified` asks whether the NAMED repo OWNS the number, and the ownership oracle
  (`allocated()`: worktree walk + git log + headings) finds no record of 004–009:
  - worktree: no files (never existed);
  - git log: no adds (confirmed both boxes);
  - headings: the style `## <date> — <repo> Issue NNN CLOSED:` is UNREAD — the Issue-823 date-led
    arm requires the kind word immediately after the date delimiter, and the repo name sits
    between (riir-infer `heading_unread=37/39`, `novel=3` — three numbers NO other oracle knows,
    which are exactly these).
- `n ≤ top=1004` (the INHERITED riir-ai carve counter, not the local lane) → IN-LOCAL-RANGE →
  UNDECIDED forever, ceiling `max_in_local_range: 0` breached.

## Why the obvious repairs are wrong

1. **Document repair is impossible** — the owner's name is ALREADY written on every heading;
   qualification fails on ownership, not on naming.
2. **Reading `.highwater_local` into `top_allocated` (the first-session proposal) would NOT
   resolve a single row** — IN-LOCAL-RANGE is UNDECIDED by design (`n ≤ top`); raising or
   splitting `top` never moves a row out of the bucket. The first session's proposed mechanism
   was wrong.
3. **Re-pinning `max_in_local_range: 0 → 9` is BLOCKED by the repo's own rule** (AGENTS.md,
   heading-oracle section): "Do not re-pin max_in_local_range for a heading-blind row — a repo
   at or near b/b unread cannot have its IN-LOCAL-RANGE count trusted." riir-infer is 37/39
   unread — the warning applies verbatim. The sweep stays red on riir-infer until the oracle
   can read the records.

## The priced instrument arms (owner call)

- **Arm A — the own-name heading variant** (resolves all 9 rows; the honest fix): extend
  `heading_allocated()` with `## <date> — <OWN repo name> Issue NNN ...` — the repo's own
  contract name as the intermediate token. Distinct from the REFUTED residual widening
  (`resolved`/`follow-up` — arm 2 pins the negative): the discriminator is a CONTRACT-NAME
  token, not prose, and self-naming in a self-record is an allocation declaration, not
  commentary. Risk to price: a heading citing a SIBLING by name (`## compared to riir-dapps
  Issue 017`) must NOT credit the local repo — the arm must match the OWN name only, and the
  sibling-name form must stay unread. Cost: selftest arms both directions + floor re-derivation
  across the family (heading-floor pins move where unread counts drop).
- **Arm B — the two-lane boundary via `.highwater_local`** (precision, resolves 0 of the 9):
  a repo carrying `.issues/.highwater_local` DECLARES (AGENTS.md-documented discipline,
  machine-readable) that its local lane tops at that counter and the moved-doc range above it is
  inherited-only. Citations in the GAP (`highwater_local < n < inherited floor`) are then NOT
  local-intent and classify CROSS/ORPHAN — future `Issue 980`-class rows would be born as
  actionable CROSS findings with the mechanical repeat/repeat-adjacent repairs, instead of
  collecting as UNDECIDED noise behind a red ceiling. Sound: uses the repo's own declared
  counter; repos without the file are unchanged (one new branch, membership-shaped).
- **Arm C — accept the red** (status quo): the sweep stays red on riir-infer exactly like the
  two ghost rows (dapps Proposal 17, instinct Proposal 13) — visible, documented, owner-gated.

Arms A and B compose: A clears the standing 9, B prevents the next 980. A alone greens
riir-infer; B alone leaves the ceiling breached.

## Evidence

- Sweep: `python3 scripts/citation_drift_sweep.py --full` — riir-infer block, 9 `undecided:` rows
  at HISTORY.md:122/126/130/134/138/247/248/248/249 (post-980-repair count).
- Both-boxes file census: `git log --all --name-only --diff-filter=A -- .issues/` on M3 and
  4090 (`E:\git\riir-infer`) — identical sets, no 004–009.
- First-session summary misdiagnosis ("oracle doesn't credit its .highwater_local small lane")
  corrected here: reading the counter moves no row (bucket is UNDECIDED by design); the real
  seam is the ownership oracle's three legs all blind for narrated-only numbers.

## Summary

**(1) Original task**: pick up the compaction session's remaining items — the riir-infer
in-local-range instrument blind spot.
**(2) Accomplished**: qualified the one genuine row (riir-ai Issue 980, riir-infer develop);
censused both boxes (files never existed anywhere); corrected the first session's mechanism;
priced three instrument arms with the floor re-pin ruled out by the heading-blind rule. This
issue = the record.
**(3) What remains**: owner verdict on Arm A / Arm B / Arm C (Arm A recommended — it is the
only arm that greens riir-infer honestly); the two ghost citations (dapps 17 / instinct 13)
remain owner-gated; the 4090 riir-infer checkout (ahead 7 / behind 5) belongs to the ACTIVE
dq_s3_matrix session there — sync after it lands and pushes.
**(4) Active plan state**: no .plans/ in flight; this issue open.
