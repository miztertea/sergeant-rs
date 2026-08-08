# Depot
## Proposal for a Rust-Native Agent Execution Surface

**Status:** Proposed  
**Scope:** Depot only  
**Relationship to Sergeant:** Clean-room successor informed by Sergeant, not a fork  
**Primary implementation:** Rust  
**Primary deployment:** Local user daemon with native agent backends  
**Primary interface model:** One execution surface, many clients

---

# 1. Executive Summary

**Depot is a local agent execution runtime.**

It receives intent, constructs an isolated work surface across one or more Git repositories, routes that work to a native agent harness, observes the resulting execution as completely as the harness permits, preserves the complete execution trajectory, and exposes that state through a common client API.

Depot is not another model API wrapper.

Depot does not implement Claude, Codex, OpenCode, or Prime Agent's reasoning loops. Those systems increasingly provide durable native processes, sessions, threads, event streams, histories, and authentication mechanisms of their own.

Depot sits above them.

```text
                         CLIENTS

       Claude        Codex       OpenCode      Terminal
       skill          skill        skill        / CLI
          │              │            │             │
          └──────────────┴──────┬─────┴─────────────┘
                                │
                           Depot API
                                │
                         ┌──────┴──────┐
                         │ Depot daemon│
                         │             │
                         │ work        │
                         │ routing     │
                         │ workflows   │
                         │ trajectory  │
                         │ recovery    │
                         │ observation │
                         └──────┬──────┘
                                │
                ┌───────────────┼────────────────┐
                │               │                │
              Claude          Codex          OpenCode
             sessions         threads         sessions
                │               │                │
          native runtime    App Server      native server
                                │
                                └──────────── Prime Agent
                                               sessions
                                               daemon
```

The daemon is the application.

The CLI, TUI, embedded HTML dashboard, MCP server, agent skills, and future clients are projections over the daemon.

A user should eventually be able to:

```text
git clone <repo>
cd <repo>
claude
```

and tell Claude:

> Implement the authentication change and have Depot validate it independently.

Claude's repository instructions and Depot skill submit the work to the local daemon. Because the request originated from Claude, Depot defaults the execution backend to Claude using the user's existing native Claude authentication. The user may override the backend, profile, model, workflow, or work surface explicitly.

The same repository could instead be opened in Codex or OpenCode without changing Depot's execution contracts.

---

# 2. Why Depot Exists

Sergeant proved that local multi-agent software execution is useful.

It also discovered the hard parts.

The hard parts were not fundamentally:

- starting another shell;
- making a worktree;
- sending a prompt.

The hard parts were:

- execution identity;
- ownership;
- durable delivery;
- recovery after interruption;
- distinguishing process state from work state;
- waiting and human input;
- exact retries;
- cleanup;
- observability;
- correlating work across repositories;
- maintaining enough evidence to know what actually happened.

Those concerns accumulated inside Bash scripts, tmux panes, filesystem sentinels, background watcher processes, action leases, process-group checks, generated briefs, retry logic, callback state, and increasingly numerous supporting executables.

Recent Sergeant failure modes we examined were overwhelmingly consequences of those boundaries: stale or ambiguous process identity, orphaned work with incomplete leases, terminal workers that could not safely converge, unreliable tmux activity witnesses, and installed components disagreeing about their contract revisions.

Depot does not translate those mechanisms into Rust.

It replaces the environment that made them necessary.

The existing Sergeant repository remains valuable as:

```text
reference implementation
+
requirements mine
+
regression-test oracle
+
failure-mode catalog
```

It does not become Depot's codebase.

---

# 3. Product Definition

Depot is:

> **A durable local execution surface for routing, supervising, observing, recovering, and understanding agentic work across one or more repositories.**

It knows everything about execution that it can reasonably observe.

That includes both state Depot owns and state Depot observes from the execution harness.

Depot therefore separates three forms of knowledge.

| Category | Meaning | Example |
|---|---|---|
| **Owned** | Depot is authoritative and may mutate it | Work status, selected backend, execution binding, workflow stage |
| **Observed** | Another runtime reported it; Depot records it | Claude waiting state, Codex tool call, OpenCode message, token usage |
| **Referenced** | Another system remains authoritative but Depot knows where the evidence lives | Git commit, native transcript, artifact path, native session file |

The rule is:

> **Know as much as possible without pretending to own what belongs elsewhere.**

---

# 4. Explicit Non-Goals

Depot v1 is not:

