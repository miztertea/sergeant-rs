# Workflow catalog

Named workflows are published packages. Their executable source under `.sergeant/workflows/` remains authoritative; these pages explain selection.

| Workflow | Use it for | Avoid it for |
|---|---|---|
| `author-document` | evidence-grounded document creation | conversational ideation |
| `fix-defect` | reproduce, repair, and verify a defect | unbounded investigation |
| `implement-change` | bounded product/code change with review | a read-only question |
| `investigate` | answer a bounded question with evidence | shipping a known implementation |
| `remediate-findings` | resolve an existing typed finding set | discovering findings from scratch |
| `review-change` | independently review an existing change | implementing the original change |
| `validate-and-ship` | final validation and shipping evidence | changing acceptance criteria |

Omitting `--workflow` binds the embedded `software-change` loop (`00-prepare`, `10-implement`, `20-review`, `30-close`). It is embedded, not a forkable package. Drafts under `.sergeant/drafts/workflows/` are not admitted procedure.

Each stock package's [source index](../../.sergeant/index.md) links to its current stage summary.
