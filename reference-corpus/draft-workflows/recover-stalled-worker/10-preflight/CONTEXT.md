# 10-preflight: preflight

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-collect-signals/output/README.md | L4 | upstream artifact produced by `00-collect-signals` |

## Purpose

Stall proof, lease convergence, drain check, relaunch-metadata completeness, and old identity all run to completion before the attempt is stamped.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

Stall proof, lease convergence, drain check, relaunch-metadata completeness, and old identity all run to completion before the attempt is stamped.

## Behavior contract

- **A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch.**
  (trigger: a worker's status is in_progress with a recorded stall diagnostic; outcome: a stalled worker gets exactly one automatic relaunch attempt, ever, before requiring human input)
  — `BU-P6-071`, `reference/sergeant-upstream/bin/sgt-recover` (L6-10)
- **A recovery attempt refuses to proceed while an unfinished notification action-lease exists, unless the lease's owner is provably dead (its execution target no longer resolves at all and its recorded process is not running) — anything else, including a target that merely looks idle, must fail closed to preserve exact-once delivery evidence.**
  (trigger: a stall-recovery attempt finds an outstanding action lease from the stalled worker; outcome: recovery either converges the lease from the agent's own proof, proceeds only after proving the owner is truly dead, or refuses and escalates — never silently discards pending delivery evidence)
  — `BU-P6-073`, `reference/sergeant-upstream/bin/sgt-recover` (L140-155)
- **A recovery attempt runs every pre-flight validation — stall proof, lease convergence, drain check, relaunch-metadata completeness, prior-execution-instance identity — to completion before stamping the attempt as made, so that any pre-flight failure leaves the stalled worker untouched and eligible for a real recovery attempt later.**
  (trigger: a stall recovery is being attempted; outcome: a recovery attempt is 'used up' (the one-shot budget consumed) only once the coordinator has actually committed to relaunching, never by a pre-flight check that merely failed to validate)
  — `BU-P6-075`, `reference/sergeant-upstream/bin/sgt-recover` (L229-232, L260-264)
- **Stall recovery (sgt-recover) must be refused while a drain is active, consistent with drain admission control blocking new relaunches — a stalled worker under an active drain is not relaunched by recovery.**
  (trigger: a worker appears stalled while a drain is active; outcome: recovery relaunch respects the same admission-control boundary as ordinary dispatch/respond relaunches, so a drain is a reliable global 'stop starting new work' switch)
  — `BU-P7-092`, `reference/sergeant-upstream/tests/sgt-recover-drain-test.sh` (line 2)
- **A missing `.sergeant-gate-generation` file must not leak a raw shell input-redirection error to stderr; and a pending action lease must not unconditionally refuse recovery — the lease owner's liveness and staleness must be adjudicated: a provably dead owner does not block recovery, while a live owner, a reused worker-session identifier, or an unprovable owner still fails closed.**
  (trigger: sgt-recover encounters a pending action lease while attempting stall recovery; outcome: recovery is never blocked by a lease whose owner is provably gone, while remaining fail-closed whenever ownership cannot be proven dead (including the PID-reuse case))
  — `BU-P7-093`, `reference/sergeant-upstream/tests/sgt-recover-lease-owner-test.sh` (lines 1-14)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
