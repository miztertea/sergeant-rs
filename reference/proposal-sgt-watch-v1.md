



⸻

type: proposal
title: “Sergeant WATCH: Harness Subscription CLI”
description: >-
Proposal for a read-only sgt watch command that lets an active coding
harness subscribe to Sergeant Work transitions without polling, MCP
configuration, callbacks, or harness-specific runtime integration.
status: proposed
resource: sergeant-rs
tags:

* sergeant-rs
* watch
* subscription
* events
* sse
* cli
* harness
* orchestration
* proposal
    timestamp: 2026-08-13
    repository: miztertea/sergeant-rs
    audit_revision: 5756b5d989774a3f91643f3b67a41ddf50df4c11
    relationship: >-
    Thin client projection over the existing P0 HTTP/SSE event surface and
    Work inspection API. This proposal does not alter workflow execution,
    Work state, backend behavior, daemon settlement, or harness lifecycle.

⸻

Sergeant WATCH

Harness Subscription CLI

Status: Proposed
Audit basis: miztertea/sergeant-rs@5756b5d
Proposed product surface: sgt watch
Primary objective: Let a harness wait efficiently for delegated Sergeant Work to require attention or produce a result
Primary consumer: An active, copiloted coding harness acting as the estate orchestrator
Initial example: Claude Code occupying the role defined by AGENTS.md
Hard boundary: No harness wake-up, process launch, session resurrection, MCP requirement, callback delivery, durable subscription record, notification acknowledgment, or new source of truth

Sections are numbered for contract citation, following the repository’s proposal convention.

⸻

1. Executive Summary

Sergeant’s North Star now supports this interaction:

operator
    ↓
active coding harness
    ↓ shapes and delegates intent
sgt run
    ↓
durable Work executes independently
    ↓
harness later inspects or responds

The missing connection is not execution.

It is the harness’s ability to wait for a meaningful Work transition without repeatedly asking:

sgt work show <id>
sleep
sgt work show <id>
sleep
sgt work show <id>

Polling makes the harness remain the scheduler. It also encourages repeated tool calls, repeated model turns, arbitrary delay intervals, and orchestration logic reconstructed independently by every harness.

Sergeant already has the correct lower-level mechanism:

append-only journal
    ↓
durable event publication
    ↓
GET /v1/events/stream
    ↓
resume by journal sequence

sgt watch exposes that mechanism as a stable harness-facing CLI contract.

The primary interaction becomes:

sgt run "fix the settlement retry bug" --repo payments-api
# submitted 01K...
sgt --json watch 01K...

The second command blocks silently until the Work:

needs input
becomes blocked
fails
completes
or is canceled

It then emits one current, authoritative Work snapshot and exits.

A harness can invoke that command in the foreground and spend no additional reasoning turns while it waits. A harness with a background-command or monitor facility can run it asynchronously, continue talking to the operator, and receive the same output later.

For a continuously operating Captain role:

sgt --json watch --follow

subscribes to future attention and terminal transitions across the estate.

Sergeant does not decide how a harness backgrounds the process, how stdout becomes model context, or whether the event wakes a dormant session. Those are harness-adapter and session-lifecycle concerns.

The command’s responsibility ends here:

Wait for durable Sergeant state to change, then report the current authoritative state through a stable process interface.

The central rule is:

The event triggers the read. The Work snapshot carries the meaning.

⸻

2. Audit Basis and Current Repository State

2.1 Existing event surface

Current main already exposes:

GET /v1/events
GET /v1/events/stream

The live endpoint already provides the hard subscription properties:

* authenticated loopback transport;
* SSE framing;
* event IDs equal to journal sequence numbers;
* resume through Last-Event-ID or from;
* attachment to the live broadcast before history replay;
* sequence-based deduplication at the history/live boundary;
* journal refill when a subscriber falls behind the broadcast buffer;
* clean termination when the daemon shuts down;
* no alternate runtime state outside the journal and projections.

No new event bus, database, message broker, or daemon-side subscription object is required.

2.2 Existing client support

ApiClient already provides:

stream_events(from: u64) -> EventStream

and EventStream already decodes the daemon’s SSE stream into the stable Event envelope.

The CLI currently exposes commands for:

submit
list
show
transcript
respond
retry
extend
cancel
status
analytics
doctor
estate management

It does not expose the existing live event stream as a blocking command.

2.3 Existing client doctrine

The TUI establishes the correct client behavior:

An event does not carry the screen’s state. It says something changed, and the client re-reads the API.

