# 07-transfer-ownership

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a coordinator other than the original dispatcher needs to claim validation ownership

**Outcome:** ownership transfer requires cryptographic-strength process-ancestry proof of pane identity, and never displaces a live legitimate owner

**Statement (the operative rule):** Validation ownership belongs to the dispatching tmux pane; a coordinator in any other pane must claim ownership explicitly, and a claim is accepted only when the claiming pane proves it really runs inside the pane it names by walking its own process ancestry — a caller that merely exports `TMUX_PANE` cannot satisfy this — and the prior owner must be takeover-eligible (dead/absent pane, mismatched recorded identity, or explicit release) with a live unreleased owner never displaced.

## What must become true here (durable outcome)

Ownership transfer requires cryptographic-strength process-ancestry proof of pane identity, and never displaces a live legitimate owner — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0168`: Every ownership transfer appends the timestamp, reason, repository, prior and new pane, and both identity tuples to an owner-only `coordinator_handover.log`; a release is consumed by the claim that uses it, so it cannot be replayed later by a third pane.
- `BU-0369`: Accepting a coordinator ownership handover from a previously unseen pane requires proving that this process genuinely lives inside that pane's process tree (walking up to 64 ppid hops); merely exporting the TMUX_PANE variable cannot satisfy this proof.
- `BU-0370`: Claiming coordinator ownership determines its handover reason by checking, in order: whether the prior owner explicitly released ownership (released-by-owner); otherwise refusing outright if the prior owner's pane is still live and has not released; otherwise recording whether the prior pane still exists but was recycled to a different identity, or is simply gone.
- `BU-0371`: --release-ownership is an ownership-only operation: it never inspects worker readiness and never launches a validation run, and it can only be performed by the pane currently recorded as the owner (both its tmux pane ID and its recorded identity must match exactly).

