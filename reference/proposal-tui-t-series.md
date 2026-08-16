---
type: proposal
title: "Sergeant-rs T-Series: Work-Centered Operator Cockpit"
description: >-
  Revised T-Series proposal to replace Sergeant-rs's deliberately minimal
  Ratatui Fleet/Detail proof with a modern, keyboard-first operator cockpit
  for setting intent, operating durable Work, discovering admitted workflows,
  managing the estate, and reading health and evidence. This revision
  supersedes the 2026-08-11 T-Series proposal, consumes the shipped North Star
  MVP and WATCH surfaces, accounts for PR #111's now-merged integration
  branch, and constrains implementation through the Taste design audit and
  the repository's Ponytail ladder.
status: proposed
resource: sergeant-rs
tags:
  - sergeant-rs
  - tui
  - ratatui
  - operator-cockpit
  - work-centered
  - estate
  - usability
  - proposal
timestamp: 2026-08-15
repository: https://github.com/miztertea/sergeant-rs
audit_revision: 242abe3c4a889c2b666c7ce34b32812dd1ee8d61
integration_review:
  pull_request: 111
  branch: integration/path-to-mac-2026-08-15
  revision: bceed965c24de7fa781001e3bd7835d8ef58b139
  merged: true
  merge_commit: 3a46b87c17d249655708ed5ac32f6704738776cf
supersedes:
  path: reference/proposal-tui-t-series.md
  revision: a9a25fa68938323d9585edc687fbf0e965084c2e
relationship: >-
  Complete revision of the existing T-Series proposal, not a competing
  proposal. It preserves the journal-first daemon, Work-centered domain,
  API-only boundary for daemon-owned facts, terminal-lifecycle guarantees,
  and separate P2-JOURNAL program. It updates the proposed interface for the
  shipped MVP, explicit sgt tui entry point, WATCH, transcript, output,
  envelope, Estate, dashboard deletion, completed_dirty, and PR #111's now-
  merged retained-state surfaces.
---

# Sergeant-rs T-Series
## Work-Centered Operator Cockpit

