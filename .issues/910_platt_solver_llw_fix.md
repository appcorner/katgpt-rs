# Issue 910 — the Platt solver needs the Lin–Lin–Weng 2007 fix (damped Newton + the base-rate start), not a candidate fallback as the primary repair

**Status:** OPEN — filed 2026-09-30 from the Issue-909 verdict round 1 (REVISE; the reviewer independently solved the true smoothed MLE on the committed 909 fixtures and measured the three-candidate fallback shipping the floor where a genuinely better fit exists).

## The review finding (measured on the committed fixtures)

The reviewer's reference solver (same Platt targets + logit clip; init
`(w, c) = (0, ln((n⁺+1)/(n⁻+1)))`; Newton with Armijo backtracking)
reproduces production exactly on `massive_intent_en` (w=3.6342, loss 87.219)
and finds materially better optima on the two degenerate windows:

| window | stall loss (Issue 909) | **true MLE** (w, c, T) | loss | constant map (shipped by the 909 fallback) | identity |
|---|---|---|---|---|---|
| banking77 | 1051.02 | **8.674, 34.62, T=0.115** | **71.57** | 97.29 | 601.4 |
| xnli_en | 3151.38 | **0.193, 1.353, T=5.18** | **133.08** | 135.35 | 579.3 |
| massive_intent_en | 87.22 | 3.634, 8.744, T=0.275 | 87.22 | 136.6 | 242.8 |

banking77's raw AUC on the window is 0.85 — **the band carries real
z-usable signal**; the true MLE is a sharp T=0.115 map covering
[0.235, 0.9995]. The Issue-909 claim that the constant map is "the honest
answer when the window carries no z-usable signal" is FALSE for this data,
and shipping the floor by construction breaks the module's own
Report-the-Floor rule (G2: a calibrator must BEAT the constant base-rate
predictor — the 909 fallback makes it equal on the degenerate windows).

## The label correction (909's audit)

"The early-break theory refuted" is imprecise. What the f64 mirror refuted
is the PRECISION half (the stall is not f32-specific). The loop DOES break
at a non-minimum; the root cause is the **undamped full Newton step from
the identity start through a near-singular Hessian** — the known flaw in
Platt's original algorithm, fixed by Lin, Lin & Weng 2007 (*A note on
Platt's probabilistic outputs*, appendix A): the base-rate start point +
a backtracking (Armijo) line search. Wording to correct in: the HISTORY.md
909 entry, reflex Issue 056, the audit test's doc, the `fit_window` doc.

## The repair

1. `solve_smoothed_mle` becomes the LLW solve: init
   `(0, ln((n⁺+1)/(n⁻+1)))`, Newton step with Armijo backtracking
   (c=1e-4, β=0.5, ≤32 halvings), keeping the det-break (now safe — a
   degenerate Hessian at a damped, loss-monotone iterate is a legitimate
   stop), the non-finite guard, the step-norm tolerance, the iteration
   cap, and the `W_MIN` projection.
2. The Issue-909 three-candidate argmin + saturation guard stay as the
   NET underneath (with the LLW solver the argmin should be a no-op on
   the fixtures — verified); the guard remains the f32-tie invariant.
3. `COMMITMENT_VERSION` 1→2 — the same evidence window now produces
   different (correct) parameters; the bump is the honest signal.
4. Validation fixes demanded by the review:
   - audit assertion (c) becomes "production loss within ε of the f64
     LLW reference optimum" (not "≤ constant floor" — that locked the
     floor in as the target);
   - a held-out floor test: fit on half of each real window, log-loss +
     Brier on the other half vs the constant base-rate map fit on the
     same half (Report-the-Floor at the window level, honestly reported);
   - a synthetic property sweep (band width × base rate × n): the fitted
     loss never loses to the constant floor, no tie-collapse, monotone;
   - revert the accidental `y2` fixture flip in `metrics_known_vectors`
     (semantic no-op, unexplained diff).
5. Reflex re-measure (Q4 discipline): with real temperatures back
   (T=0.115 / 5.18), the threshold-scale mismatch class RETURNS — the
   `--gate-fit-calibrated` lever's founding question is live again. Land
   the solver, re-run the 15-suite re-baseline + the lever cells, THEN
   settle the lever posture (owner gate unchanged).

## Gates

Same matrix as 909: module tests, Bench 807, Bench 808, katgpt-claim,
bridge calibrated, clippy at the feature postures; then the reflex
re-baseline (Bench 095) + lever cells.