sgt watch adopts the same rule.

It will not maintain an independent Work reducer, reinterpret arbitrary event payloads, or present an event as if it were the current Work record.

2.4 Existing AgentOS loop

AGENTS.md already tells the harness to:

1. load estate context;
2. inspect existing Work;
3. shape intent;
4. select a workflow;
5. submit with sgt run;
6. monitor;
7. respond to genuine judgment gates;
8. collect the outcome.

sgt watch changes step 6 from a polling practice into a first-class subscription surface.

Decision WATCH-01 — Rung 2, already in the codebase: Implement sgt watch as a projection over the existing event stream and Work APIs. Do not add a second subscription mechanism.

⸻

3. Product Intent

sgt watch exists for an active harness that is coordinating Sergeant Work.

That harness may be:

* Claude Code;
* Codex;
* OpenCode;
* Goose;
* another agent CLI;
* a shell script;
* a future Sergeant adapter;
* or a human terminal.

The command does not identify or configure the consumer.

It presents a normal process contract:

stdin       unused
stdout      matching notices
stderr      diagnostics
exit status command outcome
lifetime    until a match, signal, or stream failure

The harness chooses how to use that contract.

3.1 Foreground wait

Captain delegates Work
    ↓
Captain invokes `sgt watch <id>`
    ↓
tool process remains blocked
    ↓
Sergeant executes independently
    ↓
matching transition occurs
    ↓
watch prints one notice and exits
    ↓
Captain resumes reasoning

While the command is blocked:

* Sergeant does not poll;
* the command does not invoke a model;
* the daemon continues executing the Work;
* the waiting harness need not spend additional reasoning turns.

Actual token accounting remains a property of the consuming harness, not a promise made by sgt.

3.2 Background subscription

A harness capable of supervising a background process may invoke:

sgt --json watch <id> --follow

or:

sgt --json watch --follow

It may then continue its conversation with the operator while the command remains attached.

sgt watch does not implement the background process mechanism. It only provides a quiet, line-oriented process suitable for one.

3.3 Captain behavior

A Captain session may:

delegate Work A
delegate Work B
continue talking to the operator
receive a watch notice for Work A
inspect/adjudicate Work A
continue or respond
receive a watch notice for Work B

The command enables orchestration without requiring the harness to remain the execution scheduler.

⸻

4. Explicit Non-Goals

This proposal does not add:

Excluded capability	Reason
Waking or launching a harness	No recipient exists when a harness session is closed
Resuming a Claude/Codex/OpenCode session	Harness lifecycle belongs to an adapter or supervisor
MCP configuration	Subscription must work after an ordinary clone and harness launch
MCP Channels	A possible future harness adapter, not the Sergeant contract
Webhooks or callbacks	No external delivery consumer has been admitted
Desktop or mobile notifications	Presentation layer concern
Email, Discord, ntfy, or Slack delivery	Connector concern
Durable subscription identity	No consumer currently requires server-side subscription state
Notification acknowledgment	Sergeant Work state remains authoritative
A notification inbox	The fleet and journal already preserve state
A new event family	Existing work.* transitions already express the needed facts
A raw journal tail command	This command serves orchestration, not low-level debugging
A polling fallback inside sgt watch	The live event API already exists
Automatic daemon restart after stream closure	That would undermine an intentional sgt daemon stop
Arbitrary filter language	No current consumer requires one
Workflow changes	A workflow does not need to know that a client is watching
Backend changes	Claude, Docker, fake, and future backends remain unaware of watchers

The command is deliberately not an alerting subsystem.

A watch process is an attached client, not a durable delivery promise.

⸻

5. Command Contract

5.1 Grammar

sgt watch [WORK_ID] [--follow]

The existing global flags continue to apply:

--json
--data-dir <dir>

Examples:

# Wait for one delegated Work to need attention or finish.
sgt watch 01KZWE3VE3QM3VZ8ES5GM5JF6J
# Stable machine-readable form for a harness.
sgt --json watch 01KZWE3VE3QM3VZ8ES5GM5JF6J
# Continue watching the same Work after nonterminal notices.
sgt --json watch 01KZWE3VE3QM3VZ8ES5GM5JF6J --follow
# Wait for the next future attention/result transition anywhere in the estate.
sgt --json watch
# Continuous estate-wide Captain subscription.
sgt --json watch --follow

5.2 Why sgt watch is top-level

The command can be scoped to one Work, but it can also subscribe across the estate.

