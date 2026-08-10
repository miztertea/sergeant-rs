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
| `repo-to-icm` | published | [`workflows/repo-to-icm/index.md`](workflows/repo-to-icm/index.md) |

`.sergeant/drafts/workflows/` holds generated, human-reviewable candidates —
never admitted procedure, never listed here (`docs/icm/convention.md` §2).
