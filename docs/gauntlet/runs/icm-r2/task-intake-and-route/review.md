# Independent adversarial review: task-intake-and-route

Reviewer pass per `reference/proposal-icm-r-procedure-authority.md` §8.11
(independent adversarial review) and `docs/adr/0013-icm-r0-owner-rulings.md`
decision 7 (a later, fresh-execution, review-only, no-edit-authority stage
in the same workflow qualifies as independent). This review has no edit
authority over the producer's draft
(`docs/gauntlet/runs/icm-r2/task-intake-and-route/adjudication-draft.md`)
or the live package (`.sergeant/workflows/task-intake-and-route/`).

Method: every citation was independently re-resolved against
`.sergeant/workflows/task-intake-and-route/` (all six stage `CONTEXT.md`
files, `workflow.toml`, `index.md`, top-level `CONTEXT.md`), the two
delegation targets (`direct-implementation/06-pr-and-merge/CONTEXT.md`,
`dispatch/90-reconcile-fleet/CONTEXT.md`), `AGENTS.md` (current,
top-level), `skills/estate-navigation/SKILL.md`,
`.sergeant/common/contexts/bounded-judgment.md`, `.sergeant/workflows/
load-project/CONTEXT.md`, and the upstream sources
(`reference/sergeant-upstream/AGENTS.md`, `docs/what-is-sergeant.md`,
`docs/using-sergeant.md`) — not from the producer's own citations, per the
task's re-derivation requirement.

## Per-unit re-derivation

### BU-P1-026 (01-load-context) — verdict: CONFIRMED

