# Issue 904 — `fit_woodbury` panics (cholesky_f32 assert) on an indefinite f32 sample Gram at small λ — a runtime fit path with no warn-and-keep

**Status:** OPEN 2026-09-28 — filed from the riir-ai Proposal 048 T8 GOAT's first run (measured specimen below); riir-ai engine-side mitigation landed same day (see "Mitigation landed").

## The defect

`KarcForecaster::fit_woodbury` (`src/karc/mod.rs` — the `d_h > N` branch of
`fit_ridge`) builds the N×N sample Gram `X·Xᵀ` in **f32** (`simd_dot_f32`)
and factorizes it via `cholesky_f32` (`src/linalg/ridge_solve.rs:187`).
`cholesky_f32` **asserts** (`panic!`) when a pivot is below
`-tol` — it clamps only *near*-singular pivots. On a genuinely indefinite
f32 Gram (accumulation loss at long dot products + small λ), the fit
**panics instead of returning `FitError`**.

Every caller of `tick_karc` (riir-engine `karc_bridge`) wraps `fit_ridge`
in a `match` whose `Err` arm is `log::warn!("fit_ridge failed …; keeping
previous fit")` — the documented degrade-and-keep contract for "a
degenerate λ or singular Gram". The panic bypasses that contract entirely:
one bad fit kills the whole tick loop.

## Measured specimen (2026-09-28)

`NpcKarcState` promoted to `KarcLodTier::Lod2` (d_h=512) under the **default**
`HlaKarcConfig` (`lambda=1e-4`, `min_samples_before_fit=HLA_KARC_D_H=256` —
typed for Lod1's d_h, per its own field doc "must be ≥ d_h to keep
`fit_ridge` on the f64 direct path (the f32 Woodbury path panics at small
λ)"). The NPC crosses the fit gate at N=256 < 512 → Woodbury → first fit:

```
panicked at crates/katgpt-core/src/linalg/ridge_solve.rs:214:13:
matrix not positive definite in cholesky_f32 (pivot -37.95 < -0.0078)
```

through the REAL riir-engine tick path (the Proposal 048 T8 GOAT,
`riir-games-civ/tests/karc_tier_gate_goat.rs` `g2_chain_cold_pauses_and_combat_promotes_to_lod2`).
Never fired before because no production consumer promoted to Lod2
(`karc_lod_dispatch`'s civ-side wiring was per-NPC manual, unused) and no
lane executed the engine karc tests (the Layer-1.9e class; riir-ai's guard
1.9f row now does).

The engine test `lod2_observe_fit_forecast_roundtrip` passes because it
fits with λ=1e-2 and a small sample count — strong regularization keeps the
f32 Gram definite. The fast-config pattern in `karc_bridge`'s own tests
(`fast_karc_config`: min_samples 32) likewise pairs with `lambda=1e-2`.
The unsafe combination is exactly the DEFAULT λ with a below-d_h sample
count.

## Mitigation landed (riir-ai, same day)

riir-engine `tick_karc` now derives the fit floor from the ACTIVE tier:
`effective_min_samples = max(config.min_samples_before_fit, tier.d_h())`
(riir-ai commit — see the Proposal 048 T8 landing note). The invariant the
default config documents is now ENFORCED at the one seam every tier flows
through, for every consumer. That makes the Lod2-under-default-config
specimen impossible; it does NOT fix the underlying panic class.

## Requested upstream hardening (this repo owns `fit_woodbury`/`cholesky_f32`)

Options, cheapest first:
1. **`fit_woodbury` degrades**: wrap the Cholesky in a catch-free check —
   e.g. attempt the factorization, and on assertion failure return
   `FitError::Singular`. Requires `cholesky_f32` to expose a non-panicking
   variant (`try_cholesky_f32 -> Result<(), NotPositiveDefinite>`) — the
   panicking form stays for callers that want the assert (tests).
2. **f64 sample Gram** when `k` (sample count) is small — the comment
   already says "sample count is small in this regime, so f32 precision
   suffices"; the specimen shows it doesn't, always. f64 at N ≤ 512 doubles
   the Gram memory (N²×8B = 2 MiB at N=512) and the dot cost — acceptable
   off the per-tick path (fits are tau_reest-cadenced).
3. Document-only: widen `HlaKarcConfig::min_samples_before_fit`'s doc to
   name the per-tier invariant — weakest; conventions are exactly what
   failed here (the doc existed and the Lod2 tier bypassed it).

Option 1 is the contract-correct one: `tick_karc` already has the
warn-and-keep `Err` arm — the fit path just needs to be ALLOWED to return
`Err`.
