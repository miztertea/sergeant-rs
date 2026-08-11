# 10-assign-ownership: assign ownership

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Exactly one owning repo per behavior, with role / deliverable / acceptance recorded.

Trigger (workflow-level): Resolved project context shows more than one repository owns the requested outcome (not merely that the project has several repos).

## What must become true here (durable outcome)

Exactly one owning repo per behavior, with role / deliverable / acceptance recorded.

## Behavior contract

- **For each required behavior, cross-repo-work names exactly one owning repository, including a repository only when it must change or produce delivery evidence, and records repo/role/delivers/acceptance for it.**
  (trigger: decomposition begins; outcome: every required behavior has exactly one named owner with a stated observable deliverable and acceptance evidence)
  — `BU-P5-041`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 21-26)
- **The per-repository ownership record has a fixed shape: repo name, resolved role, the observable behavior or artifact it delivers, and the repo-native command or evidence that proves completion.**
  (trigger: an owning repository is recorded; outcome: every owner record is comparable and machine-checkable)
  — `BU-P5-042`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 27-34)
- **Ambiguous repository ownership is resolved using the project graph and existing contracts first; the user is asked only when two repositories could legitimately own a user-visible or durable contract.**
  (trigger: ownership of a behavior is not obvious from role alone; outcome: the user is interrupted only for genuinely contested ownership, not every ambiguity)
  — `BU-P5-043`, `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 36-38)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
