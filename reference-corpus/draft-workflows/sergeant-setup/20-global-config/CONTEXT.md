# 20-global-config: global config

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../10-install-commands/output/README.md | L4 | upstream artifact produced by `10-install-commands` |

## Purpose

One machine-wide `dev_root` exists and parses; an existing file is never overwritten without backup + diff + confirmation.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

One machine-wide `dev_root` exists and parses; an existing file is never overwritten without backup + diff + confirmation.

## Behavior contract

- **sergeant-setup checks for ~/.config/sergeant/config.yaml; if missing, it asks the user for a dev_root path, then shows a full preview of the file content and requires explicit y/N confirmation before writing anything.**
  (trigger: the global config file does not exist; outcome: the global config is written only after the user has seen and approved its exact content)
  — `BU-P5-020`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 147-158)
- **If the global config is present and valid, sergeant-setup verifies dev_root is set and reports [ok] without further action.**
  (trigger: the global config already exists and parses; outcome: an already-correct global config is left untouched)
  — `BU-P5-021`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (line 159)
- **If the global config exists but fails to parse as YAML (checked via yq), sergeant-setup reports the parse error and stops; it never overwrites the file without a timestamped backup, a diff preview, and explicit confirmation.**
  (trigger: the global config exists and is invalid; outcome: an unparseable config is never silently replaced)
  — `BU-P5-022`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 160-162)
- **A global config.yaml sets one machine-wide dev_root used as the base for every relative repo path in every project YAML on that machine.**
  (trigger: global configuration is created during install; outcome: the machine has exactly one place that governs relative-path resolution for every project)
  — `BU-P8-044`, `reference/sergeant-upstream/docs/getting-started.md` (L85-94)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
