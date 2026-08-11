# Sergeant Multi-Harness Adapter Suitability and Contract Assessment — Version 2

**Date:** August 11, 2026  
**Status:** Revised research report; supersedes the first deep-research report  
**Repositories:** `miztertea/sergeant-rs` and `kunchenguid/no-mistakes`  
**Priority harnesses:** Codex, OpenCode, Goose  
**Revision focus:** Re-evaluate Claude Code as a family of runtime strategies rather than only `claude -p`

---

## Executive assessment

The first report got the broad architectural conclusion right but evaluated Claude too narrowly.

It correctly concluded that Sergeant should be generalized around durable runtime ownership, sessions, turns, interactions, events, and typed capabilities rather than around “a subprocess that returns a result.” It also correctly identified OpenCode’s server and Codex App Server as richer control planes than a simple print-mode invocation.

The incompleteness was treating this as the full Claude shape:

```text
Claude Code
  └── claude -p
       └── one process per turn
            └── stream-json
                 └── resume by session id
```

As of August 2026, Claude exposes several materially different execution surfaces:

```text
Claude provider family
├── direct CLI turn
│    └── claude -p / --print
├── deterministic direct CLI turn
│    └── claude --bare -p
├── local supervised session
│    └── claude --bg + per-user supervisor + agent view
├── embedded agent runtime
│    └── Python or TypeScript Agent SDK
├── native orchestration overlays
│    ├── subagents
│    ├── agent teams
│    ├── direct teammate messaging
│    ├── shared task graph
│    ├── dynamic/background workflows
│    └── goals
├── Anthropic-hosted agent runtime
│    └── Managed Agents
└── Anthropic-hosted automation
     └── Claude Code routines
```

These are not equivalent transports, and several are much closer to Codex App Server, OpenCode Server, or Goose ACP than to a one-shot CLI. Claude Agent View now runs detached, durable local conversations under a per-user supervisor, exposes shell management commands and JSON session inventory, preserves conversations across process replacement, allows replies and attachment, carries subagents and workflows into the background, and uses the same terminal credentials as interactive Claude Code. That is a daemon-like runtime and should be assessed as one. [C2]

The Agent SDK offers the richest typed Claude integration: native message objects, sessions, resume/fork, structured interaction handling, hooks, subagent lifecycle events, permissions, skills, MCP, usage, and TypeScript-only lifecycle/team/worktree hooks. But Sergeant is Rust, so adopting it means a local Python or TypeScript sidecar rather than a direct library dependency. It also changes the authentication and product boundary enough that it cannot be assumed to preserve Sergeant’s present “use the harness the human already authenticated in the terminal” contract. [C4][C5][C6][C7]

The correction is therefore:

> **The unit of comparison is not a harness. It is a harness plus a runtime strategy.**

The revised top-level recommendation is:

1. Keep the current `claude -p` adapter as the stable, directly measured Claude transport and compatibility fallback.
2. Treat **Claude Agent View / supervisor mode as the strategic local terminal-auth Claude transport** and run it through the same admission suite applied to Codex App Server and OpenCode Server.
3. Treat the **Agent SDK as the strategic typed-control transport** when a Python/TypeScript sidecar and its authentication model are acceptable.
4. Model `--bare` as a **configuration and authentication profile for `-p`**, not as a replacement transport.
5. Model subagents, agent teams, messaging, workflows, and goals as **orchestration capabilities layered on top of a runtime**, not as separate backends.
6. Treat Managed Agents and routines as separate hosted backend families. They are not upgrades to Sergeant’s local process adapter.
7. Extend Sergeant’s adapter seam now so all runtime strategies can fit without engine-specific branches.

The original report’s cross-harness conclusion still holds:

> Sergeant does not need to replace its engine. It needs to make `RuntimeScope` executable, separate session and turn identity, add an event-driven settlement path, and represent orchestration capabilities with more detail than a single `native_subagents` boolean.

---

## What changed from the first report

The first report scored Claude Code at **83/100** using only `claude -p ... --output-format stream-json`. That score remains reasonable for the direct process-per-turn transport, but it is not a score for “Claude support.”

This version changes five things:

1. **Runtime strategy becomes part of adapter identity.**
2. **Claude receives a full provider-family assessment.**
3. **Native orchestration becomes a first-class scoring dimension.**
4. **Authentication and configuration discovery are separated from transport.**
5. **Push/event settlement becomes a required engine primitive**, because persistent runtimes and Sergeant’s own Cerberus findings show that “observe only immediately after a command” is insufficient.

The revised report does not discard the original Codex, OpenCode, Goose, ACP, Rovo, Pi, or Copilot analysis. It preserves those conclusions and places them beside the missing Claude runtime choices.

---

## Research scope and evidence standard

The assessment draws from:

- the current Sergeant backend contract and Claude implementation;
- Sergeant issues, commits, and open PRs, including the Cerberus close-out work;
- the current `no-mistakes` adapters and process-supervision infrastructure;
- official current documentation for Claude Code, the Claude Agent SDK, Managed Agents, Codex, OpenCode, and Goose;
- community implementation evidence where official documentation is incomplete;
- live facts recorded in Sergeant’s own contract tests and gauntlet evidence.

The evidence hierarchy is:

```text
live Sergeant measurement
    > current official protocol documentation
        > upstream implementation and tests
            > independent community implementation
                > discussion or anecdote
```

A capability may be:

```text
documented
implemented by the adapter
measured against the installed harness
admitted for a specific profile/version
```

Only the last state should cause Sergeant to advertise it as available.

---

## Revised scoring method

The numbers are suitability scores for Sergeant’s adapter model, not judgments about model quality.

| Criterion | Weight |
|---|---:|
| Explicit semantic lifecycle and terminal outcome | 12 |
| Durable session identity, resume, and recovery | 12 |
| Structured streaming events | 8 |
| Native runtime control and inspection | 10 |
| Interrupt and cancellation semantics | 8 |
| Questions and approval round-trips | 8 |
| Complete history and reconciliation | 8 |
| Local process fit and intended authentication model | 10 |
| Model, profile, permission, and configuration control | 6 |
| Usage and model evidence | 4 |
| Native orchestration visibility and control | 8 |
| Native schema-constrained output | 3 |
| Protocol stability and feature/version negotiation | 3 |
| **Total** | **100** |

