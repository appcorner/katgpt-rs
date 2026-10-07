# Issue 924 — citation_weight's dialects are case-sensitive; lowercase `plan N` prose is invisible to duplicate adjudication

**Status:** OPEN — filed 2026-10-07 from the seal-game-editor Issue 205 renumber (branch `docs/renumber-duplicate-plans`, commit `5cf52f9f`): a measured corpus contradiction of the dialect table's coverage claim.

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

- [ ] Re-measure the dialect counts case-insensitively over at least riir-ai + seal-game-editor corpora; record the lowercase-form counts in the `DIALECTS` comment the way the `R<NNN>` row is recorded
- [ ] Admit the lowercase forms (or compile the pattern per-kind with `re.IGNORECASE`); keep the strict-margin + UNRESOLVED semantics unchanged
- [ ] Sweep the citation-instrument family for the same case-sensitivity; repair as one shared mechanism if more than one carries a dialect table
- [ ] Re-run the seal-game-editor 241/272/276 adjudication with the fixed instrument and confirm it now sees the 5 lowercase sites (the hand-read record in that repo's issue 205 is the oracle)
