# 10-respond-to-worker

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the worker response-delivery step is about to be used

**Outcome:** the five-step precondition/delivery sequence is followed before and after responding

**Statement (the operative rule):** Before responding to a worker: read the exact finding/question and recommendation, ask only for missing product/risk/security/privacy/destructive/irreversible decisions, record the decision in the owning task tracker task, verify no unconsumed response generation already exists, and require the matching worker to acknowledge/consume the response after sending.

## What must become true here (durable outcome)

The five-step precondition/delivery sequence is followed before and after responding — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0156`: The supervisor nudge includes a scoped token (`notification_id|target_nonce`); the agent writes the acknowledgement but does not act yet, proceeding only after the targeted supervisor sends acceptance and the scoped acceptance file contains the same token, then records completion in the named completion file.
- `BU-0157`: The notified worker reads `.sergeant-response`, its ID, and gate generation, applies the decision exactly once, restores truthful status, and writes `.sergeant-response-applied` with the matching ID, generation, and status.
- `BU-0177`: A pending response is never overwritten; the exact waiting worker is resumed with the worker response-delivery step or the caller waits for the current generation to reach a terminal outcome, the stalled-worker recovery step is never used for an active response generation, and if the worker already applied the response with an existing archive entry, the same response-acknowledgement step command is rerun from the recorded worker pane to finish acknowledgement and cleanup.
- `BU-0179`: Repeated notifications are diagnosed by comparing task, repo, state generation, message digest, and timestamp, since they can indicate stale fleet records, unconsumed responses, or an expected blocked worker incorrectly reclassified orphaned; duplicate tasks or duplicate responses are never created in reaction.
- `BU-0275`: When a worker escalates, the coordinator reads its context/evidence/exact question/recommendation/options, obtains the human decision without inferring consequential intent, and only then runs the worker response-delivery step; the worker consumes/removes the response, clears its message, logs the decision to the task tracker, and returns to in_progress.
- `BU-0405`: Before making any state change, the worker response-delivery step acquires this repo's response lock and registers a cleanup trap that releases the lock and removes the private response-input file on exit.
- `BU-0412`: The worker response-delivery step refuses to publish a response unless the recorded worktree is re-verified, at response time, to still be the exact owned checkout — its actual git pointer and git directory must match the values recorded at dispatch — even after a migration attempt.
- `BU-0413`: The worker response-delivery step refuses to publish a response unless the worker's canonical intent revision still matches at response time; if it does not, one migration attempt is made and the check is repeated before refusing.
- `BU-0414`: The worker response-delivery step only accepts a response when the worker's current status is exactly one of needs_input, blocked, waiting, or orphaned; any other status is refused.
- `BU-0415`: Publishing the local response transport writes the generation marker, response id, response body (fleet-state and worktree copies), atomically via temp-file-then-rename, and clears any stale acknowledgement marker before writing the new response id, so a leftover ack can never be mistaken for acknowledgement of the new response.
- `BU-0416`: A delivery-escalation marker is considered armed only when it names the exact response id and gate generation currently in play; a marker from an earlier response or generation does not arm escalation.
- `BU-0417`: For an orphaned worker with no valid recorded gate generation, the worker response-delivery step initializes the generation to 1 and records it, rather than refusing the response for lack of a generation.
- `BU-0418`: If the response text supplied to the worker response-delivery step differs from an already-stored pending response, the newly supplied text is ignored and the stored response is reused verbatim, with a warning printed.
- `BU-0419`: A pending response is only redelivered when its recorded generation matches the worker's current gate generation in both fleet state and the worktree, and its recorded response id matches on both sides; any mismatch refuses the operation.
- `BU-0420`: When the recorded pane is live and its identity matches the worker's dispatch record, the worker response-delivery step notifies it and waits, with the lock released, for acknowledgement within a bound; on timeout it arms the delivery-escalation marker and reports the response as stored and recoverable rather than lost.
- `BU-0421`: Once a live pane acknowledges delivery within the bound, the worker response-delivery step clears the unacknowledged-delivery marker and exits successfully without relaunching any worker.
- `BU-0422`: When there is no live local pane and the relaunch metadata (tmux, session, window, agent) is incomplete, the worker response-delivery step leaves the response file ready on disk and exits successfully without attempting a relaunch.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0397`: The worker response-delivery step requires exactly two positional arguments, a task ID and a repo name; an invalid argument count is rejected before any other work happens.
- `BU-0398`: A task ID supplied to the worker response-delivery step must match a restricted identifier pattern (starts alphanumeric, then alphanumeric/./_/- only); a malformed task ID is rejected.
- `BU-0399`: A repo name supplied to the worker response-delivery step must match the same restricted identifier pattern as a task ID; a malformed repo name is rejected.
- `BU-0400`: The worker response-delivery step reads the response body into a privately created temporary file: the file is created under a restrictive umask and explicitly chmod'd to owner-only before any content is written to it.
- `BU-0401`: The private response-input temporary file is always removed when the worker response-delivery step exits, via a trap registered before the file is created.
- `BU-0402`: The worker response-delivery step refuses to proceed if the response body read from standard input is empty.
- `BU-0403`: The worker response-delivery step refuses to proceed if the named task does not exist in fleet state.
- `BU-0404`: The worker response-delivery step refuses to proceed if the named repo does not exist under the named task in fleet state.

