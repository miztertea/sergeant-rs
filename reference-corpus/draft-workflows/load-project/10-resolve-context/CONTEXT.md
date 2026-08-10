# 10-resolve-context: resolve context

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-resolve-project-name/output/README.md | L4 | upstream artifact produced by `00-resolve-project-name` |

## Purpose

Owning repos, absolute paths, clone state, roles/groups, and the layered instruction set are recorded as the governing context.

Trigger (workflow-level): A project is named, registered, edited, synced, or listed; or repository ownership is not already established.

## What must become true here (durable outcome)

Owning repos, absolute paths, clone state, roles/groups, and the layered instruction set are recorded as the governing context.

## Behavior contract

- **load-project runs sgt-context <project> and records the owning repositories, resolved absolute paths and clone state, group membership and roles, instructions inherited in defaults-then-group-then-repository order, and any configured Graphify output and included groups.**
  (trigger: the project name is known; outcome: a complete, resolved context snapshot exists for downstream workflows to consume)
  — `BU-P5-093`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 19-25)
- **A raw project YAML is read directly only when a required field is absent from the resolved sgt-context output.**
  (trigger: sgt-context output is missing a needed field; outcome: the resolved context view is always preferred over the raw source file, used only as a fallback)
  — `BU-P5-094`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 26-27)
- **Completion evidence for load-project is the sgt-context block showing every owning repository as cloned, plus the instructions and paths that will govern execution.**
  (trigger: context resolution finishes; outcome: there is one concrete, checkable artifact proving the checkpoint is satisfied)
  — `BU-P5-096`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 31-32)
- **The generated context block reports each repo's clone status as one of exactly three states — cloned with its current branch, present but not a git repo, or not cloned at all — so a reading agent never has to infer clone state from absence.**
  (trigger: a project context block is generated; outcome: the reported clone status is always one of three explicit, distinguishable states)
  — `BU-P6-021`, `reference/sergeant-upstream/bin/sgt-context` (L67-75)
- **If a project has a knowledge graph configured, the emitted context block tells the agent whether a built graph report already exists to read or names the exact command to build one, rather than silently omitting graph information.**
  (trigger: a project has graphify.output configured; outcome: an agent orienting itself always knows whether an architecture graph is readable or must first be generated)
  — `BU-P6-022`, `reference/sergeant-upstream/bin/sgt-context` (L136-139)
- **If sgt-context and the raw project YAML disagree, the sgt-context failure is treated as blocking and the YAML is preserved for diagnosis.**
  (trigger: resolved context and raw source disagree; outcome: disagreement between resolved and raw sources is never silently resolved by picking one)
  — `BU-P5-111`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 75)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
