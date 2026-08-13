# 20-register-or-edit: register or edit

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-resolve-context/output/README.md | L4 | upstream artifact produced by `10-resolve-context` |

## Purpose

A project definition is written to the Sergeant-owned config path and validated, or the prior definition is restored.

Trigger (workflow-level): A project is named, registered, edited, synced, or listed; or repository ownership is not already established.

## What must become true here (durable outcome)

A project definition is written to the Sergeant-owned config path and validated, or the prior definition is restored.

## Behavior contract

- **Registering or editing a project starts by reading docs/schema.md and, when editing, the existing YAML.**
  (trigger: the user asks to add or change a project; outcome: the schema and any existing content are read before any change is proposed)
  — `BU-P5-097`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 36-38)
- **Project YAML is written only to ~/.config/sergeant/<project>.yaml, and credentials, tokens, or secret values are never placed in it.**
  (trigger: a project YAML is being written; outcome: project config never becomes a place secrets are stored)
  — `BU-P5-098`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 39-40)
- **Repository paths in project YAML are always either absolute or relative to the global dev_root.**
  (trigger: a repository entry is being written; outcome: repository paths are always resolvable without ambiguity)
  — `BU-P5-099`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 41)
- **After writing project YAML, load-project runs sgt-list and requires the project to appear exactly once, then runs sgt-context <project> and requires every edited field needed by agents to appear in the resolved output.**
  (trigger: project YAML has been written or edited; outcome: the write is verified end-to-end, not merely assumed to have parsed)
  — `BU-P5-101`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 44-46)
- **If validation fails after a registration/edit, load-project restores the prior YAML or leaves the new file uncommitted, and reports the exact command error.**
  (trigger: post-write verification fails; outcome: a failed edit never leaves the project in a half-written, unverified state)
  — `BU-P5-103`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 48-49)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Retired helper content (execution-surface re-triage, MVP-5 F2, 2026-08-12)

This stage previously folded `30-sync-repositories`/`40-report-state`
(demoted machinery, N1 adjudication A4) as helper invocations covering
`list-projects`, `project-status`, `project-sync`, and `project-task-list`.
The §2a execution-surface test's SPLIT verdict
(`docs/icm/retriage-2026-08-11.md`) found those four to be command surfaces,
not procedures with a bounded outcome (§6.2) — CLI-SURFACE, not workflow
content — and they retired to `docs/icm/re-homing-record-2026-08-12.md`
rather than staying here. `sgt repo list` + `sgt doctor` answer the
status/listing half today; `sgt repo add <name> --origin <url>` (clone if
missing, verify if present) answers most of the sync half — see the
`estate-navigation` skill for the honest remaining gap (no bulk "pull
existing repos" verb exists yet). The register/edit judgment above this
section is unaffected; only the folded machinery moved.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