| Not Depot | Reason |
|---|---|
| A model gateway | Native harnesses already own model interaction and subscription auth |
| A replacement reasoning loop | Claude/Codex/OpenCode/Prime already do this |
| A distributed cluster scheduler | The first problem is one user's workstation |
| A secrets vault | Native harnesses own their credentials |
| A proprietary identity system | OS user identity and native backend profiles are sufficient initially |
| A general DAG engine | Workflow complexity must earn its way in |
| A Git replacement | Git owns source history |
| A graph database product | The graph is initially a projection |
| A Sergeant compatibility layer | Compatibility would preserve accidental architecture |
| A tmux manager | Native harness processes remove the requirement |

Remote execution, team tenancy, centralized control planes, RBAC, shared credentials, and cross-machine dispatch can be added later without changing the local execution contracts if the boundaries are correct.

---

# 5. Core Architecture

Depot consists of six internal layers.

```text
┌──────────────────────────────────────────────────┐
│                 Client Surface                   │
│ CLI · TUI · HTTP · HTML · MCP · Agent Skills    │
├──────────────────────────────────────────────────┤
│                 Command API                      │
│ submit · inspect · respond · cancel · query      │
├──────────────────────────────────────────────────┤
│                 Work Runtime                     │
│ work · workflows · surfaces · state · recovery   │
├──────────────────────────────────────────────────┤
│                 Backend Router                   │
│ backend · profile · model · native capability    │
├──────────────────────────────────────────────────┤
│             Backend Adapters                     │
│ Claude · Codex · OpenCode · Prime                │
├──────────────────────────────────────────────────┤
│             Native Agent Runtimes                │
│ background processes · servers · sessions        │
└──────────────────────────────────────────────────┘
```

A seventh concern cuts vertically through all six:

```text
Trajectory
events · raw evidence · graph · analytics · OTel
```

---

# 6. The Daemon Is the Application

Depot runs as one long-lived user process.

The first client invocation ensures it exists.

```text
depot
  │
  ├─ daemon reachable?
  │       yes ───────────────► connect
  │
  └─ no
      └─ spawn daemon
          └─ connect
```

Initial service installation is unnecessary.

A later command may install Depot as an OS-native user service:

```text
macOS    → launchd
Linux    → systemd --user
Windows  → user service/background service
```

but auto-spawn is sufficient to prove the architecture.

The daemon is the only process permitted to mutate Depot runtime state.

Clients do not manipulate runtime files directly.

That is one of the most consequential changes from Sergeant.

---

# 7. Client Architecture

Every interface consumes the same logical surface.

```text
                    Depot API

        ┌──────────────┼───────────────┐
        │              │               │
       CLI            TUI             MCP
        │              │               │
      HTML            SDK          Agent skill
        │              │               │
        └──────────────┼───────────────┘
                       │
                    daemon
```

## Canonical Local Transport

Depot v1 uses:

**HTTP/JSON over loopback + Server-Sent Events.**

Rust stack:

```text
axum
tower
tokio
serde
serde_json
```

Why HTTP rather than inventing an IPC protocol:

- cross-platform;
- easy Rust support;
- HTML dashboard naturally consumes it;
- MCP wrapper can consume it;
- CLI/TUI can consume it;
- debugging is trivial;
- SSE is enough for one-way event subscription;
- POST handles command traffic;
- a future remote gateway can proxy the same logical API.

Default binding:

```text
127.0.0.1 only
```

The daemon publishes a local runtime descriptor containing endpoint, PID/runtime identity, API revision, and a random bearer token protected by user filesystem permissions.

No unauthenticated LAN listener exists by default.

---

# 8. API Contract

The logical API begins deliberately small.

```text
Work
  submit
  inspect
  list
  respond
  cancel
  retry
  resume

Executions
  inspect
  interrupt
  stop

Backends
  list
  capabilities
  health

Profiles
  list
  inspect

Events
  history
  subscribe

Graph
  neighborhood
  trajectory

System
  health
  version
  capabilities
```

Representative HTTP projection:

```text
POST /v1/work
GET  /v1/work
GET  /v1/work/{id}

POST /v1/work/{id}/input
POST /v1/work/{id}/cancel
POST /v1/work/{id}/resume

GET  /v1/executions/{id}

GET  /v1/backends
GET  /v1/profiles

GET  /v1/events
GET  /v1/events/stream

GET  /v1/graph/work/{id}

GET  /healthz
GET  /v1/system
```

