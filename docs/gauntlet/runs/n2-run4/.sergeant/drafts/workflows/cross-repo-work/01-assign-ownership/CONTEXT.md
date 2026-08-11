# 01-assign-ownership

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** a requested outcome is being decomposed across repositories

**Outcome:** ownership assignment is unambiguous (exactly one owner per behavior) and scoped to repos that actually must change

**Statement (the operative rule):** For each required behavior, exactly one repository is named as owning its implementation, and a repository is included only when it must change or produce delivery evidence.

## What must become true here (durable outcome)

Ownership assignment is unambiguous (exactly one owner per behavior) and scoped to repos that actually must change — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0268`: The user is asked about repository ownership only when two repositories could legitimately own a user-visible or durable contract; otherwise ambiguity is resolved from the project graph and existing contracts.

