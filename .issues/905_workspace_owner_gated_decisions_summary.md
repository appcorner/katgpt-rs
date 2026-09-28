# Workspace owner-gated decisions — consolidated summary (2026-09-28)

**Status:** OPEN — audit complete; Claude verdict round 1 REVISE + round 2 fixes applied; awaiting owner ratification per ID. **Nothing in this issue executes anything** — every item needs the owner's explicit go. Do not act on any row without the owner naming its ID.

## Purpose

One place listing every OPEN owner-gated decision across the workspace, each with evidence, classification, and a recommended decision (REC). Classification:

- **OPEN** — owner action needed now/soon; work or work-products are waiting on it.
- **TRIGGER** — policy already decided; the owner (or a cred holders) acts at a defined future trigger (release, mainnet ceremony, first consumer). Recorded for completeness; not urgent.

Top context facts this audit confirmed on disk: the **goal-salience soak is LIVE** (`/tmp/goal_salience_soak/authority.log` ticking at ~9.7/10 Hz, 1 relay client, 6230 soak lines — day 5 of 7); the replay-econ Phase-3 plan (P015) is fully landed and inert at `replay_cut_bp=0`; the covariability promotion bar (Plan 589) is met and reads "default-on remains an owner call"; `asset_vessel`'s stated blocker (Plan 029) is closed.

## A. Money, chain ceremonies, tokenomics

| ID | Repo | Evidence | Gate (short) | Class | REC |
|---|---|---|---|---|---|
| A1 | riir-dapps + riir-clippy | dapps `HISTORY.md:3650` · clippy `.proposals/015:3,254` · **dapps `.benchmarks/013_replay_econ_sim.md`** | Replay rewards Phase-3: ratify non-zero `replay_cut_bp`? | OPEN | **HOLD cut at 0 (dated hold)** — corrected on the reviewer's read of the econ-sim: 100–300 bp pays verifiers NOTHING (verifier share needs ≥2,223 bp at weight 9; 6,667 at weight 3); the bench's own v1 rec is keep 0, arm later on devnet only as ONE act (cut ≥2,223 bp + canary 10% with bound raised to ≥1000 + the G2 ≤5% re-wording + the four monitoring counters exposed). The credit side echoes the submitter's own verdict vector, so arming pays for unchecked verdicts |
| A2 | riir-dapps | `AGENTS.md` mainnet bundle rows | One-window mainnet bundle: treasury fund → `/tuna/genesis` ×2 → mainnet crank token → SettlementSubmitter ceremony; apex GitHub OAuth app (Plan-022 D6, currently 503 fail-closed) | TRIGGER | Pre-write the runbook now (curl+secret-put sequence exists in HISTORY); fire nothing before the "SEAL shop ready for real money" trigger; apex OAuth belongs in the same window |
| A3 | seal-remake + riir-chain | seal-remake `.plans/010:3` · riir-chain `.docs/06_operations/token_registration_ceremony.md` | Ratify §3 table: D1 NO_REMINT set, D2 gated transfer set, D3 cap, D4 authority | OPEN | Ratify as-is — NO_REMINT is right for a fixed-price shop + faucet economy; fold execution into the A2 window |
| A4 | riir-chain | `.plans/archive/032:281` · `.plans/037:965` | Owner Call 1: pin target tx rate + define p99 semantics (blocks S2 DoD + F21 bound) | OPEN | Pin semantics as END-TO-END (`POST /wallet/tx` → signed receipt) and name the rate explicitly (below the ~1,900 tx/s drain cap) — the 129–170 ms figure is end-to-end-only and the two p99 readings differ ~1000×; **re-measure after 032's flush→seal→ack path** (the 08-21 number predates it) |
| A5 | riir-chain | `docs/builds/033-R2-build-record.md:122` | R1 keyed MAC key-management scheme for ledger-at-rest (last unkeyed surface) | OPEN | Decide before mainnet; REQUIREMENTS first: HKDF with its own domain-separation from a dedicated at-rest root + a rotation & re-MAC story. Deriving from the genesis key is acceptable ONLY with explicit owner acceptance of the coupling (one leak = signing + integrity both lost, no independent rotation) |
| A6 | seal-online-remaster | `.plans/046:166` D1 | Regenerate archetype-0 NPCB + upload to R2 (owner infra creds) | OPEN | One wrangler command once creds exist; factory-verified at pinned digest — schedule with next deploy window |
| A7 | riir-dapps | `AGENTS.md:141` | healqual fleet-join machine-consent decision (`lane: "healqual"`) | OPEN | Decline for now (card already renders single-machine honestly); revisit at second contributing machine |
| A8 | riir-dao | `HISTORY.md:21` | CF Workers-AI token for metered advisory leg | TRIGGER | Defer — groq→infron ladder closed the blocked leg; add token only when free legs exhaust |

