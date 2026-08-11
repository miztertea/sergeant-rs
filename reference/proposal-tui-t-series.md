---
type: proposal
title: "Sergeant-rs T-Series: Work-Centered Terminal Interface"
description: >-
  Proposal to replace Sergeant-rs's deliberately minimal P0 Ratatui fleet/detail
  stub with a work-centered terminal interface for setting intent, operating
  durable Work, discovering repository-owned workflows, and responding to
  agent-initiated questions. The program preserves existing Work and execution
  semantics, adds one narrow read-only workflow-catalog projection required by
  the equal-client boundary, closes the known TUI layout and reconnect gaps, and
  disables the embedded web dashboard until the terminal interaction model is
  proven. Full journal and DuckDB exploration is reserved for a separate
  proposal.
status: proposed
resource: sergeant-rs
tags:
  - sergeant-rs
  - tui
  - ratatui
  - usability
  - workflows
  - work-centered
  - proposal
timestamp: 2026-08-11
repository: https://github.com/miztertea/sergeant-rs
audit_revision: a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6
relationship: >-
  Presentation successor/addendum to sections 7, 29-31, and 34 of
  reference/proposal-depot-rust-execution-surface.md, the M6 client-surface
  contract, and the now-merged N3 executor-aware workflow surface. It replaces
  the P0 presentation while preserving the journal-first runtime and existing
  mutation semantics. A separate future proposal owns the global Journal and
  DuckDB query surface.
---

# Sergeant-rs T-Series
## Work-Centered Terminal Interface

