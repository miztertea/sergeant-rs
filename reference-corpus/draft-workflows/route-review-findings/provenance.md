# Provenance — Route Review Findings

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W16** `route-review-findings`.

## Stages

### `00-parse-and-sanitize`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-082` | Independent review findings are routed to tracked work as a bounded, evidence-preserving procedure: parse and sanitize the reviewer's structured output, retain a sanitized copy before any external side effect, route each actionable finding to exactly one deduplicated task, and — only once every finding has reached tracked work — publish a blocking gate if any finding is severe enough to block, or clear it otherwise. | `reference/sergeant-upstream/bin/sgt-review-findings` (L2) |

### `10-retain-artifact`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-084` | A retained, sanitized artifact of parsed findings is written to durable storage before any external side effect (td calls), so a routing failure that happens after parsing never destroys the only copy of a review's findings; the artifact's location is included in the failure diagnostic as an explicit, retryable next action. | `reference/sergeant-upstream/bin/sgt-review-findings` (L427-430) |

### `20-route-each`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-085` | A finding is deduplicated against an existing tracked-work item using a marker scoped to the exact review axis, source, finding ID, parent mission, and branch — never axis/source/id alone — because reviewers emit generic finding IDs (e.g. 'spec-1') that would otherwise collide across unrelated review sessions and let one session's update silently overwrite another's evidence. | `reference/sergeant-upstream/bin/sgt-review-findings` (L524-528) |
| `BU-P6-086` | A matched existing tracked-work item is only ever updated (priority, labels reopened-if-closed) when its stored content digest still matches the incoming finding's digest; a divergent stored body is refused and left completely untouched — no description, title, or status change — so nothing stored can ever be silently lost by a write that never happens. | `reference/sergeant-upstream/bin/sgt-review-findings` (L586-592) |
| `BU-P7-063` | Deduplication of routed findings must be scoped to the parent task and branch, not applied globally, so an identical-looking finding on a different branch is never silently treated as an already-seen duplicate. | `reference/sergeant-upstream/tests/sgt-review-findings-test.sh` (line 492) |
| `BU-P7-064` | The router must refuse (not silently accept) findings whose review-artifact composition or comparison the owner explicitly ruled must be rejected, per an adjudicated decision rather than an inferred default. | `reference/sergeant-upstream/tests/sgt-review-findings-test.sh` (line 541) |

### `30-publish-or-clear-gate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-082` | Independent review findings are routed to tracked work as a bounded, evidence-preserving procedure: parse and sanitize the reviewer's structured output, retain a sanitized copy before any external side effect, route each actionable finding to exactly one deduplicated task, and — only once every finding has reached tracked work — publish a blocking gate if any finding is severe enough to block, or clear it otherwise. | `reference/sergeant-upstream/bin/sgt-review-findings` (L2) |

