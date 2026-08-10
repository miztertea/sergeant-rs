# 00-read-schema: read schema

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

The schema is read before any behavior change; a missing schema stops the run before any page is written.

Trigger (workflow-level): A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## What must become true here (durable outcome)

The schema is read before any behavior change; a missing schema stops the run before any page is written.

## Behavior contract

- **The digest schema (~/wiki/SCHEMA.md) is read before changing digest behavior or curated structure.**
  (trigger: digest logic or curated structure is about to change; outcome: changes are made with the governing schema in hand, not from memory)
  — `BU-P5-137`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 48)
- **If the digest schema is missing or unreadable, wiki stops without writing any curated pages.**
  (trigger: the schema cannot be read; outcome: no curated content is ever written against an unknown or absent schema)
  — `BU-P5-145`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 71)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
