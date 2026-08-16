# Package adjudication: wayfinder

ICM-R3 full-reconciliation pass (`reference/proposal-icm-r-procedure-
authority.md` §10.4; method per §8; record shape per
`docs/icm/record-shapes.md` §6). Producer pass only — independent review is
a separate step (§8.11 of the proposal; §6.2/6.3 of `docs/icm/
convention.md`) and has not run yet. This record is itself draft; it does
not self-promote (ADR 0013 decision 6, decision 7).

## Original intention

Plan a body of work too large for one agent session as a shared map of
decision tickets on the repository's issue tracker, and resolve those
tickets one at a time until the way to a named destination is clear
(`reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md`
frontmatter description, L3; `BU-P4-075`). Promoted into the N1 reference
corpus as candidate **W33** (`docs/gauntlet/contracts/N1.md`,
`docs/icm/promotion-spec-2026-08-11.md`), with a full behavior-unit
citation trail archived at
`docs/gauntlet/promoted-provenance/wayfinder.md`. Already adjudicated as
**WORKFLOW**, not a Captain skill, at the 2026-08-11 retriage
(`docs/icm/retriage-2026-08-11.md:38`: "`30-resolve-one` claims/resolves
one ticket at a time with a loop back to `10-map-frontier` — genuine
per-decision judgment"), with a noted partial dependency on `grilling` for
`00-name-destination` only. This ICM-R3 pass does not re-run the N1
extraction from upstream `SKILL.md`; it (a) applies the Placement and
Bounded-Judgment ladders on top of the already-cited N1 content, (b)
checks the package's compliance with ADR 0013, and (c) — per this Work's
own inventory instruction to read the entire package plus every file it
delegates to — checks the *live* package's content against its own already-
adjudicated citation trail (`docs/gauntlet/promoted-provenance/
wayfinder.md`, `reference-corpus/{synthesis,helper-map,shared-context-
map,provenance-map}.md`), which surfaced a completeness gap described
below (BU-P4-077 through BU-P4-084).

## Current trigger and outcome

Four ordinary actor stages, no branching in `workflow.toml`
(`00-name-destination` → `10-map-frontier` → `30-resolve-one` →
`40-regraduate-fog`, looping back to `10-map-frontier` by fresh
re-invocation when fog remains — see BU-WAY-09 below):

- **Trigger:** a destination is named that requires mapping fog before it
  can be reached (`CONTEXT.md` "Trigger"; `index.md` "Use when").
- **Outcome:** a map (a tracker issue) with a named destination, a set of
  child-issue decision tickets resolved one at a time (at most one
  non-research ticket per session), and a Decisions-so-far index that
  durably records the route actually walked, until no tickets remain.

## Driver and admission boundary

Driver: **stage actor**, all four stages. Admission boundary: **in-Work**
— the workflow receives an already-named (or, at `00-name-destination`,
about-to-be-settled-live) destination and a bounded outcome (a resolved or
partially resolved map), not live Captain dialogue about *whether* Work
should exist. Applying the execution-surface test (`convention.md` §2a,
also used in the `validate-and-ship` ICM-R2 precedent): "would a human
type `sgt run '<intent>' --workflow wayfinder`?" — yes, once a destination
worth mapping is named; the four-stage sequence is durable, checkpointed,
and meaningful independent of whether the invoking Captain session is
still open (fresh execution per stage, per `docs/icm/convention.md` §1a).
This matches the already-recorded retriage disposition (**WORKFLOW**,
`docs/icm/retriage-2026-08-11.md:38`) and the package's own stage table
(every stage `actor-stage (§6.4, judgment)`).

One stage narrows this boundary locally: `00-name-destination` delegates
its own outcome to the **grilling** *skill* (`skills/grilling/SKILL.md`),
run live in the current session rather than dispatched as a separate Work
— see BU-WAY-01 below. This is a stage narrowing its own execution
locally, not the workflow's admission boundary; the workflow as a whole is
still admitted once, in-Work, per the retriage disposition above.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-WAY-01 | `00-name-destination/CONTEXT.md` (Delegation) — the destination is named by running the **grilling** operator skill live, in-session, not by dispatching a Work item; citation: `BU-P4-094`, upstream SKILL.md "Invocation / Chart the map" L111 | PL-5 (stage), narrowing to PL-2 (Captain-driven, in-session) for the delegated sub-step | J5 (governing: R-NS-6 — conversation is the harness's job, never engine work, `docs/icm/re-homing-record-2026-08-12.md`) | STAND | `00-name-destination` |
| BU-WAY-02 | `10-map-frontier/CONTEXT.md` — breadth-first mapping; stop and do not create a map if no fog exists; ask the user how to proceed instead; citations `BU-P4-095`, `BU-P4-088`, `BU-P4-089` | PL-5 | J2 (delegated: fog-vs-ticket sharpness test, named in the stage's own Behavior contract) with a **J0** carve-out (no fog found → the stage must ask the user how to proceed rather than silently produce a trivial map) | STAND | `10-map-frontier` |
| BU-WAY-03 | `10-map-frontier/CONTEXT.md` — out-of-scope work is never recorded as fog; citation `BU-P4-091` | PL-5 | J2 (delegated: scope-vs-fog classification) | STAND | `10-map-frontier` |
| BU-WAY-04 | `10-map-frontier/CONTEXT.md` Helper section (folded `20-create-tickets`, N1 adjudication A4) — create specifiable tickets first, wire blocking edges in a second pass; citation `BU-P4-096` | PL-6 (deterministic-mechanism helper, correctly demoted per A4's default rule — no additional checkpoint argument existed) | J5 (governing: two-pass order is a hard sequencing constraint — issues need ids before they can reference each other) | STAND | `10-map-frontier` (helper) |
| BU-WAY-05 | `30-resolve-one/CONTEXT.md` — load only the low-resolution map body, choose the named or first-frontier ticket, claim by self-assignment before starting work; citation `BU-P4-098` | PL-5 | J2 (delegated: which ticket to choose when the user did not name one) + J5 (governing: claim before work, always) | STAND | `30-resolve-one` |
| BU-WAY-06 | `30-resolve-one/CONTEXT.md` — HITL vs. AFK ticket types; the agent never answers on the human's behalf for a HITL ticket; citation `BU-P4-085` | PL-5 | J5 (governing: HITL ticket resolution requires a live human exchange — the agent may not self-answer) | STAND | `30-resolve-one` |
| BU-WAY-07 | `30-resolve-one/CONTEXT.md` — research-type and task-type ticket resolution; citations `BU-P4-086`, `BU-P4-087` | PL-5 | J2 (delegated: whether an item is research/task/decision, per the named types) | STAND | `30-resolve-one` |
| BU-WAY-08 | `30-resolve-one/CONTEXT.md` — at most one non-research ticket per session; recording a resolution as comment + close + one-line Decisions-so-far pointer; out-of-scope discovery mid-resolution is closed and recorded, not resolved on-route; citations `BU-P4-093`, `BU-P4-099`, `BU-P4-092` | PL-5 | J5 (governing: the one-ticket-per-session cap and the fixed three-part resolution sequence) + J2 (delegated: in-scope vs. out-of-scope reclassification during resolution) | STAND | `30-resolve-one` |
| BU-WAY-09 | `40-regraduate-fog/CONTEXT.md` — plan-don't-do default, overridable only via the effort's own Notes; parallel research-subagent firing; concurrent-editing awareness; loop back to `10-map-frontier` represented as fresh re-invocation, not engine-level looping; citations `BU-P4-076`, `BU-P4-097`, `BU-P4-100`; engine-gap `BU-P4-090` (G7, dynamic ticket graph) | PL-5 (stage); the rejected G7 claim is correctly resolved below PL-7 | J4 (an effort's own Notes are an explicit, already-recorded standing decision the stage must honor without reconfirming) + J5 (governing: G7 is closed — no engine looping primitive is available or needed; `reference-corpus/synthesis.md` §5, "ownership preference, not a representational failure") | STAND | `40-regraduate-fog` |
| BU-WAY-10 | `reference-corpus/shared-context-map.md` Part 3 + `helper-map.md` "Workflow-local helpers" — "Refer by name" section (`BU-P4-077`, `BU-P4-080`, upstream "Refer by name" L17) and "The Map" structural/mechanical section (`BU-P4-078`, `BU-P4-079`, `BU-P4-081`–`BU-P4-084`, upstream "The Map" L21) — **already dispositioned by the reference corpus as belonging to this package (workflow-local context / workflow-local helper), but absent from every file in the live `.sergeant/workflows/wayfinder/` tree** | PL-6 (map/ticket structural mechanics: single tracker issue labeled `wayfinder:map`, tickets as child issues sized to one session, `wayfinder:<type>` labels, claim-by-assignment, native-blocking dependency, map-as-index-not-store) and PL-3/local-context (naming-by-title-not-bare-id convention, `BU-P4-077`/`080`) | **not yet dispositioned in-package** — see "The missing-content gap" below | intended: `10-map-frontier` (helper, map/ticket structure) and workflow-level `CONTEXT.md` or `00-name-destination` (local context, refer-by-name) |
| BU-WAY-11 | `CONTEXT.md` (L1) "Provenance" — "See `provenance.md` for the complete stage-to-behavior-unit mapping" — **no `provenance.md` exists under `.sergeant/workflows/wayfinder/`**; the actual file is `docs/gauntlet/promoted-provenance/wayfinder.md` | N/A (dangling reference, not a placement question) | — | **FOLD** (correct the reference in place; no placement change) | `CONTEXT.md` |
| BU-WAY-12 | All four stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph, none names J2 delegations, J1 local choices, or J0 escalation triggers in the shape `convention.md` §6.1/§7.3 requires | N/A (authoring-format compliance) | J5 (`docs/icm/convention.md` §6.1, ADR 0013 decision 4: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always ... omission is never ambiguous" — a governing requirement this package predates and does not yet satisfy) | STAND (package identity correct; in-place content amendment required) | all four stage `CONTEXT.md` files |
| BU-WAY-13 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |

## The missing-content gap (BU-WAY-10, full record)

Reading the live package end to end and cross-checking it against its own
already-archived citation trail (`docs/gauntlet/promoted-provenance/
wayfinder.md`) and the reference corpus's own synthesis decisions
(`reference-corpus/provenance-map.md:146`, which lists all 26 units
`BU-P4-075` through `BU-P4-100` as sourced from upstream `SKILL.md`)
surfaces a completeness gap the promotion step did not catch:

- `reference-corpus/shared-context-map.md` Part 3 ("Workflow-local
  contexts, kept local per §6.6") assigns `BU-P4-077`, `BU-P4-080`
  ("Refer by name", upstream L17) to `W33 wayfinder` — every map and
  ticket is referred to by its name/title in human-facing narration, never
  a bare id, number, or slug.
- `reference-corpus/helper-map.md` "Workflow-local helpers" assigns
  `BU-P4-078`, `BU-P4-079`, `BU-P4-081`–`BU-P4-084` ("The Map", upstream
  L21) to `W33 wayfinder` — the map is a single tracker issue labeled
  `wayfinder:map`; tickets are its child issues, sized to one ~100K-token
  session, each carrying exactly one `wayfinder:<type>` label; a session
  claims a ticket by self-assignment before any work; blocking uses the
  tracker's native dependency relationship (falling back to a body
  convention only if the tracker lacks it); the map is an index that
  gists and links, never restating a decision already recorded on its
  ticket.
- `docs/icm/agents-invariant-dispositions.md` independently corroborates
  the same set from the AGENTS.md-candidate side: BU-1001 (refer by
  name), BU-1002 (map is a single issue, tickets are children), BU-1003
  (map is index not store), BU-1005 (open tickets found by query, not
  listed inline), BU-1006 (four `wayfinder:<type>` labels), BU-1008
  (native-blocking-first dependency rule) are all dispositioned `skill:
  wayfinder` — i.e., *this* package's own subject matter, not a
  cross-cutting AGENTS.md invariant — with the explicit note "Map/ticket/
  HITL-AFK/fog-of-war rules are that workflow's own subject matter
  (published WORKFLOW per retriage)."

None of these eight upstream-sourced units (`BU-P4-077`–`BU-P4-084`)
appear anywhere in `.sergeant/workflows/wayfinder/` today — not in
`CONTEXT.md`, not in `00-name-destination` or `10-map-frontier` (the two
stages where map creation and naming actually happen), not in any
`references/` file (none exists). The package's stages already *assume*
this structure — `30-resolve-one/CONTEXT.md` instructs claiming "by
self-assignment" (`BU-P4-098`) and choosing "the first frontier ticket in
order," but the frontier/claim/label/blocking mechanics that make those
instructions actionable were never actually landed in the package that
depends on them. This is a promotion-step gap, not a placement error: the
reference corpus already correctly classified where this content belongs
(workflow-local helper → `10-map-frontier`; workflow-local context →
`CONTEXT.md` or `00-name-destination`); it simply was never written when
the package was promoted (`docs/icm/promotion-spec-2026-08-11.md`).

This is recorded as a disposition gap, not resolved by this producer: per
this Work's task brief and `bounded-judgment.md`'s own J2 discipline, a
producer may classify and cite content but authoring wholesale new stage
prose from upstream evidence the corpus already extracted is exactly the
"complete behavior-unit adjudication records; produce revised draft
packages" work the *next* review/remediation pass — not this one — is
scoped to perform once this finding is accepted. Recommendation: land
`BU-P4-078/079/081-084` as a new "Helper invocation: map and ticket
structure" subsection in `10-map-frontier/CONTEXT.md` (alongside the
existing "Helper invocation: create tickets"), and `BU-P4-077/080` as a
short "Refer by name" note in either the workflow-level `CONTEXT.md` or
`00-name-destination/CONTEXT.md` (both stages already load `../CONTEXT.md`
as an L1 input, so a workflow-level home is more directly usable by every
downstream stage).

Two upstream ticket-type subsections — **Prototype** and **Grilling**
(`reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md` "Ticket
Types") — were never extracted as behavior units at all in the original
N1 harvest (only `BU-P4-085` generic HITL/AFK, `BU-P4-086` research, and
`BU-P4-087` task exist; no unit names Prototype or Grilling as a distinct
ticket type). `30-resolve-one/CONTEXT.md` today only names research and
task tickets by name; a Prototype- or Grilling-type ticket is currently
resolvable only under the generic HITL/AFK rule (`BU-P4-085`), with no
stage content naming which skill it should invoke. This is a citation
completeness gap in the *original* N1 extraction, one layer upstream of
this package's own promotion — flagged here as found during Inventory,
not resolved: re-opening N1 extraction is out of this producer's own
scope (the task brief scopes this pass to applying the ladders to
already-cited content, following the ICM-R2 `validate-and-ship` precedent
of not re-running extraction).

## Surviving package design

No stage moves, merges, splits, or renames. The four-stage sequence
(`00-name-destination`, `10-map-frontier`, `30-resolve-one`,
`40-regraduate-fog`), its single delegation to `grilling`, and every
already-cited N1 behavior unit remain correctly placed at PL-4 (package) /
PL-5 (each stage) / PL-6 (the one identified helper, `BU-P4-096`). The
package requires **in-place content amendment**, not restructuring:

1. Add the eight missing behavior units (`BU-P4-077`–`BU-P4-084`) per the
   "missing-content gap" section above — a helper subsection in
   `10-map-frontier/CONTEXT.md` and a local-context note at workflow level
   or in `00-name-destination/CONTEXT.md`.
2. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the four stage `CONTEXT.md` files,
   replacing the current `## Judgment required` boilerplate with named J2
   delegations, J1 local choices, and J0 escalation triggers specific to
   that stage — largely a direct restatement of the J-boundary column
   above, which was derived from this package's existing Behavior contract
   prose.
3. Add an `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2).
4. Correct the dangling `provenance.md` reference in `CONTEXT.md`'s
   "Provenance" section to point at `docs/gauntlet/promoted-provenance/
   wayfinder.md`, the file that actually carries the stage-to-behavior-unit
   mapping.
5. (Lower priority, flagged not required) Consider whether Prototype- and
   Grilling-type tickets deserve their own named behavior units in a
   future N1-extraction pass, per the note above.

None of these five amendments changes which package owns the behavior, so
none triggers this ADR's REHOME/SPLIT/HARVEST draft-and-rehome step
(`docs/adr/0013-icm-r0-owner-rulings.md` decision 6; task brief). They are
recorded here as the concrete remediation this adjudication found, for the
owner/reviewer to schedule — following the same shape the ICM-R2
`validate-and-ship` precedent used for its own BU-VAS-13/14/15 findings.

## Inputs and outputs

Inputs: each of the four stages' Inputs tables was checked against
`record-shapes.md` §1a — all four comply (paths resolve inside the
workflow's own directory; `Layer` values are `L1`/`L4`; every `L4` input
names a file an earlier stage's `output/` declares, in stage order). The
one delegation (`00-name-destination` → `grilling`) is prose delegation to
an operator skill, not a workflow-composition reference, and is correctly
not listed as an Inputs-table row (it is not a file dependency).

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. Three of four are `evidence` (Work-branch record only);
`40-regraduate-fog`'s is `promote` (workflow deliverable — the resolved or
partially resolved map itself), correctly reflecting that it is the
workflow's terminal stage. `40-regraduate-fog/output/README.md` itself
already flags, at promotion time, that the workflow has no dedicated
finalize step (D9, `convention.md` §1a open questions) — recorded there as
a surfaced gap, not a defect in this stage's own disposition; this
adjudication does not re-litigate that already-recorded finding.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change under this disposition. The five remediation
items above are ordinary content edits to an admitted workflow and should
go through this repository's normal review path for workflow content
changes, not a new draft-and-promote cycle (`docs/icm/convention.md` §2 —
the draft/admitted split governs *new or substantially rewritten* content;
adding missing sections and correcting a dangling reference in an
already-admitted package is neither). Per ADR 0013 decision 6, only the
promotable form of this change (once actually made) needs independent
review before it lands; this adjudication record itself, being ICM-R3
producer output, needs the reconciliation's own independent-reviewer step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **REHOME `00-name-destination` (or the whole package) to a Captain
  skill**, on the theory that "settle scope via a live interview" looks
  conversational start-to-finish. Rejected: the workflow as a whole passes
  the execution-surface test (`convention.md` §2a) — `10-map-frontier`
  through `40-regraduate-fog` are fresh, durable, stage-bound executions
  with declared Inputs/outputs and a real multi-session durable outcome (a
  tracker-resident map), not live dialogue about whether Work should
  exist. Only the first stage's sub-step narrows locally to a live skill
  invocation, exactly as the already-recorded retriage disposition
  anticipated (`docs/icm/retriage-2026-08-11.md:38`'s "partial E3
  dependency for that sub-step only").
- **Treat the BU-P4-090 loop-back (G7) as reopened for this pass.**
  Rejected: G7 was already adjudicated and rejected
  (`reference-corpus/synthesis.md` §5; `engine-pressure.md:809`) as an
  ownership preference, not a representational gap, and
  `40-regraduate-fog/CONTEXT.md`'s own "Additional note" already records
  the correct resolution (loop-back represented as fresh re-invocation).
  Re-opening it without new evidence would violate the "first honest
  rung" discipline (proposal §4.8) this ladder exists to enforce.
- **Author the eight missing behavior units' full stage prose now, on
  this producer's own authority**, resolving BU-WAY-10 rather than just
  recording it. Rejected: the reconciliation method's own step ordering
  (§8.2–§8.7: contract, inventory, harvest, normalize, classify — *then*
  synthesize/draft) and this ADR's own producer/reviewer separation (ADR
  0013 decisions 6–7) argue for recording the classified gap now and
  letting the scheduled remediation pass draft and land the content, the
  same way the ICM-R2 `validate-and-ship` precedent recorded its own
  BU-VAS-13/14/15 gaps without drafting their fixes inline.
- **Treat BU-WAY-10 or the missing Prototype/Grilling ticket-type units as
  an engine-gap (PL-7) claim.** Rejected outright: nothing about either
  gap requires the runtime to own a new durable fact; both require only
  that this package's own content (or, for the ticket types, the upstream
  N1 harvest) state behavior it currently omits. Lower rungs have not even
  been attempted, so PL-7 is unreached per the ladder's first-honest-rung
  rule (proposal §4.8).

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  four stage `CONTEXT.md` files and its `CONTEXT.md`/`index.md`/
  `workflow.toml` was read in full and traced to its already-archived N1
  provenance (`docs/gauntlet/promoted-provenance/wayfinder.md`); no new
  citation was fabricated for this pass. The eight additional units named
  in BU-WAY-10 were independently cross-checked against
  `reference-corpus/{shared-context-map,helper-map,provenance-map,
  synthesis}.md` and `docs/icm/agents-invariant-dispositions.md`, all of
  which agree on their assignment to this package.
- Placement-valid: every stage's already-recorded PL-5 rung
  (`actor-stage (§6.4, judgment)`) was independently re-derived from the
  Placement Ladder in this pass and confirmed. The package's WORKFLOW
  (PL-4) disposition, and the local PL-2 narrowing at
  `00-name-destination`'s delegated sub-step, were independently
  re-derived and confirmed against the already-recorded retriage finding
  and the *current* (post-rehome) content of `skills/grilling/SKILL.md`,
  per this Work's own package-specific instruction to verify that
  citation is accurate — confirmed accurate: the delegation correctly
  names the retired-workflow-to-skill history (R-NS-6, the resolved E3
  dependency) and the current `grilling` skill content matches what
  `00-name-destination/CONTEXT.md` describes.
- Authority-valid: **not yet** — this is precisely what BU-WAY-12/13
  found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the five remediation items under "Surviving package design" land.
- Structurally valid: all four stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. One structural
  violation found and recorded: `CONTEXT.md`'s "Provenance" section names
  a `provenance.md` file that does not exist in this package's own tree
  (BU-WAY-11).
- Completeness-valid (this pass's own addition, beyond the five §9.1
  claims, following the same "read the entire package" inventory
  discipline the proposal's §8.3 requires): **not yet** — BU-WAY-10 found
  eight already-adjudicated behavior units missing from the live package
  entirely, and a further two upstream subsections (Prototype/Grilling
  ticket types) that were never extracted as behavior units at all.
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the package;
  `reference/proposal-icm-r-procedure-authority.md` §9.3's execution-
  validation claims (needs_input on a real/scripted J0 case, operation
  without Captain present) remain to be measured separately.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6–7); it
  does not self-promote.