A separate confidence label expresses how much of the score is supported by current official protocol and live measurement.

This matters because a runtime can have excellent theoretical primitives while remaining a poor production target today. Claude Agent View is the clearest example: its lifecycle is attractive, but its public machine interface is currently less complete and less stable than OpenCode’s HTTP API or Codex App Server’s documented protocol.

---

## Updated cross-harness summary

### Local and user-operated runtimes

| Harness and runtime strategy | Fit | Confidence | Current recommendation |
|---|---:|---|---|
| **OpenCode Server** — `opencode serve` + HTTP/SSE | **96** | High | First-class strategic adapter |
| **Codex App Server** — local stdio protocol | **95** | Medium-high | First-class strategic adapter |
| **Claude Agent SDK sidecar** — Python/TypeScript host | **92** capability fit | Medium | Strategic typed transport; conditional on sidecar/auth decision |
| **Claude Agent View supervisor** — `--bg` + supervisor | **86** provisional | Medium-low | Strategic terminal-auth spike; experimental admission only |
| **Goose ACP** | **83** provisional | Medium | First-class candidate after live protocol measurement |
| **Claude direct** — `claude -p` + stream JSON | **83** | High | Keep as stable baseline and fallback |
| **Codex exec** — `codex exec --json` | **79** | High | Easy first Codex adapter and compatibility mode |
| **Goose run** — `goose run --output-format stream-json` | **72** | Medium | Useful reduced-capability adapter |
| **Cursor / generic ACP** | **65–80** | Medium | Shared transport with target-specific admission |
| **Rovo Dev server** | **~63** | Medium | Reduced-capability persistent-server adapter |
| **Pi JSONL** | **~60** | Medium-low | Reduced-capability process adapter |
| **GitHub Copilot CLI** | **~55** | Medium | Conditional adapter; fragile typed-output semantics |

### Hosted Claude runtimes

| Runtime | Native capability fit | Fit to Sergeant’s present local scope | Recommendation |
|---|---:|---:|---|
| **Claude Managed Agents** | Very high | Low-medium | Separate hosted backend family, not a replacement Claude CLI adapter |
| **Claude Code routines** | High for autonomous triggers | Low as a general interactive backend | Treat as an external trigger/executor integration |
| **Claude Code on the web/cloud sessions** | High for remote work | Different work-surface and custody model | Separate backend/profile family |

The hosted scores should not be compared directly with local runtimes. They change who owns the environment, credentials, filesystem, lifecycle, and audit boundary.

---

# Part I — Sergeant’s current architecture

## What Sergeant already got right

Sergeant’s backend contract is already richer than the `no-mistakes` agent abstraction. It has:

- `RuntimeScope`;
- explicit capabilities;
- `prepare`, `launch`, `send`, `observe`, `interrupt`, `resume`, `history`, and `stop`;
- separate native evidence and workflow-semantic signals;
- raw vendor evidence plus normalized events;
- profiles and model selection;
- strict unsupported behavior rather than silent emulation;
- two-phase execution reservation before external work;
- deferred cleanup outside the core lock.

That is the correct foundation.

The current contract’s most important rule is:

> A native process exiting is evidence about a process, not proof that a Sergeant stage completed.

That rule is exactly what persistent servers and multi-agent runtimes require.

## What the Claude baseline accidentally hid

The current Claude implementation compresses several identities into one handle:

```text
Sergeant execution
    ≈ Claude conversation
        ≈ current turn process
```

A first `claude -p` process starts a conversation; later processes resume it. There is no separately owned long-running Claude runtime, so `RuntimeScope::PerExecution` was enough.

OpenCode, Codex App Server, Goose ACP, Claude Agent View, and the Agent SDK expose a larger hierarchy:

```text
runtime
  └── session / conversation / thread
       └── turn / request
            ├── agent worker / teammate / subagent
            ├── task
            ├── tool
            └── interaction
```

Sergeant should preserve this hierarchy instead of forcing every native identifier into `ExecutionHandle.native_id`.

## Cerberus already proved two missing invariants

The open Cerberus work recorded two important findings:

1. Claude Code 2.1.227 did not emit the `post_turn_summary` record on that host, so the adapter withdrew the actor-ask capability it had measured on 2.1.226.
2. A turn could end without the engine being re-driven to observe and settle it, leaving work active even though the native turn was over.

Those are not merely Claude parser bugs. They prove two generic requirements:

```text
protocol-derived semantic features must be runtime-admitted,
not assumed from a version or stale fixture

and

every launched turn must cause a later settlement attempt,
even when no client command follows it
```

The second requirement becomes even more important for persistent runtimes whose events arrive asynchronously.

---

# Part II — What `no-mistakes` really provides

## The valuable reuse is protocol knowledge, not architecture

`no-mistakes` has a narrow common operation resembling:

```text
Run(prompt, cwd, env, schema, session?) → Result
```

That is appropriate for bounded validation pipeline steps. Sergeant owns durable work, recovery, user interaction, lifecycle reconciliation, and long-lived execution identity, so it must keep the richer contract.

`no-mistakes` remains highly valuable for:

- CLI argument construction;
- JSONL and SSE parser behavior;
- session ID extraction and resume grammar;
- structured-output handling;
- usage extraction;
- transient error classification;
- stderr and malformed-stream handling;
- managed-server startup and health checks;
- process-group termination and pipe drainage;
- adapter-specific edge cases.

## Updated Claude leverage estimate

The first report said roughly 90% of Claude support was already embodied in Sergeant. That is true only for **direct `claude -p` support**.

