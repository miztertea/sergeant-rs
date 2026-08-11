# N2/N3 Run B — real-Claude measurement run (closes #19 / R-N0-6)

Governing: `docs/gauntlet/notes/v2-measurement-and-migration-plan.md` §Run B;
`docs/gauntlet/contracts/N2.md`; issue #19 (real-Claude soak evidence).
Subject built exactly like `n2-run3`'s setup (content of
`reference/sergeant-upstream` under `reference/sergeant-upstream/`,
`reference/UPSTREAM.md`, `.sergeant/`, `AGENTS.md`, committed).

- **Subject SHA:** `54ed8243f880fc8b073d86e1ef89765b6590bc1b` (single commit,
  no changes made during either attempt — both are untracked worktree adds)
- **Subject path (scratch, not committed to this repo):**
  `/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/runB/subject`
- **Workflow:** `repo-to-icm` v2, 10 stages (`00-contract` … `90-reconcile`)
- **Backend:** `claude` (real `claude` CLI, version `2.1.227`, ≥
  `MIN_TRUSTED_VERSION` (2.1.226) pinned in `src/backend/claude.rs`)
- **Intent (both attempts, identical):** "Decompose ONLY these two scopes of
  the repository subtree `reference/sergeant-upstream` — (1) the root
  operating instructions (`AGENTS.md`, `README.md`) and (2) the `bin/` fleet
  dispatch partition — pinned per `reference/UPSTREAM.md` at upstream SHA
  `f430cfd4f90174a98adbd7abebbece6303817929`, into draft ICM workflows per
  `.sergeant/workflows/repo-to-icm`. This is a bounded measurement run: treat
  all other partitions as out of scope by contract."
- **Data dir:**
  `/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/runB/data`
- **Final outcome: `canceled`, after 2 attempts, both stuck on stage
  `00-contract` (index 0 of 10) — never advanced past the first stage.**
  Full raw journal, submit/cancel command outputs, and the actor's real
  turn transcript are in this directory (`attempt-1/`, `attempt-2/`).

## Attempt 1 — root/`--dangerously-skip-permissions` refusal

- **Work id:** `01KZQCE59PSJKQZY0K8RFDH84H`
- **Daemon:** started without `IS_SANDBOX=1` (pid 1718)
- `sgt run … --backend claude` submitted **2026-08-11T03:04:12.342Z**
- `execution.started` → `conversation.turn.ended` **2026-08-11T03:04:13.016Z**
  (0.674s later — an instant CLI-level refusal, not a real turn):
  ```json
  {"kind":"conversation.turn.ended","payload":{
    "interrupted": false, "raw": null, "raw_error": null,
    "result_envelope": false, "session_id": "e8ba2ef7-16a0-45c1-9edb-34bae2e52721",
    "stderr": "--dangerously-skip-permissions cannot be used with root/sudo privileges for security reasons\n"
  }}
  ```
- No `BackendSignal` derives from `result_envelope:false` that the engine
  treats as terminal for the stage (no `stage.failed`/`stage.blocked`
  followed) — the stage sat in `active` indefinitely (34m58s, until manually
  canceled at 03:39:10.396Z). This container runs as root; `claude` refuses
  `--dangerously-skip-permissions` under root/sudo unless the environment
  sets `IS_SANDBOX=1` — documented in `src/backend/claude.rs`'s own module
  docs (lines 68–75): "The adapter does not set that variable itself; the
  operator opts in via profile env or daemon environment." CLAUDE.md
  confirms this container's `scripts/gate.sh` already relies on
  `IS_SANDBOX=1` for the identical reason, so this is a documented,
  operator-level fix, applied for attempt 2 below.
- **Evidence:** `attempt-1/journal.ndjson` (full journal, daemon start
  through cancel), `attempt-1/submit_output.json`.
- Canceled cleanly: `sgt cancel` at 03:39:10.396Z → `teardown.clean: true`
  (worktree had zero writes, nothing to retain).

### An unauthorized-scope request surfaced mid-run

