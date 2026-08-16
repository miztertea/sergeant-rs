────────

type: proposal
title: “Sergeant-rs Procedure Authority: Decision Ladders and Library Reconciliation”
description: >-
Proposal to separate Captain skills, actor methods, workflows, stages, and
deterministic mechanisms through a citeable placement ladder; add a
bounded-judgment ladder to every skill, workflow, and stage; and reconcile
the published procedural library behavior-unit by behavior-unit without
changing Sergeant’s Rust runtime unless a measured engine gap survives every
lower rung.
status: proposed
resource: sergeant-rs
tags:

• sergeant-rs
• icm
• workflows
• skills
• bounded-judgment
• rehoming
• procedure
• proposal
timestamp: 2026-08-15
repository: https://github.com/miztertea/sergeant-rs
audit_revision: 3a46b87c17d249655708ed5ac32f6704738776cf
relationship: >-
Content-layer successor and corrective addendum to
reference/proposal-next-iteration-icm-workflows.md. It preserves the P0/N3
execution substrate, extends the existing ICM decomposition ladder with a
driver/admission-boundary test, and operationalizes NORTH-STAR.md’s split
between Captain-owned dialogue and sgt-owned durable execution. It does not
supersede the T-series, P2-JOURNAL, WATCH, adapter, release, or runtime
proposals.

────────

Sergeant-rs Procedure Authority

Decision Ladders and Library Reconciliation

Status: Proposed
Audit basis: miztertea/sergeant-rs@3a46b87
Proposed workstream identifier: ICM-R — subject to owner adjudication
Relationship to the current engine: Preserve it
Primary objective: Make every procedural instruction explicit about where it belongs and what judgment its actor is authorized to exercise
Primary unit of review: One source-cited behavior unit, never merely an existing directory
Hard boundary: No changes to src/, API routes, journal schema, Work state, backend traits, TUI behavior, or workflow.toml grammar in the initial workstream
Code-change threshold: A separately adjudicated engine-gap record proving why every lower rung fails

Sections are numbered for contract citation, following the repository’s proposal convention.

────────

1. Executive Summary

Sergeant’s central execution boundary is not the problem this proposal corrects.

The current runtime already binds one durable Work to one pinned workflow and one shared work surface, then launches a fresh execution for each declared stage attempt. For the Claude adapter, preparation mints a new native session identity for the stage; stage completion stops that execution before the next stage is reserved. A needs_input answer resumes the same incomplete stage execution, while an explicit retry creates a new attempt. That is the execution model ICM-style procedural work requires: one Work, sequential checkpoint contracts, fresh actor context at each checkpoint, and durable handoff through the work surface and journal rather than private conversational memory. R12R14

The defect is one layer above that runtime.

Sergeant currently has a rich body of workflow and skill content, but the library was assembled through a decomposition ladder that does not ask two questions early enough:

1. Who drives this behavior? Captain, a stage actor, deterministic machinery, or Sergeant itself?
2. On which side of Work admission does it happen? Before a Work exists, during a Work, or after a Work returns?

Because the existing ladder asks whether something is a reusable bounded procedure before it asks who drives it, a conversational Captain procedure can look workflow-shaped. task-intake-and-route, for example, turns a user’s request into a choice of execution mode and asks the user about unresolved scope or risk—the very work AGENTS.md and the North Star assign to the interactive harness before sgt run. direct-implementation similarly describes in-session work even though dispatching it as a Sergeant Work changes the meaning of “direct.” The earlier re-homing round correctly moved grilling and grill-with-docs from workflows to skills after real runs showed that durable execution was the wrong interaction surface, but it did not yet generalize that lesson across the whole catalog. R9R15

A second gap remains inside every surviving package: bounded judgment.

Many stage contracts already contain implicit authority rules. validate-and-ship/40-drive-gates, for example, permits the actor to handle auto-fix and no-op findings but explicitly reserves ask-user findings for the user. That is exactly the concept needed. What is missing is a shared, citeable ladder that lets every actor state why it may decide, why it must stop, and which unresolved choice caused needs_input. R16

This proposal therefore creates two related but distinct ladders:

```text
PLACEMENT LADDER
Where does this behavior belong?

PL-0  absorbed or obsolete
PL-1  stable invariant
PL-2  Captain skill
PL-3  actor skill / shared method
PL-4  Sergeant workflow
PL-5  workflow stage
PL-6  deterministic mechanism
PL-7  engine gap
```

and:

```text
BOUNDED-JUDGMENT LADDER
What may this actor decide here?

J5  governing constraint decides
J4  explicit user / bound Work decision decides
J3  settled authoritative record decides
J2  current skill or stage delegates the decision
J1  local, reversible, non-contractual choice
J0  not delegated, conflicting, or risk-changing -> needs_input
```

Both ladders use Ponytail’s governing technique: understand the actual problem, ask the rungs in order, and stop at the first rung that honestly holds. The point is not to cite the highest-sounding representation. The point is to avoid inventing a more powerful surface when an existing, simpler one already fits. Safety, validation, authority, and accessibility are never removed merely to reach a lower rung. R3E4

The library is then reconciled behavior-unit by behavior-unit. Each current skill and workflow is read for its original intention, source evidence, trigger, durable outcome, driver, admission boundary, decisions, inputs, outputs, and review requirements. A package may:

• STAND as authored;
• REHOME wholly to another surface;
• SPLIT across skill, workflow, stage, CLI, and doctrine;
• HARVEST useful behavior into a different package;
• be ABSORBED by functionality Sergeant already owns;
• FOLD non-checkpoint behavior into an owning stage or skill;
• or RETIRE when no surviving behavior remains.

The classification applies to behavior units first. The package disposition is synthesized afterward. This prevents a directory’s current label from deciding its own future.

The first workstream is content-only. It changes Markdown, catalogs, shared contexts, package placement, and—where adjudication changes a workflow—its existing stage content or stage list. It does not require a new Rust type, parser field, Work state, API route, or TUI feature. The runtime already supports fresh stage executions, actor-authored questions, needs_input, respond, retry, pinned stage context, shared work surfaces, and ordinary Git-tracked artifacts. R8R12

The central rules are:

> **Captain shapes and admits the Work. Sergeant executes the admitted Work.**

> **A stage actor may decide only what its current contract delegates. Reaching J0 means stop and ask, not guess.**

> **Classify behavior, not directories. Preserve provenance while rehoming.**

> **A producer does not independently promote its own output. Generated or transformed artifacts remain draft until an independent review or explicit human gate accepts them.**

> **Do not change the engine to enforce a content rule until the rewritten content has been exercised and a concrete failure proves the lower-rung form inadequate.**

The proposed outcomes are:

1. adjudicate the two ladders and their precedence;
2. add one canonical shared bounded-judgment reference and package templates;
3. run a representative pilot across clean skills, clean workflows, likely rehomes, and likely splits;
4. reconcile all 23 published workflows and four current operator skills;
5. validate every surviving package structurally, semantically, and through real execution;
6. measure the residual failures before considering any runtime work.

────────

2. Audit Basis and Proposal Lineage

2.1 Audit revision

This proposal is pinned to main at 3a46b87c17d249655708ed5ac32f6704738776cf, the merge of the Path-to-Mac sprint on 2026-08-15. “Current” in this document means that revision.

Decision ICMR-01 — Ponytail R1: Pin the audit revision. A proposal about source placement and package counts cannot design against an unqualified moving branch.

The surviving integration branch differs from that main line only by a later retrospective edit. It is useful evidence about the work that produced main, but it does not carry an alternate workflow, skill, proposal, or runtime design. This proposal therefore treats main as authoritative and the integration retrospective as supplementary history.

2.2 Proposal family reviewed

This proposal was written as a continuation of the repository’s existing proposal family, not as an independent taxonomy imposed afterward.

P0 execution surface

reference/proposal-depot-rust-execution-surface.md establishes the daemon as the application, Work as durable intent, native harnesses as owned elsewhere, and “general DAG engine” as an explicit non-goal until workflow complexity earns its way in. That boundary remains correct. R1

Next-iteration ICM proposal

reference/proposal-next-iteration-icm-workflows.md is the direct parent of this work. It already rules that:

• the current engine is a credible durable substrate;
• the first ICM arc should run on current ordered actor stages and fresh execution per stage;
• old Sergeant should be manually adjudicated before trusting a generator to grade itself;
• behavior units should be classified through a Ponytail-derived ladder;
• runtime machinery should be added only when a real procedure cannot fit lower-rung forms. R2

This proposal does not replace those decisions. It addresses what the first corpus pass revealed after promotion: the representation ladder needs a driver/admission-boundary test, and every resulting actor surface needs an explicit authority ladder.

S-series

