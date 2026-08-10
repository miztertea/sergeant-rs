# Provenance — Wake and Resume

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W14** `wake-and-resume`.

## Stages

### `00-validate-condition` (folded into `10-evaluate`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-097` | A wake condition's field names and value characters are both drawn from a strict allowlist — no field outside a fixed vocabulary is accepted, no value may begin with a dash (so it can never be misread as a flag by gh/td), and every field is additionally screened for secret-shaped names — before the condition is ever evaluated. | `reference/sergeant-upstream/bin/sgt-wake` (L23-32, L69-74) |
| `BU-P7-098` | A wake-condition file may only contain the allowlisted field names and alphanumeric-safe values for its declared kind; it must never be used to persist arbitrary shell commands, prompt bodies, response text, tokens, or secrets. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume', wake-condition paragraph) |

### `10-evaluate` (kept — see Adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-096` | Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored. | `reference/sergeant-upstream/bin/sgt-wake` (L9-16) |
| `BU-P6-100` | Evaluating an external GitHub check status always binds the query to the worker's own recorded worktree's remote, never to whatever repository the scheduler process happens to be running from, because the scheduler is normally invoked from somewhere other than the worker's own repository. | `reference/sergeant-upstream/bin/sgt-wake` (L284-291) |

### `20-classify-outcome` (folded into `10-evaluate`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-098` | A wake condition distinguishes 'unmet' (may still become true on a later attempt) from 'escalate' (has become permanently unsatisfiable, so continuing to retry would be dishonest and wasteful) — for example a GitHub check that has already concluded with a non-success outcome can never become success, so it escalates rather than being retried until the attempt budget or deadline runs out. | `reference/sergeant-upstream/bin/sgt-wake` (L268-274, L486-491) |
| `BU-P7-097` | A wake condition past its optional `deadline` must transition the worker to a failed status with a recorded reason (and never call sgt-respond to resume it), rather than continuing to wait past a caller-specified bound. | `reference/sergeant-upstream/tests/sgt-wake-test.sh` (lines 401-402) |

### `30-resume` (folded into `10-evaluate`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-096` | Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored. | `reference/sergeant-upstream/bin/sgt-wake` (L9-16) |

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02):

| Stage | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|
| `00-validate-condition` | none | Swapping the allowlist-check implementation leaves the checkpoint — no unsafe field/value ever reaches evaluation — unchanged. | **Demoted** — folded into `10-evaluate` as a preceding helper invocation. |
| `10-evaluate` | **yes** — "This is the direct source of engine-gap **G1**... the *scheduling* of this stage... is exactly what no lower rung can own." | Not an implementation-swap argument at all: it argues the *engine itself* lacks a rung for periodic, processless re-evaluation — categorically different from, and stronger than, the test asks about. | **Kept.** |
| `20-classify-outcome` | none | Swapping the classification implementation leaves the checkpoint — met/unmet/escalate/failed correctly distinguished — unchanged. | **Demoted** — folded into `10-evaluate`. |
| `30-resume` | none | Swapping the resume implementation leaves the checkpoint — the worker resumed on a met outcome — unchanged. | **Demoted** — folded into `10-evaluate`. Its `promote` output disposition is inherited by the merged stage output (see `10-evaluate/output/README.md`). |

Stage count: 4 extracted → 1 surviving. No behavior unit was deleted; all citations above remain cited, under `10-evaluate`'s own contract or its "Helper invocations" section (see that stage's `CONTEXT.md`).

