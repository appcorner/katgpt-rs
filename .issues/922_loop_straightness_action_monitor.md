# Issue 922: Loop-straightness monitor family — min-action floor, η efficiency ratio, chord-projection stalemate signal (DEC-native, modelless)

**Status:** Open — fusion idea, novelty TBD. Signal-diff REQUIRED against the existing stalemate/collapse signals (`cgsp_runtime` collapse bridge, `swarm/deliberation.rs` cadence churn probes, `latent_functor/reestimation` settling classes, `self_evolve` trajectory outcomes) before ANY implementation lands; consumers ride riir-ai Issue 1037.
**Date:** 2026-10-07
**Research:** [riir-train/.research/468_LiFT_Staged_Depth_Target_Recipe.md](../../riir-train/.research/468_LiFT_Staged_Depth_Target_Recipe.md) (arXiv:2610.05538 LiFT, Path 0 rows 8/9/12 — the modelless residue)
**Source:** LiFT appendix C closed-form analysis (min-action identity, chord-projection diagnostic, grid-invariance) — extractable WITHOUT the paper's trained objective.
**Related:** katgpt-dec `stokes_calculus.rs` (`line_integral`, `boundary_flux_mass` — the landing substrate) · Plan 304 GainCostLoopHalter + Issue 898 loop-knob instruments (adjacent budget axes, different signals) · riir-ai Issue 1037 (game-side consumers) · riir-refine `self_evolve` (fix-trajectory consumer) · riir-reflex cascade (escalation-vs-depth consumer)

## The primitive (closed form, zero-alloc, O(K·d))

For any iterative refinement trajectory `x_0 → x_1 → … → x_K` with deltas `Δ_k = x_{k+1} − x_k` on grid weights `δ_k` (Σδ_k = 1):

```
A_disc = ½·Σ_k ‖Δ_k‖²/δ_k  ≥  ½·‖x_K − x_0‖²      (Cauchy–Schwarz floor — LiFT appendix C.3)
η      = ‖x_K − x_0‖² / Σ_k ‖Δ_k‖²/δ_k  ∈ (0, 1]   (action efficiency; η = 1 iff straight constant-speed path)
```

- `η = 1` exactly for a synthetic straight trajectory (property test by construction — the d∘d=0-class gate).
- `1 − η` is the exact wasted-motion fraction (backtracking, orbiting, A/B/A oscillation).
- Chord-projection residual (LiFT's β_k measurement, learned-property → runtime-assertion flip): `β_k = ⟨x_k − x_0, x_K − x_0⟩/‖x_K − x_0‖²` tracked against `k/K` offline; live form projects each new delta onto the RUNNING chord, orthogonal-residual fraction `r_k` feeds a sigmoid drive `σ(λ·(EMA(r) − τ))` — **the house "stalemate is a drive, never a counter" law fed by a geometric signal instead of a tick counter**.
- Grid-invariant: the weighted action's floor holds on ANY nonuniform grid (knot-wise minimizer), so a nonuniform deliberation schedule admits the same diagnostic.
- Boundary-vs-interior pattern: the floor reads only two endpoints; the cost samples K interior knots — the summation axis is K, not latent dim, so no curse of dimensionality.

## Landing

`katgpt-rs/crates/katgpt-dec/src/stokes_calculus.rs` — `path_action` / `action_efficiency` helpers beside the existing `line_integral`/`boundary_flux_mass` family; pure functions over a trajectory log the runtime already traverses.

## Consumers (each needs its own signal-diff before wiring)

| Consumer | Signal it replaces/augments | Repo |
|---|---|---|
| Deliberation cadence (stalemate → re-plan) | tick-count churn probes | riir-ai Issue 1037 |
| cgsp MCTS collapse bridge | collapse detection from orthogonal-residual growth | riir-ai |
| riir-refine `self_evolve` heal trajectories | a heal pass that edits code back and forth scores η ≪ 1 | riir-refine |
| riir-reflex cascade | escalation-vs-depth straightness | riir-reflex |

## GOAT gate sketch

- **G1:** synthetic straight trajectory returns η = 1.0 bit-exact; adversarial A/B/A oscillation fixture scores below a measured threshold; nonuniform-grid knot identity (floor holds at every refinement).
- **G2:** sub-µs per K=8 trajectory in `--release`.
- **G4:** alloc-free (counting allocator canary).
- Feature flag + kill-switch per house convention. Full riir-poc head-to-head required ONLY if a quality-parity claim against an incumbent stalemate detector is later made (§3.6).

## Explicit non-goals

- The Hodge extension (coexact = oscillation energy) is INTERPRETIVE ONLY — it needs a `CellComplex` state-graph substrate that does not exist at trajectory granularity; do not file without pricing that substrate.
- No training surface — the trained LiFT objective lives in riir-train Plan 395 arm A, not here.
