# Sergeant workflow catalog

Root catalog (`docs/icm/convention.md` §1 rule 1; `docs/icm/record-shapes.md`
§1 rule 2). Lists every `status: published` workflow under
`.sergeant/workflows/`. This file is the list, not an entry — it carries no
`kind: workflow` front matter of its own, and no per-workflow authored field
(description, tags) is duplicated here beyond a short pointer; each
workflow's own `index.md` is the source for those (`docs/icm/convention.md`
§3 rule 2, the same anti-duplication rule that governs `AGENTS.md`).

23 packages (down from 35 at the MVP-5 F2 execution-surface re-triage,
2026-08-12): 12 retired — 9 CLI-SURFACE, 1 OPERATOR-SKILL, and the 2
R-NS-6-dissolved `grilling`/`grill-with-docs` — to
`skills/` (operator skills, this repository's canonical skill root) or to
`docs/icm/re-homing-record-2026-08-12.md` (CLI-verb candidates and engine
gaps). Provenance for every retired package is preserved in git history.

| Workflow | Status | Index |
|---|---|---|
| `code-review` | published | [`workflows/code-review/index.md`](workflows/code-review/index.md) |
| `cross-repo-work` | published | [`workflows/cross-repo-work/index.md`](workflows/cross-repo-work/index.md) |
| `deepen-module` | published | [`workflows/deepen-module/index.md`](workflows/deepen-module/index.md) |
| `diagnose-bug` | published | [`workflows/diagnose-bug/index.md`](workflows/diagnose-bug/index.md) |
| `direct-implementation` | published | [`workflows/direct-implementation/index.md`](workflows/direct-implementation/index.md) |
| `dispatch` | published | [`workflows/dispatch/index.md`](workflows/dispatch/index.md) |
| `implement` | published | [`workflows/implement/index.md`](workflows/implement/index.md) |
| `load-project` | published | [`workflows/load-project/index.md`](workflows/load-project/index.md) |
| `prototype` | published | [`workflows/prototype/index.md`](workflows/prototype/index.md) |
| `recover-stalled-worker` | published | [`workflows/recover-stalled-worker/index.md`](workflows/recover-stalled-worker/index.md) |
| `repo-to-icm` | published | [`workflows/repo-to-icm/index.md`](workflows/repo-to-icm/index.md) |
| `research` | published | [`workflows/research/index.md`](workflows/research/index.md) |
| `resolving-merge-conflicts` | published | [`workflows/resolving-merge-conflicts/index.md`](workflows/resolving-merge-conflicts/index.md) |
| `sergeant-setup` | published | [`workflows/sergeant-setup/index.md`](workflows/sergeant-setup/index.md) |
| `task-intake-and-route` | published | [`workflows/task-intake-and-route/index.md`](workflows/task-intake-and-route/index.md) |
| `tdd` | published | [`workflows/tdd/index.md`](workflows/tdd/index.md) |
| `to-spec` | published | [`workflows/to-spec/index.md`](workflows/to-spec/index.md) |
| `to-tickets` | published | [`workflows/to-tickets/index.md`](workflows/to-tickets/index.md) |
| `triage` | published | [`workflows/triage/index.md`](workflows/triage/index.md) |
| `validate-and-ship` | published | [`workflows/validate-and-ship/index.md`](workflows/validate-and-ship/index.md) |
| `vet-external-skill` | published | [`workflows/vet-external-skill/index.md`](workflows/vet-external-skill/index.md) |
| `wayfinder` | published | [`workflows/wayfinder/index.md`](workflows/wayfinder/index.md) |
| `worker-mission` | published | [`workflows/worker-mission/index.md`](workflows/worker-mission/index.md) |

`.sergeant/drafts/workflows/` holds generated, human-reviewable candidates —
never admitted procedure, never listed here (`docs/icm/convention.md` §2).

Operator skills (never dispatched as Work; loaded directly by the harness)
live at `skills/<name>/SKILL.md`: `sergeant-help`, `grilling`,
`grill-with-docs`, `estate-navigation`.
