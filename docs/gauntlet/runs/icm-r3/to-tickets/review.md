# Independent review: to-tickets ICM-R3 adjudication

Independent adversarial review (`reference/proposal-icm-r-procedure-
authority.md` §8.11, `docs/icm/convention.md` §6.3) of
`docs/gauntlet/runs/icm-r3/to-tickets/adjudication-draft.md`. Fresh
execution, review-only contract, no edit authority over the producer's
draft, the live package, or any destination surface. Every claim below was
independently re-derived against the actual package content
(`.sergeant/workflows/to-tickets/**`), the upstream source
(`reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md`), and
the cited destination/governing surfaces — not accepted from the
producer's own citations.

Checklist applied per §8.11: source fidelity; rung order (PL and J);
Captain/workflow boundary; stage/helper boundary; authority grants and
missing J0 cases; package identity/naming; duplicated or drift-prone
content; false pairing assumptions; unjustified engine gaps.

## Behavior-unit dispositions

### BU-P4-058 (package identity: plan/spec/etc. → tracer-bullet tickets is a distinct triggerable procedure) — verdict: CONFIRMED

Independently re-read `CONTEXT.md`, `index.md`, and `workflow.toml`. The
four-stage sequence, trigger phrase, and PL-4 package rung match. The
PL-2/PL-4 discriminator (§5.6: "a workflow may ask a bounded question
during execution, but conversation cannot be its primary product") holds
on direct inspection — `20-confirm-breakdown`'s confirmation gate
(`BU-P4-068`) is one bounded question inside a pipeline that otherwise
runs start to terminal report without deciding *whether* an already-given
artifact should become Work. J5 citation (stage order fixed by
`workflow.toml`) is a defensible reading of "governing constraint" — the
actor has no discretion to reorder or skip stages.

### BU-P4-064 (no automatic td-instruction writes to repo guidance files) — verdict: CONFIRMED

Verbatim in `00-load-project-context/CONTEXT.md`, cited to upstream
`SKILL.md` "Do not automatically add td instructions to repository
guidance files" (§1). The producer's cross-check against `BU-1311`
(same rule, dispositioned to `AGENTS.md` in
`docs/icm/agents-invariant-dispositions.md` line 206) is correct and not
duplicative — one is the always-on doctrine statement, the other is this
stage's own local restatement as it applies to td-instruction files
specifically. No drift-prone duplication found.

### Delegation to `load-project` (stale-not-yet-broken cross-package reference) — verdict: CONFIRMED

Independently re-derived: `00-load-project-context/CONTEXT.md` line 31
names **load-project**; `load-project` is still `status: published` and
still listed in `.sergeant/index.md` today (checked directly), so the
reference is not currently broken. `docs/gauntlet/runs/icm-r3/load-project/
adjudication-draft.md`'s "Cross-package consequence" section independently
names this exact dependency and commits to retargeting it to
`estate-navigation` at `load-project`'s own reconcile-and-publish step;
that draft's own independent review (`load-project/review.md`,
"Cross-package consequence" entry) confirms the citation and the
correctly-scoped remedy. The producer's rung walk (J5/J4/J3/J2, concluding
"record now, edit later") is sound: `load-project`'s ABSORBED
classification is still an unreviewed producer draft and does not qualify
as J3 settled authority (`bounded-judgment.md` §J3 excludes drafts
explicitly). Editing the delegation target now would assert a retirement
that has not been accepted.

### BU-P4-065 (investigation ticket only for a genuinely blocking unknown, named deliverable) — verdict: CONFIRMED