**Status:** Proposed  
**Main audit basis:** [`miztertea/sergeant-rs@242abe3`](https://github.com/miztertea/sergeant-rs/tree/242abe3c4a889c2b666c7ce34b32812dd1ee8d61)  
**Merged integration review:** [PR #111 at `bceed96`](https://github.com/miztertea/sergeant-rs/pull/111), merged into `main` at [`3a46b87`](https://github.com/miztertea/sergeant-rs/commit/3a46b87c17d249655708ed5ac32f6704738776cf)  
**Supersedes:** [`reference/proposal-tui-t-series.md@a9a25fa`](https://github.com/miztertea/sergeant-rs/blob/a9a25fa68938323d9585edc687fbf0e965084c2e/reference/proposal-tui-t-series.md)  
**Interactive entry point:** `sgt tui`; bare `sgt` remains the static estate-aware homepage  
**Primary objective:** Make the existing durable delegation loop obvious, beautiful, and operable from one keyboard-first terminal cockpit  
**Daemon mutation boundary:** Submit, respond, retry, extend, cancel, and any accepted retained-state disposal retain their daemon/API meanings  
**Local estate boundary:** Repository, group, and Doctor behavior is shared through narrow typed operations extracted on contact, never duplicated in the TUI  
**Journal boundary:** Global journal/DuckDB query and exploration remains owned by P2-JOURNAL  
**Web disposition:** The dashboard has been deleted; this proposal adds no browser surface or parity obligation  
**Scope discipline:** A surface adds usability, not hidden engine behavior; this is not a repository-wide CLI refactor  

Sections are numbered for contract citation. Every normative decision names its lowest viable Ponytail rung. The complete register is §22.

**Implementation status (2026-08-16):** T0 through T4 are built, tested, and merged to `integration/t-series-2026-08-15` (head PR #131) — see `GAUNTLET.md`'s T-SERIES-BUILD entry for the full record. `status: proposed` above is left as-is rather than changed to `published`, since that branch has not merged to `main`; this is the owner's call, not this document's. Two named gaps are explicitly not built by this program: the slash palette (§15.3) and the Workflows-screen half of the `@` chooser (§15.4) — tracked, not silently dropped.

---

# 1. Executive Summary

Sergeant now proves its North Star claim:

> A developer can hand Sergeant meaningful work and stop babysitting it.

The MVP has shipped the hard parts underneath the interface: estate topology, isolated multi-repository surfaces, pinned workflows, bounded turns and wall-clock ceilings, actor and deterministic execute stages, durable transcript and evidence, output pointers, recovery, Watch, Doctor, and a successful walk-away ship gate. The current TUI remains intentionally close to its original M6 proof: a Fleet list, a property sheet, forty recent events, and a small response/cancel keymap. The proof is architecturally strong and humanly thin.

The revised T-Series makes `sgt tui` the **operator cockpit for durable Work**.

The top-level destinations are:

```text
Home    Fleet    Workflows    Estate
```

- **Home** combines intent, attention, active Work, and recent outputs.
- **Fleet** is the complete browser over durable Work.
- **Workflows** is the admitted procedure catalog and launch surface.
- **Estate** is the full repository, group, and health destination.

Opening a Work from any destination always enters one canonical Work surface:

```text
Thread    Workflow    Evidence    Graph    Details
```

The default Thread is now a real journal-backed conversation surface because `GET /v1/work/{id}/transcript` exists. It interleaves transcript turns with only those semantic system lines that the journal can prove. It never invents hidden reasoning, files, progress, or process activity.

The cockpit uses the interaction grammar people already know from modern agent harnesses:

```text
top navigation
optional attention drawer
scrolling conversation or list body
fixed state-aware composer
contextual actions and help
```

The resemblance is intentional and bounded. Work, not chat, remains the durable center. The composer creates Work on Home and answers genuine `needs_input` requests inside Work. It does not become an unruled active-turn guidance channel.

The visual direction is an evolved modern terminal application:

```text
DESIGN_VARIANCE   4 / 10
MOTION_INTENSITY  3 / 10
VISUAL_DENSITY    7 / 10
```

It is dark, calm, dense, and information-rich without becoming cramped. It uses fewer full boxes, more whitespace and single dividers, one cyan/cool-blue focus accent, semantic gold/red/green state color, and motion only where something is actually active. The one honest progress bar is the Work's turn-envelope consumption. Workflow progress remains ordinal.

This proposal deliberately refuses the architectural detour that surfaced during design discussion. It does **not** refactor the whole CLI into a universal service layer. It follows an **extract-on-contact** rule:

```text
daemon-owned Work facts      existing ApiClient only
workflow discovery           one narrow read-only daemon projection
repo/group behavior          small shared local operations used by CLI and TUI
Doctor checks                one shared structured report used by CLI and TUI
everything else              remains where it is until a second surface needs it
```

The result is a full-fledged cockpit without turning a usability milestone into a multi-day platform rewrite.

**Decision T2-01 (R2):** Revise the existing T-Series proposal in place conceptually. Do not create a parallel TUI program with competing decisions.

**Decision T2-02 (R2):** The TUI is a Work-centered operator cockpit. It is not a process monitor, terminal multiplexer, system dashboard, or replacement agent harness.

---

# 2. Problem Statement and Outcome

## 2.1 The human problem

General-purpose coding harnesses are capable conversational actors, but durable work requires more than a conversation. Intent, estate context, procedure, execution bounds, outputs, human gates, and evidence must remain coherent after the harness turn or terminal session ends.

Sergeant solves that durable-work problem. The current TUI does not yet make the solution perceptible.

The operator still has to translate a flat table and a raw event tail into answers:

```text
What needs me?
What is running?
What is waiting, blocked, or failed?
Which Work should I open?
What did the actor actually say?
Which stage and attempt am I looking at?
How much of the turn envelope has been spent?
Where did the output land?
Which action is legal now?
What repositories and groups make up this estate?
Is the installation healthy?
```

The TUI's job is to answer those questions without owning the underlying facts.

This matches the work-centered theory already recorded in Notion:

- [Work-Centered Intelligence](https://app.notion.com/p/3ac27ada618f81728a73fbd7ac90c61c) argues that systems should be designed from durable work outward, with the prompt as a portal into Work rather than the container for it.
- [WorkPacket](https://app.notion.com/p/39a27ada618f818cba42f5efe8ffe1f0) states the interface boundary directly: surfaces show packet state and human decisions but do not own Work state.
- [Intelligent Work Environments](https://app.notion.com/p/3ac27ada618f817b8418e50151dd7015) connects ecological interface design to making work-domain constraints perceptible rather than forcing the operator to reconstruct them mentally.

**Decision T2-03 (R2):** The interface is organized around the questions an operator asks of durable Work, not around internal tables, processes, or module boundaries.

## 2.2 The product outcome

A successful T-Series session feels like this:

```text
sgt tui
  ↓
Home shows what needs attention and what is already moving
  ↓
The operator selects a repo/group, workflow, profile, and bounded intent
  ↓
Work appears in Fleet and the global drawer
  ↓
Opening it shows one coherent thread, procedure, evidence, and output
  ↓
The actor asks a question
  ↓
The operator answers deliberately from the fixed composer
  ↓
The Work resumes
  ↓
The operator returns later and sees either:
    trustworthy output
    an honest, actionable stop
    or retained state requiring explicit review
```

The cockpit also makes the estate operable:

```text
Estate / Repositories
Estate / Groups
Estate / Health
```

The operator can inspect and manage the working set through the same validated semantics the CLI already uses.

**Decision T2-04 (R2):** The acceptance unit is the complete operator loop, not the existence of individual screens.

---

# 3. Supersession and Audit Basis

## 3.1 Why the 2026-08-11 proposal must be revised

The original T-Series proposal was correct about Work-centered presentation, the canonical Work surface, local `/` and `@` grammars, deliberate multiline submission, and the separate Journal program. Its audit point predates the MVP and several owner rulings.

The following assumptions are now stale:

| Previous proposal assumption | Current fact |
|---|---|
| Bare `sgt` opens the TUI | Bare `sgt` is a static estate-aware homepage; `sgt tui` is explicit |
| Web should be unmounted but retained as a stub | Dashboard source, routes, assets, command, and tests were deleted under ADR 0011 |
| T-Series owns issues #11 and #16 | #11, #16, and #26 have shipped fixes that T-Series must preserve |
| Thread must be synthesized mainly from event history | A dedicated Work transcript endpoint and CLI exist |
| Work exposes no output pointer or envelope | Work views expose output and enforced turn-envelope facts |
| Current mutations are submit/respond/retry/cancel | `extend` also exists and is distinct from retry |
| Terminal completion is only `completed` | Operator-facing `completed_dirty` exists |
| There is no headless return path | `sgt watch` is shipped and adjudicated |
| Estate administration remains outside the TUI discussion | Repo/group lifecycle and Doctor are now mature CLI surfaces |
| Dashboard reactivation issues remain | #15 and #21 closed when the dashboard was deleted |

Sources:

- [`src/cli.rs@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/cli.rs)
- [`src/api.rs@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/api.rs)
- [`src/tui.rs@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/tui.rs)
- [`README.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/README.md)
- [PR #105](https://github.com/miztertea/sergeant-rs/pull/105)
- [PR #69](https://github.com/miztertea/sergeant-rs/pull/69)

**Decision T2-05 (R1/R2):** Preserve sound decisions from the existing proposal; delete or rewrite every decision whose premise has been superseded by shipped behavior.

## 3.2 Main revision

The binding source audit is pinned to:

```text
main = 242abe3c4a889c2b666c7ce34b32812dd1ee8d61
```

At this revision:

- `sgt tui` is explicit and no-spawn.
- the TUI still has only Fleet and Detail screens;
- automatic SSE reconnect, auth-failure stop, and terminal safety are implemented;
- transcript, output, envelope, extend, Watch, Estate, Doctor, and harness passthrough exist;
- the dashboard is absent;
- the workflow root catalog lists 23 admitted workflows;
- Ratatui 0.30.2 is installed and Crossterm is consumed through Ratatui's re-export.

## 3.3 Integration branch, now merged

PR #111 merged into `main` at 2026-08-15T15:28:02Z, per `gh pr view 111`:

```text
PR #111
integration/path-to-mac-2026-08-15
head = bceed965c24de7fa781001e3bd7835d8ef58b139
merge commit = 3a46b87c17d249655708ed5ac32f6704738776cf
```

The branch's candidate surfaces are now shipped fact on `main`:

- ceiling interruption lands in `blocked` rather than wedged `active`;
- human transcript rendering includes journal timestamps;
- Doctor gains a filesystem reliability check;
- `GET /v1/retained` and `sgt work retained` inspect retained state;
- `POST /v1/work/{id}/reap` and `sgt work reap` explicitly dispose retained dirty state after confirmation.

The shipping-gate defect that made the PR's own pre-merge gate runs unreliable, issue #120, is a separate, still-open risk. It is not resolved by the merge — it is tracked in §19.12 and acceptance item 57 (§21).

**Decision T2-06 (R1):** Resolved. T0 pins the actual implementation base as `main` post-merge, including PR #111's surfaces.

**Decision T2-07 (R2):** Resolved, no longer conditional. The retained/reap surfaces are consumed in Work output and Health drill-down.

## 3.4 Audit corpus

This revision is grounded in:

- [`NORTH-STAR.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/NORTH-STAR.md)
- [`AGENTS.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/AGENTS.md)
- [`docs/DEVELOPMENT.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/DEVELOPMENT.md)
- [`GAUNTLET.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/GAUNTLET.md)
- [`LESSONS.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/LESSONS.md)
- [WATCH contract](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/gauntlet/contracts/WATCH.md)
- [ADRs 0005-0011](https://github.com/miztertea/sergeant-rs/tree/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/adr)
- [current workflow catalog](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/.sergeant/index.md)
- the existing T-Series proposal in full;
- PR #111's source patches and contract record;
- the Notion work-centered research above;
- the [Taste skill at `e988add`](https://github.com/Leonxlnx/taste-skill/blob/e988add20dab0fa97d7a76781c48961c8184288e/skills/taste-skill/SKILL.md);
- official Ratatui, Crossterm, and text-area documentation cited in §8.

---

# 4. Design Method: Taste Applied Honestly

## 4.1 Boundary of the Taste skill

The Taste skill says directly that it is not intended for dashboards, dense product interfaces, or multi-step product UI. Sergeant is exactly that class of application.

This proposal therefore does not import its React, Tailwind, marketing-page, image, web-animation, or component-system prescriptions. It uses the transferable discipline:

```text
infer the real brief
audit before redesign
set design variance / motion / density explicitly
reject default AI aesthetics
use one coherent color and shape system
make hierarchy visible
provide loading / empty / error / focus states
run a mechanical pre-flight before shipping
```

**Decision T2-08 (R2):** Use Taste as an audit method and anti-slop filter, not as a web implementation authority.

## 4.2 Design read

> Reading this as: a greenfield-overhaul of a keyboard-first operator console for technical users, with a restrained dark developer-tool language, leaning toward Claude Code and Primer-style clarity rather than a sysadmin dashboard.

No military visual language is needed. Sergeant's name and domain do not require green tactical chrome, ranks, insignia, or novelty metaphors.

**Decision T2-09 (R1):** Reject military styling. Product identity comes from interaction quality and domain clarity.

## 4.3 Design dials

```text
DESIGN_VARIANCE   4 / 10
MOTION_INTENSITY  3 / 10
VISUAL_DENSITY    7 / 10
```

- **Variance 4:** geometry is stable and predictable; hierarchy changes by context, not by decorative asymmetry.
- **Motion 3:** state feedback and active execution only; no cinematic transitions or pulsing alerts.
- **Density 7:** enough information for real operations, with progressive disclosure preventing a cockpit from becoming clutter.

**Decision T2-10 (R5):** These dials are binding visual constraints, not marketing metadata.

## 4.4 Audit of the prior renderings

The previous renderings established a strong foundation:

- dark neutral canvas;
- terminal-native lines and glyphs;
- readable mono hierarchy;
- subtle cyan focus;
- semantic state color;
- consistent footer hints;
- layouts plausible in a character grid.

They also exposed recurring problems:

- almost every region received a full box and uppercase heading;
- stale System/resource, Files/diff, Artifacts, Web, and `/interrupt` surfaces re-entered the mockup;
- the visual language drifted toward a terminalized administration dashboard;
- Work was sometimes represented twice through a full view and separate quick-view modal;
- gauges appeared where the underlying ratio was not truthful.

The evolved rule is:

```text
full border      one major interactive region
single divider   subordinate related section
whitespace       grouping and breathing room
accent border    current focus only
color            semantic state only
```

**Decision T2-11 (R5):** Reduce border count and visual noise before adding decoration.

---

# 5. Governing Invariants

## 5.1 Work remains the center

The North Star says surfaces own presentation and steering, while the core owns durable execution. Work-Centered Intelligence says the actor moves and Work remains.

**Decision T2-12 (R2):** Navigation, threads, drawers, and composers are views into Work. None becomes a new durable domain object.

## 5.2 Daemon-owned truth enters through the API

The current TUI's strongest architectural property is that it imports `ApiClient` and does not reach into the journal, projection, engine, daemon, or backend registry.

**Decision T2-13 (R2):** All Work, execution, transcript, event, graph, analytics, output, envelope, and retained-state facts come through authenticated daemon routes.

The workflow catalog remains the one proposed read-only addition because workflow resolution participates in execution planning and should not become a TUI-only filesystem interpretation.

## 5.3 Estate-local behavior is shared, not tunneled through the daemon

Repo/group commands are deliberately local manifest operations. Doctor must work when no daemon is running. Forcing them through new daemon routes solely for UI purity would violate their existing lifecycle semantics.

**Decision T2-14 (R2/R6): resolved — extend the daemon API.** The proposal's owner ruled on 2026-08-16: repo/group and Doctor behavior reaches `tui.rs` exclusively through new authenticated daemon API routes, consumed via `ApiClient` like every other daemon-owned fact. `tests/m6_surfaces.rs`'s `t5_the_tui_is_a_client_like_any_other` and `t5b_the_structural_scan_sees_every_spelling_of_a_path` are **not revised** — they remain exactly as they are today, unweakened. `tui.rs` gets no second crate-internal reach path.

The CLI's existing local, no-daemon code paths — `crate::domain::manifest` for repo/group, `mod doctor`'s `Check`-producing functions for Doctor — are unchanged: the CLI keeps calling them directly, in-process, so `sgt doctor` and `sgt repo add` still work with no daemon running at all (needed for `sgt init` and pre-daemon diagnosis). The new API routes are thin daemon-side wrappers over those same existing functions; no logic is duplicated and no CLI behavior changes.

This is the option that actually satisfies "the CLI and TUI are both clients consuming the same core": repo/group and Doctor become core-owned state that both clients read identically, rather than a compromise of that principle. The CLI's direct local calls are a separate, deliberate exception for its own no-daemon requirement — not a second client-boundary path for `tui.rs`.

This is an explicit refinement of the old "TUI imports only ApiClient" source-scan rule:

```text
daemon-owned facts     ApiClient only
estate manifest edits  ApiClient only (CLI reaches crate::domain::manifest directly and locally, a separate no-daemon exception)
installation checks    ApiClient only (CLI reaches mod doctor's Check-producing functions directly and locally, a separate no-daemon exception)
```

## 5.4 SSE remains invalidation

**Decision T2-15 (R2):** A live event says the answer changed. The TUI rereads authoritative state and does not become a second reducer.

## 5.5 Observation never materializes the daemon

Watch and Doctor already belonged to the no-auto-spawn set; ADR 0009 joined `status`, Work reads, analytics, and TUI to it. All six are no-spawn today.

**Decision T2-16 (R2):** `sgt tui` continues to refuse without a running daemon and names `sgt doctor` or a dispatching verb as the remedy. The Estate/Health screens do not create an offline exception inside the TUI.

## 5.6 Execution is not dialogue

R-NS-6 distinguishes execution mechanics from the harness-owned conversation. `respond` answers a parked request. It is not a generic message operation. Whether a transport's actor can ask mid-run is itself a measured per-transport capability with runtime withdrawal, never new hold machinery; its named consequence is that the WORKFLOW-IF-E3 category is empty and grilling-class packages are operator skills.

**Decision T2-17 (R1/R2):** T-Series adds no arbitrary active-turn guidance, continuous chat, embedded harness session, or PTY supervision.

## 5.7 Process liveness is not Work truth

**Decision T2-18 (R2):** Spinners, labels, and actions derive from journal-backed projected state. Silence, elapsed time, or a process table never creates a Work transition.

## 5.8 Ponytail is binding in both directions

```text
R1  do not build it when the need is unproven
R2  reuse current repository capability
R3  use the standard library
R4  use a native platform/terminal capability
R5  use an installed dependency
R6  add a tiny local composition or extraction
R7  add the minimum new machinery after naming failed lower rungs
```

Overbuilt frameworks and underbuilt shortcuts are both violations.

**Decision T2-19 (R2):** Every implementation deviation names its rung and the lower rungs it exhausted.

---

# 6. Scope Contract

## 6.1 In scope

T-Series may:

1. replace the current Fleet/Detail hierarchy with Home, Fleet, Workflows, Estate, and canonical Work;
2. render a modern coherent visual system through Ratatui;
3. submit Work through current intent, workflow, backend, profile, workspace, repository, envelope, and origin fields;
4. expand an Estate group into the same repository selection current `sgt run --group` produces;
5. add one authenticated read-only workflow catalog route over admitted procedure;
6. display the authoritative Work transcript and Work-local event evidence;
7. display output pointer, teardown, reservation, execution, stage executor, envelope, and reported state;
8. invoke current respond, retry, extend, cancel, and retained-state disposal semantics;
9. expose the complete existing repo/group lifecycle through Estate;
10. expose Doctor checks and named remedies through Estate/Health;
11. derive a global Attention drawer from Work state;
12. use Watch's adjudicated state vocabulary without launching Watch inside the TUI;
13. add a fixed local slash palette and workflow chooser;
14. add a multiline deliberate composer with portable confirmation fallback;
15. preserve and test reconnect, authentication-failure, signal, PTY, shutdown, idle-CPU, and no-spawn behavior;
16. update README screenshots only from the real implemented Ratatui application.

## 6.2 Explicit non-goals

**Decision T2-20 (R1):** T-Series does not include:

- global Journal/DuckDB query, full-text search, arbitrary SQL, saved searches, alerts, or a Journal tab;
- CPU, memory, process, or host resource monitoring;
- OpenTelemetry collector queries;
- mouse interaction;
- embedded web, browser controls, or parity work;
- Files, diffs, artifacts, commits, findings, or code viewers without current API facts;
- a spatial graph canvas;
- arbitrary active-Work guidance;
- interrupt behavior added merely because the backend trait contains an interrupt capability;
- continuous interactive harness sessions;
- embedded PTY or harness process supervision;
- `sgt init`, daemon foreground/stop administration, or harness passthrough screens;
- JSON/JSONL automation;
- workflow authoring, editing, generation, publication, or draft promotion;
- a model catalog or capability catalog not exposed by the current API;
- a universal application service, command bus, plugin system, or repository-wide CLI rewrite;
- configurable keybindings;
- mouse-enabled text editing;
- a second durable notification/read-state store;
- archive, snooze, pin, or dismissal semantics for Work;
- consumption of any candidate surface beyond what PR #111 actually shipped.

---

# 7. Information Architecture

## 7.1 Top-level destinations

**Decision T2-21 (R2/R5):**

```text
Home    Fleet    Workflows    Estate
```

There is no top-level Work tab because Work is the object entered from the other destinations. There is no System tab because Health belongs to the Estate and resource monitoring is out of scope. There is no Explore placeholder because the Journal program has not yet defined that product.

A representative header:

```text
sergeant   Home  Fleet  Workflows  Estate             ? 2  ! 1    live
```

The right side shows:

- `? N` for current human input requests;
- `! N` for blocked/failed/completed-dirty Work;
- connection truth;
- optional admission-paused status.

## 7.2 Canonical Work

**Decision T2-22 (R2):** Every Work selection opens one canonical full-body Work surface.

Entry points include:

```text
Home attention/current/recent rows
Fleet
Workflow recent-run row derived from loaded Fleet
Attention drawer
Graph or analytics result carrying work_id
Retained-state result
```

`Esc` returns to the exact prior destination, filter, selection, focus, and scroll position.

## 7.3 Global Attention drawer

**Decision T2-23 (R2/R5):** `~` toggles one global left drawer. It stores no notification state.

```text
NEEDS INPUT
  needs_input

STOPPED
  blocked
  failed
  completed_dirty

WAITING
  waiting

RUNNING
  pending
  active

FINISHED
  completed
  canceled
```

Watch does not treat `pending` or `active` as notice-producing states, but the cockpit may show them under Running because the drawer is a fleet view, not the Watch protocol itself.

The drawer:

- opens by default at wide widths;
- overlays at narrow widths;
- restores selection and scroll position;
- leads with intent, not ULID;
- never blinks or pulses;
- bounds Finished rows.

## 7.4 Overlays

The application has a small fixed set:

```text
slash command palette
workflow chooser
help
cancel confirmation
extend envelope
repo add/remove
group edit/remove
retained-state preview/reap confirmation, if available
connection detail
terminal-too-small notice
```

**Decision T2-24 (R1/R5):** Overlays are contextual views over existing operations, not a modal framework or second navigation system.

---

# 8. Visual System and Ratatui Feasibility

## 8.1 Theme

**Decision T2-25 (R5):** One dark theme governs the whole TUI.

Semantic tokens:

```text
background       near-black neutral
surface          slightly raised cool neutral
surface-muted    secondary region
border           low-contrast cool gray
text             off-white
muted            cool gray
focus/accent     restrained cyan or cool blue
success          green
attention        gold/yellow
warning          amber where reliable, else yellow
danger           red
info/reference   restrained violet only for workflow references
```

No pure black/white requirement is imposed on terminal users. Where truecolor is unavailable, the implementation falls back to standard/256-color equivalents while preserving labels and glyphs.

The terminal controls the font. The TUI controls only weight, brightness, underline, dim, and spacing.

## 8.2 Shape and hierarchy

**Decision T2-26 (R5):**

```text
major focus region     full border
secondary grouping     one divider or title line
metadata               aligned text and whitespace
selected row           background or accent marker
focused control        accent border/title/cursor
```

Nested boxes inside boxes are prohibited unless the inner region is independently focusable.

Uppercase micro-headings are rationed. A screen should not look like every paragraph is a subsystem.

## 8.3 Built-in widgets

Ratatui 0.30.2 already supplies and re-exports the primitives needed here:

- `Tabs`
- `Table` and `List`
- `Paragraph` with wrapping
- `Scrollbar`
- `Block`
- `Clear` for overlays
- `Gauge` and `LineGauge`
- styled `Span`, `Line`, and `Text`

Official references:

- [Ratatui 0.30.2](https://docs.rs/ratatui/0.30.2/ratatui/)
- [built-in widgets](https://docs.rs/ratatui/0.30.2/ratatui/widgets/)
- [LineGauge](https://docs.rs/ratatui/0.30.2/ratatui/widgets/struct.LineGauge.html)
- [TestBackend](https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.TestBackend.html)

Canvas exists, but is not evidence that a spatial graph is the right terminal interaction.

**Decision T2-27 (R5):** Build the visual system from Ratatui primitives already in the dependency graph. Add no UI framework, animation crate, or charting package.

## 8.4 State glyphs

**Decision T2-28 (R5):** Every important state uses text, glyph, and color.

| State | Glyph | Treatment |
|---|---:|---|
| pending | `·` | muted |
| active | spinner frame | cyan |
| waiting | `○` | muted blue/gray |
| needs_input | `?` | gold, bold |
| blocked | `!` | amber/yellow |
| failed | `×` | red |
| completed | `✓` | green |
| completed_dirty | `!` | gold with `output needs review` |
| canceled | `-` | dim gray |
| reconnecting | `↻` | yellow with explicit label |
| auth failed | `!` | red with explicit label |

Every glyph is tested for one-cell width. ASCII fallback is mandatory.

## 8.5 Motion

**Decision T2-29 (R2/R6):** Only attached Work whose projected state is `active` animates.

The spinner is a local frame array advanced by the existing event-loop tick. It communicates lifecycle motion, not native process proof.

Motion stops when:

- the SSE tail is reconnecting;
- authentication failed;
- the TUI marks data stale;
- reduced Unicode capability selects a static fallback.

No pulsing attention, animated borders, loading shimmer, or transition framework.

## 8.6 Progress

**Decision T2-30 (R1/R2/R5):** Use a `LineGauge` only for the real turn-envelope ratio:

```text
Turns  4 / 12   ━━━━━━━────────────
Ceiling  40m per turn
```

The label always states numerator and denominator. A cap of zero/unknown does not render a fabricated ratio.

Workflow progress is ordinal:

```text
stage 4 / 10
✓ plan  ✓ context  ⠹ implement  · verify  · close
```

It never becomes 40 percent complete.

## 8.7 Multiline editor

The original proposal specified a custom local editor. Current ecosystem research changes the lower-rung analysis.

[`ratatui-textarea`](https://docs.rs/ratatui-textarea/latest/ratatui_textarea/) is a maintained Ratatui-native multiline editor with insertion, deletion, wrapping, cursor movement, scrolling, and custom key mappings. A mature Ratatui application such as GitUI currently pairs Ratatui 0.30 with `ratatui-textarea`.

**Decision T2-31 (R7, dependency-tree gated):** Prefer one narrow wrapper over a pinned compatible `ratatui-textarea` release instead of hand-building cursor-aware multiline editing.

Failed lower rungs:

- R1 fails: multiline deliberate input is settled.
- R2 fails: the current one-line `String` buffer cannot satisfy it.
- R3/R4 fail: standard Rust and terminal events do not provide editor behavior.
- R5 fails: the currently installed dependency set has no editor.
- R6 fails: correct wrapping, visual-row cursor movement, paste, delete, and scrolling are not a tiny composition.

The dependency is admitted only if a T0 spike proves:

```text
one resolved Ratatui version
one resolved Crossterm version
no direct conflicting crossterm edge
no search/regex feature
no mouse requirement
no editor-owned submit behavior
pure access to the local draft for testing
the m6_surfaces Ratatui/Crossterm gate test still passes unmodified
```

If that proof fails, the dependency is refused and the bounded local-editor fallback from the original proposal is used. The user-visible contract does not change.

**T0 ran this spike on 2026-08-16, read-only (no edit to this repository's `Cargo.toml`/`Cargo.lock`), and it proves the dependency admitted. Outcome per condition:**

1. **One resolved Ratatui version.** Both this repository's `Cargo.lock` and a scratch crate (`ratatui = "0.30.2"` + `ratatui-textarea = "0.9.2"`, built outside this repository) resolve exactly one `ratatui`: `0.30.2`. `ratatui-textarea` depends on `ratatui-core`/`ratatui-widgets` at the same versions Ratatui itself pins — no second Ratatui line.
2. **One resolved Crossterm version, shared.** In the scratch crate, `cargo tree -i crossterm` shows exactly one resolved `crossterm` (`0.29.0`), reached by both `ratatui` and `ratatui-textarea` through the same `ratatui-crossterm v0.1.2` crate — `ratatui-textarea` has no `crossterm` dependency of its own; its `crossterm` feature (`crossterm = ["dep:ratatui-crossterm"]`, `default = ["crossterm"]`) routes through the identical shared crate Ratatui already uses. This repository's own `Cargo.lock` additionally carries a second, unrelated `crossterm 0.28.1` pulled in by `comfy-table` (a `duckdb`/`arrow-cast` transitive dependency, for the DuckDB journal feature — nothing to do with the TUI). That duplication predates this spike, is untouched by it, and does not grow to a third version when `ratatui-textarea` is added: it would still resolve to the exact same shared `0.29.0` Ratatui already pulls.
3. **No direct conflicting crossterm edge.** Confirmed by the same `cargo tree -i crossterm` output above — `ratatui-textarea` has zero direct edge to `crossterm`; its only path to it is through `ratatui-crossterm`, the identical crate Ratatui itself depends on.
4. **No search/regex feature.** `cargo add ratatui-textarea --dry-run` against this repository's actual `Cargo.toml` (aborted before any file was written) reports the feature set that would be added: `+ crossterm`, `- arbitrary`, `- portable-atomic`, `- search`, `- serde`, `- termion`, `- termwiz`. The crate's own `Cargo.toml` confirms `default = ["crossterm"]` and `search = ["dep:regex"]` — `regex` is not pulled in without explicitly opting into the `search` feature, which this proposal does not request.
5. **No mouse requirement.** `ratatui-textarea-0.9.2/src/input/crossterm.rs` optionally maps `MouseEventKind::ScrollUp`/`ScrollDown` into `Key::MouseScrollUp`/`MouseScrollDown` *if* the host application forwards a `crossterm::event::Event::Mouse` into `TextArea::input()`. Nothing in the crate requires mouse capture or calls it unprompted. Decision T2-33 keeps mouse capture disabled entirely, so the host never enables `EnableMouseCapture` and never forwards a `Mouse` event — this optional path stays permanently unused.
6. **No editor-owned submit behavior.** `TextArea::input()` (`textarea.rs:283`) maps `Key::Enter` to `insert_newline()` only (`textarea.rs:296-301`, `:730-734`) — there is no submit/send concept anywhere in the crate. The host decides whether a keystroke means "newline" or "submit" by choosing whether to call `.input()` at all before forwarding it, exactly as §8.8's `Ctrl+Enter`-vs-`Enter` distinction already requires independent of this dependency.
7. **Pure access to the local draft for testing.** `pub fn lines(&self) -> &[String]` and `pub fn into_lines(self) -> Vec<String>` (`textarea.rs:2170`, `:2186`) read the buffer directly with no terminal, render, or event-loop dependency — usable from a plain unit test.
8. **The m6_surfaces gate test still passes unmodified.** `cargo test --test m6_surfaces the_tui_stack_is_ratatui_with_crossterm_reached_through_it` against this repository as-is: `test result: ok. 1 passed; 0 failed`. `git status` before and after the spike (including this test run) shows no working-tree change — the spike touched no product file.

All eight conditions hold. The dependency is admitted, gated as above; T1 implements the narrow wrapper described in Decision T2-31.

## 8.8 Enhanced keyboard protocol

Crossterm `KeyEvent` carries code, modifiers, kind, and state. `PushKeyboardEnhancementFlags` can enable the Kitty keyboard protocol in compatible terminals and should be paired with `PopKeyboardEnhancementFlags`.

References:

- [Crossterm `KeyEvent`](https://docs.rs/crossterm/0.29.0/crossterm/event/struct.KeyEvent.html)
- [`PushKeyboardEnhancementFlags`](https://docs.rs/crossterm/0.29.0/crossterm/event/struct.PushKeyboardEnhancementFlags.html)
- [`KeyboardEnhancementFlags`](https://docs.rs/crossterm/0.29.0/crossterm/event/struct.KeyboardEnhancementFlags.html)

**Decision T2-32 (R4/R6):** Preserve full `KeyEvent` values and opportunistically request disambiguated modified keys. Failure or lack of terminal support is nonfatal.

```text
Enter                 newline
Ctrl+Enter            submit when distinguishable
Tab to Send + Enter   universal deliberate fallback
```

Push/pop is integrated into the existing terminal lifecycle guard and tested on every exit path. Ctrl+Enter is never the only route.

## 8.9 Mouse

**Decision T2-33 (R1):** Mouse capture remains disabled. Every action is reachable, visible, and testable from the keyboard.

## 8.10 Locked visual tokens and responsive geometries

**Decision T2-65 (R5):** §8.1's semantic tokens and §18's responsive compositions are prose descriptions of an intended system; T0 finalizes them as concrete, implementable values here so T1 does not invent numbers. Every named ratatui `Color` variant below is already in use in the current M6 `src/tui.rs` (`Cyan`, `DarkGray`, `Green`, `Red`, `Yellow` — no `Rgb`/truecolor call exists there today); the truecolor column is new for T1 and the ANSI column is its fallback, per §8.1's existing "falls back to standard/256-color equivalents" rule — not a second, competing rule.

### Color roles

| Token | Truecolor (`Color::Rgb`) | 256-color index | ANSI/16-color fallback (`ratatui::style::Color`) |
|---|---|---:|---|
| background | `#0c0f13` (12,15,19) | 233 | `Black` |
| surface | `#12161c` (18,22,28) | 234 | `Black` |
| surface-muted | `#181d25` (24,29,37) | 235 | `DarkGray` |
| border | `#2b323c` (43,50,60) | 238 | `DarkGray` |
| text | `#e6e9ef` (230,233,239) | 253 | `White` |
| muted | `#7c8695` (124,134,149) | 245 | `Gray` |
| focus/accent | `#57b6c9` (87,182,201) | 73 | `Cyan` |
| success | `#3fb968` (63,185,104) | 77 | `Green` |
| attention (gold) | `#d7a72c` (215,167,44) | 178 | `Yellow` |
| warning (amber) | `#d68a3c` (214,138,60) | 173 | `Yellow` — amber has no distinct 16-color slot; per §8.1 it collapses onto the same fallback as attention/gold at that tier, distinguished by label text and glyph, never by color alone (§19.11's "no color-only states") |
| danger | `#e0564f` (224,86,79) | 203 | `Red` |
| info/reference (violet) | `#a487e0` (164,135,224) | 140 | `Magenta` |

Terminal-capability detection follows existing Crossterm/terminfo practice already governing §8.1; a capability that cannot report truecolor or 256-color uses the ANSI column outright. No token is ever pure `Black`/`White` at the truecolor tier, satisfying T2-25's "no pure black/white requirement" while still giving `Black`/`White` as an honest 16-color-terminal fallback name.

### Spacing scale

A terminal cell is the only unit; there is no sub-cell spacing.

| Scale step | Cells | Use |
|---|---:|---|
| none | 0 | a divider or border glyph touches its neighboring region directly |
| inline | 1 | gap between a state glyph and its label; blank column between adjacent metadata fields within one row |
| block | 2 | a focusable `Block`'s interior left/right padding; a blank row separating two stacked sections that have no divider between them |
| region | 3 | outer margin between the Attention drawer or contextual rail and the primary body at Wide composition (§18.1) |

### Responsive breakpoints (§18.1-18.3, locked)

| Tier | Columns | Drawer | Contextual rail | Primary body |
|---|---|---|---|---|
| Wide | ≥ 150 | inline, fixed 28 cols | inline, fixed 32 cols | remaining columns (≥ 90) |
| Medium | 100-149 | inline-optional, fixed 24 cols when open; overlay when closed-then-toggled | folded into a selectable subview inside the primary body, no reserved columns | remaining columns |
| Narrow | 80-99 | full-body overlay, temporarily replaces the primary body | full-body overlay, temporarily replaces the primary body | entire body when no overlay is open |
| Below 80 columns, or below 24 rows | — | — | — | §17.7's terminal-too-small notice; no composition is attempted |

The 24-row floor (§18.4) is symmetric with the 80-column floor: either one being violated hands off to §17.7 rather than a degraded layout attempt. §19.10's geometry matrix (`80x24`, `120x36`, `180x48`) exercises Narrow, Medium, and Wide respectively at representative interior sizes.

---

# 9. Home

Home is the command center for current Work and new intent.

A wide composition:

```text
┌ Attention ─────────┬ Home / New Work ─────────────────────┬ Recent Outputs ┐
│ needs input        │ target       payments (group)         │ payment worker │
│ blocked            │ workflow     @implement               │ a412ce7  23m   │
│ waiting            │ backend      default                  │                │
│ running            │ profile      sonnet                   │ Running Now    │
│ completed-review   │ turns        4 / 12                   │ ...            │
│                    │ ceiling      40m                      │                │
│                    │                                      │                │
│                    │ intent composer                       │                │
│                    │                                      │                │
│                    │                         [ Run Work ]   │                │
└────────────────────┴──────────────────────────────────────┴────────────────┘
```

## 9.1 New Work fields

**Decision T2-34 (R2):** Home maps to current submission semantics.

```text
intent
workflow
backend
profile
workspace
repositories[]
envelope.turn_cap
envelope.ceiling_secs
created_by = "tui"
origin.client = "tui"
origin.cwd = launch cwd
```

Target selection uses the estate's declared repositories and groups. A selected group is expanded client-side into `repositories[]` through the same shared logic as `sgt run --group`; no new group field is invented in the daemon request.

Workflow selection uses the catalog endpoint. Profiles may be listed from the current estate manifest through shared read logic.

There is no model selector because current submission has no direct model field and no public model catalog. Backend remains default or exact-name unless a current authoritative catalog is added outside this proposal.

**Decision T2-35 (R1/R2):** Do not expose capability-driven backend/model controls until an API or local authoritative source actually provides the required catalog. The layout leaves room for future valid controls but ships none disabled or guessed.

## 9.2 Deliberate submission

The intent editor is the primary focus. The request summary remains visible before submission.

```text
Ctrl+Enter submit
Tab to [ Run Work ] then Enter
Enter newline
Esc leave focus and preserve draft
```

The draft clears only after the daemon accepts the Work. Structured errors preserve it.

When admission is paused, Home shows the current daemon fact and disables Run Work with the named reason.

## 9.3 Attention and running Work

Rows show:

```text
state glyph
intent
stage coordinate
question/reason when present
turns spawned/cap for active Work
age labeled as submission age
```

ULIDs are secondary.

## 9.4 Recent outputs

**Decision T2-36 (R2):** Recent Outputs displays only terminal Work with a non-null output pointer.

It may show:

```text
intent
repository
retained branch
finalize commit
disposition
submission/terminal timestamp only where the API supplies it honestly
```

It does not imply a diff, file list, artifact inventory, or successful promotion beyond the output pointer's facts.

`completed_dirty` appears as `output needs review`, never a green success.

Home bounds this list. Fleet remains complete.

---

# 10. Fleet

Fleet is the complete Work browser.

## 10.1 Layout

Wide:

```text
filters | Work table/list
        | selected Work preview
```

Medium:

```text
Work list
selected preview below or toggle
```

Narrow:

```text
intent-first stacked rows
filter overlay
```

**Decision T2-37 (R5):** Use Ratatui `Table` or two-line `List` rows with constraints, wrapping, and explicit truncation. Never return to fixed padded strings.

## 10.2 Row priority

```text
intent
state
stage
target
workflow
turn envelope
backend/executor
age
short id
```

At 80 columns, intent/state/stage survive before the rest.

## 10.3 Filters

**Decision T2-38 (R2):** Local Fleet filters use fields already returned in the Fleet projection:

```text
text
state
terminal/nonterminal
workflow
workspace
repository/target where present
backend
envelope pressure
```

This is not Journal search.

## 10.4 Reported states

Fleet visibly distinguishes:

```text
completed
completed_dirty
needs_input
waiting
blocked
failed
active
pending
canceled
```

`completed_dirty` is grouped with review-required Work, not normal success.

## 10.5 Actions

Enter opens canonical Work. Mutations are available through the contextual palette and confirmations, not single-key destructive shortcuts.

Retained-state markers may appear on relevant terminal rows only when the current API reports them.

---

# 11. Workflows

## 11.1 Catalog authority

The current root catalog lists 23 admitted workflows and excludes drafts. It delegates description/tags to each workflow's `index.md`; `workflow.toml` owns executable identity and stage order.

Sources:

- [`.sergeant/index.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/.sergeant/index.md)
- [`docs/icm/convention.md@242abe3`](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/icm/convention.md)

**Decision T2-39 (R2):** Show only root-indexed published workflows plus the honest embedded fallback when not overridden. Never scan arbitrary directories or list drafts.

## 11.2 Read-only catalog route

The proposal retains the original minimum addition:

```text
GET /v1/workflows?cwd=<percent-encoded path>
```

The route reuses:

- current estate/workspace discovery;
- current workflow loader and validation;
- root publication boundary;
- current embedded fallback;
- existing Axum and `ApiClient` patterns.

It performs no mutation and appends no event.

**Decision T2-40 (R2/R6/R7):** Add one workflow catalog projection because the TUI must not privately reinterpret executable procedure.

Failed lower rungs:

- R1 fails: workflow discovery in the TUI is a settled requirement, not an unproven need.
- R2 fails: no existing authenticated route already exposes the catalog to a client.
- R6 fails: a new authenticated route, its own versioned response contract, and the Axum handler wiring it requires are more than a tiny local composition or extraction.

**T0 finalized this schema on 2026-08-16, grounded in `src/domain/workflow.rs`'s `WorkflowDefinition`/`StageDefinition` and each workflow's own `index.md` front matter (`.sergeant/workflows/implement/index.md` read as the concrete example). No field below is invented; every one is traced to a real struct field or a real front-matter key. `status`/`description`/`tags` require T2 to add `index.md` front-matter parsing — no code path reads it today (`.sergeant/index.md`'s own root table is hand-maintained prose, never machine-read); this is new T2 loader work, not a T0 finding.**

Success response, `200`, `Content-Type: application/json`:

```json
{
  "workflows": [
    {
      "name": "implement",
      "version": "2",
      "source": "/home/operator/repos/payments/.sergeant/workflows/implement",
      "content_hash": "9f2b1c4d5e6f...  (64 lowercase hex chars, BLAKE3)",
      "status": "published",
      "description": "Implement a piece of work from a spec or ticket set, explicit-invocation-only.",
      "tags": ["implementation", "explicit-invocation"],
      "stages": [
        { "id": "10-implement-with-tdd", "kind": "actor", "harness": null, "profile": null, "requires_ask": false },
        { "id": "30-review", "kind": "actor", "harness": null, "profile": null, "requires_ask": false }
      ]
    }
  ]
}
```

When no repository catalog resolves for `cwd` and the built-in fallback answers instead (`WorkflowDefinition::embedded()`, name always `software-change`):

```json
{
  "workflows": [
    {
      "name": "software-change",
      "version": "1",
      "source": "embedded",
      "content_hash": "...  (64 lowercase hex chars)",
      "stages": [
        { "id": "00-prepare", "kind": "actor", "harness": null, "profile": null, "requires_ask": false },
        { "id": "10-implement", "kind": "actor", "harness": null, "profile": null, "requires_ask": false },
        { "id": "20-review", "kind": "actor", "harness": null, "profile": null, "requires_ask": false },
        { "id": "30-close", "kind": "actor", "harness": null, "profile": null, "requires_ask": false }
      ]
    }
  ]
}
```

`status`, `description`, and `tags` are absent (not `null` — omitted) for the embedded entry: it has no `index.md` to read them from.

**`workflows[]` — CatalogEntry:**

| Field | Type | Required | Source |
|---|---|---|---|
| `name` | string | yes | `WorkflowDefinition.name` (`workflow.toml` `[workflow].name`) |
| `version` | string | yes | `WorkflowDefinition.version` (`workflow.toml` `[workflow].version`; a quoted TOML string, e.g. `"2"`, never numeric) |
| `source` | string | yes | `WorkflowDefinition.source` verbatim: the literal `"embedded"`, or the loaded workflow directory's path (`root.join(".sergeant/workflows").join(name)`, typically absolute) — the proposal's earlier `"source": "repository"` example did not match any real value and is corrected here |
| `content_hash` | string, 64-char lowercase hex | yes | `WorkflowDefinition.content_hash` — BLAKE3 over a canonical projection of name/version/stage identity; `source` is deliberately excluded from the hash |
| `status` | string | no — absent for the embedded entry | the workflow's own `index.md` front matter `status:` field, verbatim. Today this route only ever returns catalog-listed workflows, so the value is always `"published"` in practice (T2-39 excludes drafts before they reach this route); any other value observed here is a loader defect, not a UI state to design for |
| `description` | string | no — absent for the embedded entry | the workflow's own `index.md` front matter `description:` field, verbatim |
| `tags` | array of string | no — absent for the embedded entry (never an empty array standing in for "none") | the workflow's own `index.md` front matter `tags:` field, verbatim |
| `stages` | array of StageEntry, non-empty | yes | `WorkflowDefinition.stages`, in pinned execution order |

**`workflows[].stages[]` — StageEntry:**

| Field | Type | Required | Source |
|---|---|---|---|
| `id` | string | yes | `StageDefinition.id` (directory name, e.g. `"10-plan"`) |
| `kind` | `"actor"` \| `"execute"` | yes | `StageDefinition.kind` (`#[serde(rename_all = "snake_case")]`; TOML omission defaults to `"actor"`, but the resolved value is always present in the response) |
| `harness` | string or `null` | yes, nullable | `StageDefinition.harness` — `null` means "use the Work actor default" |
| `profile` | string or `null` | yes, nullable | `StageDefinition.profile` — `null` means "use the Work/profile default" |
| `requires_ask` | boolean | yes | `StageDefinition.requires_ask` (defaults `false`) |

Deliberately excluded from `StageEntry`: `context` (the stage's full `CONTEXT.md` prompt text — never surfaced by this or any current route) and `execute` (the pinned container spec: image/command/workdir/workspace_access/network/env, present internally only when `kind == "execute"`, currently only `NetworkPolicy::None` → `"none"` exists as a variant). §11.3's Detail view lists only stage order, stage kind, and declared harness/profile; §6.2 excludes a "generalized backend capability matrix." Neither field is needed by any T-Series UI surface today; both stay server-side until a concrete consumer requires them (§16.6's forward rule).

**Edge shapes, not errors:**

- `200` with `{"workflows": []}` when `cwd` resolves to a workspace with no admitted catalog and the embedded fallback itself fails to load (fails closed, per §19.4) — an empty array, not a `4xx`.
- `200` with a single-entry `{"workflows": [...]}` (the embedded `software-change` entry) when no repository catalog resolves but the embedded fallback loads.

**Errors**, in the same structured `error.name`/`error.detail`/`error.remedy` shape every other authenticated route already uses:

- `400` when `cwd` is missing, not percent-decodable, or not an absolute path.

No mutation. No event append (unchanged from the original minimum addition above).

## 11.3 Screen

```text
catalog list | workflow detail | usage/recent Work derived from loaded Fleet
```

Detail may show:

```text
name/version/source/status
description/tags
stage order
stage kind
declared harness/profile
content identity
recent Work using this workflow
```

It does not show a generalized backend capability matrix. Backend capabilities exist internally, but current public surfaces do not provide a complete capability/catalog read model to the TUI.

## 11.4 Actions

```text
Enter inspect
Use in New Work
@ filter/select
/ commands
Esc back
```

No workflow editing, validation, generation, or publication.

## 11.5 Live catalog versus pinned Work

**Decision T2-41 (R2):** Workflows shows what new Work can bind now. Canonical Work shows the workflow definition pinned when that Work bound. The two are labeled distinctly and never silently reconciled.

---

# 12. Estate

Estate is a full top-level destination.

```text
Estate
  Repositories
  Groups
  Health
```

**Decision T2-42 (R2):** Estate is first-class because estate topology and health are prerequisites to setting useful intent, not miscellaneous settings.

## 12.1 Repositories

The screen consumes current repo lifecycle semantics:

```text
sgt repo list
sgt repo add
sgt repo remove
```

List/detail may show only current manifest facts:

```text
name
path
origin when declared
instruction policy
group membership derived from current groups
present/valid result when current validation provides it
```

### Add

Fields:

```text
name
origin, optional only when repo already exists
instruction policy: local | suppress
```

The existing clone/register operation runs off the render loop. The TUI shows a spinner, current phase text where the operation itself supplies one, elapsed time, and final structured result. It does not fabricate clone percentage.

### Remove

Confirmation states:

```text
This removes the estate declaration.
It does not delete repos/<name> from disk.
```

Existing group-reference refusal remains authoritative.

**Decision T2-43 (R2/R6):** Reuse repo behavior exactly. The TUI never shells out to `sgt repo`, parses text, or reimplements manifest validation.

## 12.2 Groups

The screen consumes:

```text
sgt group list
sgt group add
sgt group remove
```

It supports:

- create group;
- extend existing membership;
- remove selected members;
- remove group;
- display/edit the brief only if current add semantics can safely replace it.

Every member must already be declared. Existing refusal text and remedies are preserved structurally.

**Decision T2-44 (R2/R6):** Group editing is full lifecycle parity with current CLI semantics, not a read-only viewer.

## 12.3 Health

Health renders the current Doctor check report:

```text
status
check name
summary/detail
remedy
```

It does not reinvent health policy.

Current checks include git, claude, environment, data directory, Docker, journal, projection, daemon, permission_mode, estate, and disk pressure. Filesystem reliability joins the report, per PR #111.

**Decision T2-45 (R2/R6):** Extract Doctor's structured `Check`/`Report` result from CLI formatting and let both CLI and TUI consume it.

Health is not a resource dashboard. Disk facts appear only when Doctor already measures them.

A selected failing/warning check receives one clear detail/remedy panel.

## 12.4 Retained state

The integration branch's retained/reap surfaces are merged:

- Health's disk-pressure detail may open a Retained Work overlay;
- Work Details may show the retained binding, path, reason, and byte count;
- the operator may preview exactly what Reap would delete;
- Reap requires explicit confirmation;
- retained branches remain outside the deletion path.

**Decision T2-46 (R2):** Retained-state UI is consumption of a real merged API, not a T-Series invention.

## 12.5 What Estate excludes

Estate does not expose:

```text
sgt init
daemon foreground or stop
harness passthrough
data-dir relocation
raw manifest editor
CPU/memory/process monitoring
```

---

# 13. Canonical Work Surface

## 13.1 Header

**Decision T2-47 (R2):** The Work header leads with:

```text
state + state label
intent
pinned workflow
current stage / total
attempt
current stage executor/profile where present
target repos/group-derived summary
turn envelope
connection truth
```

Full IDs, native session, route source, and paths stay in Details.

`needs_input` pins the current question above the fold.

## 13.2 Views

```text
Thread    Workflow    Evidence    Graph    Details
```

**Decision T2-48 (R2/R5):** There is one canonical representation. A peek overlay may summarize it, but it cannot expose a separate action vocabulary or become a second Work screen.

## 13.3 Thread

Thread combines two authoritative reads:

```text
GET /v1/work/{id}/transcript
GET /v1/events?work_id=<id>&limit=<bounded>
```

Transcript turns carry causal sequence and, per PR #111, visible journal timestamps. Event history supplies stage, execution, surface, envelope, and lifecycle system lines.

The two streams are merged by journal sequence where possible.

Default rendering:

| Fact | Rendering |
|---|---|
| Work intent | initial intent block |
| user/assistant transcript turn | conversation turn |
| actor-authored ask | primary gold request card |
| human response | human turn |
| workflow/stage transition | compact divider/system line |
| execution reservation/start/stop | compact lifecycle line |
| tool request/completion | paired collapsible row |
| waiting | muted card |
| blocked | warning card |
| failed | danger card |
| envelope extended | explicit budget system line |
| ceiling interrupted | explicit interruption line |
| output pointer | output card |
| completed | success outcome |
| completed_dirty | output-needs-review outcome |
| canceled | muted terminal outcome |
| unknown event | Evidence-only generic line |

**Decision T2-49 (R2):** Thread renders only journal-backed facts. It never displays chain-of-thought, guessed progress, inferred file changes, or process activity.

Transcript source markers such as recovered/interrupted raw archive remain visible. Timestamp display never computes a fake "now" relationship when only the original timestamp is authoritative.

## 13.4 Workflow

The pinned ordered stage list forms the rail.

```text
✓ plan
✓ context
⠹ implement  attempt 2
· verify
· close
```

Event history may prove earlier attempts and statuses. Future stages remain not entered.

Per-stage harness/profile appears only where the pinned definition supplies it.

**Decision T2-50 (R2):** Stage order is not duration. The rail never becomes a percent bar.

## 13.5 Evidence

Evidence is the raw Work-local journal window:

```text
seq
timestamp
kind
source
execution/correlation/causation where present
payload
```

It supports local filtering over loaded rows and bounded "load older". It does not become P2-JOURNAL.

## 13.6 Graph

The existing one-Work graph is rendered as a navigable relationship tree/list.

```text
Work
├─ targets -> Repository
├─ follows -> Workflow
├─ current -> Stage
├─ executed-by -> Execution
│  ├─ uses -> Backend
│  └─ bound-to -> Native session
└─ contains -> Message / ToolCall
```

Every relationship may expose source event sequence.

**Decision T2-51 (R1/R2):** No Canvas node graph. No File, Artifact, Commit, or Finding nodes unless current events and graph projection actually supply them.

## 13.7 Details

Details contains progressive-disclosure facts:

```text
Work id and origin
workspace/repos
reported and persisted state where relevant
workflow source/content identity
route source
reservation
execution/native session
surface bindings
teardown
output pointer
envelope
retained state
```

## 13.8 Output

Output card shows only current pointer facts:

```text
repository
source repo
retained branch
finalize commit
worktree path
teardown disposition
```

No file list or diff.

## 13.9 Envelope

```text
turns spawned / effective cap
bonus turns
per-turn ceiling
latest envelope extension evidence
```

`extend` and `retry` remain separate.

## 13.10 Action matrix

**Decision T2-52 (R2):** Advertise current semantic operations only.

| Reported state | Ordinary text | Actions |
|---|---|---|
| pending | disabled | cancel, inspect |
| active | disabled | cancel, inspect |
| waiting | disabled | retry, cancel, inspect |
| needs_input | answer | respond, cancel, inspect |
| blocked | disabled | retry, extend where relevant, cancel, inspect |
| failed | disabled | retry, cancel, inspect |
| completed | disabled | inspect output/evidence |
| completed_dirty | disabled | inspect output/retained state; reap |
| canceled | disabled | inspect |

The daemon is authoritative. Races return structured errors, preserve the screen, and trigger refresh.

`extend` never implicitly retries. `retry` never implicitly extends.

---

# 14. Attention and WATCH

WATCH defines a headless blocking contract over six meaningful states:

```text
needs_input
waiting
blocked
failed
completed
canceled
```

`pending` and `active` do not emit Watch notices.

**Decision T2-53 (R2):** Reuse this vocabulary to decide which transitions deserve in-cockpit attention, but do not run or emulate `sgt watch` inside the TUI.

Attention rules:

- `needs_input`: gold and counted in `? N`;
- `blocked`, `failed`, `completed_dirty`: warning/danger and counted in `! N`;
- `waiting`: visible but not urgent gold;
- `completed`, `canceled`: transient/bounded finished section;
- `pending`, `active`: ordinary running state.

An event may trigger a brief ephemeral banner after the authoritative reread. The banner is not persisted or acknowledged.

No pulsing bell. The static count and drawer contents are sufficient.

---

# 15. Interaction Grammar

## 15.1 Persistent composer

**Decision T2-54 (R2/R5):** The bottom region is visually stable and semantically contextual.

| Context | Label | Ordinary text |
|---|---|---|
| Home | `INTENT` | submit new Work |
| Work needs_input | `ANSWER` | respond to the current request |
| Other Work | `COMMAND` | disabled as actor input |
| Fleet | `FILTER` when focused | local Fleet filter |
| Workflows | `FILTER` when focused | local catalog filter |
| Estate lists | `FILTER` when focused | local list filter |

The composer never implies chat where no message operation exists.

## 15.2 Submission keys

```text
Enter          newline
Ctrl+Enter     submit when distinguishable
Tab            focus Send / Run / Confirm
Enter          activate focused action
Esc            leave focus, preserve draft
```

Blank input is refused. Draft clears only after accepted mutation.

## 15.3 Slash palette

**Decision T2-55 (R2/R5/R6):** `/` at the first non-whitespace position opens a fixed local palette.

Core vocabulary:

```text
/home
/fleet
/workflows
/estate
/back
/refresh
/answer
/retry
/extend
/cancel
/evidence
/graph
/details
/analytics
/retained
/reap
/help
/quit
```

Explicitly absent:

```text
/interrupt
/files
/web
/watch
/daemon
/init
/claude
/codex
/opencode
/goose
```

Slash elsewhere in prose is literal.

The palette is a Rust enum and match, not a shell or plugin language.

## 15.4 Workflow chooser

**Decision T2-56 (R2/R5/R6):**

- On Home, `@` selects the existing workflow request field and renders a chip outside durable intent.
- In Workflows, `@` focuses catalog selection.
- In a needs-input answer, `@name` inserts literal reference text only.
- In existing Work, it never rebinds procedure.

## 15.5 Confirmation

Durable and destructive actions require deliberate review:

- Run Work: deliberate send.
- Respond: deliberate send.
- Cancel: confirmation naming Work.
- Retry: confirmation naming stage/attempt.
- Extend: explicit added turns and resulting cap.
- Repo remove: state that directory is not deleted.
- Group remove: list affected members.
- Reap: preview exact paths/bytes and state that retained branch remains.

## 15.6 Help

`?` opens contextual help derived from the same fixed key/action table used by the footer. No configurable binding subsystem.

---

# 16. Data and Extraction Boundaries

## 16.1 Work client

Work remains on `ApiClient`.

Representative typed additions are convenience methods only:

```text
submit
fleet
work
transcript
events
respond
retry
extend
cancel
graph
analytics
workflow_catalog
retained
reap
```

## 16.2 Estate operations

Per §5.3/Decision T2-14, repo/group behavior reaches `tui.rs` exclusively through new authenticated `ApiClient` routes, thin daemon-side wrappers over the existing `crate::domain::manifest` functions the CLI already calls locally. No logic is duplicated; no CLI behavior changes.

Route contracts:

```text
GET /v1/estate/repos
  -> 200 { repos: [ { name, path, origin?, instructions? }, ... ] }

POST /v1/estate/repos
  body { name, origin?, instructions? }        (manifest::add_repo)
  -> 201 { name, path, origin?, instructions? }
  -> error body carries the same structured ManifestError name/detail/remedy add_repo returns

DELETE /v1/estate/repos/{name}
  (manifest::remove_repo)
  -> 204 on success
  -> error body carries the same structured ManifestError (e.g. RepoInUseByGroups) remove_repo returns

GET /v1/estate/groups
  -> 200 { groups: [ { name, repos: [...], brief? }, ... ] }

POST /v1/estate/groups
  body { name, repos: [...], brief? }           (manifest::add_group, mkdir-p semantics: creating an
                                                  existing group unions in new members; re-adding an
                                                  existing member is a no-op)
  -> 200 { name, repos: [...], brief? }

DELETE /v1/estate/groups/{name}
  body { repos?: [...] }                        (manifest::remove_group; omitted/empty repos removes
                                                  the whole group, otherwise removes just the named
                                                  members, each of which must already be a member)
  -> 204 on success
  -> error body carries the same structured ManifestError (e.g. NotAGroupMember) remove_group returns
```

CLI owns Clap/stdout/JSON/exit code, calling `crate::domain::manifest` directly and locally — unchanged, so `sgt repo add`/`sgt group add`/etc. keep working with no daemon running (needed for `sgt init` and pre-daemon diagnosis). TUI owns forms/focus/rendering, calling only these routes via `ApiClient`.

**Decision T2-57 (R2/R6):** Extract on contact. Do not build `ApplicationService`, `CommandBus`, a generic command trait, or a second internal API. The daemon route handlers are thin wrappers, not a new abstraction layer.

## 16.3 Doctor report

Per §5.3/Decision T2-14, Doctor's report reaches `tui.rs` exclusively through a new authenticated `ApiClient` route, a thin daemon-side wrapper over `mod doctor`'s existing `Check`-producing functions and `Report::to_json`, which the CLI already calls locally.

Route contract:

```text
GET /v1/doctor
  -> 200 the same Report::to_json() shape the CLI's `sgt doctor --json` already prints:
     {
       healthy: bool,
       data_dir: string,
       checks: [
         { name, status: "ok" | "warn" | "fail", detail, remedy? },
         ...
       ]
     }
```

The current CLI text and JSON are rendered from `doctor::Report` directly and locally — unchanged, so `sgt doctor` keeps working with no daemon running. TUI Health renders the same JSON shape, fetched via `ApiClient`.

**Decision T2-58 (R2/R6):** Shared result, one diagnostic implementation — the daemon route serializes the same `Report` the CLI prints, never a second computation of the checks.

## 16.4 Workflow catalog

Workflow discovery stays daemon-projected. `src/tui` never reads `.sergeant` directly.

## 16.5 Long local operations

Clone, Doctor, and any retained-state inspection/reap that performs blocking work run outside the render/event loop. The screen remains responsive and shows honest indeterminate activity.

**Decision T2-59 (R3/R5/R6):** Use existing Tokio blocking boundaries. Add no job system or progress protocol.

## 16.6 Forward rule

**Decision T2-60 (R2):** New domain behavior added after T-Series lives below presentation from the start. Existing behavior is extracted only when a second surface consumes it.

---

# 17. Connection, Loading, Empty, and Error States

## 17.1 Startup

`sgt tui` performs its current first authoritative read before terminal initialization. With no daemon, it refuses normally and names the remedy.

**Decision T2-61 (R2):** No offline shell, daemon auto-start, or TUI-only Doctor mode.

## 17.2 Live connection

Existing states remain:

```text
Attached
Reconnecting
AuthFailed
```

Rules already shipped under #16 remain binding:

- capped backoff;
- authoritative refresh before Attached;
- manual `r` attempt;
- authentication failure stops automatic retries;
- command status never overwrites connection truth.

During Reconnecting/AuthFailed:

- stale label is persistent;
- active spinners stop;
- writes are disabled or fail clearly;
- navigation and already-loaded evidence remain usable.

## 17.3 Loading

Each lazy region has a shaped textual loading state:

```text
Loading transcript...
Loading graph...
Running health checks...
Adding repository...
```

One spinner per active operation, not per empty panel.

## 17.4 Empty states

Examples:

```text
No Work yet
Describe an intent on Home.

No published workflows found
Exact-name submission still uses daemon resolution; inspect the catalog error.

No repositories declared
Use Estate / Repositories / Add.

No groups declared
Create one from declared repositories.

All checks healthy
No remedy required.

No conversation recorded
Use Evidence for lifecycle events.
```

## 17.5 Errors

Structured errors remain visible beside the action that caused them. Forms preserve input. Errors never disappear merely because focus changed.

Catalog failure, manifest failure, Doctor failure, transport failure, and mutation conflict remain distinct.

## 17.6 Terminal lifecycle

Existing guarantees remain:

- panic restoration;
- SIGTERM/SIGHUP restoration;
- dead PTY exit;
- bounded key-reader shutdown;
- initial-draw hangup handling;
- no idle busy loop.

Keyboard enhancement push/pop is added to the same lifecycle and mutation-probed.

## 17.7 Terminal too small

Below `80x24`, render a safe minimal notice with current dimensions and exit/back help. Never panic, overlap composer/footer, or corrupt raw mode.

---

# 18. Responsive Composition

**Decision T2-62 (R5):** Test and implement three compositions.

## 18.1 Wide: 150 columns and above

```text
drawer | primary body | contextual rail
```

Home may use Attention, New Work, and Recent Outputs simultaneously.

## 18.2 Medium: 100-149 columns

```text
optional drawer | primary body
```

Contextual rail moves below or into a selectable subview.

## 18.3 Narrow: 80-99 columns

```text
primary body only
drawer and context as overlays/full-body views
```

Home fields stack. Fleet uses two-line rows. Estate detail replaces list temporarily. Work tabs remain horizontally scrollable or collapse to a view chooser.

## 18.4 Height

At 24 rows:

- header and footer remain one row each;
- composer gets a bounded minimum;
- body scrolls;
- nonessential summaries disappear before primary state/question/actions.

No fixed 60/40 split survives regardless of content.

---

# 19. Testing and Validation

## 19.1 Philosophy

**Decision T2-63 (R2):** Continue Ratatui `TestBackend` semantic and geometry testing. Do not use brittle whole-frame golden snapshots as the primary contract.

Ratatui's own widget tests use `TestBackend` and buffer assertions, including narrow-area behavior. T-Series follows that pattern.

## 19.2 Pure state tests

Cover:

- attention grouping and counts;
- reported-state treatment;
- intent-first row projection;
- envelope ratio;
- stage rail;
- transcript/event merge by sequence;
- current question pinning;
- output card;
- action availability;
- navigation-stack restoration;
- workflow live-versus-pinned labeling;
- Estate list projections;
- visual token/glyph fallback.

## 19.3 Composer tests

Cover:

- insertion/newline/delete/cursor/wrap/paste;
- ordinary Enter never submits;
- Ctrl+Enter submits only with reported modifier;
- Send fallback;
- draft preservation;
- blank refusal;
- slash parser boundary;
- `@` semantics;
- dependency wrapper behavior if `ratatui-textarea` is adopted;
- no mouse path.

## 19.4 Workflow catalog tests

Retain and update the original proposal's contract:

- embedded fallback;
- indexed published workflow;
- drafts excluded;
- unindexed directory excluded;
- path traversal rejected;
- missing/malformed/disagreeing records fail closed;
- repository fallback override;
- cwd discovery matches submission;
- no event append;
- TUI remains endpoint-backed.

## 19.5 Estate parity tests

**Decision T2-64 (R2; L7):** For every repo/group operation, run the CLI (which calls `crate::domain::manifest` directly and locally) and the TUI (which calls the same `manifest` functions through the `/v1/estate/repos`/`/v1/estate/groups` routes via `ApiClient`, per §16.2) against equivalent fixtures and assert the same structured result and filesystem/manifest outcome.

Pin:

- add existing repo;
- clone new repo;
- instruction policy;
- remove declaration without deleting directory;
- group reference refusal;
- add/extend group;
- remove member/group;
- atomic write and lock failure;
- TUI cancellation before confirm causes no mutation.

## 19.6 Doctor parity

The CLI text/JSON renders `doctor::Report` directly and locally; TUI Health renders the same report's JSON, fetched via `ApiClient` from `GET /v1/doctor` (§16.3). Tests assert no check disappears or changes status/remedy between surfaces.

Filesystem reliability is included, per PR #111.

## 19.7 Live daemon tests

Using fake backend:

- Home submission;
- group expansion;
- workflow selection;
- transcript rendering;
- actor ask and multiline response;
- retry;
- extend without automatic retry;
- cancel confirmation;
- output pointer;
- completed_dirty;
- state race refresh;
- Watch-state attention transition.

## 19.8 Retained/reap tests

- retained list is API-backed;
- preview does not mutate or auto-spawn;
- reap requires confirmation;
- branch survives;
- exact per-binding result renders;
- Health disk detail reaches retained overlay.

## 19.9 Reconnect and lifecycle

Preserve all current #11/#16/#26 tests and add coverage for the new shell, drawer, editor, overlays, and keyboard enhancement cleanup.

## 19.10 Geometry matrix

Render every major surface at:

```text
80x24
120x36
180x48
```

Fixtures:

```text
Home empty/full/admission-paused
Fleet every state and long values
Work active/needs-input/blocked/failed/completed/completed-dirty
Workflow catalog/detail/error
Estate repos/groups/health
Attention open/closed/overlay
palette/chooser/confirmations
reconnecting/auth-failed
terminal-too-small
retained/reap
```

Assertions:

- no collision;
- question/state/action remain visible;
- composer/footer never overlap;
- focus is visible;
- long values remain contained;
- drawer collapses as contracted;
- no one-cell glyph violation;
- no workflow percent gauge;
- no active animation while stale.

## 19.11 Taste pre-flight

Before screenshots or close-out, mechanically audit:

```text
one theme
one focus accent
semantic color only
few full borders
no nested-box wallpaper
no fake data precision
no unsupported controls
no color-only states
no false workflow percentage
no duplicated Work representation
no stale web/resource/file surfaces
every empty/loading/error/confirmation state present
```

## 19.12 Gates

Each milestone runs the repository's current shipping procedure, not a hand-rolled stale copy. The gate defect in PR #111 must be resolved before its result is trusted.

---

# 20. Program Shape

This is one program with bounded slices. No slice waits for a universal refactor.

## 20.1 T0: Adjudication and revision freeze

T0 begins only once a proposal-grading gauntlet unit has closed against this document, e.g. T-SERIES-1.

Decision T2-14 (§5.3/§16.2/§16.3) was resolved by the proposal's owner on 2026-08-16: extend the daemon API with new authenticated routes for repo/group and Doctor, reached via `ApiClient` (§5.3). T0 no longer needs to make this decision — it is already made; T0's remaining task is unchanged:

- pin current main and integration disposition;
- re-audit CLI/API/TUI/workflow catalog;
- contract the workflow catalog route's response shape;
- spike `ratatui-textarea` dependency resolution;
- contract the visual token set and responsive geometries;
- write T1 only after rulings.

No product code.

### T0 record (run 2026-08-16)

**Main and integration disposition, pinned.** `gh pr view 111` and `gh pr view 126` both report `MERGED` into `main`: PR #111 at merge commit `3a46b87c17d249655708ed5ac32f6704738776cf` (2026-08-15T15:28:02Z), PR #126 at merge commit `0a3b5eb83367ce28ceab41088348344e08c19e30` (2026-08-16T02:03:26Z). `integration/t-series-2026-08-15`'s current head is a merge commit (`991e258`) whose second parent is `0a3b5eb` directly — `git merge-base --is-ancestor` confirms both merge commits are ancestors of the branch head. The branch is caught up to `main` as of the merge commit already on it; no action was needed.

**CLI/API/TUI/workflow-catalog re-audit: no drift found.** `git diff --stat 3a46b87..HEAD -- src/cli.rs src/api.rs src/tui.rs .sergeant/index.md` is empty — none of these four files changed at all between PR #111's merge and the current branch head, including across PR #126 (which touched `src/daemon.rs`, `src/platform/*`, `src/runtime/engine.rs`, `src/runtime/surface.rs`, and tests — perf/reliability work unrelated to the CLI/API/TUI surface this proposal audits). Every factual claim already in this proposal that names these four files was spot-checked directly against the current repository and holds: the Doctor check set (`git`, `claude`, `environment`, `data_dir`, `docker`, `journal`, `projection`, `daemon`, `permission_mode`, `estate`, `disk_pressure`, `filesystem`, all in `src/cli.rs`'s `mod doctor`) matches §12.3 exactly; `Command::Tui` and the `extend`/`retained`/`reap` routes exist in `src/api.rs` as described; `completed_dirty` is already a recognized state in both `src/watch.rs` and `src/tui.rs`; `.sergeant/index.md` still lists exactly 23 published workflows; `Cargo.lock` still pins `ratatui 0.30.2` with Crossterm reached only through `ratatui-crossterm` (§8.7's spike, below, is the first place this is proven rather than merely observed). Nothing new since PR #126 landed falls outside what the T-SERIES-1 gauntlet's assumptions axis already covered.

The workflow catalog route's schema (§11.2) and the `ratatui-textarea` spike (§8.7) are recorded in place in their own sections rather than duplicated here; §8's visual token table is likewise recorded in §8.10.

## 20.2 T1: Cockpit foundation and immediate Work value

- application shell and `Home / Fleet / Workflows / Estate` navigation;
- focus model, overlays, footer/help, Attention drawer;
- responsive Fleet;
- canonical Work shell;
- transcript-backed Thread;
- Workflow rail (§13.4);
- Evidence (§13.5);
- Graph (§13.6);
- Details (§13.7);
- output/envelope/completed-dirty;
- respond/retry/extend/cancel;
- multiline composer;
- preserve reconnect and terminal safety.

This produces visible value without Estate extraction or workflow-catalog API completion.

## 20.3 T2: Workflow discovery

- workflow catalog endpoint;
- Workflows list/detail;
- Home `@` chooser;
- live versus pinned workflow labeling;
- recent usage derived from loaded Fleet;
- catalog error/empty states.

## 20.4 T3: Estate

- build the `/v1/estate/repos` and `/v1/estate/groups` API routes (§16.2) as thin wrappers over `crate::domain::manifest`;
- Repositories full lifecycle, consumed via `ApiClient`;
- Groups full lifecycle, consumed via `ApiClient`;
- build the `/v1/doctor` API route (§16.3) as a thin wrapper over `mod doctor`;
- Health, consumed via `ApiClient`;
- retained/reap consumption;
- no generic service layer.

## 20.5 T4: Close-out and polish

- all responsive fixtures;
- visual pre-flight;
- repeated lifecycle/load/hygiene runs;
- real Ratatui screenshots;
- README and help updates;
- ledger/lessons/ADR/proposal supersession updates;
- explicit handoff to P2-JOURNAL.

T4 cannot close on acceptance item 57 (§21) until issue #120 is independently resolved, or T4 manually verifies a non-empty diff before trusting a shipping-gate `passed` result.

## 20.6 Parallel boundaries

P2-JOURNAL may proceed independently if it does not silently define T-Series navigation or modify the same files without coordination.

Interactive durable sessions require their own proposal and evidence. They are not a T-Series tail item.

---

# 21. Acceptance Contract

T-Series is complete when:

1. `sgt tui` opens the cockpit; bare `sgt` remains the static homepage.
2. Observation still never auto-spawns a daemon.
3. Top navigation is exactly Home, Fleet, Workflows, Estate.
4. No Journal, System, Explore, or Web placeholder appears.
5. Home can submit current Work request fields without becoming a second planner.
6. Repo/group target selection expands into current repository semantics.
7. Home does not invent model or capability catalogs.
8. Home uses deliberate multiline submission.
9. Home shows current attention, active Work, and pointer-backed recent outputs.
10. Fleet is complete, responsive, and intent-first.
11. `completed_dirty` is visibly review-required, not normal success.
12. Every Work entry opens the same canonical Work surface.
13. Work has Thread, Workflow, Evidence, Graph, Details.
14. Thread uses the authoritative transcript and journal-backed system events.
15. Thread exposes recovered/interrupted transcript provenance honestly.
16. No chain-of-thought, guessed progress, file change, or process inference appears.
17. Actor-authored questions are pinned and gold.
18. Ordinary Work text is accepted only for `needs_input`.
19. Respond retains current semantics.
20. Retry and extend remain separate.
21. Cancel remains deliberate.
22. Output displays pointer facts only.
23. Envelope consumption is the only progress gauge.
24. Workflow progression is ordinal only.
25. Graph contains only current proven nodes and edges.
26. Evidence is Work-local and never advertised as P2-JOURNAL.
27. Attention derives entirely from current Work state.
28. No Watch process or Watch screen exists inside TUI.
29. Workflows uses the root admitted catalog and embedded fallback.
30. Drafts and unindexed workflows are absent.
31. Live catalog and pinned Work procedure are distinct.
32. Estate is a full destination.
33. Repositories supports current list/add/remove semantics including clone.
34. Repository removal never deletes the directory.
35. Groups supports current add/extend/member removal/group removal semantics.
36. Health renders the same checks and remedies as `sgt doctor`.
37. Health does not invent CPU/memory/process monitoring.
38. `sgt init`, daemon administration, and harness passthrough remain outside.
39. Work daemon facts still enter only through ApiClient.
40. Estate/Doctor behavior is shared through small typed extractions, not duplicated.
41. No generic service layer, command bus, or full CLI rewrite lands.
42. The dashboard remains deleted and no browser control returns.
43. No mouse path is required or documented.
44. No Files/diff/artifact view appears without current API support.
45. `/` is a local fixed palette, not a new CLI grammar.
46. `@` selects/reference workflows without rebinding current Work.
47. Ctrl+Enter is an enhancement, not the only send route.
48. Keyboard enhancement flags are restored on every exit.
49. Reconnect and auth-failure truth remain visible and correct.
50. Active animation stops while state is stale.
51. Every major empty/loading/error/confirmation state exists.
52. Every major surface passes semantic/geometry tests at 80x24, 120x36, 180x48.
53. Existing PTY, signal, shutdown, idle-CPU, no-spawn, and reconnect tests remain green.
54. Retained/reap is consumed only through its real API and confirmation contract.
55. (Vacuous: PR #111 has merged, so the "no retained/reap placeholder if it does not land" branch this item described can no longer fire.)
56. The final screenshots come from the real TUI, not image-generation mockups.
57. The shipping gate actually executes and passes; a skipped false-green is failure.
58. The ledger records every amendment, R7, deferred finding, and integration disposition.

---

# 22. Ponytail Decision Register

The rung is the lowest viable resolution.

| Decision | Rung | Resolution |
|---|---:|---|
| T2-01 | R2 | Revise the existing proposal rather than create a competing program |
| T2-02 | R2 | Make the TUI a Work operator cockpit, not a process/system/harness replacement |
| T2-03 | R2 | Organize around operator questions of current Work |
| T2-04 | R2 | Grade the complete delegation loop |
| T2-05 | R1/R2 | Preserve sound old decisions and remove superseded premises |
| T2-06 | R1 | Resolved: T0 pins `main` post-merge, including PR #111's surfaces |
| T2-07 | R2 | Resolved: retained/reap is consumed, its real surface having landed |
| T2-08 | R2 | Use Taste as design audit, not web implementation authority |
| T2-09 | R1 | Reject military styling |
| T2-10 | R5 | Bind visual variance/motion/density |
| T2-11 | R5 | Reduce boxes before adding ornament |
| T2-12 | R2 | Work remains the durable center |
| T2-13 | R2 | Daemon-owned facts remain ApiClient-only |
| T2-14 | R2/R6 | Extend the daemon API for repo/group/Doctor; TUI stays ApiClient-only |
| T2-15 | R2 | SSE invalidates; authoritative reads decide |
| T2-16 | R2 | TUI remains no-auto-spawn |
| T2-17 | R1/R2 | Respond is not generalized into continuous chat |
| T2-18 | R2 | Never infer Work state from process liveness |
| T2-19 | R2 | Apply Ponytail in both overbuild and shortcut directions |
| T2-20 | R1 | Exclude adjacent feature work explicitly |
| T2-21 | R2/R5 | Use Home/Fleet/Workflows/Estate navigation |
| T2-22 | R2 | One canonical Work from every entry point |
| T2-23 | R2/R5 | Derive Attention from Fleet/Work state |
| T2-24 | R1/R5 | Use a fixed overlay set, not a modal framework |
| T2-25 | R5 | One dark semantic theme |
| T2-26 | R5 | Full border only for major focus regions |
| T2-27 | R5 | Use installed Ratatui primitives |
| T2-28 | R5 | Communicate state through text, glyph, color |
| T2-29 | R2/R6 | Animate attached active Work only |
| T2-30 | R1/R2/R5 | Gauge only actual envelope consumption |
| T2-31 | R7 | Prefer narrowly wrapped `ratatui-textarea`, dependency-tree gated |
| T2-32 | R4/R6 | Opportunistic enhanced keys plus universal Send fallback |
| T2-33 | R1 | No mouse |
| T2-34 | R2 | Home maps to current submission body |
| T2-35 | R1/R2 | No guessed model/capability selectors |
| T2-36 | R2 | Recent outputs require a real output pointer |
| T2-37 | R5 | Responsive Fleet Table/List, no fixed padding |
| T2-38 | R2 | Fleet filters only current fields |
| T2-39 | R2 | Catalog only admitted/indexed procedure and fallback |
| T2-40 | R2/R6/R7 | One minimum workflow-catalog endpoint |
| T2-41 | R2 | Separate live catalog and pinned Work workflow |
| T2-42 | R2 | Estate is first-class |
| T2-43 | R2/R6 | Reuse repo lifecycle through shared operations |
| T2-44 | R2/R6 | Full group lifecycle parity |
| T2-45 | R2/R6 | Doctor and Health consume one report |
| T2-46 | R2 | Retained UI consumes the now-merged surfaces |
| T2-47 | R2 | Intent/state/procedure/stage/envelope lead Work |
| T2-48 | R2/R5 | Thread/Workflow/Evidence/Graph/Details |
| T2-49 | R2 | Thread is journal-backed only |
| T2-50 | R2 | Workflow rail is ordinal |
| T2-51 | R1/R2 | Terminal graph is a proven relationship tree |
| T2-52 | R2 | Advertise current actions only |
| T2-53 | R2 | Reuse WATCH vocabulary without embedding Watch |
| T2-54 | R2/R5 | Persistent context-aware composer |
| T2-55 | R2/R5/R6 | Fixed local slash palette |
| T2-56 | R2/R5/R6 | Context-aware workflow chooser/reference |
| T2-57 | R2/R6 | Extract Estate behavior on contact only |
| T2-58 | R2/R6 | Extract one structured Doctor result |
| T2-59 | R3/R5/R6 | Keep blocking local effects off render loop |
| T2-60 | R2 | New behavior lives below presentation going forward |
| T2-61 | R2 | No offline TUI exception |
| T2-62 | R5 | Three responsive compositions |
| T2-63 | R2 | TestBackend semantics and geometry over whole-frame goldens |
| T2-64 | R2 | Pin CLI/TUI Estate parity and mutation behavior |
| T2-65 | R5 | Lock §8's color roles, spacing scale, and §18's breakpoint geometries as concrete values |

Any implementation decision not represented here is logged in the milestone report. Every new R7 names failed lower rungs.

---

# 23. Dispositions

## 23.1 Adopted from the previous proposal

- Work-centered Home/Fleet/Workflows concept.
- canonical Work surface.
- Attention drawer.
- deliberate multiline composer.
- local slash palette.
- workflow chooser/reference.
- endpoint-backed workflow catalog.
- ordinal workflow rail.
- Work-local Evidence and graph.
- separate Journal proposal.
- API invalidation discipline.
- TestBackend semantic testing.

## 23.2 Revised

| Previous decision | Revision |
|---|---|
| Bare `sgt` is the TUI | `sgt tui` is explicit; bare `sgt` remains homepage |
| Home/Fleet/Workflows | Estate added as full destination |
| Doctor CLI-only | Shared Doctor report powers Estate/Health |
| Repo/group not addressed by the predecessor | Full current lifecycle included through extract-on-contact |
| Local custom editor | Maintained text-area dependency preferred under a hard compatibility gate |
| Submit/respond/retry/cancel | Extend added and kept distinct |
| Event-derived conversation | Authoritative transcript becomes Thread backbone |
| Web disabled but retained | Web already deleted; all web proposal text removed |
| #11/#16 owned fixes | Those fixes are shipped guarantees to preserve |
| no output/envelope UI | Both are current Work facts and primary |
| completed only | completed_dirty is a separate operator-facing condition |

## 23.3 Rejected alternatives

### Full CLI service refactor

Rejected. It would force unrelated daemon, observation, static homepage, local manifest, Doctor, and exec-replacement command families into one abstraction and likely stall the TUI on regression work.

### New daemon APIs for repo/group/Doctor

Rejected. They would distort local/no-daemon semantics. Share the existing local implementation instead.

### TUI shelling out to `sgt`

Rejected. Text parsing creates drift and loses structured errors.

### Read-only Estate

Rejected. It would immediately send operators back to the CLI for ordinary management despite mature semantics already existing.

### System/resource dashboard

Rejected. CPU/memory/process accounting is unowned feature work. Doctor's existing disk facts remain valid.

### Embedded harness/PTTY

Rejected. ADR 0006 deliberately chooses `exec`, never supervise. Interactive durable sessions need separate domain and authority decisions.

### Active-turn chat/guidance

Rejected. Current `respond` is an answer to `needs_input`, not a general message contract.

### Separate Watch screen

Rejected. The TUI already consumes SSE. Watch supplies the attention vocabulary for headless clients.

### Files/diff/artifacts

Rejected. The current output pointer does not supply their content.

### Spatial graph canvas

Rejected. Canvas availability does not make it the correct terminal interaction.

### Mouse

Rejected. Keyboard-first is complete and easier to test.

### Web parity

Rejected. The dashboard was deleted by owner ruling.

### Capability-driven model selector now

Rejected. Capability types exist internally, but no complete current public catalog supports such a control. Future seam only.

### Local workflow filesystem scan from TUI

Rejected. Executable procedure remains endpoint-projected through the same loader the daemon trusts.

### Quick View as a second Work representation

Rejected. Any peek is a summary of canonical Work and cannot have independent actions.

## 23.4 Deferred, not rejected

- P2-JOURNAL and future Journal destination.
- durable human-initiated turns against existing Work.
- human attach where adapters expose and publicize it.
- capability-aware backend/model/profile chooser after a trustworthy read model exists.
- richer output browsing after real artifact/file/change events and APIs exist.
- browser client after a future owner ruling.

---

# 24. Falsifiers and Source Map

## 24.1 Sharp falsifiers

The proposal is violated if any implementation:

1. launches the TUI from bare `sgt`;
2. auto-spawns a daemon merely to open the TUI;
3. reads daemon Work facts directly from journal/filesystem;
4. duplicates repo/group validation in `src/tui`;
5. changes CLI repo/group outcomes while claiming presentation-only extraction;
6. builds a generic service/command framework before a second concrete consumer requires it;
7. accepts ordinary text on active Work;
8. makes extend retry automatically;
9. shows workflow percentage completion;
10. animates stale Work while reconnecting;
11. shows a model selector without a current source of valid models;
12. displays Files/diffs/artifacts from output-pointer metadata alone;
13. represents `completed_dirty` as green completion;
14. stores read/unread/dismissed attention state;
15. starts `sgt watch` from the TUI;
16. brings back Web/System/Explore placeholders;
17. requires Ctrl+Enter with no portable Send fallback;
18. fails to pop keyboard enhancement flags on an exit path;
19. lets Health disagree with `sgt doctor`;
20. deletes a repository directory on declaration removal;
21. claims a fact beyond what PR #111 actually shipped;
22. trusts a shipping gate that skipped its stages;
23. ships screenshots generated from design-image tooling rather than the application;
24. passes only because a whole-frame snapshot was updated to match a regression.

## 24.2 Primary repository sources

- [North Star](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/NORTH-STAR.md)
- [Agent operating contract](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/AGENTS.md)
- [Development rules](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/DEVELOPMENT.md)
- [Current CLI](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/cli.rs)
- [Current API](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/api.rs)
- [Current TUI](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/tui.rs)
- [Backend capabilities](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/src/backend/mod.rs)
- [Workflow catalog](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/.sergeant/index.md)
- [WATCH contract](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/gauntlet/contracts/WATCH.md)
- [ADR 0006 harness passthrough](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/docs/adr/0006-harness-passthrough.md)
- [Existing T-Series proposal](https://github.com/miztertea/sergeant-rs/blob/242abe3c4a889c2b666c7ce34b32812dd1ee8d61/reference/proposal-tui-t-series.md)
- [Merged integration PR #111](https://github.com/miztertea/sergeant-rs/pull/111)

## 24.3 Design and implementation references

- [Taste skill, pinned](https://github.com/Leonxlnx/taste-skill/blob/e988add20dab0fa97d7a76781c48961c8184288e/skills/taste-skill/SKILL.md)
- [Ratatui 0.30.2](https://docs.rs/ratatui/0.30.2/ratatui/)
- [Ratatui widgets](https://docs.rs/ratatui/0.30.2/ratatui/widgets/)
- [Ratatui LineGauge](https://docs.rs/ratatui/0.30.2/ratatui/widgets/struct.LineGauge.html)
- [Ratatui TestBackend](https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.TestBackend.html)
- [ratatui-textarea](https://docs.rs/ratatui-textarea/latest/ratatui_textarea/)
- [Crossterm KeyEvent](https://docs.rs/crossterm/0.29.0/crossterm/event/struct.KeyEvent.html)
- [Crossterm keyboard enhancement](https://docs.rs/crossterm/0.29.0/crossterm/event/struct.PushKeyboardEnhancementFlags.html)
- [Work-Centered Intelligence](https://app.notion.com/p/3ac27ada618f81728a73fbd7ac90c61c)
- [WorkPacket](https://app.notion.com/p/39a27ada618f818cba42f5efe8ffe1f0)
- [Intelligent Work Environments Research Map](https://app.notion.com/p/3ac27ada618f817b8418e50151dd7015)

---

# 25. Closing Ruling

The original T-Series correctly recognized that Sergeant's interface should resemble a modern agent harness while remaining Work-centered. The shipped MVP makes that design substantially more powerful and substantially more concrete.

The revised center is:

> **`sgt tui` is the gorgeous, modern operator cockpit for durable Work and the estate that makes the Work possible.**

It sets intent. It shows attention. It opens one canonical Work. It lets the operator answer, retry, extend, cancel, collect output, and understand evidence. It manages repositories and groups through existing validated semantics. It renders Doctor as Health. It remains honest about the boundaries it does not own.

It reaches that destination by reusing the product that already exists, not by pausing to build a new internal platform first.
