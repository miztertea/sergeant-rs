# 00-verify-readiness: verify readiness

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A published readiness marker asserts the exact intent revision, the exact reviewed head, and an explicit pass on every review axis; any mismatch refuses with its own reason.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

A published readiness marker asserts the exact intent revision, the exact reviewed head, and an explicit pass on every review axis; any mismatch refuses with its own reason.

## Behavior contract

- **A validation run can only be launched once the worker has published a readiness marker asserting the exact intent revision, the exact reviewed head commit, and that all three independent review axes (standards, spec, readiness) explicitly passed — a stale head, a mismatched intent revision, or any axis not equal to 'passed' each refuse the launch with its own specific reason.**
  (trigger: the coordinator attempts to launch validation; outcome: validation can never be launched against work that has not been reviewed, or against a different commit than the one that was reviewed)
  — `BU-P6-130`, `reference/sergeant-upstream/bin/sgt-validate` (L236-269)
- **A worker may only request the final no-mistakes boundary by writing durable validation-ready evidence (intent_revision, head_sha, and pass/fail for standards/spec/readiness review) and notifying the coordinator; the worker itself is forbidden from running no-mistakes.**
  (trigger: a worker's native validation and independent reviews all report zero blockers; outcome: only the coordinator, never the worker itself, ever crosses the final shipping-gate checkpoint)
  — `BU-P8-082`, `reference/sergeant-upstream/docs/using-sergeant.md` (L312-317 (Final no-mistakes boundary))

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