The S-series demonstrates how to run an orthogonal workstream that changes no product behavior, starts with measurement, and refuses production code added merely for convenience or testability. This proposal adopts the same discipline: content first, real execution second, engine change only after a measured residual. R4

T-series

The T-series supplies the closest house-format precedent. It pins an audit revision, states hard boundaries, numbers decisions for citation, preserves existing semantics, and closes with a Ponytail Decision Register. It also says a proposal is a timestamped model that milestone evidence may narrow or amend. This proposal uses the same form. R5

WATCH

The WATCH proposal demonstrates another relevant pattern: expose an already-correct lower-level mechanism rather than inventing a parallel one. Here, the already-correct mechanisms are stage context, shared files, fresh execution, needs_input, and respond; the first answer should be better content contracts, not a new authority engine. R6

Foundation rationalization

The foundation proposal explicitly treats historical boundaries as correct for the system that existed when they were written, then corrects them when actual use proves the system has changed. The present catalog has the same shape: it contains historically adjudicated procedures that predate the current Captain/sgt boundary and later engine capabilities. Reclassification is not an accusation that the earlier work was careless; it is the normal supersession step after the substrate changed. R3

Adapter research

The adapter report distinguishes documented, implemented, measured, and admitted capability. This proposal applies the same distinction to procedure:

```text
authored
structurally valid
semantically reviewed
executed on a representative case
admitted
```

A package that merely parses and walks every stage is not yet semantically validated. R7

2.3 Repository materials reviewed

The audit included:

• NORTH-STAR.md and AGENTS.md for ownership of dialogue, judgment, intent shaping, workflow selection, execution, monitoring, and response;
• docs/icm/convention.md and docs/icm/record-shapes.md for filesystem layers, Inputs tables, behavior units, classification records, and publication boundaries;
• .sergeant/workflows/repo-to-icm/_config/icm-ladder.md for the current decomposition ladder and its first-matching-rung rule;
• docs/icm/promotion-spec-2026-08-11.md, docs/icm/retriage-2026-08-11.md, and docs/icm/re-homing-record-2026-08-12.md for the prior promotion, reclassification, and rehoming methods;
• the 23-package workflow catalog and four current operator skills;
• representative packages including grilling, task-intake-and-route, sergeant-setup, validate-and-ship, code-review, and repo-to-icm;
• workflow loading, stage reservation, Claude preparation/launch, stage progression, needs_input, respond, and retry implementation;
• the current N4 draft to ensure this proposal does not accidentally claim an unassigned N-series identifier or reopen execute-stage work. R8R11R13R15R17

2.4 External research reviewed

Four external sources corroborate the repository’s direction:

1. Interpretable Context Methodology. ICM argues that sequential, human-reviewed workflows often do not need framework-level orchestration: numbered folders can represent stages, Markdown can carry stage context, and local scripts can own mechanical work. It separates stable reference material from per-run working artifacts, treats every output as an edit surface, and makes stage inputs and outputs explicit. E1
2. Anthropic’s effective-agent patterns. Anthropic distinguishes workflows—predefined orchestration paths—from agents that dynamically choose their own process and tools. It recommends the simplest adequate pattern, adding complexity only when outcomes demonstrably improve, and explicitly recognizes human checkpoints and blockers. E2
3. Agent Skills specification and authoring guidance. The Agent Skills standard treats a skill as a scoped directory with SKILL.md plus optional references, scripts, and assets, loaded progressively. Its authoring guidance recommends synthesizing from real project artifacts, refining through real execution, designing coherent units, and matching prescriptiveness to task fragility. E3
4. Ponytail. Ponytail’s ladder asks whether work needs to exist, already exists, is available in the standard or native environment, or can be solved by a smaller existing form before admitting new implementation. It explicitly says the ladder follows problem understanding and never trades away safety or validation. E5

These sources are corroboration, not authority over Sergeant. The repository’s own North Star, contracts, measured behavior, and owner rulings remain primary.

2.5 Evidence hierarchy

The proposal uses this order:

```text
owner rulings and current measured Sergeant behavior
        >
current source, tests, journal contracts, and admitted package content
        >
committed proposals, gauntlet records, and retrospectives
        >
connected design records in Notion
        >
official external specifications and primary publications
        >
proposal inference
```

Where this document predicts a package disposition, it labels the prediction as a hypothesis. No current package is reclassified merely because this proposal names it as suspicious.

────────

3. Findings

3.1 The stage execution boundary already matches the intended model

A workflow is resolved and pinned before execution. At stage entry, the engine selects the current stage’s pinned executor, creates a new execution identity, passes the Work intent plus the current stage’s CONTEXT.md, reserves the native identity, and launches outside the core lock. When that stage reports completion, the engine records stage.completed, stops its execution, and reserves the next stage. R12

For Claude, prepare mints a session UUID before launch, and launch starts the first turn with that session identity. The prompt contains the Work intent and that stage’s context. A successful turn with no actor-authored question becomes StageCompleted; a question becomes NeedsInput and resumes through the same session when sgt respond supplies the answer. R14

Therefore:

```text
Work
  -> stage 00 / execution A / native session A
  -> stage 10 / execution B / native session B
  -> stage 20 / execution C / native session C
```

is already the product’s actual model.

Finding ICMR-F1: No runtime rewrite is needed to make one folder equal one fresh stage execution. The current code already does it.

3.2 The ICM filesystem is richer than the runtime—and deliberately so

The convention defines Layer 1 orientation, Layer 2 stage contracts, Layer 3 stable references, and Layer 4 per-run artifacts. Downstream stages name upstream outputs in Inputs tables, and the convention explicitly says file handoff—not shared conversation state—is how context flows between fresh stage executions. Only workflow.toml and each stage’s CONTEXT.md are interpreted by the engine today; the remaining structure is ordinary Git content the actor navigates. R8

This is not an accidental omission. The first ICM program intentionally tested whether human-readable procedure could carry the semantics before teaching the runtime more grammar.

Finding ICMR-F2: Inputs, outputs, references, authority, and review policy can all be strengthened initially as content contracts. The fact that the engine does not parse them is not itself an engine gap.

3.3 The current decomposition ladder lacks the driver/admission-boundary discriminator

The existing ladder is rigorous about invariants, reusable procedures, durable checkpoints, judgment, helpers, shared content, and engine gaps. It also contains an important safeguard: a classifier must answer each rung’s own question rather than jumping to a lower rung because a behavior “looks deterministic.” R11

But §6.2 currently asks:

> Is it a reusable procedural outcome with a recognizable trigger, bounded outcome, and completion condition?

That condition is necessary for a workflow, but not sufficient. A Captain skill may also have a trigger, bounded outcome, and completion condition. The missing discriminator is:

```text
Does this procedure receive an already-admitted Work intent,
or is its job to converse with the user and decide what Work should exist?
```

The current execution-surface amendment in docs/icm/convention.md partly recognized this by separating workflow, CLI surface, and operator skill and by adding an absorbed-by-engine check. The prior re-triage and re-homing records used those categories successfully. The remaining catalog still needs a uniform rung order that applies those questions before “workflow.” R9

Finding ICMR-F3: The current ladder is a representation ladder without a complete ownership/admission axis. That omission makes Captain work over-promotable to workflow.

3.4 The library contains local examples of bounded judgment, but no shared language

validate-and-ship/40-drive-gates already distinguishes three kinds of gate finding:

• auto-fix: the actor may authorize within the contract;
• no-op: no substantive decision is needed;
• ask-user: the finding challenges deliberate intent or product behavior and belongs to the user.

It also records a disputed standing-consent exception rather than silently resolving the conflict. R16

That contract is stronger than generic instructions such as “use judgment” or “ask when unsure.” It says who may decide what and why. Similar authority rules appear across security review, destructive actions, scope changes, project setup, and shipping gates, but each package expresses them in local prose.

Finding ICMR-F4: Sergeant already contains bounded-judgment behavior. It lacks a normalized ladder and required local specialization that make the behavior citeable and reviewable across packages.

3.5 Structural promotion proved execution order, not procedural truth

The promotion spec ran representative workflows against the unscripted fake backend and asserted that:

• the workflow bound the declared stage order;
• every stage entered and completed;
• each stage received a distinct execution identity;
• the Work reached completed.

The same spec explicitly records what that gate did not prove. sergeant-setup/30-project-interview depended on repeated human answers, yet the unscripted fake run completed it without exercising a single needs_input transition. A clean mechanical run therefore coexisted with an untested semantic requirement. R10

Finding ICMR-F5: “The engine can walk the package” and “the package performs its intended procedure” are separate admission claims. Every surviving skill and workflow needs both structural and semantic validation.

3.6 The prior re-homing round was directionally correct but package-level