| Claude runtime strategy | Estimated `no-mistakes` protocol-discovery leverage |
|---|---:|
| Direct `claude -p` | **70–85%** |
| `claude --bare -p` | **60–75%**, mostly the same transport plus new auth/config rules |
| Agent View supervisor | **5–15%** |
| Agent SDK sidecar | **5–15%** |
| Agent teams and teammate messaging | **0–10%** |
| Managed Agents | **Near zero** |
| Routines | **Near zero** |

The reusable `no-mistakes` process-supervision pattern is valuable across all local child-process adapters, but it does not provide the supervisor, team, hook, or SDK protocol integration.

## The remembered upstream bug

The exact `no-mistakes` bug the owner remembers finding still cannot be identified honestly from the inspected evidence. There are several defensive fixes and quirks in its current code, but none can be tied confidently to that specific test incident. This report therefore does not invent an upstream PR.

---

# Part III — Claude provider-family assessment

## 1. Direct `claude -p`: the stable baseline

### Native shape

```text
runtime = one Claude process for the current turn
session = durable Claude session ID
turn = that process invocation
workers = optional subagents within the turn
```

Current Claude documentation describes `-p` as the CLI surface of the Agent SDK. It supports noninteractive execution, stdin, JSON or stream-JSON output, session resume, structured output, model and tool policy, MCP, skills, custom agents, background subagents/workflows, retry events, and session initialization metadata. Newer versions can forward nested subagent text and expose the parent tool-use ID needed to rebuild the subagent tree. [C1]

### Strengths

- Reuses the installed Claude binary.
- Uses the existing terminal-authenticated Claude Code product.
- Simple local process boundary.
- Durable session ID and resume.
- Structured stream and result envelope.
- Native usage/model metadata.
- Supports subagents without Sergeant implementing the model loop.
- Mature enough to retain as the production baseline.

### Limitations

- Runtime control is process-oriented.
- Full native history remains tied to local transcript storage.
- Native interaction and completion records have changed across versions.
- A process ending without the expected envelope creates ambiguity.
- Long-running child work and subagents complicate process cleanup.
- One turn is one subprocess, so interruption is weaker than a native turn-interrupt protocol.
- Agent teams are not represented simply by parsing the ordinary top-level result.
- A successfully returned result is not, by itself, proof of a completed Sergeant stage.

### Revised recommendation

Keep it and rename it conceptually:

```text
backend = claude
transport = direct
```

Do not let it remain the definition of “Claude.”

### Fit

**83/100, high confidence.**

---

## 2. `--bare`: not the next transport

Anthropic says `--bare` is recommended for scripted and SDK calls and will become the default for `-p` in a future release. But the meaning is frequently misunderstood.

`--bare` does not replace `-p`. It changes startup and configuration discovery:

- skips hooks;
- skips skills;
- skips plugins;
- skips MCP auto-discovery;
- skips automatic memory and `CLAUDE.md`;
- loads only what is supplied explicitly;
- does not read OAuth credentials or the system keychain;
- requires an Anthropic API key, `apiKeyHelper`, or provider-specific credentials. [C1]

Therefore:

```text
claude -p
  = transport

--bare
  = deterministic configuration + authentication profile
```

For Sergeant, this creates at least three Claude direct profiles:

```text
terminal-context
  existing Claude Code login
  user/project configuration as explicitly selected

controlled-terminal
  existing Claude Code login
  restricted setting sources
  Sergeant injects the execution kit explicitly

bare-api
  API/provider credentials
  no ambient Claude configuration
  all tools, agents, MCP, plugins, and prompts supplied explicitly
```

Sergeant must not silently move a terminal-auth profile to `--bare` when Anthropic changes the default. Probe and profile materialization need to make the effective mode visible.

### Recommendation

Model bare mode in profile/auth/configuration capabilities. Do not create a separate backend and do not describe it as a successor to `-p`.

---

## 3. Claude Agent View and the local supervisor

### Why this changes the report

Agent View is no longer just a UI feature. It exposes a local runtime model:

```text
per-user Claude supervisor
  └── detached Claude session processes
       ├── durable local conversation
       ├── session state
       ├── reply / attach
       ├── stop / respawn / remove
       ├── optional worktree
       └── subagents, workflows, monitors, and background commands
```

Claude documents:

- `claude --bg "<prompt>"` to launch a detached session;
- `claude agents --json` to enumerate sessions;
- `claude attach`, `logs`, `stop`, `respawn`, and `rm`;
- `claude daemon status` and daemon stop operations;
- a per-user supervisor;
- state under `CLAUDE_CONFIG_DIR`;
- supervisor and worker process replacement while conversations persist;
- the same credentials as interactive Claude Code;
- configuration, MCP, plugins, model, effort, and permissions carried into dispatched sessions;
- worktree isolation for editing sessions;
- input-needed, working, completed, and other session states. [C2]

This is semantically much closer to:

```text
OpenCode Server
Codex App Server
Goose ACP
```

than to direct `-p`.

### What it gives Sergeant

Potentially:

- managed runtime scope (`PerProfile` or `PerWorkspace`);
- durable sessions independent of Sergeant’s process;
- terminal-auth reuse;
- session inventory after Sergeant restart;
- attach and human takeover;
- native stop and respawn operations;
- user-visible “needs input” state;
- native worktree lifecycle;
- long-running subagents and workflows that survive detach/reattach;
- supervisor self-restart and worker reconnection;
- direct mapping to `human_attach=true`;
- stronger recovery than `/proc` scanning alone.

### What it does not yet give Sergeant publicly

The documented automation surface is largely shell commands, JSON inventory, logs, and local state. The public docs mention a supervisor socket, but do not define that socket as a supported protocol for third-party clients.

The missing public contract includes:

- a stable push event stream;
- a stable session-create API returning all identifiers before work;
- documented typed reply/approval/question RPCs;
- complete machine-readable transcript/history retrieval;
- turn-level IDs and native turn interruption;
- machine-readable usage and nested-worker events comparable to the direct stream;
- protocol version/schema negotiation.

