# Issue 915 — two_fidelity_bai: certified root intervals systematically exclude V* on wide-envelope fixtures (Bench 915 / Plan 615 T2)

**Status:** OPEN — G1 certificate-validity FAIL, Plan 615 Phase 2 GOAT gate held; module stays opt-in (already is).

**Discovered by:** `benches/bench_615_two_fidelity_bai_goat.rs` (Plan 615 Phase 2, T2.1–T2.3) on the paper's own fixture family — balanced b-ary stochastic minimax trees with the CONTRACT-honest envelope `B(h) = 0.12·(1 − 2^−h)` (B(0) = 0, leaf fast values exact, verified node-by-node: max |bias| ≤ envelope everywhere).

## Evidence (40 trees/setting shakedown, the shipped fixture family)

- **(D=5, b=8): 24 of 40 trees** carry CERTIFIED root intervals whose
  effective endpoints exclude the true minimax value — every miss is
  `true > hi` (the interval caps below V*), by 0.013–0.078. Sample:
  `t7 node 2: certified [0.649988, 0.756171], true 0.769402`; `t7 node 6:
  certified [0.675626, 0.746762], true 0.824594`. Multiple root children
  per tree miss simultaneously (t7: nodes 2–7 all invalid).
- **(D=7, b=6): same shape** from the first trees (t0: nodes 1–5 invalid;
  t1, t9: same).
- **Guard-fires blow the δ budget the same way:** 24/40 trees at (5,8)
  terminate UNCERTIFIED (empty-intersection guard, B.3 default action) —
  the detectable-E_δ path fires at ~60% vs the δ = 0.05 budget.
- 2FFS pick errors at (5,8): 6/40 vs ε = 0.02 — CP95 upper ≫ δ.
- The two failure classes co-occur per tree (a guard-fired tree and a
  certified-invalid tree are both `fast ∩ backup = ∅ or ⊄ V*` events).

## Fixture honesty (verified, not assumed)

Node-by-node numeric check on the failing trees: `max |fast − V*| −
envelope ≤ 0` holds for EVERY node (e.g. t7 node 6: bias −0.1112 vs
envelope 0.1125 ✓). Slow samples are unbiased `μ ± 0.05` with declared
σ = 0.05 ≥ the uniform's 0.05/√3 (loose-honest). Ground truth is an f64
exact minimax fold; interval misses (0.013–0.078) are orders above f32
representation noise (1e-7).

## Why the module tests never saw it

The in-module PAC smoke (`random_small_trees_recommend_epsilon_optimal_roots`)
uses a **0.3 root gap** (child-0 subtree ~0.8 vs others ≤ 0.4) and a
**DECREASING** envelope (`bias_scale·2^−h` — loosest at LEAVES; its doc
comment claims "B(0) = 0" but B(0) = bias_scale ≠ 0 — the fixture
contradicts the trait contract it sits beside). Under gaps that large,
lazy resolution never has a chance to disagree with the truth; the paper's
fixture family (uniform leaf means, root-side envelope mass) is where the
divergence lives.

## Working hypothesis (to be confirmed by the owning fix session)

The interval machinery is containment-preserving by induction IF every
backed-up interval contains its node's V* and the Eq. 6 fold runs over ALL
children (expand reveals all children and refresh_backup folds all present
children — valid). The observed `true > hi` at MIN root children says some
effective `hi` in the fold landed below the true min — i.e. an interval
derived from the ρ_k/2 lazy-discharge / blocking-child recursion was used
as if it were a full Eq. 6 child interval. The T1.5 landing's recorded v1
divergence ("the paper's all-live-children recursion is the recorded v1
divergence, cost-character only, PAC never weakened") is the natural
suspect: on gapless uniform fixtures the recursion discharges children
whose later refinement would have RAISED the min — the discharged child's
stale low endpoint caps the parent's `hi` below V*, and Lemma 2.3 then
propagates the too-low endpoint upward into a certified-miss. Confirm or
replace this paragraph with the real mechanism when fixing — the bench
reproduces deterministically per seed (t7 = seed 0x615_0007 at (5,8)).

## What this is NOT

- NOT a bench-fixture bug (honesty verified numerically, above).
- NOT an ε artifact (reproduces at ε = 0.02 and 0.1).
- NOT a G2 finding (BAI-MCTS/UCT comparisons carry their own estimates;
  their error rates are reported separately by the bench).

## Gate state

Plan 615 T2.5 verdict: **GOAT FAIL — the certificate body (the product of
the primitive) is invalid on the paper's own fixture family.** G2 paired
wins were observed in the same runs but are MOOT while G1 fails (a
certified wrong answer is not a result). The module stays opt-in (it never
left opt-in). Re-arm: fix the containment defect, then re-run
`bench_615_two_fidelity_bai_goat` — the G1 arm (CP95 ≤ δ on picks,
certified-interval validity, guard-fires within the δ budget) is the
acceptance gate; the bench's certificate-validity diagnostic prints the
failing (tree, node, interval, truth) tuples deterministically per seed
(`B615_DEBUG=1` adds per-node vstar/fast/bias/envelope dumps).

En-route observation filed in the same investigation: the module's VecTree
fixture envelope contradicts the trait contract's B(0) = 0 (decreasing
where the contract demands nondecreasing-from-zero) — the fixture should
move to the contract shape when this issue is fixed so the smoke exercises
the real regime.
