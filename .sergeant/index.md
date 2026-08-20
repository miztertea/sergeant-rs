# Sergeant workflow catalog

Root catalog (`docs/icm/convention.md` §1 rule 1; `docs/icm/record-shapes.md`
§1 rule 2). Lists every `status: published` workflow under
`.sergeant/workflows/`. This file is the list, not an entry — it carries no
`kind: workflow` front matter of its own, and no per-workflow authored field
(description, tags) is duplicated here beyond a short pointer; each
workflow's own `index.md` is the source for those (`docs/icm/convention.md`
§3 rule 2, the same anti-duplication rule that governs `AGENTS.md`).

18 packages (up from 17 at the 2026-08-20 wave-4 backlog close-out, which
authored and published `validate-intent` (#201); down from 35 at the
MVP-5 F2 execution-surface re-triage, 2026-08-12; down from 23 at ICM-R2,
2026-08-16; down from 20 at ICM-R3, 2026-08-16). Prior retirements: 12 — 9 CLI-SURFACE, 1 OPERATOR-SKILL, and
the 2 R-NS-6-dissolved `grilling`/`grill-with-docs` — to `skills/`
(operator skills, this repository's canonical skill root) or to
`docs/icm/re-homing-record-2026-08-12.md` (CLI-verb candidates and engine
gaps). ICM-R2 retirements (`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r2/`,
`docs/adr/0013-icm-r0-owner-rulings.md`): `task-intake-and-route`
(ABSORBED — every behavior already duplicated on an already-published
surface), `sergeant-setup` (SPLIT — folded into
`skills/estate-navigation/SKILL.md` and `AGENTS.md`'s Guardrails),
`direct-implementation` (HARVEST — folded into `AGENTS.md`'s "When NOT to
use `sgt`"). ICM-R3 retirements (`sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r3/`, all 16
remaining unreconciled packages processed): `tdd` (REHOME — folded into
`.sergeant/common/contexts/tdd.md` and `test-quality.md` as shared
contexts, since it named only two stages and was a technique, not a
workflow), `to-spec` (REHOME — folded into `skills/to-spec/SKILL.md`,
since its defining behavior synthesizes from the current conversation, a
dependency a dispatched Work cannot receive), `load-project` (ABSORBED —
every behavior already duplicated on `estate-navigation`). The other 13
ICM-R3 packages (`implement`, `worker-mission`, `diagnose-bug`,
`prototype`, `deepen-module`, `dispatch`, `triage`, `wayfinder`,
`resolving-merge-conflicts`, `vet-external-skill`, `recover-stalled-
worker`, `cross-repo-work`, `to-tickets`) all reached STAND, gaining the
Authority-envelope/Bounded-judgment doctrine plus per-package in-place
fixes; see each package's own `sergeant-rs-workspace/knowledge/evidence/gauntlet/runs/icm-r3/<pkg>/
adjudication-draft.md` for its diff. Provenance for every retired package
is preserved in git history.

| Workflow | Status | Index |
|---|---|---|
| `code-review` | published | [`workflows/code-review/index.md`](workflows/code-review/index.md) |
| `cross-repo-work` | published | [`workflows/cross-repo-work/index.md`](workflows/cross-repo-work/index.md) |
| `deepen-module` | published | [`workflows/deepen-module/index.md`](workflows/deepen-module/index.md) |
| `diagnose-bug` | published | [`workflows/diagnose-bug/index.md`](workflows/diagnose-bug/index.md) |
| `dispatch` | published | [`workflows/dispatch/index.md`](workflows/dispatch/index.md) |
| `implement` | published | [`workflows/implement/index.md`](workflows/implement/index.md) |
| `prototype` | published | [`workflows/prototype/index.md`](workflows/prototype/index.md) |
| `recover-stalled-worker` | published | [`workflows/recover-stalled-worker/index.md`](workflows/recover-stalled-worker/index.md) |
| `repo-to-icm` | published | [`workflows/repo-to-icm/index.md`](workflows/repo-to-icm/index.md) |
| `research` | published | [`workflows/research/index.md`](workflows/research/index.md) |
| `resolving-merge-conflicts` | published | [`workflows/resolving-merge-conflicts/index.md`](workflows/resolving-merge-conflicts/index.md) |
| `to-tickets` | published | [`workflows/to-tickets/index.md`](workflows/to-tickets/index.md) |
| `triage` | published | [`workflows/triage/index.md`](workflows/triage/index.md) |
| `validate-and-ship` | published | [`workflows/validate-and-ship/index.md`](workflows/validate-and-ship/index.md) |
| `validate-intent` | published | [`workflows/validate-intent/index.md`](workflows/validate-intent/index.md) |
| `vet-external-skill` | published | [`workflows/vet-external-skill/index.md`](workflows/vet-external-skill/index.md) |
| `wayfinder` | published | [`workflows/wayfinder/index.md`](workflows/wayfinder/index.md) |
| `worker-mission` | published | [`workflows/worker-mission/index.md`](workflows/worker-mission/index.md) |

`.sergeant/drafts/workflows/` holds generated, human-reviewable candidates —
never admitted procedure, never listed here (`docs/icm/convention.md` §2).

Operator skills (never dispatched as Work; loaded directly by the harness)
live at `skills/<name>/SKILL.md`: `sergeant-help`, `grilling`,
`grill-with-docs`, `estate-navigation`, `to-spec` (added at ICM-R3, REHOME
from the retired `to-spec` workflow — its defining behavior synthesizes
from the current conversation, which a dispatched Work cannot receive).