Those paths are an API rendering, not the internal Rust domain model.

---

# 9. Core Domain Model

Depot's initial persistent concepts are:

```text
Workspace
Repository
Work
WorkflowRun
StageRun
Execution
Backend
Profile
Artifact
Event
```

## Workspace

A workspace is the repository surface from which work originates.

A workspace may contain one repository or many.

Single-repository use requires **zero configuration**.

Depot discovers:

```text
git rev-parse --show-toplevel
```

and treats that repository as the workspace.

Multi-repository use adds an optional checked-in:

```text
depot.toml
```

Example:

```toml
[workspace]
name = "payments"

[[repository]]
name = "api"
path = "../payments-api"

[[repository]]
name = "web"
path = "../payments-web"

[[repository]]
name = "infra"
path = "../payments-infra"
```

`depot.toml` declares topology and defaults.

It never stores transient work state.

---

# 10. Work

A Work record represents one durable unit of intent accepted by Depot.

Conceptually:

```text
Work
  id
  workspace
  intent
  targeted repositories
  workflow
  state
  created_by
  created_at
```

Core work states remain intentionally small:

```text
pending
active
waiting
needs_input
blocked
completed
failed
canceled
```

Workflow stage is orthogonal.

This prevents:

```text
"in implementation"
"in review"
"in merge"
```

from becoming unrelated top-level state-machine values.

---

# 11. Work Surfaces

Execution occurs inside a materialized work surface.

For a one-repository assignment:

```text
Work Surface
└── repo worktree
```

For multi-repository work:

```text
Work Surface
├── api/
│   └── Git worktree
├── web/
│   └── Git worktree
└── infra/
    └── Git worktree
```

Each repository binding records:

```text
source repository
base branch
base SHA
worktree path
work branch
current HEAD
```

Depot shells out to the installed Git CLI rather than embedding libgit2 initially.

Git already defines the behavior we want.

Depot's responsibility is deterministic orchestration around it.

Runtime work surfaces live outside the source checkout in Depot's user data directory.

Checked-in repository files remain declarative.

---

# 12. Workflow Boundary

Sergeant currently embeds substantial procedure inside shell scripts.

Depot moves procedure out of runtime code.

A workflow is versioned filesystem content.

Initial structure follows the useful ICM stage model:

```text
.depot/
└── workflows/
    └── software-change/
        ├── workflow.toml
        ├── 00-prepare/
        │   └── CONTEXT.md
        ├── 10-implement/
        │   └── CONTEXT.md
        ├── 20-review/
        │   └── CONTEXT.md
        └── 30-close/
            └── CONTEXT.md
```

`workflow.toml` is machine-readable.

`CONTEXT.md` is actor-readable.

Depot v1 supports:

```text
ordered stages
explicit entry
explicit completion
waiting
needs input
blocked
retry
failure
cancellation
```

It does **not** begin as a generalized DAG scheduler.

Workflow code calls deterministic Depot capabilities.

For example:

```text
surface.prepare
execution.start
execution.send
artifact.require
git.inspect
stage.complete
execution.stop
```

Procedure belongs in the workflow.

Mechanics belong in Depot.

Reasoning belongs in the execution backend.

---

# 13. Frontend Affinity and Routing

One of Depot's nicest properties is that the front-end harness can naturally select the execution backend.

Every client request carries origin metadata.

```json
{
  "origin": {
    "client": "claude"
  }
}
```

Routing precedence:

```text
explicit --backend
        ↓
origin/client affinity
        ↓
workspace default
        ↓
global default
        ↓
fail with available options
```

Depot never silently moves work to a different commercial provider merely because one backend is unavailable.

Thus:

```text
Claude front end   → Claude backend
Codex front end    → Codex backend
OpenCode front end → OpenCode backend
Prime front end    → Prime backend
Terminal           → configured default
```

unless explicitly overridden.

Examples:

```bash
depot run "fix the failing authorization tests"
```

```bash
depot run \
  --backend codex \
  "independently review the authorization implementation"
```

```bash
depot run \
  --backend claude \
  --profile enterprise \
  --model claude-opus-4-7 \
  "resolve the review findings"
```

These remain distinct concepts:

```text
origin client
backend
profile
model
workflow
```

They must never collapse into one overloaded `agent` field.

---

# 14. Profiles and Authentication

Depot should **not become an authentication broker**.

The native harness owns credentials.

The user authenticates using the harness's native flow:

```text
Claude    → Claude login / Claude.ai device flow
Codex     → Codex / ChatGPT device flow
OpenCode  → native provider auth
Prime     → Prime native OAuth/provider auth
```

Depot stores only launch/profile configuration.

A profile is a named execution context:

```text
backend
executable
config/home location if needed
environment overrides
default model
runtime options
```

No OAuth refresh token needs to be copied into Depot.

This preserves subscription economics and avoids inventing a credential layer.

Prime Agent is especially instructive here because its own provider implementation already supports subscription OAuth and its daemon is explicitly designed around local session ownership. Its current daemon protocol is versioned JSONL and includes commands, event envelopes, monotonically sequenced events, cursors, replay, snapshots, attach/reattach, and session lifecycle semantics.

---

# 15. Backend Contract

Depot's central extension boundary is not a plugin framework.

It is a small native-runtime contract.

Conceptually:

```text
PROBE
Can this backend operate here?
What version/capabilities exist?

ENSURE RUNTIME
Start or attach to any backend-level service required.

START
Create or adopt one native execution context.

SEND
Deliver work/input to that execution context.

OBSERVE
Report current native evidence.

SUBSCRIBE
Stream native events where supported.

HISTORY
Retrieve durable native history where supported.

INTERRUPT
Stop the current turn/action.

RESUME
Resume an existing context.

STOP
Retire active execution without corrupting recoverable state.
```

Each backend also advertises capabilities.

```text
persistent_sessions
native_background
streaming
history
resume
interrupt
model_selection
profiles
approval_flow
human_attach
usage
native_subagents
```

Depot never requires every backend to support every capability.

Missing capability means:

```text
unsupported
```

not emulation unless emulation is proven necessary.

---

# 16. Native Backend Implementations

## Claude

Depot targets the native Claude Code execution surface rather than a foreground REPL.

The adapter will use the background-session interface already being exercised in our design (`--bg` and resume semantics), with capability/version probes and contract tests preventing unsupported CLI versions from launching work.

Claude's structured JSON/JSONL material is particularly valuable as raw trajectory evidence.

Depot should ingest it where stable, but private Claude filesystem layouts must remain adapter details rather than core contracts.

Conceptually:

```text
Depot
  │
  ├─ launch native Claude background session
  ├─ retain Claude session identity
  ├─ resume/send later turns
  ├─ inspect native state
  ├─ ingest structured Claude events/logs
  └─ stop session explicitly when Depot retires execution
```

---

## Codex

Codex App Server is almost exactly the execution interface Depot wants.

Its protocol supports durable threads, resume, turns, interrupt, persisted history, and streaming JSON-RPC notifications including turn lifecycle, agent-message deltas, command executions, file changes, approvals, diffs, and token usage.

Mapping:

```text
Depot Execution → Codex thread
Depot Input     → turn/start
Interrupt       → turn/interrupt
History         → thread/read / turns / items
Events          → JSON-RPC notifications
```

A Codex turn completing is **not** equivalent to Depot Work completing.

It means only that one native turn completed.

---

## OpenCode

OpenCode's server architecture is likewise naturally compatible.

`opencode serve` exposes a headless HTTP server. Its API supports sessions, session status, message history, asynchronous prompts, abort, diffs, permission responses, and an SSE event stream. OpenCode's own TUI is already a client of this server.

Mapping:

```text
Depot Execution → OpenCode session
Depot Input     → prompt_async
Interrupt       → session abort
History         → session message API
Events          → /event SSE
```

Depot should therefore never drive OpenCode by sending keystrokes to its TUI.

---

## Prime Agent

Prime Agent demonstrates perhaps the closest architectural relative.

Its current daemon protocol explicitly describes itself as a **local daemon JSONL protocol** and defines:

```text
versioned command envelopes
event envelopes
event sequences
resume cursors
event replay
attach snapshots
session create/attach/reattach
prompt
kill
resident/client-owned lifecycle
```

Depot should connect to that native daemon rather than treating Prime Agent as a foreground terminal program.

Prime's internal subagent/session tree remains Prime's concern.

Depot records it observationally where useful.

---

# 17. Runtime Scope

Different harnesses have different daemon models.

Depot core must not assume:

```text
one backend daemon per worker
```

or:

```text
one global daemon per backend
```

Adapters declare their runtime scope.

```text
external
per-profile
per-workspace
per-execution
```

Examples conceptually:

```text
Claude
  native supervisor/session infrastructure
  execution = native background session

Codex
  App Server process
  many threads

OpenCode
  server instance
  many sessions

Prime
  native daemon
  many sessions
```