A Sergeant implementation that reverse-engineers `roster.json`, `state.json`, or the private socket would be depending on private storage/protocol. That may be useful for a spike, but it must not be misrepresented as a stable public adapter.

### Operational caveats

Agent View is a research preview. Sessions are local, rate limits multiply with parallelism, and worktree/session cleanup has documented edge cases. The feature has been changing quickly across Claude Code releases. [C2]

### Recommended Sergeant strategy

```text
backend = claude
transport = supervisor
stability = experimental
runtime_scope = per_profile or per_workspace
```

Admission should proceed in two phases:

**Phase A: public CLI surface only**

- launch with `--bg`;
- enumerate with `agents --json`;
- inspect `logs`;
- stop and respawn through documented commands;
- validate state transitions;
- use transcript/session APIs only where documented.

**Phase B: optional private-protocol spike**

- inspect whether the supervisor socket can support a reliable typed client;
- do not ship it as stable without an upstream protocol commitment;
- feature-gate by exact Claude version and schema fingerprint.

### Fit

**86/100 provisional, medium-low confidence.**

The score is lower than Codex App Server or OpenCode despite the attractive lifecycle because the machine control protocol is less public and less complete.

---

## 4. Claude Agent SDK sidecar

### Native shape

```text
runtime = Sergeant-owned Python or TypeScript sidecar
session = Agent SDK session
turn = query / response cycle
workers = SDK subagents and tasks
events = typed SDK message stream + hooks
```

The Agent SDK is the strongest documented Claude control surface. It offers:

- native message objects and response streams;
- built-in tools;
- permissions and programmatic approval handling;
- sessions, continue, resume, and fork;
- session enumeration and message retrieval;
- MCP;
- skills, commands, memory, plugins, and project configuration;
- structured output;
- subagents;
- usage and OpenTelemetry support;
- hooks for tools, permissions, stop/failure, subagent start/stop, session lifecycle, tasks, teammate idle, config changes, instructions, worktrees, directory changes, and files. [C4][C5][C6]

### Why it is attractive for Sergeant

It can replace fragile event-shape inference with typed lifecycle callbacks.

A TypeScript sidecar could translate:

```text
SessionStart       → runtime/session events
SubagentStart      → worker.started
SubagentStop       → worker.completed
PermissionRequest  → interaction.approval
Stop               → turn.completed
StopFailure        → turn.failed
TeammateIdle       → worker.idle
TaskCreated        → task.created
TaskCompleted      → task.completed
WorktreeCreate     → surface.native.created
WorktreeRemove     → surface.native.removed
```

The SDK also provides complete session enumeration and message reads, making recovery and history materially better than scraping a private transcript format. [C5]

### Why it is not an immediate drop-in

1. **Language boundary:** The official SDK is Python and TypeScript only. Anthropic explicitly tells other languages to use the CLI subprocess surface. Sergeant would need a maintained sidecar protocol. [C4]
2. **Authentication boundary:** The sidecar must use an authentication mode that is valid for the SDK deployment. It cannot simply assume it may impersonate the user’s interactive Claude Code login. Current Claude documentation distinguishes terminal/product authentication from bare/API-style scripted authentication. [C1][C7]
3. **Product boundary:** Sergeant would be embedding Claude’s runtime as a library host rather than launching the user’s terminal harness unchanged.
4. **Version coupling:** The sidecar, SDK package, bundled CLI behavior, and Sergeant adapter schema must be pinned together.
5. **Teams are not simply an SDK option:** Anthropic’s SDK feature guide says agent teams are a CLI feature and are not directly configured through normal SDK options, even though TypeScript hooks expose teammate/task events. [C8]

### Recommendation

Treat this as:

```text
backend = claude
transport = agent-sdk
host = typescript-sidecar
stability = candidate
auth = explicit
```

TypeScript is the stronger host because its hook coverage includes session lifecycle, teammate/task, worktree, and configuration events that Python lacks.

### Fit

**92/100 capability fit, medium confidence.**

The capability fit is excellent. The practical Sergeant decision remains conditional because of sidecar ownership, authentication, and distribution.

---

## 5. Subagents, agent teams, workflows, and goals are capabilities

The first report used one boolean:

```text
native_subagents: true | false
```

That is no longer sufficient.

### Claude subagents

Direct `-p` can expose nested subagent messages through `parent_tool_use_id`; newer versions can forward subagent text and nested depths. The Agent SDK adds explicit subagent start/stop hooks and agent IDs. [C1][C6]

This is a delegated-worker model:

```text
main session
  └── subagent
       └── returns result to parent
```

### Claude agent teams

Agent teams are different:

```text
lead session
  ├── teammate A
  ├── teammate B
  └── shared task graph

A ↔ B direct messaging
A/B ↔ lead
```

Claude documents:

- separate context windows;
- a fixed lead;
- direct teammate messaging;
- shared tasks with dependencies;
- self-claiming;
- autonomous lead plan approval;
- human access to individual teammates;
- team hooks;
- independent teammate sessions. [C3]

It also documents serious current limits:

- experimental and disabled by default;
- in-process teammates are not restored by session resume;
- task status can lag;
- shutdown can be slow;
- one team per session;
- no nested teams;
- fixed lead;
- permissions are inherited at spawn;
- some background-agent combinations are unsupported. [C3]

Therefore, agent teams should begin as an **opaque or partially observable orchestration capability**, not as durable Sergeant child executions.

### Dynamic/background workflows

Claude Agent View explicitly carries running subagents, workflows, background shell commands, and scheduled loops into the background process and allows new ones after detachment. [C2]

For Sergeant, that means a Claude session may be internally active even when the lead turn appears idle. The adapter must distinguish:

```text
lead waiting
team still running
background workflow running
all native activity terminal
```

A single top-level `result` event is no longer enough to describe the native execution tree.

### Goals

Claude’s `/goal` can keep a session working across turns until a completion condition is satisfied, and Anthropic documents it as working in interactive mode, `-p`, and Remote Control. [C9]

This is a useful native capability, but it should not replace Sergeant’s workflow semantics. It is better modeled as:

```text
native_goal_loop = available
```

Sergeant can choose to use it inside one stage when the stage has a verifiable completion condition, while retaining ownership of work state, evidence, retries, review, and escalation.

### Required orchestration capability model

Replace `native_subagents: bool` with something like:

```rust
pub enum WorkerTopology {
    None,
    DelegatedTree,
    PeerTeam,
    Dynamic,
}

pub struct OrchestrationCapabilities {
    pub topology: WorkerTopology,
    pub worker_identity: EvidenceLevel,
    pub worker_events: EvidenceLevel,
    pub worker_transcript: HistoryCapability,
    pub direct_worker_message: bool,
    pub shared_task_graph: bool,
    pub task_dependencies: bool,
    pub per_worker_interrupt: bool,
    pub per_worker_permissions: bool,
    pub resume_workers: bool,
    pub nested_workers: bool,
    pub native_goal_loop: bool,
    pub native_worktree_isolation: bool,
}
```

Claude direct mode, Agent View, Agent SDK, teams, Codex subagents, OpenCode child sessions, Goose subagents, and ACP targets can then be described without pretending they are identical.

---

## 6. Managed Agents and routines

### Managed Agents

Anthropic Managed Agents provides hosted sessions, environments, agent versions, event streams, multiple isolated session threads, shared sandbox/filesystem/vault credentials, thread interruption, and multi-agent coordination. [C10]

Architecturally, it is extremely close to the runtime/session/turn/worker contract proposed for Sergeant.

Operationally, it is a different product:

```text
Sergeant local model
  user machine / homelab
  local authenticated harness
  local worktree
  Sergeant owns lifecycle and evidence

Managed Agents
  Anthropic-hosted environment
  API-managed agent definitions
  hosted sessions and threads
  Anthropic owns runtime/sandbox lifecycle
```

It should be a separate external backend, not a new implementation of `ClaudeBackend`.

### Routines

Routines are saved Claude Code cloud configurations triggered by schedule, API, or GitHub events. They run as autonomous cloud sessions with selected repositories, environments, and connectors. [C11]

For Sergeant they are better treated as:

- an external trigger source;
- an external executor;
- or a peer orchestration system.

They are not a suitable replacement for Sergeant’s local stage adapter because each routine run owns its own cloud checkout and autonomy boundary.

---

# Part IV — Priority adapter assessment, retained and corrected

## Codex

Sergeant should retain two Codex transports:

```text
codex-exec
  codex exec --json
  process per turn
  durable thread resume
  strong easy fallback

codex-app-server
  persistent local process
  explicit thread and turn identity
  approvals, questions, history, interrupt
  strategic control plane
```

`no-mistakes` provides high leverage for `codex exec`: argument grammar, JSONL event parsing, thread extraction, resume, schema handling, result selection, and usage accounting.

Codex App Server remains the strategic integration because it exposes explicit thread creation, turn start, turn interruption, history, runtime state, approvals, user input, and subagent threads. Its main risk is protocol maturity, not architectural fit.

**Recommendation unchanged:** App Server strategic; `exec` compatibility/fallback.

---

## OpenCode

OpenCode remains the strongest structural match:

```text
runtime = opencode serve
session = server session
turn = prompt/message execution
events = SSE
control = HTTP
```

It provides explicit session creation, status, messages/history, abort, permissions, child sessions, model selection, structured output, and an event bus. `no-mistakes` already supplies much of the hard protocol-discovery work: server startup, health, HTTP calls, SSE filtering, message-part reconstruction, tool events, session idle, and usage.

**Recommendation unchanged:** first-class high-priority adapter.

---

## Goose

Goose still has two plausible transports:

```text
goose-run
  process per invocation
  stream-json
  reduced capability

goose-acp
  persistent protocol process
  sessions and interaction
  strategic candidate
```

`no-mistakes` does not have a first-class Goose adapter. Its generic ACP integration provides useful protocol-level leverage but not a complete Goose implementation.

**Recommendation unchanged:** spike ACP first while preserving `goose run` as the fallback.

---

# Part V — Secondary adapters

## Cursor and generic ACP

A shared ACP transport remains the multiplicative investment. Capabilities must be admitted per target; ACP membership does not guarantee identical history, interaction, usage, or child-agent semantics.

## Rovo Dev

Rovo remains useful as a second persistent-server/SSE proof. It helps test whether Sergeant’s runtime layer is truly generic beyond OpenCode. Its demonstrated session recovery and typed-output guarantees remain weaker.

## Pi

Pi remains a straightforward JSONL process adapter. Session behavior should be measured independently because the `no-mistakes` implementation intentionally disables sessions.

## GitHub Copilot CLI

Copilot remains implementable but weaker for typed outcomes. Prompt-enforced JSON that requires searching backward through assistant messages is not equivalent to native schema output.

---

# Part VI — Proposed Sergeant adapter contract, version 2

## 1. Provider and runtime strategy are separate

```rust
pub struct BackendId {
    pub provider: String,   // "claude", "codex", "opencode", "goose"
    pub transport: String,  // "direct", "supervisor", "agent-sdk", "app-server", "acp"
}
```

Examples:

```text
claude/direct
claude/supervisor
claude/agent-sdk
codex/exec
codex/app-server
opencode/server
goose/run
goose/acp
acp/cursor
```

Profiles select a transport explicitly. The engine does not infer it from the provider name.

---

## 2. Make runtime ownership executable

The existing `RuntimeScope` should drive a managed-runtime layer:

```rust
pub enum RuntimeScope {
    External,
    PerProfile,
    PerWorkspace,
    PerExecution,
}

pub enum RuntimeTransport {
    ChildProcess,
    SupervisorCli,
    StdioRpc,
    HttpSse,
    Acp,
    Sidecar,
    HostedApi,
}

pub struct RuntimeRequest {
    pub profile_id: String,
    pub workspace_id: Option<String>,
    pub cwd: PathBuf,
}

pub struct RuntimeLease {
    pub runtime_id: String,
    pub generation: u64,
    pub transport: RuntimeTransport,
    pub pid: Option<u32>,
    pub endpoint: Option<String>,
    pub native_version: String,
}
```

