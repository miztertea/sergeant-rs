---
type: proposal
title: "Sergeant-rs Next Iteration: Measured ICM Workflows and Portable Execution"
description: >-
  Proposal to turn Sergeant's proven durable execution substrate into a measured
  ICM workflow system, beginning with a repository-to-workflow workflow and the
  original Sergeant corpus, then adding per-stage harness selection and
  Docker-backed deterministic execution without weakening the journal-first core.
status: proposed
resource: sergeant-rs
tags:
  - sergeant-rs
  - icm
  - workflows
  - docker
  - harnesses
  - cross-platform
  - proposal
timestamp: 2026-08-10
repository: https://github.com/miztertea/sergeant-rs
audit_revision: 27c00ef7cc9136400b4881974399d834fdce0a47
relationship: >-
  Successor/addendum to reference/proposal-depot-rust-execution-surface.md.
  It preserves that proposal's P0 invariants and extends the workflow boundary.
---

# Sergeant-rs Next Iteration
## Measured ICM Workflows and Portable Execution

**Status:** Proposed  
**Audit basis:** [`miztertea/sergeant-rs@27c00ef`](https://github.com/miztertea/sergeant-rs/tree/27c00ef7cc9136400b4881974399d834fdce0a47)  
**Relationship to P0:** Extension, not replacement  
**Primary objective:** Prove that real organizational procedure can be expressed as durable ICM workflows before generalizing the engine  
**Primary full-runtime contract:** The user's Git, the user's Docker Engine, and one or more user-authenticated agent harnesses  

---

# 1. Executive Summary

Sergeant-rs has already solved the difficult substrate problem.

It owns durable Work, isolated Git worktrees, native agent sessions, explicit stage transitions, crash-tolerant journaling, recovery, analytics, graph projection, and equal API-backed clients. The P0 architecture is not a toy workflow runner. It is a credible local execution surface whose strongest properties are exactly the ones the next iteration must preserve: the journal is truth; Work state is not process state; ambiguity fails closed; procedure is data; native harnesses retain their own identity and authentication; and all read models remain disposable projections.

The next iteration should **not** begin by building a generalized workflow language.

It should begin by using the workflow system that exists today to answer a harder question:

> Can Sergeant reconstruct the procedural knowledge of an arbitrary repository into reviewable draft ICM workflows?

The first self-hosting workload should therefore be a `repo-to-icm` workflow. It will accept a repository as its subject, inventory its behavioral artifacts, extract normalized behavior units, classify each unit through a Ponytail-derived ICM decomposition ladder, synthesize candidate workflows, and emit draft workflow packages plus an evidence-backed grammar-pressure report.

Before that workflow is trusted, the vendored snapshot of [`callmeradical/sergeant`](https://github.com/miztertea/sergeant-rs/tree/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream) should be decomposed manually into an adjudicated reference corpus. The generator's maiden voyage is then a measurement against that reference—not an ungrounded demonstration where the generator grades itself.

This first arc can run on the current engine:

```text
current ordered actor stages
        +
repository-local workflow content
        +
shared context/helper conventions
        +
fresh execution per stage
        +
Git-tracked draft artifacts
```

That experiment will reveal which behaviors fit the existing grammar and which do not.

Only after that evidence exists should Sergeant extend the runtime with the minimum additional stage semantics that this conversation has identified:

```text
ACTOR STAGE
  A selected native harness acts on behalf of the user.
  It inherits the user's harness authentication, repository trust,
  Git configuration, and organization-specific tools.

EXECUTE STAGE
  A declared operation runs in a declared container image through
  the user's local Docker Engine.
  No implicit host shell, ambient harness state, or organization-specific
  integration is assumed.
```

An actor stage may use Claude, Codex, OpenCode, Goose, or another measured harness. Different stages in one workflow may select different harnesses. An actor can invoke the user's `git`, `gh`, Jira CLI, cloud CLI, or any other tool available in the user's environment because the harness is acting as that user. Sergeant does not need first-class knowledge of GitHub, GitLab, Jira, ServiceNow, or any other organization-specific system.

An execute stage is different. It describes a reproducible computation: image, argv, working directory, mounts, environment policy, network policy, and expected exit semantics. Docker supplies the execution environment. Sergeant records both the requested image reference and the immutable image identity and platform that actually ran.

The resulting boundary is intentionally small:

```text
Sergeant owns     durable orchestration, stage state, evidence, recovery
Git owns          source history and worktree mechanics
Docker owns       isolated deterministic execution
Harnesses own     reasoning loops, native sessions, user authentication
The repository owns procedure, organization-specific instructions and tools
```

The proposal recommends five ordered outcomes:

1. Complete the current performance/remediation line where it intersects the next engine revision.
2. Establish the ICM filesystem conventions and adjudicated Sergeant reference decomposition with no engine change.
3. Build and measure the actor-only `repo-to-icm` workflow on the current runtime.
4. Add per-stage actor selection and Docker-backed execute stages through a two-phase, journal-first execution boundary.
5. Add true workflow composition only if the measurements prove that shared context and helpers cannot preserve the required durable boundaries.

The central rule is:

> **Do not add workflow machinery because it is imaginable. Add it only when a real procedure cannot be represented faithfully by the lower-rung forms.**

---

# 2. Audit Basis and Method

This proposal is based on a read-only audit of the repository at commit [`27c00ef7cc9136400b4881974399d834fdce0a47`](https://github.com/miztertea/sergeant-rs/commit/27c00ef7cc9136400b4881974399d834fdce0a47). Main advanced during the audit to include the P1 performance baseline and Bug Sprint 1, so the revision is recorded explicitly rather than referring vaguely to “current main.”

The review included:

- the complete original proposal, [`reference/proposal-depot-rust-execution-surface.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/proposal-depot-rust-execution-surface.md);
- the P0 milestone contracts, especially [`M3`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/gauntlet/contracts/M3.md), [`M4`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/gauntlet/contracts/M4.md), and [`M6`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/gauntlet/contracts/M6.md);
- the current workflow, engine, backend, recovery, journal, projection, surface, API, CLI, analytics, graph, telemetry, and Claude adapter implementations;
- the append-only [`GAUNTLET.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/GAUNTLET.md) and binding [`LESSONS.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/LESSONS.md);
- the completed [`P1-PERF` contract](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/gauntlet/contracts/P1-PERF.md) and [`baseline-2026-08-10.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/perf/baseline-2026-08-10.md);
- the current open issue backlog, including [#4](https://github.com/miztertea/sergeant-rs/issues/4), [#6](https://github.com/miztertea/sergeant-rs/issues/6), [#7](https://github.com/miztertea/sergeant-rs/issues/7), [#10](https://github.com/miztertea/sergeant-rs/issues/10), [#14](https://github.com/miztertea/sergeant-rs/issues/14), [#18](https://github.com/miztertea/sergeant-rs/issues/18), [#19](https://github.com/miztertea/sergeant-rs/issues/19), [#20](https://github.com/miztertea/sergeant-rs/issues/20), and [#25](https://github.com/miztertea/sergeant-rs/issues/25);
- the vendored original Sergeant corpus under [`reference/sergeant-upstream`](https://github.com/miztertea/sergeant-rs/tree/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream), including `AGENTS.md`, `no-mistakes`, `diagnosing-bugs`, `prototype`, `dispatch`, `cross-repo-work`, `load-project`, and `sergeant-setup`;
- the full design conversation that produced this proposal, including the decision to defer Markdown-to-JavaScript compilation, distinguish stages from helper commands, use old Sergeant as a measured corpus, and converge the host contract on Git, Docker, and native harnesses.

The evidence hierarchy used here is:

```text
current implementation
        ↓
committed contracts, ledger, lessons and measured baselines
        ↓
vendored source corpus
        ↓
official external documentation and project-owned crate documentation
        ↓
proposal and hypothesis
```

Where this proposal infers a future design from current code, it says so. Where a behavior must be measured against an installed harness or a real platform, the proposal preserves Sergeant's existing rule: documentation can nominate a capability, but only measurement may mark it supported.

---

# 3. The Current System, as Implemented

## 3.1 The daemon is already the application

The original proposal's central boundary survived implementation: the daemon owns runtime state and clients project it through one loopback HTTP/SSE API. The CLI, TUI, and embedded dashboard do not own private state. The daemon holds the journal, in-memory registry, analytics projection, backend registry, and active execution handles.

The current [`Core`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/api.rs) serializes authoritative mutation through one journal-first commit path:

```text
validate transition
        ↓
append and fsync event
        ↓
fold in-memory projection
        ↓
update disposable analytics/graph projection
        ↓
broadcast live event
```

This is the correct center of gravity for the next iteration. Docker and additional harnesses should enter through this same event contract rather than creating side-state beside it.

## 3.2 Work is durable intent; stage is an orthogonal coordinate

[`WorkState`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/domain/work.rs) remains intentionally small:

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

The active workflow stage is not encoded as another Work state. It is stored separately in [`WorkRun`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/projection.rs). This means the next iteration can add actor and execute stage classes without multiplying top-level states into combinations such as `validating`, `reviewing`, or `waiting_during_release`.

The transition table also encodes a crucial recovery policy: completed and canceled Work are absorbing, while failed Work may be explicitly retried. No later process observation is allowed to resurrect terminal intent.

## 3.3 Workflows are pinned filesystem procedure

The current workflow model in [`src/domain/workflow.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/domain/workflow.rs) is deliberately small:

```text
.sergeant/workflows/<name>/
  workflow.toml
  <stage-id>/CONTEXT.md
```

`workflow.toml` supplies name, version, and ordered stage IDs. Each `CONTEXT.md` is loaded verbatim into a `StageDefinition`. The complete resolved `WorkflowDefinition`, including stage context text, is journaled in `workflow.bound` before execution.

That gives the runtime a real version boundary:

```text
Git filesystem content       authoring source
resolved WorkflowDefinition  pinned runtime procedure
journal event                historical evidence
```

Editing a workflow after Work starts cannot retroactively alter that Work. This must remain true when executor metadata and container image references are added.

The repository-local workflow wins over the built-in default. The embedded `software-change` workflow is parsed through the same model rather than existing as a second code-shaped definition. It is appropriately a skeleton/reference implementation, not the complete behavioral system.

## 3.4 Every current stage is an actor stage

The key current limitation is structural, not accidental.

[`Engine::enter_stage`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/engine.rs) always:

1. reads the pinned stage context;
2. resolves the one backend bound to the Work;
3. builds a `StartRequest` carrying intent, stage, attempt, cwd, model, profile, and context;
4. calls the native [`Backend`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/backend/mod.rs) trait;
5. journals one backend-owned `ExecutionRecord`.

The backend trait is explicitly a native harness lifecycle:

```text
probe
start
send
observe
interrupt
resume
history
stop
```

Its `BackendSignal` is the only semantic input allowed to advance a stage. Native process liveness is separate evidence. That contract fits Claude, Codex, OpenCode, Goose, and similar harnesses. It should not be overloaded to make a Docker container pretend to be a conversational agent.

## 3.5 One backend and profile are currently pinned for the whole Work

[`StartPlan`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/engine.rs), [`Route`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/router.rs), and `WorkRun` all assume one selected backend and one optional profile for the entire workflow run.

Routing precedence is:

```text
explicit backend
      ↓
origin-client affinity
      ↓
workspace default
      ↓
global default
      ↓
fail with available options
```

A tier that names an unavailable harness fails rather than silently falling through to another provider. Per-stage harness selection should preserve that refusal. The future change is not “route differently”; it is “treat the current Work route as the default for actor stages that do not declare their own harness.”

## 3.6 Git work surfaces are host-native by design

[`runtime/surface.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/surface.rs) shells out to the installed Git CLI with structured arguments and creates host-visible worktrees under the Sergeant data directory. The native harness then executes in that host worktree.

This is an important boundary. A long-lived Git worker container would create another filesystem namespace around worktree metadata while the user-facing harness still operates on the host. Reducing a dependency count from “Git plus Docker” to “Docker” would not reduce the real system; it would hide Git behind a more complex path-translation and identity boundary.

Git should remain a host prerequisite and Sergeant-owned mechanic.

## 3.7 The Claude adapter proves the right harness model—and exposes the right next refactor

[`src/backend/claude.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/backend/claude.rs) is a strong concrete adapter:

- it uses structured argv rather than shell strings;
- it creates a durable Claude session identity;
- it drives headless print-mode turns;
- it resumes the same native conversation across processes;
- it captures raw stream JSON and normalized conversation/tool/usage events;
- it verifies model pins from observed result fields;
- it gates supported behavior by measured CLI version and flags;
- it fails closed when recovery evidence is ambiguous.

It also documents a real two-phase-start gap: the session ID exists before launch, but the process may start before `execution.started` is journaled. A daemon crash in that window can leave an unjournaled native process. The adapter also uses Linux `/proc` scanning to infer turn liveness across restart and returns “unknowable” on non-Linux platforms.

Those are not reasons to discard the adapter. They identify the exact execution boundary the next engine revision should repair for every executor:

```text
reserve durable external identity
        ↓
journal reservation
        ↓
perform external start
        ↓
journal observed start/result
```

## 3.8 Recovery already prefers evidence over optimism

[`runtime/recovery.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/recovery.rs) reconciles active Work before the daemon begins serving clients. It reattaches native contexts where the adapter supports it, classifies what it can observe, blocks ambiguity, repairs crashed starts into explicit blocked Work, and now sweeps residual surfaces from terminal Work.

The Docker execution adapter should follow the same model. Container IDs and Sergeant-owned Docker labels are better restart evidence than host PID scans because the Docker daemon itself owns and reports that lifecycle.

## 3.9 The journal and projections can absorb richer execution events

The segmented NDJSON [`Journal`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/journal.rs) is forward-compatible at the event layer. The in-memory reducer, graph, and analytics folds deliberately ignore unknown event kinds rather than bricking replay.

The [`Analytics`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/analytics.rs) and [`Graph`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/graph.rs) projections are rebuilt from the journal. They can therefore add executor kind, image identity, container lifecycle, and per-stage harness dimensions without migrating authoritative state.

The graph's provenance rule—every edge points back to the journal sequence that justifies it—is especially valuable for generated workflows and later workflow telemetry.

## 3.10 The performance line constrains the next implementation

The [`2026-08-10 P1 baseline`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/perf/baseline-2026-08-10.md) found:

- a single-writer throughput plateau and latency growth under submission load;
- unbounded retained in-memory Work/run state;
- queueing amplification for graph reads;
- cold analytical query scaling problems;
- strong journal rebuild throughput;
- clean SSE fan-out behavior;
- zero state corruption across the measured crash/restart cycles;
- a critical TUI process-lifetime defect that Bug Sprint 1 subsequently repaired.

The next iteration must not put image pulls, container waits, log drains, or adapter-thread joins under the same authoritative core lock. Open issue [#14](https://github.com/miztertea/sergeant-rs/issues/14) already names the first required seam: the next backend trait revision should move archive-thread joining outside the core lock. Docker execution makes that refactor mandatory rather than optional.

---

# 4. Invariants the Next Iteration Must Preserve

The proposal extends workflow expressiveness only if the following remain true.

## 4.1 The journal is the only durable runtime truth

Container state, harness state, process handles, cached workflow indexes, and Docker image metadata are observations or referenced evidence. No executor may create an authoritative side database.

## 4.2 Work state remains separate from execution state

A running container does not imply active Work. A dead container does not by itself imply failed Work until the execute-stage contract classifies its exit. A live harness process does not override a canceled Work. The engine continues to move Work only through explicit, journaled semantic outcomes.

## 4.3 Ambiguity fails closed

Unknown container outcome, missing native session, uncertain user-command effect, mismatched image identity, or unclassifiable restart evidence lands in `blocked` with the evidence that made it ambiguous. Sergeant never guesses success.

## 4.4 Procedure remains data

Workflow structure, actor contexts, executor specifications, catalog metadata, and shared references live in versioned repository content. Rust implements mechanics, not organization-specific procedure.

## 4.5 Native harnesses retain native identity and authentication

Sergeant uses the user's installed, authenticated, trusted harnesses. It does not copy OAuth tokens, become a provider gateway, or silently substitute one commercial provider for another.

## 4.6 Docker execution does not become a shell abstraction

Sergeant supplies image, argv, cwd, environment, mounts, and policy to the Docker Engine API. It never implicitly wraps an execute stage in `bash -c`, `cmd.exe /C`, PowerShell, or the host's default shell.

## 4.7 Git remains Git

Sergeant continues to use the user's installed Git for source history and host worktrees. It does not assume GitHub, and it does not containerize Git merely to claim a smaller prerequisite list.

## 4.8 Every advertised capability is measured

The existing `LESSONS.md` rules continue to bind:

- harness behavior is measured, not inferred from docs;
- every capability flag has a contract test;
- every fix has a test that fails when the fix is reverted;
- adjacent-append crash windows are explicitly designed and probed;
- verifiers mutate only disposable copies;
- accepted review rulings remain reviewable evidence.

## 4.9 Projections remain disposable

Workflow search indexes, DuckDB telemetry, graph edges, and UI summaries are rebuilt from authored files plus journal events. No future workflow catalog should become another source of truth.

---

# 5. The Actual Missing Layer

The current `software-change` workflow is enough to prove the runtime but not enough to carry Sergeant's accumulated operating knowledge.

The original Sergeant distributes procedure across:

```text
AGENTS.md
skills and repo-scoped skills
shell scripts and command wrappers
hooks
procedural documentation
behavioral tests
state conventions
review and escalation rules
```

That content should not be translated file-for-file.

A skill file is not automatically a workflow. A script is not automatically a stage. An `AGENTS.md` heading is not automatically a context file. Tests may encode more authoritative behavior than documentation. Several source files may collectively express one procedure, while one large source file may mix permanent invariants, reusable workflows, helper mechanics, and obsolete implementation detail.

The missing layer is therefore a method for reconstructing **behavioral intent** and assigning each behavior to the lowest faithful ICM form.

The first product of this next iteration is that method, encoded as a workflow and measured against a real corpus.

---

# 6. The ICM Decomposition Ladder

The decomposition ladder adapts the IdeaOS [Ponytail Minimality Ladder](https://app.notion.com/p/39a27ada618f8100babadb321a70de9b) and the Bashful principle that the operation describes what should happen while an execution adapter decides where it runs. Bashful's current adapter boundary is documented in IdeaOS as [Swappable Agent Execution Adapters](https://app.notion.com/p/39a27ada618f8157a9a6c54d56444357?pvs=204), with the canonical implementation reference at [`miztertea/bashful@877f9d5`](https://github.com/miztertea/bashful/blob/877f9d5b4d93e85bd02b51ff562efde6188a212b/src/bashful/adapter.py).

For every extracted behavior, ask these questions in order.

## 6.1 Is it a stable operating invariant?

Does the rule apply broadly and change rarely, independent of one procedure's current stage?

Examples:

- use Sergeant for durable substantive work;
- do not silently substitute a harness;
- search the workflow catalog before inventing a procedure;
- preserve source history and authority boundaries.

**Representation:** `AGENTS.md` or another stable repository instruction surface.

## 6.2 Is it a reusable procedural outcome?

Does it have a recognizable trigger, bounded outcome, and completion condition that could be invoked independently?

Examples:

- diagnose a defect;
- prepare a prototype;
- load project context;
- perform an adversarial review;
- validate and ship a change.

**Representation:** workflow.

## 6.3 Is it a meaningful durable checkpoint inside that procedure?

Would operators care that the work entered, blocked in, retried, completed, or failed at this boundary? Does a fresh execution context matter? Should its time, cost, evidence, and failure rate be measurable independently?

**Representation:** stage.

A script does not become a stage merely because it is executable. A stage is a semantic checkpoint.

A useful test is:

> If this script were replaced tomorrow by another implementation, would the procedural checkpoint still exist?

If `release-verification` remains a meaningful boundary whether it is implemented by Bash, Python, GitHub Actions, or three commands, it is a stage. If `test.sh` is merely one tool an implementation actor uses before declaring implementation complete, it is a helper.

## 6.4 Does the checkpoint require judgment?

Does an actor need to inspect evidence, choose among alternatives, ask the user, modify work, or explain a decision?

**Representation:** actor stage with `CONTEXT.md`.

## 6.5 Is it deterministic machinery used while crossing a checkpoint?

Does it perform a repeatable operation whose invocation is subordinate to the stage outcome?

Examples:

- collect an inventory;
- validate a schema;
- run a repository-specific test command;
- normalize JSON;
- produce a diff summary.

**Representation:** helper script or executable referenced by the stage context.

## 6.6 Is the helper or context reused?

If it belongs to one workflow, keep it local. If several workflows use it with the same contract, place it under `.sergeant/common/`.

**Representation:** workflow-local helper/context or shared helper/context.

## 6.7 Does Sergeant itself need to own a new durable fact?

Can the behavior not be represented faithfully because the runtime—not the actor—must own ordering, identity, retry, recovery, authorization, isolation, or evidence semantics?

**Representation:** engine-gap finding.

An engine-gap claim must contain:

```text
behavior that cannot be represented
source evidence requiring it
lower-rung representations attempted
why each lower rung fails
minimum runtime capability required
observable acceptance test
```

“Would be convenient” and “could be more elegant” are not engine-gap evidence.

---

# 7. Phase A: What Can Be Done with the Current Engine

The first phase intentionally requires no workflow-engine change.

## 7.1 Repository layout

The following filesystem can exist today because the current loader reads only the declared workflow file and stage contexts; extra catalog, draft, helper, and shared-context files are ordinary Git content.

```text
.sergeant/
├── index.md
├── common/
│   ├── contexts/
│   │   ├── adversarial-review.md
│   │   └── evidence-policy.md
│   ├── scripts/
│   │   ├── inventory-repository.py
│   │   └── validate-draft-workflows.py
│   └── templates/
│       └── workflow-index.md
├── workflows/
│   ├── repo-to-icm/
│   │   ├── workflow.toml
│   │   ├── index.md
│   │   ├── scripts/
│   │   ├── 00-contract/CONTEXT.md
│   │   ├── 10-inventory/CONTEXT.md
│   │   └── ...
│   └── software-change/
│       └── ...
└── drafts/
    └── workflows/
        └── <generated candidates>
```

The structural publication boundary is deliberate:

```text
.sergeant/drafts/workflows/   generated, reviewable, not runnable by name
.sergeant/workflows/          admitted, versioned, runnable procedure
```

Sergeant does not need a `status = draft` enforcement feature before it can keep generated work out of the runnable namespace.

## 7.2 Stable agent instructions

The repository's `AGENTS.md` should become a small constitution rather than a procedural encyclopedia. Its job is to teach a harness how to enter the Sergeant system and resolve the repository's conventions.

A minimal shape is:

```markdown
This repository uses Sergeant for durable procedural work.

- Discover available procedures in `.sergeant/index.md`.
- Select an admitted workflow explicitly when substantive work begins.
- Follow only the active stage context supplied by Sergeant.
- Resolve `@@name` references from `.sergeant/common/contexts/name.md`.
- Treat `.sergeant/common/scripts/` and workflow-local scripts as helpers,
  not independent procedure unless the workflow declares a durable stage.
- Do not treat `.sergeant/drafts/workflows/` as published procedure.
- Use Sergeant's respond, retry, cancel and inspection surfaces rather than
  fabricating workflow state in prose.
```

This instruction changes rarely. The procedure itself stays in workflow content.

## 7.3 Greppable workflow catalog

Each admitted workflow should have an `index.md` with OKF-compatible front matter:

```markdown
---
kind: workflow
name: diagnose-bug
status: published
version: 3
description: >-
  Reproduce, isolate, prove, remediate and verify a defect.
tags:
  - debugging
  - defect
  - investigation
---

# Diagnose Bug

Use when ...
```

The root `.sergeant/index.md` can list workflows and link to their local indexes.

This immediately supports:

```text
grep
find
ripgrep
ordinary Markdown reading
agent filesystem exploration
Git review
```

A later `sgt workflow list/search/show` command can parse the same metadata and join it with observed telemetry. No future discovery feature needs to invalidate the simple filesystem representation.

## 7.4 Authored metadata and observed telemetry remain separate

Authored files may contain:

```text
name
version
status
owner
description
tags
intended inputs and outputs
publication state
```

Run counts, completion rates, last execution, blocked time, cost, token use, duration, retry frequency, and failure modes belong in the journal and DuckDB projection.

The future discovery response may join them:

```text
diagnose-bug v3
  authored status     published
  tags                debugging, defect, investigation
  observed runs       184
  completion rate     87.5%
  median duration     14m22s
  last measured       2026-08-04
```

It should never write those mutable measurements back into the workflow's front matter.

## 7.5 Shared context works now as an authoring convention

A stage context can contain:

```markdown
Apply @@adversarial-review to the current change.
```

The stable agent instructions define that token as:

```text
.sergeant/common/contexts/adversarial-review.md
```

The current actor receives the stage context and runs in the worktree, so it can read that file without engine support.

This is **context composition**, not workflow composition. Sergeant pins the textual reference in `CONTEXT.md`, but today it does not pin the transitive contents of the referenced file. That is acceptable for the measurement phase because Git preserves the source revision and the work surface records its base SHA, but the exact replay semantics of transitive workflow dependencies should remain an explicit future design question.

## 7.6 Shared helpers work now as ordinary files

A stage may say:

```markdown
Run `.sergeant/common/scripts/validate-drafts.py`.
Review its structured result and correct any defects before completing.
```

The current harness executes the helper as the user. Sergeant does not need to understand Python or shell. The helper is not a stage unless its outcome is itself a durable checkpoint.

## 7.7 True nested workflows do not exist yet

An agent can read another workflow's files, and it could even submit another `sgt run`, but neither behavior creates a real child workflow inside the parent's state machine.

A hidden pseudo-subworkflow would lose:

- parent/child identity;
- pinned composition;
- stage-level visibility in the parent;
- deterministic retry and cancellation semantics;
- parent-aware recovery;
- per-subworkflow telemetry;
- a clear completion contract.

Therefore the current phase may use shared contexts and helpers, and may generate grammar-pressure findings for repeated procedures, but it should not pretend that instruction-level inclusion is durable workflow composition.

---
# 8. Build the Reference Before the Generator

The original Sergeant corpus should be treated as a requirements mine, not as the design authority for Sergeant-rs.

Its value is that it contains years of accumulated behavior in imperfect forms:

- permanent operating instructions mixed with procedure;
- reusable skills with different levels of maturity;
- shell helpers that encode both mechanics and policy;
- tests that preserve behavior more precisely than documentation;
- interactive setup flows;
- cross-repository coordination;
- independent review and adjudication loops;
- model/harness launch evidence;
- recovery conventions born from concrete failures.

That messiness makes it a better first measurement than a clean synthetic repository.

## 8.1 The manual reference corpus

Before `repo-to-icm` runs, produce an adjudicated reference decomposition of the vendored snapshot at the audited Sergeant-rs revision.

The reference should contain:

```text
reference-corpus/
├── source-inventory.md
├── behavior-units.ndjson
├── classification-ledger.md
├── permanent-instructions.md
├── draft-workflows/
│   ├── diagnose-bug/
│   ├── prototype/
│   ├── load-project/
│   ├── cross-repo-work/
│   ├── no-mistakes/
│   ├── sergeant-setup/
│   └── ...
├── helper-map.md
├── shared-context-map.md
├── obsolete-mechanisms.md
├── engine-pressure.md
└── provenance-map.md
```

The reference corpus is not required to preserve old Sergeant's filenames or implementation mechanisms. It preserves behavioral intent and traceability.

## 8.2 Representative procedures already visible in the corpus

The audit found several useful stress cases.

### Diagnosing bugs

[`diagnosing-bugs/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md) has a clear sequential shape: establish a feedback loop, reproduce and minimize, form hypotheses, instrument, fix with regression coverage, and clean up/postmortem. It is a strong low-ambiguity reference workflow.

### Prototype

[`prototype/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/prototype/SKILL.md) branches between logic and UI prototypes, emphasizes throwaway artifacts, requires one-command execution, and captures different evidence depending on prototype type. It stresses conditional procedure without requiring the runtime to become a DAG immediately.

### Cross-repository work

[`cross-repo-work/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/skills/cross-repo-work/SKILL.md) establishes repository ownership, dependency order, per-repository gates, and reconciliation across outputs. It tests the difference between one workflow over a multi-repository surface and a generalized multi-agent scheduler.

### No mistakes

[`no-mistakes/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md) combines preconditions, validation modes, independent findings, classification, user gates, fixes, branch custody, and final reconciliation. It is likely to produce legitimate pressure for shared procedures, independent actor selection, and deterministic validation stages.

### Sergeant setup

[`sergeant-setup/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md) is interactive, idempotent, consent-gated, environment-sensitive, and divided into required and optional prerequisites. It is useful evidence for `needs_input`, user authority, platform detection, and the rule that Sergeant must not silently reconfigure external tools.

### Dispatch and the old worker protocol

[`dispatch/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/skills/dispatch/SKILL.md) contains a large amount of process ownership, tmux, sentinel, callback, and worker-lifecycle machinery that Sergeant-rs now replaces structurally. The reference decomposition must distinguish the durable procedure from obsolete mechanisms rather than faithfully recreating accidental Bash/tmux architecture.

## 8.3 Reference-authoring method

The gold/reference decomposition should itself use a bounded review loop:

1. **Inventory:** enumerate all behavior-bearing files, tests, and command surfaces.
2. **Extract:** produce atomic behavior units with source citations.
3. **Normalize:** rewrite units in implementation-independent language.
4. **Classify:** apply the ICM decomposition ladder.
5. **Synthesize:** cluster units into candidate workflows and stages.
6. **Refute:** have an independent reviewer challenge boundaries, missing behaviors, and engine-gap claims.
7. **Adjudicate:** accept, reject, merge, or park each finding with evidence.
8. **Freeze:** publish a versioned reference corpus for the generator measurement.

The reference is “gold” only in the narrow evaluation sense: it is a reviewed expected result for this source snapshot. It remains reviewable and may improve when the generator exposes a better decomposition.

---

# 9. The `repo-to-icm` Workflow

## 9.1 Purpose

`repo-to-icm` converts a repository's distributed procedural knowledge into **draft** ICM workflow packages and an evidence-backed report.

It does not publish workflows. It does not modify the engine. It does not assume that every source artifact deserves an ICM artifact. It does not decide that old behavior should survive merely because it exists.

Its contract is:

```text
INPUT
  repository at a specific Git revision
  optional scope and exclusions
  target ICM conventions

OUTPUT
  source inventory
  normalized behavior units
  classification ledger
  draft workflows
  permanent-instruction candidates
  shared context/helper candidates
  obsolete-mechanism findings
  engine-pressure findings
  provenance map
  measurement summary
```

## 9.2 Why it can run on the current engine

Every stage in version 0 is an ordinary actor stage. The current engine starts a fresh execution for every stage, which is valuable here: inventory, synthesis, and adversarial review do not silently share one unbounded model turn.

Because one backend is currently bound to the whole Work, version 0 uses one selected harness throughout. That is a constraint to measure, not a reason to delay the workflow.

Deterministic helpers may be invoked by the actor from stage context. For example, inventory can call a repository-local Python helper and review its structured output. This is less efficient than a future execute stage, but it is fully expressible today and gives the later execute-stage design a real workload.

## 9.3 Proposed stages

| Stage | Current kind | Durable outcome |
|---|---|---|
| `00-contract` | actor | Establish repository revision, scope, exclusions, output paths, and success criteria. |
| `10-inventory` | actor | Produce a deterministic inventory of behavioral artifacts and identify unreadable/generated/vendor regions. |
| `20-harvest` | actor | Extract source-cited behavior units without assigning ICM forms yet. |
| `30-normalize` | actor | Rewrite behavior units independently of source filenames and old implementation mechanisms. |
| `40-classify` | actor | Apply the ICM decomposition ladder and record confidence/alternatives. |
| `50-synthesize` | actor | Cluster classified units into workflow, stage, context, helper, and invariant candidates. |
| `60-draft` | actor | Materialize draft workflow packages and catalog entries under the draft namespace. |
| `70-lint` | actor | Invoke structural validators; repair malformed metadata, broken references, duplicate identities, and missing provenance. |
| `80-adversarial-review` | actor | Fresh execution challenges coverage, over-staging, hidden file-shape translation, and speculative engine gaps. |
| `90-reconcile` | actor | Adjudicate findings and emit the final measurement package. |

The workflow is deliberately ordered. Any pressure for branching, loops, or nested procedure should be recorded rather than hidden inside an invented control language.

## 9.4 Behavior-unit record

Each extracted unit should be machine-readable and independently understandable. A minimal NDJSON shape is:

```json
{
  "id": "BU-0042",
  "statement": "Before changing a repository, verify that the requested repository belongs to the loaded project.",
  "source": {
    "path": "AGENTS.md",
    "locator": "Standard Workflow / Load context",
    "quote_hash": "sha256:..."
  },
  "scope": "cross-repository work",
  "trigger": "a work request names or implies a project repository",
  "outcome": "repository membership is established before mutation",
  "authority": "user-context actor",
  "confidence": "high",
  "notes": "The old implementation uses project YAML and shell helpers; those are mechanisms, not the normalized behavior."
}
```

The record should preserve enough source identity to re-open the evidence, but avoid copying entire source files into generated artifacts.

## 9.5 Classification record

Classification should be explicit and refutable:

```json
{
  "behavior_id": "BU-0042",
  "representation": "stage-context",
  "workflow": "cross-repo-work",
  "stage": "00-establish-scope",
  "rationale": "The rule is needed only while establishing scope; it is not a reusable procedure or independent durable checkpoint.",
  "alternatives_considered": [
    "AGENTS.md invariant",
    "separate workflow",
    "helper"
  ],
  "engine_gap": null
}
```

Engine-gap records use the stronger template from §6.7 and must name the lower-rung attempts that failed.

## 9.6 Draft output

Generated workflow packages should land outside the runnable namespace:

```text
.sergeant/drafts/workflows/<candidate>/
├── index.md
├── workflow.toml
├── provenance.md
├── scripts/
├── 00-.../CONTEXT.md
└── ...
```

A generated `provenance.md` should map every workflow and stage to the behavior units that justify it. A candidate with no source evidence is either a justified design inference—clearly marked as such—or unsupported invention.

## 9.7 Structural validator

The initial validator can be a helper script invoked by the actor. It should check:

- valid front matter;
- unique workflow names and stage IDs;
- declared stage order matches directories;
- all actor stages have context;
- referenced shared contexts/helpers resolve inside `.sergeant/`;
- no path traversal;
- no draft is accidentally placed in `.sergeant/workflows/`;
- every workflow/stage has provenance;
- no engine-gap record omits failed lower-rung alternatives;
- no generated script is executable solely because a Markdown instruction forgot to classify it.

When execute stages exist, this validator becomes a natural first execute-stage workload.

## 9.8 Measurement against the reference corpus

The maiden voyage is performed in this order:

```text
1. Freeze the vendored Sergeant source snapshot.
2. Freeze the adjudicated manual reference decomposition.
3. Run repo-to-ICM without exposing the reference outputs to the generator.
4. Compare generated and reference behavior units and representations.
5. Review disagreements blind to which side produced them where practical.
6. Classify each disagreement:
     generator miss
     generator invention
     gold miss
     legitimate alternate decomposition
     ambiguous source
     genuine engine pressure
7. Publish the scorecard and update the workflow only through a reviewed change.
```

## 9.9 Measurement dimensions

The scorecard should avoid one misleading “accuracy” number. Measure at least:

### Source coverage

What proportion of behavior-bearing source regions produced at least one traceable behavior unit?

### Behavioral recall

How many reference behaviors were recovered, regardless of exact workflow grouping?

### Behavioral precision

How many generated behavior units were supported by source evidence rather than invented from generic software-development priors?

### Workflow-boundary agreement

Did the generator identify similar reusable outcomes, or merely mirror source file boundaries?

### Stage-boundary agreement

Did it find meaningful durable checkpoints, over-fragment commands into stages, or collapse independently measurable boundaries into prose?

### Representation agreement

Did it distinguish permanent instructions, workflows, stages, actor context, helpers, shared content, obsolete mechanisms, and engine gaps?

### Engine-gap quality

How many proposed engine features survived lower-rung refutation and independent review?

### Provenance completeness

Can every generated artifact be traced to source behavior units, and can every source-cited behavior be reopened at the pinned revision?

### Draft validity

Do generated packages pass the structural validator without manual syntax repair?

### Review convergence

How many adversarial findings remain unresolved after reconciliation, and what kinds recur?

## 9.10 Avoid overfitting to Sergeant

Old Sergeant is measurement number one, not the universal workflow ontology.

After the maiden voyage, run the same workflow against repositories with different behavioral shapes:

- a library with a small `CONTRIBUTING.md` and CI scripts;
- an infrastructure repository with runbooks and policy checks;
- a product repository with issue/PR templates and release automation;
- a documentation or research repository with few executable helpers;
- a multi-repository workspace declaration.

A grammar feature should not be promoted because one historically complex Bash project made it tempting.

---

# 10. Shared Context, Shared Helpers, and Future Shared Workflows

There are three different forms of reuse. They should not collapse into one syntax or runtime mechanism.

```text
HELPER
  Reusable deterministic machinery.
  Answers: how is this operation performed?

SHARED CONTEXT
  Reusable actor guidance.
  Answers: how should the actor reason while performing this stage?

SHARED WORKFLOW
  Reusable durable procedure.
  Answers: which independently observable state machine should run?
```

## 10.1 Helpers now

Use repository-relative references to workflow-local or common scripts. The actor invokes them. Their existence and path are Git facts.

## 10.2 Context inclusion now

Use an agent-facing convention such as:

```text
@@adversarial-review
```

resolved from:

```text
.sergeant/common/contexts/adversarial-review.md
```

This is an authoring convention. The actor reads the file; the engine remains unchanged.

## 10.3 Shared workflows later, if proven

A future durable call might eventually look like:

```toml
[stage."40-review"]
kind = "workflow"
workflow = "adversarial-review"
```

But that is not part of this iteration's first implementation.

It should be considered only when measurements show all of the following:

1. the same coherent procedure appears in multiple parent workflows;
2. copying its stages creates meaningful drift;
3. shared context cannot preserve its required retry, block, cancellation, recovery, or telemetry boundaries;
4. launching it as independent top-level Work loses necessary parent/child semantics;
5. the minimum child-workflow lifecycle can be specified without turning Sergeant into a general DAG engine.

Until then, `@@context` and shared helpers are sufficient reuse. An actor silently calling `sgt run` from inside a parent stage is not accepted as fake composition because it hides the relationship from the parent workflow.

---
# 11. Phase B: The Minimum Runtime Extension

The repo-to-ICM measurement is expected to produce one already well-supported grammar need: a durable stage may be performed either by a reasoning actor or by a controlled deterministic execution environment.

The runtime should model exactly those two stage classes first.

## 11.1 Actor stage

An actor stage delegates the checkpoint to a selected native harness acting on behalf of the user.

It inherits the user's:

- harness authentication and subscription;
- repository trust decision;
- Git identity and credentials;
- installed organization-specific CLIs;
- MCP/tool configuration allowed by the harness profile;
- operating-system environment, subject to the adapter's explicit launch policy.

The actor may use `git`, `gh`, a Jira CLI, `kubectl`, an internal deployment CLI, or any other available user tool because the workflow instructs the harness to do so. Sergeant does not need a GitHub adapter merely because one workflow says “open a PR.”

The actor contract remains the current backend contract: start a native context, observe explicit semantic signals, send human input, interrupt, resume, and stop.

## 11.2 Execute stage

An execute stage runs a declared operation in a declared container image through the user's local Docker Engine.

Its semantic result is mechanical:

```text
container could not be prepared or observed unambiguously  → blocked
container exits 0                                           → stage completed
container exits nonzero                                     → stage failed
operator cancels                                            → stage canceled
```

Stdout content never silently changes the outcome. A workflow that needs to interpret the output must place a later actor stage after the execute stage or define a structured artifact contract in a future measured extension.

## 11.3 No third native-command stage initially

The conceptual split discussed in this design is actor versus execute, not “LLM versus every other executable.”

A stage whose procedure is “use my authenticated Jira CLI to fetch the ticket, inspect it, and determine what matters” is an actor stage. The selected harness runs the user's command and reasons about the result.

A direct native-command stage may eventually save model cost for trivial user-context operations, but it introduces a third lifecycle with difficult crash semantics: a short-lived `gh pr create` may succeed remotely while Sergeant dies before recording the result. It also expands platform-specific command discovery and trust policy.

That capability should be proposed only if the repo-to-ICM measurements repeatedly produce stages that are meaningful durable checkpoints, require user authority, require no judgment, and are wasteful or unsafe to route through a harness. It is not necessary to establish the next architecture.

## 11.4 Different actor stages may use different harnesses

A workflow should be able to express:

```text
00-understand             Claude
10-implement              Codex
20-validate               Docker / Python image
30-review                 Claude
40-adversarial-review     OpenCode
50-close                  Claude or another admitted harness
```

This is not ensemble machinery in Sergeant. It is stage-local actor selection.

The workflow declares what harness a checkpoint requires. The harness adapter owns its native lifecycle. Sergeant preserves one stage state machine and one trajectory.

---

# 12. Backward-Compatible Workflow Schema

The existing ordered stage list should remain valid. A repository should not have to rewrite every workflow merely because stage metadata becomes possible.

A minimally disruptive extension is to retain order under `[workflow]` and add optional metadata tables keyed by stage ID.

## 12.1 Legacy workflow remains valid

```toml
[workflow]
name = "software-change"
version = "1"
stages = ["00-prepare", "10-implement", "20-review", "30-close"]
```

Each stage defaults to:

```text
kind          actor
context       <stage>/CONTEXT.md
harness       Work actor default
profile       Work/profile default
model         profile or harness ambient default
```

## 12.2 Per-stage actor selection

```toml
[workflow]
name = "software-change"
version = "2"
stages = ["00-prepare", "10-implement", "20-review", "30-close"]

[stage."00-prepare"]
kind = "actor"
harness = "claude"
profile = "analysis"

[stage."10-implement"]
kind = "actor"
harness = "codex"
profile = "implementation"

[stage."20-review"]
kind = "actor"
harness = "claude"
profile = "review"

[stage."30-close"]
kind = "actor"
```

`CONTEXT.md` remains required for actor stages.

## 12.3 Execute stage

```toml
[workflow]
name = "software-change"
version = "2"
stages = ["00-prepare", "10-implement", "20-validate", "30-review"]

[stage."20-validate"]
kind = "execute"
image = "python:3.13-slim"
command = ["python", ".sergeant/common/scripts/validate_pipeline.py"]
workdir = "/workspace"
workspace_access = "read_write"
network = "none"
```

A shell is explicit when required:

```toml
[stage."20-validate"]
kind = "execute"
image = "ubuntu:24.04"
command = ["bash", "-lc", "./scripts/validate.sh && ./scripts/check-artifacts.sh"]
workdir = "/workspace"
workspace_access = "read_write"
network = "none"
```

Sergeant sees `bash` as an executable inside the image. It does not parse or manufacture the shell expression.

## 12.4 Execute-stage human readability

An execute stage may carry an optional stage-local `CONTEXT.md` or `README.md` explaining:

- why the checkpoint exists;
- what its command validates or produces;
- what a nonzero exit means;
- which evidence a reviewer should inspect;
- whether it mutates the worktree.

The execute driver does not consume that prose. It is documentation and future discovery material. The machine contract remains in `workflow.toml`.

## 12.5 Actor-default semantics

The current Work-level route becomes an **actor default**, not an override of explicit stage requirements.

Suggested precedence for each actor stage:

```text
stage.harness explicitly names a harness
        ↓
otherwise Work actor default, resolved by today's routing chain
        ↓
fail with available harnesses
```

If a workflow explicitly requires Claude for review and Claude is unavailable, Sergeant does not silently use Codex. The same no-substitution rule already implemented in `runtime/router.rs` applies stage by stage.

The Work submission fields `--backend` and `--profile` remain useful as defaults for legacy or portable workflows. A future explicit per-stage override can be designed if real use demands it; it is not needed in the first schema.

---

# 13. Domain Model Changes

## 13.1 StageDefinition becomes a tagged executor definition

Conceptually:

```rust
struct StageDefinition {
    id: String,
    executor: StageExecutor,
    documentation: Option<String>,
}

enum StageExecutor {
    Actor(ActorStage),
    Execute(ExecuteStage),
}

struct ActorStage {
    context: String,
    harness: HarnessSelection,
    profile: Option<String>,
}

struct ExecuteStage {
    image: String,
    command: Vec<String>,
    workdir: String,
    workspace_access: WorkspaceAccess,
    network: NetworkPolicy,
    env: BTreeMap<String, String>,
}
```

This is a conceptual contract, not mandated Rust spelling.

The resolved and journaled `WorkflowDefinition` must include the full executor specification. Editing `workflow.toml` after Work begins cannot alter a later stage's image, command, harness, or policy.

## 13.2 Backend remains a harness adapter

The current `Backend` trait should remain semantically focused on native reasoning harnesses. It may eventually be renamed `HarnessAdapter` for clarity, but renaming is not required to add execute stages.

Docker execution belongs behind a separate internal contract, for example:

```text
ContainerRuntime
  probe
  resolve_image
  create
  start
  logs
  inspect
  stop
  remove
  reconcile
```

The commonality between a harness and a container exists at the engine's execution lifecycle—not at the adapter protocol itself.

## 13.3 WorkRun stores actor defaults, not one authoritative backend

The current `WorkRun.backend`, route source, and profile are whole-run facts. In the new model:

- preserve the resolved actor default and its source for backward compatibility;
- record the selected harness/profile on each actor execution;
- allow execute stages to have no harness;
- expose current-stage executor details in API views;
- keep all prior execution attempts in the trajectory even if the current projection still exposes only the latest binding initially.

## 13.4 ExecutionRecord becomes executor-aware

The current `ExecutionRecord` explicitly means “native execution context owned by a backend.” The next model should not hide a Docker container in fields named `backend` and `native_id` without saying what it is.

A tagged binding is clearer:

```rust
enum ExecutionBinding {
    Harness {
        harness: String,
        native_id: Option<String>,
        profile: Option<Profile>,
    },
    Container {
        container_id: String,
        container_name: String,
        image_requested: String,
        image_id: String,
        repository_digest: Option<String>,
        platform: Platform,
    },
}
```

The stable outer record still carries:

```text
execution_id
work_id
stage_id
attempt
binding
lifecycle state
timestamps/evidence pointers
```

DuckDB and graph projections may add executor kind and container/image columns because they rebuild from the journal.

---

# 14. The Execution Effect Boundary

Docker should not be bolted into the current synchronous `enter_stage` path while the core mutex is held.

The next revision should make external effects explicit.

## 14.1 Current pressure

Today API mutation handlers hold the core lock while engine methods plan, start, observe, stop, and append related events. Most current backend starts are short process spawns, but open issue [#14](https://github.com/miztertea/sergeant-rs/issues/14) already records a blocking archive-thread join inside `ClaudeBackend::stop` while the lock is held.

Image pulls, container creation, streamed logs, and container waits are much longer-lived. Putting them under the same lock would worsen the measured single-writer and read-queue behavior from P1.

## 14.2 Two-phase executor lifecycle

The engine should separate authoritative decisions from external effects:

```text
UNDER CORE LOCK
  validate current Work/stage/attempt
  allocate Sergeant execution id
  resolve and pin executor spec/defaults
  append execution.reserved
  return an effect request

OUTSIDE CORE LOCK
  reserve/create native external identity
  launch or attach
  stream/observe execution

UNDER CORE LOCK
  verify execution id and attempt are still current
  append observed lifecycle/result events
  perform the legal stage/Work transition
```

This is not a generalized job queue. It is the minimum boundary required to keep slow external I/O out of the authoritative lock and to make crash windows inspectable.

## 14.3 Repair the Claude start window at the same seam

The Claude adapter already generates a session ID before spawning. The engine should be able to journal that reserved identity before launch.

A possible harness contract evolution is:

```text
prepare(request) → PreparedHarnessExecution
journal prepared identity
launch(prepared)
observe/reattach by prepared identity
```

This closes the documented “spawned before `execution.started`” window and gives every future harness the same rule.

Because this changes the §15 trait, it is also the named trigger for resolving B3/#14: stop/interrupt should return or expose any completion/join handle so the daemon can release the core lock before waiting for transcript archival.

## 14.4 Docker reservation and ownership

A Docker execution can be reserved before container creation with:

- a Sergeant execution ID;
- a deterministic container name derived from execution ID;
- Sergeant ownership labels;
- the pinned execute-stage spec.

Journal that reservation first. Then create the stopped container through the API. If the daemon dies after creation but before journaling the Docker-generated container ID, recovery can locate the exact reserved name/labels through Docker rather than scanning arbitrary host processes.

Sergeant must never use a broad `docker system prune` or delete containers merely because their names resemble Sergeant's. Cleanup requires the recorded execution identity plus ownership labels.

## 14.5 Late results remain subordinate to durable state

An executor completion callback must verify that:

- Work still exists;
- the stage and attempt are still current;
- the execution ID matches the active binding;
- terminal Work has not absorbed the run;
- cancellation or retry has not superseded the result.

A container exiting successfully after the user canceled the Work is recorded as late evidence; it does not complete the canceled Work.

---

# 15. Execute-Stage Semantics

## 15.1 Entry

On first entry to an execute stage:

1. append `stage.entered` with executor kind and attempt;
2. reserve the execution identity and pinned specification;
3. resolve the requested image reference for the selected platform;
4. record the immutable identity that will be used;
5. create and start the container;
6. begin bounded output capture and lifecycle observation.

## 15.2 Completion

The container exit code is the stage's mechanical result:

```text
0       stage.completed
nonzero stage.failed
```

The stage detail should carry a concise summary: exit code, duration, image identity, platform, and output artifact references. Full stdout/stderr belongs in the blob store, not an unbounded event payload.

## 15.3 Retry

The requested image reference is pinned in the WorkflowDefinition. The first successful image resolution for that stage should also be pinned to the Work run.

By default, retry uses the same immutable resolved image identity rather than silently re-resolving `latest` or a mutable version tag to a different image. Each attempt still records what actually ran. An explicit future operator action may request refresh/re-resolution; it should never happen implicitly during retry.

## 15.4 Waiting and needs-input

An execute stage itself does not reason or ask questions. It may be running, complete, failed, canceled, or blocked by unambiguous runtime problems.

A procedure that needs a human decision based on execute output places an actor stage after the execute stage. That actor can inspect the output artifact and signal `needs_input` through its harness.

## 15.5 Cancellation

Cancellation should:

1. record the Work/stage cancellation before destructive external action where current transition ordering requires it;
2. ask Docker to stop/kill the exact owned container according to policy;
3. continue capturing the final observable outcome;
4. remove the container when safe;
5. journal each request and result honestly.

A stop request is not the same as proof that the container stopped, just as the current backend contract distinguishes a stop request from native truth.

## 15.6 Timeout

Do not invent a universal timeout in the first execute-stage grammar. Different validation workloads have radically different expected durations.

A later optional stage timeout is reasonable after cancellation and recovery are proven. When added, it is an authored policy and its firing is a journaled fact—not an HTTP client timeout masquerading as a stage outcome.

---
# 16. Docker Execution Adapter

The Bashful design contributes the right boundary:

> The operation describes what to run; the adapter decides where and under what isolation.

Bashful's documented adapter model uses Docker as the default isolated execution substrate and host subprocess as a fallback. Sergeant should borrow the adapter boundary, not the fallback. A host-subprocess execute path would recreate the shell, platform, dependency, and ambient-environment problem this design is intended to remove.

The relevant Bashful sources are:

- [IdeaOS — Swappable Agent Execution Adapters](https://app.notion.com/p/39a27ada618f8157a9a6c54d56444357?pvs=204)
- [`miztertea/bashful` adapter implementation](https://github.com/miztertea/bashful/blob/877f9d5b4d93e85bd02b51ff562efde6188a212b/src/bashful/adapter.py)

## 16.1 Rust client

The recommended first implementation client is [`bollard`](https://docs.rs/bollard/latest/bollard/), currently documented as a Tokio-compatible Rust client for the Docker API. Its [`Docker`](https://docs.rs/bollard/latest/bollard/struct.Docker.html) connection surface supports the platform transports Sergeant needs, including Unix sockets and Windows named pipes, and exposes the container, image, log, wait, inspect, stop, and remove operations directly.

Using the Engine API rather than scraping the `docker` CLI matters because it gives Sergeant:

- structured request and response types;
- Docker API version negotiation;
- native streaming of logs and wait results;
- explicit container IDs, labels, image IDs, state and exit codes;
- no second command-output grammar to parse;
- no host-shell dependency;
- one adapter boundary that can be contract-tested.

Docker's canonical Engine API documentation is [docs.docker.com/reference/api/engine](https://docs.docker.com/reference/api/engine/).

`bollard` is a nominated implementation choice, not a permanent architectural dependency. The durable contract is an internal `ContainerExecutor` interface whose behavior is tested against a real local Docker Engine. If measurement finds a correctness gap in Bollard, Sergeant may use a smaller direct client without changing workflow semantics.

## 16.2 Local Engine only in the first implementation

Docker contexts can point at local or remote daemons. Docker documents contexts as endpoint and TLS metadata selected independently of the CLI process: [Docker contexts](https://docs.docker.com/engine/manage-resources/contexts/).

A remote daemon is incompatible with Sergeant's first execute-stage contract because a bind source is resolved on the **daemon host**, not the API client host. Docker's bind-mount documentation states that bind mounts are created against the Docker daemon host: [bind mounts](https://docs.docker.com/engine/storage/bind-mounts/).

The first adapter should therefore accept only a Docker endpoint proven to be local to the Sergeant host:

```text
Linux/macOS local Unix socket       accepted
Windows local named pipe            accepted
Docker Desktop local endpoint       accepted after bind-mount probe
remote tcp:// / ssh:// context       refused for execute stages
```

“Local” must be established from the resolved connection configuration and a real bind-mount capability probe, not from the context name `default`.

Remote Docker, Kubernetes jobs, SSH workers, Firecracker, E2B, or another execution substrate may later implement the same executor contract, but each needs its own workspace-transfer and identity model. They are not aliases for local Docker.

## 16.3 Readiness is a lifecycle probe, not a socket check

The Docker probe should prove the exact lifecycle Sergeant depends on:

```text
connect
  ↓
negotiate API
  ↓
ensure known probe image
  ↓
create container
  ↓
bind-mount scratch directory
  ↓
start
  ↓
read/write a marker
  ↓
capture stdout/stderr
  ↓
wait for exit
  ↓
inspect exit and image identity
  ↓
remove container
```

A successful `ping` proves only that something answered. A successful lifecycle probe proves that the user's Engine permits the operations and mounts Sergeant will actually require.

The probe should use a tiny known image whose purpose and expected output are owned by Sergeant. Docker's `hello-world` image is suitable for an installation smoke test but does not prove worktree bind mounts. The stronger probe mounts a temporary directory, verifies a read and a write, records the resolved platform, and cleans up its own labeled container.

The probe result is a capability record:

```json
{
  "available": true,
  "endpoint_kind": "unix",
  "api_version": "...",
  "server_version": "...",
  "os": "linux",
  "architecture": "amd64",
  "bind_mount": true,
  "log_stream": true,
  "wait": true,
  "cleanup": true
}
```

No field should be reported merely because the API documentation lists it.

## 16.4 Image resolution and immutable execution evidence

A workflow may author a mutable reference:

```toml
image = "python:3.13-slim"
```

or even:

```toml
image = "python:latest"
```

That is acceptable as an authoring choice. It is not acceptable as historical evidence.

Docker distinguishes mutable tags from immutable content digests; its documentation recommends digests when an exact image is required: [pulling by digest](https://docs.docker.com/reference/cli/docker/image/pull/#pull-an-image-by-digest-immutable-identifier).

For every execute stage, Sergeant should record at least:

```json
{
  "image_requested": "python:3.13-slim",
  "image_id": "sha256:...",
  "repo_digests": ["python@sha256:..."],
  "platform": {
    "os": "linux",
    "architecture": "arm64",
    "variant": "v8"
  }
}
```

The Docker image-inspect API exposes the fields needed for this evidence: [Engine API image inspect](https://docs.docker.com/reference/api/engine/version/v1.49/#tag/Image/operation/ImageInspect).

The lifecycle should be:

1. use a matching local image when policy allows it;
2. otherwise pull the authored reference for the daemon platform;
3. inspect the resolved image;
4. journal the immutable image ID, any repository digest, and platform before start;
5. create the container using the immutable ID or digest;
6. verify after creation that the container reports the same image identity.

For a given Work and stage attempt lineage, the first successful resolution becomes the retry pin. A retry should use the pinned immutable image, not silently ask what `latest` means now.

If the local image was pruned before retry, Sergeant may pull by the recorded repository digest when one exists. If it has only a local image ID that can no longer be resolved, the stage blocks with evidence rather than reinterpreting the mutable tag.

Image refresh is an explicit future operator action, not retry behavior.

## 16.5 Registry authentication belongs to the user's Docker environment

The design assumes Sgt is acting inside the user's configured development environment. That includes the user's Docker registry login and credential helpers.

Docker's login documentation describes its platform credential stores and `credsStore` / `credHelpers` configuration: [docker login and credential stores](https://docs.docker.com/reference/cli/docker/login/#credential-stores).

The Engine API accepts registry authentication supplied by the client for pull requests. Opening the local Engine socket does not automatically give a Rust client the Docker CLI's decoded registry credentials.

The adapter therefore needs an explicit Docker-auth resolution seam:

```text
requested registry
        ↓
read user's Docker config location
        ↓
resolve registry-specific credential helper or inline auth
        ↓
obtain pull credential in memory
        ↓
call Engine API
        ↓
drop credential
```

Credentials must never enter:

- workflow files;
- journal payloads;
- blob artifacts;
- structured errors;
- logs;
- container environment unless the workload explicitly requires a separately governed secret mechanism.

The [`docker_credential`](https://docs.rs/docker_credential/latest/docker_credential/) crate is a candidate for invoking Docker credential helpers. It must be evaluated against Docker Desktop on Windows and macOS and common Linux helpers before adoption.

A lower-rung first release may support:

```text
public images                 pull automatically
already-present private image run by immutable local ID
missing private image         block with actionable "pre-pull or configure helper" evidence
```

That is preferable to inventing an incomplete credential broker inside Sergeant.

## 16.6 Worktree mount contract

The default execution filesystem is:

```text
host Work surface root  →  /workspace
container workdir       →  /workspace or authored descendant
```

The stage must explicitly declare whether the workspace is read-only or read-write:

```toml
workspace_access = "read_only"
```

or:

```toml
workspace_access = "read_write"
```

Making this field explicit in the first schema is better than silently granting write access to a container because most coding workloads happen to need it.

The first implementation should permit no arbitrary host mounts. Later mount support must name:

- a Sergeant-owned source class, not an unconstrained host path;
- read-only or read-write access;
- the container destination;
- why the stage requires it.

The Docker socket is never mounted into an execute container. Docker warns that bind mounts can modify host files and are tied to the daemon host; the socket would grant the workload authority to create privileged sibling containers and mount arbitrary host paths. Sergeant itself owns Docker API access.

## 16.7 Default isolation

The first execute-stage policy should be conservative:

```text
privileged                 false
host pid namespace         false
host network               false
extra Linux capabilities   none
host devices               none
Docker socket              absent
extra mounts               absent
workspace mount            authored read_only/read_write
network                     none unless explicitly enabled
```

`network = "none"` should mean Docker's no-network mode. A later named network policy may admit outbound access, but the first schema should not accept arbitrary Docker network names or host networking.

The image itself supplies Bash, Python, Node, Rust, Java, or another runtime. Sergeant does not install package managers inside a running container on the author's behalf and does not infer a shell from file extension.

## 16.8 User and file-ownership behavior must be measured

Container UID/GID behavior differs across native Linux and Docker Desktop's virtualization layers. A root process in a Linux container can leave root-owned files in a native Linux bind mount, while Docker Desktop mediates ownership differently.

The first implementation must measure at least:

- native Linux rootful Docker;
- native Linux rootless Docker where supported;
- Docker Desktop on macOS Intel and Apple silicon where available;
- Docker Desktop on Windows with WSL2 backend;
- worktrees whose paths contain spaces and non-ASCII characters.

Potential policies include:

```text
run as image default
run as host uid:gid on Unix
run as a Sergeant-provided non-root user
require image to declare a compatible user
```

No policy should be selected from intuition. The acceptance criterion is that execute stages do not leave the user's Git worktree in an ownership state the user cannot edit or clean.

## 16.9 Output capture must be streaming and bounded

The current Claude adapter accumulates raw stdout and stderr in memory before storing the completed stream in the blob store. The P1 performance work makes it clear that the next executor should not repeat that shape for arbitrary build logs.

Docker exposes multiplexed stdout/stderr streams. Sergeant should consume them incrementally and maintain three products:

```text
1. complete raw evidence
   streamed to a temporary spool and finalized into the blob store;

2. normalized lifecycle events
   start, periodic progress metadata if justified, exit, capture outcome;

3. bounded human tail
   last N bytes/lines for API, TUI and dashboard display.
```

The journal should not contain one event per log line by default. A large compiler log would multiply journal volume and projection work without adding durable semantics.

Suggested final event shape:

```json
{
  "exit_code": 1,
  "duration_ms": 48321,
  "stdout": "b3:...",
  "stderr": "b3:...",
  "stdout_bytes": 9182234,
  "stderr_bytes": 2819,
  "tail": "...bounded text...",
  "capture": "complete"
}
```

A disk-full or blob-write failure is recorded as a named capture failure. It does not turn a known container exit into an unknown exit, but no observer is allowed to believe the full log exists.

The implementation should evolve the blob store to accept a streamed/file finalization path rather than requiring all bytes in one `Vec<u8>`.

## 16.10 Container ownership and naming

Every Sergeant container should carry deterministic identity:

```text
name:
  sgt-<execution-id>

labels:
  io.sergeant.managed=true
  io.sergeant.execution=<execution-id>
  io.sergeant.work=<work-id>
  io.sergeant.stage=<stage-id>
  io.sergeant.attempt=<attempt>
  io.sergeant.schema=execution/v1
```

The journal still owns identity. Labels are recovery evidence, not a competing state store.

Container creation must fail closed when the deterministic name already exists but its labels do not match the reserved execution. Sergeant never adopts an unlabeled or differently labeled container merely because the name looks right.

## 16.11 Recovery matrix

Docker gives Sergeant stronger post-restart evidence than host process scanning because the Engine retains a named container object and lifecycle state.

Recovery should classify each reserved execute-stage execution as follows:

| Journal evidence | Docker evidence | Required disposition |
|---|---|---|
| reserved, no recorded container ID | no matching labeled container | no external effect proven; retry may recreate after recording reconciliation |
| reserved, no recorded ID | one exact name+label match | adopt its container ID and inspect state |
| reserved | multiple conflicting matches | block as ambiguous; delete nothing |
| started | running | reattach logs/wait and continue |
| started | exited with exit code | record exit and drive stage exactly once |
| started | created but never started | start only when policy and journal prefix prove it was the intended next effect; otherwise block |
| started | paused/restarting/dead/unknown | map only measured states; otherwise block |
| started | missing | block with evidence; do not fabricate failure or success |
| canceled/terminal Work | container still running | request stop, record late lifecycle, never change terminal Work |
| completed capture, container remains | remove exact owned container and record cleanup result |

Every adjacent append in this lifecycle must have a crash injection test, following `LESSONS.md` L6.

## 16.12 Cleanup is exact, never global

Sergeant removes only the container whose ID and ownership labels match the execution being retired.

It does not:

- prune images;
- prune volumes;
- prune networks;
- run `docker system prune`;
- delete every container bearing a prefix;
- remove unrelated build cache;
- delete a mismatched container to free a desired name.

Stale owned containers may eventually be reported by `sgt doctor` or a dedicated reconciliation command. Deletion remains evidence-backed and individually addressed.

---

# 17. Harness Registry, Capability Discovery, and Doctor

The user's phrasing defines the product boundary:

> Sergeant drives the user's Git, the user's Docker, and the user's Claude, Codex, Goose, OpenCode, or other harnesses.

Sergeant does not create identities for those systems. It discovers whether it can faithfully act through them.

## 17.1 Harnesses are adapter-owned capabilities

The existing `BackendRegistry` is the right conceptual home for native harnesses, though the product language should converge on **harness** rather than treating every executor as a backend.

Each harness adapter owns:

```text
executable discovery
version parsing and minimum measured version
required launch flags/protocols
authentication evidence
workspace trust/readiness evidence
model/profile transport
native session identity
start/send/observe/interrupt/resume/stop
raw and normalized event capture
recovery and reconciliation
```

`sgt doctor` must ask each adapter for its own probe. It must not contain a second copy of Claude, Codex, Goose, or OpenCode's rules.

This preserves `LESSONS.md` L8:

> A capability flag is a claim; every advertised verb needs a contract test.

## 17.2 Installation probe versus workspace probe

A single Boolean `available` becomes too coarse once trust and authentication depend on the current repository.

Separate:

```text
installation probe
  Is the binary present?
  Is its version measured?
  Are required flags/protocols present?
  Is general authentication visible when safely inspectable?

workspace probe
  Can this harness run non-interactively as this user in this work surface?
  Has the user accepted the folder/repository trust prompt?
  Can it create or resume the native identity Sergeant requires?
```

A harness may be installed and authenticated but not ready for a newly materialized worktree because its trust decision is path-specific. The adapter should return that distinction as structured evidence and an exact remedy.

The adapter must not edit another harness's trust file or click through a safety prompt silently. The user or the harness's supported setup command owns that authorization.

## 17.3 Cheap and live probes

`sgt doctor` should remain safe and inexpensive by default:

```text
sgt doctor
  binary/version/flags
  inspectable auth/trust status
  Git capability
  Docker lifecycle capability
  data dir/journal/projection/daemon
```

Some claims cannot be established without executing the harness. Provide an explicit opt-in path:

```text
sgt doctor --live
```

A live harness probe should use the smallest possible non-destructive turn and prove the adapter's actual contract—for example, cwd visibility, native session identity, resume, and stop—while reporting that it may consume provider quota.

A live Docker probe is not optional because container lifecycle itself is deterministic and token-free; the ordinary doctor may perform it against a temporary directory and known probe image, with a cache-aware fast path when appropriate.

## 17.4 Doctor becomes a capability report

A future human rendering may look like:

```text
Host
  ✓ git       2.x; worktree create/remove probe passed
  ✓ docker    local Engine; API ..., linux/arm64; bind mount passed

Harnesses
  ✓ claude    installed, version measured, authenticated
              workspace trust: ready
              start resume interrupt usage model-selection
  ✓ codex     installed, version measured, authenticated
              workspace trust: ready
              start resume interrupt ...
  ! opencode  executable found; adapter version unmeasured
  - goose     not installed

Runtime
  ✓ data dir
  ✓ journal
  ✓ projections
  ✓ daemon descriptor
```

Machine JSON should distinguish:

```text
healthy              base Sergeant storage/control plane is sound
work_capabilities    which workflow requirements can run here
warnings             usable but incomplete evidence
failures             broken invariant or required dependency
```

## 17.5 Degraded daemon, strict work admission

The daemon should be able to start when Docker is unavailable so that:

- `sgt doctor`, `sgt status`, the TUI and dashboard remain available;
- existing journal state can be inspected;
- blocked recovery can be understood;
- the user receives an actionable remedy rather than an absent control plane.

But work admission must be strict.

During planning, Sergeant knows the pinned workflow. It should derive its requirements:

```text
actor harnesses named by its stages
a usable actor default when stages inherit one
Docker execute capability when any execute stage exists
Git and worktree capability
profiles/models named by actor stages
```

If a requirement is unavailable, reject the submission **before Work or worktree side effects**, with structured available options and remedies.

An actor-only workflow may therefore run when Docker is unavailable. A mixed or execute workflow may not.

The supported installation profile may still state plainly that Git and Docker are required for the full Sergeant feature set and at least one harness is required for actor stages.

## 17.6 Workflow compatibility

Discovery can later show:

```text
release-change
  requires:
    git
    docker
    harness:claude
    harness:codex
  compatible: false
  missing:
    harness:codex — executable not found
```

Compatibility is evaluated against the resolved workflow, not against free-text tags alone.

## 17.7 Installer boundary

A Sergeant installer may:

- select the correct signed/released multi-architecture `sgt` binary;
- verify or guide installation of Git;
- verify or guide installation of Docker Desktop or Docker Engine;
- run `sgt doctor` and explain failures;
- optionally scaffold repository-local Sergeant files with explicit user consent.

It should not silently:

- modify Claude/Codex/Goose/OpenCode trust or credential files;
- configure organization-specific Git remotes;
- log the user into GitHub, Jira, cloud providers, or registries;
- accept Docker Desktop terms;
- enable virtualization or WSL features without a visible platform-specific action;
- install arbitrary harnesses the user did not select.

The user supplies authority. Sergeant verifies and uses it.

---

# 18. Cross-Platform Contract: Linux, macOS, Windows, and WSL

The repository currently builds around several Unix/Linux assumptions even though much of its Rust and process code is already portable.

The target contract should be explicit:

```text
Linux native             first-class
macOS native             first-class
Windows native           first-class
WSL2                     supported as Linux, with Windows-host Docker behavior measured
```

“Cross-platform” means the same durable contracts pass on each platform. It does not mean every private implementation is identical.

## 18.1 Never construct shell command strings in core

Rust's [`std::process::Command`](https://doc.rust-lang.org/stable/std/process/struct.Command.html) supplies the correct primitive:

```text
program
argv[]
cwd
environment
stdin
stdout
stderr
```

Sergeant already invokes Git and Claude in this structured form. Keep that invariant for every Sgt-owned host process.

Core code should never manufacture:

```text
cd ... && ...
quoted shell fragments
globs
redirections
pipes
host-shell variable expansion
```

A workflow that requires Bash or PowerShell names it explicitly inside an execute image, where the image defines the shell contract.

## 18.2 Platform paths must use platform APIs

The CLI currently defaults to `$XDG_DATA_HOME/sergeant` or `~/.local/share/sergeant`; that is not a native Windows path contract.

The [`directories`](https://docs.rs/directories/latest/directories/) crate is a candidate for resolving user data/config/cache directories through platform conventions while preserving explicit overrides:

```text
--data-dir
SGT_DATA_DIR
platform-native project data directory
```

Existing users' paths need a documented compatibility/migration rule. Sergeant must never silently select a fresh empty data directory and make existing Work appear to vanish.

## 18.3 Runtime descriptor identity needs more than a PID

The current `pid_alive` implementation checks `/proc/<pid>` on Linux and returns `true` everywhere else—the fail-closed direction, but one that prevents ordinary stale-descriptor recovery on macOS and Windows.

A PID alone is also reusable. A future descriptor should include a process-instance discriminator that can be verified across supported platforms, such as a measured process start time/token alongside the PID.

Possible implementation sources include a small platform module or a bounded use of [`sysinfo`](https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html), but the contract is:

```text
endpoint healthy                 descriptor live
endpoint dead + exact process instance dead  stale
endpoint dead + instance alive or unprovable ambiguous; refuse second daemon
```

This platform process check is for Sergeant's own daemon descriptor. It must not become the recovery method for harness sessions.

## 18.4 Harness recovery must not depend on process scanning

The current Claude adapter scans Linux `/proc` for a session ID in argv and reports liveness as unknowable elsewhere. Issue [#18](https://github.com/miztertea/sergeant-rs/issues/18) already records that portability debt.

The preferred recovery ladder is:

```text
1. native durable harness identity and supported query/reattach protocol
2. child handle while this daemon remains alive
3. harness-owned durable transcript/session evidence
4. ambiguity → blocked
```

Operating-system process enumeration may be supplementary evidence for a measured adapter, never the sole correctness primitive or a cross-harness abstraction.

A harness whose native APIs cannot distinguish “still running” from “session exists” must say so. Sergeant blocks rather than writing a portable-looking guess.

## 18.5 Tiny platform boundary

Host-specific behavior should concentrate under a small internal boundary, conceptually:

```text
platform/
  dirs
  daemon_process_identity
  child_tree
  signals
  file_permissions
  durable_replace
  browser_open
```

Examples:

- Unix process groups versus Windows Job Objects for owned child-tree termination;
- SIGTERM/SIGHUP versus Windows control events;
- Unix `0600` modes versus Windows ACLs for the runtime descriptor;
- directory fsync and rename-replace semantics;
- `xdg-open` / `open` versus the Windows shell opener.

Do not scatter `cfg(target_os)` branches throughout workflow and engine code.

## 18.6 Filesystem durability must be verified per platform

`runtime/fsutil.rs` correctly centralizes durable creation, atomic replacement and secret-file creation, but its assumptions must be tested on the target filesystems:

- replacing an existing file atomically;
- syncing files and directories;
- retaining owner-only descriptor protection;
- advisory lock behavior across spawned children;
- paths on APFS, ext4 and NTFS;
- Docker Desktop shared filesystems;
- long paths, spaces, Unicode, case folding and reserved Windows names.

Where a platform cannot provide the exact Unix primitive, the implementation must preserve the contract or explicitly narrow support. A no-op presented as durability is not portability.

## 18.7 Git remains host-native

The first implementation continues to use the installed Git CLI for repository discovery, worktree creation, branch identity, teardown and inspection.

That is a feature:

- the native harness sees the same worktree path the user sees;
- Git credentials and configuration remain the user's;
- no container/host `.git` path translation is introduced;
- Git's own cross-platform implementation defines path and worktree behavior.

The cross-platform test matrix must include:

- single and multi-repository workspaces;
- source and data paths containing spaces;
- repositories on different drives where supported;
- default branches with unusual names;
- symlink/junction behavior;
- worktree cleanup after daemon interruption;
- file-mode and line-ending differences that affect dirty detection.

A persistent Git worker container is not recommended. It reduces the dependency count cosmetically while introducing a second filesystem namespace and leaving native harnesses dependent on host Git anyway.

## 18.8 Docker transport and Desktop behavior

Docker Desktop is the practical local Engine on macOS and Windows. Docker documents Windows installation and its WSL2 backend at:

- [Install Docker Desktop on Windows](https://docs.docker.com/desktop/setup/install/windows-install/)
- [Docker Desktop WSL 2 backend](https://docs.docker.com/desktop/features/wsl/)
- [Install Docker Desktop on macOS](https://docs.docker.com/desktop/setup/install/mac-install/)

Bollard's local connection support should isolate Unix-socket versus named-pipe transport. Sergeant still needs a real lifecycle/bind-mount test on every platform; transport support in a crate is not proof that the complete execution contract works.

Docker Desktop's user-level socket behavior is documented in its general FAQ: [Docker Desktop socket location](https://docs.docker.com/desktop/troubleshoot-and-support/faqs/general/).

## 18.9 Build and test matrix

The minimum automated matrix should compile and run deterministic core tests on:

```text
ubuntu-latest
macos-latest
windows-latest
```

Tests that require Docker should run only where the CI environment provides a real local Engine with bind mounts. Harness contract tests remain opt-in and execute on measured machines with the real harness installed and authenticated.

A release is not “Windows supported” because `cargo build` succeeded. Platform qualification requires:

```text
install
first daemon start
Git worktree lifecycle
Docker probe
actor workflow
execute workflow
cancel/retry
kill-and-restart recovery
TUI start/exit
runtime descriptor replacement
uninstall/cleanup boundaries
```

The results should become a versioned platform-support table rather than an evergreen prose promise.

---

# 19. Workflow Discovery, Metadata, and Telemetry

The proposed filesystem is useful before Sergeant adds any discovery command. Agents can read `.sergeant/index.md`, use `find`, and grep OKF-style front matter.

Later, the daemon and CLI can provide a richer projection without changing authorship.

## 19.1 Authored workflow identity

Each workflow index should carry human-facing metadata:

```yaml
kind: workflow
name: diagnose-bug
status: published
version: 3
description: ...
tags: [debugging, defect, investigation]
```

The machine execution definition remains `workflow.toml`. The index is discovery material and should not become a competing source for stage order or executor semantics.

The resolver should eventually compute a content identity over the pinned workflow package—not merely trust a human version string. At minimum include:

- normalized workflow descriptor;
- actor stage contexts;
- stage executor metadata;
- explicitly included shared contexts;
- optionally declared helper dependencies when exact pinning is introduced.

The content identity should be journaled with `workflow.bound`.

## 19.2 CLI/API discovery

A future read-only surface may provide:

```text
sgt workflow list
sgt workflow search "investigate flaky test"
sgt workflow show diagnose-bug
```

The API behind it should return:

```text
authored definition
source path
status/version/tags
requirements
compatibility with this host
content identity
observed run statistics
```

Search can begin with exact fields and text indexing. Semantic retrieval should earn itself from a real discovery failure; it does not belong in the first milestone.

## 19.3 Observed telemetry is derived, not written back

Run count, completion rate, stage duration, retry rate, token use, Docker image identity, blocked episodes and failure families come from the journal and DuckDB projection.

They do not get rewritten into `index.md` after every run.

This maintains the existing authority boundary:

```text
workflow files       declared procedure
journal              observed execution truth
DuckDB/API           derived operational view
```

## 19.4 Projection extensions

The analytical schema should eventually distinguish:

```text
workflow name/version/content identity
stage kind: actor | execute
actor harness/profile/model
container requested image/resolved identity/platform
stage attempts and durations
output artifact refs and sizes
executor failure class
```

Representative questions become:

```text
Which workflows produce repeated remediation loops?
Which stage boundaries are most often retried?
Which harness produces the best review acceptance for this workflow?
Which container image revisions changed validation outcomes?
Where do execute stages dominate wall time or log volume?
Which draft workflows have never been measured?
```

The graph can add executor and image nodes only when journal events justify them, preserving its current source-sequence rule.

## 19.5 Measurement status

Authored metadata may declare a review lifecycle:

```yaml
status: draft | candidate | published | deprecated
measurement_policy: standard
```

But facts such as:

```text
measured runs
last measured date
completion rate
median duration
known failure clusters
```

are derived.

A workflow is not “proven” merely because its front matter says `published`. Publication is a governance decision; measurement is evidence.

---

# 20. Migration and Compatibility

The next iteration should be additive across workflow files, APIs and the event journal.

## 20.1 Existing workflows remain valid

A workflow with only:

```toml
[workflow]
name = "software-change"
version = "1"
stages = ["00-prepare", "10-implement", "20-review", "30-close"]
```

continues to resolve as actor stages using the Work actor default. Existing `CONTEXT.md` files require no edit.

The built-in skeleton remains a valid reference workflow. It should not be inflated into every procedure Sergeant knows; richer workflows live in the repository catalog.

## 20.2 Submission compatibility

Current fields retain meaning:

```text
--backend / request.backend   actor default
--profile / request.profile   actor-profile default
origin affinity              default actor routing tier
workspace default            actor default
```

They no longer imply that every stage must use that harness when a stage explicitly names another one.

A legacy client can submit a mixed workflow only when every actor stage can inherit the one default or names its own harness in the workflow.

## 20.3 Journal compatibility

Do not rewrite old journal events.

New event payloads should be versioned where their shape becomes nontrivial. Older unknown event kinds remain ignorable by projections, matching the current reducer posture.

Existing `execution.started` records describe harness executions. New records may introduce an executor-aware schema or separate event families, but replay of the old form must continue to reconstruct legacy Work.

A safe pattern is:

```text
execution.reserved
execution.started
execution.completed / failed / reconciled / stopped
```

with a tagged `executor` payload. If preserving the old `execution.started` payload is cleaner, projections can accept both schema revisions and normalize them internally.

## 20.4 Projection compatibility

DuckDB and graph files remain disposable. Change their schema and rebuild from the journal.

In-memory snapshot schema must be revised only if snapshot loading is adopted; the daemon currently rebuilds from full replay. Do not introduce snapshot migration as collateral work for workflow stages.

## 20.5 API compatibility

Existing work views should continue to expose their current fields. Additive fields may include:

```text
stage.kind
stage.executor
execution.kind
execution.harness
execution.container
workflow.content_identity
workflow.requirements
```

The API revision should change only when a client-visible contract becomes incompatible. Do not overload `backend` to sometimes mean a harness and sometimes Docker.

## 20.6 Profile compatibility

Current profiles are harness launch configuration. Keep them that way.

Container images and execute-stage policy belong in the workflow stage or a future explicitly named execution-environment catalog. They should not be smuggled into a generic profile whose meaning changes by executor kind.

## 20.7 Source compatibility

The repo-to-ICM workflow writes generated candidates under a non-runnable draft root. Promotion to `.sergeant/workflows/` is an explicit reviewed change.

This means an early generator can improve repeatedly without any migration of runtime state and without a `draft` enforcement feature in the engine.

---
# 21. Proposed Milestones

This is a sequence of proofs. Each milestone should have its own bounded contract and gauntlet record. Later milestones do not retroactively become the justification for earlier machinery.

## 21.1 N0 — Finish the measured runtime-remediation line

**Outcome:** The P1 performance and Bug Sprint findings that directly affect executor expansion are either fixed and pinned or explicitly ruled/deferred with triggers.

Priority intersections:

- [#14](https://github.com/miztertea/sergeant-rs/issues/14): Claude stop/join while the core lock is held;
- [#18](https://github.com/miztertea/sergeant-rs/issues/18): `/proc` portability;
- [#19](https://github.com/miztertea/sergeant-rs/issues/19): real-Claude soak;
- [#20](https://github.com/miztertea/sergeant-rs/issues/20): systematic crash-point injection;
- [#4](https://github.com/miztertea/sergeant-rs/issues/4): unbounded retained terminal Work/run memory;
- [#6](https://github.com/miztertea/sergeant-rs/issues/6): single-writer submission plateau;
- [#7](https://github.com/miztertea/sergeant-rs/issues/7): graph-query queueing;
- [#10](https://github.com/miztertea/sergeant-rs/issues/10): cold analytical query scaling.

Not every issue must be solved before authoring workflows. The contract must decide which are prerequisites for adding another external executor and why.

**Non-goal:** No ICM grammar or Docker implementation.

**Gate:** Existing regression, performance and hygiene suites remain green; every fix has a mutation/revert probe; the next engine contract states its allowed lock-hold and memory budgets using the measured baseline.

## 21.2 N1 — ICM filesystem convention and Sergeant reference decomposition

**Outcome:** Commit or otherwise review a human-authored, evidence-backed decomposition of the vendored Sergeant corpus into:

```text
stable instructions
candidate workflows
candidate stages
actor contexts
execute-stage candidates
workflow-local helpers
shared contexts/helpers
engine-gap arguments
uncertain classifications
```

Produce the initial `.sergeant/` catalog convention, OKF-style index shape, provenance record shape, and draft publication boundary.

This milestone is intentionally content-only. It proves that the taxonomy is understandable before asking an agent to apply it.

**Non-goal:** No workflow-engine change. No claim that the manual decomposition is perfect; disagreements are preserved as adjudicated evidence.

**Gate:** Every selected source artifact has a disposition; every candidate workflow can be traced to behavior rather than file type; at least two independent reviewers challenge workflow and stage boundaries; unresolved disagreements are explicit.

## 21.3 N2 — Actor-only `repo-to-icm` workflow on the current engine

**Outcome:** Implement the workflow described in Section 9 using only current ordered actor stages and ordinary repository files.

Run it against the same vendored Sergeant revision used for the manual corpus. Produce:

- normalized behavior units;
- draft workflow packages under the non-runnable draft root;
- a grammar-pressure report;
- a comparison against the manual reference;
- the complete Sergeant trajectory and usage evidence.

Run the same workflow against at least two additional repositories with meaningfully different instruction cultures so the workflow is not tuned only to Sergeant.

**Non-goal:** No automatic publication. No semantic workflow selection service. No nested workflow runtime.

**Gate:** The run is reproducible from a named repository revision; all reference behaviors are either represented or explicitly classified as missed/disputed; generated drafts pass structural lint; fresh reviewers find no unexplained high-impact omission; no engine feature is proposed without a lower-rung failure argument.

## 21.4 N3 — Executor-aware stage model and two-phase external-effect boundary

**Outcome:** Introduce the backward-compatible tagged stage definition and executor-aware execution record while preserving legacy actor workflows.

Implement:

- per-stage actor harness/profile resolution;
- whole-workflow capability preflight before side effects;
- `execution.reserved` or equivalent durable reservation;
- external start/stop waits outside the core lock;
- Claude start-window repair;
- #14/B3 lock-wait repair;
- executor-aware API/projection rendering.

Use the fake backend and at least two distinct fake harness identities to prove per-stage routing without provider tokens.

**Non-goal:** No Docker yet. No native command stage. No workflow recursion.

**Gate:** A workflow runs actor stages on different registered harnesses; unavailable explicit stage harness fails before Work/worktree creation; no external process wait occurs under the core lock; crash injection covers reservation→launch→record windows; all legacy M3/M4 behavior remains green.

## 21.5 N4 — Docker-backed execute stages

**Outcome:** Add the local Docker executor and the `kind = "execute"` schema.

Implement:

- local Engine connection and capability probe;
- image resolution/pull/inspect and immutable identity pinning;
- worktree mount and explicit access policy;
- default no-network/no-privilege posture;
- streaming bounded output capture;
- exit-to-stage mapping;
- exact cancel/retry/recovery/cleanup;
- execute lifecycle events, projections and UI rendering.

The first mixed workflow should contain actor → execute → actor and prove that output evidence is available to the following actor without Sergeant interpreting it.

**Non-goal:** No remote Docker, Kubernetes, Podman, build service, secret injection, arbitrary mounts, or package installation abstraction.

**Gate:** See the Docker-specific acceptance tests in Section 22; all create/start/exit/cancel crash windows are injected; image identity is exact; a large-log test proves bounded memory; no leaked owned containers remain after clean and crash-recovery runs.

## 21.6 N5 — Doctor, installer, and platform qualification

**Outcome:** Turn Git, Docker and harness discovery into one capability surface and qualify the runtime on Linux, macOS and Windows.

Implement:

- platform-native directories with compatibility migration;
- daemon process-instance verification beyond Linux `/proc`;
- adapter-owned installation/workspace probes;
- Docker lifecycle and bind-mount doctor check;
- workflow compatibility preflight/reporting;
- degraded daemon health model;
- multi-architecture release artifacts and a consent-based installer/bootstrap path;
- platform support table backed by executed qualification suites.

**Non-goal:** No silent configuration of harness credentials or organization tools.

**Gate:** The full qualification scenario passes on the declared supported platforms; an unavailable Docker Engine remains diagnosable; actor-only versus execute-workflow admission behaves exactly as documented.

## 21.7 N6 — Workflow discovery and operational measurement

**Outcome:** Add read-only workflow catalog APIs/CLI and derived workflow telemetry.

Implement:

- `workflow list/search/show`;
- authored metadata parsing;
- content identity;
- host compatibility view;
- executor/image/harness projection fields;
- workflow and stage analytics queries;
- UI display of requested versus resolved execution identity.

**Non-goal:** No automatic workflow choice or recommendation engine.

**Gate:** Deleting DuckDB and rebuilding from the journal reproduces every observed statistic; editing front matter cannot alter historical execution facts; search results identify their source files and compatibility evidence.

## 21.8 N7 — Workflow composition only if measured

**Trigger:** The reference decomposition and multiple repo-to-ICM runs show repeated procedures that:

- are invoked from more than one parent workflow;
- need independent durable entry, retry, block, measurement or recovery;
- cannot be represented faithfully by shared context or helpers;
- create harmful duplication when inlined.

**Possible outcome:** A workflow stage that binds and executes another pinned workflow while retaining parent/child trajectory.

**Non-goal until triggered:** No recursion, dynamic DAG, parallel branches, condition language or arbitrary workflow calls.

**Gate:** A separate proposal names the real procedures that require composition, defines cycle/identity/retry semantics, and proves why context inclusion is insufficient.

---

# 22. Acceptance Criteria and Test Strategy

The next iteration should be held to the repository's existing standard: behavior is not complete until a regression test fails when the behavior is removed.

## 22.1 Phase A: current-engine workflow work

The ICM content work passes when:

1. The manual Sergeant reference records the exact vendored source revision.
2. Every reference behavior links to one or more source paths and extracts.
3. Each behavior has one primary ICM classification and any credible alternative.
4. Candidate engine gaps include a lower-rung refutation.
5. Generated workflows live only under the draft root.
6. Actor-stage contexts never claim that an instruction-level include is a durable subworkflow.
7. Shared helper/context references resolve inside the pinned worktree.
8. Structural lint rejects missing contexts, duplicate stages, invalid references, draft output outside the allowed root, and source references that do not exist.
9. The generator preserves uncertainty instead of inventing confidence.
10. Fresh reviewers can reproduce the comparison from the source revision and artifacts.

## 22.2 Reference-comparison criteria

The maiden voyage should report, not hide:

```text
reference behaviors total
matched behaviors
missed behaviors
extra unsupported behaviors
workflow-boundary agreements/disagreements
stage-boundary agreements/disagreements
representation agreements/disagreements
engine-gap agreements/disagreements
unresolved review findings
```

A useful first success criterion is not an arbitrary percentage. It is:

> No reference behavior with a confirmed safety, identity, recovery, delivery or human-decision consequence is silently absent from the final adjudicated output.

Lower-impact misses remain visible measurements for the next workflow revision.

## 22.3 Tagged-stage parser and pinning

Tests must prove:

- every legacy workflow parses identically;
- an undeclared stage metadata table is rejected;
- actor stage contexts remain pinned verbatim;
- execute metadata is pinned in `workflow.bound`;
- unknown stage kinds fail closed;
- actor-only fields on execute stages and execute-only fields on actor stages are rejected rather than ignored;
- workflow content identity changes when any execution-relevant field changes;
- editing files after bind does not change the running Work.

## 22.4 Per-stage harness routing

The matrix must cover:

```text
legacy Work default used by every unqualified actor stage
explicit stage harness overrides Work default
stage profile belongs to its named harness
unavailable explicit stage harness fails before Work creation
no silent provider substitution
one workflow uses harness A → harness B → harness A
retry uses the same pinned stage harness/profile/model decision
restart reconstructs the same decision from the journal
```

Every advertised adapter capability remains paired with an installed-harness contract test.

## 22.5 Two-phase external effects

For every external lifecycle, inject process death or simulated append failure at least at:

```text
before reservation append
immediately after reservation append
external identity created but before started append
external process/container started but before started append
result observed but before result append
result append before stage transition append
stage terminal before stop/cleanup request
cleanup complete before cleanup append
```

Recovery must converge without:

- starting the external effect twice;
- losing an owned external identity;
- adopting an unrelated external identity;
- advancing the wrong attempt;
- reviving terminal Work;
- deleting unproven state.

## 22.6 Core-lock discipline

Instrumentation tests should prove that the authoritative core lock is not held while:

- pulling or inspecting an image;
- creating/starting/waiting/stopping/removing a container;
- waiting for a harness process;
- joining a transcript/log archive worker;
- probing a slow external executable;
- reading a large output stream.

A deliberately stalled fake executor must not block independent read or mutation requests beyond the explicitly allowed journal commit interval.

## 22.7 Docker contract tests

Run against a real local Docker Engine and prove:

1. API negotiation and server/platform evidence.
2. Worktree bind mount with a path containing spaces.
3. Explicit read-only mount prevents writes.
4. Explicit read-write mount permits and preserves writes.
5. `network = "none"` creates no usable external network path.
6. No Docker socket or undeclared host mount exists in container inspect output.
7. Requested mutable tag resolves to journaled immutable image identity and platform.
8. Retry uses the pinned immutable identity after the tag is changed to point elsewhere.
9. Exit `0` completes the stage; nonzero fails it with captured evidence.
10. Cancel addresses only the exact labeled container and cannot affect a look-alike.
11. Restart while running reattaches and records one result.
12. Restart after exit but before result append records the result once.
13. Missing container fails closed.
14. Conflicting deterministic name/labels fail closed and delete nothing.
15. Cleanup leaves zero owned containers for terminal runs.
16. Pull failure and private-registry auth failure produce sanitized actionable evidence.
17. Container-created files remain usable by the host user on every qualified platform.

## 22.8 Large-output and scale tests

A synthetic execute stage should emit at least 1 GiB split across stdout and stderr.

Acceptance:

- complete captured bytes are recoverable by blob reference;
- journal event count does not grow linearly with log lines;
- peak RSS does not grow proportionally with output and remains within a predeclared 64 MiB increment on the reference host;
- API/TUI rendering uses a bounded tail;
- cancellation during output does not deadlock the reader or lose the named capture state;
- disk-full injection records partial/unarchived evidence honestly;
- daemon remains responsive to unrelated work.

Repeat with many short execute stages to expose container lifecycle and journal overhead, not only one long stream.

## 22.9 Docker image and cache pressure

Measurements should include:

- repeated runs with image already present;
- cold pull;
- two concurrent requests for the same missing image;
- retries by digest;
- daemon restart during pull;
- local image pruned between attempts;
- multi-architecture tag on at least amd64 and arm64;
- image inspect and pull event volume.

Sergeant is not an image garbage collector. Tests should prove it does not remove images or caches it did not create.

## 22.10 Doctor tests

Each doctor check must have a failing fixture whose remediation text names a real action.

Cover:

```text
Git missing / broken worktree
Docker endpoint missing
remote Docker context selected
container create denied
bind mount denied
probe container cannot be removed
harness binary missing
harness version too old/unmeasured
harness auth missing
workspace trust missing
journal broken
projection rebuild broken
stale daemon descriptor
ambiguous live process instance
```

`--json` remains stable and does not expose credentials or tokens.

## 22.11 Platform qualification

For Linux, macOS and Windows, execute the same scenario:

```text
install binary
resolve data directory
start daemon
create temp Git repository
run actor-only workflow
run execute-only workflow
run mixed workflow
cancel an active stage
retry a failed stage
kill daemon during active actor/execute stage
restart and reconcile
open/close TUI cleanly
verify descriptor permissions/ACL intent
verify no leaked worktree/container/process
```

WSL qualification must state whether Sgt, Git and Docker access all live inside WSL or cross the Windows boundary. Mixed path models should be unsupported until measured.

## 22.12 Mutation and revert probes

Every fix or new invariant ships with a test that fails when the relevant behavior is removed.

High-value probes include:

- remove executor kind from the pinned workflow;
- silently substitute an unavailable harness;
- move Docker wait back under the core lock;
- re-resolve a mutable image on retry;
- drop ownership-label verification;
- buffer all container output in memory;
- omit one crash-window reconciliation branch;
- treat missing container as failed rather than blocked;
- allow execute stage to inherit a host shell;
- write observed statistics into authored workflow metadata.

The milestone commit history should keep build and fix commits separable enough for the repository's L7/L10 audit practice.

---

# 23. Explicit Non-Goals

This proposal does not authorize:

| Non-goal | Reason |
|---|---|
| Markdown-to-JavaScript workflow compilation | Separate later idea; the current experiment is filesystem procedure and measured grammar pressure. |
| A generalized DAG engine | Ordered stages are sufficient until real procedure proves branching/parallelism is required. |
| Automatic semantic workflow selection | Discovery first; selection remains caller/instruction/configuration-driven. |
| A first-class native command stage | A third crash/authority lifecycle must earn itself from repeated measured need. |
| Native nested/subworkflow execution now | Shared context/helpers can be measured first. |
| GitHub, GitLab, Jira, Linear, ServiceNow or cloud-specific core code | Those are user/organization workflow concerns, not Sergeant invariants. |
| Credential or secret brokering | Harnesses, Git, Docker and organization tools own user authentication. |
| Remote Docker or Docker-over-SSH/TCP | Local bind-mounted worktree contract only. |
| Podman, Kubernetes jobs, Firecracker, E2B or remote workers | Future adapters only after a separate evidence-backed contract. |
| Arbitrary host mounts, privileged containers or Docker socket injection | Too much host authority for the first execute contract. |
| Containerized Git worktree ownership | Host Git and native harness path identity remain aligned. |
| Containerizing native agent harnesses | Breaks the intended use of user-native auth/trust/session state. |
| Silent Docker/harness installation or trust modification | External authorization requires visible user consent and native setup. |
| A Sergeant image/package manager | Workflows name images; Docker resolves them. |
| Automatic image refresh on retry | Retry reproduces the pinned attempt environment. |
| Image/volume/build-cache garbage collection | Separate retention problem; never collateral cleanup. |
| Snapshot adoption or journal compaction | Existing backlog/retention work owns that problem. |
| Mutable telemetry written into workflow files | Authored procedure and observed execution remain separate. |
| Treating front-matter `published` as proof of quality | Publication and measurement are different facts. |

---

# 24. Questions Resolved by Measurement, Not Preference

The proposal chooses safe initial defaults while naming the measurements that may change them.

## 24.1 Stage-kind spelling

Use:

```toml
kind = "actor"
kind = "execute"
```

`execute` is clearer in authored procedure than an abbreviation. CLI/TUI may render `exec` compactly, but the schema remains explicit unless usability measurement strongly favors another term.

## 24.2 Private image authentication

Initial default:

```text
public pull                   supported
private image already local   supported by immutable local identity
private pull                  supported only after credential-helper integration is measured
```

Do not block the entire execute-stage milestone on becoming a Docker credential broker. Do not claim private-pull support until helpers work on qualified platforms.

## 24.3 Workspace access

Require `workspace_access` explicitly for execute stages in the first schema.

This may later gain a repository policy default after authors demonstrate that the repetition adds more noise than safety. The first version should reveal how often stages genuinely need write access.

## 24.4 Artifact declaration

Initial output contract:

- stdout/stderr captured as evidence;
- file mutations remain in the worktree;
- actor stages may inspect known paths described by context;
- no generalized artifact manifest.

Add declared artifacts only when workflows need Sergeant to require, collect, publish or transfer named files as durable objects.

## 24.5 Transitive workflow pinning

Initial pin:

- workflow descriptor;
- actor contexts;
- executor metadata;
- explicit shared-context contents when the include convention is resolved during bind;
- source repository/base SHA and Work branch.

Helper scripts remain Git content at the pinned worktree revision; record their paths and optionally hashes in the repo-to-ICM provenance report.

If exact historical replay or helper mutation during an active Work becomes a demonstrated failure, add declared dependency hashing/copying rather than snapshotting the entire repository into `workflow.bound`.

## 24.6 Container user mapping

No universal choice is made in prose. Qualify candidate policies on each platform and select the lowest one that preserves editable worktree output without weakening image compatibility.

The selected behavior becomes explicit execution evidence.

## 24.7 Network policy

Initial supported value:

```toml
network = "none"
```

A second bounded value such as `outbound` may be added only with a clear Docker implementation and threat model. Arbitrary network names, host networking and implicit default bridge access are out of scope initially.

Repositories whose validation needs package downloads can use an image with dependencies already present or wait for an admitted outbound policy. That friction is evidence, not a reason to silently enable networking.

## 24.8 Per-stage model/profile policy

Harness, profile and model remain distinct.

The first per-stage schema may name harness and profile. Model may continue to come from the profile until multiple real workflows demonstrate that stage-local model pins are necessary independently of profiles.

Every adapter still verifies what actually ran when the harness exposes evidence.

## 24.9 Direct user-context command execution

A native command stage becomes justified only when measurement repeatedly finds checkpoints that:

- require the user's ambient authority;
- require no agent judgment;
- remain meaningful durable boundaries;
- are materially wasteful or less safe when routed through an actor;
- have a recoverable/idempotent lifecycle that can be specified honestly.

Until then, an actor stage performs `gh`, Jira, cloud, or other user commands under repository instructions.

## 24.10 Shared workflow threshold

Propose true composition when at least two independent parent workflows need the same multi-stage procedure and inlining demonstrably loses maintainability or measurement fidelity, while context/helper inclusion demonstrably loses a required durable boundary.

One aesthetically reusable review prompt is not sufficient.

## 24.11 Image refresh

The first operator surface need not expose refresh. Authors update the workflow image reference/version and submit new Work.

If long-lived Work legitimately needs an environment refresh, add an explicit command that creates a new attempt with a journaled re-resolution decision. Never mutate the existing attempt's evidence in place.

---

# 25. Recommendation

The next iteration should be approved as **two coupled but sequential programs**.

## Program A — Make procedure visible before changing the engine

1. Finish the relevant performance/remediation work.
2. Establish the ICM decomposition ladder and filesystem convention.
3. Manually decompose the vendored Sergeant corpus into an adjudicated reference.
4. Build `repo-to-icm` as an ordinary current-engine actor workflow.
5. Run Sergeant as measurement #1 and compare the output to the reference.
6. Run unrelated repositories as measurements #2 and #3.
7. Publish the grammar-pressure report before proposing more workflow machinery.

This program turns the existing repository corpus into evidence and tests whether the current “ordered fresh actor contexts” model is more capable than it first appears.

## Program B — Add only the execution semantics the evidence already supports

1. Split stage execution into actor and execute.
2. Move harness choice to the actor stage while retaining Work defaults.
3. Introduce one two-phase external-effect boundary for every executor.
4. Repair Claude's known start/stop lifecycle debts through that boundary.
5. Add local Docker execution with immutable image evidence and bounded logs.
6. Generalize doctor into Git/Docker/harness capability discovery.
7. Qualify Linux, macOS, Windows and WSL with executed contracts.
8. Add workflow catalog and telemetry projections.
9. Revisit shared workflows and native command stages only when measured.

The architecture should remain:

```text
                           SERGEANT

             durable Work, stages, journal, recovery
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
      Git                  Docker                Harnesses
 host work surfaces   controlled execution   user-authorized actors
        │                     │                     │
        └─────────────────────┴─────────────────────┘
                              │
                    repository-owned procedure
```

The product line is:

> **Sergeant knows Git, Docker, and measured harness contracts. It does not know the user's organization. The repository expresses organizational intent through ICM workflows, contexts, helpers, and stable agent instructions.**

The implementation rule is:

> **Reserve durable intent before external effects, execute outside the core lock, record what actually happened, and fail closed whenever ownership or outcome cannot be proven.**

The workflow rule is:

> **A stage is a durable procedural checkpoint, not a command. Use an actor when judgment or user-context action is required; use execute when a declared environment should perform the same computation every time; use helpers beneath either when the operation is not itself a checkpoint.**

And the governance rule remains Ponytail:

> **No new stage class, composition primitive, scheduler, adapter or metadata system without a real procedure whose lower-rung representation has failed under measurement.**

---

# 26. Source Map

## 26.1 Sergeant-rs implementation and design record

- Repository at audited revision: [`miztertea/sergeant-rs@27c00ef`](https://github.com/miztertea/sergeant-rs/tree/27c00ef7cc9136400b4881974399d834fdce0a47)
- Original proposal: [`reference/proposal-depot-rust-execution-surface.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/proposal-depot-rust-execution-surface.md)
- README: [`README.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/README.md)
- Workflow domain: [`src/domain/workflow.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/domain/workflow.rs)
- Work state: [`src/domain/work.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/domain/work.rs)
- Execution record: [`src/domain/execution.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/domain/execution.rs)
- Engine: [`src/runtime/engine.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/engine.rs)
- Backend contract: [`src/backend/mod.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/backend/mod.rs)
- Claude adapter: [`src/backend/claude.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/backend/claude.rs)
- Routing: [`src/runtime/router.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/router.rs)
- Projection: [`src/runtime/projection.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/projection.rs)
- Recovery: [`src/runtime/recovery.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/recovery.rs)
- Git operations: [`src/runtime/git.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/git.rs)
- Work surfaces: [`src/runtime/surface.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/surface.rs)
- Journal: [`src/runtime/journal.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/journal.rs)
- Blob store: [`src/runtime/blob.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/blob.rs)
- Filesystem durability helpers: [`src/runtime/fsutil.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/fsutil.rs)
- Daemon: [`src/daemon.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/daemon.rs)
- API: [`src/api.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/api.rs)
- CLI and doctor: [`src/cli.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/cli.rs)
- Analytics: [`src/runtime/analytics.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/analytics.rs)
- Graph: [`src/runtime/graph.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/runtime/graph.rs)
- Telemetry: [`src/telemetry.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/src/telemetry.rs)
- M3 acceptance tests: [`tests/m3_execution.rs`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/tests/m3_execution.rs)
- Demo: [`scripts/demo.sh`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/scripts/demo.sh)
- Gauntlet ledger: [`GAUNTLET.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/GAUNTLET.md)
- Lessons: [`LESSONS.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/LESSONS.md)
- P1 performance contract: [`docs/gauntlet/contracts/P1-PERF.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/gauntlet/contracts/P1-PERF.md)
- P1 baseline: [`docs/perf/baseline-2026-08-10.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/docs/perf/baseline-2026-08-10.md)
- Current issue backlog: [`miztertea/sergeant-rs/issues`](https://github.com/miztertea/sergeant-rs/issues)

## 26.2 Original Sergeant corpus

- Vendored snapshot: [`reference/sergeant-upstream`](https://github.com/miztertea/sergeant-rs/tree/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream)
- Root operating instructions: [`AGENTS.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/AGENTS.md)
- Original README and agent-distro framing: [`README.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/README.md)
- No-mistakes: [`.agents/skills/no-mistakes/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md)
- Diagnosing bugs: [`.agents/skills/diagnosing-bugs/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md)
- Prototype: [`.agents/skills/prototype/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/prototype/SKILL.md)
- Sergeant setup: [`.agents/skills/sergeant-setup/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/.agents/skills/sergeant-setup/SKILL.md)
- Load project: [`skills/load-project/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/skills/load-project/SKILL.md)
- Cross-repo work: [`skills/cross-repo-work/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/skills/cross-repo-work/SKILL.md)
- Dispatch: [`skills/dispatch/SKILL.md`](https://github.com/miztertea/sergeant-rs/blob/27c00ef7cc9136400b4881974399d834fdce0a47/reference/sergeant-upstream/skills/dispatch/SKILL.md)
- Upstream public repository: [`callmeradical/sergeant`](https://github.com/callmeradical/sergeant)

## 26.3 IdeaOS, Ponytail, and Bashful

- [IdeaOS Agent Instructions](https://app.notion.com/p/39a27ada618f815aab89daafc635514f?pvs=204)
- [Ponytail Minimality Ladder](https://app.notion.com/p/39a27ada618f8100babadb321a70de9b)
- [Swappable Agent Execution Adapters](https://app.notion.com/p/39a27ada618f8157a9a6c54d56444357?pvs=204)
- [`miztertea/bashful` execution adapter](https://github.com/miztertea/bashful/blob/877f9d5b4d93e85bd02b51ff562efde6188a212b/src/bashful/adapter.py)

## 26.4 Docker

- [Docker Engine API](https://docs.docker.com/reference/api/engine/)
- [Engine API SDK guidance](https://docs.docker.com/reference/api/engine/sdk/)
- [Bind mounts](https://docs.docker.com/engine/storage/bind-mounts/)
- [Docker contexts](https://docs.docker.com/engine/manage-resources/contexts/)
- [Protect Docker daemon access](https://docs.docker.com/engine/security/protect-access/)
- [Pull by immutable digest](https://docs.docker.com/reference/cli/docker/image/pull/#pull-an-image-by-digest-immutable-identifier)
- [Engine API image inspect](https://docs.docker.com/reference/api/engine/version/v1.49/#tag/Image/operation/ImageInspect)
- [Docker login and credential stores](https://docs.docker.com/reference/cli/docker/login/#credential-stores)
- [Docker Desktop on Windows](https://docs.docker.com/desktop/setup/install/windows-install/)
- [Docker Desktop WSL2 backend](https://docs.docker.com/desktop/features/wsl/)
- [Docker Desktop on macOS](https://docs.docker.com/desktop/setup/install/mac-install/)
- [Docker Desktop general FAQ / socket behavior](https://docs.docker.com/desktop/troubleshoot-and-support/faqs/general/)

## 26.5 Rust libraries and standard primitives

- [`std::process::Command`](https://doc.rust-lang.org/stable/std/process/struct.Command.html)
- [`bollard`](https://docs.rs/bollard/latest/bollard/)
- [`bollard::Docker`](https://docs.rs/bollard/latest/bollard/struct.Docker.html)
- [`directories`](https://docs.rs/directories/latest/directories/)
- [`docker_credential`](https://docs.rs/docker_credential/latest/docker_credential/)
- [`sysinfo::Process`](https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html)

These crates are implementation candidates. Their inclusion in this source map is not a claim that the next Cargo dependency set is already decided. Each must satisfy the same measured-contract and Ponytail rules as the rest of Sergeant.

## 26.6 Native harness ecosystem

- [Claude Code CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-usage)
- [OpenAI Codex](https://openai.com/index/codex-now-generally-available/)
- [OpenCode](https://github.com/anomalyco/opencode)
- [Goose](https://github.com/block/goose)

These sources establish that the harnesses exist and expose native execution surfaces. They do not override Sergeant's doctrine that adapter support is measured against the installed executable and authenticated user environment.

---

# 27. The Next Iteration in One Sentence

> **Sergeant-rs should turn repository-owned procedure into measured, durable ICM workflows by first learning from real repositories on its existing actor-stage engine, then adding only two proven execution boundaries—user-authorized native harness actors and Docker-backed deterministic execution—behind the same journal-first, fail-closed, cross-platform runtime.**