The 2026-08-12 re-homing round retired packages already absorbed by sgt, moved conversational packages to skills, and split setup/project behavior across CLI and workflow surfaces. It preserved provenance and corrected catalogs. That was necessary and should be treated as precedent. R17

Its limitation is granularity. A package was often assigned a top-level verdict even when individual behavior units belonged on different surfaces. The user requirement for this round is stricter: understand every step and the intention of the package, then stand, split, harvest, or retire from the behavior-unit evidence upward.

Finding ICMR-F6: The next pass must classify units first and synthesize package disposition second.

3.7 No code prerequisite is visible

The current engine already supports:

• ordered, pinned stage contexts;
• fresh execution per stage attempt;
• actor and execute stage kinds;
• per-stage harness/profile binding;
• actor-authored questions;
• needs_input and respond;
• explicit retry;
• shared worktree artifacts;
• journal evidence and output pointers;
• content-addressed raw transcripts;
• a publication boundary between draft and admitted workflow content. R8R13

A placement ladder and authority ladder require none of the following to exist first:

```text
new Work states
new event kinds
new API fields
new workflow.toml fields
new TUI controls
new backend capability
new artifact database
new scheduler
```

Finding ICMR-F7: The first complete reconciliation can be attempted with zero Rust changes. Code work is a possible conclusion of the campaign, not its prerequisite.

────────

4. Invariants

Every change under this proposal is checked against these rules.

4.1 The journal remains the only durable runtime truth

The ladders are authored procedure and review evidence. They do not create a second Work state, execution state, or runtime ledger. Work transitions still occur only through the existing journal path.

4.2 Work state remains distinct from process and conversation state

A stage actor’s confidence, prose, or process exit does not itself redefine the Work. The backend’s explicit semantic signal and existing engine transitions remain authoritative.

4.3 One declared stage attempt remains one fresh execution

Rewriting a workflow may merge or split checkpoints, but every surviving stage retains the current fresh-execution contract. No content rewrite reintroduces one long-lived actor “walking folders” through conversational memory.

4.4 Execution is not dialogue

Captain owns live user conversation, intent shaping, workflow selection, and interpretation. Sergeant owns durable execution and message mechanics to an admitted Work. A package whose primary value is live conversation belongs to a Captain skill unless a future owner ruling changes R-NS-6. R15

4.5 Procedure remains data

The ladders, authority envelopes, stage contracts, inputs, outputs, review rubrics, and re-homing records remain versioned repository content. This proposal adds no procedure-specific branch to Rust.

4.6 Authority narrows as work moves inward

The precedence is:

```text
binding law / policy / safety / repository doctrine
        -> explicit user intent and Work constraints
            -> workflow authority envelope
                -> stage or skill authority envelope
                    -> actor-local choice
```

A lower layer may narrow authority. It may not silently widen a higher layer.

4.7 Cross-stage context is explicit

A stage never relies on “what I just planned,” “the change I just made,” or any other pronoun whose antecedent is another execution’s private context. Contract-bearing handoffs are named files, Work fields, or authoritative observations declared in the Inputs table.

4.8 Lowest viable rung

Every placement decision stops at the first honest rung. An engine gap that could have been a skill, workflow, stage, shared context, CLI verb, execute stage, or helper is rejected. R3E5

4.9 A producer does not self-promote

A producing skill or workflow may perform its own self-check, but a promotable output requires an independent review position or explicit human acceptance. Review may be a separate workflow, a different stage execution with a genuinely independent contract, or a human gate. “The same actor says its own output is correct” is evidence, not admission.

4.10 Measured behavior outranks authored confidence

A package can be structurally correct and semantically wrong. Real execution traces, missed holds, false activations, unnecessary escalations, and ambiguous outcomes feed back into the package source. R7

4.11 Proposals are timestamped models

Milestone evidence may narrow or supersede this proposal. Implementation must not silently drift away from it; amendments are recorded and reviewed. R5

────────

5. The Placement Ladder

5.1 Purpose

The Placement Ladder answers one question:

> **What is the lowest-authority, smallest-surface representation that faithfully owns this behavior?**

It is applied to one normalized, source-cited behavior unit at a time. Ask the rungs in order and stop at the first one that holds.

A behavior record cites its rung as PL-N. A package disposition may contain many rungs because a package may be split or harvested.

5.2 PL-0 — Absorbed or obsolete

Question: Does the current product, platform, or admitted procedure already own this behavior, or is the source mechanism a historical implementation whose policy has been superseded?

Destination: Retire the duplicate or obsolete mechanism. Rehome any surviving policy to its actual owner.

Examples:

• a workflow wrapper around shipped sgt respond;
• install or repair procedure already owned by sgt init / sgt doctor;
• tmux-pane lifecycle behavior structurally replaced by native execution identity;
• duplicate shipping instructions already owned by validate-and-ship.

Required evidence: Name the owning surface and show the behavior equivalence. “Seems redundant” is not enough.

5.3 PL-1 — Stable invariant

Question: Must this rule apply broadly across many tasks and change rarely, independent of one trigger or stage?

Destination: AGENTS.md, repository doctrine, policy, or another stable instruction surface.

Examples:

• preserve source history;
• do not silently substitute a harness;
• use real respond, retry, and cancel operations rather than fabricating state in prose;
• secrets never enter committed procedure or output.

A workflow-specific rule is not promoted here merely because it is important.

5.4 PL-2 — Captain skill

Question: Is the behavior driven by the interactive harness before Work admission, between Work items, or while interpreting Work with the user?

A Captain skill commonly:

• conducts live dialogue;
• shapes or revises intent;
• identifies missing user decisions;
• decides whether work should remain direct or become durable Work;
• selects a workflow, repositories, profile, and envelope;
• turns user conversation into a bounded submission;
• interprets findings or results with the user;
• decides what follow-on Work to create.

Destination: skills/<name>/SKILL.md, loaded and run in the current harness session.

Discriminator: If the procedure’s job is to decide what Work should exist, it cannot itself require an already-existing Work merely to make that decision.

5.5 PL-3 — Actor skill or shared method

Question: Is this a reusable reasoning or operating technique that an actor applies inside a Captain interaction or a workflow stage, without owning a complete durable Work lifecycle?

Examples:

• a TDD technique;
• a document-critique rubric;
• threat modeling;
• root-cause hypothesis ranking;
• evidence quality assessment;
• a reusable reconciliation method.

Destination: A skill, workflow-local reference, or .sergeant/common/ shared context according to reuse and activation needs.

Discriminator from PL-2: Captain skills own interaction and Work steering. Actor skills own a reusable method and may be invoked by Captain or by a stage actor.

Discriminator from PL-4: An actor skill does not independently own one admitted intent from start to terminal Work outcome.

5.6 PL-4 — Sergeant workflow

Question: Given an already-defined intent, repositories, constraints, and expected outcome, can Sergeant execute this procedure durably from admission to a terminal result whether or not the Captain remains present?

A workflow must have:

• a recognizable trigger after intent shaping;
• a bounded outcome;
• a completion condition;
• explicit inputs and outputs;
• durable checkpoints where needed;
• a coherent authority envelope;
• a result that is meaningful independent of the original conversation continuing.

Examples likely to fit include independent code review, defect diagnosis, external-skill vetting, merge-conflict resolution, and repository-to-ICM generation—subject to behavior-unit review.

A workflow may ask a bounded question during execution, but conversation cannot be its primary product.

5.7 PL-5 — Workflow stage

Question: Inside an admitted workflow, is this a meaningful durable checkpoint whose independent execution, evidence, retry, cost, failure, or authority boundary matters?

Apply the existing reimplementation test:

> If the current mechanism were replaced tomorrow, would operators still care that the procedure entered, blocked in, retried, completed, or failed at this boundary?

A stage is justified when:

• a fresh execution context reduces contamination or preserves independence;
• downstream work depends on an explicit artifact or verdict;
• a different actor/harness/profile is appropriate;
• the checkpoint may pause, fail, retry, or be measured independently;
• the authority envelope materially changes;
• independent review must not share the producing execution.

A heading, command, or script is not automatically a stage.

5.8 PL-6 — Deterministic mechanism

Question: Is the behavior repeatable machinery whose output follows mechanically from declared inputs and whose invocation does not itself require substantive judgment?

Possible destinations depend on scope:

• CLI verb: independent operation on Sergeant or estate state;
• execute stage: meaningful durable checkpoint implemented mechanically;
• workflow-local helper: machinery subordinate to one stage;
• shared helper: identical contract reused by multiple workflows;
• ordinary tool invocation: no new packaged surface at all.

PL-6 is evaluated after PL-5 so deterministic work that is still a meaningful durable checkpoint can remain an execute stage.

5.9 PL-7 — Engine gap

