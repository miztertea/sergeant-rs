# 70-relaunch-if-needed: relaunch if needed

## Inputs

| File | Layer | Why |
|---|---|---|
| ../60-notify-coordinator/output/README.md | L4 | upstream artifact produced by `60-notify-coordinator` |

## Purpose

Convergence attempted through the single finalizer before any refusal; superseded identities preserved as evidence.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

Convergence attempted through the single finalizer before any refusal; superseded identities preserved as evidence.

## Behavior contract

- **An outstanding notification action-lease from the worker being responded to is first attempted to converge through the one shared finalizer, using only the agent's own exact completion proof; only if that convergence fails does responding refuse with a specific remediation pointing at the exact evidence path.**
  (trigger: a response relaunch would otherwise clear an outstanding action lease; outcome: a legitimate but unrecorded completion is never discarded by a relaunch, and a genuinely unfinished lease is refused with a concrete remediation, never silently overwritten)
  — `BU-P6-079`, `reference/sergeant-upstream/bin/sgt-respond` (L417-435)
- **A response relaunch never allows a second, superseding tmux pane to displace the first without preserving the first pane's superseded notification-target identity as evidence — and if that evidence would conflict with already-recorded evidence, the relaunch refuses outright rather than losing the older evidence.**
  (trigger: a relaunch is superseding an existing notification target; outcome: the evidence trail for who was ever asked to act, and when they were superseded, is never lost or silently overwritten)
  — `BU-P6-080`, `reference/sergeant-upstream/bin/sgt-respond` (L437-449)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P6-080` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
