# 10-validate-target: validate target

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-precondition-check/output/README.md | L4 | upstream artifact produced by `00-precondition-check` |

## Purpose

The target's status is one of the four respondable states and its recorded identity/ownership evidence verifies; anything else refuses.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

The target's status is one of the four respondable states and its recorded identity/ownership evidence verifies; anything else refuses.

## Behavior contract

- **A response can only ever be published against a worker whose current status is needs_input, blocked, waiting, or orphaned — any other status refuses the response outright, so a response is never silently applied to a worker that was not actually asking for one.**
  (trigger: an operator supplies a response for a specific task/repo; outcome: responses are only ever delivered to workers in one of exactly four states that legitimately mean 'this worker is waiting for input')
  — `BU-P6-078`, `reference/sergeant-upstream/bin/sgt-respond` (L202-205)
- **Publishing a response requires verifying worker identity and ownership evidence (session identity, worktree pointer/directory) recorded at dispatch time before the response is written, so a response can never be delivered to the wrong worker or a worktree Sergeant no longer actually owns.**
  (trigger: a response is about to be published for a specific fleet task/repo; outcome: response delivery is bound to a durably recorded, ownership-verified worker identity rather than a bare task/repo name lookup)
  — `BU-P7-060`, `reference/sergeant-upstream/tests/sgt-respond-test.sh` (lines 9-46)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