Therefore:

sgt work watch <id>

would falsely imply that a Work ID is always required.

sgt watch is parallel to:

sgt status
sgt analytics
sgt web

It is a client surface over the daemon as a whole, optionally narrowed to one Work.

Decision WATCH-02 — Rung 7, minimum that works: Add one top-level command with one optional Work ID and one lifetime flag. Do not begin with subcommands, a query language, or subscription configuration.

⸻

6. Matching Semantics

6.1 Default watch set

The initial command watches for these Work states:

needs_input
blocked
failed
completed
canceled

These states mean one of two things:

Captain attention may be required
    needs_input
    blocked
    failed
the delegated Work has ended
    completed
    canceled

The command does not emit for:

pending
active
waiting

Those are ordinary execution states.

waiting in particular is not automatically a human judgment gate. It represents an external condition and remains nonterminal.

6.2 Fixed initial vocabulary

Version 1 does not add:

--state
--kind
--event
--filter
--where

The initial consumer need is clear: tell the orchestrating harness when delegated Work needs attention or has produced an outcome.

A generic filter language would expose internal event taxonomy before a second use case has earned it.

A future requirement may add state selection without changing the basic output contract.

Decision WATCH-03 — Rung 7: Begin with the five orchestration-relevant states rather than a general-purpose event-filtering CLI.

6.3 One-shot is the default

Without --follow, the command:

1. waits for the first matching notice;
2. writes exactly one notice;
3. exits successfully.

This is the preferred tool-call behavior:

call tool
    ↓
wait without model turns
    ↓
receive one current result
    ↓
resume reasoning

The harness can re-arm the command after responding to needs_input, retrying a failed Work, or beginning another delegation.

6.4 Follow mode

With --follow, the command remains attached after nonterminal matches.

For a Work-scoped watcher:

needs_input → emit and continue
blocked     → emit and continue
failed      → emit and continue
completed   → emit and exit 0
canceled    → emit and exit 0

failed remains watchable because Sergeant permits an explicit retry from that state.

For an estate-wide watcher, --follow remains attached until:

* the caller interrupts it;
* the stream closes;
* or an unrecoverable client/API error occurs.

6.5 Work-scoped initial state

A Work-scoped watcher is level-aware.

After establishing a race-free event cursor, it reads the Work’s current state.

When the Work already matches the watch set:

sgt watch <id>

emits a current_state notice immediately.

This covers:

* a Work that completed before the harness invoked watch;
* a Captain session returning after a delay;
* a command being re-run after an earlier watch process ended;
* a daemon restart followed by a fresh watch invocation.

An unknown Work ID fails immediately with the existing structured work_not_found error rendered through the CLI.

6.6 Estate-wide initial state

An estate-wide watcher is edge-triggered.

It begins at the current journal head and reports future matching transitions only.

It does not replay every historical completed or canceled Work in the estate.

The existing Captain loop already requires:

sgt status
sgt work list

before new orchestration begins. Those commands reconcile the current fleet. sgt watch then observes what changes after that reconciliation.

This preserves separate responsibilities:

status/list    What currently exists?
watch          What changes from here?
journal        What happened historically?

Decision WATCH-04 — Rung 7: A scoped watch checks current state; an unscoped watch begins at the current journal head. This avoids both missed scoped results and an estate-wide flood of historical completions.

⸻

7. Event and Snapshot Doctrine

A watch notice is not the raw event.

The event’s responsibility is:

identify that relevant state changed
provide sequence and provenance
cause the client to re-read the Work

The Work API’s responsibility is:

state the current Work condition
identify the current stage
identify the execution and backend
expose the output pointer
expose the turn envelope
carry the current question or blocking detail

The watch loop therefore follows:

matching work.* event
    ↓
GET /v1/work/{id}
    ↓
confirm current state still matches
    ↓
emit current Work snapshot

This handles rapid transitions honestly.

Example:

work.needs_input event arrives
operator answers through another client immediately
Work resumes and completes
watch processes the older event

The command must not emit a stale claim that the Work still needs input.

It re-reads the current snapshot. When the Work is now completed, the notice reports completion. The event remains provenance for why the client woke, but it does not override current state.

Events invalidate. Snapshots describe.

This is the same client contract already used by the TUI.

Decision WATCH-05 — Rung 2: Reuse the TUI’s event-as-invalidation pattern instead of adding a second client-side Work reducer.

⸻

8. Race-Free Subscription Sequence

8.1 Work-scoped sequence

