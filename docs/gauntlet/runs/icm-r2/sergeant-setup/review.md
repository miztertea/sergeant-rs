# Package adjudication review: sergeant-setup

Independent adversarial review, ICM-R2 pilot (`reference/proposal-icm-r-
procedure-authority.md` §8.11; ADR 0013 decision 7 — this is a later,
review-only execution in the same workflow, fresh inputs, no edit
authority over the producer's draft or the live package). Reviews
`docs/gauntlet/runs/icm-r2/sergeant-setup/adjudication-draft.md` against
the actual live package content at `.sergeant/workflows/sergeant-setup/`,
`AGENTS.md`, `skills/estate-navigation/SKILL.md`,
`.sergeant/workflows/load-project/`, and the cited MVP-1/synthesis/retriage
sources — not against the producer's own citations taken on faith.

Files read in full for this review: `.sergeant/workflows/sergeant-setup/
{CONTEXT.md,index.md,workflow.toml,_config/standing-constraints.md,
05-file-capability-gaps/CONTEXT.md,05-file-capability-gaps/output/README.md,
30-project-interview/CONTEXT.md,30-project-interview/output/README.md}`;
`.sergeant/workflows/load-project/{CONTEXT.md,20-register-or-edit/
CONTEXT.md}`; `skills/estate-navigation/SKILL.md` (live) and the draft at
`docs/gauntlet/runs/icm-r2/sergeant-setup/draft/skills/estate-navigation/
SKILL.md`; `AGENTS.md` (routing table, Guardrails); `docs/icm/convention.md`
§6; `docs/gauntlet/contracts/MVP-1.md` (R-MVP1-3/4/5); `GAUNTLET.md` (B7
row); `docs/icm/retriage-2026-08-11.md`; `reference-corpus/synthesis.md`
(G5/U3/X8); `.sergeant/index.md`; the draft retirement tree under
`docs/gauntlet/runs/icm-r2/sergeant-setup/draft/.sergeant/workflows/
sergeant-setup/`.

## `BU-P5-004`/`BU-P5-005`/`BU-P7-036` (write-scope guardrail) — verdict: CONFIRMED

Independent re-derivation: `AGENTS.md` Guardrails (lines 191-195) already
states "`sgt init`/`sgt repo add`/`sgt group add` write only within the
estate they scaffold... never to another harness's own configuration"
tagged `<!-- BU-1263, BU-1264, BU-1295 -->`. This is the same governing
constraint the three cited units assert. PL-0/J5/ABSORBED is correct: the
current admitted procedure already owns this behavior, verbatim.

## `BU-P5-006` (td/Graphify/Treehouse consent-gating) — verdict: NEEDS-REVISION

Independent re-derivation: PL-1/HARVEST-to-`AGENTS.md` is the right
destination — no existing `AGENTS.md` bullet covers this (the nearest
guardrail, "a missing tool or capability surfaces as `sgt doctor`'s named
remedy," line 200, is a different rule about detection, not consent-gated
initialization). But the draft's factual claim that the unit "currently
constrains no live behavior anywhere in the corpus" overstates what was
verified. `_config/standing-constraints.md` is today's live Layer-3 file,
still cites `BU-P5-006` by name, and still states it "applies across every
stage of this workflow" — and the workflow itself is still `status:
published, version 3` in `.sergeant/index.md`, not retired. The correct,
narrower claim (which the draft's own "Already-dead in practice" section
two paragraphs earlier already establishes with better evidence) is that
nothing in `AGENTS.md`'s routing table currently sends a harness to this
workflow, not that the constraint is inert. This does not change the
disposition — the unit still needs to land in `AGENTS.md` once the package
retires — but the draft should tighten this one sentence to match the
"unreachable, not inert" framing it uses correctly elsewhere in the same
document.

## `BU-P5-012` (unsupported capability → tracked td issue) — verdict: NEEDS-REVISION

Independent re-derivation of the rung: PL-2 fits — `05-file-capability-
gaps/CONTEXT.md`'s own contract ("drafts a td issue... shows it for
explicit y/yes approval... on decline it reports the gap") is PL-2's own
listed example ("decides whether work should remain direct or become
durable Work") almost verbatim, and HARVEST into `estate-navigation` is a
better fit than leaving it a workflow stage now that nothing routes here.
Confirmed against the draft `SKILL.md`'s "Filing tracked work for a gap
`sgt doctor` can't remedy" section, which faithfully reproduces the y/yes
gate and the report-on-decline behavior.

Flagging a compliance gap the draft does not surface: `docs/icm/
convention.md` §6.1 requires "Every Captain skill's `SKILL.md` carries the
same conceptual section adapted to its driver (what it may decide, what it
must ask the user, what it must not do, its durable handoff if any)" —
this is the skill-level analog of the `## Bounded judgment` section actor
stages must carry, and it is a required section, not optional ceremony.
Neither the live `skills/estate-navigation/SKILL.md` nor the draft addendum
adds this section, despite the draft substantially expanding the skill's
decision-bearing surface with two new PL-2 judgment procedures (this unit
and the `BU-P5-024` registration fragment below). The draft's prose does
gesture at bounded judgment inline — "Don't invent values for fields that
don't exist; if a user asks for one of these, say plainly that
sergeant-rs's estate model doesn't have that field yet" is functionally a
J0 escape-to-honesty clause — but it is not structured as the required
section, so a reviewer checking `estate-navigation` against §6.1 in
isolation cannot find "what this skill must ask the user" or "what this
skill must not do" as named boundaries. This is a real, fixable gap in the
migration, not a reason to reject the destination: the fix is adding the
§6.1 section to `estate-navigation/SKILL.md` in the same change that lands
this HARVEST, covering both this unit and the interview fragment below.
This predates the draft (the live skill already lacks the section for its
existing `sgt-context`/`sgt-sync` content) but the draft makes it worse by
adding more judgment-bearing content without closing the gap, so it is
fair to raise here rather than deferring it to "someone else's problem."

## `BU-P5-023` (skip interview in favor of "Phase 5" repair) — verdict: CONFIRMED

Independent re-derivation: `30-project-interview/CONTEXT.md` cites Phase 4
vs. Phase 5 by upstream phase numbers. `CONTEXT.md`'s own "Retired" note
lists `40-repair-existing` among the stages retired at MVP-5 F2, and
nothing in the surviving two-stage tree implements a repair path. The
citation is confirmed dangling. RETIRE with the intent parked (not silently
resolved) is the right call — re-deriving "don't re-interview an
already-registered project" against `sgt repo add`'s actual idempotent
behavior is exactly the kind of judgment this docs-only pass correctly
declines to make unilaterally.

## `BU-P5-024` (strict-order project interview) — verdict: CONFIRMED

Independent re-derivation against `docs/gauntlet/contracts/MVP-1.md`
R-MVP1-3/4/5: the schema is `[estate]`/`[[repo]]`/`[group.<name>]` with
only `instructions = "local" | "suppress"` per repo — no free-text
`agent_instructions`, no per-repo role field, no project-level GitHub
identity, no Graphify path. The draft's PL-0(mostly)/PL-2(fragment) split is
correct: most of this citation targets fields that do not exist, and
RETIRE-with-park is the honest disposition for those (the schema decision
is out of this pass's authority per ADR 0013 decision 10's runtime freeze).
The transplantable fragment (name, clone origin, group membership,
iterate) is faithfully reproduced in the draft `SKILL.md`'s "Registering
repos and groups interactively" section, which also correctly declines to
invent the missing fields rather than silently dropping the requirement to
mention them. See the `BU-P5-012` entry above for the one structural gap
(missing §6.1 section) that also applies to this HARVEST fragment.

## `BU-P5-026`/`BU-P5-027` (full-file preview + timestamped backup) — verdict: CONFIRMED

Independent re-derivation: the live `estate-navigation` draft's
registration flow writes incrementally via `sgt repo add`/`sgt group add`,
never a single project-file write — there is no "whole file" moment left to
preview or back up. `AGENTS.md`'s `BU-0050` guardrail ("standing
authorization... never extends to... destroying preserved state") and
`sgt init`'s already-confirmed no-op-not-reset behavior cover the
protective intent these two units originally served. ABSORBED is correct;
there is no gap here comparable to the `BU-P5-006` sentence above, because
the draft does not claim a stronger absorption than what `AGENTS.md`
actually states.

## `BU-P8-045` (registration field validation) — verdict: CONFIRMED

Independent re-derivation: same schema-mismatch basis as `BU-P5-024` —
role, free-text instructions, and clone-URL-presence checks target fields
that either don't exist (role) or are enforced differently (URLs are
required by `sgt repo add <name> --origin <url>`'s own signature, not by a
post-hoc validation pass). RETIRE-with-park is consistent with `BU-P5-024`'s
treatment and does not silently drop the "instructions must be concrete,
not aspirational" quality bar — it correctly notes that bar is worth
re-applying only once a free-text field exists to apply it to.

