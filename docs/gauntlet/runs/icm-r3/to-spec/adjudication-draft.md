# Package adjudication: to-spec

ICM-R3 full-reconciliation pass (`docs/adr/0013-icm-r0-owner-rulings.md`;
`reference/proposal-icm-r-procedure-authority.md` §10.4). Method: proposal
§8 (Contract, Inventory, Harvest, Normalize, Placement classification,
Authority classification, Synthesis). Record shape: `docs/icm/
record-shapes.md` §6. Producer pass only — independent review is a
separate step (§8.11 of the proposal; §6.2/6.3 of `docs/icm/convention.md`)
and has not run yet. This record and the accompanying draft content are
themselves draft; neither is self-promoting (ADR 0013 decisions 6-7).

No prior hypothesis was supplied for this package at dispatch. This
adjudication investigated fresh, including re-reading the package's
current (post-N1-promotion) content in full rather than trusting its own
`CONTEXT.md`/`index.md` self-description.

## Original intention

`.sergeant/workflows/to-spec/CONTEXT.md`: "Turn a plan/design into a
published spec ticket: gathered context, sketched seams, confirmed with
the user, published on template." Candidate **W31** from the N1
reference-corpus decomposition of `reference/sergeant-upstream`
(`docs/gauntlet/contracts/N1.md`), decomposed from a single upstream
source, `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md`
(byte-identical to `.claude/skills/to-spec/SKILL.md` — verified with
`diff`, no divergent copy). Promoted per `docs/icm/
promotion-spec-2026-08-11.md`; the full behavior-unit citation trail is
archived at `docs/gauntlet/promoted-provenance/to-spec.md`. Two admitted
actor stages today (`workflow.toml` version 2, `index.md`
`status: published`): `00-gather-context`, `10-sketch-seams` (the latter
absorbing a demoted `20-write-and-publish` per N1 adjudication A4).

The upstream source is not a generic procedure document — it is itself
already an Agent/Claude *Skill* (`.agents/skills/to-spec/SKILL.md`,
`.claude/skills/to-spec/SKILL.md`), addressed to whatever harness is
already mid-conversation with a user: "This skill takes the current
conversation context and codebase understanding and produces a spec... Do
NOT interview the user — just synthesize what you already know." N1's
decomposition converted this Captain-shaped skill into a two-stage
Sergeant workflow without applying a driver/admission-boundary test — the
exact gap this proposal's §3.3 (Finding ICMR-F3) names as the reason a
Captain procedure can look workflow-shaped under the old ladder.

## Current trigger and outcome

Trigger (`CONTEXT.md`, workflow-level): "A design needs to be turned into
a spec-shaped ticket before implementation." Outcome: `00-gather-context`
synthesizes spec content (never by interview) from repository exploration
plus "what has already been discussed"; `10-sketch-seams` sketches the
minimal/highest-leverage test seams, confirms them with the user, then
(via a folded helper invocation) writes the spec onto a fixed template,
publishes it to "the project issue tracker," and applies a
`ready-for-agent` triage label "without requiring additional triage."

## Driver and admission boundary

As packaged: no stage claims `driver: deterministic-mechanism`; both
stage `CONTEXT.md` files self-declare "actor stage (ladder §6.4,
judgment)." Admission boundary as packaged: **in-work**, reachable only
through `sgt run --workflow to-spec` (or workflow-selection fallback) —
i.e. through ordinary Work admission and dispatch, like any of the other
22 published workflows.

Applying the Placement Ladder's PL-4 discriminator directly to this
package's own content (proposal §5.6: "a result that is meaningful
independent of the original conversation continuing"; "a workflow may ask
a bounded question during execution, but conversation cannot be its
primary product") surfaces a structural mismatch, not a stylistic one:

