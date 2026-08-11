# 30-project-interview: project interview

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../20-global-config/output/README.md | L4 | upstream artifact produced by `20-global-config` |

## Purpose

A complete project definition is captured from the user, previewed in full, and written only after confirmation.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

A complete project definition is captured from the user, previewed in full, and written only after confirmation.

## Behavior contract

- **If the project YAML already exists and the user wants to modify it, Phase 4 (the new-project interview) is skipped entirely in favor of Phase 5 (repair existing YAML).**
  (trigger: a project YAML already exists; outcome: the new-project interview never runs against an existing project)
  — `BU-P5-023`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 166-167)
- **The new-project interview asks, in strict order and waiting for each answer before proceeding: project name (YAML filename stem, must match [a-z0-9_-]+), per-repository name/path/clone-URL/role/group, per-group description and shared agent_instructions, default agent instructions applied to every repository, project-level GitHub identity, and an optional Graphify output path.**
  (trigger: a new project is being registered; outcome: all information needed to write a complete project YAML has been collected in a fixed order)
  — `BU-P5-024`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 169-184)
- **Before writing the project YAML, sergeant-setup shows a complete preview of the file content and requires explicit confirmation; the file is written only after confirmation.**
  (trigger: all interview answers are collected; outcome: the project YAML is written only after the user has seen and approved the exact content)
  — `BU-P5-026`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 186-195)
- **If a project YAML file already exists when Phase 4 would write a new one, a timestamped backup is created at <name>.yaml.bak.<timestamp> before writing.**
  (trigger: an existing file would be overwritten; outcome: no existing project YAML is lost without a backup)
  — `BU-P5-027`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 197-198)
- **Registering a first project requires the copied YAML's name to match its filename, every repo to have a unique name and correct path, clone URLs present wherever sgt-sync may need to clone, roles/groups that identify real ownership, and agent instructions that state commands and observable constraints rather than vague quality slogans.**
  (trigger: an operator edits a copied project.yaml.example into a real project registration; outcome: the registered project is unambiguous, clonable, and carries agent instructions an actor can actually act on rather than merely aspirational prose)
  — `BU-P8-045`, `reference/sergeant-upstream/docs/getting-started.md` (L102-110)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Additional note

This is the corpus's U3 case: multi-round, data-dependent human interview inside one procedural checkpoint. Engine-gap **G5** (survives, narrowed to a re-enterable `needs_input` stage) is the recorded finding — see `reference-corpus/synthesis.md` §5. Until that capability exists, represent the interview as: this stage ends in `needs_input` once per unanswered question, is re-entered on each answer, and reads its own accumulated prior answers back out of durable state (the Work's response history) rather than assuming a single unbroken turn.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
