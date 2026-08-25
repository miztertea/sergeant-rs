# Sergeant workflow catalog

This is the present-tense catalog of every published package under `.sergeant/workflows/`. Each package's `index.md` owns its trigger, description, and stage summary. Drafts under `.sergeant/drafts/workflows/` are reviewable candidates, never admitted runnable procedure.

| Workflow | Status | Index |
|---|---|---|
| `author-document` | published | [`workflows/author-document/index.md`](workflows/author-document/index.md) |
| `fix-defect` | published | [`workflows/fix-defect/index.md`](workflows/fix-defect/index.md) |
| `implement-change` | published | [`workflows/implement-change/index.md`](workflows/implement-change/index.md) |
| `investigate` | published | [`workflows/investigate/index.md`](workflows/investigate/index.md) |
| `remediate-findings` | published | [`workflows/remediate-findings/index.md`](workflows/remediate-findings/index.md) |
| `review-change` | published | [`workflows/review-change/index.md`](workflows/review-change/index.md) |
| `validate-and-ship` | published | [`workflows/validate-and-ship/index.md`](workflows/validate-and-ship/index.md) |

## Embedded default

Omitting `--workflow` binds `software-change`: `00-prepare`, `10-implement`, `20-review`, and `30-close`. It is recorded with source `embedded` and is not a forkable package.

Selecting a named package or deliberately choosing the embedded default is Captain's responsibility. Operator skills under `skills/<name>/SKILL.md` are interactive Captain procedure, never dispatched Work.