The backend gains:

```rust
fn ensure_runtime(
    &self,
    request: &RuntimeRequest,
) -> Result<RuntimeLease, BackendError>;

fn inspect_runtime(
    &self,
    lease: &RuntimeLease,
) -> Result<RuntimeObservation, BackendError>;

fn release_runtime(
    &self,
    lease: &RuntimeLease,
) -> Result<Completion, BackendError>;
```

Mapping:

| Transport | Runtime lease |
|---|---|
| Claude direct | current turn process |
| Claude supervisor | per-profile Claude supervisor |
| Claude Agent SDK | Sergeant-owned sidecar |
| Codex exec | current turn process |
| Codex App Server | persistent app-server |
| OpenCode | persistent server |
| Goose run | current process |
| Goose ACP | persistent ACP process |
| Rovo | persistent server |

---

## 3. Separate execution, session, turn, and worker identity

```rust
pub struct PreparedExecution {
    pub execution_id: String,
    pub request: StartRequest,
    pub reserved_native_session_id: Option<String>,
}

pub struct SessionHandle {
    pub execution_id: String,
    pub runtime_id: Option<String>,
    pub runtime_generation: Option<u64>,
    pub native_session_id: String,
}

pub struct TurnHandle {
    pub execution_id: String,
    pub sergeant_turn_id: String,
    pub native_session_id: String,
    pub native_turn_id: Option<String>,
    pub pid: Option<u32>,
}

pub struct WorkerHandle {
    pub execution_id: String,
    pub sergeant_turn_id: String,
    pub native_worker_id: String,
    pub parent_worker_id: Option<String>,
    pub topology: WorkerTopology,
}
```

The durable sequence becomes:

```text
execution.reserved
runtime.ensured
native.session.created
native.identity.bound
turn.reserved
turn.started
worker/task/tool/interaction events
exactly one turn terminal
```

Adapters that cannot allocate native identity before work are represented honestly rather than forced to mimic Claude’s caller-selected session ID.

---

## 4. Add a push/event settlement path

The current event sink journals normalized evidence, but the engine also needs to know that new evidence may change work state.

A backend runtime should be able to return a wakeup subscription:

```rust
pub trait BackendSubscription: Send {
    fn recv(&mut self) -> Result<BackendWake, BackendError>;
}

pub enum BackendWake {
    RuntimeChanged { runtime_id: String },
    SessionChanged { execution_id: String },
    TurnChanged { execution_id: String, turn_id: String },
    InteractionPending { execution_id: String, interaction_id: String },
    WorkerChanged { execution_id: String, worker_id: String },
    TransportLost { runtime_id: String, evidence: String },
}
```

The daemon then:

1. journals the native event;
2. schedules an idempotent settlement attempt;
3. observes the relevant native object;
4. commits any Sergeant state transition;
5. deduplicates by native event identity or settlement generation.

This closes the Cerberus “turn ended but nothing called OBSERVE again” class and supports HTTP/SSE, stdio RPC, ACP, Agent SDK callbacks, and supervisor state changes.

---

## 5. Replace capability booleans with typed detail

Keep derived booleans for simple routing, but make the authoritative capabilities typed.

```rust
pub enum SessionCapability {
    None,
    ResumeOnly,
    ResumeAndReadFull,
    ResumeReadAndFork,
}

pub enum InterruptCapability {
    None,
    ProcessTreeTermination,
    NativeSessionStop,
    NativeTurnInterrupt,
    PerWorkerInterrupt,
}

pub enum InteractionCapability {
    None,
    Questions,
    Approvals,
    QuestionsAndApprovals,
    Elicitation,
}

pub enum StructuredOutputCapability {
    None,
    PromptEnforced,
    NativeSchema,
}

pub enum UsageCapability {
    None,
    Estimated,
    PartialNative,
    PerTurnNative,
    CumulativeNative,
}

pub enum HistoryCapability {
    None,
    AdapterObservedOnly,
    NativePartial,
    NativeComplete,
}

pub enum EvidenceLevel {
    None,
    Inferred,
    Documented,
    Structured,
    LiveMeasured,
}

pub struct CapabilitySet {
    pub sessions: SessionCapability,
    pub interrupt: InterruptCapability,
    pub interaction: InteractionCapability,
    pub structured_output: StructuredOutputCapability,
    pub usage: UsageCapability,
    pub history: HistoryCapability,
    pub orchestration: OrchestrationCapabilities,
    pub streaming: EvidenceLevel,
    pub human_attach: EvidenceLevel,
    pub model_selection: EvidenceLevel,
}
```

This removes false equivalences:

```text
kill process tree          != native turn interrupt
resume session             != read complete history
subagents may exist        != worker tree is observable
team exists                != team can be resumed
agent text says JSON       != native schema validation
logs command exists        != stable event stream
state file exists          != supported public protocol
```

---

## 6. Add capability provenance and stability

```rust
pub enum CapabilityProvenance {
    Documentation,
    AdapterFixture,
    UpstreamFixture,
    LiveAdmission,
}

pub enum StabilityTier {
    Stable,
    Preview,
    Experimental,
    PrivateProtocol,
}

pub struct AdmittedCapability<T> {
    pub value: T,
    pub provenance: CapabilityProvenance,
    pub stability: StabilityTier,
    pub harness_version: String,
    pub protocol_fingerprint: Option<String>,
    pub measured_at: String,
}
```

Claude Agent View and Agent Teams should never appear indistinguishable from a stable OpenCode HTTP endpoint merely because both can theoretically perform the same action.

---

## 7. Make authentication and configuration explicit

