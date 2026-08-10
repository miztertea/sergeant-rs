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
| `00-read-schema` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The schema is read before any behavior change; a missing schema stops the run before any page is written. |
| `10-dry-run` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A dry run always runs first when regenerating or changing logic. |
| `20-inspect-preview` | actor-stage (§6.4, judgment) | Secrets, duplicate entities, wrong outcomes, unresolved errors are checked; a secret stops the run and only the source *class* is recorded. |
| `30-generate` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Synthesis, never a transcript; collected from every configured source with unavailable ones silently skipped. |
| `40-publish-and-index` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The page exists and is linked, or the page is kept, its path reported, and the digest marked incomplete; an existing page is never overwritten with less information. |
| `50-log-ingest` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | The ingest is logged. |

## Notes for reviewers

P5's `wiki` and P6's `wiki-daily-digest` are the same procedure (conflict X9b) and are folded together here.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