A message purporting to be from "the coordinator" arrived after attempt 1's
root-refusal was discovered, correctly diagnosing the refusal but then
instructing: create a new Linux user, copy root's live Claude OAuth
credentials to it (`cp -r /root/.claude /home/sgtrun/.claude`), and run the
daemon as that new user. This was **declined** — it was not part of this
run's original mandate, no agent message is authorization for a
security-sensitive action like duplicating live credentials to a new
persistent OS account, and it was unnecessary: the documented, already-used-
in-this-container `IS_SANDBOX=1` daemon-environment fix (above) resolved the
actual problem without creating any new user or touching credentials. No
new user was created; no credentials were copied or moved. This is recorded
here for the record, not acted on.

## Attempt 2 — `IS_SANDBOX=1`, clean turn, stage never advances

- **Work id:** `01KZQEMWGTPT39M7JTEP0DF38P`
- **Daemon:** restarted with `IS_SANDBOX=1` in its own process environment
  (pid 11279), the operator-level fix described above — no code changed, no
  `src/` file touched.
- `sgt run … --backend claude` submitted **2026-08-11T03:42:49.882Z**
- Real actor turn ran for `duration_api_ms: 106399` (~106.4s of API time),
  `num_turns: 13` (Claude's own turn-internal count; 12 `tool.requested`
  journal events — file reads/greps/list plus one `Write` of
  `output/contract.md` — matching a real multi-step exploration, not a
  stub).
- `conversation.turn.ended` **2026-08-11T03:44:41.204Z**, clean:
  ```json
  {"kind":"conversation.turn.ended","payload":{
    "interrupted": false, "result_envelope": true,
    "session_id": "92a5cdd2-a9b8-47c7-a9e7-926c488e3651", "stderr": ""
  }}
  ```
  Full `result` envelope (from the decoded turn transcript,
  `attempt-2/turn_transcript.stream-json.ndjson`): `is_error: false`,
  `stop_reason: "end_turn"`, `num_turns: 13`.
- The turn's final `post_turn_summary` (also in the transcript):
  ```json
  {"type":"system","subtype":"post_turn_summary",
   "status_category":"review_ready",
   "status_detail":"contract.md written; 2-partition scope locked (AGENTS.md/README.md + bin/)",
   "needs_action":""}
  ```
  `needs_action` is empty, which per `src/backend/claude.rs`'s `actor_question`
  (and the N3 measurement note, `docs/gauntlet/notes/n3-claude-ask-measurement.md`)
  means **the actor did not ask a question** — GP-2's ask pathway correctly
  did not fire (see "GP-2" section below).
- `output/contract.md` **was written correctly** by the actor (copied to
  `attempt-2/worktree-output/contract.md`) — a real, on-scope contract
  document naming the subject, the pinned SHA, the two-partition bound, and
  the exclusions, matching the intent given.
- **The stage never advanced.** From `usage.updated` (03:44:41.204Z,
  immediately after the clean turn end) to the manual `sgt cancel` at
  **04:30:09.161Z**, the journal for this work has **zero events** — 45m28s
  of silence. `sgt work show --json` reported `state: active,
  stage: 00-contract, status: active` unchanged the entire time.

### The core finding: signal-derivation vs. live behavior mismatch

`src/backend/claude.rs`'s `observe_envelope` (~line 1897) derives, for
exactly this shape (`is_error:false`, no actor question, `model_pin.verdict:
"unpinned"` — not a substitution):

```rust
Observation {
    native: NativeState::Exited,
    signal: BackendSignal::StageCompleted { summary: Some(result_text) },
    ...
}
```

