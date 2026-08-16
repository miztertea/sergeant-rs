# Package adjudication: deepen-module

ICM-R3 full-reconciliation pass, `reference/proposal-icm-r-procedure-
authority.md` §10.4; method per §8; record shape per
`docs/icm/record-shapes.md` §6; owner rulings per
`docs/adr/0013-icm-r0-owner-rulings.md`. Producer pass only — independent
review is a separate step (§8.11 of the proposal; `docs/icm/convention.md`
§6.2/6.3) and has not run yet. This record is itself draft and does not
self-promote (ADR 0013 decisions 6-7).

## Original intention

Turn a shallow module into a deep one at a deliberately chosen seam:
classify the dependency cluster to decide whether a port is needed at all,
generate and compare at least three independently designed interfaces
under distinct constraints, then replace the old shallow-module tests with
tests written at the new interface (`.sergeant/workflows/deepen-module/
CONTEXT.md` "Purpose"; `index.md`). Promoted candidate **W25** from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`,
`docs/icm/promotion-spec-2026-08-11.md`), decomposed from
`reference/sergeant-upstream`'s `codebase-design` skill
(`reference-corpus/synthesis.md` §1). Full N1 citation trail archived at
`docs/gauntlet/promoted-provenance/deepen-module.md`. This ICM-R3 pass does
not re-run N1 extraction; it re-derives placement and authority
classification against the current package content and checks it against
the archived provenance and the current (post-ICM-R2) upstream source.

## Current trigger and outcome

One linear stage list (`workflow.toml`: `00-classify-dependencies`,
`10-design-it-twice`, `20-test-at-new-interface`), one entry point at
`00-classify-dependencies`.

Trigger (workflow-level, restated identically in `CONTEXT.md` and every
stage's own Layer-2 contract): "A module's interface needs redesign, or a
port/adapter decision needs to be made deliberately rather than by
default."

Outcome: a dependency classification that determines whether a port is
needed at all; at least three independently generated, structurally
different interface designs compared on depth/locality/seam placement and
resolved to an opinionated recommendation; old shallow-module tests
deleted and replaced with tests asserting only through the new interface.

## Driver and admission boundary

Driver: **stage actor**, all three stages (each already labeled
"actor-stage (§6.4, judgment)" in the package's own stage table — verified,
not merely copied, against the Placement Ladder below).

Admission boundary: **post-Work, in-Work**. The execution-surface test
(`convention.md` §2a — "would a human type `sgt run '<intent>' --workflow
deepen-module`?") holds: a human (or an upstream workflow/skill) has
already identified a specific module or dependency cluster as a deepening
candidate before this package's first stage runs; nothing in any stage's
contract asks the actor to decide *whether* a module should be deepened in
the first place, only *how*. `10-design-it-twice`'s own contract
(`BU-P4-023`) is explicit that the stage must "proceed immediately to
spawning sub-agents without waiting for a reply" after showing the user its
problem framing — the opposite of Captain-style live dialogue gating on a
user decision. This confirms PL-4/PL-5, not PL-2: the procedure's job is
not to decide what Work should exist, and it does not block on a live user
turn to make progress (§5.4's discriminator).

Known consumers/delegations, verified by direct search (not assumed from
the package's own text): `worker-mission/20-implement` and its parent
`CONTEXT.md` name `deepen-module` as one of five disciplines
`10-triage-and-route` may select, invoked today as context composition
rather than true nested-workflow invocation (an already-recorded,
already-scoped engine gap — `convention.md` §4 rule 1 / §7.7 — not unique
to this package and not re-litigated here). `tdd/CONTEXT.md` hands
refactoring off to "`code-review`/deepen-module discipline" by the same
informal-composition pattern. Both are pre-existing, correctly-scoped
references; neither requires a change to this package.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-P4-013` | `CONTEXT.md` (Purpose/Trigger, workflow-level) — deepening is a distinct, bounded procedure: classify, apply seam discipline, replace tests | PL-4 | J5 (contract-level: package identity and stage order are fixed by `workflow.toml`, §1 rule 4) | STAND | `deepen-module` (workflow) |
| `BU-P4-014` | `00-classify-dependencies/CONTEXT.md` — in-process dependencies: always merge, no adapter | PL-5 | J2 (delegated: classify a candidate's dependency category) | STAND | `00-classify-dependencies` |
| `BU-P4-015` | `00-classify-dependencies/CONTEXT.md` — local-substitutable: deepen against the stand-in, seam stays internal | PL-5 | J2 (same delegation) | STAND | `00-classify-dependencies` |
| `BU-P4-016` | `00-classify-dependencies/CONTEXT.md` — remote-but-owned: port + in-memory/production adapters | PL-5 | J2 (same delegation) | STAND | `00-classify-dependencies` |
| `BU-P4-017` | `00-classify-dependencies/CONTEXT.md` — true-external: port + mock adapter | PL-5 | J2 (same delegation) | STAND | `00-classify-dependencies` |
| `BU-P4-018` | `DEEPENING.md` "Seam discipline," L29 — one adapter is a hypothetical seam, two adapters (typically prod+test) justify a real one | PL-5 (stage-context, workflow-local per `reference-corpus/shared-context-map.md` line 347 and `synthesis.md` §"Workflow-local contexts") | J2 (same delegated classification decision as BU-P4-014..017 — whether the classification result actually justifies exposing a port) | **FOLD — missing from the live package; not a placement change, a content gap** | `00-classify-dependencies/CONTEXT.md` (add to Behavior contract) |
| `BU-P4-019` | `DEEPENING.md` "Seam discipline," L30 — internal seams (private, test-only) are not exposed through the public interface merely because tests use them | PL-5 (stage-context, workflow-local, same map entry) | J2 (same delegation — what belongs on the deepened module's public interface) | **FOLD — missing from the live package** | `00-classify-dependencies/CONTEXT.md` (add to Behavior contract) |
| `BU-P4-022` | `10-design-it-twice/CONTEXT.md` — parallel-sub-agent pattern; first idea unlikely to be best | PL-5 | J2 (delegated: how to explore alternative designs) | STAND | `10-design-it-twice` |
| `BU-P4-023` | `10-design-it-twice/CONTEXT.md` — frame the problem for the user, then proceed without waiting for a reply | PL-5 | J1 (local, reversible: sequencing framing-then-spawn does not gate on or change what the user may later decide) | STAND | `10-design-it-twice` |
| `BU-P4-024` | `10-design-it-twice/CONTEXT.md` — produce 3+ radically different designs, each under an explicit distinguishing constraint | PL-5 | J2 (delegated: which constraints to assign, how many designs) | STAND | `10-design-it-twice` |
| `BU-P4-025` | `DESIGN-IT-TWICE.md` Process step 2, L30 — each sub-agent's brief must include both `codebase-design` vocabulary and the project's own domain (`CONTEXT.md`) vocabulary | PL-5 (stage-context, workflow-local per `shared-context-map.md` line 347) | J2 (same delegation as BU-P4-024 — how each sub-agent is briefed) | **FOLD — missing from the live package** | `10-design-it-twice/CONTEXT.md` (add to Behavior contract) |
| `BU-P4-026` | `10-design-it-twice/CONTEXT.md` — present designs sequentially, compare by depth/locality/seam placement, end with an opinionated recommendation (hybrid if warranted) | PL-5 | J2 (delegated: how to compare and what to recommend) | STAND | `10-design-it-twice` |
| `BU-P4-020` | `20-test-at-new-interface/CONTEXT.md` — delete old shallow-module unit tests rather than keep them alongside new ones | PL-5 | J5 (governing: "replace, don't layer" is stated as an unconditional discipline in `DEEPENING.md`, not a case-by-case judgment call) | STAND | `20-test-at-new-interface` |
| `BU-P4-021` | `20-test-at-new-interface/CONTEXT.md` — tests assert observable outcomes through the interface, not internal state | PL-5 | J5 (governing: same unconditional discipline) | STAND | `20-test-at-new-interface` |
| n/a (authoring-format compliance) | all three stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate; no stage names J2 decision classes, J1 local choices, or J0 escalation triggers in the ADR 0013 shape | N/A | J5 (`convention.md` §6.1 + ADR 0013 decision 4: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always... omission is never ambiguous" — a governing requirement this package predates) | STAND (package identity correct; in-place amendment required) | all three stage `CONTEXT.md` files |
| n/a | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| n/a | `CONTEXT.md` line 28 — "See `provenance.md` for the complete stage-to-behavior-unit mapping"; no `provenance.md` exists anywhere under `.sergeant/workflows/deepen-module/` | N/A (dangling reference, but **systemic**, not package-specific — see below) | J1 (local, cosmetic — correcting or leaving the reference changes no behavior) | STAND, no action required by this pass | `CONTEXT.md` (owner/catalog-wide, not this package) |

## The workflow-local-context gap (BU-P4-018/019/025, full record)

This package's own `reference-corpus/` provenance already records that
three behavior units — `BU-P4-018`, `BU-P4-019` (both `DEEPENING.md`
"Seam discipline") and `BU-P4-025` (`DESIGN-IT-TWICE.md` Process step 2) —
were extracted during N1 and classified `representation: shared-context`,
`workflow: deepen-module` (i.e. workflow-local context, not a
cross-workflow `.sergeant/common/contexts/` entry — `synthesis.md`'s own
"Workflow-local contexts (kept local per §6.6)" table lists `deepen-module:
BU-P4-018 BU-P4-019 BU-P4-025` explicitly, and `shared-context-map.md` line
347 repeats the same three IDs against this package).

`convention.md` §4's own rule is direct: content that is not truly shared
across two-or-more consumers "MUST be written out in full or referenced by
its actual path, not through `@@`" — i.e. workflow-local content belongs
written directly into the owning stage's `CONTEXT.md`. That never
happened here. Reading the live package end to end (Inventory, §8.3) finds
no trace of the seam-discipline rule ("one adapter is a hypothetical seam,
two adapters justify a real one") or the sub-agent-briefing rule ("include
both architecture and domain vocabulary") anywhere in
`00-classify-dependencies/CONTEXT.md` or `10-design-it-twice/CONTEXT.md`.
`00-classify-dependencies` currently states the four dependency categories
and what each implies, but never states the seam-discipline test that
governs whether a port should be exposed *once* the classification lands
on "remote-but-owned" or "true-external" — the two categories where a port
is actually in play. `10-design-it-twice` currently instructs sub-agents to
be briefed with "file paths, coupling details, dependency category... what
sits behind the seam" (via `BU-P4-023`'s neighboring content) but never
states the vocabulary-consistency requirement.

This is not a placement error — both rungs are correctly PL-5, correctly
scoped to the same two stages the provenance already names — it is a
**promotion/drafting gap**: real, cited, upstream content that N1's own
classification records as belonging to this package never actually landed
in the admitted package text. It is the same class of defect
`GAUNTLET.md` backlog row B7 named for `sergeant-setup`/`load-project`
(a duplication rather than an omission there, but the same root cause: a
package's own citation trail and its actual delivered content diverged
during promotion and nothing since has reconciled them).

**Rungs checked, for whether this producer may fix the gap directly
(bounded-judgment.md order):**

- **J5** — No governing constraint forbids adding previously-cited,
  already-adjudicated behavior-unit content into the stage it was already
  assigned to. Nothing here changes scope, public behavior, or authority —
  it completes content the corpus already classified.
- **J4** — No user/Work decision is in tension with adding it; the task
  brief instructs verifying the package's actual current content against
  its citations and completing the reconciliation.
- **J3** — The N1 provenance (`promoted-provenance/deepen-module.md`
  intersected with `reference-corpus/behavior-units/P4.ndjson`,
  `shared-context-map.md`, `synthesis.md`) is exactly the kind of "settled
  authoritative record" J3 describes: an already-adjudicated classification
  this producer did not invent, reusable and citable as-is.
- **J2** — This stage of the reconciliation method (Step 4/5/8 —
  Normalize, Placement classification, Draft) explicitly delegates
  completing a package's cited-but-undelivered content back into its
  already-classified destination.

**Conclusion: J2/J3, not J0.** Restoring `BU-P4-018`, `BU-P4-019`, and
`BU-P4-025` into their already-cited stage contracts is in-place content
completion, not a placement or disposition change, and this producer marks
it as required remediation below rather than leaving a silent gap.

## Surviving package design

No stage moves, merges, splits, or renames; PL-4 (package) / PL-5 (each
stage) is confirmed, not merely inherited from the package's own table.
Disposition is **STAND**, requiring in-place content amendment, not
restructuring:

1. Add `BU-P4-018` and `BU-P4-019` (seam discipline) to
   `00-classify-dependencies/CONTEXT.md`'s Behavior contract, citing
   `DEEPENING.md` "Seam discipline" exactly as the rest of that stage
   already cites the same file's "Dependency categories" section.
2. Add `BU-P4-025` (sub-agent briefing vocabulary) to
   `10-design-it-twice/CONTEXT.md`'s Behavior contract, citing
   `DESIGN-IT-TWICE.md` Process step 2 exactly as the rest of that stage
   already cites the same file.
3. Replace each of the three stages' `## Judgment required` boilerplate
   with a `## Bounded judgment` section per `convention.md` §7.3 /
   `.sergeant/common/contexts/bounded-judgment.md`, naming this record's
   J2 delegations, J1 local choices, and (see below) the one J0 case this
   pass identifies.
4. Add a `## Authority envelope` section to `CONTEXT.md` (L1) per
   `convention.md` §7.2.
5. Leave the catalog-wide `provenance.md` reference as-is; it is not a
   defect specific to this package (verified: 18 of the 19 other
   `.sergeant/workflows/` packages that mention `provenance.md` in their
   own `CONTEXT.md` have the identical non-local reference, pointing
   instead at `docs/gauntlet/promoted-provenance/<name>.md`) — correcting
   it catalog-wide is out of this single-package pass's scope.

One additional J0 case surfaces for the stage-level `## Bounded judgment`
sections that amendment 3 above must add to `10-design-it-twice`: none of
this stage's cited content (`BU-P4-022`..`026`) states what happens if the
opinionated recommendation `BU-P4-026` requires is itself contested or
ambiguous after comparison — i.e. no stage content currently tells the
actor when to stop recommending and ask instead of picking one design
autonomously. Checked: **J5** no constraint requires or forbids autonomous
selection; **J4** no user/Work decision pre-authorizes it either way;
**J3** no settled record addresses it; **J2** the stage delegates
comparison and recommendation, not final selection between materially
different production designs (the design choice determines the shape of
code `20-test-at-new-interface` will then commit tests against, a
downstream-binding effect, not a reversible local one); **J1** does not
apply for the same reason. **Conclusion: J0** — when amendment 3 drafts
this stage's `## Bounded judgment` section, "the recommendation is close,
contested, or the design candidates trade off materially different
production risk" should be named as a `needs_input` trigger rather than
left implicit. This producer records the gap; drafting the actual J0
clause text is amendment 3's job, not invented here as a fait accompli.

## Inputs and outputs

Inputs: all three stages' Inputs tables were read and verified against
`record-shapes.md` §1a — `00-classify-dependencies` correctly declares only
`../CONTEXT.md` (L1, first stage only); `10-design-it-twice` and
`20-test-at-new-interface` each correctly declare their immediate
predecessor's `output/README.md` (L4). No undeclared contract-bearing
dependency found; no violation of §1a rule 1.

Outputs: `00-classify-dependencies` and `10-design-it-twice` both declare
`evidence` (Work-branch record only); `20-test-at-new-interface` declares
`promote` (workflow deliverable), correctly reflecting that it is the
terminal stage. `docs/gauntlet/promoted-provenance/deepen-module.md`'s own
curation note already records that this `promote`-with-no-finalize-step
shape is accepted, human-reviewed disposition, not a defect — this pass
does not reopen that.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (`index.md`) — its structural and provenance
identity does not change. The five remediation items above are ordinary
content edits to an admitted workflow and go through this repository's
normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding previously-cited content and a required section to an
already-admitted stage's `CONTEXT.md` is neither). Per ADR 0013 decision
6, only the promotable form of this change (once actually made) needs
independent review before it lands; this adjudication record itself needs
ICM-R3's own reviewer step (`reference/proposal-icm-r-procedure-
authority.md` §8.11) before its findings are treated as settled.

## Alternatives considered

- **Treat the missing BU-P4-018/019/025 content as HARVEST into a new
  shared `.sergeant/common/contexts/seam-discipline.md`.** Rejected: the
  N1 corpus itself already classified these as workflow-local
  (`shared-context-map.md`'s "kept local per §6.6" table), meaning no
  second consumer was found to justify promotion to a truly shared
  context; `to-spec/CONTEXT.md`'s `BU-P4-052` reuses the *seam*
  vocabulary generally but restates its own seam-minimization heuristic
  rather than depending on this package's specific one/two-adapter rule,
  so it is not evidence of a second consumer of this exact rule. Revisit
  only if a second package is later found to need the identical rule
  verbatim.
- **Classify `10-design-it-twice` as PL-2 (Captain skill)** on the theory
  that "show the user," "present sequentially so [the user] can absorb,"
  and "the user wants a strong read" name live dialogue. Rejected: the
  stage's own contract (`BU-P4-023`) explicitly instructs proceeding
  without waiting for a reply, and the execution-surface test holds for an
  already-identified deepening candidate — see "Driver and admission
  boundary" above. The upstream skill's language addresses an interactive
  Claude session generically; the *package's own* stage contract already
  normalizes it correctly (non-blocking), and this pass found no
  daylight between the normalized statements and the source.
- **Resolve the new `10-design-it-twice` J0 case (contested recommendation)
  on this producer's own authority**, drafting the actual `needs_input`
  trigger text now rather than only naming the gap. Rejected: this
  reconciliation pass's job (per its own task brief) is producing the
  adjudication record, not landing the content amendments themselves;
  inventing the clause's exact wording without a reviewer pass first would
  collapse the self-check/independent-review separation this ladder exists
  to preserve (`convention.md` §6.2/6.3).
- **Leave BU-P4-018/019/025 undispositioned** on the theory that the
  package's `Final disposition` is STAND regardless, so the gap doesn't
  change the verdict. Rejected: §8.6/§8.9 require every behavior unit
  dispositioned before a package is ready to publish, and a citation that
  resolves to nothing in the live text is exactly the "package cannot be
  called authority-valid" failure mode `reference/proposal-icm-r-procedure-
  authority.md` §9.1 warns against — silence here would misrepresent this
  package as more complete than it is.

## Final disposition
STAND

## Validation evidence

- Source-valid: every citation currently in the live package was traced to
  `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md`
  and `DESIGN-IT-TWICE.md` and matches verbatim. The task brief's own
  hint — that this package "cites codebase-design's upstream DEEPENING.md
  directly per GAUNTLET.md backlog item B8's precedent" — was independently
  re-verified rather than trusted: `GAUNTLET.md` row B8 names this exact
  citation shape as the precedent for the 17 `agents-invariant` units with
  no live skill host; `.claude/skills/codebase-design` is confirmed (via
  `diff -r` and `ls -la`) to be a symlink to `.agents/skills/codebase-
  design`, so the two paths are the same content and the citation path
  used throughout this package (`.agents/skills/...`) is the canonical
  one. Additionally, three cited-but-undelivered units (BU-P4-018/019/025)
  were found by cross-checking the live package against its own archived
  provenance and the `reference-corpus/` extraction artifacts — not
  assumed complete from the package's own text.
- Placement-valid: every stage's already-recorded PL-5 rung and the
  package's own PL-4 rung were independently re-derived from the Placement
  Ladder (`reference/proposal-icm-r-procedure-authority.md` §5) in this
  pass and confirmed, including a specific check of the PL-2/PL-4
  discriminator against `10-design-it-twice`'s live-dialogue-sounding
  prose (see Alternatives considered).
- Authority-valid: **not yet** — this pass found the same class of gap
  ICM-R2's `validate-and-ship` pass found (no `## Bounded judgment` or
  `## Authority envelope` sections in the ADR 0013 shape), plus a new J0
  case surfaced while drafting the remediation this pass recommends (the
  contested-recommendation trigger for `10-design-it-twice`). The package
  cannot be called authority-valid until the five remediation items under
  "Surviving package design" land.
- Structurally valid: all three stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
