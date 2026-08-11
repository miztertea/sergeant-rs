# Sergeant workflow catalog

Root catalog (`docs/icm/convention.md` §1 rule 1; `docs/icm/record-shapes.md`
§1 rule 2). Lists every `status: published` workflow under
`.sergeant/workflows/`. This file is the list, not an entry — it carries no
`kind: workflow` front matter of its own, and no per-workflow authored field
(description, tags) is duplicated here beyond a short pointer; each
workflow's own `index.md` is the source for those (`docs/icm/convention.md`
§3 rule 2, the same anti-duplication rule that governs `AGENTS.md`).

| Workflow | Status | Index |
|---|---|---|
| `code-review` | published | [`workflows/code-review/index.md`](workflows/code-review/index.md) |
| `deepen-module` | published | [`workflows/deepen-module/index.md`](workflows/deepen-module/index.md) |
| `diagnose-bug` | published | [`workflows/diagnose-bug/index.md`](workflows/diagnose-bug/index.md) |
| `drain-fleet` | published | [`workflows/drain-fleet/index.md`](workflows/drain-fleet/index.md) |
| `load-project` | published | [`workflows/load-project/index.md`](workflows/load-project/index.md) |
| `monitor-fleet` | published | [`workflows/monitor-fleet/index.md`](workflows/monitor-fleet/index.md) |
| `project-graph` | published | [`workflows/project-graph/index.md`](workflows/project-graph/index.md) |
| `prototype` | published | [`workflows/prototype/index.md`](workflows/prototype/index.md) |
| `reconcile-and-cleanup-fleet` | published | [`workflows/reconcile-and-cleanup-fleet/index.md`](workflows/reconcile-and-cleanup-fleet/index.md) |
| `recover-stalled-worker` | published | [`workflows/recover-stalled-worker/index.md`](workflows/recover-stalled-worker/index.md) |
| `repo-to-icm` | published | [`workflows/repo-to-icm/index.md`](workflows/repo-to-icm/index.md) |
| `research` | published | [`workflows/research/index.md`](workflows/research/index.md) |
| `resolving-merge-conflicts` | published | [`workflows/resolving-merge-conflicts/index.md`](workflows/resolving-merge-conflicts/index.md) |
| `respond-to-worker` | published | [`workflows/respond-to-worker/index.md`](workflows/respond-to-worker/index.md) |
| `route-review-findings` | published | [`workflows/route-review-findings/index.md`](workflows/route-review-findings/index.md) |
| `to-spec` | published | [`workflows/to-spec/index.md`](workflows/to-spec/index.md) |
| `vet-external-skill` | published | [`workflows/vet-external-skill/index.md`](workflows/vet-external-skill/index.md) |

`.sergeant/drafts/workflows/` holds generated, human-reviewable candidates —
never admitted procedure, never listed here (`docs/icm/convention.md` §2).
