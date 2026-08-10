# Provenance — Worker Mission (software-change)

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W9** `worker-mission`.

## Stages

### `00-pin-scope` (folded into `10-triage-and-route`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-005` | A dispatched worker's mission brief pins a pre-implementation source of truth: fetch refs, pin a fixed base commit (normally the merge-base with origin/main), and record base SHA, commit list, and diff scope before implementation begins. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 1. Pin scope and source of truth') |

### `10-triage-and-route`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-007` | Routing work before implementation requires triage: read the full originating context, check for redundant/prior work, and classify into one of five categories (huge/foggy, hard bug or perf regression, uncertain design/UI, approved feature/fix, merge/rebase conflict), each of which loads a different canonical skill. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 2. Route the work') |

### `20-implement`

No directly-cited units (delegated or structural — see the stage's own CONTEXT.md).

### `30-independent-review`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-013` | Independent review before completion must run every axis named in the brief's authoritative axis list as separate parallel subagents whose contexts cannot contaminate each other, even if the loaded review skill itself names fewer axes, and their outputs must stay in separate, unblended, unreranked sections. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 7. Independent {{REVIEW_AXIS_LABEL}}-axis review') |

### `40-escalate-or-continue`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-009` | Every Sergeant notification must be acknowledged, then explicitly accepted by the supervisor, then acted on exactly once and marked complete — each step writing the same supervisor-scoped token to a distinct named file; repeated nudges carrying the same token are retries of the same action, never new work. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume') |
| `BU-P7-012` | Before every new `needs_input` or `blocked` publication, a worker must increment a monotonic per-worktree gate-generation counter and persist it before writing the waiting status and message; a repeated blocker message is a new gate only when the generation actually advanced. | `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume') |

### `50-publish-result` (folded into `40-escalate-or-continue`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-066` | sgt-td-memory must record handoff evidence only from a verified worktree, and every git field it stores (branch, HEAD, etc.) must resolve from that specific worktree rather than from the supervisor's own current working directory — proven with two real linked worktrees on different branches/commits, not simulated. | `reference/sergeant-upstream/tests/sgt-td-memory-worktree-test.sh` (lines 1-18) |
| `BU-P7-110` | The interactive worker's wait for harness readiness must be bounded and its outcome reported — a harness that never renders must be caught and reported, not hang forever — and separately, a harness that becomes ready without ever acknowledging the notification must NOT be misrecorded as orphaned. | `reference/sergeant-upstream/tests/sgt-worker-readiness-test.sh` (lines 1-9) |

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02):

| Stage | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|
| `00-pin-scope` | none | Swapping the ref-fetch/base-pin implementation leaves the checkpoint — a recorded, reproducible diff scope exists before implementation — unchanged. | **Demoted** — folded into `10-triage-and-route` as a preceding helper invocation. |
| `10-triage-and-route` | yes (engine-gap G6 note), but already `actor-stage` (§6.4) — n/a to the demotion test | n/a | **Kept** as extracted. |
| `20-implement`, `30-independent-review` | n/a (extracted as actor-stage, §6.4) | n/a | **Kept** as extracted. |
| `40-escalate-or-continue` | yes (obsolete-mechanism M5 note), but already `actor-stage` (§6.4) — n/a to the demotion test | n/a | **Kept** as extracted. |
| `50-publish-result` | none | Swapping the handoff-recording/readiness-wait implementation leaves the checkpoint — verified-worktree evidence, bounded readiness — unchanged. | **Demoted** — folded into `40-escalate-or-continue` as a following helper invocation. Its `promote` output disposition is inherited by the merged stage output (see `40-escalate-or-continue/output/README.md`). |

Stage count: 6 extracted → 4 surviving. No behavior unit was deleted; `BU-P7-005`, `BU-P7-066`, and `BU-P7-110` remain cited, now under `10-triage-and-route`'s or `40-escalate-or-continue`'s "Helper invocations" section (see those stages' `CONTEXT.md`).

## N1 verifier round 2 (finding V3) — routing note, not a citation

`BU-P5-150` (vertical-slice discipline, `20-implement`), `BU-P5-151` (remediation loop, `30-independent-review`/`20-implement`), `BU-P5-152` (commit/PR/CI/review-thread/dependency-order delivery, `40-escalate-or-continue`), and `BU-P5-153` (td decision logging and `td review` timing, `40-escalate-or-continue`) are the four remaining rows of the same ordered dispatch worker-contract list that `BU-P5-075`–`BU-P5-089` already belong to. Per record-shapes.md §3's one-unit-one-stage rule and this package's own established pattern for that range (see `dispatch/80-monitor/CONTEXT.md`'s "render brief" helper), these four units are cited once, in `dispatch/provenance.md`'s `60-render-brief` section (content `dispatch` *authors* into the brief), not duplicated here. This package is the content's *executor* — the stage each unit's parenthetical above names is where a worker actually carries out that contract item during a mission — but the binding citation stays with the authoring package.

