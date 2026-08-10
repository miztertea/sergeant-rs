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

## Helpers (folded per N1 adjudication A4)

This workflow originally ended in two further stages, `30-sync-repositories` and `40-report-state`. Both are demoted and fold into this stage (now the workflow's terminal stage) as helper invocations performed after registration/edit completes:

- **Sync repositories.** A missing required repository is synced only once the requested work actually requires it, via sgt-sync <project>; the workflow stops if cloning or pulling fails. sgt-sync runs only when repositories actually need cloning or refreshing. Syncing treats three distinct repo states differently: an already-cloned repo on a named branch is pulled fast-forward-only, an existing non-git directory is left untouched with a warning, and a missing repo with a configured URL is cloned. If a required repository entry has no clone URL or a required executable is missing, load-project stops naming the exact gap.
  — `BU-P5-095`, `BU-P5-102`, `BU-P6-013`, `BU-P6-014`, `BU-P5-109`, `BU-P5-110`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 28-29, 47, 73, 74), `reference/sergeant-upstream/bin/sgt-sync` (L30-45, L33-39)
- **Report state.** A read-only per-repo report of clone/branch/cleanliness/ahead-behind status, plus a filtered, unified view of open tracked work across the project's repos (repos without an initialized task database are silently skipped rather than erroring the whole listing).
  — `BU-P6-012`, `BU-P6-035`, `reference/sergeant-upstream/bin/sgt-status` (L1-2), `reference/sergeant-upstream/bin/sgt-td-list` (L2-13)

`30-sync-repositories` carried no argument beyond the §6.5 boilerplate. `40-report-state` carried an "Additional note" ("Borderline per synthesis.md (closer to a query than a checkpoint); kept as a stage because operators do care whether it succeeded before planning") that was weighed against §6.3's reimplementation test and failed: swapping the status/listing implementation would leave nothing about the surrounding checkpoint changed, and the note's own framing ("closer to a query than a checkpoint") concedes the point. See `provenance.md`'s "Adjudication A4" section.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