A scoped watch must not miss a transition between checking current state and attaching to the event stream.

The correct order is:

1. GET /v1/system
      obtain journal head H
2. GET /v1/events/stream?from=H
      server attaches live first
      server replays everything after H
3. GET /v1/work/{id}
      inspect current authoritative state
4. if current state matches
      emit current_state
      exit or continue according to --follow
5. consume events after H

The existing SSE implementation already closes the history/live race:

subscribe to broadcast
    ↓
read journal history after H
    ↓
deduplicate by sequence
    ↓
continue live

Therefore a transition can occur at any point during steps 1–3 without being lost.

8.2 Estate-wide sequence

An estate-wide watch requires no initial Work snapshot:

1. GET /v1/system
      obtain journal head H
2. GET /v1/events/stream?from=H
3. consume future matching transitions

8.3 Lag behavior

When the server-side broadcast receiver falls behind, the existing stream implementation refills from the authoritative journal after the last sent sequence.

sgt watch adds no independent replay logic.

8.4 Duplicate suppression

The history/live seam is already sequence-deduplicated by the daemon.

The watch client must additionally avoid duplicate notices when several queued events lead to the same current snapshot.

For each watched Work, it retains only an in-memory notice fingerprint:

state
current stage id
current stage attempt
current stage status

When a later trigger resolves to the same fingerprint, no second notice is emitted.

This state:

* exists only inside the watch process;
* is never journaled;
* does not survive process exit;
* does not become subscription acknowledgment;
* does not claim authority over Work state.

⸻

9. Output Contract

9.1 JSON mode is JSON Lines

For a streaming command, one giant JSON document cannot be completed until the process exits.

Therefore the existing global --json flag means:

One complete compact JSON object per emitted watch notice.

This is NDJSON/JSONL on stdout.

No opening array, closing array, commas between records, progress messages, or keep-alive lines are written.

A one-shot watch emits one JSON object.

A follow watch emits zero or more independently parseable objects.

9.2 Watch notice schema

{
  "schema": "sergeant.watch/v1",
  "reason": "state_transition",
  "trigger": {
    "seq": 1842,
    "id": "01K...",
    "timestamp": "2026-08-13T18:42:31.417Z",
    "kind": "work.completed"
  },
  "snapshot": {
    "work": {
      "id": "01K...",
      "state": "completed",
      "intent": "fix the settlement retry bug"
    },
    "stage": {
      "stage_id": "30-close",
      "index": 3,
      "attempt": 1,
      "status": "completed",
      "of": 4
    },
    "workflow": {
      "name": "software-change",
      "version": "1"
    },
    "backend": "claude",
    "output": {
      "repositories": []
    },
    "envelope": {
      "turns_spawned": 4,
      "turn_cap": 12,
      "turn_ceiling_secs": 900
    }
  }
}

For an already-matching scoped Work:

{
  "schema": "sergeant.watch/v1",
  "reason": "current_state",
  "trigger": null,
  "snapshot": {
    "...": "the current GET /v1/work/{id} response"
  }
}

9.3 Snapshot ownership

snapshot is the complete current body returned by:

GET /v1/work/{id}

The watch command does not invent a reduced parallel Work schema.

This has two benefits:

1. a harness receives the output pointer, current stage, question/detail, backend, surface, execution, and envelope without an immediate second command;
2. future additive fields on the Work view automatically become available without revising sergeant.watch/v1.

The wrapper schema defines subscription provenance. The nested snapshot remains the Work API contract.

9.4 Trigger minimization

trigger contains only stable event-envelope provenance:

seq
id
timestamp
kind

It does not copy the arbitrary event payload into the harness-facing notice.

This prevents a raw backend, tool, transcript, or future event payload from entering the orchestrator context merely because it happened to trigger the read.

When more evidence is needed, Captain explicitly requests:

sgt work show <id>
sgt work transcript <id>

9.5 Human output

Without --json, each notice is one concise line:

01K...  completed    30-close#1   fix the settlement retry bug
01K...  needs_input  20-review#1  which retry policy should govern this adapter?
01K...  blocked      10-implement dependency contract could not be reconciled

Human rendering may include a compact output-pointer suffix for completed Work.

It must:

* normalize embedded whitespace;
* remain one physical stdout line per notice;
* avoid printing raw event payloads;
* avoid progress or heartbeat lines.

9.6 Stdout and stderr discipline

stdout
    watch notices only