Question: Can the behavior not be represented faithfully because the runtime itself must own a new durable fact—identity, ordering, retry, recovery, authorization, isolation, evidence, or settlement semantics?

This is the final rung.

An engine-gap record must include the existing six-field template:

```text
behavior
source_evidence
lower_rungs_attempted
why_each_fails
minimum_runtime_capability_required
observable_acceptance_test
```

Identical generic reasons copied across lower rungs are rejection evidence: they show the ladder was not actually applied. R11

5.10 Scope and disposition modifiers

The rung identifies representation. Separate fields identify the result of applying it:

|Modifier  |Meaning                                                                   |
|----------|--------------------------------------------------------------------------|
|`STAND`   |Existing package remains the correct surface and identity                 |
|`REHOME`  |Whole package moves to another surface                                    |
|`SPLIT`   |Behavior units become two or more owned artifacts                         |
|`HARVEST` |Useful units move into another package; original identity does not survive|
|`ABSORBED`|Existing product surface already owns the behavior                        |
|`FOLD`    |Unit becomes context or a helper inside an owning package                 |
|`RETIRE`  |No surviving behavior remains after normalization                         |

Shared/local is another modifier, not a rung. A method becomes shared only when two or more consumers use the same contract.

5.11 Required classification record

Every behavior-unit classification records:

```yaml
behavior_id: BU-...
placement_rung: PL-N
representation: ...
driver: captain | stage-actor | deterministic | runtime
admission_boundary: pre-work | in-work | post-work | always
owning_package: ...
disposition: STAND | REHOME | SPLIT | HARVEST | ABSORBED | FOLD | RETIRE
rationale: why this rung, not the adjacent rungs
alternatives_considered:
  - ...
source_evidence:
  - ...
```

The current classification-record discipline—rationale, alternatives, and adjudication before settlement—continues to apply. R11

────────

6. The Bounded-Judgment Ladder

6.1 Purpose

The Bounded-Judgment Ladder answers a different question:

> **What authority allows this actor to decide this material question without returning to a human or higher authority?**

The actor checks J5 through J0 in order. It cites the first rung that actually resolves the decision. If two higher rungs conflict, the result is J0, not silent precedence invented by the actor.

The ladder governs material decisions: choices that affect scope, acceptance, user-visible behavior, security, privacy, authority, destructive action, irreversible state, promoted artifacts, or a downstream stage’s interpretation. Trivial tool mechanics do not require a citation unless the stage contract says otherwise.

6.2 J5 — Governing constraint

Basis: Binding law, safety policy, repository doctrine, authority boundary, workflow prohibition, or stage contract requires or forbids the action.

Actor behavior: Apply the constraint and cite its source. A lower rung cannot override it.

Examples:

• do not expose a secret;
• do not force-push a preserved branch;
• review actors do not edit the implementation under review;
• a stage may not widen repository scope;
• an output marked draft may not be promoted by its producer.

If two governing constraints conflict, land at J0.

6.3 J4 — Explicit user or bound Work decision

Basis: The user, accepted Work intent, acceptance criteria, exclusions, repository selection, or explicit standing authorization already decides the question and is compatible with J5.

Actor behavior: Apply the recorded decision without asking the user to reconfirm it.

Examples:

• the user explicitly selected PostgreSQL;
• acceptance requires backward compatibility;
• exclusions forbid a schema migration;
• the user authorized one named destructive cleanup action.

Standing authorization is scoped. It never overrides J5 and is not generalized beyond what was actually granted.

6.4 J3 — Settled authoritative record

Basis: An accepted upstream artifact, ADR, prior stage output, pinned specification, authoritative system observation, or previously adjudicated decision settles the question.

Actor behavior: Reuse it and cite the artifact. Do not reopen settled intent merely because another choice is possible.

Examples:

• the accepted architecture record names the serialization format;
• a prior stage’s reviewed contract fixes the target API;
• the repository manifest authoritatively identifies the owning repo;
• a review ledger already adjudicated an identical finding.

A draft, self-authored output, stale observation, or unsupported inference does not qualify as J3.

6.5 J2 — Delegated actor judgment

Basis: The active skill or stage explicitly delegates this class of decision within named bounds.

Actor behavior: Inspect evidence, choose, and record the rationale and rung.

Examples:

• classify a finding’s severity under a supplied rubric;
• select which evidence sources to inspect;
• rank falsifiable hypotheses;
• choose among implementation designs that all preserve the bound contract;
• decide whether two findings are substantively duplicates.

The package must name the delegation. “Use your best judgment” without a bounded decision class is not a J2 grant.

6.6 J1 — Local, reversible, non-contractual choice

Basis: The choice is local to the current implementation, easily reversible, and cannot change scope, authority, security, data, public behavior, acceptance, or another actor’s contract.

Actor behavior: Choose conservatively. Record the choice when it materially affects review or maintenance; otherwise proceed without ceremony.

Examples:

• a private variable name;
• ordering equivalent local helper functions;
• selecting an already-installed formatter command where output is identical;
• choosing a temporary filename inside the assigned work surface.

A choice is not J1 merely because the actor believes the risk is low.

6.7 J0 — Not delegated, conflicting, or risk-changing

Basis: No higher rung resolves the question, evidence conflicts, authority is missing, or the choice would change scope, policy, security/privacy posture, destructive effects, irreversible state, public behavior, acceptance, or promotion.

Actor behavior: Do not guess. Stop before the undecided effect and produce one precise question.

For a workflow stage:

1. record the unresolved decision;
2. state which rungs were checked and why they did not settle it;
3. preserve the evidence already gathered;
4. state the actor’s recommended answer when one can be responsibly offered;
5. end the turn with one direct question so the existing backend signal places the Work in needs_input.

For a Captain skill, ask the question live and wait for the user’s answer before continuing.

Canonical shape:

```markdown
## Decision required — J0

**Decision:** May the public response schema change?
**Checked:** J5 no policy grants a breaking change; J4 acceptance requires
compatibility; J3 no accepted migration record exists; J2 this stage may
propose designs but may not alter public behavior; J1 does not apply.
**Evidence:** ...
**Recommendation:** Preserve the existing schema and add an optional field.
**Question:** Should this Work preserve backward compatibility, or may it make
an intentional breaking API change?
```

6.8 Conflict rule

The ladder is not a numeric override table. A user request that conflicts with binding policy does not become valid because J4 is “below” J5; the conflict itself is J0 unless the governing source defines an authorized exception process.

6.9 Authority inheritance

The hierarchy is narrowing only:

```text
repository / organizational doctrine
        -> Work intent and explicit user decisions
            -> workflow authority envelope
                -> stage or skill specialization
                    -> actor decision
```

A stage may narrow its workflow. A skill loaded by a stage may narrow the stage. Neither may widen the parent contract.

6.10 Decision evidence

Every package defines where material decisions are recorded. Recommended default:

```markdown
| Decision | Rung | Evidence | Resolution |
|---|---|---|---|
| ... | J2 | ... | ... |
```

The table may live in a declared Layer-4 artifact, review report, proposal, or final summary. The requirement is traceability, not one universal filename.

────────

7. Required Instruction Shapes

7.1 One canonical ladder source

The full bounded-judgment ladder is written once, proposed location:

```text
.sergeant/common/contexts/bounded-judgment.md
```

The placement ladder is primarily an authoring and adjudication method and belongs in the normative ICM documentation plus repo-to-icm’s stable classification configuration:

```text
docs/icm/convention.md
.sergeant/workflows/repo-to-icm/_config/icm-ladder.md
```

No package copies the full definitions. Packages reference the canonical source and add only their local specialization. This follows the existing anti-duplication and shared-context rules. R8

Decision ICMR-02 — Ponytail R2: Reuse the existing shared-context and ICM configuration surfaces. Do not introduce a runtime authority schema.

7.2 Workflow-level authority envelope

Every workflow Layer-1 CONTEXT.md gains a concise section:

```markdown
## Authority envelope

This workflow receives an already-admitted Work intent.

### Workflow may decide
- ...

### Workflow may not decide
- ...

### Human or Captain gates
- ...

### Decision record
Material decisions cite J-rungs in ...
```

Layer 1 remains orientation, not a super-stage. The authority envelope applies across the run and is narrowed by each stage.

7.3 Stage-level bounded judgment

Every actor stage gains:

```markdown
## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- ...

### J1 — local choices allowed
- ...

### J0 — must become `needs_input`
- ...

### Completion boundary
This stage may complete only when ...

### Decision evidence
Write material decisions to ...
```

An execute stage does not need model judgment, but its stage contract still states which outcomes are mechanical and which ambiguous conditions block rather than guess.

7.4 Skill-level bounded judgment

Every skill gains the same conceptual section, adapted to its driver:

```markdown
## Bounded judgment

### This skill may decide
- ...

### This skill must ask the user
- ...

### This skill must not do
- ...

### Durable handoff
When the conversation produces a Work-worthy intent, submit ...
```

Captain skills explicitly state whether they may create Work, merely recommend a workflow, or only shape an artifact for later confirmation.

Actor skills state the decision classes they contribute when loaded inside another contract. They do not claim authority independently of the caller.

7.5 Inputs remain explicit

Every contract-bearing dependency appears in the stage Inputs table. The bounded-judgment source itself is listed when its content is required for the stage. Exploration remains legal; hidden dependency does not. R9

7.6 Outputs and review policy

Every package that produces a promotable artifact states:

• the artifact class;
• draft location;
• who or what independently reviews it;
• acceptance criteria;
• promotion action;
• failure/remediation path.

This is initially content, not engine grammar.

7.7 No conversational continuity language

During reconciliation, remove or rewrite phrases such as:

```text
what you just planned
what you just implemented
the conclusion you reached earlier
continue where you left off
```

when their antecedent is another execution’s private context. Replace them with explicit inputs:

```text
../00-prepare/output/plan.md
../10-implement/output/implementation-record.md
current Git diff
bound Work acceptance criteria
```

7.8 Completion boundary

Each stage states what must become true before it may report completion. This does not yet make the engine validate the artifact. It gives the actor and reviewer a falsifiable contract and creates the evidence needed to decide later whether runtime enforcement is necessary.

────────

8. Library Reconciliation Method

8.1 Unit of work

The review unit is one source-cited behavior unit. Existing directories, names, and stage boundaries are evidence, not conclusions.

This extends the existing repo-to-icm method rather than inventing another pipeline. Its current sequence—contract, inventory, harvest, normalize, classify, synthesize, draft, self-check, adversarial review, reconcile—is the correct shape. The classification method and synthesis targets are what change. R2

8.2 Step 1 — Contract

For each package, record:

• source identity and current status;
• original intention;
• trigger;
• bounded outcome;
• current driver;
• current admission boundary;
• known consumers and delegations;
• expected artifacts and promotion effects;
• historical provenance and prior adjudications.

No reclassification begins before the original intention is stated in a form the owner can dispute.

8.3 Step 2 — Inventory

Read the entire package:

• SKILL.md or workflow CONTEXT.md;
• every stage CONTEXT.md;
• workflow.toml and indexes;
• references, helpers, scripts, templates, and output declarations;
• every delegated package;
• relevant AGENTS.md, North Star, ADR, issue, and historical source citation;
• real execution evidence where it exists.

A package is not judged from its name or first page.

8.4 Step 3 — Harvest

Extract atomic behavior units with exact source citations, following the current record-shape rules. Split independently triggerable behaviors. Separate behavioral intent from historical mechanism. R11

8.5 Step 4 — Normalize

Rewrite each unit in implementation-independent language without losing its source meaning.

Example:

```text
historical mechanism:
  inspect tmux pane status before waking worker

normalized behavior:
  do not infer Work progress solely from process liveness; reconcile durable
  execution evidence before resuming or replacing an actor
```

8.6 Step 5 — Placement classification

Apply PL-0 through PL-7 in order. Record driver, admission boundary, rationale, and alternatives.

The classifier may not classify a behavior as workflow merely because it has a trigger and outcome. It must first answer PL-2 and PL-3.

8.7 Step 6 — Authority classification

For every behavior that survives in a skill, workflow, or stage, identify:

• what J5 constraints bind it;
• which explicit J4 decisions it consumes;
• which J3 records it treats as settled;
• which decision classes it delegates at J2;
• which local choices remain J1;
• which decisions must land at J0.

A package with no meaningful answer is not ready to publish.

8.8 Step 7 — Synthesis

Cluster by behavioral contract, driver, and durable outcome—not by source file.

Synthesis may produce:

• one Captain skill;
• one actor skill;
• one workflow;
• several packages;
• additions to an existing package;
• a CLI/engine documentation note;
• no surviving package.

A source file mapping one-to-one onto a new package is not evidence of correctness; it may be file-shape mirroring, the same failure already documented in the ICM ladder. R11

8.9 Step 8 — Draft and rehome

All generated or substantially rewritten packages land in a reviewable draft location or branch. They do not replace admitted procedure until validation completes.

Moves preserve provenance. A rehome is a move plus a disposition record, not deletion followed by reconstructed prose.

8.10 Step 9 — Self-check

The producing actor verifies:

• every behavior unit is dispositioned;
• citations resolve;
• every rung rationale is specific;
• stage order and Inputs resolve;
• authority envelopes are complete;
• every generated artifact has a review path;
• no package assumes hidden conversation continuity.

Self-check is necessary but not promotion authority.

8.11 Step 10 — Independent adversarial review

A separate actor position challenges:

• source fidelity;
• rung order;
• Captain/workflow boundary;
• stage/helper boundary;
• authority grants and missing J0 cases;
• package identity and naming;
• duplicated or drift-prone content;
• false pairing assumptions;
• unjustified engine gaps.

8.12 Step 11 — Reconcile and publish

Every finding is accepted, rejected, merged, or parked with rationale. The owner or delegated promotion gate accepts the final disposition. Catalogs, routing tables, delegations, and provenance update in the same change.

8.13 Package adjudication record

Canonical record shape:

```markdown
# Package adjudication: <name>

## Original intention

## Current trigger and outcome

## Driver and admission boundary

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|

## Surviving package design

## Inputs and outputs

## Review and promotion policy

## Alternatives considered

## Final disposition
STAND | REHOME | SPLIT | HARVEST | ABSORBED | RETIRE

## Validation evidence
```

────────

9. Validation Contract

9.1 Validation has five distinct claims

A package may be:

1. source-valid — behavior is traceable to real evidence;
2. placement-valid — the correct surface owns it;
3. authority-valid — its actor’s decisions and escalation boundary are explicit;
4. structurally valid — files, stages, inputs, outputs, and catalogs resolve;
5. execution-valid — representative use produces the intended outcome and holds at the intended boundaries.

No one claim substitutes for another.

9.2 Skill validation

Each skill is tested for:

• correct trigger activation;
• false-positive activation on nearby tasks;
• whether it needs live dialogue;
• whether it asks only genuine decisions rather than discoverable facts;
• whether J2/J1 autonomy is useful rather than over-prescriptive;
• whether J0 causes one precise question and a real pause;
• whether any produced artifact has a durable handoff and review policy;
• whether the skill remains coherent and progressively disclosed. E3

9.3 Workflow validation

Each workflow is tested for:

• admission with already-defined intent;
• stage order and fresh execution identities;
• explicit cross-stage file handoffs;
• stage completion against declared outcomes;
• needs_input behavior on at least one real or scripted J0 case when the workflow can ask;
• retry without hidden conversational state;
• terminal output and review disposition;
• operation when Captain is no longer present.

An unscripted fake-backend walk remains useful as a mechanical gate, but it does not count as semantic validation. R10

9.4 Decision-ladder validation

Reviewers challenge:

• whether a cited rung actually authorizes the decision;
• whether a higher rung was ignored;
• whether a J1 choice hides a product or authority decision;
• whether J0 questions are overused to transfer ordinary work back to the user;
• whether standing authorization was generalized beyond its scope;
• whether a stage widened its parent workflow.

9.5 Independent review and promotion

The default promotion chain is:

```text
produce draft
    -> self-check
        -> independent review
            -> remediation or acceptance
                -> explicit promotion
```

“Independent” means a distinct review contract and execution position. A later stage may qualify when it receives only the artifact and review rubric, does not inherit the producing conversation, and cannot silently edit the subject it reviews.

9.6 Skill/workflow relationships are many-to-many

This proposal rejects a mandatory one-to-one pair.

Several skills may produce plans, proposals, ADRs, or specifications that all route to one document-review policy. A PR-finishing skill and an automated webhook may both route to the same review workflow. A help or navigation skill may produce no promotable artifact and need no workflow. A scheduled workflow may have no user-facing skill.

Pairing belongs to the artifact and promotion policy, not naming symmetry.

9.7 “Everything generated must be reviewed” is scoped to promotable effects

The recommended invariant is:

> Any artifact or change that will be merged, published, installed, admitted, signed, released, or treated as settled must pass an independent review or explicit human acceptance gate.

Ephemeral explanations, navigation answers, and non-promotable scratch do not require a new Work merely to exist. This avoids turning review into ceremonial recursion.

────────

10. Proposed Workstream

The identifier ICM-R is provisional. Contracts may rename it during adjudication.

10.1 ICM-R0 — Proposal challenge and owner rulings

Outcome:

• challenge the two ladders for missing surfaces, authority conflicts, and over-escalation;
• adjudicate the owner decisions in §19;
• amend this proposal;
• record accepted definitions and hard boundaries;
• write no product or library changes.

