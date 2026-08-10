# Provenance — Project Graph

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W2** `project-graph`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-105` | Project Graphify is run by reading the configured output path from sgt-context, stopping to request or add that path if unconfigured, then running sgt-graphify <project>. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 58-63) |
| `BU-P5-106` | graphify query is used for focused questions; GRAPH_REPORT.md is read for broad architecture, community, and god-node context. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 64-65) |
| `BU-P5-112` | Stale graph output is refreshed via sgt-graphify only when architecture work actually requires it or the user explicitly requests it, never automatically. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 76) |

## Stages

### `00-resolve-output-path`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-100` | A single project-level graphify.output path, located outside any source repository, is configured when project Graphify is required. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 42-43) |
| `BU-P5-107` | Generated project graph output is never published inside an owning source repository. | `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 66) |
| `BU-P8-103` | A recurring or recursive graphify output is diagnosed by inspecting the project's configured graphify.output path and keeping exactly one output per project, located outside every source repository, rather than regenerating or moving an existing graph without first confirming the intended global per-project path. | `reference/sergeant-upstream/docs/troubleshooting.md` (L148-152 (Graphify output is wrong or recursive)) |

### `10-extract-per-repo`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-089` | Extracting a repo's graph data stages the repo into a filtered, non-source-repo scratch location before scanning whenever any output-exclusion pattern applies, so the extraction tool can never re-ingest an already-published graph nested inside the source repo it is scanning. | `reference/sergeant-upstream/bin/sgt-graphify` (L349-355) |
| `BU-P6-090` | When no supported LLM API key is configured, graph extraction is run in code-only mode rather than being allowed to abort the entire multi-repo run on encountering a non-code file it cannot semantically process without a key. | `reference/sergeant-upstream/bin/sgt-graphify` (L356-359) |
| `BU-P7-088` | sgt-graphify must fall back to a `--code-only` extraction mode when no LLM API key is configured, rather than failing outright, so graph extraction remains available without an LLM dependency. | `reference/sergeant-upstream/tests/sgt-graphify-test.sh` (line 660) |

### `20-merge-or-fail`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-091` | Building a project graph across many repos is all-or-nothing at the merge step: if extraction fails for any included repo, the whole run fails before attempting to merge or publish anything, rather than publishing a graph silently missing some repos. | `reference/sergeant-upstream/bin/sgt-graphify` (L430-433) |

### `30-publish-atomically`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-088` | Publishing a freshly built project knowledge graph is atomic: concurrent readers see either the complete old graph or the complete new graph throughout the whole run, never a partial or missing state, regardless of whether the configured output is a plain directory or a symlink pointing inside or outside a source repo. | `reference/sergeant-upstream/bin/sgt-graphify` (L7-10) |
| `BU-P7-003` | A published cross-repo knowledge graph is replaced only after a graphify run completes in full, and the output directory may sit inside a source repo without its own artifacts being re-ingested as source. | `reference/sergeant-upstream/schema/project.yaml.example` (lines 90-92) |
| `BU-P7-086` | Publishing a merged project graph must be atomic via a symlink swap (`mv -T`): if that atomic rename fails, the old symlink must remain pointing at the previous, still-valid output rather than being left dangling or partially updated. | `reference/sergeant-upstream/tests/sgt-graphify-test.sh` (lines 551-557) |

## Notes

**Demoted/merged candidates:** `40-consume` (BU-P5-105/106/112: query the published graph for focused questions or read the report for broad context) failed the §6.3 reimplementation test — "I ran a query" is not a checkpoint operators track — and is demoted to shared context/helper (see `reference-corpus/synthesis.md` §3 for the destination map; this milestone does not materialize helper-map.md).