stderr
    connection errors
    malformed stream errors
    invalid Work IDs
    stream-closed diagnostics
    argument errors

A harness using --json can treat every stdout line as protocol.

Decision WATCH-06 — Rung 2: Extend the existing global --json convention as JSONL for the one command whose response is inherently streaming. Do not add a competing --jsonl flag.

⸻

10. Silence While Waiting

A successful attachment prints nothing until a matching notice exists.

It does not print:

connected
watching…
heartbeat
still waiting
received irrelevant event
reconnecting

SSE keep-alive comments remain transport details and produce no CLI output.

This matters for both humans and harnesses:

* a foreground tool remains quiet while blocked;
* a background monitor does not repeatedly inject noise;
* stdout activity means organizationally relevant state changed;
* no model is invited to reason about heartbeat text.

A later --verbose diagnostic mode may be added if a measured debugging need appears. It is not part of this proposal.

⸻

11. Stream Closure and Failure Semantics

11.1 No automatic daemon restart

The existing CLI normally auto-spawns a missing daemon when a command begins.

That rule still applies when sgt watch starts.

Once attached, however, the watcher does not automatically spawn or reconnect to a replacement daemon when the stream closes.

Automatic reconnection could produce:

operator runs `sgt daemon stop`
    ↓
daemon shuts down cleanly
    ↓
watch client sees stream close
    ↓
watch client immediately starts a new daemon

That would make clean shutdown impossible while a watcher exists.

On stream closure before successful one-shot completion:

stderr:
sgt: watch stream closed after journal seq 1842; rerun `sgt watch` to resubscribe
exit:
nonzero

A harness may choose to re-run the command.

A fresh scoped invocation checks current Work state before waiting, so resubscription is safe without a durable client cursor.

11.2 Malformed frames

SSE comment/keep-alive frames remain ignorable.

A frame containing data: that cannot decode as a Sergeant event is not a keep-alive and must not be silently skipped.

It ends the command with a structured client diagnostic.

This may require tightening EventStream so it distinguishes:

comment frame
valid Event frame
malformed data frame
transport end/error

A subscription that silently drops an undecodable event is not trustworthy.

11.3 Signals

SIGINT, SIGTERM, or harness cancellation simply ends the client process and closes its HTTP connection.

No journal event is written.

No Work is canceled.

No subscription cleanup command is needed because no durable subscription exists.

11.4 Exit behavior

Condition	Exit
One-shot notice emitted	0
Scoped follow reaches completed or canceled	0
Caller interrupts process	Native signal exit
Unknown Work ID	1
API error	1
Stream closes before completion	1
Malformed event frame	1
Invalid CLI syntax	Clap’s existing argument-error exit

⸻

12. Architecture and Ownership

┌────────────────────────────────────────────────────────────┐
│ Harness                                                    │
│ Claude / Codex / OpenCode / Goose / shell                  │
│                                                            │
│ foreground wait or harness-owned background process        │
└───────────────────────────┬────────────────────────────────┘
                            │ stdout / process exit
                            ▼
┌────────────────────────────────────────────────────────────┐
│ `sgt watch`                                                │
│                                                            │
│ scope · match · re-read · render                           │
│ no Work mutation · no durable subscription state           │
└───────────────────────────┬────────────────────────────────┘
                            │ ApiClient
                ┌───────────┴────────────┐
                ▼                        ▼
       /v1/events/stream          /v1/work/{id}
       trigger/provenance         current meaning
                │                        │
                └───────────┬────────────┘
                            ▼
┌────────────────────────────────────────────────────────────┐
│ Sergeant daemon                                            │
│ journal · projection · Work runtime · workflows            │
└────────────────────────────────────────────────────────────┘

Ownership remains:

Core
    durable Work and event truth
API
    subscription and current Work views
CLI watch
    attached filtering and rendering
Harness
    process supervision and orchestration judgment
Harness adapter / future supervisor
    wake, resume, or session lifecycle

12.1 No engine awareness

The engine does not know:

* that a watcher exists;
* how many watchers exist;
* whether a watcher is a human or agent;
* whether a harness is foregrounded;
* whether a notice caused reasoning;
* whether a notice was acknowledged.

Work execution is identical with zero, one, or many watchers.

12.2 No event vocabulary change

The command consumes the current state-transition families:

work.needs_input
work.blocked
work.failed
work.completed
work.canceled

It does not add:

watch.started
watch.delivered
watch.acknowledged
captain.notified

Those would turn a transient client attachment into core domain state.

12.3 No new API endpoint