No code changes.

10.2 ICM-R1 — Doctrine, templates, and pilot instruments

Outcome:

• add the canonical bounded-judgment context;
• amend docs/icm/convention.md, record-shapes.md, and the repo-to-icm classification method;
• add package-adjudication and decision-record templates;
• amend AGENTS.md only where routing doctrine must change;
• define the draft/promotion gate for rewritten skills and workflows;
• select and freeze the representative pilot corpus.

No src/, API, journal, TUI, backend, or workflow grammar changes.

10.3 ICM-R2 — Representative pilot

Recommended pilot:

|Package                |Why it is in the pilot                                           |
|-----------------------|-----------------------------------------------------------------|
|`grilling`             |Existing workflow-to-skill rehome with measured dialogue evidence|
|`sergeant-help`        |Clean read-only Captain skill baseline                           |
|`task-intake-and-route`|Strong pre-Work/Captain candidate                                |
|`sergeant-setup`       |Known split across CLI, dialogue, and procedural behavior        |
|`validate-and-ship`    |Complex durable workflow with explicit authority distinctions    |
|`code-review`          |Independent review workflow and multi-stage evidence handoff     |
|`repo-to-icm`          |The classifier/generator that must consume the new method itself |

Outcome:

• complete behavior-unit adjudication records;
• produce revised draft packages;
• run structural and representative semantic validation;
• adversarially review the ladder itself using the pilot’s disagreements;
• amend the method before full-corpus application.

No current package is moved merely because §12 predicts its likely outcome.

10.4 ICM-R3 — Full library reconciliation

Subject:

• all 23 published workflows;
• all four current operator skills;
• every shared context, helper, and delegation they depend on;
• the built-in software-change workflow as a separate embedded package.

Run in bounded waves organized by package relationships, not alphabetical order. A delegation cluster is adjudicated together so names never point at deleted or rehomed identities.

Outcome:

• every behavior unit dispositioned;
• every surviving package carries an authority envelope;
• catalogs and routing tables reflect final locations;
• provenance preserved;
• draft replacements independently reviewed and promoted;
• absorbed or retired packages removed cleanly.

10.5 ICM-R4 — Dogfood and measurement

Run real work through a representative set:

• one live Captain interview that produces a Work-ready artifact;
• one unattended multi-stage workflow;
• one workflow that reaches J0 and resumes through respond;
• one independent document or code review;
• one split package whose Captain skill submits its workflow counterpart;
• one rehomed or absorbed case proving the old surface is no longer needed.

Measure:

• false skill activations;
• unnecessary user questions;
• J0 questions that should have been J2/J1;
• unauthorized decisions that escaped review;
• stages that assume missing context;
• output contracts actors fail to honor;
• review findings by category;
• retries caused by package ambiguity;
• any case that appears to require runtime enforcement.

Close with a residual-gap report, not a presumption that code work follows.

10.6 ICM-R5 — Optional enforcement, only if earned

This milestone does not exist automatically.

It may be proposed only when ICM-R4 produces one or more accepted PL-7 engine-gap records. Each becomes a separate, narrowly scoped contract. Possible future subjects—none pre-approved here—include:

• machine validation of declared outputs before stage completion;
• structured authority metadata in workflow definitions;
• stronger skill catalog discovery across harnesses;
• artifact review/promotion state;
• decision-rung projection in the TUI.

The residual evidence chooses the subject. This proposal does not.

────────

11. Expected Change Surface

11.1 Expected content changes

Likely files include:

```text
NORTH-STAR.md                     only if an owner ruling changes it
AGENTS.md                         routing/doctrine references
docs/icm/convention.md            placement and authority rules
docs/icm/record-shapes.md         adjudication/decision record shapes
.sergeant/common/contexts/        bounded-judgment source
.sergeant/workflows/repo-to-icm/  revised classification method
.sergeant/index.md                final admitted workflow catalog
skills/                           revised and rehomed skills
.sergeant/workflows/              revised and rehomed workflows
docs/gauntlet/                    adjudication, validation, and provenance
README.md                         only where public routing changes
```

11.2 Explicitly out of scope initially

```text
src/**
tests/**
Cargo.toml
API routes and schemas
journal event families
WorkState
backend traits
TUI interaction
web surface
workflow.toml grammar
new package registry
new scheduler
```

11.3 Optional non-Rust validation tooling

A structural checker may eventually verify section presence, references, catalog consistency, and known record shapes. It is not presumed.

Ponytail order:

1. use existing repo-to-icm validation and review;
2. use a simple documented checklist;
3. measure repeated misses;
4. only then extend a script with the minimum check that catches them.

A validator that appears before the content shape stabilizes will freeze the wrong schema.

11.4 Runtime-change gate

No runtime change enters scope without:

1. a source-cited behavior;
2. a PL-7 classification;
3. actual lower-rung attempts;
4. rung-specific failure evidence;
5. the smallest new durable fact required;
6. a falsifiable acceptance test;
7. owner adjudication;
8. a separate proposal or milestone contract.

This is the existing engine-gap discipline, applied rather than merely cited. R11

────────

12. Initial Package Hypotheses

These are prioritization hypotheses, not dispositions.

12.1 task-intake-and-route

Likely contains:

• PL-2 Captain behavior: shape intent, decide direct versus durable, choose workflow, resolve user decisions;
• PL-0 absorbed behavior: submit, watch, respond, retry through existing sgt surfaces;
• possible PL-3 reusable routing method.

Hypothesis: SPLIT/HARVEST, with no surviving workflow identity.

12.2 direct-implementation

Its defining promise is implementation in the current session when one repository owns the outcome. Dispatching it changes that interaction model.

Hypothesis: REHOME to Captain skill, with review/shipping behavior routed to existing durable workflows.

12.3 sergeant-setup

Known contents already span:

• setup and repair absorbed by sgt init / sgt doctor;
• live project interview behavior;
• capability-gap judgment;
• historical upstream project-YAML concepts that do not cleanly map to the current estate model.

Hypothesis: SPLIT, with substantial PL-0 retirement and PL-2 skill behavior. Any surviving workflow stage must prove an already-defined intent and independent durable outcome.

12.4 dispatch

The package may conflate:

• Captain’s decision to dispatch;
• shaping repository-owned assignments;
• durable cross-repository execution and reconciliation.

Hypothesis: SPLIT, not wholesale retirement. The durable assignment procedure may remain workflow-shaped even when the selection procedure moves to Captain.

12.5 worker-mission

A Work that already represents one delegated assignment may legitimately need durable triage, implementation, independent review, and escalation stages.

Hypothesis: likely STAND or REWRITE, but its workflow-selection behavior must be tested against the Captain boundary.

12.6 load-project

Likely mixture of interactive estate shaping, deterministic repo operations, and persistent project/estate definition.

Hypothesis: SPLIT, with the current estate model—not upstream project YAML—deciding final destinations.

12.7 Review and investigation workflows

code-review, vet-external-skill, diagnose-bug, resolving-merge-conflicts, and repo-to-icm appear naturally compatible with already-defined intents, fresh stage execution, explicit artifacts, and independent evidence.

Hypothesis: likely STAND after authority and handoff rewrites, not guaranteed unchanged.

12.8 Skills already rehomed

grilling, grill-with-docs, sergeant-help, and estate-navigation provide useful positive and negative baselines. They still require the same source, authority, execution, and artifact validation as workflows.

────────

13. Alternatives Considered

13.1 Add authority fields to workflow.toml now

Rejected.

The vocabulary has not yet been tested across the corpus. Adding fields now would make Rust schema, hashing, journal replay, API projections, tests, and TUI display depend on an unmeasured content model. Content contracts are the lower rung.

13.2 Add output enforcement to the engine now

Rejected as a prerequisite.

Stage completion currently follows explicit backend semantics. Actors may fail to produce declared artifacts, but this proposal first makes completion contracts explicit and measures the failure rate. A runtime check may later be justified by a concrete PL-7 record.

13.3 Move most or all workflows to skills

Rejected.

The distinction is not “interactive good, workflow bad.” Durable independent review, diagnosis, generation, and cross-repository work are exactly what Sergeant is for. Wholesale movement would discard the fresh-execution, isolation, journal, retry, and evidence boundaries the runtime already provides.

13.4 Keep the catalog and add a generic decision paragraph everywhere

Rejected.

That would normalize prose without correcting placement. A Captain procedure would remain a workflow with better escalation wording.

13.5 Classify packages only

Rejected.

A package can contain absorbed mechanics, Captain dialogue, actor method, durable stages, and helpers simultaneously. Package-first classification guarantees useful behavior is either dragged onto the wrong surface or lost with the package.

13.6 Require one skill for every workflow and one workflow for every skill