## B. GPU budget & training queue (riir-train + riir-ai)

| ID | Repo | Evidence | Gate (short) | Class | REC |
|---|---|---|---|---|---|
| B1 | riir-train | `.issues/README.md:27` | Issue 565 GateProjWeights repair (written, unapplied) — owner go; unblocks 483/470/545 compiles | OPEN | **Grant go now** — minutes of CPU, highest leverage per minute in the queue |
| B2 | riir-train | `.plans/360:7` | OPSD/GDSD pilot, ≤16 GPU-h | OPEN | Approve — cheapest decisive ask; modelless half already landed |
| B3 | riir-train | `.issues/512:22` | SPACE L4 re-attempt campaign 40–80 GPU-h | OPEN | Approve once B1/B2 land; pre-registered bar + kill-criterion bound the loss |
| B4 | riir-train | `.plans/344:137` | Phase-1 full-recipe launch despite failed Phase-0 STOP clause | OPEN | **Decline** — respect the plan's own STOP; re-open needs a new hypothesis, not a retry |
| B5 | riir-train | `.plans/342:107` | (a) dump v2_best drafts 1–2 h vs (b) retrain vs (c) wait | OPEN | Pick (a) as recommended in-plan |
| B6 | riir-train | `.issues/487:10` | 14.5-day campaign declined; subscale slice vs fold-in | OPEN | Fold into the next Bonsai run (zero marginal window cost) |
| B7 | riir-train | `.research/425:5` | Sterling flagship 165–315 GPU-h budget | OPEN | Defer behind B1–B3; revisit only after those resolve |
| B8 | riir-train | ~11 `.research/42x–45x` notes | DISTILLED — pending owner next-step | OPEN | Batch-triage after B2/B5 resolve; most fold into those plans |
| B9 | riir-ai | `.plans/528_bonsai_go_arena:428` | G-BONSAI-BEATS/CRUSHES-MOKA bout ~6.4 h 4090 | OPEN | **Decline the bout, do NOT retire the gates** — gates are Phase-3/4 pre-registered success criteria (Plan 528: re-derive only if G-LORA-HELPS ever passes); record both gates DORMANT+conditional. Retiring deletes the conditional path to answer a question nobody is asking |
| B10 | riir-ai | `.research/372:3` | Two unmeasured 4090 prefill levers (A has strengthened prior art) | TRIGGER | One quiet-window 4090 cell each when the box is idle; not scheduled otherwise |

## C. Feature promotions (GOAT/soak-gated)

