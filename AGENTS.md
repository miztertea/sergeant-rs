This repository uses Sergeant for durable procedural work.

- Discover available procedures in `.sergeant/index.md`.
- Select an admitted workflow explicitly when substantive work begins.
- Follow only the active stage context supplied by Sergeant.
- Resolve `@@name` references from `.sergeant/common/contexts/name.md`.
- Treat `.sergeant/common/scripts/` and workflow-local scripts as helpers,
  not independent procedure unless the workflow declares a durable stage.
- Do not treat `.sergeant/drafts/workflows/` as published procedure.
- Use Sergeant's respond, retry, cancel and inspection surfaces rather than
  fabricating workflow state in prose.