This keeps native platform process ownership where it belongs.

---

# 18. State and Trajectory

Depot's conceptual persistence model is:

> **State change produces an immutable event. Current state is a projection of those events.**

Depot should preserve much more than top-level lifecycle state.

The execution trajectory may contain:

```text
work
workflow
stage
execution
native session
conversation
messages
tool activity
approvals
usage
file changes
Git changes
artifacts
findings
waits
human responses
retries
errors
recovery
completion
```

That makes Depot a durable execution-memory system rather than merely a worker launcher.

---

# 19. Event Contract

Every event uses a stable envelope.

Example:

```json
{
  "schema": "depot.event/v1",
  "seq": 4831,
  "id": "01K...",
  "timestamp": "2026-08-08T01:42:12.437Z",

  "source": {
    "type": "backend",
    "name": "claude"
  },

  "workspace_id": "ws_...",
  "work_id": "work_...",
  "execution_id": "exec_...",

  "correlation_id": "corr_...",
  "causation_id": "evt_...",

  "kind": "tool.completed",

  "payload": {
    "tool": "bash",
    "duration_ms": 1742,
    "exit_code": 0
  }
}
```

IDs use ULIDs.

Sequence is daemon-local and monotonically increasing.

`correlation_id` groups one logical operation.

`causation_id` records why this event happened.

Those two fields become extremely important for later trajectory graphs.

---

# 20. Raw Events and Normalized Events

Depot should preserve both.

```text
Native event
    │
    ├──────────────► raw event archive
    │
    └─ adapter normalization
             │
             ▼
        Depot event stream
```

Never throw away valuable vendor fidelity merely because Depot has a normalized vocabulary.

A Codex file-change event, OpenCode message part, Prime event, or Claude JSONL record may contain information that Depot does not understand yet.

Future Depot versions should be able to reinterpret historical execution.

---

# 21. Storage: JSONL + DuckDB

For the design currently in front of us, the strongest implementation is:

```text
JSONL = durable trajectory
DuckDB = rebuildable projection/query engine
filesystem blobs = large evidence
```

## JSONL

Depot maintains append-only segmented event journals.

Example:

```text
<data-dir>/
└── journal/
    ├── 00000001.ndjson
    ├── 00000002.ndjson
    └── ...
```

Only the daemon writes.

Each committed line is one complete event.

A trailing incomplete line after a crash is detectable and recoverable.

Large content does not have to live inline.

---

## Blob Store

Large data receives a content-addressed reference.

```text
blobs/
└── b3/
    └── <blake3-hash>
```

Useful for:

```text
large diffs
command output
transcript chunks
screenshots
artifacts
raw payloads
```

The event stores:

```json
{
  "blob_ref": "b3:..."
}
```

This keeps the journal readable and bounded.

---

## DuckDB

Depot embeds DuckDB through its Rust client.

The DuckDB file is a **projection**, not the source of truth.

```text
<data-dir>/
└── projections/
    └── depot.duckdb
```

If deleted:

```text
JSONL
  ↓
replay
  ↓
rebuild depot.duckdb
```

No execution history is lost.

---

# 22. DuckDB Projections

Initial analytical/read models:

```text
events
work
executions
stages
messages
tool_calls
artifacts
usage
repositories
git_changes
graph_nodes
graph_edges
```

Clients do not access DuckDB directly.

They ask the daemon.

This preserves the one-owner architecture.

DuckDB answers questions such as:

```text
Which backend produces the most retries?

How much time is spent waiting for humans?

What models generate the highest validation success rate?

Which workflows produce repeated remediation loops?

How frequently does a tool call precede a failure?

How much agent activity occurs before a meaningful file change?

How long does work remain blocked?

What did this execution touch?
```

---

# 23. The Graph

Depot's graph is initially a **derived temporal projection**, not a graph database.

Representative nodes:

```text
Workspace
Repository
Work
Workflow
Stage
Execution
Backend
Profile
Model
NativeSession
Message
ToolCall
Artifact
File
Commit
Finding
Client
```

Representative edges:

```text
Work          → targets        → Repository
Work          → follows        → Workflow
Execution     → executes       → Work
Execution     → uses           → Backend
Execution     → bound_to       → NativeSession
Execution     → produced       → Artifact
ToolCall      → changed        → File
Execution     → produced       → Commit
Message       → caused         → ToolCall
Finding       → concerns       → Artifact
Stage         → preceded       → Stage
Execution     → superseded     → Execution
```

