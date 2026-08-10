# 60-route-findings: route findings

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-drive-gates/output/README.md | L4 | upstream artifact produced by `50-drive-gates` |

## Purpose

Every actionable finding becomes one deduplicated owning-repo task with a deterministic severity→priority mapping; correctness/security/data-integrity/test findings can never be deferred or ignored; no finding is fixed inside the run.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

Every actionable finding becomes one deduplicated owning-repo task with a deterministic severity→priority mapping; correctness/security/data-integrity/test findings can never be deferred or ignored; no finding is fixed inside the run.

## Behavior contract

- **A no-mistakes finding's severity and disposition together determine, deterministically, both its td priority and whether it is even allowed to be deferred: correctness/security/data-integrity/test findings must gate or ask the user and can never be routed to td or ignored, while cosmetic/evidence findings create no td card at all.**
  (trigger: a no-mistakes review finding is classified with a kind and a requested disposition; outcome: a finding either blocks/escalates, is silently dropped as non-actionable, or is routed to td for later work — and which of those three happens is a deterministic function of kind, never left to caller discretion)
  — `BU-P6-023`, `reference/sergeant-upstream/bin/sgt-no-mistakes-finding` (L67-77)
- **An existing td task matched by a finding's deduplication marker is reopened if closed and has its deferral cleared before being updated, so a finding that recurs after being closed or snoozed is never silently left in a stale closed/deferred state.**
  (trigger: a finding matches an existing but closed or deferred td task; outcome: a recurring finding always surfaces as live tracked work again)
  — `BU-P6-026`, `reference/sergeant-upstream/bin/sgt-no-mistakes-finding` (L176-185)
- **The no-mistakes run is validation-only and must not fix findings; actionable findings are routed into separate, deduplicated owning-repository td tasks with sgt-no-mistakes-finding.**
  (trigger: a no-mistakes run produces findings; outcome: findings become tracked, deduplicated repository work rather than being fixed in-run)
  — `BU-P1-080`, `reference/sergeant-upstream/README.md` (README.md L304)
- **Applying a disposition to a no-mistakes finding must route it through the same `td` invocation contract (run-id, head-sha, finding-id, severity, kind, file, line, description, intent) regardless of disposition, and the routing behavior itself (e.g. `--disposition td`) is directly observable in the exact `td` invocation logged.**
  (trigger: a no-mistakes finding needs a disposition applied (e.g. routed to td as debt); outcome: a finding's routing is deterministic and inspectable — the exact fields passed to td are asserted, not merely 'some td call happened')
  — `BU-P7-065`, `reference/sergeant-upstream/tests/sgt-no-mistakes-finding-test.sh` (lines 50-70, 88-90)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Delegation

This stage's outcome is produced by running **route-review-findings** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
