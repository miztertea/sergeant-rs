# Provenance — Wake and Resume

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W14** `wake-and-resume`.

## Stages

### `00-validate-condition`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-097` | A wake condition's field names and value characters are both drawn from a strict allowlist — no field outside a fixed vocabulary is accepted, no value may begin with a dash (so it can never be misread as a flag by gh/td), and every field is additionally screened for secret-shaped names — before the condition is ever evaluated. | `reference/sergeant-upstream/bin/sgt-wake` (L23-32, L69-74) |
| `BU-P7-098` | A wake-condition file may only contain the allowlisted field names and alphanumeric-safe values for its declared kind; it must never be used to persist arbitrary shell commands, prompt bodies, response text, tokens, or secrets. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume', wake-condition paragraph) |

### `10-evaluate`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-096` | Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored. | `reference/sergeant-upstream/bin/sgt-wake` (L9-16) |
| `BU-P6-100` | Evaluating an external GitHub check status always binds the query to the worker's own recorded worktree's remote, never to whatever repository the scheduler process happens to be running from, because the scheduler is normally invoked from somewhere other than the worker's own repository. | `reference/sergeant-upstream/bin/sgt-wake` (L284-291) |

### `20-classify-outcome`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-098` | A wake condition distinguishes 'unmet' (may still become true on a later attempt) from 'escalate' (has become permanently unsatisfiable, so continuing to retry would be dishonest and wasteful) — for example a GitHub check that has already concluded with a non-success outcome can never become success, so it escalates rather than being retried until the attempt budget or deadline runs out. | `reference/sergeant-upstream/bin/sgt-wake` (L268-274, L486-491) |
| `BU-P7-097` | A wake condition past its optional `deadline` must transition the worker to a failed status with a recorded reason (and never call sgt-respond to resume it), rather than continuing to wait past a caller-specified bound. | `reference/sergeant-upstream/tests/sgt-wake-test.sh` (lines 401-402) |

### `30-resume`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-096` | Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored. | `reference/sergeant-upstream/bin/sgt-wake` (L9-16) |

