# ICM Record Shapes

Governing document: `reference/proposal-next-iteration-icm-workflows.md`
§§6.7, 7.3–7.4, 9.4–9.5. Milestone: `docs/gauntlet/contracts/N1.md`.

This document is normative for four record shapes: the `index.md`
front-matter shape, the authored/observed metadata split it depends on, the
behavior-unit record, the classification record, and the engine-gap claim
template. It defines shapes only — no parser, no CLI, no Rust type exists
yet for any of these. Where an example is given it is the canonical shape;
deviation from a field name or required/optional status below is a
violation unless this document is amended.

Companion document: `convention.md` defines the filesystem layout these
records live inside.

---

## 1. `index.md` front matter (OKF-compatible)

Every admitted workflow's `index.md` carries front matter in this shape
(§7.3). The root `.sergeant/index.md` is the catalog, not an entry — it has
its own shape, defined in rule 2 below:

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

Fields:

| Field | Required | Meaning |
|---|---|---|
| `kind` | yes | The record's ICM type. `workflow` for a workflow `index.md`. Other `kind` values are reserved for future catalog entries (e.g. shared-context indexes); a document with no `kind` is not a catalog entry and MUST NOT be linked from `.sergeant/index.md`. |
| `name` | yes | The workflow's identity. MUST equal the containing directory name under `.sergeant/workflows/` or `.sergeant/drafts/workflows/`. A mismatch is a violation — it is exactly the kind of ambiguity `@@name` resolution and catalog listing depend on not existing. |
| `status` | yes | One of `draft`, `published`. MUST agree with filesystem location per `convention.md` §2.3: `published` is only legal under `.sergeant/workflows/`. `status` is authored metadata (§2 below) — it is never inferred from run history. |
| `version` | yes | A monotonically increasing integer, bumped on any change to the workflow's stage sequence, context content that changes behavior, or `status`. Two admitted workflows sharing a `name` and `version` with different content is a violation — version is the freshness signal readers and generators rely on. |
| `description` | yes | One to a few sentences: what the workflow is for and its bounded outcome (§6.2's "recognizable trigger, bounded outcome, completion condition"). A description that only restates the name (e.g. `name: diagnose-bug`, `description: diagnoses bugs`) is a violation — it fails the greppability purpose of §7.3. |
| `tags` | no | A flat list of free-text topical tags. Tags are authored, not derived; they MUST NOT be populated from observed telemetry (§2 below). |

Rules:

1. Front matter is parsed independently of the Markdown body. The body
   below the closing `---` is free-form documentation for a human or
   harness reading the file directly; it MUST NOT be required to recover
   `name`, `status`, or `version` — those are only in front matter. A
   workflow whose identity can only be determined by reading prose is a
   violation of the "greppable" requirement (§7.3).
2. `.sergeant/index.md` (the root catalog) is itself a `kind: workflow`-free
   document; it is the list, not an entry. It MUST enumerate every
   `status: published` workflow's `name` and MAY link to that workflow's own
   `index.md`. A published workflow missing from the root list is a
   violation (`convention.md` §1.1).

## 1a. Stage `CONTEXT.md` Inputs table (Layer 2 contract)

Adopted from the published ICM protocol (`convention.md` §1a, register row
D9). Every stage `CONTEXT.md` opens with an Inputs table naming exactly
which files load at stage entry:

```markdown
## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |
| references/reproduction-method.md | L3 | the method this stage applies |
| ../_config/evidence-policy.md | L3 | what counts as evidence |
| ../00-reproduce/output/reproduction.md | L4 | upstream artifact consumed here |
```

Rules:

1. Paths are relative to the stage directory; every path MUST resolve
   inside the workflow's directory or `.sergeant/common/` (a `@@name`
   reference in the prose is equivalent to an Inputs row for its resolved
   file and SHOULD be listed for completeness).
2. `Layer` is `L1`, `L3`, or `L4` — a stage never inputs another stage's
   `CONTEXT.md` (L2 is a contract between the engine and that stage's
   actor, not shared material).
3. An L4 input MUST name a file some earlier stage's `output/` declares.
   An L4 input from a *later* stage is a violation (the order in
   `workflow.toml` is the dependency order).
4. The table is the stage's declared dependency set. Exploration beyond it
   is actor judgment and legal; a contract-bearing dependency missing from
   the table is a violation (`convention.md` §1a rule 1).

## 2. Authored metadata versus observed telemetry

Two categories of fact exist about a workflow and MUST NOT be mixed in one
storage location (§7.4).

**Authored** (lives in front matter, hand- or generator-written, changes
only when a human or reviewed generator edits the file):