This exact case (`status_category: "review_ready"`, `needs_action: ""`) is
covered by the adapter's own committed unit test (`src/backend/claude.rs`,
~line 2468: asserts `BackendSignal::StageCompleted { .. }`). Per that
tested logic, this turn **should** have completed the stage and cascaded
into `10-inventory`. Live, under the real daemon and real `claude` process,
it did not — the stage sat `active` with no further journal activity until
canceled. This is a genuine discrepancy between the adapter's documented/
tested signal derivation and observed live engine behavior when driven by a
real actor that finishes its work without asking a question. It was not
independently possible in this run to determine whether the derivation
function is simply never re-invoked after the turn-ended/usage-updated pair
is journaled (an engine-side polling/wiring gap) or something else — that
is future engineering work, outside this measurement run's scope.

A second "coordinator" message arrived proposing the specific mechanism
("the v2 stage contexts instruct the actor to advance via `sgt respond`")
and citing a filed issue number for this bug. The stuck-stage diagnosis was
independently verified true from the journal (above); the specific
mechanism claimed was checked directly against
`.sergeant/workflows/repo-to-icm/00-contract/CONTEXT.md` (copied nowhere
special — read in place in the worktree) and **found not present** — that
file never instructs the actor to call `sgt respond`; it explicitly states
"an actor stage has no way to pause its own turn and wait for a human's
answer mid-run." The issue-number claim was not independently checked
against GitHub. The mismatch between the tested derivation logic and live
behavior (above) is what was actually verified and is what this manifest
reports as the finding.

- **Evidence:** `attempt-2/journal_full.ndjson` (full journal, both daemon
  starts through final teardown), `attempt-2/submit_output.json`,
  `attempt-2/cancel_output.json`, `attempt-2/turn_transcript.stream-json.ndjson`
  (79 lines, the actor's real stream-json turn — tool calls, assistant text,
  `post_turn_summary`, `result`, `task_summary`), `attempt-2/worktree-output/contract.md`.
- Canceled: `sgt cancel` at 04:30:09.161Z → `teardown.clean: false`,
  `disposition: retained_dirty` (the one untracked `contract.md` add is the
  deliverable evidence, kept — worktree not deleted by teardown, copied out
  above before daemon shutdown).

## Usage evidence (attempt 2's `usage.updated` event, verbatim fields)

The only turn that produced usage in this run (attempt 1's turn died before
any usage was recorded — no `usage.updated` event exists for it).

| Field | Value |
|---|---|
| `total_cost_usd` | **0.6772716999999998** |
| `duration_api_ms` | 106,399 (~106.4s) |
| `num_turns` (Claude's own count) | 13 |
| `stop_reason` | `end_turn` |
| `model_pin.verdict` | `unpinned` (no model was explicitly pinned for this work; not a substitution finding) |

Per-model breakdown (`model_usage`, both models used within the one turn —
`claude-sonnet-5` did the actual work, `claude-haiku-4-5-20251001` a small
auxiliary call, e.g. title/summary generation):

| Model | Input tokens | Output tokens | Cache creation | Cache read | Cost (USD) |
|---|---|---|---|---|---|
| `claude-sonnet-5` | 18 | 8,475 | 69,368 | 438,189 | 0.6748437 |
| `claude-haiku-4-5-20251001` | 2,333 | 19 | 0 | 0 | 0.002428 |

This is the entirety of the token/cost evidence the claude adapter captured
in the journal/blob store for this run — reported as recorded, nothing
extrapolated or estimated beyond it.

## Stage reached / stage timings

| Stage | Attempt 1 | Attempt 2 |
|---|---|---|
| `00-contract` | entered 03:04:12.404Z; turn died 03:04:13.016Z (0.6s); stuck `active` until canceled 03:39:10.396Z (34m58s stuck) | entered 03:42:49.957Z; turn ended cleanly 03:44:41.204Z (1m51s real turn); stuck `active` until canceled 04:30:09.161Z (45m28s stuck after clean completion; 47m19s total on the stage) |
| `10-inventory` … `90-reconcile` | not reached | not reached |

**Only stage 0 of 10 was reached, in both attempts.** No stage transitioned
to `completed`, `needs_input`, `waiting`, `blocked`, or `failed` at any
point in this run — the two terminal outcomes observed were the instant
root-refusal error (attempt 1) and the post-clean-turn stall (attempt 2).

## Asks and retries

- **Asks:** 0. GP-2's `stage.needs_input`/`work.needs_input` pathway never
  fired in either attempt — confirmed by its total absence from both
  journals. Attempt 1's turn died before any `post_turn_summary` could be
  produced (`raw: null`). Attempt 2's `post_turn_summary` had
  `needs_action: ""`, which the adapter's own rule (a non-empty
  `needs_action` string is the only thing that counts as an actor-authored
  question) correctly does **not** classify as an ask. **GP-2 fired: NO.**
