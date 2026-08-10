# 30-deliver-and-accept: deliver and accept

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-publish-response/output/README.md | L4 | upstream artifact produced by `20-publish-response` |

## Purpose

Bounded readiness gate; on timeout, a nonce-scoped unreachable record plus a recoverable gate — never a fabricated acknowledgement.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

Bounded readiness gate; on timeout, a nonce-scoped unreachable record plus a recoverable gate — never a fabricated acknowledgement.

## Behavior contract

- **A worker's readiness gate for delivering a notification is bounded, not infinite: it waits at most a fixed timeout per notification target, and on timeout reports the unreachable state exactly once as durable, nonce-scoped evidence plus a recoverable needs_input gate — it never fabricates acknowledgement, acceptance, delivery, completion, or an action lease.**
  (trigger: a harness never becomes ready to receive a notification; outcome: an unreachable harness always surfaces as an actionable, recoverable needs_input state, never a misleading terminal orphaned status and never an infinite hang)
  — `BU-P6-114`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L378-386)
- **The full durable notification handshake (nudge delivered, ack token written, acceptance confirmed, instruction followed exactly once, completion published) must be exercised end-to-end for EVERY harness in the shared registry, twice — once for the initial notification and once for a response notification delivered to a relaunched worker — because a prior test iterated harnesses but never actually reached the handshake files for any harness but one, letting a defect go unnoticed for every other harness.**
  (trigger: any supported harness receives an initial or response notification; outcome: the durable handshake contract is proven identically for every harness the shared registry supports, not merely for the one harness earlier tests happened to cover deeply)
  — `BU-P7-109`, `reference/sergeant-upstream/tests/sgt-worker-handshake-test.sh` (lines 1-15)
- **sgt-respond must never leave a response indefinitely pending merely because delivery to a live pane exceeded its bounded acknowledgement timeout; rerunning the identical command is the documented bounded-recovery path, performing exactly one worker relaunch and retiring the unresponsive original pane only after the replacement is validated.**
  (trigger: a delivered response's acknowledgement timeout elapses with no ack from the worker pane; outcome: an operator has one deterministic, safe, idempotent next action (rerun the command) rather than being dead-ended between 'already pending' and 'not yet acknowledged' error states)
  — `BU-P7-059`, `reference/sergeant-upstream/tests/sgt-respond-recovery-test.sh` (lines 1-13)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P7-059` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