Verbatim in `10-extract-decisions-and-unknowns/CONTEXT.md`, matches
upstream `SKILL.md` §2 ("Create a short investigation ticket only when an
unknown cannot be answered from existing evidence. Investigation tickets
must name the decision or artifact they produce."). J2 delegation
(judging genuinely-blocking-vs-answerable) is correctly scoped — this is
exactly the kind of evidence-inspection judgment §6.5's examples describe.

### BU-P4-068 (confirm breakdown unless immediate publication was requested) — verdict: CONFIRMED

Verbatim in `20-confirm-breakdown/CONTEXT.md`, matches upstream `SKILL.md`
§4. J4 (user's explicit "publish immediately" request governs whether to
ask) + J2 (how to present for confirmation) is the correct compound
citation — the ladder's own §6.3 example set includes exactly this shape
("the user explicitly selected X" gates whether an ask is needed at all).

### BU-P4-070 (do not mark newly published tasks in_progress) — verdict: CONFIRMED

Verbatim in the "Helper invocation: publish" section, matches upstream
`SKILL.md` §5 ("Do not mark tasks `in_progress`..."). J5 governing,
unconditional — correct; this is not a decision an actor weighs, it is a
constraint on what state a ticket may claim.

### BU-P4-071 (cross-repo blockers as counterpart id + merge order, not a fabricated dependency edge) — verdict: CONFIRMED

Verbatim, matches upstream `SKILL.md` §5 ("td dependencies are
repository-local... Do not invent a native dependency edge..."). J5
governing, unconditional — correct, same reasoning as `BU-P4-070`.

### BU-P4-072 (one worker per owning repo, default concurrency) — verdict: CONFIRMED

Verbatim in `40-report-frontier/CONTEXT.md`, matches upstream `SKILL.md`
§7. J2 (what counts as "the project explicitly supports more") is a
reasonable delegated-judgment scope.

### BU-P4-073 (reporting the frontier is not authorization to dispatch) — verdict: CONFIRMED

Verbatim, matches upstream `SKILL.md` §7 ("Do not dispatch unless the user
asked to begin implementation."). J5 governing, correctly tied to proposal
invariant 4.4 ("execution is not dialogue" / no silent-trigger rule).

### BU-1297/1298/1299/1301/1302/1303/1304/1305 (ticket-quality rules — FOLD, missing from live package) — verdict: CONFIRMED

Independently re-read `20-confirm-breakdown/CONTEXT.md` in full: none of
the eight rules (vertical-slice sizing, one-fresh-context sizing,
one-owning-repository, expand-migrate-contract, epics-as-programs,
no-duplicate-tracker-entries, preserved finding IDs, observable-acceptance
readiness) appear anywhere in the stage's Behavior contract — the stage
states only that granularity/ownership/blocking edges must be *confirmed*,
never what correct granularity or a ready ticket actually look like.
Independently verified against `docs/icm/agents-invariant-dispositions.md`
lines 197-205: all nine rows (`BU-1297`-`BU-1305`) are dispositioned
`skill: to-tickets` verbatim as the producer states, and all eight
substantive rules trace cleanly to upstream `SKILL.md`'s "Principles" and
"Ticket Quality Checklist" sections. This is a real promotion/drafting
gap, not a placement error — the rung walk (J5/J4/J3/J2, landing on
J2/J3 "not J0") correctly treats `agents-invariant-dispositions.md`'s own
placement judgment as a settled authoritative record. FOLD is a defensible
use of the modifier: these units have no live text anywhere yet, so adding
them to their already-classified destination is exactly what FOLD
describes.

### BU-1300 (counterpart tickets + merge order — citation-only addition) — verdict: CONFIRMED

`BU-P4-071`'s live text already substantively covers this rule; adding
only the citation, not new prose, is correct and avoids inventing
duplicate text for an already-covered constraint.

### BU-1311 (do not auto-add tracker instructions to guidance files — confirmed not a duplicate home) — verdict: CONFIRMED

`docs/icm/agents-invariant-dispositions.md` line 206 dispositions this to
`AGENTS.md`, not `skill: to-tickets` — independently verified. The
producer's use of this row only to confirm `BU-P4-064` is not a second,
drift-prone home for the same rule is correct and appropriately scoped
(no edit to `AGENTS.md` proposed or needed).

### Authoring-format compliance — missing `## Bounded judgment` sections — verdict: CONFIRMED

All four stage `CONTEXT.md` files carry only the generic "## Judgment
required" boilerplate (verified by direct read of all four files) — no
stage names J2 decision classes, J1 local choices, or J0 escalation
triggers in the ADR 0013 shape. `docs/icm/convention.md` §6.1 and ADR
0013 decision 4 do require this "always... omission is never ambiguous."
Correctly flagged as a required in-place amendment, not a placement
defect.

### Missing `## Authority envelope` in `CONTEXT.md` (L1) — verdict: CONFIRMED

Verified directly: `to-tickets/CONTEXT.md` has no such section.
`docs/icm/convention.md` §6.1 requires one on every workflow's Layer-1
`CONTEXT.md`. Correctly flagged.

### False "no `kind = \"execute\"` stage exists" claim — verdict: CONFIRMED

`20-confirm-breakdown/CONTEXT.md`'s "Helper invocation: publish" section
contains this claim verbatim, and it is false as of this branch:
`.sergeant/workflows/repo-to-icm/workflow.toml` line 44 defines
`65-self-check` with `kind = "execute"`, independently verified. The
producer's decision to correct the claim while explicitly parking (not
resolving) whether "publish" should itself become an execute stage is the
right level of restraint — building that would be new `workflow.toml`
content outside this pass's own scope, and the placement question (is a
td-publish operation truly mechanical, or does it still require judgment
about counterpart/merge-order framing?) is not obviously settled either
way.

### `provenance.md` dangling reference — verdict: CONFIRMED

`CONTEXT.md` line 37 references `provenance.md` for the stage-to-behavior-
unit mapping; no such file exists under `.sergeant/workflows/to-tickets/`
(verified: `find` lists only `CONTEXT.md`, `index.md`, `workflow.toml`,
and the four stage directories). Leaving this as a catalog-wide,
not-this-package's-scope item is consistent with how the same class of
reference was handled in this round's `deepen-module` pass, per the
producer's own citation — reasonable scope discipline.

## Additional finding — a third candidate J0 case not surfaced

### Partial-publish-failure state during the "Helper invocation: publish" step — verdict: NEEDS-REVISION

The producer names two candidate J0 triggers (contested-blocking evidence
in `10-extract-decisions-and-unknowns`; irreducibly-cross-repo ownership in
`20-confirm-breakdown`) but the "Helper invocation: publish" text itself
(read in full, both live and upstream) has no statement of what happens
when publish partially fails — e.g., an epic and some tickets are created
in td, then a later `td create --depends-on <local-blocker-id>` call fails
(network, td error, or a blocker id that was never actually created),
leaving a partially-published, internally-inconsistent dependency graph.
Independently checked against the ladder: J5 no constraint requires or
forbids a specific recovery; J4 no user/Work decision pre-authorizes
either "roll back what was created" or "report the partial state and
stop"; J3 no settled record addresses it; J2 the stage's delegated
judgment (present and folded-helper text) covers *what* to publish and
*how* to represent cross-repo blockers, not *how to recover* from a
publish operation that succeeds partway; J1 does not apply — a
half-published epic/ticket graph is not locally reversible or
non-contractual, since downstream stages (`40-report-frontier`) and any
later dispatch depend on the graph's consistency. This resolves to **J0**
by the same reasoning the producer applied to their own two candidates,
and should be added as a third candidate alongside them in the remediation
item 3 (`## Bounded judgment` sections) the producer already recommends
building. This does not change the Final disposition — it is the same
class of in-place content gap already found, just one instance the
producer's own pass did not surface.

## Overall verdict

**Recommend STAND, same as the producer's Final disposition — the
adjudication draft is source-valid, placement-valid, and structurally
valid on independent re-derivation, but is not yet ready to be treated as
authority-valid** until the producer's own remediation items 1-4 and 6
land (item 5 correctly gated on `load-project`'s own reconcile-and-publish
step), plus the one additional J0 candidate (partial-publish-failure)
identified above is folded into remediation item 3 alongside the
producer's own two.

Nothing in this review disputes the package's PL-4/PL-5/PL-6 rung
placement, the STAND disposition, the stage/helper boundary (folded
publish helper), the Captain/workflow boundary, or any individual
behavior-unit's source fidelity. Every citation traced to its owning
source (upstream `SKILL.md`, `docs/icm/agents-invariant-dispositions.md`,
`docs/icm/convention.md`, `.sergeant/workflows/repo-to-icm/workflow.toml`)
verified correct on direct, independent inspection. The one addition above
is a missing-J0-case finding, not a rung, boundary, or fidelity dispute —
it should be treated as NEEDS-REVISION for the remediation plan's
completeness, not for the Final disposition itself.
