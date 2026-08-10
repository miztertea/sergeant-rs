# Provenance — Wiki Digest

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W35** `wiki-digest`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-130` | wiki maintains Sergeant's automatic activity captures and a curated daily session digest. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 3) |
| `BU-P5-131` | wiki is loaded only for explicit ingest/backfill/regenerate/inspect/change requests on wiki output; routine dispatch, notification, and cleanup commands write automatic captures without any coordinator action or wiki-skill invocation. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 7-9) |
| `BU-P1-135` | When the user asks to ingest, backfill, regenerate, inspect, update, or change the wiki, load the wiki procedure, which owns capture behavior, digest generation, schema ownership, and index updates. | `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L116, Procedural skills table row) |

Routed here at N1 verifier round 2 (finding V3): AGENTS.md's routing table names the `wiki` skill, which this package's own notes identify as the same procedure as P6's `wiki-daily-digest` (conflict X9b, folded together as `wiki-digest`); `BU-P1-135` corroborates `BU-P5-131`'s trigger from a second, independent source document.

## Stages

### `00-inspect-preview`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-139` | The proposed session page is inspected for secret material, duplicate entities, incorrect PR/task outcomes, and unresolved generation errors before it is accepted. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 51-52) |
| `BU-P5-146` | If a dry run contains secrets, wiki stops, records only the affected source class (not the secret itself), and fixes redaction before retrying. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 72) |
| `BU-P5-147` | If PR or td state cannot be resolved, the outcome is marked unresolved rather than having completion inferred. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 73) |
| `BU-P5-137` (folded helper: read schema, formerly `00-read-schema`) | The digest schema (~/wiki/SCHEMA.md) is read before changing digest behavior or curated structure. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 48) |
| `BU-P5-145` (folded helper: read schema) | If the digest schema is missing or unreadable, wiki stops without writing any curated pages. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 71) |
| `BU-P5-138` (folded helper: dry run, formerly `10-dry-run`) | --dry-run is run first whenever regenerating an existing day or changing digest logic, before any non-dry run. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 49-50) |
| `BU-P6-093` (folded helper: dry run) | A day's digest, once written, is never silently regenerated on a later run; the operator must explicitly delete the existing page to force resynthesis, unless running in dry-run mode. | `reference/sergeant-upstream/bin/wiki-daily-digest` (L411-414) |
| `BU-P5-140` (folded helper: generate, formerly `30-generate`) | The non-dry-run digest command is only run once the dry-run preview satisfies the schema. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 53) |
| `BU-P5-143` (folded helper: generate) | The digest must synthesize outcomes, decisions, blockers, and next state; it must never reproduce the conversation as a transcript. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (lines 57-58) |
| `BU-P6-092` (folded helper: generate) | Synthesizing a daily activity digest is a bounded procedure that collects session content from every configured AI-agent history source for one day (silently skipping any source that is unavailable), enriches it with merged pull requests and completed tracked-work items for that day, and produces one synthesized markdown page per day rather than a raw log dump. | `reference/sergeant-upstream/bin/wiki-daily-digest` (L1-7) |
| `BU-P5-141` (folded helper: publish and index, formerly `40-publish-and-index`) | After a real digest run, ~/wiki/sessions/YYYY-MM-DD.md must exist and be linked from ~/wiki/index.md. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 54) |
| `BU-P5-148` (folded helper: publish and index) | An existing curated page is never overwritten with a version containing less information; the existing page is preserved and the rejected update is reported. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 74) |
| `BU-P5-149` (folded helper: publish and index) | If the index update fails, the generated page itself is kept, its exact path is reported, and the digest is left explicitly marked incomplete. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 75) |
| `BU-P5-142` (folded helper: log ingest, formerly `50-log-ingest`) | The schema-required ingest log entry is appended or verified after every digest run. | `reference/sergeant-upstream/skills/wiki/SKILL.md` (line 55) |

## Adjudication A4 (N1-BH-02 sweep)

Original stages: `00-read-schema`, `10-dry-run`, `20-inspect-preview`, `30-generate`, `40-publish-and-index`, `50-log-ingest`. Five of the six were classified "Deterministic-machinery candidate" (§6.5) at extraction: every stage except `20-inspect-preview`. None of the five carried an "Additional note" checkpoint argument, so all five demote by A4's default rule.

**Decision:** `00-read-schema` and `10-dry-run` fold forward, and `30-generate`, `40-publish-and-index`, `50-log-ingest` fold backward, all into `20-inspect-preview` — the package's sole judgment-bearing stage — which is renamed `00-inspect-preview` (now the workflow's only stage). No stage in this package required the §6.3 case-by-case reimplementation test — none of the five demoted stages carried an Additional note argument to weigh. The behavior units are not deleted — see `00-inspect-preview/CONTEXT.md`'s "Helpers (folded per N1 adjudication A4)" section. Stage count drops from 6 to 1.

This is the sweep's clearest illustration of finding N1-BH-02: a linear script pipeline (read schema, dry-run, generate, publish, log) had been staged one-for-one as if each mechanical step were an independent durable checkpoint, when the package's own extraction only ever found one point requiring actor judgment.

## Notes

**Synthesis notes:** P5's `wiki` and P6's `wiki-daily-digest` are the same procedure (conflict X9b) and are folded together here.