Every derived edge records its source event sequence.

The history therefore remains inspectable:

```text
what happened?
        → JSONL chronology

how are the things related?
        → graph

what patterns exist across executions?
        → DuckDB analytics
```

The graph may eventually earn a dedicated graph engine.

Depot does not require one initially.

---

# 24. Current State

The daemon maintains a reducer-derived in-memory state projection.

```text
journal
  ↓
reducers
  ↓
current state
```

Daemon restart:

```text
load optional snapshot
        ↓
replay journal after snapshot sequence
        ↓
reconstruct current state
        ↓
reconcile native backends
```

Snapshots are an optimization.

They are not canonical state.

---

# 25. Recovery Model

Recovery is one of Depot's primary responsibilities.

## Client failure

No consequence.

Clients do not own execution.

## Depot daemon failure

On restart:

```text
replay trajectory
        ↓
discover executions believed active
        ↓
ask matching adapter for native identity
        ↓
reattach / resume / classify
```

No new worker is created until prior ownership is reconciled.

## Native backend failure

Adapter determines whether the native session is:

```text
still alive
resumable
recoverable
irrecoverable
ambiguous
```

Ambiguity fails closed.

## Work versus Execution

This separation is absolute.

```text
native turn completed
≠
work completed

native process alive
≠
work active

native process dead
≠
work failed
```

The workflow or an explicit Depot operation changes Work state.

This directly eliminates one of Sergeant's most persistent categories of lifecycle bugs.

---

# 26. Input and Idempotency

Every mutation command has a command ID.

```json
{
  "command_id": "01K...",
  "operation": "work.respond"
}
```

If the same command arrives twice:

```text
same command_id
      ↓
same result
```

not:

```text
execute twice
```

This replaces much of the complexity previously represented by nonce files, action leases, and duplicate-delivery protection.

The event journal records acceptance and result.

---

# 27. Conversation and Tool Observation

Depot should ingest structured conversation and execution activity whenever a backend exposes it.

Normalized event families include:

```text
conversation.user
conversation.assistant.started
conversation.assistant.delta
conversation.assistant.completed

tool.requested
tool.started
tool.output
tool.completed
tool.failed

approval.requested
approval.resolved

filesystem.changed
git.changed

usage.updated
```

Raw vendor events remain retained separately.

This creates enough information for:

```text
live TUI
HTML command center
execution replay
analytics
trajectory graph
OTel
audit
future research
```

without coupling the core to one harness's event schema.

---

# 28. OpenTelemetry

Depot has first-class structured observability.

Rust instrumentation uses:

```text
tracing
tracing-subscriber
opentelemetry
opentelemetry-otlp
```

Internal execution events remain Depot events.

OTel is an export/projection.

Representative spans:

```text
work
 ├─ stage.prepare
 ├─ stage.implement
 │    └─ execution.turn
 │         ├─ tool.shell
 │         ├─ tool.edit
 │         └─ tool.test
 ├─ stage.review
 └─ stage.close
```

Representative metrics:

```text
depot_work_active
depot_execution_active
depot_execution_duration_seconds
depot_stage_duration_seconds
depot_wait_duration_seconds
depot_needs_input_total
depot_backend_failure_total
depot_backend_restart_total
depot_token_input_total
depot_token_output_total
depot_tool_duration_seconds
depot_journal_append_seconds
depot_event_ingest_lag
```

An OTLP endpoint may be configured later without modifying workflow code.

---

# 29. Embedded HTML Dashboard

The binary includes its own dashboard.

No Node runtime is required.

Technology:

```text
Axum
server-rendered HTML templates
embedded static assets
small vanilla JavaScript
EventSource / SSE
```

The dashboard is not a second backend.

It is an API projection.

Initial screens:

```text
Fleet
Work
Execution
Trajectory
Backends
Profiles
System
```

A work page can show:

```text
intent
workflow/stage
repository surface
execution backend/profile/model
native session
conversation
tool activity
Git changes
artifacts
state transitions
graph neighborhood
usage
timing
```

---

# 30. TUI

The native terminal UI uses:

```text
ratatui
crossterm
```

`depot` with no subcommand opens the TUI.

The TUI communicates exclusively through the local API.

It does not receive privileged access to daemon internals.

That is an important architectural test:

> If the TUI needs a private shortcut, the API is incomplete.

---

# 31. CLI

CLI parsing uses:

```text
clap
```

Representative commands:

```text
depot
depot status
depot work list
depot work show <id>
depot run "<intent>"
depot respond <id>
depot cancel <id>

depot backends
depot profiles
depot doctor

depot web
depot mcp

depot daemon
```

CLI output supports human text and machine JSON.

---

# 32. MCP

MCP is merely another Depot client.

Conceptually:

```text
MCP client
   ↓
depot mcp
   ↓
Depot local API
   ↓
daemon
```

Tools may expose:

```text
work_submit
work_get
work_list
work_respond
work_cancel
execution_get
backend_list
```

Resources may expose:

```text
depot://work/<id>
depot://execution/<id>
depot://trajectory/<id>
```

The MCP process contains no execution logic.

It does not touch storage.

It does not speak Claude/Codex/OpenCode protocols.

---

# 33. Repository Agent Integration

The repository-facing integration is intentionally tiny.

A project may contain:

```text
AGENTS.md
.depot/
  workflows/
  depot.toml        # only when needed

.agent/skill equivalent(s)
```

The exact harness instruction location can differ by client, but each skill teaches the same thing:

```text
Depot is the execution surface.

Use Depot to:
- submit durable work
- inspect work
- answer worker questions
- request independent execution
- cancel or resume work
- retrieve execution evidence

Do not manipulate Depot runtime files.
```

A harness-specific skill is principally a deterministic client wrapper.

It should not reimplement orchestration.

`depot init` can scaffold these files for new projects.

---

# 34. Rust Technology Choices

The initial implementation stack is:

| Function | Technology |
|---|---|
| Language | Rust stable |
| Async runtime | Tokio |
| HTTP/API | Axum + Tower |
| Serialization | Serde / serde_json |
| Configuration | TOML |
| CLI | Clap |
| TUI | Ratatui + Crossterm |
| HTTP client | Reqwest |
| Process management | tokio::process |
| Event journal | NDJSON / JSONL |
| Analytics/read model | DuckDB embedded Rust client |
| Large-object identity | BLAKE3 |
| IDs | ULID |
| Structured internal telemetry | tracing |
| External telemetry | OpenTelemetry / OTLP |
| Git operations | installed Git CLI |
| HTML | Embedded server-rendered assets + minimal browser JS |

This is one binary.

No Python runtime.

No Node runtime.

No external database server.

No Redis.

No tmux.

No shell watchdog swarm.

If later requirements invalidate one of those choices, the contracts remain.

---

# 35. Suggested Source Layout

Start with one crate.

Do not prematurely create a workspace of twelve architectural crates.

```text
depot/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── daemon.rs
│   ├── api.rs
│   ├── tui.rs
│   ├── web.rs
│   │
│   ├── domain/
│   │   ├── work.rs
│   │   ├── execution.rs
│   │   ├── workspace.rs
│   │   ├── workflow.rs
│   │   └── event.rs
│   │
│   ├── runtime/
│   │   ├── journal.rs
│   │   ├── projection.rs
│   │   ├── router.rs
│   │   ├── recovery.rs
│   │   └── surface.rs
│   │
│   ├── backend/
│   │   ├── mod.rs
│   │   ├── claude.rs
│   │   ├── codex.rs
│   │   ├── opencode.rs
│   │   └── prime.rs
│   │
│   └── telemetry.rs
│
├── workflows/
├── skills/
├── web/
└── tests/
```

Extract crates when a demonstrated boundary needs independent compilation or reuse.

Not before.

---

# 36. Mapping Sergeant's Lessons Into Depot

The rewrite should continually ask:

> Why did Sergeant need this mechanism?

Then preserve the invariant, not the implementation.

| Sergeant mechanism | Underlying requirement | Depot expression |
|---|---|---|
| tmux pane identity | Know exactly which execution we own | Native session/thread ID + execution generation |
| `tmux send-keys` | Deliver new work | Native backend transport |
| fleet status files | Durable current work state | Event reducers/projection |
| progress watcher | Determine meaningful activity | Native events + observed activity |
| notification nonce | Avoid duplicate delivery | Command IDs/idempotency |
| action lease | Fence ownership after interruption | Daemon execution ownership generation |
| cleanup process-group sweep | Stop only owned execution | Backend stop contract + native identity |
| `sgt-watch` | Query execution state | Depot API/projections |
| callbacks | Publish state changes | Event subscribers |
| generated brief procedure | Encode workflow | Versioned workflow stages |
| tmux battery drain | Durable execution outlives coordinator incorrectly | Native background runtime + explicit retirement |