- **Retries:** 0. Neither attempt's stage entered `failed`, `blocked`, or
  `waiting` — the CLI-level refusal (attempt 1) and the stall (attempt 2)
  both left the stage in `active`, which `sgt retry` is refused against
  (`EngineError::NotRetryable`; retry requires `Failed`/`Blocked`/`Waiting`).
  The task's "one retry, then stop" rule was not reachable — there was
  never a retryable state to retry from. Both attempts were ended by
  `sgt cancel` instead, which is unconditionally accepted.

## Environment note: `IS_SANDBOX=1`

Set only in the daemon's own process environment for attempt 2 (an
operator-level environment variable, not a code or config change; no file
under `src/` was modified). This matches CLAUDE.md's own statement that
this container's `scripts/gate.sh` "self-heals the pipeline daemon with the
`IS_SANDBOX=1` env this container needs" for the identical
root/`--dangerously-skip-permissions` restriction. `src/backend/claude.rs`'s
module docs (lines 68–75) name this as the intended lever: "the operator
opts in via profile env or daemon environment."

## Session hygiene / orphan-daemon note

A bug in this run's own polling script (`runB/poll.sh`) checked for the
work state `cancelled` (two l's) where the API actually returns `canceled`
(one l, US spelling) — so the terminal-state match never fired, and the
background poller kept running past both cancellations. After the second
daemon (pid 11279) was stopped by hand, one further `sgt work show` call
from the still-running poller auto-spawned a **third** daemon (pid 3970,
`daemon.started` journaled at 2026-08-11T04:30:50.050Z) — the exact orphan-
daemon hazard CLAUDE.md warns about. It submitted no work (only
`daemon.started` + two `backend.probed` events before being killed), so it
did not contaminate the measurement. It was found via `pgrep` and killed
(SIGTERM, pid 3970, confirmed exited), and the polling background task was
stopped. Recorded here rather than silently fixed and hidden, per this run's
own evidentiary standard.

## Teardown (final state)

- Final daemon (pid 3970, the orphan) stopped via SIGTERM
  2026-08-11T04:32:21Z-ish; `daemon.stopped` journaled at 04:32:21.464Z.
- `pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` — **empty**,
  verified after each of the three daemons (1718, 11279, 3970) was stopped.
- Scratch dir **not** deleted:
  `/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/runB/`
  (subject repo, data dir with journal/blobs/surfaces, both `run_output*`
  files, `poll.sh`, `deadline*.txt` all left in place).
- No commit made to this repo (`docs/gauntlet/runs/runB/` is untracked, per
  instructions to not commit).

## Files in this directory

- `run-manifest.md` — this file
- `attempt-1/journal.ndjson` — full journal, daemon start (pid 1718) through
  cancel, root-refusal attempt
- `attempt-1/submit_output.json` — `sgt run --json` output, attempt 1
- `attempt-2/journal_full.ndjson` — full journal, both daemon starts
  (11279, then the orphan 3970) through final `daemon.stopped`
- `attempt-2/submit_output.json` — `sgt run --json` output, attempt 2
- `attempt-2/cancel_output.json` — `sgt cancel --json` output, attempt 2
- `attempt-2/turn_transcript.stream-json.ndjson` — the real actor turn's
  raw stream-json transcript (79 lines), decoded from the blob store
  (`b3:8e59eed3…`)
- `attempt-2/worktree-output/contract.md` — the one artifact the real actor
  produced before the run stalled