## B7 backlog finding re-characterization — verdict: CONFIRMED

Independent re-derivation against `.sergeant/workflows/load-project/
20-register-or-edit/CONTEXT.md`: that stage's actual contract is
read-schema-and-existing-YAML, write, then verify via `sgt-list`/
`sgt-context`, restoring on failure — it contains no field-by-field
interview, no full-file preview, and no timestamped backup. The producer's
correction of GAUNTLET.md's "duplicates... wholesale" framing is accurate:
the literal stage content does not overlap wholesale; what genuinely
duplicates without delegation is the narrower "persist a project
definition, gated on confirmation, to the Sergeant-owned config path"
boundary. Both packages' targets being independently obsolete under the
current estate model is also independently confirmed (`load-project/
CONTEXT.md`'s own admission that it "still describes upstream's
`~/.config/sergeant/<project>.yaml` registry mechanism, which has no
sergeant-rs analog yet").

## PL-2 reclassification of `30-project-interview` (resolving G5 for this instance) — verdict: CONFIRMED

Independent re-derivation against `reference-corpus/synthesis.md` (G5,
"survives, narrowed" — a re-enterable `needs_input` stage requirement) and
`AGENTS.md`'s routing table: both of this package's live triggers
("estate isn't set up" and "repos/groups... not confirmed") already route
to `sgt init`/`sgt doctor` or `estate-navigation`, both pre-Work,
interactive-session-only paths — consistent with PL-2's own discriminator
("if the procedure's job is to decide what Work should exist, it cannot
itself require an already-existing Work merely to make that decision").
Treating the interview as live Captain-session dialogue genuinely removes
the need for a re-enterable engine capability in this specific case,
because a Captain session has no cross-turn re-entry problem to solve. The
draft is appropriately careful not to generalize this past the one
instance — correctly flagged as not a general G5 resolution.

## Package identity, naming, and Captain/workflow boundary — verdict: CONFIRMED

Independent re-derivation: `estate-navigation` is the correct existing
destination, not a new package — its own header already states the
identical "does not exist in sergeant-rs and is not re-created here"
disclaimer for the obsolete upstream project-registry model that
`sergeant-setup`'s citations describe, and its front matter/trigger list
already overlaps with both migrated behaviors' triggers ("register a new
repo/group", "file a ticket for a missing prerequisite" — draft addition).
No naming collision found. The alternative of REHOMEing the whole package
as one lump is correctly rejected against `record-shapes.md` §6 rule 4 and
proposal §8.8's file-shape-mirroring warning — the units genuinely have
different destinations (`AGENTS.md` for the write-scope/consent invariants,
`estate-navigation` for live interview content, RETIRE for the
schema-mismatched fragments).

## Retirement mechanics — verdict: CONFIRMED

Independent re-derivation: the draft retirement tree (`draft/.sergeant/
workflows/sergeant-setup/{CONTEXT.md,index.md,workflow.toml}`) correctly
sets `status: retired`, `version: 4`, empty `stages = []`, and preserves a
provenance pointer to `docs/gauntlet/promoted-provenance/sergeant-setup.md`
rather than deleting history. The claim that the live `CONTEXT.md` cites a
`provenance.md` that was never actually created is independently verified
— `.sergeant/workflows/sergeant-setup/provenance.md` does not exist.
`.sergeant/index.md`'s catalog row (`sergeant-setup | published |
workflows/sergeant-setup/index.md`) does still need updating to `retired`
at reconcile-and-publish time, as the draft itself already states.

## Duplicated/drift-prone content between `AGENTS.md` and `estate-navigation` — verdict: NEEDS-REVISION (minor, pre-existing pattern)

Independent check: the draft `SKILL.md` restates guardrail text in its own
"Guardrails this inherits from `AGENTS.md`" section rather than only citing
`AGENTS.md` by reference, which is drift-prone if the two texts diverge
later. This is not a new defect the draft introduces — the live
`skills/estate-navigation/SKILL.md` already follows this exact pattern for
its existing guardrail bullets — so it is not grounds to reject the draft's
approach. It is worth naming explicitly for the reconcile-and-publish step,
since the draft is about to add a second restated bullet (the new
`BU-P5-006` text) to an already-duplicated surface; if this pattern is
judged acceptable for the two existing bullets it can stay acceptable for
the third, but it should be a conscious choice, not an unexamined one.

## Overall verdict: SPLIT — CONFIRMED

Independent re-derivation of the package verdict from the behavior-unit
table (per `record-shapes.md` §6 rule 1 — units before verdict, not the
reverse): six units are PL-0/ABSORBED already covered by existing
`AGENTS.md` guardrails or the current incremental-write model; one unit is
PL-1/HARVEST into a new `AGENTS.md` bullet; one unit plus one fragment are
PL-2/HARVEST into `skills/estate-navigation/SKILL.md`; two units plus one
fragment are J0/RETIRE, correctly parked pending a `sergeant.toml` schema
decision this pass has no authority to make. No unit survives at PL-4/PL-5.
This produces genuinely disjoint destinations (`AGENTS.md`, `estate-
navigation`, RETIRE) rather than one target, so SPLIT is the correct
modifier over HARVEST (which would understate the AGENTS.md-bound units)
or REHOME (which the draft's own alternatives-considered section already
correctly rejects as file-shape mirroring). The "already-dead in practice"
finding — `AGENTS.md`'s routing table has no row for `sergeant-setup` and
explicitly routes both live triggers elsewhere — is independently
confirmed and materially strengthens the case for retiring the package
identity rather than merely reorganizing its stages.

**Two items to fold in before this is publish-ready**, neither of which
changes the disposition: (1) tighten the `BU-P5-006` "constrains no live
behavior" sentence to the more accurate "unreachable via routing, not
inert" framing the draft already uses elsewhere; (2) add the `docs/icm/
convention.md` §6.1 skill-level Bounded-judgment-equivalent section to
`skills/estate-navigation/SKILL.md` in the same change that lands the
`BU-P5-012`/`BU-P5-024` HARVEST, since the draft is adding new
decision-bearing content to a skill that does not yet carry the section
convention requires for it.