Upstream source checked at `reference/sergeant-upstream/AGENTS.md` line
136 ("Load context — run `sgt-context <project>` and identify the owning
repository or repositories, inherited instructions, configured paths, and
cross-repository dependencies before selecting an execution mode.") —
citation resolves. Current `01-load-context/CONTEXT.md` restates this
almost verbatim as its behavior contract. Independently re-checked against
current `AGENTS.md` "Standard workflow loop" step 1 (`sgt doctor`/`sgt repo
list`, "never infer which repo or estate you're in from the current
directory") and `skills/estate-navigation/SKILL.md`, which opens by
naming step 1 as the policy it specializes ("`AGENTS.md`'s standard
workflow loop step 1 already states this as always-on policy; load this
skill when a task needs more estate-navigation detail than that one line
covers"). PL-0/ABSORBED holds: the stage's own current mechanism
(`Delegation` section) points at the legacy, unreconciled `load-project`
workflow rather than at these already-published surfaces — that is
corroborating evidence the package trails current product, not a
counter-argument, since `load-project`'s own `CONTEXT.md` self-reports as
describing "upstream's `~/.config/sergeant/<project>.yaml` registry...
which has no sergeant-rs analog yet."

### BU-P1-027 / folded `02-check-queue` (03-choose-mode helper) — verdict: CONFIRMED

Upstream source at line 137 ("Check the queue — run `sgt-td-list
<project>` and reuse a matching task in direct or dispatch mode; create a
task only when no canonical task exists.") resolves. Current `AGENTS.md`
step 2 ("Check running work," `sgt status`/`sgt work list`, "reuse or
resume a matching Work item instead of creating a duplicate for the same
intent") independently confirmed as covering the same policy under the
renamed CLI surface (`sgt-td-list` → `sgt status`/`sgt work list`). The
N1 A4 fold of `02-check-queue` into `03-choose-mode` as a helper is a
settled J3 record from a prior adjudication (`CONTEXT.md` "Notes for
reviewers"), correctly not relitigated here.

### BU-P1-028, BU-P1-003, BU-P1-108, BU-P8-053, BU-P8-054 (03-choose-mode) — verdict: CONFIRMED

All five upstream citations independently resolved:
`reference/sergeant-upstream/AGENTS.md` lines 14–20 (dispatch-mode
trigger and step 3's mode language), `docs/what-is-sergeant.md` lines
60–72 (Direct/Dispatch mode definitions, worktree/instruction/fleet-state
language), `docs/using-sergeant.md` lines 15–33 (the same direct/dispatch
criteria restated for the "choose direct or dispatch mode" procedure).
Current `AGENTS.md` "When NOT to use `sgt`" section independently
re-checked line by line against the stage's five behavior-contract
bullets: the four dispatch-mode criteria (cross-repo, two-or-more
independent repo tasks, isolated review worker, explicit worker request)
and the two-part direct-mode test (explicit in-session request AND single
owning repository) are both present, near-verbatim, with routing judgment
explicitly assigned to "the harness (this session)," matching the PL-2
discriminator (§5.4: "the interactive harness before Work admission...
decides whether work should remain direct or become durable Work"). The
draft's alternative-hypothesis test (SPLIT into a PL-2 Captain skill) is
correctly rejected on the evidence: the content this stage would populate
a Captain skill with already exists as already-published invariant text
in `AGENTS.md` itself, so there is nothing left to rehome into a new
skill file.

### BU-P1-030 (05-confirm-decisions) — verdict: CONFIRMED

Upstream line 140 resolves verbatim. Independently checked against
`.sergeant/common/contexts/bounded-judgment.md`'s J0 section: "the choice
would change scope, policy, security/privacy posture, destructive
effects, irreversible state, public behavior, acceptance, or promotion" —
the stage's own criteria list (repository ownership, user-visible
behavior, security/privacy policy, data retention, destructive action, an
irreversible tradeoff) is a workflow-local instance of the same test, not
an additional rule. ABSORBED into the canonical ladder is correct per
ADR 0013 decision 4/§7.1's no-duplication principle.

### BU-P1-029 / folded `04-reconcile-state` (05-confirm-decisions helper) — verdict: CONFIRMED

Upstream line 139 resolves. Current `AGENTS.md` step 2's "reuse or resume
a matching Work item instead of creating a duplicate" and step 6's
journal-backed reconciliation language ("a Work item isn't progressing
merely because a process for it exists; trust the journal-backed state")
jointly cover the cited policy (`sgt-watch --sync-all`, inspect active
workers/branches/worktrees/gates). No gap found.

### BU-P1-031 (06-execute) — verdict: CONFIRMED

Upstream lines 141–143 resolve. Current `AGENTS.md` steps 4–5 ("Choose a
workflow"/`sgt run "<intent>"` with envelope flags) independently
confirmed as the same fork point, and the two named destinations
(`direct-implementation`, `dispatch`) are themselves live, admitted
workflows in this corpus that already own their own launch preconditions
(verified: `direct-implementation/06-pr-and-merge` and
`dispatch/90-reconcile-fleet` both exist and carry their own behavior
contracts, checked directly, not merely cited). This stage's "moment of
admission" framing (submission itself still Captain-shaped per PL-2's own
example bullet, "turns user conversation into a bounded submission") is
independently supportable from the ladder text, not just asserted.

### BU-P1-033 (08-handle-decisions) — verdict: CONFIRMED

Upstream line 145 resolves. Current `AGENTS.md` step 7 ("Respond to
`needs_input`," "reserved for genuine human-judgment gates... not relayed
for findings a workflow could apply itself") independently confirmed as
matching language and matching scope (genuinely-missing-decisions only,
no re-asking).

### BU-P1-038 + folded BU-P1-032 (08-handle-decisions: resume preconditions + `07-monitor` helper) — verdict: CONFIRMED

Upstream lines 144 and 148 both resolve. Current `AGENTS.md` step 6
independently checked and found to carry the identity/liveness policy
verbatim ("a Work item isn't progressing merely because a process for it
exists; trust the journal-backed state these surfaces read, not liveness
alone"). The package's own "Notes for reviewers" tmux-pane translation
note (`pane` → "the durable execution or session identity this project
already journals") is independently sound: `sergeant-rs` has no tmux pane
in its architecture (headless per-turn processes, journal-backed
identity), confirmed by reading `AGENTS.md`'s own step 6 language, which
uses no pane terminology at all — the translation is not inventing a
reading, it is reporting that the destination surface already made the
same substitution independently.

### BU-P1-034 (09-reconcile-deliver) — verdict: CONFIRMED

Upstream line 146 resolves. Independently verified both delegation
targets in full: `direct-implementation/06-pr-and-merge/CONTEXT.md`
carries `BU-P1-013` (PR open, CI/review/merge-authorization gate before
declaring delivery complete) and its folded helper `BU-P1-014` (record
handoff/PR/merge/deployment/cleanup outcomes) — this is the same content
as upstream step 9's direct-mode half. `dispatch/90-reconcile-fleet/
CONTEXT.md` carries `BU-P5-070`/`BU-P5-071` (itemized per-repo
reconciliation gate list, dependency merge order, "never reconciled
merely because a PR exists") and `BU-P1-006` (reconcile merge order,
PRs, cross-repo implications) — the dispatch-mode half. Current
`AGENTS.md` step 8 ("Collect," output pointer and spend) independently
confirmed as covering the cross-mode remainder the draft claims for it.
No behavior in the upstream step-9 citation was found uncovered by this
three-way split. The disposition is correct, though see the package-level
finding below on `Alternatives considered` completeness.

## Package-level findings

### Finding 1 — J-boundary column does not cite the ladder directly (NEEDS-REVISION, cosmetic)

`docs/icm/record-shapes.md` §6 rule 2 requires the `J boundary` column to
"cite the ladders directly... not a paraphrase." Eight of the nine rows
use "N/A — invariant" rather than a J-rung citation. This is defensible in
substance — units with an `ABSORBED` disposition do not survive as a new
artifact, so there is no authority envelope left to state a J-rung
against — but the word "invariant" is a placement-ladder term (PL-1,
"Stable invariant") reused informally in the J-boundary column, which
risks being misread as a rung citation when it is not one. The one row
that does cite a J-rung correctly (`BU-P1-030`, "J0 already-canonical")
shows the more precise phrasing is available and was used inconsistently.
Recommend, before promotion: replace "N/A — invariant" with something
like "N/A — disposition is ABSORBED, no surviving artifact to state an
authority envelope for" on the other eight rows, to avoid the term
collision and satisfy the rule's literal citation requirement. This does
not change any row's disposition.

### Finding 2 — `Alternatives considered` omits an explicit REHOME rejection (NEEDS-REVISION, completeness)

`Alternatives considered` addresses STAND, SPLIT/HARVEST, FOLD, and
RETIRE, but not `REHOME` ("whole package moves to another surface") —
the disposition modifier closest in shape to the driver-classification
argument the draft itself makes (Captain-driven, not Sergeant-driven,
which is exactly the shape of a would-be REHOME-to-Captain-skill
argument). The per-unit analysis supports rejecting REHOME too (every
cited behavior already lives on an existing, already-published surface,
so there is no "another surface" left to move content to — that is
precisely what makes `ABSORBED` rather than `REHOME` the correct
modifier per §5.10's definitions), but the draft never states this
explicitly. Self-check step 9 (§8.10) requires "every rung rationale is
specific" — recommend adding one sentence to `Alternatives considered`
naming REHOME and why it does not apply, before this record is treated as
promotion-ready.

### No other findings survive challenge

Checked and not sustained as findings:

- **Source fidelity** — every one of the nine citation groups
  independently resolved against the named upstream file and line range
  (`reference/sergeant-upstream/AGENTS.md`, `docs/what-is-sergeant.md`,
  `docs/using-sergeant.md`). No citation was found to misquote or
  misattribute its source. (One citation, `BU-P1-003`'s "AGENTS.md
  L15-17," is off by one line against the actual "Use dispatch mode"
  paragraph start at line 14 — trivial, does not affect the citation's
  substance, not worth a disposition-level finding.)
- **Rung order (PL)** — PL-0 is correctly distinguished from PL-1: the
  destination surfaces the draft names (`AGENTS.md`, `bounded-judgment.md`)
  are the actual PL-1/canonical-ladder homes already; PL-0 here correctly
  classifies *this package's own mechanism* as the redundant duplicate,
  per §5.2's own destination language ("rehome any surviving policy to
  its actual owner" — the owner already exists and already holds the
  policy).
- **Rung order (J)** — see Finding 1 (format issue only, not a
  misclassification).
- **Captain/workflow boundary** — independently re-derived, not merely
  trusted from the draft's citation of proposal lines 63–69: the
  proposal's own Executive Summary names this exact package as the
  Captain-vs-workflow conflation example, before any pilot classification
  existed to bias that text. The stage-by-stage admission-boundary
  mapping (pre-work 01/03/05, at-admission 06, post-admission narration
  08/09) is independently supportable against the PL-2 discriminator's
  own example-bullet list (§5.4), not just asserted by the draft.
- **Stage/helper boundary** — the N1 A4 fold of
  `02-check-queue`/`04-reconcile-state`/`07-monitor` is a settled J3
  record from a prior adjudication cycle, correctly treated as such
  (not relitigated) rather than re-decided by this pass.
- **Authority grants and missing J0 cases** — no unit's absorption
  destination was found to drop a J0 trigger present in its source
  citation. (Separately, none of the six live stages' `CONTEXT.md` files
  carry an ADR-0013-decision-4-shaped `## Bounded judgment` section — they
  predate that ruling and use the older `## Judgment required`
  boilerplate instead. This is real, but it is evidence *for* the
  package's staleness/ABSORBED disposition, not a defect in the draft's
  argument, since the package's disposition means no such section will
  ever need to be added.)
- **Package identity/naming** — no naming conflict found; nothing to
  dispute.
- **Duplicated or drift-prone content** — this is the draft's own central
  finding (every unit duplicates already-published content) and is
  independently confirmed above unit by unit.
- **False pairing assumptions** — the `01-load-context` → `load-project`
  delegation is real but explicitly scoped out by the draft as a separate
  package's problem; that scoping is correct, not a false pairing.
- **Unjustified engine gaps** — none claimed (no PL-7 rows), consistent
  with ADR 0013 decision 10's runtime freeze.

## Overall verdict

**ABSORBED — CONFIRMED.** Every behavior-unit disposition in the
producer's table independently re-derives to the same PL-0/ABSORBED
classification against the actual current package content and its cited
destinations, not merely against the producer's own citations. The
package's central architectural claim — that this is a Captain-driven,
pre/around-Work procedure misclassified as a PL-4 Sergeant workflow — is
independently supportable from the placement ladder's own PL-2
discriminator and corroborated by the proposal's own Executive Summary,
which names this exact package as its conflation example. No unit
survives as a new Captain skill, actor skill, or shared method once
tested against current product content, because the content already
exists on already-published surfaces (`AGENTS.md`'s routing table,
standard workflow loop, and "When NOT to use `sgt`" section;
`bounded-judgment.md`'s J0 test; `direct-implementation`/`dispatch`'s own
terminal reconciliation stages).

Two NEEDS-REVISION findings (J-boundary column phrasing; missing explicit
REHOME rejection in `Alternatives considered`) are cosmetic/completeness
issues in the record's own self-check discipline, not disputes of the
classification. Recommend the producer fold both into the draft before
Captain's reconcile-and-publish pass (§8.12); neither should block
`ABSORBED` as the package's final disposition.