```text
name, version, status, owner, description, tags,
intended inputs and outputs, publication state
```

**Observed** (lives in the journal and its DuckDB projection, derived from
actual runs, never hand-edited):

```text
run counts, completion rate, last execution, blocked time,
cost, token use, duration, retry frequency, failure modes
```

Rules:

1. A workflow's `index.md` front matter MUST NOT contain any observed
   field. `run_count`, `last_run`, `median_duration`, or any similarly
   derived key appearing in front matter is a violation — it is exactly
   the "write mutable measurements back into the workflow's front matter"
   the proposal rules out (§7.4).
2. A future discovery surface MAY join authored and observed fields for
   display, but the join is read-only and happens outside the authored
   file:

   ```text
   diagnose-bug v3
     authored status     published
     tags                debugging, defect, investigation
     observed runs       184
     completion rate     87.5%
     median duration     14m22s
     last measured       2026-08-04
   ```

   Nothing in this join is permitted to be persisted back into
   `index.md`. A tool that writes the joined view back into the authored
   file collapses the separation this section defines and is a violation
   regardless of how convenient the round-trip would be.
3. Because observed data is a journal projection, it obeys the same
   rebuild-on-start invariant as every other projection in this codebase
   (see `CLAUDE.md`: "Everything else ... is a disposable projection
   rebuilt from it"): deleting the projection and rebuilding from the
   journal MUST reproduce identical observed fields. This record-shapes
   document does not define the projection; it defines that authored
   fields may never leak into it as if they were observed, and vice versa.

## 3. Behavior-unit record

A behavior unit is one atomic, source-cited statement of behavior, produced
during extraction (§9.4). NDJSON, one record per line:

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

Field rules:

| Field | Required | Rule |
|---|---|---|
| `id` | yes | Unique within the corpus, stable once assigned (later records may reference it; renumbering breaks classification records and provenance). Format `BU-####` is the convention used here; any scheme is legal if unique and stable. |
| `statement` | yes | Normalized, implementation-independent language (§8.3 step 3 "Normalize"). A statement that names an old-Sergeant mechanism (tmux, a specific shell script's filename, a sentinel file) where the mechanism is not itself the behavior is a violation — mechanism and behavioral intent are recorded separately (see the N1 contract's binding rule and §8.2's dispatch/tmux stress case). |
| `source.path` | yes | Repository-relative path to the source artifact. |
| `source.locator` | yes | A locator inside that path precise enough for a human to re-open the exact evidence — a heading path, function name, line range, or test name. A `locator` of "somewhere in the file" is not a locator. |
| `source.quote` | yes | The quoted source text itself, verbatim, ≤500 characters. A span longer than 500 bytes records its first 500 characters here plus a `span_bytes` count; the hash still covers the full span. Without the preimage the hash rule below is unenforceable by construction — a verifier cannot reproduce a hash whose span it must guess (N1 adjudication A2, from finding R3-02). |
| `source.quote_hash` | yes | `sha256:<hex>` over the **exact contiguous byte span quoted from the cited file — no normalization, no trimming beyond the span choice itself** (the convention finding R3-02 showed must be stated, not assumed). **A behavior unit whose `quote` does not appear contiguously in the cited file, or whose `quote_hash` does not verify against that span, is invention, and is rejected at lint, not review** (N1 contract, binding rule). A unit whose statement cannot be re-anchored to a real contiguous span is not silently deleted: it is marked `confidence: low` with a `citation: disputed` note — an unverifiable citation is a fact the corpus records (A2). |
| `scope` | yes | The procedural context the behavior applies within (a workflow name, "cross-repository work", or similarly bounded scope) — not yet the classification (that is §4 below); scope narrows where to look. |
| `trigger` | yes | What condition makes the behavior apply. |
| `outcome` | yes | What durably differs once the behavior has been followed. |
| `authority` | yes | Who or what enforces or performs the behavior (an actor role, "user-context actor", "the runtime", etc.). |
| `confidence` | yes | `high`, `medium`, or `low`, reflecting how directly the source supports the statement (a direct instruction is `high`; an inferred convention pieced together from behavior is `low`). `confidence` is not a substitute for citation — even a `low`-confidence unit MUST carry a real `quote_hash`. |
| `notes` | no | Free text — mechanism/intent separation notes, open questions, cross-references. |

Additional rules:

1. One behavior unit states one behavior. A record whose `statement`
   contains an unmarked conjunction of independently-triggerable behaviors
   ("verify X and also log Y and also notify Z") is a violation — split it
   into separate units so each can be independently classified, refuted,
   and traced.
2. A behavior unit MUST NOT be manufactured to justify a classification
   decided in advance. Extraction (§9.3 stage `20-harvest`) precedes
   classification (`40-classify`); a corpus where every unit's `statement`
   suspiciously matches an existing workflow's stage boundary is grounds
   for a reviewer to challenge whether extraction or classification ran
   first in practice (§8.3 step 6, "Refute").

## 4. Classification record

A classification record assigns one behavior unit to one ICM representation
via the decomposition ladder (§6, §9.5):

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

Field rules:

| Field | Required | Rule |
|---|---|---|
| `behavior_id` | yes | MUST reference an existing behavior unit's `id`. A classification record whose `behavior_id` does not resolve is a violation — it classifies nothing. |
| `representation` | yes | One of the ladder's terminal representations: `agents-invariant` (§6.1, → `AGENTS.md`), `workflow` (§6.2), `stage` (§6.3), `stage-context` (§6.4, → actor guidance inside a stage's `CONTEXT.md`), `helper` (§6.5), `shared-helper` / `shared-context` (§6.6), `engine-gap` (§6.7), or `obsolete-mechanism` (the §8.1/§8.2 disposition: mechanism the current runtime replaces structurally, recorded with any surviving policy re-homed). No other value is legal; a representation outside this set means the ladder was not actually applied. *(Amended at N1 adjudication A1: this vocabulary is what the extraction instructions defined and the 966-unit corpus uses; the enum first written here conflicted with this document's own worked example.)* |
| `workflow` | conditional | Required when `representation` is `stage`, `actor-stage`, `helper`, or a workflow-local variant. Omitted for `invariant`, `engine-gap`, or a `shared-context`/`shared-helper` with no single owning workflow. |
| `stage` | conditional | Required when `representation` is `stage` or `actor-stage`. MUST match an actual stage directory name (`convention.md` §1) once the workflow is materialized; during pre-synthesis classification it names the *intended* stage. |
| `rationale` | yes | States *why this rung and not an adjacent one* — not a restatement of the behavior. A rationale that could be copy-pasted unchanged onto a different representation choice is too generic and is a violation of intent: it does not discriminate. |
| `alternatives_considered` | yes | The other representations weighed and rejected, as a list of ladder rung names. An empty list is legal **only** where no adjacent rung was facially plausible (N1 adjudication A9). It is REQUIRED to be non-empty for every unit carrying a workflow or stage boundary, every `engine-gap` unit, and every unit named in a classification-ledger conflict — boundary-bearing classifications must be refutable, and an unrecorded rejection is unrefutable. |
| `engine_gap` | yes (nullable) | `null` unless `representation` is `engine-gap`, in which case it MUST be the full template from §5 below, not a summary. A `representation: engine-gap` record with a `null` or partial `engine_gap` field is a violation. |

Additional rules:

1. Classification is a claim, not a fact, until it survives refutation
   (§8.3 steps 6–7). A classification record with no corresponding entry
   (accept/reject/merge/park, with reviewer identity or role) in
   `classification-ledger.md` has not been adjudicated and MUST NOT be
   treated as settled — this document defines the record's shape, not its
   adjudication state, which is the reference corpus's responsibility
   (`docs/gauntlet/contracts/N1.md`'s Outcome §2), not this document's.
2. `representation: workflow` and `representation: invariant` records
   normally have no `stage` and describe the unit at the coarsest grain;
   they still require `rationale` and `alternatives_considered` to the same
   standard as stage-level records. "It's obviously a workflow" is not a
   rationale.

## 5. Engine-gap claim template

An engine-gap claim asserts that Sergeant's runtime — not an actor, not a
helper — must own a new durable fact (ordering, identity, retry, recovery,
authorization, isolation, or evidence semantics) to represent a behavior
faithfully (§6.7).

Required fields, verbatim from §6.7:

```text
behavior that cannot be represented
source evidence requiring it
lower-rung representations attempted
why each lower rung fails
minimum runtime capability required
observable acceptance test
```

As a record (nested inside a classification record's `engine_gap` field, or
standalone in `engine-pressure.md`):

```json
{
  "behavior": "Two workflows both need to invoke a shared 'run adversarial review' procedure with its own retry/measurement, not just shared text.",
  "source_evidence": ["BU-0117", "BU-0142"],
  "lower_rungs_attempted": [
    "shared context (@@adversarial-review)",
    "shared helper script",
    "duplicate workflow-local stage in each parent"
  ],
  "why_each_fails": {
    "shared context (@@adversarial-review)": "Pulls text into the current actor's turn; produces no independent durable checkpoint, retry, or measurement — the parent's single stage absorbs an unbounded sub-procedure (§7.7).",
    "shared helper script": "A helper's outcome is subordinate to the calling stage; it cannot itself block, retry, or be measured independently (§6.5).",
    "duplicate workflow-local stage in each parent": "Loses a single identity for the reused procedure; drift between the two copies is undetectable and untracked."
  },
  "minimum_runtime_capability_required": "A stage kind that binds and executes another pinned workflow while retaining parent/child trajectory identity, retry, and cancellation (§7.7's listed losses).",
  "observable_acceptance_test": "Two distinct parent workflows each invoke the same child workflow identity; the child's own stage-level retry and measurement are visible in the parent's trajectory without the parent re-implementing them."
}
```

Rules:

1. All six fields are required. A claim missing any one of them — most
   commonly `lower_rungs_attempted` or `why_each_fails` — is incomplete and
   MUST be rejected at lint, per the N1 contract's binding rule ("an
   engine-gap claim without named failed lower rungs is auto-rejected").
2. `lower_rungs_attempted` MUST name actual ladder rungs from §6 (invariant,
   workflow, stage, actor-stage, helper, shared context/helper) — not
   restate the claimed gap in different words. A claim that only asserts
   "the current engine can't do this" without naming which lower-rung
   representations were tried and how they concretely fell short does not
   meet this bar.
3. `why_each_fails` MUST give a reason specific to that rung's actual
   mechanics (as in the example: a shared context has no independent
   checkpoint; a helper's outcome is subordinate). A reason that is
   identical across every listed rung ("it's not powerful enough") is a
   violation — it shows the rungs were not actually attempted or reasoned
   about individually.
4. "Would be convenient" or "could be more elegant," in any field, is not
   engine-gap evidence and is grounds for outright rejection of the claim,
   not merely a request for more detail (§6.7, N1 contract).
5. `observable_acceptance_test` MUST describe something checkable after the
   capability exists (a scenario, not a restatement of the desired feature
   name). "The engine supports nested workflows" is not an acceptance test;
   the example above ("two parents invoke the same child identity and its
   retry/measurement are visible without re-implementation") is.
6. An engine-gap claim is evidence for the reference corpus's
   `engine-pressure.md` and for N1's Unknowns (e.g. U3 on `needs_input`
   semantics); it is not by itself authorization to change the engine. Per
   this milestone's non-goals, no engine-gap claim produced under this
   record shape triggers implementation before a later milestone's
   contract accepts it (`docs/gauntlet/contracts/N1.md` Non-goals; proposal
   §21.8's trigger conditions for workflow composition are one example of
   the bar a claim must eventually clear).

## 6. Package-adjudication record (ICM-R)

Canonical shape for the ICM-R library-reconciliation pass
(`reference/proposal-icm-r-procedure-authority.md` §8.13,
`docs/adr/0013-icm-r0-owner-rulings.md`). One record per package under
review, produced by the reconciliation's producer step and checked by its
independent reviewer step (§6.2/6.3 in `convention.md`) before Captain's
own reconcile-and-publish pass.

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
STAND | REHOME | SPLIT | HARVEST | ABSORBED | FOLD | RETIRE

## Validation evidence
```

Rules:

1. **Behavior units before package verdict.** The `Behavior-unit
   dispositions` table is filled in before `Final disposition` is chosen —
   a package's overall verdict is synthesized from its units' individual
   PL/J classifications, never decided first and back-filled (§8.8 of the
   proposal; the same discipline this repo's classification-record rule
   already requires at behavior-unit granularity, §4 above, applied now at
   package granularity too).
2. **`PL rung` and `J boundary` cite the ladders directly** (e.g. `PL-4`,
   `J2`) — not a paraphrase. A unit surviving in a skill, workflow, or
   stage states its J5 constraints, consumed J4 decisions, settled J3
   records, delegated J2 classes, remaining J1 choices, and what must land
   at J0, per `bounded-judgment.md`'s own worked-example discipline.
3. **`Disposition` uses the modifier vocabulary** (STAND, REHOME, SPLIT,
   HARVEST, ABSORBED, FOLD, RETIRE — proposal §5.10), not free text.
4. **A source file mapping one-to-one onto a new package is not evidence
   of correctness** — the same file-shape-mirroring failure §5 above
   already warns against at the behavior-unit level (N2's `sgt-recover`/
   `sgt-respond`/`sgt-watch` tell) applies here too: synthesis clusters by
   behavioral contract and durable outcome, not by which file a unit
   happened to originate in.