This proposal does not add:

GET /v1/watch
POST /v1/subscriptions
GET /v1/notifications

The current event stream plus Work view already satisfy the contract.

Decision WATCH-07 — Rung 2: Keep watch entirely on the read/client side. The daemon already exposes the required primitives.

⸻

13. Proposed Implementation Shape

The feature should remain small and independently testable.

13.1 CLI parsing

src/cli.rs adds:

Watch {
    id: Option<String>,
    #[arg(long)]
    follow: bool,
}

The dispatch arm:

1. resolves the existing data directory;
2. uses the existing daemon auto-spawn path;
3. constructs watch options;
4. delegates to the watch loop;
5. renders notices according to the global --json flag.

13.2 Watch module

A focused module such as:

src/watch.rs

owns:

WatchOptions
WatchNotice
WatchReason
WatchTrigger
watch loop
target-state classification
notice fingerprinting
human rendering
JSON serialization

It depends on:

ApiClient
Event / EventStream
serde
serde_json

It does not depend on:

Journal
Projection
Engine
Backend
Daemon internals
DuckDB
Git
Docker

The module boundary prevents a long-running control loop from turning cli.rs into another execution layer.

13.3 API client tightening

The existing client can be reused.

The only likely API-client adjustment is making stream consumption distinguish:

valid event
keep-alive/comment
malformed data frame
clean/transport closure

No daemon route or response shape changes.

13.4 Dependencies

No new crate is required.

Current dependencies already provide:

* HTTP;
* SSE byte streaming;
* async execution;
* JSON;
* CLI parsing;
* event types;
* signal/process behavior.

Decision WATCH-08 — Rung 2: Use existing Rust dependencies and existing API types. Add no event-stream or messaging library.

⸻

14. Harness Integration Contract

sgt watch is intentionally weaker and more portable than an MCP tool.

A harness needs only the ability to execute a command.

14.1 Foreground harness contract

execute:
    sgt --json watch <work-id>
expect:
    silence until one notice
    one JSON object on stdout
    process exit 0

14.2 Background harness contract

execute in harness-owned background mode:
    sgt --json watch --follow
expect:
    zero or more JSON objects
    one object per line
    no heartbeat output
    process remains attached

14.3 What the harness does after a notice

notice received
    ↓
read snapshot
    ↓
apply Captain decision ladder
    ├── completed
    │     inspect output and acceptance
    ├── needs_input
    │     answer locally or escalate operator judgment
    ├── blocked
    │     retry, reroute, remediate, or escalate
    ├── failed
    │     inspect evidence and adjudicate retry
    └── canceled
          reconcile expected versus actual cancellation

needs_input does not automatically mean “ask the human.”

It means a lower execution layer has stopped for input. Captain still determines whether:

* existing intent resolves it;
* an established contract resolves it;
* Captain has delegated authority to answer;
* or the operator is genuinely required.

14.4 No harness-specific content in core

The CLI does not emit:

Claude tool syntax
Codex notifications
OpenCode events
MCP messages
plugin manifests

A harness-specific adapter may later wrap this command without changing it.

⸻

15. Documentation Changes

15.1 AGENTS.md

The standard loop should replace polling-shaped monitoring guidance with:

After submitting durable Work, use `sgt --json watch <id>` when this
session should wait for the next attention or terminal transition.
A harness may use its own background-command facility with `--follow`
when conversation should continue while Work runs.
A watch notice is a trigger. Use its current Work snapshot and the
ordinary `show`/`transcript` surfaces for adjudication; do not interpret
raw event payloads or assume `needs_input` must be relayed to the operator.

The initial fleet steps remain unchanged:

sgt status
sgt work list

watch does not replace reconciliation of already-existing Work.

15.2 README.md

The day-to-day section gains:

sgt watch <id>          # wait until this Work needs attention or ends
sgt watch --follow      # future attention/result transitions across the estate

The README should say explicitly:

* default is one-shot;
* --json produces one JSON object per line;
* no output means no matching transition has occurred;
* watch does not wake or launch a harness.

15.3 sergeant-help

The operator help skill should route questions about waiting, subscriptions, and avoiding polling to sgt watch.

15.4 No workflow documentation changes

Workflows remain unaware of clients.

⸻

16. Test and Assurance Plan

16.1 Unit tests

The watch module must pin:

