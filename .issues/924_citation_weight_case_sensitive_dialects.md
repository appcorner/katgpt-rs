# Issue 924 — citation_weight's dialects are case-sensitive; lowercase `plan N` prose is invisible to duplicate adjudication

**Status:** CLOSED 2026-10-07 — all four tasks landed same-day (code commit + hygiene record in HISTORY.md; file removed per noise-reduction). Measured, repaired, oracle-confirmed.

## The finding

`scripts/citation_weight.py`'s `DIALECTS` table admits `"Plan"` and `"P"` (and
the other kinds' long forms) — **case-sensitive**. seal-game-editor's corpus
cites plans in lowercase prose (`plan 241`, `plan 272`), and those sites are
invisible to the instrument:

| Pair | instrument's ambiguous sites | reality (case-insensitive sweep) |
|---|---|---|
| `Plan 241` | 4 — of which 3 were the *issue-205 document itself* (the doc discussing the ambiguity) | the 4th + **3 live lowercase `plan 241` sites** (`.plans/237:4`, `.plans/257:19`, `.proposals/003:3`) |
| `Plan 272` | 3 — 273:49 + 274:8 + issue-205 | plus **1 live lowercase site** (`scripts/README.md:159`) |

Consequence, both directions:

1. **Undercounted weight** — the instrument returned WEAK (+1, 3/4 UNRESOLVED)
   and UNDECIDABLE where hand reads found every real site decidable by
   subject. The 514 precedent (hand reads overturn UNRESOLVED) still worked,
   but only because the sites were read by hand anyway.
2. **Silent stale citations after a renumber** — the loser's citation surface
   is what must be rewritten in the same commit. A renumber trusting the
   instrument's site list would have left **5 live citations pointing at the
   wrong plan** (three of them naming the GLB lane as `plan 241` after 241
   stayed with a different plan). The editor renumber caught them only via a
   manual case-insensitive sweep.

## Why this is the recorded failure shape

The `DIALECTS` comment itself documents the last narrow-dialect miss
(`R<NNN>` = 30% of research citations, "found by accident") — this is the
same class one case axis over: the table was measured over riir-ai's corpus
(where prose says `Plan N`), and seal-game-editor's house style says `plan N`.

## The family check (before fixing in one instrument)

`citation_weight.py` is the adjudication half; per the rule-in-one-instrument
law, grep the family before landing a repair: `numbering_drift_sweep.py`,
`issue_citation_gate.py`, `citation_drift_sweep.py` — any sibling carrying a
citation-dialect regex needs the same lowercase admission (or a shared
dialect table), each with its own re-measured counts.

## Tasks

- [x] Re-measure the dialect counts case-insensitively over at least riir-ai + seal-game-editor corpora; record the lowercase-form counts in the `DIALECTS` comment the way the `R<NNN>` row is recorded
  - **Measured 2026-10-07** (tracked text files): seal-game-editor is lowercase-DOMINANT — `plan` 1,119 vs `Plan` 591, `issue` 719 vs 228 (~2/3 and ~3/4 of its citations were invisible); riir-ai 406 `plan` / 340 `issue`; katgpt-rs 224 / 146 lowercase sites. Lowercase bare-letter forms measured as NOISE, not signal (riir-ai: 68 `p<NNN>` + 80 `b<NNN>` identifier-shaped sites) — so the bare letters stay UPPERCASE-only. Recorded in the `DIALECTS` comment.
- [x] Admit the lowercase forms (or compile the pattern per-kind with `re.IGNORECASE`); keep the strict-margin + UNRESOLVED semantics unchanged
  - `citation_re()` wraps each long-form prefix in a scoped `(?i:…)` group (per-alternative, so the bare-letter arms keep their case); selftest pins `plan 163` matching, `p163`/`r020` NOT matching.
- [x] Sweep the citation-instrument family for the same case-sensitivity; repair as one shared mechanism if more than one carries a dialect table
  - **`issue_citation_gate.py` carried the same class** — `_HEAD`/`_HEAD_1D`/`_TAIL_1D` + `alias_trail_owners`' re-search now scope `(?i:)` over the kind word, and `citations()` normalizes the matched word back to the canonical KIND (`_CANON`) so every downstream bucket keys one spelling. Effect on this repo's pinned docs: scanned 475→476 — the previously-invisible `issue 739` in AGENTS.md (the rust-toolchain pin citation); width-bound complement still 0; gate PASS. **163 lowercase heads exist across sibling AGENTS/HISTORY docs** (riir-shader alone 53) — reported, no repair owed: this gate scans only its own repo's pinned docs, and no other instrument counts kind+number citations in those files. **`numbering_drift_sweep.py` adjudicated NOT the class** (imports `removed_by_number` file-shape machinery only, no dialect regex); **`citation_drift_sweep.py` NOT the class** (by-name stem patterns — literal filenames, case-correct as-is).
  - One prose repair: riir-kat `HISTORY.md` `issue 3` → `issue 003` ×3 (single-digit form the width bound cannot read; zero-padding is that file's own convention).
- [x] Re-run the seal-game-editor 241/272/276 adjudication with the fixed instrument and confirm it now sees the 5 lowercase sites (the hand-read record in that repo's issue 205 is the oracle)
  - **Confirmed and exceeded**: 241 sites 4→15 (verdict WEAK → `241_glb_delivery_lane` leads +5 over 9 decided); 272 sites 3→12; 276 unchanged (1). All 4 hand-read lowercase oracle sites now visible AND attributable (`.plans/237:4`, `.plans/257:19`, `.proposals/003:3`, `scripts/README.md:159`) — plus ~10 additional live lowercase sites the hand-read never swept (`tools/neuron-publish/src/*.rs` code comments, `scripts/regate.py:2`), which the hand-read's `.md`-only pass had missed entirely. Validation: citation_weight selftest PASS; issue_citation_gate selftest + full run PASS; numbering_gate PASS; numbering_drift_sweep PASS; **docs_gate 35/35 PASS** (86.3s wall).