**Status:** Proposed  
**Audit basis:** [`miztertea/sergeant-rs@a5fb875`](https://github.com/miztertea/sergeant-rs/tree/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6)  
**Relationship to P0:** Replace the P0 presentation; preserve the P0 contracts  
**Relationship to N3:** Consume the merged actor-initiated ask, execution-reservation, and per-stage executor surfaces; add no new execution semantics  
**Primary objective:** Make setting intent, understanding Work, selecting admitted procedure, and supplying human decisions natural from bare `sgt`  
**Mutation boundary:** Submit, respond, retry, and cancel retain their existing API and engine meanings  
**Additive read boundary:** One read-only workflow-catalog route exposes existing `.sergeant/` procedure through the equal-client API boundary  
**Journal boundary:** Global journal/DuckDB search and exploration belongs to a separate proposal  
**Web disposition:** `/ui` is unmounted; `sgt web` reports the dashboard disabled; source remains as a dormant future stub  

Sections are numbered for contract citation (§N), following the repository's proposal convention.

---

# 1. Executive Summary

Sergeant-rs already has the difficult product beneath the interface.

A user states an intent. The daemon discovers the repository or workspace, resolves and pins procedure, creates an isolated Git worktree, routes each actor stage to a measured native harness, records every transition in an append-only journal, and resumes or fails closed from evidence. The daemon—not the terminal—is the application. Work—not the process, conversation, TUI, or web page—is the durable center. Every projection can be rebuilt from the journal, and every client reaches the daemon through the same loopback HTTP/SSE boundary.

The current TUI is not a failed interface. It is a successful P0 proof. The M6 contract intentionally required only a live Fleet screen, a Work-detail screen, response and cancellation actions, and terminal-safe lifecycle behavior. The implementation therefore renders the API almost literally: fixed-width Fleet rows, a property sheet, and a reverse-chronological raw event tail. That proved the architecture. It did not yet answer the ordinary human questions:

```text
What should Sergeant do next?
What needs me right now?
What is this Work trying to accomplish?
Where is it in its procedure?
What did the agent ask?
What can I safely do from here?
What evidence explains what happened?
```

The current main revision strengthens the case for a real operator interface. N3 has shipped actor-initiated questions, per-stage actor selection, execution reservations, and current-stage executor details. The public event vocabulary now includes `conversation.ask`, `execution.reserved`, and `execution.abandoned`; the Work view exposes the current reservation and the executor pinned for the current stage. These are existing facts that the P0 screen does not yet arrange into a human-usable experience.

This proposal makes bare `sgt` the primary interactive surface.

The top navigation is intentionally small:

```text
Home    Fleet    Workflows
```

- **Home** is the front door: state intent, choose admitted procedure when needed, see the Work requiring attention, and return to current Work.
- **Fleet** is the complete browser over durable Work.
- **Workflows** is the read-only catalog of admitted repository procedure discovered through `.sergeant/index.md`, each workflow's `index.md`, and its authoritative `workflow.toml`.

A later **Journal** surface is deliberately absent. Sergeant's journal and DuckDB projection deserve their own proposal, query contract, evidence model, and performance review. This program keeps Work-local Evidence, the existing one-Work graph, and current canned analytics reachable, but it does not disguise a bounded event tail as the historical exploration product.

A collapsible **Attention drawer**, toggled by `~`, is available across the TUI. It groups existing Work states into human-relevant queues: needs input, trouble, in flight, waiting, and terminal. A static gold `? N` indicator in the header makes pending human decisions visible when the drawer is closed. The drawer stores no notifications and creates no new state; it is a client-side view over the Fleet.

Opening a Work from Home, Fleet, the Attention drawer, a graph result, or an analytics row always opens one canonical Work surface. That surface intentionally borrows the interaction grammar people already know from agent harnesses and modern developer tools:

```text
horizontal navigation
optional left drawer
Work header and workflow rail
scrolling semantic thread
fixed composer/action region at the bottom
secondary Workflow, Evidence, Graph, and Details views
```

The resemblance is an affordance, not an architectural claim. The scrolling thread renders only journaled facts: Work intent, workflow binding, stage transitions, current executor, execution lifecycle, agent messages, tool activity, actor-authored questions, human responses, usage, blockers, failures, and terminal outcomes. It never presents hidden chain-of-thought, inferred file changes, invented artifacts, or guessed progress.

The bottom composer is persistent, but its behavior is state-aware:

```text
Home              ordinary text submits new Work
Work.needs_input  ordinary text answers the current request
all other Work    ordinary text is disabled; / opens valid local commands
```

`Enter` inserts a newline. `Ctrl+Enter` deliberately submits when the terminal reports the modifier. A visible Send action, reached by `Tab` and activated by ordinary `Enter`, is the universal fallback for terminals that cannot distinguish `Ctrl+Enter`. This makes durable submission and resumption harder to trigger by accident without depending on terminal-specific keyboard extensions.

Two lightweight TUI grammars make the interface discoverable without changing Sergeant's CLI or engine:

```text
/command     navigate the TUI or invoke an existing Sergeant operation
@workflow    select or reference admitted repository procedure
```

Slash commands are a fixed local enum over existing views and operations; they are not new `sgt` subcommands and are never sent to an actor. On Home, selecting `@repo-to-icm` sets the existing `workflow` submission field and appears as a chip outside the durable intent. Inside an existing Work, the pinned workflow cannot change; an `@workflow` selection merely inserts a textual reference into an answer.

The workflow catalog is the proposal's one narrow additive read surface. The repository already defines the catalog and publication boundary: `.sergeant/index.md` lists admitted workflows, each admitted workflow owns an `index.md`, and `workflow.toml` owns executable identity, version, stage order, and N3 executor metadata. The TUI may not read those files directly because clients are equal. The daemon therefore exposes one side-effect-free catalog endpoint using the same workspace discovery and workflow loader that submission already trusts. It adds no workflow grammar, mutation, publication, or durable state.

The embedded web dashboard is disabled for this program. Its source and assets remain as a future stub; `/ui` is not mounted, and `sgt web` reports that the browser surface is unavailable while the terminal interaction model is being proven. Dashboard issues [#15](https://github.com/miztertea/sergeant-rs/issues/15) and [#21](https://github.com/miztertea/sergeant-rs/issues/21) remain visible reactivation prerequisites rather than becoming a parallel usability lane.

The central design rule is:

> **Make the existing work domain perceptible. Add only the smallest read surface required to expose repository-owned procedure through the architecture that already exists.**

Every normative decision in this proposal carries its Ponytail rung. The complete register is §20.

---

# 2. Audit Basis and Method

This proposal is based on a read-only audit of main at [`a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`](https://github.com/miztertea/sergeant-rs/commit/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6).

**Decision T-01 (R1):** The audit revision is pinned. “Current” means `a5fb875`, not an unqualified moving branch.

That revision contains the completed workstreams that were still open during the first design pass:

- [PR #28](https://github.com/miztertea/sergeant-rs/pull/28), the S-series coverage and stabilization program, merged at `6a7bedf`; its close-out reports 94.63% line coverage and 294 tests plus two opt-in at its own ship point.
- [PR #43](https://github.com/miztertea/sergeant-rs/pull/43), N3 and generator v2, merged as the audit commit; N3 shipped the two-phase external-effect boundary, tagged stage definitions, per-stage actor selection, current-stage executor details, actor-initiated ask, and new reservation events.
- [PR #48](https://github.com/miztertea/sergeant-rs/pull/48), merged into the N3 branch before #43 landed, promoting binding development rules and moving gauntlet scripts into visible `resources/` paths.

The audit included:

- [`CLAUDE.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/CLAUDE.md), especially journal truth, single ownership, equal clients, Work/process separation, current test discipline, and the instruction to extend the API rather than give a client a private shortcut;
- [`AGENTS.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/AGENTS.md), including discovery through `.sergeant/index.md` and the instruction to use real respond, retry, cancel, and inspection surfaces;
- [`reference/proposal-depot-rust-execution-surface.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/reference/proposal-depot-rust-execution-surface.md), especially §§7, 10, 21–31, 34, and 40;
- [`reference/proposal-next-iteration-icm-workflows.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/reference/proposal-next-iteration-icm-workflows.md), plus the N0–N3 contracts and rulings that now govern stage executors and actor asks;
- [`docs/gauntlet/contracts/N3.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/docs/gauntlet/contracts/N3.md), particularly the explicit statement that actor-authored questions resume through today's `respond` operation;
- the current TUI in [`src/tui.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/tui.rs), CLI in [`src/cli.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/cli.rs), API and client in [`src/api.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/api.rs), and embedded browser stub in [`src/web.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/web.rs);
- the current workflow model in [`src/domain/workflow.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/domain/workflow.rs), including tagged actor stages, per-stage harness/profile fields, pinned stage bindings, and content identity;
- the repository-owned catalog in [`.sergeant/index.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/.sergeant/index.md), [`repo-to-icm/index.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/.sergeant/workflows/repo-to-icm/index.md), and its [`workflow.toml`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/.sergeant/workflows/repo-to-icm/workflow.toml);
- the normative [ICM filesystem convention](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/docs/icm/convention.md), which defines `.sergeant/index.md` as the discovery surface, requires every admitted workflow to be listed, and excludes `.sergeant/drafts/workflows/` from runnable procedure;
- the current analytical and graph projections in [`src/runtime/analytics.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/runtime/analytics.rs) and [`src/runtime/graph.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/runtime/graph.rs);
- [`GAUNTLET.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/GAUNTLET.md), [`LESSONS.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/LESSONS.md), [`reference/notes/ideaos-agent-contract.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/reference/notes/ideaos-agent-contract.md), and [`reference/notes/gauntlet-pattern.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/reference/notes/gauntlet-pattern.md);
- the M6 Ratatui test strategy in [`tests/m6_surfaces.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/tests/m6_surfaces.rs);
- the measured P1 baseline and current open client/runtime issues that constrain the presentation: [#11](https://github.com/miztertea/sergeant-rs/issues/11), [#16](https://github.com/miztertea/sergeant-rs/issues/16), [#26](https://github.com/miztertea/sergeant-rs/issues/26), [#45](https://github.com/miztertea/sergeant-rs/issues/45), [#46](https://github.com/miztertea/sergeant-rs/issues/46), [#47](https://github.com/miztertea/sergeant-rs/issues/47), [#15](https://github.com/miztertea/sergeant-rs/issues/15), and [#21](https://github.com/miztertea/sergeant-rs/issues/21);
- the repo screenshots in `docs/img/` and the owner-reviewed, Ratatui-tempered mockups produced during the design conversation.

The IdeaOS/Notion review included:

- [Ponytail Minimality Ladder](https://app.notion.com/p/39a27ada618f8100babadb321a70de9b);
- [IdeaOS Agent Instructions](https://app.notion.com/p/39a27ada618f815aab89daafc635514f);
- [Sergeant](https://app.notion.com/p/3ad27ada618f8175a6afc0dcd1707799);
- [Work-Centered Intelligence](https://app.notion.com/p/3ac27ada618f81728a73fbd7ac90c61c);
- [WorkPacket](https://app.notion.com/p/39a27ada618f818cba42f5efe8ffe1f0);
- [Work Filesystem](https://app.notion.com/p/3ac27ada618f819d8196fa78ab420224);
- [Shared-Engine Human-Agent Workbench](https://app.notion.com/p/39a27ada618f81999694e0fbb019ca50);
- [Ecological Interface Design: Theoretical Foundations](https://app.notion.com/p/3ac27ada618f81909dd5d48e1f9b9912);
- [Intelligent Work Environments Research and Prior Art Map](https://app.notion.com/p/3ac27ada618f817b8418e50151dd7015);
- [The new rules of context engineering for Claude 5 generation models](https://app.notion.com/p/3af27ada618f8188806de090bd721054);
- [Bugle](https://app.notion.com/p/3ab27ada618f81158b63ff644f4ac548);
- the deliberately parked [Garrison Business User Workspace](https://app.notion.com/p/3ab27ada618f812db874fbebc0eaf9d8).

The evidence hierarchy is:

```text
current implementation at a5fb875
        ↓
committed contracts, ledger, lessons, measured baselines, and open issues
        ↓
current README and admitted .sergeant content
        ↓
adjudicated reference corpus and vendored historical evidence
        ↓
IdeaOS research and adjacent-system concepts
        ↓
owner-reviewed mockups and interaction preferences
```

## 2.1 Owner amendments after the strict draft

The first written draft deliberately excluded every capability without an existing endpoint. Review exposed that this collapsed cheap client affordances, existing repository-owned procedure, and genuine new runtime behavior into one bucket.

**Decision T-02 (R1/R2):** The settled proposal supersedes that strict boundary in exactly three ways:

1. `/` is admitted as a local TUI palette over existing views and operations.
2. `@` is admitted as a local workflow-selection/reference affordance.
3. One read-only workflow-catalog endpoint is admitted because the catalog already exists in `.sergeant/`, and the equal-client invariant forbids the TUI from reading it privately.

The owner also settled the following boundaries:

- global journal/DuckDB exploration receives its own proposal;
- top navigation is `Home / Fleet / Workflows`, not `Work / Workflows`;
- the web dashboard is disabled rather than developed in parallel;
- CPU, memory, disk, OpenTelemetry query integration, and mouse operation are not part of this work;
- issue #16 auto-reconnect belongs in the TUI usability program;
- `Enter` inserts a newline and `Ctrl+Enter` is the deliberate send chord, with a visible Send fallback.

These are current proposal inputs, not implementation afterthoughts.

---

# 3. Doctrine

## 3.1 Work is the durable center

**Decision T-03 (R2):** The interface is organized around durable Work, not processes, harnesses, sessions, panes, or raw events.

This directly reuses the implemented domain boundary. Work carries intent and durable lifecycle state. Workflow stage is an orthogonal coordinate. Execution is a native context. A per-turn OS process is evidence rather than Work state. N3 adds reservation and current-stage executor details without collapsing those coordinates.

Work-Centered Intelligence supplies the broader design language:

> **Don't push work through the model. Put the model in the work.**

The scrolling thread is therefore a portal into Work. It is not the authoritative object, and the TUI never implies that a conversation owns the procedure or continuity.

## 3.2 Intent is the primary human act

**Decision T-04 (R2):** Home begins with the question “What should Sergeant do?” and maps the answer to the existing Work submission operation.

The README already defines the product this way: submit intent, get a durable agent run, watch it or walk away. Starting new Work must not require leaving the TUI for an otherwise equivalent CLI invocation.

## 3.3 Familiar harness grammar is an affordance

**Decision T-05 (R2/R5):** The interface deliberately uses the familiar shape of contemporary agent harnesses—top navigation, a switchable left rail, a scrolling thread, and a bottom composer—implemented with the Ratatui stack already pinned in the repository.

Familiarity reduces orientation cost. It does not license hidden chat semantics. The composer and thread remain constrained by Sergeant's state machine and journal evidence.

## 3.4 The interface reveals constraints; it does not invent them

**Decision T-06 (R2):** Work state, workflow stage, executor, execution lifecycle, input request, and evidence remain visibly separate.

Ecological Interface Design begins from the work domain and makes invariant constraints perceptible rather than forcing the operator to reconstruct them mentally. Sergeant already contains those distinctions; the TUI must preserve them while improving hierarchy.

The UI may derive labels, grouping, truncation, and navigation. It may not derive new Work state, authority, completion, file changes, success, or progress.

## 3.5 Clients remain equal

**Decision T-07 (R2):** `src/tui.rs` continues to receive runtime and repository procedure only through `ApiClient`.

The TUI gets no journal handle, DuckDB connection, engine reference, backend registry, daemon state, or filesystem shortcut. The workflow catalog is exposed through an endpoint precisely because repository-owned workflow content participates in execution planning and must not become a TUI-only interpretation.

## 3.6 Progressive disclosure replaces architectural display

**Decision T-08 (R2/R5):** Intent, attention, current stage, current question, and meaningful activity are primary. ULIDs, full paths, native IDs, reservation IDs, route source, content hashes, raw payloads, graph provenance, and SQL are secondary.

This follows both the Work Filesystem's “one responsibility per surface” rule and Anthropic's context-engineering guidance: place information where it becomes useful rather than loading every fact at once. The current property sheet is accurate but hierarchically flat. The redesign changes presentation, not truth.

## 3.7 Mutations require deliberate confirmation

**Decision T-09 (R2/R5):** Durable submission and human response do not fire on naked Return.

`Enter` means newline inside a composer. `Ctrl+Enter` means submit when distinguishable. The focused Send action is the portable confirmation path. Cancel retains explicit confirmation. Retry is a named action, never an incidental effect of typing.

## 3.8 Ponytail is binding

**Decision T-10 (R2):** Every implementation choice follows the repository's Ponytail ladder in order:

```text
R1  skip it when the need is not demonstrated
R2  reuse what is already in the repository
R3  use the standard library
R4  use a native platform capability
R5  use an already installed dependency
R6  add one local line or tiny composition
R7  add the minimum new machinery, naming failed lower rungs
```

R7 decisions in this proposal are exceptional and name the failed lower rungs. A future builder does not get to reinterpret “rich TUI” as permission for a UI framework, state store, plugin system, animation package, or generalized command language.

---

# 4. Current Surface, as Implemented

## 4.1 The P0 TUI

The current [`src/tui.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/tui.rs) contains:

```text
Screen::Fleet
Screen::Detail
```

It already proves several load-bearing properties:

- all state enters through `ApiClient`;
- SSE events invalidate and trigger authoritative rereads rather than becoming a second client reducer;
- response and cancel use real mutation endpoints;
- detached live state is durable on screen rather than a transient status message;
- SIGTERM, SIGHUP, pty disappearance, panic, and requested exit share bounded terminal restoration behavior;
- the keymap is separated from action execution and therefore testable.

The problem is presentation hierarchy, not missing architectural discipline.

## 4.2 Existing v1 routes

**Decision T-11 (R2):** T-series reuses every existing route below and adds only the catalog route defined in §10.3.

| Existing route | Existing meaning | TUI use |
|---|---|---|
| `GET /v1/system` | version, API revision, data dir, journal head | header and connection overlay |
| `GET /v1/work` | all Work, current stage, resolved Work backend | Home, Fleet, Attention drawer |
| `POST /v1/work` | submit Work | Home composer |
| `GET /v1/work/{id}` | Work, stage, surface, execution, reservation, workflow, backend, route, teardown | canonical Work view |
| `POST /v1/work/{id}/input` | answer Work in `needs_input` | Work composer |
| `POST /v1/work/{id}/retry` | retry the current retryable stage | Work action |
| `POST /v1/work/{id}/cancel` | cancel Work | Work action |
| `GET /v1/events` | journal history after seq, optionally Work-filtered and limited | Work thread and Evidence |
| `GET /v1/events/stream` | resumable SSE journal tail | live invalidation |
| `GET /v1/graph/work/{id}` | one Work's provenance neighborhood | Work Graph view |
| `GET /v1/analytics` | canned-query index and projection counts | palette-accessed Analytics utility |
| `GET /v1/analytics/{name}` | one canned result | Analytics utility |

`ApiClient` already provides generic authenticated `get` and `post` methods in addition to typed helpers. The implementation may add typed convenience methods without changing endpoint meaning.

## 4.3 N3 facts now available to the TUI

**Decision T-12 (R2):** The TUI visibly consumes N3's merged public facts rather than designing as though main were still P0.

The Work view now exposes:

```text
reservation
stage.executor
workflow stages
```

The SSE vocabulary now includes:

```text
execution.reserved
execution.abandoned
conversation.ask
```

The semantic consequences are presentation-only:

- an actor-authored `conversation.ask` becomes the primary gold question card;
- `stage.executor` identifies the harness/profile actually responsible for the current checkpoint;
- a reservation can be shown as a compact launch-state line rather than an unexplained gap before `execution.started`;
- `execution.abandoned` is visible evidence, not silently folded into failure prose.

The proposal does not add or modify the N3 signal pathway.

## 4.4 Existing mutation semantics

The operator can currently:

```text
submit
respond
retry
cancel
```

The runtime decides whether a mutation is legal. The TUI can hide actions known to be invalid in the projected state, but it is not a second authorization or transition engine.

**Decision T-13 (R1/R2):** The TUI invokes only those existing mutation operations and always rereads authoritative state after a successful write.

It does not synthesize active-turn guidance, implicit interrupts, workflow rebinding, terminal Work resurrection, or local-only “acknowledged” state.

## 4.5 Existing event vocabulary

The event stream carries Work lifecycle, workflow binding, stage lifecycle, execution lifecycle, surface lifecycle, conversation messages and asks, tool calls, usage, commands, daemon events, and backend probes.

**Decision T-14 (R2):** The semantic thread is a pure presentation fold over known event kinds. Unknown events remain visible in Evidence and never receive an invented card.

Raw evidence always remains reachable because a semantic summary can be wrong even when the event is right.

## 4.6 Existing graph and analytics limits

The graph currently derives relationships among Work, workflow, stages, executions, backends, profiles, native sessions, repositories, messages, tool calls, models, workspaces, and clients. Every edge carries the journal sequence that justifies it. It deliberately does not claim Artifact, File, Commit, or Finding nodes because no current event family reports those facts.

DuckDB currently exposes five canned questions:

```text
blocked_time_per_work
backend_retries
execution_touched
tool_calls_before_failure
token_totals_per_work
```

**Decision T-15 (R1/R2):** T-series renders those exact graph and analytics results. It adds no graph canvas, arbitrary SQL, global search language, file browser, artifact model, or commit view.

## 4.7 The workflow catalog already exists as content

The current ICM convention states:

- `.sergeant/index.md` is the root discovery surface;
- every admitted workflow under `.sergeant/workflows/` must appear there;
- each workflow's `index.md` owns its human description and tags;
- `workflow.toml` owns executable name, version, stage order, and N3 executor metadata;
- `.sergeant/drafts/workflows/` is not admitted procedure.

**Decision T-16 (R2):** Workflow discovery uses this existing publication contract. It does not scan for arbitrary TOML files, list drafts, or introduce a second catalog format.

**Decision T-17 (R2/R6/R7):** Expose that existing catalog through one minimum authenticated read-only endpoint because the equal-client boundary forbids `src/tui.rs` from reading repository files privately. The exact route, response, lower-rung analysis, and failure behavior are specified in §10.3–§10.4.

## 4.8 CLI-only diagnostics

`sgt doctor` performs local tool, filesystem, journal, projection, and daemon checks and intentionally does not auto-spawn the daemon. That operation is not available through the HTTP API.

**Decision T-18 (R1):** Doctor remains a CLI-only diagnostic. The TUI has a small connection overlay, not a System dashboard.

## 4.9 The journal surface is a separate product gap

The journal and DuckDB projection are substantially richer than the current operator query surface. A real Journal product must decide query grammar, pagination, historical windows, result provenance, performance, and the relationship between DuckDB answers and source journal events.

**Decision T-19 (R1):** Global Journal/DuckDB exploration is removed from T-series and reserved for a separate proposal. T-series does not pre-allocate its API routes, query syntax, saved views, or top-level tab behavior.

Work-local Evidence and current canned analytics stay reachable because they already exist; they are not represented as the eventual Journal product.

---

# 5. Scope Contract

## 5.1 In scope

The program may:

1. replace the TUI's screen hierarchy, view models, keymap, focus, scrolling, and rendering;
2. submit new Work through the existing `POST /v1/work` fields, with intent primary;
3. add one read-only workflow-catalog route over admitted `.sergeant/` content and the embedded fallback;
4. add fixed local `/` and `@` grammars to the TUI;
5. group, sort, filter, truncate, and search already-loaded Fleet rows client-side;
6. present one canonical Work surface from every TUI entry point;
7. render a semantic Work thread from existing journal events, including N3 asks and reservation events;
8. render the pinned workflow stage list, current executor, current stage, and attempt from existing Work data;
9. answer, retry, cancel, refresh, and navigate through existing API operations;
10. render Work-local raw Evidence, the existing graph neighborhood, and current canned analytics;
11. add responsive Ratatui layouts, standard terminal colors, focus styling, scrollbars, and restrained activity indicators;
12. close issue #11 with width-aware layout and a falsifiable geometry test;
13. close issue #16 with truthful auto-reconnect, refresh-before-resume, capped backoff, and explicit authentication failure;
14. disable the web route and make `sgt web` report unavailability;
15. update README, screenshots, tests, ledger, and proposal cross-references when implementation lands.

## 5.2 Explicit non-goals

**Decision T-20 (R1):** The following are not part of T-series:

- a global journal query API, full-text search, arbitrary SQL, saved search, or alerting;
- new Work states, stage kinds, event kinds, mutation routes, workflow grammar, or durable stores;
- workflow authoring, publication, mutation, or generation from the TUI;
- listing `.sergeant/drafts/workflows/` as runnable procedure;
- arbitrary input to active Work;
- active-turn interrupt semantics;
- changing actor-initiated ask behavior;
- changing per-stage routing or executor semantics;
- file content, diff, artifact, commit, or finding views not supported by current events;
- host, daemon, or child-process CPU/memory/disk metrics;
- OpenTelemetry query integration;
- mouse interaction;
- a web-dashboard redesign;
- a graph canvas;
- a generalized plugin, command, notification, or keybinding framework;
- archival, retention, dismissal, or pinning semantics for old Work;
- issue #26's pre-loop terminal-hangup repair;
- issue #45's dropped-daemon-under-load investigation.

The one new API route is read-only and exposes existing admitted procedure. No other API addition is licensed by this proposal.

---
# 6. Information Architecture

## 6.1 Top-level navigation

**Decision T-21 (R2/R5):** The top navigation contains exactly:

```text
Home    Fleet    Workflows
```

- **Home** is intent plus attention.
- **Fleet** is the complete Work browser.
- **Workflows** is admitted-procedure discovery and selection.

There is no top-level **Work** tab because Work is the singular object entered from other surfaces. There is no top-level **Journal** tab until the separate journal proposal defines one. There is no **System** tab because the API exposes only basic connection identity, not a diagnostic or resource model.

The header also contains:

```text
sergeant   Home  Fleet  Workflows      ? 2      live · seq 128492
```

The selected mode is visibly styled. `Tab` and `Shift+Tab` move among focusable regions; left/right move within the top navigation when it has focus. The live indicator opens a read-only overlay containing version, API revision, data directory, journal head, and connection state.

## 6.2 Attention drawer

**Decision T-22 (R2/R5):** A collapsible left drawer, toggled by `~`, groups the existing Fleet into operator-relevant queues.

```text
NEEDS YOU
  needs_input

TROUBLE
  blocked
  failed

IN FLIGHT
  pending
  active

WAITING
  waiting

TERMINAL
  completed
  canceled
```

The order is attention order, not a new priority field. Within each group, Work remains ordered by its existing ULID/submission order unless a screen-local sort is selected.

The drawer:

- is open by default on wide terminals;
- is closed by default on medium terminals;
- becomes an overlay on narrow terminals;
- restores the selected Work and scroll position when toggled;
- shows intent first and state/stage second;
- never stores read/unread, dismissal, snooze, or notification history.

When closed, a static gold `? N` in the header reports the count of `needs_input` Work. It does not blink or pulse. Motion is reserved for active execution state.

## 6.3 Canonical Work navigation

**Decision T-23 (R2):** Every Work selection opens the same full-screen Work surface.

Entry points include:

```text
Home summary
Attention drawer
Fleet
Workflow run history where already present in loaded Fleet data
Graph node or edge carrying a work_id
Analytics row carrying a work_id
```

The TUI maintains a navigation stack. `Esc` returns to the exact prior mode, focus, filter, selection, and scroll position. Opening Work from a filtered Fleet must not drop the user back into an unfiltered list.

The Work surface is not a small popup. It occupies the application body while preserving the global header and optional drawer.

## 6.4 Responsive composition

**Decision T-24 (R5):** Ratatui's installed layout and widget primitives implement three explicit compositions.

### Wide — approximately 150 columns and above

```text
attention drawer | main thread or mode body | contextual rail
```

### Medium — approximately 100–149 columns

```text
optional drawer | main body
contextual detail becomes a selectable view
```

### Narrow — approximately 80–99 columns

```text
main body only
drawer and secondary detail appear as overlays or full-body views
```

Below the contract minimum of `80×24`, the TUI must remain safe and legible enough to tell the operator the terminal is too small. It may reduce content, but it must not panic, overlap the composer, or leave the terminal corrupted.

Widths are acceptance fixtures, not exact visual snapshots. Content priority determines degradation:

```text
intent / question / state
    before
stage / executor / workflow
    before
backend / workspace / short id
    before
full ids / paths / provenance
```

---

# 7. Interaction Grammar

## 7.1 The persistent composer

**Decision T-25 (R2/R5):** A composer occupies the bottom region across all primary modes.

Its label and behavior depend on context:

| Context | Label | Ordinary text |
|---|---|---|
| Home | `NEW WORK` | creates a Work through `POST /v1/work` |
| Work in `needs_input` | `ANSWER` | posts to `/v1/work/{id}/input` |
| Other Work states | `COMMAND` | disabled as agent input; `/` remains available |
| Fleet | `FILTER OR COMMAND` | filters when filter focus is chosen; `/` opens palette |
| Workflows | `FILTER OR COMMAND` | filters catalog; `@` opens workflow chooser |

The composer is one visual component with context-specific semantics, not a universal chat channel.

An ordinary printable character in a read-only Work does not silently become guidance. The TUI explains that the Work is not awaiting input and offers valid commands instead.

## 7.2 Multiline input and deliberate send

**Decision T-26 (R2/R5):** Composer submission uses:

```text
Enter          insert newline
Ctrl+Enter     submit, when the terminal reports the modifier
Tab            move focus to [ Send ]
Enter on Send  submit on every supported terminal
Esc            leave composer focus; preserve the local draft
```

The fallback is load-bearing. Traditional terminal protocols may report `Ctrl+Enter` as ordinary Enter. In that case the input gains a newline rather than submitting accidentally, and the visible Send action remains available.

Blank or whitespace-only submission is refused locally; Home submission is also rejected by the existing API when intent is empty. A send attempt does not clear the draft until the API accepts the mutation. A rejected submit or response leaves the draft intact and shows the structured error.

## 7.3 Local slash palette

**Decision T-27 (R2/R5/R6):** `/` opens a fixed TUI command palette over existing views and operations.

Initial vocabulary:

```text
/home
/fleet
/workflows
/back
/refresh
/answer
/retry
/cancel
/events
/graph
/details
/analytics
/quit
```

The palette is context-filtered:

- `/answer` appears only when the selected Work is in `needs_input`;
- `/retry` appears only for a projected retryable state;
- `/cancel` appears only where cancellation is not already terminally impossible;
- `/events`, `/graph`, and `/details` require an open Work;
- `/analytics` opens the existing canned-query utility;
- `/workflows` navigates to the catalog but never changes a running Work.

The parser rule is narrow:

```text
/ as first non-whitespace input  local TUI command
/ elsewhere in text             literal character
```

The palette is a Rust enum and a match, not a shell, command language, plugin interface, or new `sgt` CLI grammar. Palette actions never enter the journal unless they invoke an existing mutation endpoint, in which case the daemon records the real command exactly as it does today.

## 7.4 Workflow mention and selection

**Decision T-28 (R2/R5/R6):** `@` invokes the admitted workflow catalog with context-sensitive meaning.

### On Home

Selecting a workflow sets the existing submission field and renders a chip outside the intent:

```text
Workflow  @repo-to-icm v2

> Decompose this repository's procedural knowledge.
```

The durable intent is the text in the composer, not UI control syntax. Choosing another workflow replaces the selected workflow. Clearing the chip restores default workflow resolution.

### In Workflows

`@` focuses catalog search. Enter opens workflow detail. A `Start Work` action returns to Home with that workflow selected.

### Inside an existing Work

The Work's workflow is already pinned and cannot be rebound. Choosing `@name` while answering inserts a literal reference into the answer text. It is not interpreted by the engine and does not load context automatically.

If the user types an unselected literal `@name`, it remains ordinary text. The TUI does not invent a workflow identity that the catalog did not resolve.

## 7.5 Focus and keyboard help

The footer always describes the valid operations for the current focus. It is not a static list of every key in the application.

Examples:

```text
Home composer     Enter newline · Ctrl+Enter send · @ workflow · / commands · Esc leave
Fleet list        ↑↓ move · Enter open · / commands · ~ attention
Needs input       Enter newline · Ctrl+Enter answer · @ reference · / commands · Esc back
Read-only Work    / commands · Tab views · ~ attention · Esc back
```

A `?` key may open a help overlay listing the same fixed keymap. It does not create a configurable keybinding subsystem.

---

# 8. Home

Home combines the two primary operator acts: state new intent and address existing Work that needs attention.

A wide rendering:

```text
sergeant   Home  Fleet  Workflows                          ? 2   live · seq 128492
┌─ ATTENTION ───────────┬──────────────────────────────────────────────────────────┐
│ NEEDS YOU          2  │ NEW WORK                                                 │
│ ? Retry handling      │ What should Sergeant do?                                 │
│ ? Auth policy         │ > Add retry handling to the settlement worker.           │
│                       │                                                          │
│ IN FLIGHT          3  │ Workflow  @software-change v1                            │
│ ⠹ ICM decomposition  │ Advanced  backend default · profile default · repo current│
│ ⠹ Release validation │                                           [ Send intent ] │
│                       ├──────────────────────────────────────────────────────────┤
│ WAITING            1  │ NEEDS YOU                                                │
│ ○ External approval  │ Retry handling · 10-implement                             │
│                       │ Should the retry budget be 3 attempts?                    │
│ TERMINAL              │                                                          │
│ ✓ Idempotency keys    │ IN FLIGHT                                                │
│ × Failed deployment   │ ⠹ Generate ICM workflows · 40-classify 5/10 · claude     │
└───────────────────────┴──────────────────────────────────────────────────────────┘
Enter newline · Ctrl+Enter send · @ workflow · / commands · ~ attention
```

## 8.1 New Work composer

**Decision T-29 (R2):** Home maps directly to the current submission body.

Default submission:

```text
intent
created_by = "tui"
origin.client = "tui"
origin.cwd = the TUI process's current directory
```

A collapsed Advanced region exposes only fields already accepted by `POST /v1/work`:

```text
workflow
backend
profile
workspace
repositories[]
```

Workflow selection normally comes from `@`; exact-name entry remains available in Advanced for recovery and automation parity. Backend, profile, workspace, and repositories remain exact-name fields. No additional catalog or preflight system is added for them.

The launch summary always shows the resolved request before submission:

```text
intent      Add retry handling to the settlement worker.
workflow    @software-change v1
backend     default
profile     default
workspace   discovered from current cwd
repositories current/default
```

This is request review, not a second planning engine. The daemon still performs authoritative discovery, routing, workflow load, and N3 preflight.

## 8.2 Attention-oriented current Work

Home groups non-terminal Work using the same state grammar as the drawer. Each item shows:

```text
state glyph
intent
current stage coordinate
current question/reason when present
resolved current-stage executor when present
time since submission, labeled honestly as submission age
short id only when needed for disambiguation
```

Intent is the primary identity. A 26-character ULID never leads a Home row.

## 8.3 Bounded terminal Work

**Decision T-30 (R1/R2):** Home shows all non-terminal Work and only a bounded slice of terminal Work.

The current Fleet body has `created_at`, not a terminal timestamp. The UI may therefore label the terminal section **Terminal**, not “Recently completed.” The full history remains in Fleet. No archive or dismissal state is introduced.

---

# 9. Fleet

Fleet is the complete Work browser, not the Home screen and not a process table.

```text
sergeant   Home  Fleet  Workflows                          ? 2   live

FLEET   27 Work          filter: state:any  workflow:any  text:"retry"

  ?  Add retry handling to the settlement worker
     needs input  ·  10-implement 1/2  ·  claude  ·  8m since submit

  ⠹  Generate ICM workflows from this repository
     active       ·  40-classify 5/10   ·  claude  ·  19m since submit

  !  Resolve deployment policy
     blocked      ·  20-review 2/4      ·  claude

  ✓  Add idempotency keys
     completed    ·  20-review 2/2      ·  claude

↑↓ move · Enter open Work · f filter · / commands · ~ attention
```

## 9.1 Width-aware rows

**Decision T-31 (R5):** Fleet rows use Ratatui constraints, wrapping, and explicit ellipsis rather than fixed string padding.

At wide widths, a row may show intent, state, stage, current executor, workspace, submission age, and short ID. At narrow widths it degrades in that order:

```text
intent + state
intent + state + stage
intent + state + stage + executor
additional metadata only when space remains
```

Independent fields never run together. Issue #11's exact failure shape must be impossible by construction.

## 9.2 Client-side filters and grouping

**Decision T-32 (R2):** Fleet filtering uses only fields already returned by `GET /v1/work`.

Supported filters:

```text
text over intent/id/workspace/workflow/backend/stage
state
workflow exact name
backend/executor name where present
terminal vs non-terminal
```

Filtering is local to the loaded Fleet. It is not the future Journal query language. Filter drafts are ephemeral and restored when returning from an opened Work.

## 9.3 Actions

Enter opens the canonical Work. The slash palette exposes valid actions. There is no mouse-only affordance and no single-key destructive mutation.

---

# 10. Workflows

Workflows is a read-only view of admitted repository procedure and a launch affordance for new Work.

## 10.1 Authoritative sources

The catalog has three existing sources with distinct responsibilities:

```text
.sergeant/index.md
    authoritative admitted-workflow list and discovery boundary

.sergeant/workflows/<name>/index.md
    human description, status, tags, use-when language

.sergeant/workflows/<name>/workflow.toml
    executable name, version, stage order, kind, harness, profile
```

The engine's `WorkflowDefinition` remains authoritative for executable validity. The catalog does not reinterpret stage context or invent procedure from prose.

## 10.2 Catalog contents

**Decision T-33 (R2):** The Workflows screen contains:

- every workflow listed as admitted/published by `.sergeant/index.md` and successfully loaded through the existing workflow loader;
- the embedded `software-change` fallback, marked `embedded`, when it is not replaced by an admitted repository-local definition;
- no workflow under `.sergeant/drafts/workflows/`;
- no unindexed directory found by broad scanning;
- no generated or inferred workflow.

A catalog row shows:

```text
@name
version
published or embedded source
short description
stage count
stage executor summary when declared
```

A workflow detail shows its description, tags, source, stage rail, stage kind, explicit harness/profile declarations, and content identity where available. Stage `CONTEXT.md` bodies are not rendered as an authoring editor; the catalog is orientation and selection, not a procedure IDE.

## 10.3 Read-only workflow-catalog API

Under Decision T-17, add one authenticated read-only route:

```text
GET /v1/workflows?cwd=<percent-encoded-path>
```

The route exists because:

- R1 fails: workflow discovery is a settled operator requirement;
- R2 succeeds for the underlying capabilities: workspace discovery, workflow loading, embedded fallback, validation, and publication convention already exist;
- R3/R4 do not supply a repository workflow catalog over HTTP;
- R5 supplies Axum, the existing API/client pattern, TOML parsing, and blocking-I/O boundary;
- R6 covers most wiring, but parsing the existing Markdown catalog/front matter requires a small bounded addition;
- R7 is limited to the minimum catalog projection and exact record parser needed to expose the existing contract.

The endpoint performs no mutation and appends no event. It executes filesystem discovery outside the core lock, following `submit_work`'s current planning precedent.

A representative response shape:

```json
{
  "context": {
    "cwd": "/work/service",
    "workspace": "service",
    "root": "/work/service"
  },
  "workflows": [
    {
      "name": "repo-to-icm",
      "version": "2",
      "status": "published",
      "source": "repository",
      "description": "Convert repository procedure into reviewable ICM drafts.",
      "tags": ["icm", "generator", "measurement"],
      "content_hash": "...",
      "stages": [
        {
          "id": "00-contract",
          "kind": "actor",
          "harness": null,
          "profile": null
        }
      ]
    },
    {
      "name": "software-change",
      "version": "1",
      "status": "embedded",
      "source": "embedded",
      "description": null,
      "tags": [],
      "content_hash": "...",
      "stages": []
    }
  ]
}
```

The exact response is contracted before implementation; the principles above are binding.

`ApiClient` gains a typed workflow-catalog method. `src/tui.rs` imports no workflow or filesystem module. The M6 equal-client structural test is amended deliberately to pin the endpoint-backed addition rather than weakened.

## 10.4 Catalog parsing and failure behavior

**Decision T-34 (R2/R3/R6):** Catalog parsing implements only the repository's current documented record shapes.

It is not a general Markdown or YAML engine. It needs only to:

- read the root catalog's admitted workflow entries;
- follow the declared relative `index.md` path;
- read the exact front-matter fields used by current workflow indexes (`name`, `status`, `version`, `description`, `tags`);
- validate executable content through `WorkflowDefinition::resolve` or the corresponding existing loader;
- distinguish repository and embedded source.

No new parsing dependency is added unless a challenge round demonstrates that a bounded parser cannot correctly handle the committed convention.

Failure rules:

- no `.sergeant/index.md` means no repository catalog; the embedded fallback remains available;
- an index entry that escapes `.sergeant/`, points to a missing file, claims a non-published status, disagrees with `workflow.toml`, or fails workflow loading makes the catalog request fail with a structured `workflow_catalog_invalid` error naming the entry and path;
- a workflow directory present but absent from the root index remains undiscoverable by design;
- catalog failure does not modify Work or prevent exact-name submission through the existing endpoint; it prevents the TUI from pretending discovery succeeded.

## 10.5 Live catalog versus pinned Work

**Decision T-35 (R2):** Workflows shows the live repository catalog for starting future Work. An existing Work always shows the workflow definition pinned in its `workflow.bound` journal event.

Editing `.sergeant/` while Work is running may change the catalog but cannot rewrite the Work's displayed procedure. The UI labels these surfaces distinctly:

```text
Catalog workflow     available for new Work now
Pinned workflow      procedure this Work actually bound
```

## 10.6 Workflow actions

The screen supports:

```text
Enter       inspect workflow
n or action Start Work   return to Home with workflow selected
@           filter/select
/           local commands
Esc         return
```

It does not edit, copy, promote, publish, validate, or generate workflows.

---

# 11. Canonical Work Surface

A Work surface should answer “what is happening here?” before “what are all its fields?”

```text
sergeant / Add retry handling to the settlement worker       ? NEEDS INPUT
@software-change v1 · 10-implement 1/2 · attempt 1 · claude   live

Thread   Workflow   Evidence   Graph   Details

INTENT
Add retry handling to the payment settlement worker.

✓ Surface materialized for service
✓ Entered 10-implement
⠹ Claude execution started

AGENT
I found the existing exponential-backoff policy.

? INPUT REQUEST
Should the retry budget be 3 attempts?

────────────────────────────────────────────────────────────────────────────
ANSWER
> Yes. Use three attempts with exponential backoff and jitter.
>
>                                                                     [ Send ]
────────────────────────────────────────────────────────────────────────────
Enter newline · Ctrl+Enter answer · @ workflow reference · / commands · Esc back
```

## 11.1 Work header

**Decision T-36 (R2):** The header shows, in order:

1. state glyph and explicit state label;
2. intent;
3. pinned workflow name/version;
4. current stage coordinate and attempt;
5. current-stage executor/harness and profile when present;
6. connection truth.

Workspace, repository names, short Work ID, and reservation state may appear on a secondary line when space permits. Full IDs and paths belong in Details.

When Work is in `needs_input`, the current question is visible without scrolling to the event that caused it.

## 11.2 Workflow rail

**Decision T-37 (R2):** The pinned `workflow.stages` array supplies the complete ordinal rail.

The current stage index and the engine's ordered-stage invariant justify these labels:

```text
indices before current    completed
current index             current status and attempt
indices after current     not entered
```

Event history may add attempt counts, question/failure details, and known transitions. The UI does not invent durations where the loaded evidence lacks timestamps.

N3 executor information is shown per stage where pinned data is available; at minimum the current stage's executor comes from `stage.executor`. The rail never reduces a mixed-harness workflow to the Work-level default.

Compact form:

```text
✓ contract  ✓ inventory  ✓ harvest  ⠹ classify  · synthesize  · draft
```

Expanded form:

```text
✓ 00-contract      completed
✓ 10-inventory     completed
✓ 20-harvest       completed
⠹ 40-classify      active · attempt 2 · claude / review-profile
· 50-synthesize
· 60-draft
```

## 11.3 Semantic thread

**Decision T-38 (R2):** Existing event kinds map to bounded, testable thread items.

| Event family | Default rendering |
|---|---|
| `work.submitted` | intent card |
| `workflow.bound` | workflow/workspace/routing system line |
| `surface.materializing/materialized/torn_down` | compact surface line |
| `stage.entered` | stage divider |
| `stage.completed` | completed-stage line with summary/detail |
| `stage.waiting` / `work.waiting` | muted waiting card |
| `conversation.ask` | primary gold actor-authored input-request card |
| `stage.needs_input` / `work.needs_input` | state transition supporting the current request; deduplicated from the ask card |
| `stage.input_received` | human-response line |
| `stage.blocked` / `work.blocked` | amber blocked card |
| `stage.failed` | red failed-stage card |
| `execution.reserved` | compact reservation/launch-preparation line |
| `execution.started` | executor-started line |
| `execution.stopped` | executor-ended line |
| `execution.abandoned` | explicit abandoned-reservation warning |
| `execution.reconciled` | restart/reconciliation line |
| `conversation.user` | user/engine prompt line where useful |
| `conversation.assistant.completed` | agent message |
| `tool.requested` / `tool.completed` | paired, collapsible tool item |
| `usage.updated` | compact usage line, collapsed by default |
| `work.completed` | green terminal outcome line |
| `work.failed` / `work.canceled` | terminal outcome line |
| unknown kind | generic evidence line with kind and sequence |

The semantic thread never says “thinking,” “making progress,” or “working on files” unless a recorded event supports that claim. An active spinner means lifecycle state `active`, not verified native process activity.

The default Work window loads the newest 200 matching events. When exactly 200 are returned, the thread offers `Load older`, which increases the existing endpoint's `limit` in bounded steps. The UI labels a partial window honestly until fewer rows than the limit are returned. This is not global Journal search.

## 11.4 Work views

**Decision T-39 (R2/R5):** The canonical Work surface has five views:

```text
Thread     human-readable recent trajectory and current request
Workflow   complete pinned stage rail and known attempt history
Evidence   raw event rows, sequence, timestamp, kind, source, payload
Graph      existing one-Work graph neighborhood as a navigable tree/list
Details    IDs, route, reservation, execution, surface, repositories, teardown
```

Graph remains terminal-native:

```text
Work: Add retry handling
├─ follows → Workflow: software-change v1
├─ targets → Repository: service
├─ stage → 10-implement #1
├─ executed-by → Execution: 01KZ...
│  ├─ uses → Backend: claude
│  └─ bound-to → Native session: ...
└─ message → agent: Should the retry budget be 3 attempts?
```

It does not draw a browser-like node canvas.

## 11.5 Composer and action matrix

**Decision T-40 (R2):** The TUI advertises only actions supported by existing semantics.

| Work state | Ordinary text | Actions |
|---|---|---|
| `pending` | disabled | refresh, cancel, inspect |
| `active` | disabled | refresh, cancel, inspect |
| `waiting` | disabled | retry, cancel, inspect |
| `needs_input` | answer | respond, cancel, inspect |
| `blocked` | disabled | retry, cancel, inspect |
| `failed` | disabled | retry, cancel, inspect |
| `completed` | disabled | inspect; navigate Home for new Work |
| `canceled` | disabled | inspect; navigate Home for new Work |

The API remains authoritative. A race may make an advertised action invalid by the time it arrives; the structured conflict is shown and state is refreshed.

Cancel requires explicit confirmation. Retry requires selecting the action and confirming the target Work. A response requires the deliberate send behavior in §7.2.

## 11.6 Workflow references inside an answer

Selecting `@workflow` inside a needs-input composer inserts a literal reference. The TUI may show a small preview from the catalog before insertion, but the answer sent to the backend is ordinary text. The current Work's pinned workflow and stage remain unchanged.

---

# 12. Live Connection and Reconnect

The current TUI tells the truth when its SSE tail closes, but issue #16 records that recovery is manual. A long-lived operator surface that remains stale until the user knows to press `r` is not complete enough for this usability program.

**Decision T-41 (R2/R6):** T-series closes issue #16 with a small explicit connection state machine.

```text
Attached
   │ stream ends / chunk error / refill failure
   ▼
Reconnecting(attempt, next_delay)
   │ successful stream open
   ├──────────────► Refresh authoritative state ► Attached
   │ 401/403
   └──────────────► AuthenticationFailed
```

Rules:

1. Reconnect uses bounded exponential backoff with a capped interval; it does not create a high-frequency retry loop.
2. `r` requests an immediate refresh/reconnect attempt without resetting historical truth.
3. A stream open is not enough. The TUI rereads current system/Fleet/selected Work state before labeling the connection attached, because the SSE gap may contain changes.
4. An authentication failure stops automatic retry and states that the daemon identity/token changed. The TUI does not retry forever with a known-invalid token.
5. Transport failures continue retrying at the capped interval while the screen remains usable and visibly stale.
6. A command result never overwrites the connection state.
7. Active spinners stop when the tail is not attached; stale data must not be animated as live.

This changes only client liveness behavior. It adds no journal event, daemon state, token rotation, or reconnect protocol.

---

# 13. Visual Language

## 13.1 State grammar

**Decision T-42 (R5):** Every state is communicated through label, glyph, and standard terminal color. Color is never the sole carrier.

| State | Glyph | Treatment |
|---|---:|---|
| `pending` | `·` | muted cyan |
| `active` | spinner frame | cyan/blue |
| `waiting` | `○` | muted gray/blue |
| `needs_input` | `?` | gold/yellow, bold |
| `blocked` | `!` | amber/orange where available, else yellow |
| `failed` | `×` | red, bold |
| `completed` | `✓` | green |
| `canceled` | `—` | dim gray |
| reconnecting | `↻` | yellow/cyan with explicit label |
| detached/auth failed | `!` | red with explicit label |

Terminal palettes vary. The textual state is always present in important contexts.

## 13.2 Motion

**Decision T-43 (R2/R6):** Only currently active Work receives animation, and only while the SSE tail is attached.

The spinner uses a small in-code frame array on the existing event-loop tick:

```text
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

No animation framework, effects crate, busy render loop, pulsing attention bell, or transition choreography is added. Needs input, blocked, failed, and completed are static because their importance comes from state, not motion.

A Unicode fallback such as `*` or `>` is used if the selected symbol width is not one cell in the test environment.

## 13.3 Progress

**Decision T-44 (R1/R2):** Workflow progress is ordinal, never a percentage gauge.

`5/10` means the fifth ordered checkpoint, not 50% of elapsed time, token cost, or effort. The UI uses a stage rail and coordinate. Ratatui gauges are not used where the domain has no honest ratio.

## 13.4 Focus and affordance

**Decision T-45 (R5):** Focus is visible through border/title style, cursor placement, and footer language.

The TUI never relies on invisible modal state. When printable keys will edit a field, a cursor is visible. When `/` will navigate, the footer says so. When text cannot be sent to the current Work, the composer states that fact rather than silently swallowing input.

## 13.5 Rich but terminal-native

The implementation may use Ratatui's current widgets for:

```text
Tabs
Table/List
Paragraph/Wrap
Scrollbar
Gauge only for honest binary/ratio states
Blocks and titles
styled spans
popups and overlays
```

It does not imitate browser pixels, graphical file icons, rounded CSS pills, or a spatial graph canvas. Box drawing, text hierarchy, whitespace, color, and restrained symbols are the visual system.

---

# 14. TUI State and Data Flow

## 14.1 Local state is interaction state

**Decision T-46 (R2):** `App` owns only ephemeral client interaction state.

Representative fields:

```text
mode and navigation stack
focus
attention drawer open/closed
Fleet rows and filters
workflow catalog and selection
selected Work and loaded event limit
current Work view
scroll offsets
composer drafts and cursors
slash/@ popup state
connection/reconnect state
last seen journal seq
status/error message
cancel/retry confirmation
spinner frame
```

None is authoritative Work state. Restarting the TUI loses drafts, filters, and cursor positions but loses no Work fact.

## 14.2 Authoritative reads

The initial Home paint fetches:

```text
/v1/system
/v1/work
/v1/workflows?cwd=<launch cwd>
```

Opening Work fetches:

```text
/v1/work/{id}
/v1/events?work_id={id}&limit={current_limit}
```

Graph and analytics are lazy: they load only when selected.

## 14.3 Refresh discipline

**Decision T-47 (R2):** SSE remains invalidation, not a client-side authoritative reducer.

An observed event may:

- advance `last_seq`;
- mark Fleet or selected Work dirty;
- trigger a debounced authoritative reread;
- update connection activity.

It may not directly decide Work state, stage, workflow, executor, surface, or legal actions. The raw event itself may appear in the selected Work's Evidence window only after the authoritative history request includes it.

Refreshes are coalesced during event bursts so the TUI does not issue one full set of reads per event. The current P1 graph and read-path measurements become budgets: the UI must not replace an idle SSE client with a polling storm.

## 14.4 Minimal multiline composer machinery

**Decision T-48 (R7):** Implement one small TUI-local multiline composer rather than adding a generalized editor dependency.

Lower rungs:

- R1 fails: multiline deliberate input is a settled usability requirement.
- R2 fails: the existing `String` buffer supports only append/backspace and Enter-to-submit.
- R3 fails: Rust's standard library provides no terminal editor.
- R4 fails: terminal protocols provide events, not editing behavior.
- R5 fails: Ratatui/Crossterm are installed but do not supply the required text-area state; adding another dependency is not lower than a bounded local implementation.
- R6 fails: cursor-aware multiline input is not one line.

The R7 addition is explicitly bounded to:

```text
insert character
insert newline
backspace and delete
left/right/up/down
home/end within line
cursor rendering
wrapped display
Ctrl+Enter submission detection
```

The current key-reader channel carries only `KeyCode` and therefore discards modifier information. T-series changes that local transport to preserve the `KeyEvent` (or an equivalently small key-plus-modifiers value). It does not require an enhanced-keyboard protocol: where the terminal cannot distinguish `Ctrl+Enter`, the event is treated as ordinary Enter and the visible Send action remains the portable confirmation path.

It excludes selection, mouse editing, undo trees, history, syntax highlighting, completion protocols beyond the fixed `/` and `@` popups, and configurable bindings. Tests drive every editing operation as pure state transitions.

A challenge round may overturn this choice in favor of a narrowly vetted text-area dependency only by showing that the local editor would be larger or less correct after the same required behaviors are counted.

## 14.5 Pure presentation shapes

The implementation may introduce TUI-local types such as:

```text
Mode
Focus
AttentionGroup
WorkBrief
WorkflowCatalogEntry
ThreadItem
WorkView
Composer
PaletteItem
ConnectionState
```

They are projections over JSON/API values and local interaction state. They are not domain types and are never serialized as durable state.

## 14.6 Physical code layout

**Decision T-49 (R1):** This proposal defines logical seams, not a mandatory module tree.

The current `src/tui.rs` is already large. Splitting rendering, interaction, composer, and view projection into focused modules is allowed when implementation evidence shows one file no longer remains reviewable. Predeclaring a miniature UI framework before that evidence is not allowed.

---
# 15. Web Dashboard Disposition

The browser surface proved the equal-client architecture, but developing two interaction models before either is settled creates duplicate presentation logic and expands the unexecuted-JavaScript gap in issue #21.

**Decision T-50 (R1/R2):** Disable the dashboard without deleting its source.

Implementation disposition:

1. `crate::web::routes(...)` is not merged into the daemon router.
2. `/ui` and its asset routes therefore receive the listener's normal structured 404.
3. `src/web.rs` and embedded CSS/JavaScript remain in the repository as a dormant future stub.
4. `sgt web` does not auto-spawn the daemon. It reports that the dashboard is disabled and directs the operator to bare `sgt`.
5. `sgt web --json` returns a stable disabled result such as:

```json
{
  "available": false,
  "surface": "web",
  "reason": "disabled while the terminal interaction model is being proven"
}
```

6. Human form exits nonzero because the requested surface is unavailable; it does not print a tokenized URL.
7. Existing web render tests may remain as unit tests for the retained stub where cheap, but no new dashboard UX or browser test tier is built in T-series.

Disabling the route is not a claim that HTML is a bad interface. It is a sequencing decision: prove one semantic interaction model, then build a browser client from settled concepts rather than maintain visual parity with a moving target.

## 15.1 Reactivation prerequisites

**Decision T-51 (R1):** Issues #15 and #21 remain open and dormant.

Before the browser surface returns:

- token-in-URL handling is reconsidered under #15's trigger and security boundary;
- dashboard JavaScript receives an executed browser test tier per #21;
- semantic parity is defined against the settled terminal concepts—Home, Fleet, admitted Workflows, Attention, canonical Work, input requests, and evidence—not against Ratatui geometry;
- the separate Journal proposal decides whether and how browser exploration differs from terminal exploration.

No web reactivation date is promised here.

---

# 16. Open-Issue Boundaries

## 16.1 Issue #11 — owned and closed

**Decision T-52 (R2/R5):** T-series owns issue #11 and closes it through responsive layout plus a falsifiable test.

The fix is not “increase padding.” It is to stop encoding independent fields as unbounded padded strings. Restoring the old renderer must fail a test that verifies state, stage, and executor/backend remain visually separable at realistic widths and with long values.

## 16.2 Issue #16 — owned and closed

Issue #16 is implemented through §12. It is a named foundation item, not hidden polish. Its commit carries `Fixes #16` only when the reconnect state machine, refresh-before-resume behavior, auth-failure stop, and regression tests all land.

## 16.3 Issue #26 — separate correctness work

**Decision T-53 (R1):** The pre-loop PTY hangup, loaded shutdown flake, and adapter/profile defects remain separate work rather than being papered over in presentation.

The TUI redesign must preserve all existing post-initialization terminal safety and must not widen the startup window. It does not claim to close #26 without its specific early-install or first-interaction fix and reproduction test.

## 16.4 Issue #45 — separate flake investigation

Issue #45 records a loaded m6 failure shape—dead daemon with `runtime.json` left behind—that may overlap the startup/signal window class. T-series changes m6 coverage and therefore must run its scenario and PTY tests under repeated load, but it does not close #45 unless the underlying cause is independently established and fixed.

## 16.5 Issues #46 and #47 — separate adapter/runtime defects

Under Decision T-53, T-series does not reinterpret adapter failure as presentation state.

Issue #46 records a measured fail-closed violation in which an envelope-less Claude turn can leave Work visibly `active` after the native process is gone. The TUI must render the authoritative state and available evidence honestly; it must not infer failure from silence, elapsed time, or absent process data the client does not own. T-series does not close #46 unless the adapter's signal path is independently fixed and pinned.

Issue #47 moves Claude permission mode into profile-owned launch configuration. Home may continue to accept the existing profile name and Work may display the pinned/effective profile fields the API supplies. The TUI does not invent permission-mode controls or claim an effective mode the current public view does not report. #47 remains adapter/profile work.

## 16.6 Dashboard issues

Issues #15 and #21 remain visible as §15.1 reactivation gates. Disabling the dashboard does not close them as “not planned.”

## 16.7 Journal gap

The absence of a rich journal/DuckDB operator surface is recorded in this proposal's boundary, not silently treated as solved by Work-local Evidence. Its separate proposal should cite this decision and the current DuckDB/event contracts rather than restate T-series.

---

# 17. Testing and Validation

## 17.1 Test philosophy

**Decision T-54 (R2):** Retain M6's proven Ratatui `TestBackend` strategy: assert semantics and important geometry, not entire pixel snapshots.

Full-frame golden snapshots were already rejected because they break on every harmless layout adjustment and mostly test box drawing. T-series adds stronger geometry assertions where geometry is the contract—especially #11, composer separation, drawer collapse, and narrow-terminal safety.

Every fix and every new behavior carries a pinning test that fails if the behavior is reverted, per LESSONS L7 and the S-series independent-prober discipline.

## 17.2 Pure view-model tests

Tests cover:

- Fleet row projection and attention grouping;
- intent-first ordering and short-ID rules;
- terminal slice bounding;
- state glyph/label/color mapping;
- stage rail derivation from ordered workflow and current index;
- N3 current-stage executor projection;
- reservation and abandoned-execution rendering;
- event-to-thread mapping, including `conversation.ask` deduplication with `stage.needs_input` and `work.needs_input`;
- unknown-event fallback to Evidence;
- Work action availability by state;
- catalog row projection;
- local filter behavior;
- navigation-stack restoration.

## 17.3 Composer and grammar tests

The composer is driven as pure state:

- character insertion at cursor;
- newline on Enter;
- no submission on ordinary Enter;
- submission on `Ctrl+Enter` when the modifier is reported;
- focused Send fallback;
- draft preservation on `Esc` and API rejection;
- clearing only after accepted submission;
- backspace/delete and cursor movement across lines;
- wrapped rendering and cursor placement;
- blank-input refusal;
- slash command recognized only as first non-whitespace input;
- slash inside prose remains literal;
- palette context filtering;
- every palette mutation maps to an existing endpoint action;
- Home `@` selection sets the workflow field and does not contaminate intent;
- Work `@` selection inserts literal text and does not rebind procedure.

A mutation probe that makes ordinary Enter submit must kill a test.

## 17.4 Workflow-catalog API tests

The read-only endpoint receives dedicated contract tests:

1. repository with no `.sergeant/index.md` returns the embedded fallback only;
2. root index with one published workflow returns its exact name, version, description, tags, source, content hash, stage order, kind, harness, and profile;
3. `.sergeant/drafts/workflows/` is never returned;
4. an unindexed admitted directory is not discovered by scan;
5. path traversal in catalog links is rejected;
6. missing index target is a structured catalog error;
7. `index.md`/`workflow.toml` name, version, or status disagreement fails closed;
8. malformed `workflow.toml` reuses the existing loader's exact failure rather than inventing a second validator;
9. local `software-change` correctly overrides the embedded fallback;
10. cwd/workspace discovery matches submission planning;
11. the route performs no journal append and no Work mutation;
12. `src/tui.rs` remains API-only under the structural source scan;
13. `ApiViews` is either narrowed with the disabled web surface or updated only through an endpoint-backed method whose route is pinned by the structural test.

The catalog parser receives fixture and mutation tests for every supported record field and failure branch.

## 17.5 Action tests through a live daemon

Against a deterministic fake backend:

- Home submission creates exactly one Work and shows the daemon-projected result;
- workflow selected through `@` arrives in the existing request field;
- actor-authored `conversation.ask` produces `needs_input` and the gold card;
- a multiline answer resumes the same execution through the existing input endpoint;
- retry works only from retryable states;
- cancel confirmation prevents accidental mutation;
- a state race returns a structured conflict, preserves screen integrity, and triggers refresh;
- completed/canceled Work cannot be resurrected through the composer.

## 17.6 Reconnect tests

Issue #16 is pinned with both pure-state and live tests:

- stream end transitions `Attached → Reconnecting`;
- backoff grows and caps;
- no busy reconnect loop;
- successful reconnect triggers state refresh before `Attached` is rendered;
- events committed during the gap appear after refresh;
- 401/403 transitions to `AuthenticationFailed` and stops automatic attempts;
- manual `r` requests an immediate attempt;
- command status cannot overwrite connection truth;
- active spinner does not run while detached;
- event decoder chunk/comment/error cases from the S-series coverage work remain green.

## 17.7 Responsive layout tests

**Decision T-55 (R5; governed by L7):** Render every major surface through `TestBackend` at:

```text
80×24
120×36
180×48
```

Fixtures include:

```text
Home empty
Home with attention and long intent
Fleet with every state and long stage/executor names
Work active
Work actor-authored needs_input
Work blocked
Work failed
Work completed
Workflows catalog and workflow detail
slash palette
@ workflow chooser
connection reconnecting/auth failed
web-disabled CLI output (outside Ratatui)
```

Assertions include:

- no panic at zero/tiny child areas;
- issue #11 collision cannot occur;
- state and current question remain visible at all supported sizes;
- composer never overlaps thread or footer;
- focused cursor lies within the composer area;
- drawer and contextual views collapse at declared widths;
- long values wrap or truncate inside their assigned regions;
- `? N` remains visible when the drawer is closed;
- workflow chips and Send action do not disappear under ordinary 80-column composition;
- Unicode glyphs occupy one cell or fall back.

The #11 test must fail when the old fixed-padding renderer is restored.

## 17.8 PTY, shutdown, and hygiene tests

Existing terminal restoration, signal, dead-pty, reader-shutdown, and idle-CPU tests remain binding. T-series adds:

- repeated open/close of drawer and overlays without raw-mode corruption;
- quitting while composer, palette, workflow chooser, reconnect timer, and cancel confirmation are active;
- a loaded repeated-run sweep informed by #45;
- daemon/TUI process leak sweep using the repository's non-self-matching `pgrep` rule;
- no high-frequency animation or reconnect CPU regression.

Issue #26 remains separately red if its specific pre-loop reproduction is run; T-series does not weaken the test to claim otherwise.

## 17.9 Web-disabled tests

Tests pin:

- `/ui` returns structured 404;
- `sgt web` does not auto-spawn the daemon;
- human output names the disabled surface and bare `sgt` alternative;
- `--json` output is stable and reports `available: false`;
- no tokenized URL is printed;
- retained web source and assets still compile as a dormant stub;
- #15/#21 references remain in docs/backlog rather than disappearing.

## 17.10 End-to-end story

A deterministic walkthrough exercises the product story:

```text
launch bare sgt in a repository
open Workflows and inspect admitted procedure
select @software-change and return Home
compose multiline intent
Tab to Send and submit
watch Work appear in Attention/Fleet
open canonical Work
observe reservation, execution, and stage transition
fake actor emits conversation.ask
answer with multiline Ctrl+Enter/send fallback
Work resumes and completes
inspect Workflow, Evidence, Graph, Details
return to exact Fleet position
kill the SSE stream and verify reconnect truth/recovery
run sgt web and receive the disabled response
quit with terminal restored
```

Every claim is re-read from the API or journal-backed endpoint. The walkthrough is not graded on its narration.

## 17.11 Gates

Each implementation milestone closes with:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
scripts/demo.sh
scripts/gate.sh "<milestone outcome>"
pgrep -f "debug/sgt [-]-data-dir"
```

Coverage remains above the repository's established CI floor. New code follows the full multi-axis loop because code is code (R-S0-12).

---

# 18. Program Shape

The proposal is one program with four bounded outcomes. This is not issue/PR decomposition; contracts are written only after the proposal is adjudicated.

## 18.1 T0 — proposal adjudication and audit freeze

Outcome:

- challenge this proposal on spec-fidelity, invariants, simplicity, and test-honesty;
- confirm `a5fb875` as audit basis or update it explicitly if main advances before T0 begins;
- adjudicate the sole read-only API addition;
- confirm the Journal boundary and web disablement;
- record accepted amendments and issue ownership;
- write the T1 contract only after rulings exist.

No product code changes.

## 18.2 T1 — surface foundation

Outcome:

- disable the web route and `sgt web` handoff;
- add the workflow-catalog endpoint and contract tests;
- establish responsive application shell, top navigation, focus, overlays, Attention drawer, composer state, and local palette infrastructure;
- close issue #11;
- close issue #16;
- preserve all terminal safety and client-equality tests.

This milestone carries the highest cross-cutting risk and receives the full gauntlet before feature screens build on it.

## 18.3 T2 — intent, Fleet, and admitted Workflows

Outcome:

- Home intent composer with deliberate multiline submission;
- Advanced request fields;
- `@workflow` selection and chips;
- Home attention summary;
- complete responsive Fleet and filters;
- Workflows catalog and detail;
- `/home`, `/fleet`, `/workflows`, `/refresh`, `/back`, `/quit` palette paths;
- deterministic launch story from workflow discovery through Work creation.

No canonical Work thread beyond the existing detail remains required at this checkpoint.

## 18.4 T3 — canonical Work and close-out

Outcome:

- canonical Work header and navigation stack;
- Workflow rail with N3 executor details;
- semantic thread including actor asks and reservations;
- multiline response composer;
- state-aware retry/cancel/inspect actions;
- Evidence, Graph, and Details views;
- palette actions for Work and analytics;
- final responsive/hygiene/load validation;
- README and real Ratatui screenshots replace the P0 images;
- ledger entry and lessons update;
- explicit handoff to the separate Journal proposal.

## 18.5 Journal proposal relationship

The Journal proposal may begin after T0 or in parallel if it does not modify T-series-owned files. T-series does not depend on it. When Journal later ships, it may add a top-level destination through the established navigation/palette patterns, but it must not force T-series to invent query contracts in advance.

## 18.6 Proposal as a timestamped model

Like the repository's other proposals, this document records the best current model. Milestone contracts may narrow or amend it when measurements prove a claim wrong. Amendments are registered and reviewed; implementation does not silently drift.

---

# 19. Acceptance Contract

T-series is complete when all of the following are true:

1. Bare `sgt` opens Home, not the literal P0 Fleet table.
2. Top navigation is exactly `Home / Fleet / Workflows`; no placeholder Journal or System tab appears.
3. Home accepts multiline intent and submits only through deliberate `Ctrl+Enter` or the focused Send action.
4. Home exposes the existing workflow/backend/profile/workspace/repository request fields without becoming a second planner.
5. `/` opens a fixed local palette and is never advertised as a new CLI language.
6. `@` selects an admitted workflow on Home and inserts only a textual reference inside an existing Work.
7. Workflow discovery comes from the endpoint-backed `.sergeant/index.md` publication boundary; drafts and unindexed directories are absent.
8. The embedded `software-change` fallback appears honestly as embedded.
9. Catalog failure is named and fail-closed; exact-name Work submission remains the daemon's existing behavior.
10. Home and the Attention drawer distinguish needs-input, trouble, in-flight, waiting, and terminal Work without new durable state.
11. The full Fleet remains reachable and issue #11 is closed by responsive layout, not padding.
12. Every Work entry opens one canonical Work surface and `Esc` restores exact prior navigation state.
13. The Work header shows intent, lifecycle state, pinned workflow, current stage, attempt, and current executor before secondary IDs and paths.
14. Actor-authored `conversation.ask` appears as the primary gold input request.
15. The semantic thread is derived only from journaled events and always offers raw Evidence.
16. Workflow progress is ordinal, never a false percentage.
17. Ordinary text is accepted only on Home and Work in `needs_input`.
18. Submit, respond, retry, and cancel retain their existing API and engine semantics.
19. Active Work is not interrupted by typing and receives no invented guidance channel.
20. Completed and canceled Work remain terminal.
21. Issue #16 is closed: reconnect uses capped backoff, refreshes before declaring live, and stops on authentication failure.
22. Connection state cannot be overwritten by command status, and stale Work is never animated as live.
23. Graph renders only current proven relationships; files, artifacts, commits, and findings are not invented.
24. Existing canned analytics remain reachable but are not represented as the future Journal product.
25. No global journal query route, arbitrary SQL, full-text search, saved view, or Journal tab is added.
26. No CPU, memory, disk, OpenTelemetry query, mouse, graph canvas, or web redesign is added.
27. `/ui` is unmounted and `sgt web` reports the browser surface disabled without printing a token URL.
28. Issues #15 and #21 remain visible reactivation prerequisites.
29. Issues #26, #45, #46, and #47 remain separate unless their actual causes are independently fixed and pinned.
30. The TUI never converts silence, elapsed time, or guessed process liveness into a Work transition; #46 remains visible rather than being papered over in presentation.
31. `src/tui.rs` still reaches state and workflow catalog only through `ApiClient`.
32. All major screens pass semantic and geometry tests at `80×24`, `120×36`, and `180×48`.
33. Composer, palette, `@`, reconnect, catalog, and mutation behaviors are mutation-probed or otherwise demonstrably falsifiable.
34. Existing signal, pty, shutdown, idle-CPU, and process-hygiene guarantees remain green.
35. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `scripts/demo.sh`, and the shipping gate pass.
36. The final ledger entry records mission outcome, environmental behavior, every rung decision, every adjudication amendment, and every deferred finding.

---

# 20. Ponytail Decision Register

The rung is the lowest viable resolution, not the most impressive implementation.

| Decision | Rung | Resolution |
|---|---:|---|
| T-01 | R1 | Pin audit revision `a5fb875`; do not design against moving main |
| T-02 | R1/R2 | Admit only local `/`, local `@`, and one endpoint-backed catalog beyond the strict draft |
| T-03 | R2 | Organize around existing durable Work |
| T-04 | R2 | Reuse current Work submission for Home intent |
| T-05 | R2/R5 | Reuse familiar harness layout with installed Ratatui |
| T-06 | R2 | Preserve existing Work/stage/executor/execution coordinates |
| T-07 | R2 | Preserve equal clients; no TUI filesystem or runtime shortcut |
| T-08 | R2/R5 | Reorder existing facts through progressive disclosure |
| T-09 | R2/R5 | Require deliberate confirmation for durable writes |
| T-10 | R2 | Apply the repository's existing Ponytail contract |
| T-11 | R2 | Reuse current v1 routes; one catalog exception only |
| T-12 | R2 | Render merged N3 ask/reservation/executor facts |
| T-13 | R1/R2 | Use only submit/respond/retry/cancel mutations already present |
| T-14 | R2 | Semantic thread is a presentation fold over existing events |
| T-15 | R1/R2 | Render current graph and canned analytics exactly; do not generalize |
| T-16 | R2 | Reuse `.sergeant/index.md`, workflow indexes, and `workflow.toml` as catalog authority |
| T-17 | R2/R6/R7 | Add one minimum read-only workflow-catalog endpoint; lower-rung reasoning in §10.3 |
| T-18 | R1 | Keep Doctor CLI-only; no System dashboard |
| T-19 | R1 | Give global Journal/DuckDB exploration its own proposal |
| T-20 | R1 | Explicitly exclude adjacent feature work |
| T-21 | R2/R5 | Use `Home / Fleet / Workflows` top navigation |
| T-22 | R2/R5 | Derive Attention drawer from existing Fleet state |
| T-23 | R2 | One canonical Work surface from every entry point |
| T-24 | R5 | Use Ratatui's existing layout primitives for responsive compositions |
| T-25 | R2/R5 | One persistent state-aware composer |
| T-26 | R2/R5 | Enter newline; Ctrl+Enter/Send deliberate submit |
| T-27 | R2/R5/R6 | Fixed local slash palette over existing actions |
| T-28 | R2/R5/R6 | Local `@` selection/reference over admitted catalog |
| T-29 | R2 | Map Home exactly to current submit body |
| T-30 | R1/R2 | Bound terminal Work on Home; Fleet remains complete |
| T-31 | R5 | Replace padded strings with responsive row layout |
| T-32 | R2 | Filter only fields already present in Fleet |
| T-33 | R2 | Show admitted and embedded workflows only |
| T-34 | R2/R3/R6 | Parse only committed catalog/front-matter shapes; no generalized parser |
| T-35 | R2 | Separate live catalog from journal-pinned Work workflow |
| T-36 | R2 | Intent/state/workflow/stage/executor lead Work header |
| T-37 | R2 | Derive workflow rail from pinned ordered stages/current index |
| T-38 | R2 | Map known events to semantic thread items with raw fallback |
| T-39 | R2/R5 | Thread/Workflow/Evidence/Graph/Details over current data |
| T-40 | R2 | Advertise only current legal action families |
| T-41 | R2/R6 | Close #16 with a small explicit reconnect state machine |
| T-42 | R5 | Text + glyph + standard color state grammar |
| T-43 | R2/R6 | Animate only attached active Work using a local frame array |
| T-44 | R1/R2 | Ordinal workflow progress, no percentage gauge |
| T-45 | R5 | Focus visible through installed Ratatui styling/cursor primitives |
| T-46 | R2 | App owns ephemeral interaction state only |
| T-47 | R2 | SSE invalidates; API reread remains authoritative |
| T-48 | R7 | Minimum local multiline composer; failed lower rungs named in §14.4 |
| T-49 | R1 | Split code only when implementation size proves the need |
| T-50 | R1/R2 | Unmount web, retain source stub, report disabled |
| T-51 | R1 | Keep #15/#21 dormant until explicit reactivation gates fire |
| T-52 | R2/R5 | Close #11 through layout plus falsifiable geometry test |
| T-53 | R1 | Keep #26/#45/#46/#47 separate; never invent a presentation-layer transition |
| T-54 | R2 | Reuse M6 TestBackend semantic testing and S-series falsifiability discipline |
| T-55 | R5 | Test three representative terminal geometries with installed TestBackend |

Any implementation decision not represented here is logged in the milestone report. Any new R7 names failed R1–R6 paths before it is admitted.

---

# 21. Source-to-Decision Map

| Source | What it constrains here |
|---|---|
| [`CLAUDE.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/CLAUDE.md) | journal truth, one owner, equal clients, tests, Ponytail, code-is-code |
| [`src/tui.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/tui.rs) | current P0 screens, key reader, SSE invalidation, terminal safety, manual reconnect |
| [`src/api.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/api.rs) | existing routes, N3 Work view, event vocabulary, structured errors, client boundary |
| [`src/domain/workflow.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/domain/workflow.rs) | ordered pinned workflows, tagged stages, per-stage executor metadata, embedded fallback |
| [`.sergeant/index.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/.sergeant/index.md) | admitted workflow discovery surface |
| [`docs/icm/convention.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/docs/icm/convention.md) | catalog authority, publication boundary, drafts excluded |
| [`docs/gauntlet/contracts/N3.md`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/docs/gauntlet/contracts/N3.md) | actor-authored ask resumes through existing respond; N3 is current behavior |
| [`src/runtime/analytics.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/runtime/analytics.rs) | current canned analytics and the evidence that global Journal deserves its own contract |
| [`src/runtime/graph.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/src/runtime/graph.rs) | proven graph node/edge vocabulary and absent file/artifact/commit facts |
| [`tests/m6_surfaces.rs`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/tests/m6_surfaces.rs) | semantic TestBackend approach and equal-client structural enforcement |
| [Issue #11](https://github.com/miztertea/sergeant-rs/issues/11) | responsive Fleet layout and collision regression |
| [Issue #16](https://github.com/miztertea/sergeant-rs/issues/16) | reconnect/backoff/refresh/auth-failure contract |
| [Issue #26](https://github.com/miztertea/sergeant-rs/issues/26) | explicit startup-hangup exclusion |
| [Issue #45](https://github.com/miztertea/sergeant-rs/issues/45) | loaded m6 flake budget and separate investigation |
| [Issue #46](https://github.com/miztertea/sergeant-rs/issues/46) | adapter fail-closed defect; forbids the TUI from deriving state from silence |
| [Issue #47](https://github.com/miztertea/sergeant-rs/issues/47) | profile-owned permission-mode work; no speculative TUI control |
| [Issues #15](https://github.com/miztertea/sergeant-rs/issues/15) and [#21](https://github.com/miztertea/sergeant-rs/issues/21) | dormant web reactivation gates |
| [Work-Centered Intelligence](https://app.notion.com/p/3ac27ada618f81728a73fbd7ac90c61c) | Work remains durable center; prompt/thread is portal |
| [WorkPacket](https://app.notion.com/p/39a27ada618f818cba42f5efe8ffe1f0) | interface surfaces state and human decisions but does not own Work state |
| [Work Filesystem](https://app.notion.com/p/3ac27ada618f819d8196fa78ab420224) | progressive disclosure, one responsibility per surface, visible stage/resources/evidence |
| [Shared-Engine Human-Agent Workbench](https://app.notion.com/p/39a27ada618f81999694e0fbb019ca50) | human and agent faces operate over one engine/model |
| [Ecological Interface Design](https://app.notion.com/p/3ac27ada618f81909dd5d48e1f9b9912) | reveal work-domain constraints without unnecessary cognitive escalation |
| [Ponytail Minimality Ladder](https://app.notion.com/p/39a27ada618f8100babadb321a70de9b) | strict solution ordering and explicit R7 burden |
| [Anthropic context-engineering source record](https://app.notion.com/p/3af27ada618f8188806de090bd721054) | progressive disclosure and distinct context surfaces |
| [Garrison Business User Workspace](https://app.notion.com/p/3ab27ada618f812db874fbebc0eaf9d8) | warning against building a visual product before the operating model is proven |

---

# 22. Final Position

Sergeant is not a chat client with a workflow engine attached. It is a durable work engine whose current human interface happens to be too literal.

The correct terminal product begins where the architecture begins:

```text
human intent
    ↓
admitted procedure
    ↓
durable Work
    ↓
actor execution and evidence
    ↓
human decision when requested
    ↓
resumption, retry, cancellation, or terminal outcome
```

The TUI should make that loop feel as immediate as a modern agent harness while remaining more honest than one:

- intent is primary;
- Work survives the screen;
- workflow is explicit and pinned;
- actor asks are first-class;
- human answers are deliberate;
- state, stage, executor, and process remain separate;
- every friendly summary has raw evidence behind it;
- current capability is never embellished to complete a mockup;
- repository procedure is discoverable through its existing catalog;
- historical Journal exploration is important enough to receive a real proposal rather than a fake search box;
- the browser waits until the terminal interaction model is proven.

The target is not “a prettier Fleet.”

> **The target is the human command surface for setting intent and operating durable procedural Work—rich enough to understand at a glance, restrained enough to remain true.**
