# Wiki Digest
Draft workflow package — candidate **W35** `wiki-digest` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

Generate and publish a schema-driven wiki digest from configured sources, previewed before publication and never regressing an existing page.

## Trigger

A digest is due (scheduled) or explicitly requested; or the schema/logic changed and needs a dry run first.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-inspect-preview` | actor-stage (§6.4, judgment; absorbs `00-read-schema`, `10-dry-run`, `30-generate`, `40-publish-and-index`, `50-log-ingest` per A4) | Secrets, duplicate entities, wrong outcomes, unresolved errors are checked; a secret stops the run and only the source *class* is recorded. |

## Notes for reviewers

P5's `wiki` and P6's `wiki-daily-digest` are the same procedure (conflict X9b) and are folded together here.

**N1 adjudication A4 (finding N1-BH-02).** This package originally decomposed the digest pipeline into six stages mirroring a linear script pipeline (read schema → dry run → inspect preview → generate → publish and index → log ingest). Only `20-inspect-preview` was ever classified "Judgment required" (§6.4); the other five were justified solely by the §6.5 deterministic-machinery boilerplate, and none carried an "Additional note" checkpoint argument. All five demote by A4's default rule and fold into the single review checkpoint, renamed `00-inspect-preview` (now the workflow's sole stage). This is the honest reading of a package whose original six-stage shape used a future `execute`-stage kind as its justification (convention §5 rule 1 forbids exactly this): the digest job is a scripted pipeline with one human-in-the-loop gate, not six independent durable checkpoints. Stage count drops from 6 to 1; the behavior units survive — see `00-inspect-preview/CONTEXT.md`'s "Helpers (folded per N1 adjudication A4)" section and `provenance.md`.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
