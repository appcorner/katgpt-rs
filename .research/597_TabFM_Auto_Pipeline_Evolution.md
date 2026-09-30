# Research 597: TabFM Technical Report + TabFM-Auto — Pipeline Evolution Around a Frozen Foundation Model

> **Source:** TabFM — [arXiv:2609.37959](https://arxiv.org/abs/2609.37959) "TabFM: A Zero-Shot Foundation Model for Tabular Data" (Kong, Louidor Ilan, Nie, Narayan, Sen, Zhou, Fu, Oymak, Das — Google Research, submitted 2026-09-29) · TabFM-Auto — [arXiv:2609.37989](https://arxiv.org/abs/2609.37989) "TabFM-Auto: Self-Evolving Pipelines for Tabular Foundation Models" (Fu, Su, Sen, Narayan, Sanghavi, Das, Kong — Google Research/DeepMind, submitted 2026-09-29) · [Google blog](https://research.google/blog/introducing-tabfm-a-zero-shot-foundation-model-for-tabular-data/) (2026-06-30) · feature explorer: deqingfu.github.io/tabfm-auto
> **Date:** 2026-09-30
> **Status:** RECORD — reference kept for later; activation deferred by owner call this session ("we may need this later but not now"). No plans/issues filed; fusion leads parked in §2.4 with novelty TBD.
> **Related Research:** 364 (TabFM — the parent note, blog-era verdict Pass; this note is its arXiv-report delta + the TabFM-Auto extension) · 354 (NPT set-attention Super-GOAT — the architecture parent) · 322 (conformal floor / "Report the Floor" — governs any future calibration adoption)
> **Related Repos:** riir-reflex (harness levers ≈ the pipeline stages; comparison-lane candidate) · riir-clippy (the frozen-corpus + evolved-pipeline architecture kin) · riir-train (synthetic-SCM pretraining, one-lined in R364)
> **Classification:** Public (published Google work; no game/chain/shard IP in the distilled principles)

## TL;DR

Two papers, one bloodline. **TabFM** (the technical report formalizes what R364 distilled from the blog: 400M-param zero-shot tabular FM, single-forward-pass ICL prediction, pretrained on hundreds of millions of synthetic SCM tables) now carries the full TabArena numbers: **1785.3 Elo zero-shot out of the box — first among default tabular foundation models, above tuned AutoGluon 1.5 extreme (1668.4)** — and TabFM+ (multi-view feature expansion + NNLS ensembling + post-hoc calibration) at 1856.0.

**TabFM-Auto** is the new unit: an LLM coding agent (Claude Code / Codex / Antigravity × Opus 5 / Gemini 3.8 Flash) evolves a **data pipeline around the frozen TabFM** — `P = (Φclean, Φfeat, Sctx, Ψpost)` — guided by dataset metadata (column names, task descriptions, auxiliary files) and 3-fold-CV validation feedback, keeping the weights fixed. Results: **1785 → 2013 Elo (+228)**, top-5 positions swept by the five configs, **#1 on MLE-Bench-Tabular** (1827 Elo, 4 Kaggle medals), and — the headline transfer finding — **pipelines discovered for TabFM transfer to OTHER frozen tabular FMs (+69 to +143 Elo) with zero further search**.

The one-sentence mechanism claim (pinned per §1.5, for the record not for novelty): *pipeline-level evolution around a frozen in-context model is lower-noise and engine-portable where end-to-end model search is noisy and engine-locked — because the frozen prior removes retraining variance and the discovered artifact is a short symbolic program rather than fitted weights.*

Owner verdict this session: **keep for reference, defer activation**. The distilled leads (§2.3–2.4) point at riir-reflex (a TabFM comparison lane; the harness levers are already a modelless pipeline-around-frozen-engine) and riir-clippy (the architecture kin — its verify loop already ships the paper's keep-only-improvements-over-identity law). Nothing filed.

---

## 1. Paper Core Findings

### 1.1 TabFM report — deltas since R364 (blog-era)

R364 recorded the architecture (alternating ABD/ABA set attention → row compression → ICL transformer on compressed rows) and the synthetic-SCM pretraining. The arXiv report adds the formal benchmark surface:

| Method | Overall Elo (51) | Classification (38) | Regression (13) |
|---|---|---|---|
| TabFM (zero-shot, single forward pass) | 1785.3 | 1768.7 | 2045.9 |
| TabFM+ (cross+SVD features, 32-way NNLS ensemble, Platt) | 1856.0 | 1836.7 | 2169.2 |
| EXAONE-Tabular | 1764.6 | 1759.5 | 1967.8 |
| AutoGluon 1.5 (extreme) | 1668.4 | 1664.6 | 1851.2 |
| TabPFN-3 | 1660.1 | 1637.7 | 1863.1 |

Zero-shot TabFM outperforms tuned AutoML pipelines and every default tabular FM evaluated. R364's verdict (Pass — architecture covered by R354/NPT set-attention; weights are Google's contribution) stands unchanged; no new architectural element appears in the report beyond what R364 §1.2 recorded.

### 1.2 TabFM-Auto — the mechanism (the distillable shape)

**The search surface.** The agent edits `pipeline.py` — four modular functions + a kwargs dict — and nothing else. The model never changes:

- **`preprocess()` (Φclean)** — data cleaning + target conditioning: recode sentinel values a TFM cannot read from magnitudes (a `0` coil width that means "unmeasured" → NaN + missingness indicator), drop corrupted rows, reversible target transforms `g(y)` (log(1+y), Box–Cox) inverted at the output.
- **`engineer()` (Φfeat)** — semantic feature engineering, "where the LLM's domain knowledge enters": domain formulas written directly from column names (Strouhal `St = fδ*/U∞` and Helmholtz `He = fc/a₀` on airfoil_self_noise, −14.6/−17.3% RMSE; MAP = (2·DBP+SBP)/3 and Shock Index HR/SBP on clinical tables; SDSS color indices u−g…i−z; ICD-9 codes → 19 organ-system chapters), plus label-free transductive features (bipartite graph degrees, pairwise co-occurrences, frequency encodings over combined train+test), collinearity pruning + truncated SVD on wide tables, and auxiliary-file summarization (seismic waveform CSVs → FFT band energies + STA/LTA ratios, 9.4× better than the best external MLE agent; 3D crystal lattices → unit-cell volume + electronegativity dispersion; molecular coords → r⁻³ distances + Karplus dihedral cos²φ).
- **`sample()` (Sctx)** — context selection: multiple context views (natural + minority-oversampled + cluster-stratified) when tables exceed the 16,384-row pretraining length or are class-imbalanced; predictions averaged across views.
- **`postprocess()` (Ψpost)** — output calibration: `g⁻¹` inversion, **log-odds prior shift toward the empirical training class prior**, temperature/Platt scaling — because the synthetic pretraining prior need not match real class balance.

**The protocol (their verification discipline — worth reading whole):**
1. Every run starts from the **identity pipeline P₀** (pass-through, default kwargs); its 3-fold-CV score sets the threshold, and the agent keeps only edits that improve over P₀.
2. Each candidate runs in a **bubblewrap-namespace sandbox** — no network, system libs read-only, test splits *unmounted*; the harness (not the pipeline) calls the frozen model.
3. **Row-order permutation** at load time against index leakage (sorted labels leaking through row positions).
4. Search once per dataset (≤96 evals or 6h on one H100), freeze P*, score official test splits once.
5. **Zero retraining noise** — the frozen core is the argument: "no retraining noise masks small feature gains" (their stated reason pipeline-search beats joint features+architecture+hyperparameter search, which MLE agents do noisily).

**The numbers that matter:**

- 5 configs (3 harnesses × 2 LLMs) take TabArena's **top-5 overall**: 2013.0 / 1993.6 / 1979.6 / 1957.8 / 1940.1 Elo. Regression gains are the largest (+467 Elo — linearized physical ratios + target transforms let the FM interpolate what tree splits approximate coarsely).
- **Pipeline taxonomy:** feature table modified in 95.1% of runs. Cat I (domain formulas, 17/51 datasets): +7.25–8.16% mean test-error reduction — ~3× Cat II (statistical/graph features on anonymized schemas, 34/51): +2.31–2.99%. Cat III (context/calibration only, 5/51): +2.16%. **Domain-readable schemas are where the LLM pays.**
- **The transfer finding:** P* found for TabFM, run unchanged around TabICLv2 (+143.3), TabPFN-3 (+130.8), EXAONE-Tabular (+88.7) — every model improves, no further search. *Pipeline gains are largely engine-agnostic; only the context-size kwarg carried over.*
- **The ablation that carries the thesis:** an unconstrained coding agent (same harness/model/6h budget, may train anything) lands at **1468.8** — 510.8 Elo BELOW TabFM-Auto, slightly worse than 4h AutoGluon. Decomposition: frozen FM prior +316.5, pipeline search +194.3. *Agents paired with a frozen foundation model beat agents searching the whole model space.*
- Cost: five 51-dataset sweeps ≈ **$17.6K** API fees, 1.28B–12.13B prompt tokens per sweep — the real economics of agent-in-the-loop search.
- Their own future work, verbatim, is our healer corpus thesis: *"Operations collected across datasets could form a reusable library that warm-starts search on new tables with fewer evaluations."*

### 1.3 Prior art (the paper's own related-work landscape, §4-satisfying for a record note)

Tabular FMs: TabPFN v1/v2/v2.5/v3 (Hollmann/Grinsztajn et al.), TabICL v1/v2, RealTabPFN, TabDPT, EXAONE — all descend from PFN (Müller et al. 2022). LLM feature engineering: **CAAFE** (Hollmann 2023b), **FeatLLM** (Han 2024), **OCTree** (Nam 2024) — LLM-appended derived columns on single tables; TabFM-Auto's delta over them = the full pipeline (clean/context/postprocess + auxiliary files + code sandbox), not just appended features. MLE agents: AIDE, R&D-Agent, MLEvolve, MLAgentBench — train models from scratch; TabFM-Auto keeps the predictor fixed. Program search: FunSearch, AlphaEvolve. No novelty claim is made in this note, so no further §4 sweep was run.

---

## 2. Distillation

### 2.1 What's training-only (→ riir-train, already one-lined by R364)

The 400M weights, the hundreds-of-millions synthetic SCM pretraining, the 8-member ensemble internals. R364's redirect stands; riir-train's deferral coverage map already carries the row. Nothing new to add.

### 2.2 The transferable decomposition — pipeline-around-frozen-core as a first-class search surface

The genuinely reusable abstraction is the **four-stage interface**. Any frozen inference core (a tabular FM, our modelless corpus engine, a healer rule corpus) has the same four surroundable stages:

| Paper stage | Concern | Nearest shipped kin |
|---|---|---|
| Φclean | make raw input readable by the core (sentinels, transforms) | tokenizer/normalization seams; healer `mask_file`/fixture normalization |
| Φfeat | add semantics the core cannot infer (domain formulas, structure) | corpus entries themselves; feature columns in harness datasets |
| Sctx | which examples condition the core | reflex `--corpus-cap` / `--cal-select-cap` (the cap levers ARE context selection) |
| Ψpost | calibrate the core's output to reality | reflex `--gate-fit-selection`/`--gate-fit-calibrated` (SigmoidGateCalibrator), temperature scaling |

Two protocol laws are equally transferable and already ship here in different clothes: **keep-only-improvements-over-identity** (the healer's `--verify` baseline→apply→re-check→auto-REVERT is the same law) and **frozen-core-means-low-noise-search** (the healer's deterministic oracle; reflex's frozen corpus — the paper's argument that frozen cores make small gains measurable is an argument *for* our modelless posture).

### 2.3 Modelless micro-primitives worth keeping on the shelf

1. **Log-odds prior shift** (Ψpost): shift mean predicted probabilities toward the empirical training prior in log-odds space — cheap, closed-form, composable with (not a replacement for) threshold fitting. reflex's gates fit thresholds on a calibration slice; the prior shift corrects a *different* bias (class-prior mismatch between pretraining/synthetic corpora and real data — exactly the bias a corpus-is-the-model engine can carry). Any future adoption runs the Report-the-Floor conformal gate (the R322 / Plan 340 law).
2. **Multi-view context ensembling under a length cap** (Sctx): natural + minority-oversampled views, averaged — the class-imbalance answer when context is capped. reflex caps corpus per-label (`--corpus-cap`); the *view-ensemble* half (evaluate over several stratified context subsets, average) is not shipped and is trivially modelless.
3. **Auxiliary-file → feature-table summarization** as an agent-editable stage (their MLE-Bench golds came from this) — the general statement: *give the agent the ability to compile unstructured side-carriers into core-readable features.* For the healer: compile non-Rust context (lockfiles, CI configs, bench artifacts) into rule-corpus features. Lead only.

### 2.4 Fusion leads (novelty TBD — owner-deferred, parked here)

| # | Fusion | Surfaces | What it would be | Status |
|---|---|---|---|---|
| F1 | **TabFM as a reflex comparison lane** | riir-reflex `src/lanes/` | TabFM served as a subprocess oracle lane (the GLiNER/AgentJev pattern: their stack serves, our Rust measures) over the harness suites — a model-based tabular opponent for the modelless engine on its own benchmark. HF/GitHub weights are public; sklearn-compatible. | Parked — owner deferred. Cheapest lead if ever activated. |
| F2 | **"Reflex-Auto" — agent loop over harness levers** | riir-reflex `harness/` | The levers (corpus-cap, cal-select-cap, gate-fit, cascade-worthiness-margin) are already a modelless pipeline-around-frozen-engine; an agent/session iterating them per-suite with validation feedback is the TabFM-Auto shape applied to our engine. Prior-art risk: this is AutoML/hyperparameter-search around a fixed predictor — novelty vs CAAFE-class work TBD, not claimed. | Parked. |
| F3 | **Pipeline-transfer across healer domains** | riir-clippy | The paper's +69..+143 engine-agnostic transfer suggests harness/verifier config discovered for one healer domain (clippy) may transfer to siblings (rust_perf, kernel_opt) unmodified. Unmeasured; the natural experiment is replaying one domain's discovered configs on another's score-bench. | Parked — the most clippy-native lead (priority #2 surface). |
| F4 | **Corpus warm-start library** | riir-clippy | Their stated future work ("operations collected across datasets form a reusable library that warm-starts search") is the healer's rule corpus + trajectory store, already shipped — the delta would be *cross-dataset* operation mining (mine recurring pipeline ops, not per-lint rules). | Parked; noted as confirmation the shape converges on ours. |
| F5 | **Frozen cognition core + evolved per-NPC perception pipelines** | riir-ai (weak) | The game angle: HLA/crowd-attention stays frozen (freeze/thaw law) while per-species perception/post-processing wrappers evolve with validation feedback. A stretch — tabular pipelines ≠ NPC cognition; R364's domain-mismatch caveat applies. Recorded for completeness, lowest priority. | Parked, weak. |

---

## 3. Verdict

**RECORD — kept for reference; activation deferred by owner call this session.** The parent verdict (R364: Pass — architecture covered by R354, weights → riir-train) stands for the TabFM model itself. TabFM-Auto's pipeline-evolution mechanism adds no missing architecture — the four stages all have shipped kin (§2.2) and both protocol laws already ship in the healer's verify loop — leaving exactly **two small uncovered modelless candidates** on the shelf (§2.3: log-odds prior shift; multi-view context ensembling under a cap — no consumer yet), parked per the owner deferral. Its genuinely novel claim (full-pipeline agent search around a frozen FM) is the paper's own contribution — CAAFE/FeatLLM/AIDE define the surrounding prior art. The fusion leads F1–F5 are application-level ideas, novelty TBD, and the owner's call is "later, not now": no plans, no issues, no PoCs. This note exists so a future session greps `2609.37989` or `TabFM-Auto` and finds the mapping pre-done.

### What ships where (if ever activated)

| Repo | Then |
|---|---|
| katgpt-rs | Nothing now. The §2.3 prior-shift / view-ensemble micro-primitives would land as katgpt-core calibration helpers behind a feature flag + GOAT gate (Report-the-Floor floor mandatory) if reflex consumes them. |
| riir-reflex | F1 (comparison lane — `src/lanes/` + the measurement law), F2 (agent-over-levers, likely a session protocol before code). |
| riir-clippy | F3 (cross-domain config-transfer experiment), F4 (cross-dataset op mining — post-mining intake shaped). |
| riir-train | Already covered by R364's one-line redirect; the report changes nothing. |

## 4. Open questions

- Is TabFM's HF checkpoint locally servable headless (CPU/token cost per harness suite)? Decides F1's price.
- Does the F3 transfer effect survive our domain gap (healer domains differ far more than tabular FMs do)?
- The paper's regression-dominant gains (+467 Elo) hinge on target linearization — is there a reflex analog (calibration-scale transforms) worth one probe? Parked with F2.

---

> **Delta on:** [Research 364](364_tabfm_zero_shot_tabular_foundation.md) — the arXiv technical report (2609.37959) formalizes the blog numbers R364 recorded; verdict unchanged. TabFM-Auto (2609.37989) is new since R364 and is distilled here.