* target-state classification;
* current_state versus state_transition notice reasons;
* notice JSON shape;
* one-line human rendering;
* whitespace normalization;
* duplicate fingerprint suppression;
* completed/canceled scoped-follow termination;
* failed/blocked/needs-input scoped-follow continuation;
* no emission for pending/active/waiting;
* raw event payload exclusion.

The event-stream decoder must pin:

* keep-alive comments are ignored;
* valid frames decode;
* chunk boundaries do not affect frames;
* malformed data: frames return an error rather than disappearing;
* stream end is distinguishable from keep-alive.

16.2 Live daemon CLI tests

A dedicated acceptance suite should run the real sgt binary against a real daemon and deterministic fake backend.

W1 — scoped wait is silent

1. Submit a Work that remains active.
2. Spawn sgt --json watch <id>.
3. Assert no stdout notice before a matching transition.
4. Transition Work to needs_input.
5. Assert exactly one JSON line.
6. Assert the line contains the current question-bearing snapshot.
7. Assert the process exits 0.

W2 — already-completed Work returns immediately

1. Complete a Work.
2. Start sgt --json watch <id>.
3. Assert a current_state notice.
4. Assert output pointer and envelope are present.
5. Assert exit 0.

W3 — no snapshot/attach race

Force a matching transition between:

journal-head read
stream attachment
current Work read

Assert the notice is emitted exactly once.

W4 — stale trigger does not produce stale meaning

1. Cause needs_input.
2. Respond immediately and allow completion before the watcher handles the queued event.
3. Assert the notice reports current completion rather than stale needs_input.

W5 — estate watch begins now

1. Create historical completed Work.
2. Start unscoped sgt --json watch.
3. Assert the historical completion is not emitted.
4. Complete a new Work.
5. Assert the new Work is emitted.

W6 — follow mode

1. Start scoped --follow.
2. Produce needs_input.
3. Assert one notice and process remains.
4. Respond and later complete.
5. Assert completion notice and clean exit.

W7 — stream closure is honest

1. Start a watcher.
2. Stop the daemon.
3. Assert the watcher exits nonzero with its last observed sequence.
4. Assert it does not restart the daemon.

W8 — stdout is protocol

For JSON mode:

* every stdout line parses independently;
* stderr text never appears on stdout;
* no startup banner or heartbeat is emitted.

16.3 Architectural tests

Source-level structural checks should establish:

* watch.rs does not import journal, projection, engine, backend, or daemon internals;
* no new API mutation route exists;
* no new event kind contains watch, subscription, or notification;
* the watch client reaches state only through ApiClient.

16.4 Product pilot

The human-facing gate is:

Open Claude Code in the Sergeant estate
    ↓
delegate real Work through sgt
    ↓
Claude invokes sgt watch for the returned Work ID
    ↓
no polling commands occur
    ↓
Sergeant runs independently
    ↓
watch returns needs_input or a terminal result
    ↓
Claude correctly resumes Captain behavior

The gate tests the CLI process contract only.

Whether Claude ran it through a foreground Bash call, background task, Monitor, plugin, or another facility is recorded as environment evidence but does not change the Sergeant acceptance result.

⸻

17. Acceptance Criteria

The proposal is satisfied when all of the following are true:

1. sgt watch <id> blocks without polling and returns the next current attention or terminal state.
2. A scoped watch returns immediately when the Work already matches.
3. sgt watch with no ID observes future matching transitions across the estate without replaying historical completions.
4. Default mode emits once and exits.
5. --follow supports a continuous attached consumer.
6. --json emits valid, one-object-per-line sergeant.watch/v1 notices.
7. The notice contains the authoritative current Work API snapshot.
8. The trigger includes event provenance but excludes arbitrary payload.
9. The snapshot/stream attachment sequence cannot lose a transition.
10. Stream lag remains recoverable through the existing journal refill.
11. Stream closure is visible and nonzero.
12. A watcher never auto-restarts a deliberately stopped daemon.
13. No new daemon route, event family, database, dependency, or durable subscription record is introduced.
14. Work executes identically with or without a watcher.
15. The feature works from an ordinary harness shell without MCP or pre-launch configuration.

The falsifier is:

A Work reaches a matching durable state while a correctly attached watch process remains silent, emits stale state as current, or requires polling to recover the result.

⸻

18. Alternatives Considered

18.1 Poll sgt work show

Rejected.

while active:
    sleep N
    show

creates arbitrary latency, repeated processes, repeated reads, and a scheduler loop in every harness.

The event stream already exists.

18.2 Add an MCP server

Rejected as the base contract.