- `00-gather-context`'s own Behavior contract (`BU-P4-050`) is triggered
  by "the user asks to turn the current conversation into a spec/PRD" and
  instructs the actor to "write the spec from what has already been
  discussed" — an explicit antecedent in *another execution's private
  context*, exactly the pattern §7.7 of the proposal names for removal
  ("what you just discussed... when their antecedent is another
  execution's private context") and exactly the cross-stage-context
  violation §4.7 forbids ("a stage never relies on... any... pronoun whose
  antecedent is another execution's private context").
- The stage's own Inputs table (`00-gather-context/CONTEXT.md`) names
  exactly one input: `../CONTEXT.md` (L1, workflow orientation). No file,
  Work field, or other named artifact carries "the current conversation"
  or "the plan/design" into the stage. This is not an oversight fixable by
  adding a missing Inputs row — the dependency is structurally
  undeclarable as a file, because it is live dialogue, not a durable
  artifact. A dispatched Work has no execution to inherit that
  conversation from (§1a rule 5 of `convention.md`: each stage's own
  `CONTEXT.md`, not prior chat, is the actor's contract).
- Applying the execution-surface test (`convention.md` §2a: "would a
  human type `sgt run '<intent>' --workflow to-spec`?"): the only way to
  supply the missing conversational content through a Work intent string
  is for the user to first distill the whole prior discussion into text
  themselves — at which point the distillation (deciding what the spec
  should say, what's in scope, what decisions were made) has already
  happened *outside* this workflow, by Captain, which is precisely PL-2's
  job (proposal §5.4: "turns user conversation into a bounded
  submission"). What would remain for the dispatched workflow to do is
  thin formatting/publishing machinery, not the judgment `BU-P4-050`
  actually describes.
- `BU-P4-053` ("confirm with the user that the proposed seams match their
  expectations") is live back-and-forth dialogue before the deliverable is
  finalized — squarely Captain's ownership per proposal §4.4 ("Captain
  owns live user conversation... Sergeant owns durable execution"), not a
  single bounded question a dispatched Work asks once and waits on.

This is the same class of PL-4 contradiction the ICM-R2 pilot found in
`direct-implementation` (`docs/gauntlet/runs/icm-r2/direct-implementation/
adjudication-draft.md`, "Driver and admission boundary" section): a
package whose entire reason to exist is that the originating conversation
continues and supplies its content cannot coherently be PL-4/PL-5,
because PL-4 requires operation independent of that conversation
continuing.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-P4-050` | `00-gather-context/CONTEXT.md` — synthesize spec content from "what has already been discussed" and codebase exploration; never interview | PL-2 (driver: Captain — depends on live conversation as its content source, cannot run independent of Captain remaining present) | J2 within the skill (delegated: what counts as sufficient synthesis before drafting) | REHOME | `skills/to-spec/SKILL.md` |
| `BU-P4-051` | `00-gather-context/CONTEXT.md` — explore repo if not already done; use domain glossary; respect ADRs in the touched area | PL-3-flavored reusable technique, but single consumer today (only this skill uses it) — not promoted to a shared `.sergeant/common/contexts/` file per §5.10 ("shared/local is another modifier... becomes shared only when two or more consumers use the same contract") | J2 (delegated: which exploration is "if not already done"; which glossary/ADR terms apply) | REHOME | `skills/to-spec/SKILL.md` (local content, not a shared context) |
| `BU-P4-052` | `10-sketch-seams/CONTEXT.md` — sketch minimal, highest-leverage test seams before drafting the implementation section | PL-3-flavored reusable technique, single consumer, same reasoning as `BU-P4-051` | J2 (delegated: which existing seam is highest/most reusable; how many new seams, if any, are unavoidable) | REHOME | `skills/to-spec/SKILL.md` |
| `BU-P4-053` | `10-sketch-seams/CONTEXT.md` — confirm the seam plan with the user before finalizing | PL-2 (live dialogue, proposal §4.4) | Not a J-rung question — this is Captain's "must ask the user" bucket (§7.4 skill shape), not a delegated decision | REHOME | `skills/to-spec/SKILL.md` |
| `BU-P4-054` | `10-sketch-seams/CONTEXT.md` Helper section (folded `20-write-and-publish`, N1 adjudication A4) — write on the fixed template, publish to "the project issue tracker," apply `ready-for-agent` "without requiring additional triage" | Drafting = PL-2 (Captain judgment, same reasoning as above); publish/label-apply = PL-6 ("ordinary tool invocation: no new packaged surface at all", proposal §5.8) | **J0** — see "The ready-for-agent conflict" below | REHOME (drafting); publish/label mechanics stated as an ordinary tool step, with the J0 gap flagged, not resolved, by this producer | `skills/to-spec/SKILL.md` |

No unit in this package survives at PL-4 or PL-5: nothing here requires a
fresh, isolated stage execution, an artifact handoff between distinct
actors, or independent review of one stage's output by another (the tests
proposal §5.7 sets for PL-5). No unit requires new runtime capability
(PL-7 unreached; every lower rung was attempted first, per the ladder's
own first-honest-rung rule, proposal §4.8).

## The ready-for-agent conflict (`BU-P4-054`, full record)

This package's own text instructs applying the `ready-for-agent` triage
label "without requiring additional triage." But this repository already
has an admitted, published package that owns the meaning of that exact
outcome: `.sergeant/workflows/triage/50-apply-outcome/CONTEXT.md`,
`BU-P3-069`: "Reaching the ready-for-agent outcome requires posting a
structured agent brief comment." These are two settled records inside the
same admitted catalog, and they directly disagree about what "reaching
ready-for-agent" requires.

Separately, the source skill (`reference/sergeant-upstream/.agents/
skills/to-spec/SKILL.md` line 9) says: "The issue tracker and triage label
vocabulary should have been provided to you — run
`/setup-matt-pocock-skills` if not." `grep -rl matt-pocock` across the
live tree (excluding `reference/sergeant-upstream/` and the retired
`docs/gauntlet/runs/n2-run4/` draft) returns nothing: no such setup skill
was ever ported into sergeant-rs, and this repository's own tracked-work
mechanism is `td` (`AGENTS.md`; `skills/sergeant-setup`'s successor
content in `docs/gauntlet/runs/icm-r2/sergeant-setup/draft/skills/
estate-navigation/SKILL.md`'s "Filing tracked work" section), not a
generic "project issue tracker" with an externally-supplied label
vocabulary. The admitted `to-spec` package's own destination is therefore
already stale against the repository's current tracked-work model, quite
apart from the placement question above.

**Rungs checked, for this producer's own classification of the gap's
current state — not a resolution of the underlying policy:**

- **J5** — No governing constraint requires or forbids a spec-drafting
  procedure from posting directly to `ready-for-agent`; nothing forbids
  requiring an agent-brief step either.
- **J4** — No explicit user or bound-Work decision visible in either
  package states which convention governs when the two disagree.
- **J3** — Two settled records exist and conflict: `to-spec`'s own text
  ("without requiring additional triage") and `triage`'s own text
  ("requires posting a structured agent brief comment"). Per the
  conflict rule (`bounded-judgment.md` §"Conflict rule" /
  `@@bounded-judgment`): two settled records in genuine conflict resolve
  to J0, not to whichever this producer finds more convenient.
- **J2** — Neither package's content names "which of these two conventions
  governs when both could apply" as a delegated decision class.
- **J1** — Does not apply: which convention a published, `ready-for-agent`-
  labeled ticket must satisfy is exactly the kind of choice that changes
  a downstream actor's (a dispatched agent's) contract — not local or
  reversible once an agent has already started work believing the label
  alone was sufficient.

**Conclusion: J0.** This producer does not resolve which convention
governs; the surviving skill draft below states the conflict explicitly
so the acting harness (and the independent reviewer) sees it rather than
guessing, and does not fabricate a resolution on the owner's behalf.

## Surviving package design

The whole package REHOMEs to a new Captain skill,
`skills/to-spec/SKILL.md`, ported directly from the upstream source with
its native `SKILL.md` shape restored (front matter `name`/`description`,
a `## Bounded judgment` section per `docs/icm/convention.md` §7.3/§6.1,
same shape as `skills/grilling/SKILL.md`). This is a straight REHOME, not
a HARVEST: every behavior unit above still needs a home, and that home is
one coherent new skill (unlike `direct-implementation`, where every unit
turned out to already be owned by an existing surface). The spec template
itself, the seam-sketching technique, and the synthesis-not-interview rule
travel together because they are all steps of the same single-session
Captain interaction, not independently reusable across other packages
today (`BU-P4-051`/`BU-P4-052`'s PL-3 flavor, addressed above under
"Behavior-unit dispositions" — no second consumer exists to justify a
shared `.sergeant/common/contexts/` extraction; if a future package needs
the same seam-sketching technique, extracting it then is the correct
order, not now on speculation, per proposal §8.8's "a source file mapping
one-to-one onto a new package is not evidence of correctness" — the
inverse failure, over-splitting on hypothetical reuse, is equally a
violation).

`.sergeant/workflows/to-spec/` does not survive: deleted on
reconciliation, per the same precedent already set by `grilling`'s and
`sergeant-help`'s prior retirements (both fully removed from
`.sergeant/workflows/` once ported to `skills/`).

The `ready-for-agent` conflict (see above) is carried into the draft
skill as an explicit, unresolved note — the skill instructs the harness
to ask the user (or consult whichever of `to-spec`/`triage`'s conventions
the user names) rather than silently picking one, until the owner rules
on it.

## Inputs and outputs

Inputs consumed by this adjudication: the package's own six files under
`.sergeant/workflows/to-spec/` (`CONTEXT.md`, `index.md`, `workflow.toml`,
two stage `CONTEXT.md`/`output/README.md` pairs); `docs/gauntlet/
promoted-provenance/to-spec.md` (archived N1 provenance); both copies of
the upstream source skill (`reference/sergeant-upstream/.agents/skills/
to-spec/SKILL.md` and `.claude/skills/to-spec/SKILL.md`, confirmed
byte-identical); `.sergeant/workflows/triage/CONTEXT.md` and
`50-apply-outcome/CONTEXT.md` (the conflicting settled record);
`skills/grilling/SKILL.md` (rehome-shape and `## Bounded judgment`
precedent); `.sergeant/index.md` (catalog listing, line 40) — read, not
edited, per this task's own instruction to leave catalog/routing changes
to the reconcile-and-publish step.

Output: this record, plus `draft/skills/to-spec/SKILL.md` (the full
REHOMEd skill content, not a live edit — ADR 0013 decision 6,
promotable-only review).

## Review and promotion policy

Artifact class: package-adjudication record plus one draft skill file.
Draft location: `docs/gauntlet/runs/icm-r3/to-spec/` (this record) and its
`draft/skills/to-spec/SKILL.md` (proposed destination content, mirroring
the destination's own path under the repository root the way
`docs/gauntlet/runs/icm-r2/sergeant-setup/draft/skills/
estate-navigation/SKILL.md` did for its own REHOMEd/extended content).
Independent reviewer: the ICM-R3 reconciliation's own review step (a
later, independently executed position per `docs/adr/0013` decision 7 —
fresh execution, explicit inputs, review-only contract, no edit authority
over this record). Acceptance criteria: every behavior-unit disposition
resolves to a real destination; the PL-4 contradiction argument holds
under adversarial challenge; the `ready-for-agent` conflict is either
confirmed as a genuine J0 or shown to be reconcilable at a lower rung this
producer missed; no citation is fabricated. Promotion action: Captain's
reconcile-and-publish pass (§8.12 of the proposal) accepts/rejects/
merges/parks each finding, then — only if accepted — deletes the live
`.sergeant/workflows/to-spec/` directory, moves `draft/skills/to-spec/
SKILL.md` into `skills/to-spec/SKILL.md`, and updates `.sergeant/
index.md` (removing the `to-spec` workflow row) in the same change.
Failure/remediation: if review finds the PL-4 contradiction does not hold
(e.g., a future Work-intent convention makes "paste the whole prior
discussion into the intent string" a coherent, load-bearing pattern for
other packages too), the package reverts toward `STAND` and this record
is superseded, not silently edited.

## Alternatives considered

- **STAND** (keep as a two-stage published workflow, only add the missing
  `## Bounded judgment`/`## Authority envelope` sections in place, the way
  `validate-and-ship` did at ICM-R2). Rejected: unlike `validate-and-ship`,
  whose stages already pass the execution-surface test and merely lacked
  authority-section boilerplate, `to-spec`'s core behavior unit
  (`BU-P4-050`) fails the PL-4 discriminator on its own content — no
  amount of added boilerplate resolves a dependency on live conversational
  context that cannot be named as a file.
- **SPLIT into a Captain-skill stub that gathers/confirms, plus a
  durable workflow stage that only does the mechanical write-and-publish.**
  Considered seriously. Rejected as the final shape here: the "mechanical"
  half left over (`BU-P4-054`'s publish/label step) is thin enough — one
  templated write, one tracker call, one label — that it is PL-6 "ordinary
  tool invocation," not a checkpoint whose independent retry, evidence, or
  authority envelope operators would care about (proposal §5.7's
  reimplementation test: replacing the publish mechanism tomorrow would
  not change what any operator cares about at a boundary). A dispatched
  stage whose entire body is "call the tracker API" is ceremony around a
  tool call, not a workflow (same shape as the rejected `direct-
  implementation` skill-wrapper alternative at ICM-R2). Also, no nested-
  workflow invocation mechanism exists today for a skill to hand off
  mid-stream to a durable stage and get a result back (`docs/icm/
  convention.md` §4 rule 1 — `@@name` is context composition, not
  workflow composition); inventing one here would be an unjustified PL-7
  claim disguised as a package decision.
- **HARVEST into an existing skill** (e.g. `estate-navigation`, on the
  theory that "file tracked work" content already lives there). Rejected:
  `estate-navigation`'s tracked-work section is about filing a gap `sgt
  doctor` can't remedy — a different trigger and a different artifact
  shape (a `td` issue for a capability gap, not a full spec ticket with
  user stories/implementation/testing sections). Forcing `to-spec`'s
  content into that skill would conflate two independently triggerable
  behaviors into one file, the same "false pairing" §8.11 of the proposal
  asks reviewers to watch for.
- **Resolve the `ready-for-agent` conflict unilaterally** (e.g., silently
  require the agent-brief comment `triage` demands, since that is the
  more specific/recent-looking record). Rejected per this Work's own
  brief and `bounded-judgment.md`'s J0 procedure: a producer at J0 states
  the gap and may offer a recommendation as evidence, but does not
  substitute its own guess for the owner's ruling, especially where a
  wrong guess changes a downstream agent's contract (whether an
  agent-brief comment exists before it starts work).

## Final disposition
REHOME

## Validation evidence

- Read every file under `.sergeant/workflows/to-spec/` in full
  (`CONTEXT.md`, `index.md`, `workflow.toml`, both stage `CONTEXT.md` and
  `output/README.md` pairs) before classifying, not just the workflow's
  own top-level description.
- Read both copies of the upstream source (`.agents/skills/to-spec/
  SKILL.md`, `.claude/skills/to-spec/SKILL.md`) and confirmed via `diff`
  that they are byte-identical — no divergent-copy risk to reconcile.
- Independently re-derived the PL-4 discriminator failure from the
  package's own current `CONTEXT.md`/stage-contract text (the "current
  conversation" antecedent, the single-input Inputs table), not merely
  from this task's own dispatch instructions — the instructions supplied
  no hypothesis for this package, so this finding was reached fresh.
- Cross-checked `BU-P4-054`'s "apply the ready-for-agent triage label
  without requiring additional triage" against the actually-admitted
  `triage` workflow's own `50-apply-outcome/CONTEXT.md` (`BU-P3-069`) and
  found a genuine, previously unrecorded conflict between two admitted
  packages — confirmed by reading `triage`'s stage contract directly, not
  inferred from its name.
- Grepped the live tree (excluding `reference/sergeant-upstream/` and the
  retired `docs/gauntlet/runs/n2-run4/` draft) for `matt-pocock` and
  confirmed no such setup skill was ever ported — the upstream source's
  own dependency is unmet in this repository, independent of the
  placement question.
- Read `skills/grilling/SKILL.md` and `docs/gauntlet/runs/icm-r2/
  sergeant-setup/draft/skills/estate-navigation/SKILL.md` as REHOME/SPLIT
  shape precedents before drafting this package's own skill content and
  before rejecting the HARVEST-into-`estate-navigation` alternative.
- This record is a producer-step output only; it has not yet passed
  independent adversarial review (proposal §8.11) or Captain's
  reconcile-and-publish pass (§8.12); it carries no promotion authority on
  its own (`docs/adr/0013-icm-r0-owner-rulings.md` decision 6).
