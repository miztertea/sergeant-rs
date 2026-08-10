# 20-inspect-preview: inspect preview

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-dry-run/output/README.md | L4 | upstream artifact produced by `10-dry-run` |

## Purpose

Secrets, duplicate entities, wrong outcomes, unresolved errors are checked; a secret stops the run and only the source *class* is recorded.

Trigger (workflow-level): A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## What must become true here (durable outcome)

Secrets, duplicate entities, wrong outcomes, unresolved errors are checked; a secret stops the run and only the source *class* is recorded.

## Behavior contract

- **The proposed session page is inspected for secret material, duplicate entities, incorrect PR/task outcomes, and unresolved generation errors before it is accepted.**
  (trigger: a dry-run preview has been produced; outcome: a generated page is human-reviewed against four specific risk categories before being made real)
  — `BU-P5-139`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 51-52)
- **If a dry run contains secrets, wiki stops, records only the affected source class (not the secret itself), and fixes redaction before retrying.**
  (trigger: a dry-run preview contains secret material; outcome: the failure is recorded without itself leaking the secret, and is fixed before any retry)
  — `BU-P5-146`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 72)
- **If PR or td state cannot be resolved, the outcome is marked unresolved rather than having completion inferred.**
  (trigger: PR or td state is unresolvable; outcome: an unknown outcome is recorded as unknown, never guessed as complete)
  — `BU-P5-147`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 73)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