| ID | Repo | Evidence | Gate (short) | Class | REC |
|---|---|---|---|---|---|
| C1 | riir-ai + riir-mmorpg-examples | riir-ai Issue 1002 · soak log LIVE day 5/7 | goal_salience default promotion after green one-week soak | OPEN | On day 7 verify against criteria WRITTEN DOWN BEFOREHAND: tick-rate floor, zero panics/reconnect storms, recorded kill-switch drill. Then promote in one commit (env kill-switch stays); any criterion missed → extend soak, never promote |
| C2 | riir-ai | `.plans/589:110` | Covariability drift-boot default-on (bar met) | OPEN | Promote with kill-switch — bench passed (6-night detection, 0 steady-state fires) |
| C3 | riir-game-sdk + riir-mmorpg-examples | sdk `Cargo.toml:483` · mmorpg `Bench 031:84` | `asset_vessel` default flip (blocker Plan 029 closed) | OPEN | Flip both repos in one commit + re-pin counts — but FIRST measure what wasmi-in-every-native-build costs (default-build time + size bench), and put the feature-off guard as a `test_gate`/local row or tie it to E1 (a plain CI row won't run: main-only pushes + Actions paused) |
| C4 | riir-mmorpg-examples | `Bench 027:5` | Hero fusion Sum/Mean verdict | OPEN | Adopt bench rec: keep opt-in; if later promoted, Mean as default arm |
| C5 | riir-mmorpg-examples | `Cargo.toml:662` | deliberation_cadence production A/B then owner call | TRIGGER | One ORCHARD_BENCH A/B produces the evidence; then decide |
| C6 | seal-game-editor | `.plans/250:4` | vessel_cas default promotion | OPEN | Keep opt-in until live pack/unpack e2e soaks green over several real trees |
| C7 | riir-clippy | `Cargo.toml:609` | structural_admission activation (no auto-serve kind active) | OPEN | Keep inert until a priced first consumer exists (P1×frontier_miner fusion) |
| C8 | riir-reflex | `.issues/052:3` | typed_decisions train cap 800→1200 (+10.7 pt A0) | OPEN | Approve — cal bytes unchanged; ripple is mechanical (pins + republish) |
| C9 | riir-reflex | `.research/003:3` | Thai lane implement-or-close | OPEN | Keep closed; reopen trigger already documented |
| C10 | seal-game-editor | `.plans/199:572` (D5) | Walk-grid movement G3 promotion post re-pin | TRIGGER | Re-pin SCN floors after the reproject, then take D5 with the bench record |

## D. Research / POC go-no-go

| ID | Repo | Evidence | Gate (short) | Class | REC |
|---|---|---|---|---|---|
| D1 | riir-ai | `.proposals/README.md:101` (035) | Research-automation agent verdict TBD | OPEN | Formal HOLD with dated re-look — manual idle-loop demonstrably covers it |
| D2 | riir-ai | `.proposals/README.md:101` (042) | QuestAST POC owner go/no-go | OPEN | Fire the Phase-0 bench (one measurement; converts TBD into data) |
| D3 | riir-ai | `.plans/541:65` (H9) | Prefill residual aliasing: fix vs accepted-risk | OPEN | Record accepted-risk explicitly with probe default-OFF; flip issue wording so it stops reading as deferred |
| D4 | riir-ai | `.issues/950:285` | Upstream arc-swap filing go | OPEN | Go — recipe validated twice; filing is cheap and unblocks the class |
| D5 | riir-ai | `.docs/01_orientation/737:97` | T-CI boundary-guard parsing + Issue 739 engram relocation go | OPEN | Grant — mechanically scoped; 739 has measured payoff (19→89 packages) |
| D6 | riir-reflex | `.research/002:107` | Native agentjev serving lane | TRIGGER | Decline for now — HTTP comparison lane already yields the data |
| D7 | riir-infer | `.issues/017:64` | gpu_transpose dead module: wire/delete/re-document | OPEN | Delete — `transpose_cubecl.rs` (Issue 572) covers the reachable use; UPDATE BOUNDARY.md's "GPU transpose kernel" ownership line to point at `transpose_cubecl.rs` in the SAME commit (017 says so itself) |
| D8 | riir-infer | `.issues/015:25` | Audio-lane PoC BOUNDARY widening + dep pick | OPEN | Reuse `objc2-core-ml` row (lightest for public substrate) |
| D9 | riir-infer | `.issues/1004:34` | Research 327–332 routing to public katgpt-rs | OPEN | Defer with T7/S8; if moved, sanitize under the fence rules |
| D10 | riir-infer | `.issues/1003:5` | S6b training-families long-term home | OPEN | Keep riir-gpu-side (by design); close as by-design absent consumer pull |
| D11 | riir-game-sdk | `.plans/006:3` (A2) | Basis-gate: fund packer lane vs retire gates | OPEN | Retire the gates — no packer since the Python retirement; packer-less gates invite silent drift |

## E. Infra, CI, hygiene

| ID | Repo | Evidence | Gate (short) | Class | REC |
|---|---|---|---|---|---|
| E1 | workspace-wide (4 gates) | deployer `Bench 002:53` · riir-shader `.issues/052:4` · riir-instinct `AGENTS.md:103` · seal-online-remaster A6 | Actions-spending pause: rehearsal re-run, stale-serve prod deploy (CF escalation), instinct devnet push, R2 upload | OPEN | One owner call: resume Actions spending OR codify manual-deploy posture per repo; riir-shader additionally needs a Cloudflare support ticket (7 version IDs + frozen-etag evidence) |
| E2 | riir-instinct | `AGENTS.md:103` | Devnet container push (creds only; local e2e green, staged artifact verified) | OPEN | Push devnet as an explicitly MANUAL, credentialed push under the manual-deploy posture (this row is E1-gated — the ordering conflict the reviewer flagged is resolved by naming it manual here) |
| E3 | riir-clippy | `HISTORY.md:671` | ops_dashboard checks out retired `gist-rs/riir-unity` | OPEN | Drop the dead checkout; mirror set = the contract repos |
| E4 | riir-clippy | `HISTORY.md:7045` | 4 files behind never-defined features (`crowd_attention_bench`/`crowd_coherence_bench`) | OPEN | Remove the targets — cargo silently skips them; they gate nothing |
| E5 | riir-clippy | `.research/084:127` (083 collision) | Renumber dual-allocation collision vs leave | OPEN | Leave as-is + one-line disambiguator; renumbering breaks more citations than it fixes |
| E6 | riir-clippy | `HISTORY.md:10184` (riir-ai 958) | `browser` feature retire-or-fix | OPEN | Retire — zero consumers, wasm32 dep graph pre-broken |
| E7 | riir-clippy | `Bench 057:42` | Standing CF sign-off (shared-account llama + CodeCureAgent loop) | OPEN | Explicit allow/deny in one HISTORY row — ambiguity silently blocks future llama-column benches |
| E8 | riir-clippy | `.distill/001:107` | Boundary-guard C8 four-rev / C9 multi-rev interpretation | OPEN | One-line adjudication in HISTORY; C-rule set actively consumes this call |
| E9 | seal-game-editor | `.plans/251:6` | Destructive `just reproject` on live working.db | OPEN | Snapshot working.db + locale DBs first; quiet window, editor closed |
| E10 | seal-game-editor | `.plans/197:406` (T4.4) | Mainnet unlock ceremony | TRIGGER | Fold into the A2 mainnet window (same key-pinning act) |
| E11 | sealm-toolkit | `README.md:373,263` | Reserved model-id band ratification + legacy_materials look sign-off | OPEN | Pin the band in neuron-db registry + client loader before content rows depend on it; route looks through content-migration-lead sign-off |
| E12 | katgpt-rs | `.benchmarks/668:71` | Base-model recirculation check blocked on Gemma license + HF token | OPEN | Accept license + provision token to the 4090, else retire the check as permanently blocked (IT-only evidence stands) |
| E13 | katgpt-rs | `HISTORY.md:3148` | Workspace `rust-version` pin (~30 manifests) | OPEN | Adopt `[workspace.package] rust-version` — cheap one-time close |
| E14 | riir-game-sdk | `Bench 030:56` (Trigger-3) | 1000³ full-population soak (`#[ignore]` env runner) | TRIGGER | One overnight 4090-idle run closes the whole Trigger-3 family |
| E15 | riir-game-sdk | `Bench 029:178` | Budget form (p50-every-run vs min-of-8+p99) + CI posture for release budget suite | OPEN | Pin min-of-8+p99 (p50-every-run unenforceable on shared disks); release suite → manual-dispatch lane |
| E16 | riir-shader | `HISTORY.md:1336` | Gamefx promotion gate (effect + posture unnamed; issue file kept open as its reader) | OPEN | Pin which effect + posture promotes so the kept-open issue can close |
| E17 | riir-shader | `HISTORY.md:70` | Rim re-aim for converted rows | TRIGGER | Flip per-family on the next conversion pass with toon-harness before/after |
| E18 | riir-reflexer | `Bench 002:110` | DEFAULT_PINS empty until first mint | TRIGGER | Pin at first mint; minting is deterministic so re-minting moves no pin |

## Owner checklist (in the reviewer-corrected order)

- [ ] B1 grant go (565 repair) — minutes of CPU, unblocks three compiles
- [ ] C8 approve typed_decisions cap lift · **C2 promote covariability drift-boot (kill-switch retained)** · C4 hero fusion stays opt-in (record)
- [ ] B4 DECLINE Plan 344 Phase-1 · B9 DECLINE bout + record gates dormant/conditional (both cost nothing)
- [ ] B2 approve OPSD pilot (≤16 GPU-h) · **B5 pick 342(a)** · B6 fold 487 into next Bonsai run · B7 defer Sterling behind B1–B3
- [ ] Record-only rows (TRIGGER/hold/decline, no action now): A6–A8, B3, B8, B10, C5–C7, C9, C10, E18
- [ ] C1 day-7 soak verdict against PRE-WRITTEN criteria → promote or extend (goal_salience)
- [ ] C3 measure wasmi default cost first, then flip asset_vessel (one commit + re-pins + local test_gate row)
- [ ] A4 pin end-to-end p99 semantics + explicit rate (re-measure post-032) · A5 key-mgmt requirements (HKDF domain-separated + rotation story) — mainnet prerequisites, no money moves
- [ ] E1 decide Actions-spending posture; then E2 manual devnet push (instinct)
- [ ] A1 record DATED HOLD at cut=0 — or the full one-act devnet arming (≥2,223 bp + canary 10% + G2 re-wording + monitoring counters)
- [ ] A2/A3/E10 mainnet bundle window only when the SEAL-release trigger fires (runbook first)
- [ ] Batch-triage: D1–D11, E3–E9, E11–E17 (one session, one commit per repo)

## Claude verdict (ping-pong record)

Round 1 (2026-09-28, reviewer session `bf9e63dd-7469-43e9-a41a-4ec8f51ff08d` — the ID returned by the verdict tool for this review; provenance is the tool call, not a signature): **REVISE** — reviewer read the cited evidence directly (dapps Bench 013 + Proposal 015; riir-chain Plans 031/032; riir-ai Plan 528; riir-infer Issue 017; sdk Cargo + mmorpg Bench 031). All corrections applied above:

- **A1 OVERTURNED by its own econ-sim** — 100–300 bp pays verifiers nothing (share needs ≥2,223 bp); bench v1 rec is hold at 0. Issue now carries the dated-hold rec + the one-act arming preconditions if the owner ever wants it.
- **B9 corrected** — decline the bout, keep the gates dormant/conditional (they are Phase-3/4 pre-registered criteria, not residue).
- **A4 completed** — pin END-TO-END p99 semantics + explicit rate, and re-measure post-032 (the 129–170 ms figure predates flush→seal→ack).
- **A5 reworded** — requirements-first (domain-separated HKDF + rotation story); genesis-key derivation only with explicit owner acceptance of the coupling.
- **C3/C1/D7/E2 tightened** — measure wasmi cost before the C3 flip + local-gate row (CI won't run it); pre-written day-7 soak criteria; BOUNDARY.md line updated in D7's same commit; E2 explicitly manual under the manual-deploy posture (resolves its E1 ordering conflict).
- **Agreed as written:** B1, B2, B3 (kill-criterion emphasized), B4, B5, B6, B7, A2, A3, C2, C4, C8, D1–D5, D8–D11, E3–E7, E12–E14 — "low-risk, reversible, and mostly keep plans' own recommendations or STOP clauses."
- **Corrected ratification order** adopted in the checklist above.

Round 2 (same session): REVISE with four small fixes — C2/B5 restored to the checklist, a record-only line added for the uncovered rows (A6–A8, B3, B8, B10, C5–C7, C9, C10, E18), G7→G2 corrected on the A1 arming precondition (reviewer checked it against dapps Bench 013), status line updated. Reviewer's closing: "Once those fixes are in, this is AGREE; a round 3 isn't needed." Fixes applied; **standing verdict AGREE (conditional per the reviewer's own round-2 words)**. Honest caveat: the formality-closing ping back to the reviewer was refused (verdict session had expired by then), so no third-round acknowledgement exists — the AGREE rests on the reviewer's conditional statement plus the fixes being applied verbatim; a fresh verdict session can re-confirm on request. Reviewer note kept verbatim: the reviewer's "safest ratifications in the batch" are C2 (covariability) and C8 (cap lift) — reversible, gated, measured.