This table should become part of Depot's design tests.

---

# 37. Testing Strategy

Depot needs three layers of testing.

## Deterministic core tests

Use a fake backend.

Exercise:

```text
state transitions
idempotency
journal recovery
workflow progression
routing
profile selection
recovery
multi-repo surfaces
graph projection
DuckDB rebuild
API behavior
```

No model tokens required.

## Backend contract tests

Opt-in tests run against installed authenticated harnesses.

Every backend must prove:

```text
probe
start
identity
send
observe
history
interrupt if supported
resume
stop
restart/reconcile
```

These tests define the minimum supported backend versions.

Not documentation guesses.

## Regression tests from Sergeant

Recreate known classes of failure:

```text
worker reports completion but native session remains alive
daemon dies during delivery
native process dies after work is preserved
same command delivered twice
old execution identity reused
terminal work still has live process
client disconnects mid-run
backend runtime restarts
work waits for input for hours
partial journal tail after crash
```

Depot should make these cases boring.

---

# 38. Initial Vertical Slice

The first build should prove architecture, not feature count.

P0 is complete when all of the following work together:

```text
one Rust binary

daemon auto-start

single-repo discovery

optional multi-repo workspace

JSONL trajectory journal

in-memory state projection

DuckDB analytical projection

HTTP/JSON API

SSE event stream

CLI client

basic TUI

basic embedded HTML view

origin-aware routing

named backend profiles

native Claude execution

native Codex execution

one simple staged workflow

Git worktree execution surface

restart and native-session reconciliation

OTel traces/metrics export

raw + normalized event capture
```

OpenCode and Prime adapters follow the same backend contract immediately after the core contract proves itself; neither should require core redesign.

MCP is added only as a thin API client once the API surface is stable enough to expose.

---

# 39. The First User Experience

A repository has been prepared for Depot.

A developer clones it.

```bash
git clone git@github.com:company/service.git
cd service
claude
```

They say:

> Add retry handling to the payment settlement worker. Have another agent independently review it and fix anything important.

The repository's Claude instructions invoke Depot.

Depot receives:

```text
origin    = claude
workspace = current repo
intent    = ...
workflow  = software-change
```

No backend was explicitly supplied.

Routing chooses:

```text
backend = claude
profile = matching/default Claude profile
```

Depot:

```text
creates Work
        ↓
materializes Git worktree
        ↓
binds workflow
        ↓
launches native Claude background execution
        ↓
captures session identity
        ↓
streams/records trajectory
        ↓
advances workflow
        ↓
dispatches independent review as another execution
        ↓
captures remediation
        ↓
records final evidence
        ↓
retires native sessions
```

The original foreground Claude remains only a client/coordinator.

It may exit at any point.

Depot continues.

The developer can later run:

```bash
depot
```

and see the exact execution.

Or open the Depot HTML dashboard.

Or ask another harness through MCP.

All surfaces see the same state because all surfaces are clients of the same daemon.

---

# 40. Design Principles

Depot should keep these as architectural invariants.

**One owner.**  
Only the daemon mutates Depot execution state.

**Native first.**  
Use native agent processes, sessions, servers, event streams, history, and authentication instead of recreating them.

**Work state is not process state.**  
Never infer one from the other without an explicit contract.

**Trajectory is durable.**  
State changes and meaningful execution evidence survive client and daemon restarts.

**Raw evidence is valuable.**  
Preserve native events so future versions can learn things the current version does not understand.

**Projections are disposable.**  
DuckDB, graph views, dashboard state, and OTel can all be rebuilt or regenerated.

**Clients are equal.**  
CLI, TUI, HTTP, MCP, and agent skills consume the same execution surface.

**Authentication stays native.**  
Depot routes profiles; it does not become a credential vault.

**Procedure is data.**  
Workflow ceremony belongs in versioned workflows rather than Rust control flow.

**The middle stays deterministic.**  
Agent intelligence belongs in the harness. Depot routes, records, supervises, and reconciles.

---

# 41. The Product in One Sentence

> **Depot is a Rust-native, local-first execution runtime that turns intent into durable, observable agent work across one or more repositories by routing work through native agent harnesses and preserving the complete execution trajectory behind one client-neutral daemon surface.**

Or more casually:

> **Depot is the yard where the work comes in, gets routed to the right truck, and everything that happens before it leaves is accounted for.**

That is the system we should build.
