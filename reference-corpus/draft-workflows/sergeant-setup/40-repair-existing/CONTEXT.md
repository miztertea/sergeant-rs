# 40-repair-existing: repair existing

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../30-project-interview/output/README.md | L4 | upstream artifact produced by `30-project-interview` |

## Purpose

An existing definition is validated, a minimal diff shown, and changes applied only after confirmation with a mandatory post-confirmation backup.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

An existing definition is validated, a minimal diff shown, and changes applied only after confirmation with a mandatory post-confirmation backup.

## Behavior contract

- **Phase 5 (repair) first validates the existing project YAML with yq; if validation fails, it reports the parse error and stops without proceeding.**
  (trigger: the user wants to modify an existing project YAML; outcome: repair never proceeds against a file whose current state is not understood)
  — `BU-P5-028`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 204-205)
- **Phase 5 computes and displays a minimal diff between the current file content and the proposed changes before any write.**
  (trigger: the existing YAML validates; outcome: the user can see exactly what would change before approving it)
  — `BU-P5-029`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 206-207)
- **Phase 5 requires explicit confirmation before any write or backup; only after confirmation does it create a timestamped backup and then write the new content. The backup is never created before confirmation, changes are never applied on decline, and the backup is mandatory whenever a write happens -- it is never skipped even if asked.**
  (trigger: a diff has been shown for an existing project YAML; outcome: every applied edit is preceded by both explicit consent and a backup, in that order, with no exception)
  — `BU-P5-030`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 208-216)
- **Backup of an existing config file must be created only AFTER the user confirms applying changes, never before; the skill must not instruct pre-confirmation backup creation even framed as a safety measure.**
  (trigger: sergeant-setup is about to modify an existing project YAML; outcome: consent strictly precedes any filesystem side effect, including a safety backup, preserving a clean 'nothing happened until yes' guarantee)
  — `BU-P7-038`, `reference/sergeant-upstream/tests/sergeant-setup-test.sh` (lines 116-121)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

Conflict X7 (synthesis.md §6): P5's own Phase 4 backs up *before* writing while Phase 5 and BU-P7-038 require the backup only *after* confirmation. This stage follows the confirm-then-backup rule as the adjudicated order (later phase, corroborated by a test) but the earlier-phase citation is preserved in provenance.md as unresolved evidence, not erased.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
