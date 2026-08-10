# Provenance — Repo Release Verification

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W19** `repo-release-verification`.

## Stages

### `00-release-verification`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-007` | Before every git push, the drain test suite must run and pass; the push is blocked on failure unless the operator explicitly opts out with git push --no-verify. | `reference/sergeant-upstream/scripts/hooks/pre-push` (L2-11) |
| `BU-P6-008` | If the tooling required to run the pre-push validation (mise, docker) is unavailable, the hook fails closed with exit 1 and an actionable message, rather than silently skipping validation and letting the push through. | `reference/sergeant-upstream/scripts/hooks/pre-push` (L29-33, L35-39) |

## Notes

**Synthesis notes:** Survives §6.3 by name — it is the proposal's own worked example. Scoped as self-hosting behavior of the *source repository*, not a Sergeant-offered procedure other repositories would install.

