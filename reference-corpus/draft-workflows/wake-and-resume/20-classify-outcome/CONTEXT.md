# 20-classify-outcome: classify outcome

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-evaluate/output/README.md | L4 | upstream artifact produced by `10-evaluate` |

## Purpose

Outcome is classified met / unmet / permanently-unsatisfiable→escalate / deadline→failed.

Trigger (workflow-level): A worker is in the `waiting` state with a recorded wake condition.

## What must become true here (durable outcome)

Outcome is classified met / unmet / permanently-unsatisfiable→escalate / deadline→failed.

## Behavior contract

- **A wake condition distinguishes 'unmet' (may still become true on a later attempt) from 'escalate' (has become permanently unsatisfiable, so continuing to retry would be dishonest and wasteful) — for example a GitHub check that has already concluded with a non-success outcome can never become success, so it escalates rather than being retried until the attempt budget or deadline runs out.**
  (trigger: a wake condition is evaluated and found not yet satisfied; outcome: a permanently-unsatisfiable condition surfaces to the operator immediately rather than silently exhausting its retry budget first)
  — `BU-P6-098`, `reference/sergeant-upstream/bin/sgt-wake` (L268-274, L486-491)
- **A wake condition past its optional `deadline` must transition the worker to a failed status with a recorded reason (and never call sgt-respond to resume it), rather than continuing to wait past a caller-specified bound.**
  (trigger: sgt-wake evaluates a wake condition whose deadline has already passed; outcome: waiting is never truly unbounded — an expired deadline converts an indefinitely stuck wait into an explicit, terminal failure rather than an eternal wait or a spurious resume)
  — `BU-P7-097`, `reference/sergeant-upstream/tests/sgt-wake-test.sh` (lines 401-402)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
