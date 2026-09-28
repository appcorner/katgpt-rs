# Owner-gate pickup — katgpt-rs

**Status:** OPEN — this repo's remaining owner-gated items. Public-repo hygiene note: the cross-workspace summary that briefly lived here as `905_workspace_owner_gated_decisions_summary.md` (2026-09-28) was relocated to the private workspace hub the same day and is deliberately NOT referenced from this public repo — no private paths, hashes, or decision content belong here. Number 905 is SPENT — never reuse it.

## Items

- [ ] E12 — base-model recirculation check blocked on Gemma license + HF token (evidence `.benchmarks/668:71`): prep a one-pager with the two options — (a) accept the Gemma license + provision an HF token to the 4090 env, or (b) retire the check as permanently blocked (IT-only evidence stands) — for the owner to pick.
- [ ] E13 — workspace `rust-version` pin (evidence `HISTORY.md:3148`): prep the `[workspace.package] rust-version` patch + the list of ~30 affected manifests; land only on owner go.