Rejected.

The relation is many-to-many and artifact-policy-driven. Naming symmetry would create duplicate procedure and empty wrappers.

13.7 Build a generalized workflow graph or branching engine

Rejected.

Nothing in this problem requires it. The current linear stage model plus explicit needs_input and Captain follow-on Work is sufficient until a real procedure proves otherwise. R1E2

13.8 Build a new procedure registry

Rejected.

The repository already has skills/, .sergeant/index.md, workflow indexes, and ordinary Git history. Discovery improvements may be considered later; they are not a precondition for correct content.

────────

14. Risks and Mitigations

14.1 Ladder bureaucracy

Risk: Every package gains ceremonial boilerplate that actors ignore.

Mitigation: Define each ladder once. Local sections contain only package-specific grants, prohibitions, and evidence locations. Reject restated generic ladders.

14.2 Rung laundering

Risk: Actors cite J2 or PL-4 without the contract actually granting it.

Mitigation: Every rung citation names the source clause and alternatives considered. Independent review challenges adjacent rungs.

14.3 Over-escalation

Risk: Actors reach J0 for ordinary implementation choices and transfer their work to the user.

Mitigation: Explicit J2 decision classes, J1 examples, a required recommendation at J0, and measurement of unnecessary questions.

14.4 Under-escalation

Risk: Actors label a product or security choice J1.

Mitigation: J1 is defined negatively and cannot affect public behavior, authority, security, data, scope, acceptance, or promotion.

14.5 Self-review disguised as independence

Risk: A later stage receives the producer’s conclusions and rubber-stamps them.

Mitigation: Review stages receive the subject, governing contract, and rubric through explicit inputs; they do not inherit private conversation and cannot silently edit the subject.

14.6 Provenance loss during rehoming

Risk: Moving content severs the reason a behavior existed.

Mitigation: Move with an adjudication record and preserve historical citations. Update every delegation and catalog in the same change.

14.7 Content drift between skill and workflow

Risk: Paired artifacts duplicate the same procedure and diverge.

Mitigation: Assign one responsibility per surface. Captain skill owns conversation and submission; workflow owns durable execution; shared method owns reusable judgment; review policy owns promotion.

14.8 Corpus scope overwhelms review quality

Risk: Twenty-seven top-level packages encourage bulk mechanical edits.

Mitigation: Pilot first, then relationship-bounded waves. No full-corpus migration until the pilot method survives adversarial review.

14.9 Harness ask capability varies

Risk: A stage documents J0 but the selected backend cannot surface an actor-authored question.

Mitigation: Continue using the existing requires_ask preflight where the stage may need J0. Validate the real target harness. A package may instead narrow its scope to decisions fully bound before admission.

14.10 Review recursion

Risk: Every review artifact requires another review forever.

Mitigation: Promotion policy terminates at a named independent review plus explicit authority gate. Review evidence is not itself promoted as a new product unless separately published.

────────

15. Acceptance Contract

The ICM-R workstream is complete when all of the following are true:

1. The proposal is challenged and owner-ruling amendments are recorded.
2. Main audit revision is pinned for each milestone.
3. The Placement Ladder and Bounded-Judgment Ladder have canonical sources.
4. The existing ICM ladder is amended rather than silently contradicted by a second classifier.
5. PL-2 asks the Captain/admission-boundary question before PL-4 workflow classification.
6. PL-7 retains the full existing engine-gap proof template.
7. Every current published workflow has a package adjudication record.
8. Every current skill has a package adjudication record.
9. Every behavior unit is source-cited and dispositioned.
10. Every boundary-bearing classification records adjacent alternatives.
11. Package dispositions are synthesized from behavior units rather than chosen in advance.
12. Every surviving workflow states an authority envelope.
13. Every surviving actor stage states J2, J1, and J0 boundaries.
14. Every surviving skill states its driver, decision boundary, and durable handoff.
15. Every J0 path produces one precise question and preserves evidence.
16. At least one real or scripted workflow run reaches needs_input from a J0 decision and resumes through sgt respond.
17. At least one Captain skill asks a live question and waits rather than inventing the answer.
18. No stage relies on another execution’s private conversational memory.
19. Every contract-bearing cross-stage dependency appears in an Inputs table.
20. Every stage declares a falsifiable completion boundary.
21. Every promotable artifact names its independent review and promotion path.
22. No producer independently promotes its own output.
23. Skill/workflow relationships are recorded as many-to-many where appropriate; no empty symmetry wrappers are created.
24. Every rehome preserves provenance and updates all delegations and catalogs atomically.
25. Absorbed behavior names the current product surface that owns it.
26. Retired behavior has no undispositioned surviving policy.
27. Structural validation and semantic validation are reported separately.
28. An unscripted fake-backend walk is never cited as proof of an interactive hold or substantive judgment.
29. Representative real executions are reviewed at the trajectory level, not final output only.
30. The pilot is completed and the method amended before the full corpus moves.
31. The full corpus is reconciled in bounded dependency waves.
32. The residual-gap report distinguishes content defects from runtime gaps.
33. No file under src/, no API route, no Work state, no journal event, no backend trait, and no TUI behavior changes during ICM-R0 through ICM-R4.
34. No workflow.toml grammar field is added during ICM-R0 through ICM-R4.
35. Any optional validation script is justified by a repeated measured miss and implements only the minimum check.
36. Any proposed runtime work carries an accepted PL-7 record and a separate contract.
37. The final close-out reports which initial package hypotheses were wrong and why.
38. The final ledger records every Ponytail decision, placement ruling, owner amendment, and deferred finding.

────────

16. Falsifiers and Runtime Escalation Triggers

The no-Rust-first hypothesis is falsified only by evidence such as:

1. Completion-contract failure: actors repeatedly report stage completion without required artifacts despite clear stage contracts, and review cannot reliably catch the omission before downstream effects.
2. Ask-signal failure: a supported target harness cannot reliably turn a documented J0 question into needs_input, despite the capability being advertised and measured.
3. Authority reconstruction failure: after restart or retry, the actor cannot reconstruct the same authority envelope from pinned content and Work state.
4. Promotion ambiguity: independent review and draft/admitted filesystem boundaries cannot prevent an unreviewed artifact from being treated as settled.
5. Skill discovery failure: Captain or stage actors cannot reliably locate a package the canonical routing doctrine names, across admitted target harnesses.
6. Artifact identity failure: downstream stages cannot unambiguously identify the exact upstream artifact they are required to consume.
7. Concurrent ownership requirement: a real accepted procedure requires runtime-owned parallel checkpoint identity or coordination that cannot be represented as separate Works, stages, or native harness subagents without losing required evidence.

Each observation is still only evidence. It becomes an engine gap only after lower-rung attempts and owner adjudication.

────────

17. Ponytail Decision Register

The rung is the lowest viable resolution, not the most elaborate implementation. The register uses the repository’s existing R1–R7 Ponytail vocabulary; the new PL and J ladders govern procedural placement and actor authority inside the resulting content.

|Decision|Ponytail rung|Resolution                                                                                                |
|--------|------------:|----------------------------------------------------------------------------------------------------------|
|ICMR-01 |R1           |Pin `3a46b87`; do not design against moving main                                                          |
|ICMR-02 |R2           |Reuse current ICM/shared-context surfaces for the ladders                                                 |
|ICMR-03 |R2           |Preserve current fresh-execution-per-stage engine                                                         |
|ICMR-04 |R2           |Extend the existing decomposition method; do not create an unrelated classifier                           |
|ICMR-05 |R2           |Reuse source-cited behavior units and classification ledgers                                              |
|ICMR-06 |R2           |Reuse `needs_input` / `respond` as J0 mechanics                                                           |
|ICMR-07 |R1/R2        |Treat Captain dialogue as a skill, not new engine hold machinery                                          |
|ICMR-08 |R2           |Treat stage files and Layer-4 artifacts as cross-execution handoff                                        |
|ICMR-09 |R2           |Normalize bounded judgment already present in packages such as `validate-and-ship`                        |
|ICMR-10 |R1           |Classify behavior units before packages                                                                   |
|ICMR-11 |R1/R2        |Allow stand, rehome, split, harvest, absorb, fold, and retire; do not force binary skill/workflow outcomes|
|ICMR-12 |R1           |Reject mandatory one-to-one skill/workflow pairs                                                          |
|ICMR-13 |R2           |Attach review to promotable artifact policy, not naming symmetry                                          |
|ICMR-14 |R2           |Preserve draft/admitted publication boundaries                                                            |
|ICMR-15 |R2           |Require explicit stage completion boundaries as content first                                             |
|ICMR-16 |R2           |Require authority narrowing, never silent widening                                                        |
|ICMR-17 |R2           |Require explicit file inputs instead of shared-conversation assumptions                                   |
|ICMR-18 |R2           |Reuse `repo-to-icm`’s contract/inventory/harvest/classify/review sequence                                 |
|ICMR-19 |R1           |Pilot representative packages before full-corpus migration                                                |
|ICMR-20 |R1/R2        |Keep ICM-R0 through ICM-R4 free of Rust/runtime changes                                                   |
|ICMR-21 |R1           |Defer validation-code changes until repeated misses prove a checklist inadequate                          |
|ICMR-22 |R7           |Admit runtime work only through a separately accepted PL-7 engine-gap contract                            |
|ICMR-23 |R2           |Validate structural, semantic, authority, and execution claims separately                                 |
|ICMR-24 |R2           |Read trajectories, not only terminal outputs, when refining packages                                      |
|ICMR-25 |R1/R2        |Preserve all proposal-family boundaries not implicated by this content work                               |