```rust
pub enum AuthMode {
    TerminalSession,
    SetupToken,
    ApiKey,
    ProviderCredentials,
    HostedAccount,
}

pub enum ConfigDiscovery {
    Ambient,
    SettingSources(Vec<String>),
    ExplicitOnly,
    Bare,
}

pub struct LaunchProfile {
    pub backend: BackendId,
    pub auth: AuthMode,
    pub config_discovery: ConfigDiscovery,
    pub permission_policy: PermissionPolicy,
    pub sandbox_policy: SandboxPolicy,
    pub model: Option<String>,
    pub environment: BTreeMap<String, String>,
}
```

Rules:

- `claude/direct + TerminalSession` must not silently become bare.
- `claude/direct + Bare` must prove valid API/provider credentials.
- `claude/supervisor` uses the documented interactive credentials of that supervisor instance.
- `claude/agent-sdk` must declare its valid SDK auth mode.
- every effective permission/sandbox/config source must be journaled;
- unsupported mappings fail rather than degrade.

---

## 8. Preserve Sergeant-owned workflow semantics

Harness-native orchestration is useful but subordinate:

```text
Sergeant workflow
  owns stage graph, evidence, retries, reviews, escalation, custody

Harness workflow / team / goal
  owns activity inside one Sergeant stage or execution
```

A Claude team deciding its shared tasks are complete does not automatically complete the Sergeant stage. A `/goal` declaring success does not bypass stage gates. A Codex subagent finishing does not bypass the parent turn. OpenCode child sessions do not become independent Sergeant work unless the workflow explicitly promotes them.

---

## 9. Terminal outcome remains an invariant

```rust
pub enum TurnTerminal {
    Completed { summary: Option<String> },
    NeedsInput { interaction: InteractionRef },
    Waiting { reason: String },
    Blocked { reason: String },
    Failed { reason: String },
    Interrupted,
}
```

Invariant:

> Every successfully launched turn eventually acquires exactly one terminal outcome. If a process, socket, supervisor worker, SDK stream, or server event source ends without a protocol-defined terminal outcome, the adapter produces a fail-closed ambiguous/transport-lost terminal with raw evidence. It never leaves the stage active with no settlement path.

The turn terminal is still not necessarily stage completion. The engine maps it according to the workflow contract and deterministic gates.

---

# Part VII — Claude-specific admission suite

Every Claude transport should run the same semantic suite, with transport-specific fixtures.

## Direct `-p`

- first turn and caller-selected session identity;
- resume across processes and directories;
- structured stream initialization and result;
- nested subagent event tree;
- API retry events;
- model pin evidence;
- stdin size and closure;
- SIGTERM and process-tree cleanup;
- result-envelope loss;
- transcript/history limits;
- ambient vs controlled settings;
- terminal-auth profile;
- bare/API profile;
- ask/approval behavior;
- permission denial behavior;
- exact terminal settlement after no client command.

## Agent View supervisor

- start a supervisor under an isolated `CLAUDE_CONFIG_DIR`;
- launch a `--bg` session;
- receive a durable native ID;
- enumerate with `agents --json`;
- map every documented state;
- reply to a waiting session;
- attach and detach without changing custody;
- stop and respawn with conversation intact;
- restart Sergeant while the supervisor remains alive;
- restart the supervisor while workers remain alive;
- restart into a new Claude version;
- delete a session and verify worktree behavior;
- classify complete history honestly;
- detect a session that disappears or stalls;
- verify model, permission, MCP, plugin, and settings carryover;
- verify subagent/workflow activity after detachment;
- prove whether usage and terminal outcomes are machine-readable;
- refuse unsupported capabilities rather than scraping private files silently.

## Agent SDK sidecar

- sidecar handshake and schema negotiation;
- SDK/package/CLI version fingerprint;
- session create, continue, resume, fork;
- session enumeration and full message read;
- structured output;
- programmatic question and approval;
- subagent start/stop;
- TypeScript teammate/task/worktree hooks;
- cancellation through abort signals;
- sidecar death and reconnection;
- external session storage;
- usage and model evidence;
- permission and sandbox mapping;
- auth-mode validation;
- raw SDK event preservation;
- exactly-once event translation.

## Agent teams

- lead and teammate identity;
- direct teammate messaging;
- shared task creation/claim/dependency;
- plan approval;
- teammate idle/completion;
- lead premature completion;
- teammate error/replacement;
- permission inheritance;
- one-team and no-nested-team limits;
- restart/resume behavior;
- task-lag behavior;
- shutdown and orphan cleanup;
- transcript visibility;
- file conflict/worktree behavior.

Until those pass, advertise:

```text
topology = PeerTeam
worker_identity = Structured or Inferred
resume_workers = false
nested_workers = false
stability = Experimental
```

---

# Part VIII — Implementation conclusions

## Does Sergeant need an engine change now?

Yes, but not a replacement.

The minimum engine change before adding the strategic adapters is:

```text
1. provider + transport identity
2. executable runtime ownership
3. separate session and turn handles
4. event-driven settlement/wakeup
5. typed interactions
6. typed orchestration capabilities
7. capability provenance and stability
8. explicit auth/config mode
```

That is a foundational refinement of the existing contract, not a new engine.

## Which Claude transport should Sergeant prefer?

### Today

```text
default admitted transport:
  claude/direct

experimental transport:
  claude/supervisor
```

### Strategic decision point

After the admission spikes:

```text
when terminal subscription auth and human attach dominate:
  claude/supervisor

when typed lifecycle, hooks, history, and programmatic control dominate:
  claude/agent-sdk

when strict deterministic scripting dominates:
  claude/direct + bare/api profile

when hosted environments and API lifecycle are desired:
  claude/managed-agents
```

There may not be one permanent “best Claude adapter.” Supporting multiple transports under one provider is the honest design, exactly as recommended for Codex (`exec` and App Server) and Goose (`run` and ACP).

## Which runtime is most likely to become Sergeant’s long-term Claude default?

**Agent View / supervisor mode is the strongest candidate for the local, terminal-authenticated default** because it preserves the installed Claude Code product while providing a durable runtime, background sessions, human attachment, session inventory, respawn, worktrees, and native long-running activity.

