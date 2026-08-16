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

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Resolving ambiguous repository ownership using the project graph and existing contracts first (`BU-P5-043`).

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- Two repositories could legitimately own a user-visible or durable contract: ask the user rather than resolving the tie unilaterally (`BU-P5-043`).

### Completion boundary
This stage may complete only once every required behavior has exactly one named owning repository with role, deliverable, and acceptance recorded — or the stage has stopped at the J0 case above.

### Decision evidence
The per-repository ownership record is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