MCP would require harness configuration and approval before or during launch. It would make an otherwise generic subscription depend on one tool protocol.

A future MCP adapter may invoke or project the same watch contract.

18.3 Add an MCP Channel or harness plugin

Deferred.

Those may improve delivery into a specific running harness, but they answer:

How does this harness receive the output?

This proposal answers:

What stable Sergeant process can the harness subscribe to?

18.4 Add callbacks or webhooks

Rejected.

Callbacks require destination registration, retries, authentication, delivery state, and failure policy. No current consumer has earned that machinery.

18.5 Add /v1/watch

Rejected.

The current event stream and Work view already provide the two required primitives.

A new endpoint would duplicate filtering and snapshot semantics inside the daemon.

18.6 Print raw events

Rejected.

Raw event flow is too broad for orchestration and may contain backend, tool, usage, or conversation details irrelevant to Captain’s decision.

The event is provenance, not the result.

18.7 Auto-reconnect forever

Rejected.

A client that respawns the daemon after an intentional stop violates daemon lifecycle authority.

Fresh invocation is already safe because scoped watch checks current state and the journal preserves history.

18.8 Store durable subscriptions

Rejected.

A running process already represents the attached subscription. No admitted requirement needs server-side subscriber identity, cursors, leases, acknowledgment, or delivery history.

18.9 Add configurable state filters now

Deferred.

The five default states exactly cover the current Captain loop. A filter surface should be added only when a second consumer produces a conflicting requirement.

⸻

19. Proposed Delivery Boundary

This is one small vertical product slice:

CLI grammar
    ↓
watch loop over current ApiClient
    ↓
stable notice rendering
    ↓
tests
    ↓
AGENTS/README/help adoption

It does not require:

* an adapter redesign;
* a new workflow;
* journal query implementation;
* TUI redesign;
* daemon scheduling;
* callback infrastructure;
* Claude-specific code;
* or a wake supervisor.

The implementation should be reviewed as a client-surface addition, not a new execution subsystem.

⸻

20. Ponytail Decision Register

The rung is the lowest viable resolution, not the importance of the decision.

ID	Rung	Decision	Why this rung
WATCH-01	R2	Build on /v1/events/stream and current Work APIs	Subscription, replay, sequence, and live delivery already exist
WATCH-02	R7	Add top-level sgt watch [WORK_ID] [--follow]	No existing CLI command exposes the stream; this is the smallest usable grammar
WATCH-03	R7	Watch five fixed orchestration-relevant states	A general filter language is not required for the admitted use case
WATCH-04	R7	Default to one-shot; make continuous attachment explicit with --follow	Foreground tool waiting is the primary use case; persistence must be requested
WATCH-05	R2	Treat events as invalidation and re-read /v1/work/{id}	The TUI already proves this client pattern and prevents a second reducer
WATCH-06	R7	Scoped watch checks current state; estate watch begins at current head	Prevents missed scoped results without replaying an estate’s entire terminal history
WATCH-07	R2	Use global --json as JSONL for streaming output	Existing machine-readable convention; arrays cannot represent an open stream
WATCH-08	R7	Emit minimal trigger provenance plus the complete current Work snapshot	Enough for orchestration without copying arbitrary event payload
WATCH-09	R2	Use current server-side sequence replay, deduplication, and lag refill	The daemon already owns this reliability mechanism
WATCH-10	R7	Exit visibly on stream closure rather than auto-restarting	Preserves sgt daemon stop authority and avoids hidden lifecycle behavior
WATCH-11	R1	Exclude wake, callbacks, MCP requirements, and durable subscriptions	None is required to let an already-running harness subscribe
WATCH-12	R2	Add no crate dependency and no daemon route	Current dependencies and API already perform every required primitive

⸻

21. Final Recommendation

Implement sgt watch as a quiet, read-only CLI client over the existing SSE and Work inspection surfaces.

Its default contract should be:

optional Work scope
+
five attention/terminal states
+
one-shot wait
+
current authoritative snapshot
+
stable JSONL

Its continuous contract should be:

the same command
+
--follow

The resulting Captain loop is:

operator expresses intent
    ↓
Captain shapes and delegates Work
    ↓
Sergeant executes durably
    ↓
Captain waits or continues conversation
    ↓
sgt watch reports a current attention/result snapshot
    ↓
Captain adjudicates the next action

No MCP is required.

No polling is required.

No harness-specific code is required.

No wake mechanism is implied.

sgt run is the delegation boundary. sgt watch is the return path.