However, it should not replace direct `-p` until Anthropic exposes or Sergeant validates a sufficiently stable machine protocol for:

- events;
- replies/interactions;
- history;
- terminal outcomes;
- usage;
- turn identity;
- version negotiation.

**The Agent SDK is the strongest candidate for the high-control default** if Sergeant accepts the TypeScript sidecar and explicit SDK authentication boundary. It is the only current Claude surface with the typed hooks and session APIs needed to rival Codex App Server and OpenCode Server cleanly.

## What should happen to the current Claude adapter?

Do not discard it.

Reclassify it:

```text
Claude provider
  ├── direct adapter       ← current implementation
  ├── supervisor adapter   ← new strategic spike
  ├── Agent SDK adapter    ← optional typed sidecar
  └── hosted adapter       ← separate future family
```

Then move process driving, stream archiving, event normalization, profile resolution, and admission evidence into reusable components shared where appropriate.

---

# Final conclusion

The first report’s headline should be revised from:

> Claude is a good baseline, not the universal shape.

to:

> **Claude direct print mode is a good baseline, but modern Claude is itself a multi-runtime ecosystem.**

OpenCode and Codex App Server still expose the cleanest stable machine control planes today. Goose ACP still deserves a strategic spike. But Claude can no longer be represented honestly by one `-p` row.

The architecture Sergeant should take into gauntlet review is:

```text
                         SERGEANT ENGINE
                                │
             durable work / workflow / evidence / policy
                                │
                 normalized backend contract v2
                                │
        ┌───────────────────────┼────────────────────────┐
        │                       │                        │
     provider                runtime                 execution
    + transport              lease                 / session / turn
        │                       │                        │
        ├── events and wakeups  ├── identity             ├── workers
        ├── interactions        ├── generation           ├── tasks
        ├── history             ├── health               ├── tools
        ├── capabilities        └── ownership            └── terminal
        └── raw evidence
                                │
       ┌─────────────┬──────────┼──────────┬─────────────┐
       │             │          │          │             │
 Claude direct   Claude      Claude      Codex       OpenCode
     -p          supervisor  Agent SDK   app-server    server
       │             │          │          │             │
       └─────────────┴──────────┴──────────┴─────────────┘
                                │
                explicit profile/auth/configuration
```

The immediate architectural investment is not “rewrite the Claude adapter around Agent View.” It is:

> **Make runtime strategy, session/turn identity, event-driven settlement, interactions, orchestration topology, and capability evidence explicit—then admit each Claude strategy by measurement.**

That preserves the work already done, allows the current Cerberus remediation to land cleanly, and prevents the engine from being frozen around the narrowest execution surface just as every major harness is moving toward persistent runtimes and multi-agent orchestration.

---

# Source appendix

## Sergeant and no-mistakes

**[S1] Sergeant repository**  
https://github.com/miztertea/sergeant-rs

**[S2] Sergeant backend contract**  
https://github.com/miztertea/sergeant-rs/blob/main/src/backend/mod.rs

**[S3] Sergeant Claude adapter**  
https://github.com/miztertea/sergeant-rs/blob/main/src/backend/claude.rs

**[S4] Sergeant Cerberus close-out PR #51**  
https://github.com/miztertea/sergeant-rs/pull/51

**[S5] no-mistakes repository**  
https://github.com/kunchenguid/no-mistakes

**[S6] no-mistakes Claude adapter**  
https://github.com/kunchenguid/no-mistakes/blob/main/internal/agent/claude.go

**[S7] no-mistakes native command driver**  
https://github.com/kunchenguid/no-mistakes/blob/main/internal/agent/native_command.go

**[S8] no-mistakes Unix process-group supervision**  
https://github.com/kunchenguid/no-mistakes/blob/main/internal/shellenv/shell_command_unix.go

## Claude official sources

**[C1] Programmatic Claude Code, `-p`, bare mode, structured streaming, subagents, retries, init metadata, resume**  
https://code.claude.com/docs/en/headless

**[C2] Agent View, background sessions, supervisor, JSON inventory, shell management, state, worktrees, limitations**  
https://code.claude.com/docs/en/agent-view

**[C3] Agent Teams, teammate messaging, task graph, permissions, and limitations**  
https://code.claude.com/docs/en/agent-teams

**[C4] Agent SDK overview and supported language/product boundaries**  
https://code.claude.com/docs/en/agent-sdk/overview

**[C5] Agent SDK sessions, resume, fork, enumeration, history, and cross-host behavior**  
https://code.claude.com/docs/en/agent-sdk/sessions

**[C6] Agent SDK hooks and lifecycle/orchestration events**  
https://code.claude.com/docs/en/agent-sdk/hooks

**[C7] Claude Code authentication and setup tokens**  
https://code.claude.com/docs/en/authentication

**[C8] Claude Code features in the Agent SDK, including the agent-team boundary**  
https://code.claude.com/docs/en/agent-sdk/claude-code-features

**[C9] Agent View and goals release documentation**  
https://code.claude.com/docs/en/whats-new/2026-w20

**[C10] Managed Agents multi-agent orchestration**  
https://platform.claude.com/docs/en/managed-agents/multiagent-orchestration

**[C11] Claude Code routines**  
https://code.claude.com/docs/en/routines

## Other priority harnesses

**[O1] Codex noninteractive mode**  
https://developers.openai.com/codex/noninteractive

**[O2] Codex App Server**  
https://developers.openai.com/codex/app-server

**[O3] Codex CLI reference**  
https://developers.openai.com/codex/cli/reference

**[O4] Codex subagents**  
https://developers.openai.com/codex/subagents

**[O5] OpenCode server**  
https://opencode.ai/docs/server/

**[O6] OpenCode agents**  
https://opencode.ai/docs/agents

**[O7] OpenCode permissions**  
https://opencode.ai/docs/permissions

**[G1] Goose project documentation**  
https://block.github.io/goose/

**[G2] Polpo multi-harness implementation evidence**  
https://github.com/pugliatechs/polpo
