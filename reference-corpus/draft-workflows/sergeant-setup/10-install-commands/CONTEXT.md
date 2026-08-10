# 10-install-commands: install commands

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/standing-constraints.md | L3 | constraints binding every stage of this workflow |
| ../05-file-capability-gaps/output/README.md | L4 | upstream artifact produced by `05-file-capability-gaps` |

## Purpose

Commands resolve on PATH, verified; failure stops with the expected source paths named.

Trigger (workflow-level): First install, a new project/repository to register, a broken or incomplete installation, or a verification request.

## What must become true here (durable outcome)

Commands resolve on PATH, verified; failure stops with the expected source paths named.

## Behavior contract

- **When cloning Sergeant, the destination path is asked for explicitly and the workflow waits for the user's answer before doing anything else.**
  (trigger: the Sergeant repository is not already cloned; outcome: a destination is established before any clone attempt)
  — `BU-P5-015`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 109-114)
- **The clone command is shown verbatim with the resolved destination and requires explicit y/N consent; any other response leaves the filesystem unchanged.**
  (trigger: a clone destination has been established; outcome: cloning never happens without an explicit, visible, confirmed command)
  — `BU-P5-016`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 116-122)
- **When mise is available, sergeant-setup first resolves the actual install directory from SGT_INSTALL_DIR, defaulting to $HOME/.local/bin.**
  (trigger: mise is present; outcome: the exact install target is known and can be shown to the user before running anything)
  — `BU-P5-017`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 124-128)
- **sergeant-setup shows the resolved install target and requires explicit consent before running mise run install; if mise is unavailable or consent is declined, it instructs the user to symlink bin/ commands onto PATH manually and to verify the result before continuing.**
  (trigger: the install directory has been resolved; outcome: commands are installed only via an explicit consented path or an explicit manual fallback, never silently)
  — `BU-P5-018`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 130-138)
- **After install, sergeant-setup verifies that at least sgt-list, sgt-context, sgt-dispatch, and sgt-watch resolve on PATH; missing commands and their expected source paths are reported, and the run stops if verification fails, resuming from Phase 2 on the next invocation.**
  (trigger: an install path (mise or manual) has been attempted; outcome: the environment is proven to have the commands the rest of setup depends on, before proceeding)
  — `BU-P5-019`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 140-143)
- **For each installable prerequisite, sergeant-setup shows the exact installation command and requires explicit y/yes consent before running it; any other response leaves the system unchanged.**
  (trigger: a prerequisite is classified installable; outcome: no installation command runs without explicit per-command consent)
  — `BU-P5-014`, `reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md` (lines 94-103)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
