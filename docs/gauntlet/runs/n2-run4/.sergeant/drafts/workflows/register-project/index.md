---
kind: workflow
name: register-project
status: draft
version: 1
description: >-
  Trigger: a new project YAML is being registered.
  Outcome: the project file satisfies all six named field-shape requirements.
  Completion: The full six/five-part procedure named in the statement above has been executed, per `BU-0131`.
tags:
  - draft
  - repo-to-icm-n2-run4
---

# register-project

**Single-behavior workflow candidate**, established entirely by `BU-0131`
(`docs/getting-started.md (docs/getting-started.md L102-110)`) — no `stage`/`stage-context`/`helper` record in this
corpus carries a `workflow` field matching `register-project` (checked against
that source file's other extracted units and against every `workflow`
field value used corpus-wide). Reported as a single-behavior candidate
per synthesis-method.md's "what must not happen" rather than padded out
or merged into an adjacent-topic cluster it does not actually name.

**Trigger:** a new project YAML is being registered

**Outcome:** the project file satisfies all six named field-shape requirements

**Completion condition:** the full six/five-part procedure named in the statement above has been executed, per `BU-0131`.

**Ordering:** stages run in the fixed pipeline order listed in `workflow.toml` / `CONTEXT.md`.

See `CONTEXT.md` for workflow orientation and `provenance.md` for the source evidence trace back to `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson` (behavior_id citations resolved via `.sergeant/workflows/repo-to-icm/50-synthesize/output/candidates.md`).
