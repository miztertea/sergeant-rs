# 00-inspect-preview: inspect preview

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

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

## Helpers (folded per N1 adjudication A4)

This workflow originally decomposed the digest pipeline into six stages (`00-read-schema`, `10-dry-run`, `20-inspect-preview`, `30-generate`, `40-publish-and-index`, `50-log-ingest`). `20-inspect-preview` was the package's only stage ever classified "Judgment required" (§6.4) — every other stage was justified solely by the §6.5 deterministic-machinery boilerplate, with no "Additional note" checkpoint argument anywhere in the package. Per N1 adjudication A4 (finding N1-BH-02), all five demote by default and fold into this stage (renamed from `20-inspect-preview`, now the workflow's sole stage) as helper invocations bracketing the one real review checkpoint:

- **Read schema** (performed before the dry run). The digest schema (~/wiki/SCHEMA.md) is read before changing digest behavior or curated structure. If the digest schema is missing or unreadable, wiki stops without writing any curated pages.
  — `BU-P5-137`, `BU-P5-145`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 48, 71)
- **Dry run** (performed before this stage's review). `--dry-run` is run first whenever regenerating an existing day or changing digest logic, before any non-dry run. A day's digest, once written, is never silently regenerated on a later run; the operator must explicitly delete the existing page to force resynthesis, unless running in dry-run mode.
  — `BU-P5-138`, `BU-P6-093`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 49-50), `reference/sergeant-upstream/bin/wiki-daily-digest` (L411-414)
- **Generate** (performed once this stage's review accepts the preview). The non-dry-run digest command is only run once the dry-run preview satisfies the schema. The digest synthesizes outcomes, decisions, blockers, and next state; it never reproduces the conversation as a transcript. It collects session content from every configured source (silently skipping any unavailable), enriched with merged PRs and completed tracked-work items.
  — `BU-P5-140`, `BU-P5-143`, `BU-P6-092`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 53, 57-58), `reference/sergeant-upstream/bin/wiki-daily-digest` (L1-7)
- **Publish and index.** After a real digest run, `~/wiki/sessions/YYYY-MM-DD.md` must exist and be linked from `~/wiki/index.md`. An existing curated page is never overwritten with a version containing less information. If the index update fails, the generated page itself is kept, its exact path is reported, and the digest is left explicitly marked incomplete.
  — `BU-P5-141`, `BU-P5-148`, `BU-P5-149`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 54, 74, 75)
- **Log ingest.** The schema-required ingest log entry is appended or verified after every digest run.
  — `BU-P5-142`, `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 55)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
