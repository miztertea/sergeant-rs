# 50-archive-evidence: archive evidence

## Inputs

| File | Layer | Why |
|---|---|---|
| ../40-apply-and-acknowledge/output/README.md | L4 | upstream artifact produced by `40-apply-and-acknowledge` |

## Purpose

Body, generation, applied status and proof archived atomically; the recorded generation is fixed at acknowledgement time.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

Body, generation, applied status and proof archived atomically; the recorded generation is fixed at acknowledgement time.

## Behavior contract

- **A successfully acknowledged response is archived (body, gate_generation, applied_status, proof) under a mode-700 directory with a mode-600 body file, and the archive's recorded gate_generation is fixed at acknowledgement time — later changes to the live response_generation counter must not retroactively alter it.**
  (trigger: a response is acknowledged and archived; outcome: the archived record of what was approved and when is both access-restricted (private secrets) and immutable to later state changes, giving replay/audit a fixed fact)
  — `BU-P7-042`, `reference/sergeant-upstream/tests/sgt-ack-response-test.sh` (lines 100-110)
- **The single response-lock-protected action-lease finalizer must be a no-op success when there was never a notification or never an accepted lease, must record neither a spurious completion nor a spurious pending outcome in those cases, and must never fabricate a completion that the agent itself never durably proved.**
  (trigger: any worker-exit or recycling path finalizes an accepted action lease; outcome: an action lease's terminal disposition is always an accurate, explicit, and singly-sourced record — never guessed, never silently dropped, never duplicated across two competing finalizer implementations)
  — `BU-P7-052`, `reference/sergeant-upstream/tests/sgt-lease-finalizer-test.sh` (lines 1-13)
- **A completed turn's finalization must be atomic: it publishes the completion record and writes a finalization record together, must not leave a pending marker behind, and must not leak the response lock it acquired to do so.**
  (trigger: a worker's turn completes with proof already published; outcome: finalization leaves exactly one consistent durable record (never both a pending and a finalized marker simultaneously) and releases its own lock)
  — `BU-P7-053`, `reference/sergeant-upstream/tests/sgt-lease-finalizer-test.sh` (lines 94-99)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
