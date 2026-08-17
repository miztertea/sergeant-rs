# 40-escalate-on-second-attempt: preflight, launch replacement, retire original, then escalate on second attempt

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-collect-signals/output/README.md | L4 | upstream artifact produced by `00-collect-signals` |

## Purpose

Exactly one bounded recovery attempt is made; a second stall escalates to needs-input. N1 adjudication A4 folded the three mechanical recovery stages — preflight, launch replacement, retire original — in ahead of this stage's own judgment: swapping each one's implementation would leave this stage's checkpoint (exactly one bounded attempt, ever) unchanged, so they run first as ordered helper invocations.

Trigger (workflow-level): A worker is `in_progress` with a stall classification recorded by the watcher.

## What must become true here (durable outcome)

Stall proof, lease convergence, drain check, relaunch-metadata completeness, and old identity all run to completion before the attempt is stamped; the replacement is launched and validated live before the original is retired; the original is retired only after the replacement is proven live; and — because every invocation is stamped — a second stall on the same worker escalates to needs-input rather than retrying.

## Behavior contract

- **A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch.**
  (trigger: a worker's status is in_progress with a recorded stall diagnostic; outcome: a stalled worker gets exactly one automatic relaunch attempt, ever, before requiring human input)

## Bounded judgment

Apply `@@bounded-judgment`.

### J5 — governing constraint
- A stall recovery attempt is gated on concrete stall proof; every invocation is stamped so a second attempt always escalates rather than retries.
- Refuse while an unfinished action-lease exists unless the owner is provably dead; anything else fails closed.
- Every pre-flight validation runs to completion before the attempt is stamped as made.
- Recovery is refused while a drain is active.
- Lease-owner liveness/staleness is adjudicated fail-closed on unprovable ownership.
- The replacement is only launched, and its identity validated, before the original is ever terminated.
- Exactly one bounded recovery attempt per invocation: terminate, relaunch, atomically update fleet metadata, deliver notification.

### J1 — local choices allowed
- Error-message text quality when reporting a lease-owner adjudication failure — no raw shell error is leaked to stderr.

### J0 — must become `needs_input`
- A second stall on an already-stamped worker.
- A first, correctly-blocked preflight failure for an operational reason — an active drain or an unprovable lease owner — does **not** consume the one-shot stamp, so it is neither a completed recovery nor a "second attempt" escalation. State this explicitly rather than leaving it silent: the stage completes with `evidence`-only output recording the block, and continued visibility relies on `sgt-watch`'s own independent periodic reclassification to re-surface the worker later — the same division of labor the upstream architecture uses between watcher and recoverer.

### Completion boundary
This stage may complete only once the one bounded recovery attempt is made (preflight validated, replacement launched and proven live, original retired only after) — or the stage has stopped/recorded one of the J0 cases above.

### Decision evidence
The stamp state, preflight results, and replacement viability are this stage's own durable output, recorded per `output/README.md`.

## Helper invocations (folded stages, N1 adjudication A4)

Three stages extracted as their own candidates (ladder §6.5, "deterministic-machinery candidate") carried no "Additional note" arguing they survive §6.3's reimplementation test. They fold in here as ordered helper invocations, run before this stage's own escalation judgment.

**1. preflight** (formerly `10-preflight`) — stall proof, lease convergence, drain check, relaunch-metadata completeness, and old identity all run to completion before the attempt is stamped. Note: the first unit below is the same unit cited in this stage's own "Behavior contract" above — the corpus cites it at both the gating checkpoint (preflight validates the stamp) and the escalation checkpoint (a second stamped attempt escalates); it is one fact serving both, not duplicated evidence.

- **A stall recovery attempt is gated on concrete stall proof — status must be in_progress and the fleet diagnostic must begin with a stall-classification marker written by the watcher — and every invocation is stamped so a second attempt always escalates to needs_input instead of retrying, guaranteeing exactly one bounded relaunch.**
  (trigger: a worker's status is in_progress with a recorded stall diagnostic; outcome: a stalled worker gets exactly one automatic relaunch attempt, ever, before requiring human input)
- **A recovery attempt refuses to proceed while an unfinished notification action-lease exists, unless the lease's owner is provably dead (its execution target no longer resolves at all and its recorded process is not running) — anything else, including a target that merely looks idle, must fail closed to preserve exact-once delivery evidence.**
  (trigger: a stall-recovery attempt finds an outstanding action lease from the stalled worker; outcome: recovery either converges the lease from the agent's own proof, proceeds only after proving the owner is truly dead, or refuses and escalates — never silently discards pending delivery evidence)
- **A recovery attempt runs every pre-flight validation — stall proof, lease convergence, drain check, relaunch-metadata completeness, prior-execution-instance identity — to completion before stamping the attempt as made, so that any pre-flight failure leaves the stalled worker untouched and eligible for a real recovery attempt later.**
  (trigger: a stall recovery is being attempted; outcome: a recovery attempt is 'used up' (the one-shot budget consumed) only once the coordinator has actually committed to relaunching, never by a pre-flight check that merely failed to validate)
- **Stall recovery (sgt-recover) must be refused while a drain is active, consistent with drain admission control blocking new relaunches — a stalled worker under an active drain is not relaunched by recovery.**
  (trigger: a worker appears stalled while a drain is active; outcome: recovery relaunch respects the same admission-control boundary as ordinary dispatch/respond relaunches, so a drain is a reliable global 'stop starting new work' switch)
- **A missing `.sergeant-gate-generation` file must not leak a raw shell input-redirection error to stderr; and a pending action lease must not unconditionally refuse recovery — the lease owner's liveness and staleness must be adjudicated: a provably dead owner does not block recovery, while a live owner, a reused worker-session identifier, or an unprovable owner still fails closed.**
  (trigger: sgt-recover encounters a pending action lease while attempting stall recovery; outcome: recovery is never blocked by a lease whose owner is provably gone, while remaining fail-closed whenever ownership cannot be proven dead (including the identifier-reuse case))

**2. launch replacement** (formerly `20-launch-replacement`) — the replacement is validated live before the original is retired.

- **During recovery, the replacement worker is only launched, and its identity validated, before the original stalled worker instance is ever terminated, so that any failure in the relaunch sequence leaves the original stalled process intact for investigation rather than losing the supervisor entirely.**
  (trigger: a stall recovery attempt is relaunching a worker; outcome: a failed recovery attempt never leaves a Work with no supervisor at all)
- **Recovery must validate a replacement supervisor's liveness, published identity, and notification-target creation BEFORE killing the stalled original — the kill must be strictly ordered after the replacement is confirmed live, and every abort path must restore fleet state so the recorded worker identity still points at the surviving original.**
  (trigger: sgt-recover replaces a stalled worker supervisor with a fresh one; outcome: recovery can never end up with neither a working original nor a working replacement — the destructive step (killing the original) only happens once the replacement is proven viable)

**3. retire original** (formerly `30-retire-original`) — the original is retired only after the replacement is proven live.

- **Stall recovery performs exactly one bounded recovery attempt per invocation for an in-progress worker: terminate the stalled worker, relaunch a fresh worker, atomically update fleet metadata, and deliver a recovery notification — a single bounded operation, not an open-ended retry loop.**
  (trigger: an in-progress worker appears stalled (no observable progress); outcome: stall recovery is a single, boundable, observable action rather than an unbounded retry loop that could mask a genuinely broken worker)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
