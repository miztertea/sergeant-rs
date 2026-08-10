# 20-publish-response: publish response

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-validate-target/output/README.md | L4 | upstream artifact produced by `10-validate-target` |

## Purpose

The response is durably stored (even under an active drain) before any delivery is attempted.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

The response is durably stored (even under an active drain) before any delivery is attempted.

## Behavior contract

- **Publishing a response must still durably store a delivered response even while a project or global drain is active, but must hold the relaunch of a stalled worker until the drain is lifted — admission control gates only the relaunch action, never the response storage itself.**
  (trigger: an operator responds to a needs_input worker while a drain is active; outcome: an operator's response is never lost merely because the fleet is draining; only the potentially-conflicting relaunch action is deferred)
  — `BU-P7-058`, `reference/sergeant-upstream/tests/sgt-respond-drain-test.sh` (lines 1-3)
- **`sgt-respond` must publish a response with no response-lock artifact left over on success, on immediate abort (mktemp failure), and on recovery from an empty, dead-PID, or stale-symlink leftover lock — but must fail immediately and actionably ("Response lock has an invalid owner") without touching the pending response when the lock file is not a recognizable lock shape at all.**
  (trigger: sgt-respond attempts to publish a response while an existing (possibly stale) response.lock is present; outcome: every recoverable lock shape (empty dir, dead PID, dangling symlink) converges to a clean publish, while an unrecognizable lock shape fails closed and preserves the original pending response untouched)
  — `BU-P7-035`, `reference/sergeant-upstream/tests/runtime-bash-test.sh` (lines 84-172)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