Any implementation or migration decision not represented here is logged in the milestone report. Any new R7 decision names the failed R1–R6 paths before admission.

────────

18. Source-to-Decision Map

|Source                                                                                                                                                                                  |What it constrains here                                                                              |
|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------|
|[P0 execution-surface proposal](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-depot-rust-execution-surface.md)              |daemon/Work center, native harness boundary, no unearned general workflow engine                     |
|[Next-iteration ICM proposal](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-next-iteration-icm-workflows.md)                |current-engine first arc, manual adjudicated corpus, lower-rung minimality, fresh execution per stage|
|[Foundation rationalization](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-foundation-rationalization.md)                   |evidence-driven boundary correction and Ponytail minimality                                          |
|[S-series proposal](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-s-series-stabilization.md)                                |measurement first, orthogonal no-product-behavior workstream                                         |
|[T-series proposal](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-tui-t-series.md)                                          |audit pin, numbered decisions, acceptance contract, Ponytail register, timestamped model             |
|[WATCH proposal](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-sgt-watch-v1.md)                                             |expose/reuse existing mechanism rather than parallel machinery                                       |
|[Adapter research v2](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/reference/proposal-harness-adapter-research-v2.md)                         |authored/documented/measured/admitted distinction                                                    |
|[ICM convention](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/docs/icm/convention.md)                                                         |four layers, explicit file handoff, publication and execution-surface tests                          |
|[ICM record shapes](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/docs/icm/record-shapes.md)                                                   |Inputs tables, source-cited behavior units, classification records                                   |
|[Promotion spec](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/docs/icm/promotion-spec-2026-08-11.md)                                          |mechanical engine walk is not semantic or interactive validation                                     |
|[Current ICM ladder](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/.sergeant/workflows/repo-to-icm/_config/icm-ladder.md)                      |ordered first-match classification, reimplementation test, engine-gap template                       |
|[Workflow/engine implementation](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/src/domain/workflow.rs)                                         |pinned stage content, execution reservation, stage progression                                       |
|[Engine progression/retry implementation](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/src/runtime/engine.rs)                                 |stop current execution, reserve next stage, fresh retry attempt                                      |
|[Claude adapter](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/src/backend/claude.rs)                                                          |per-stage session identity, question -> needs_input, successful no-question -> completion            |
|[North Star and AGENTS](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/NORTH-STAR.md)                                                           |Captain owns judgment/dialogue; sgt owns durable execution                                           |
|[`validate-and-ship` gate stage](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md)|existing concrete bounded-judgment precedent                                                         |
|[Re-homing record](https://github.com/miztertea/sergeant-rs/blob/3a46b87c17d249655708ed5ac32f6704738776cf/docs/icm/re-homing-record-2026-08-12.md)                                      |precedent for skill/workflow/CLI rehoming with provenance                                            |
|[Ponytail README](https://github.com/DietrichGebert/ponytail/blob/c57811a01bcc35d103ef9532378cd22fc3005133/README.md)                                                                   |understand first, stop at first viable rung, preserve safety/validation                              |
|[ICM paper](https://arxiv.org/html/2603.16021v2)                                                                                                                                        |numbered stage folders, explicit context/handoffs, stable references vs per-run artifacts            |
|[Anthropic effective agents](https://www.anthropic.com/engineering/building-effective-agents)                                                                                           |workflow vs dynamic agent distinction, human checkpoints, simplest adequate pattern                  |
|[Agent Skills specification](https://agentskills.io/specification)                                                                                                                      |scoped skill directories, progressive disclosure, references/scripts/assets, validation              |
|[Agent Skills best practices](https://agentskills.io/skill-creation/best-practices)                                                                                                     |synthesize from real artifacts, refine with execution, coherent units, calibrated control            |
|[WorkPacket](https://app.notion.com/p/39a27ada618f818cba42f5efe8ffe1f0)                                                                                                                 |orchestration assembles intent/context/method/authority before reasoning                             |
|[Work Filesystem](https://app.notion.com/p/3ac27ada618f819d8196fa78ab420224)                                                                                                            |actor-ready world, one responsibility per surface, explicit continuation/evidence                    |
|[Skill Libraries as Simulated Work Environments](https://app.notion.com/p/3ac27ada618f81fea604eac1a3029dd5)                                                                             |separate procedure skill from volatile organizational state and broader work structure               |

────────

19. Owner Decisions Required at ICM-R0

This proposal recommends defaults but does not silently make these owner rulings:

1. Names: Are Placement Ladder (PL) and Bounded-Judgment Ladder (J) the accepted terms and identifiers?
2. Skill taxonomy: Do Captain skills and actor skills share the current skills/ root with metadata distinguishing driver, or should the repository use separate subdirectories/catalog sections?
3. Universal scope: Does “every skill and workflow must be validated” include embedded software-change, shared contexts, and helpers as first-class review subjects? This proposal recommends yes, with validation depth proportional to effect.
4. Stage requirement: Must every actor stage carry a local ## Bounded judgment section, or may a stage with no local specialization explicitly declare “inherits workflow envelope unchanged”? This proposal recommends an explicit local section either way so omission is never ambiguous.
5. Decision recording: Must every J2 decision be recorded, or only material J2 decisions? This proposal recommends material decisions only.
6. Generated-output invariant: Is independent review required for all generated files or only artifacts that will be promoted, merged, published, installed, admitted, signed, released, or treated as settled? This proposal recommends the latter.
7. Review independence: May a later stage in the same workflow qualify as independent when it has a fresh execution, explicit inputs, a review-only contract, and no edit authority? This proposal recommends yes.
8. Pilot corpus: Is the seven-package pilot in §10.3 accepted or amended?
9. Likely rehomes: May the pilot explicitly test the hypotheses for task-intake-and-route, direct-implementation, and sergeant-setup, or must those remain untouched until the general method is proven on less controversial packages?
10. Runtime freeze: Is “no Rust/runtime changes through ICM-R4” a hard contract or a default that an urgent, independently proven engine gap may interrupt? This proposal recommends a hard contract for this workstream; urgent runtime defects remain separate work.
11. Proposal placement: Should this file land under reference/ as a proposal, with a later docs/gauntlet/contracts/ICM-R0.md, following the existing house pattern?
12. Review workflow names: Should generic future review procedures be organized by artifact class (review-document, review-pr, review-skill, review-workflow) or be derived only after the corpus shows which distinctions matter? This proposal recommends deriving them during synthesis rather than pre-creating four wrappers.

────────

20. Conclusion

Sergeant does not need a new execution engine to become the system described in the owner’s model.

The engine already provides the crucial structure: durable Work, pinned procedure, fresh execution per stage attempt, explicit holds, retry, recovery, isolated surfaces, and evidence. The repository also already contains the beginnings of the right content architecture: ICM layers, source-cited behavior units, a first-match decomposition ladder, a draft/admitted boundary, re-homing precedent, and local authority rules such as auto-fix versus ask-user.

What remains is reconciliation.

The Placement Ladder makes the ownership boundary explicit before procedure is admitted. The Bounded-Judgment Ladder makes the actor’s authority explicit after it enters a skill, workflow, or stage. Behavior-unit review prevents existing package boundaries from deciding their own correctness. Independent promotion prevents generation from becoming publication by momentum.

The first move is therefore not Rust.

It is to rewrite the procedural library so every surviving package can answer, with citations:

```text
Why does this behavior exist?
Who drives it?
When does it run relative to Work admission?
What durable outcome does it own?
What may its actor decide?
What must become needs_input?
What files carry context between executions?
What independently validates and promotes the result?
```

Once those answers have been exercised against real work, any remaining engine gap will be smaller, better evidenced, and harder to confuse with a content problem.

That is the same architecture Sergeant has pursued from P0 onward:

> **Use the lowest rung that faithfully owns the fact. Let measured work—not imagined generality—earn the next one.**

────────

References
