# Independent review: load-project ICM-R3 adjudication

Independent adversarial review (`reference/proposal-icm-r-procedure-
authority.md` §8.11, `docs/icm/convention.md` §6.3) of
`docs/gauntlet/runs/icm-r3/load-project/adjudication-draft.md`. Fresh
execution, review-only contract, no edit authority over the producer's
draft, the live package, or any destination surface. Every claim below was
independently re-derived against the actual package content
(`.sergeant/workflows/load-project/**`) and its cited destination surfaces,
not accepted from the producer's own citations.

Checklist applied per §8.11: source fidelity; rung order (PL and J);
Captain/workflow boundary; stage/helper boundary; authority grants and
missing J0 cases; package identity/naming; duplicated or drift-prone
content; false pairing assumptions; unjustified engine gaps.

## Behavior-unit dispositions

### BU-P5-090 / BU-P5-091 / BU-P1-132 (package identity, trigger set) — verdict: CONFIRMED

Independently re-derived: `docs/gauntlet/promoted-provenance/load-project.md`
lines 12-14 verify all three citations verbatim against
`reference/sergeant-upstream/skills/load-project/SKILL.md` and upstream
`AGENTS.md` L113. `AGENTS.md`'s live routing table (checked directly,
lines 42-51) has no `load-project` row; line 48 routes the equivalent
trigger ("repos/groups/health aren't already confirmed") to
`estate-navigation`. PL-0 (absorbed) is the correct first rung to check
per `docs/icm/convention.md` §2a rule 4, and it holds: the durable intent
survives at a stronger surface, no lower rung needed to be reached.

### BU-P5-092 (unknown name → `sgt-list`, require exact registered name) — verdict: CONFIRMED

`sgt-list` does not exist in sergeant-rs's CLI surface (verified: not in
`src/cli.rs`'s command set, not referenced anywhere outside
`reference/sergeant-upstream` and this package). No "project name" concept
exists in `sergeant.toml` (`[estate]`/`[[repo]]`/`[group.<name>]`, per
`docs/gauntlet/contracts/MVP-1.md` R-MVP1-3, directly re-checked). RETIRE
is correct.

### BU-P5-108 (unregistered project → stop and ask) — verdict: CONFIRMED

`skills/estate-navigation/SKILL.md` "Registering repos and groups
interactively" (lines 81-96, read in full) is a genuine live analog,
gated per-repository/group rather than per-project, matching the
producer's characterization exactly.

### BU-P5-093 (`sgt-context`: repos, paths, clone state, roles/groups, instructions, Graphify) — verdict: CONFIRMED

`skills/estate-navigation/SKILL.md`'s own header (lines 1-14, read in
full) states in its own words that `sgt-context`'s upstream registry
mechanism "does not exist in sergeant-rs and is not re-created here."
`docs/gauntlet/contracts/MVP-1.md` R-MVP1-4 (independently re-read, lines
102-124) confirms instruction resolution is pinned by `sgt` itself at
Work bind time from `[[repo]] instructions = "local" | "suppress"` — a
stronger rung than a wrapping workflow stage doing runtime composition, as
the producer claims. This is not a case of the producer inflating "moved
elsewhere" into "moved to a stronger place" — R-MVP1-4's text literally
supports "resolved and pinned by sgt itself," independently verified.

### BU-P5-094 (raw YAML fallback only when `sgt-context` omits a field) — verdict: CONFIRMED

No resolved-view-vs-raw-source duality exists in the current schema
(`sergeant.toml` is read directly, one file, no synthesized intermediate
view to prefer over it). RETIRE with destination "none" is accurate — this
is a real case where no analog exists anywhere, not merely a relocated
intent.

### BU-P5-096 / BU-P6-021 (three-state clone-status completion evidence) — verdict: CONFIRMED

`sgt-context`'s specific three-state output shape is a binary
implementation detail with no workflow-content analog to inherit; `sgt
repo list`/`sgt doctor` are Rust-owned CLI output, not workflow-content
surfaces the reconciliation pass could rewrite. Correctly scoped as
RETIRE rather than a false ABSORBED claim onto CLI internals this pass
does not own.

### BU-P6-022 (Graphify build-status passthrough) — verdict: CONFIRMED

Independently checked: `project-graph` is cited as a separate,
not-yet-adjudicated N1 candidate
(`docs/gauntlet/promoted-provenance/project-graph.md` exists as a distinct
file). Correctly scoped out rather than invented a destination that does
not yet exist — this is the right discipline for §8.11's "unjustified
engine gaps / false pairing assumptions" check: the producer did not
force a pairing with a package that has not itself been adjudicated.

### BU-P5-111 (`sgt-context`/raw-YAML disagreement blocking) — verdict: CONFIRMED

Depends entirely on the `sgt-context`/raw-source pair established by
BU-P5-093/094, both confirmed retired above. No independent issue.

### BU-P5-097/098/099/101/103 (schema-read, no-secrets, path rules, post-write verify, restore-on-failure) — verdict: NEEDS-REVISION

The registry-write mechanism (RETIRE) is correctly re-derived — no
`~/.config/sergeant/<project>.yaml` target exists. The producer's
secondary claim, that the *protective intents* survive at a stronger rung,
is only partly borne out on independent re-check:

- No-secrets-in-config: confirmed at `AGENTS.md` line 220 ("Secrets never
  enter a commit, a project/estate config file, or workflow..."), a
  genuine J5 governing constraint, correctly cited.
- "Gated, individually-reversible, no monolithic preview-and-confirm
  ceremony": confirmed at `estate-navigation` lines 81-96 for the
  *registration* half (add a repo/group).
- Post-write verification and restore-on-failure (BU-P5-101, BU-P5-103):
  **not actually re-homed.** Independent read of
  `skills/estate-navigation/SKILL.md` in full finds no post-registration
  verification step analogous to "run sgt-list, require exactly one match,
  run sgt-context, require every edited field to appear" and no
  restore-prior-file-on-failed-verification behavior. The draft's own
  prose papers over this by attributing it to "`sgt repo add`'s idempotent,
  refusal-on-conflict mechanics," but idempotent-refusal-on-conflict is a
  *pre-write* guard (declining to re-clone or overwrite), not a *post-write
  verify-then-restore* behavior — a materially different protective shape.
  This is the same class of gap the producer itself flags precisely for
  `sergeant-setup`'s B7 (citing "load-project never backs up before
  overwrite" as a real, narrower-than-interview distinction) but does not
  apply the same scrutiny to its own claim that BU-P5-101/103 land
  somewhere. The correct disposition for these two units specifically is
  plain **RETIRE** (no analog), not "RETIRE/ABSORBED" bundled with the
  other three — bundling a confirmed absorption (no-secrets, gated-write)
  with an unconfirmed one (verify-then-restore) in one table row
  overstates what survives.

### BU-P5-095/102/109/110, BU-P6-013/014 (folded sync helper) — verdict: CONFIRMED

Independently re-checked against `docs/icm/re-homing-record-2026-08-12.md`
(the "SPLIT verdicts executed" table, read in full): this re-homing was
executed at MVP-5 F2 on 2026-08-12, prior to and independent of this pass.
`sgt repo add <name> --origin <url>` and the honestly-named "no bulk pull"
gap in `estate-navigation` match the record. STAND (no further action) is
correct — this row documents prior work, not a new classification by this
producer.

### BU-P6-012/035 (folded report-state helper) — verdict: CONFIRMED

Same re-homing record, same conclusion: `sgt repo list` + `sgt doctor`
already ship this. STAND is correct.

### `CONTEXT.md` Provenance dangling-pointer note — verdict: CONFIRMED

Independently verified: `CONTEXT.md` line 54 states `provenance.md` "was
never actually created for load-project," redirecting to
`docs/gauntlet/promoted-provenance/load-project.md`, which does exist and
was read in full for this review. Moot-under-RETIRE framing is correct;
this is an authoring-hygiene note, not a placement question, and the
producer is right not to force it into the PL/J columns.

## Package-level findings beyond the per-unit table

### Rung order — J-column misuse across the RETIRE rows — verdict: NEEDS-REVISION

`record-shapes.md` §6 rule 2 states the J boundary documents "what J5
constraints bind [a unit], which explicit J4 decisions it consumes... what
must land at J0" for **a unit surviving in a skill, workflow, or stage**.
For every unit in this table dispositioned pure RETIRE (BU-P5-092,
BU-P5-108, BU-P5-094, BU-P5-096/BU-P6-021, BU-P6-022, BU-P5-111), no actor
survives to exercise any bounded judgment at all — there is no stage, no
skill, no decision left to bound. Citing "J0" for these rows is a category
error: J0 in the Bounded-Judgment Ladder means "an actor faced a decision
no higher rung resolved and must stop and ask" (§6.7 of the proposal,
canonical shape requires a **Question** field). No question is being
asked here, and no actor is deciding anything — the registry concept
simply does not exist. The producer appears to be using "J0" as a stand-in
for "this placement question itself was not resolved by inspecting a
higher-rung source," which is a PL-column concern (does PL-0 apply?), not
a J-column concern (what may an actor decide?). The correct entry for
these rows is `N/A` or a dash, not a J-rung citation that implies an
actor-facing needs_input condition that will never actually fire because
the package is retiring. This does not change any disposition, but it is
a real rung-order defect per §8.11's own checklist item, and record-
shapes.md's own rule ("the same discipline this repo's classification-
record rule already requires... applied now at package granularity too")
means the table should not carry a citation that misapplies the ladder it
claims to cite.

### Disposition-column / destination-text inconsistency — verdict: NEEDS-REVISION

Several rows disposition as pure `RETIRE` with destination `none`, while
their own prose in the same cell names a live absorbing analog:
BU-P5-090/091/BU-P1-132 ("already ABSORBED at `skills/estate-navigation/
SKILL.md` header and `AGENTS.md`'s live routing-table row 48" — that is
literally the definition of ABSORBED, §5.10 of the proposal, not RETIRE);
BU-P5-108 ("`estate-navigation`'s ... section is the current analog");
BU-P5-096/BU-P6-021 ("`sgt repo list`/`sgt doctor` ... are the current
completion-evidence surfaces"). Per §5.10, RETIRE means "no surviving
behavior remains after normalization" and ABSORBED means "existing product
surface already owns the behavior" — these are not the same claim, and
the table asserts both in the same row without reconciling which one is
true. This matters because record-shapes.md §6 rule 1 requires the
package's Final disposition to be *synthesized from* the individual
rows, not decided first and back-filled; a reader auditing the Final
disposition of `ABSORBED` against a table where a majority of rows say
`RETIRE` cannot verify the synthesis without first resolving this
row-level contradiction themselves. The producer should either (a)
relabel the rows above as `ABSORBED` to match their own destination
prose, or (b) if `RETIRE` is intended literally (no surviving behavior at
all, full stop), remove the destination text that describes a surviving
analog. As written, the table is internally inconsistent about its own
central claim.

### Final disposition — verdict: DISPUTED (recommend ABSORBED, but corrected)

The producer's Final disposition is **ABSORBED**. Independent re-derivation
agrees with the underlying conclusion — no code, workflow, or skill needs
new content, and the intents load-project served are independently already
served by `estate-navigation`, `AGENTS.md`, and `sgt`'s own CLI mechanics —
but disputes how the table gets there. As tabulated, 6 of 11 substantive
rows are labeled `RETIRE` (not `ABSORBED`), 2 are labeled `RETIRE /
ABSORBED` combined (one of which, per the finding above, bundles a
confirmed and an unconfirmed absorption), and 2 are `STAND` describing
prior work. A package-level `ABSORBED` verdict synthesized from a table
that is majority-`RETIRE`-by-label is not auditable on its face, even
though the underlying facts (independently re-checked above) do support
calling it ABSORBED once the row labels are corrected to match their own
destination prose. Recommend: relabel the rows per the finding above, then
ABSORBED is the right Final disposition — this review does not find a
different correct verdict, only an unreconciled table that currently
doesn't prove it. This is not a reason to hold the retirement itself; it
is a reason the producer's draft needs one more revision pass before
independent review can certify the record as internally consistent.

### Captain/workflow boundary — verdict: CONFIRMED

`estate-navigation` is itself independently classified PL-2 (Captain
skill) in its own header ("Both new sections below are live, Captain-
session judgment (PL-2): they decide whether Work should exist"). Moving
load-project's registration intent there, rather than to another
workflow, is consistent with the PL-2 discriminator (§5.4 of the
proposal): registering a repo/group is pre-Work-admission judgment, not a
durable checkpoint inside an already-admitted Work. No boundary violation
found.

### Cross-package consequence (`to-tickets/00-load-project-context`) — verdict: CONFIRMED

Independently re-derived via direct grep across `.sergeant/workflows/`,
`skills/`, `AGENTS.md`, `docs/DEVELOPMENT.md`, `docs/icm/` for the literal
token `load-project`: the only other live delegator is
`to-tickets/00-load-project-context/CONTEXT.md` line 31 ("This stage's
outcome is produced by running **load-project** to its own completion")
and `to-tickets/CONTEXT.md` line 29. `cross-repo-work/CONTEXT.md` was
independently checked and contains no reference, matching the producer's
note about an early false-positive grep. The follow-on correctness claim
(retiring `load-project` requires fixing this delegation at
reconcile-and-publish, naming `estate-navigation` instead) is sound and
correctly scoped as belonging to that later step, not this producer pass.

### Out-of-scope claim about `docs/icm/retriage-2026-08-11.md`'s prior verdict — verdict: CONFIRMED

The producer's claim that the 2026-08-11 "stays WORKFLOW" verdict is
superseded, not merely disagreed with, is independently supported: that
retriage predates `estate-navigation`'s current content (which itself
states it was "Extended at ICM-R2," after 2026-08-11) and predates the
owner pre-ruling recorded in `estate-navigation`'s own header. The
evidence-hierarchy citation (proposal §2.5) is applied correctly — current
measured product state outranks a pre-ladder retriage note.

## Overall verdict

**Recommend ABSORBED, same as the producer's Final disposition, but the
draft is NEEDS-REVISION before it can be certified as internally
consistent.** Every individual re-derivation of "does the current product
already own this" was independently confirmed against primary sources
(estate-navigation's own file, `AGENTS.md`, `docs/gauntlet/contracts/
MVP-1.md`, `docs/icm/re-homing-record-2026-08-12.md`) rather than accepted
from the producer's citations. No hidden STAND or SPLIT case was found —
the package genuinely has no surviving content to place. Two defects
require a revision pass before reconcile-and-publish:

1. Two units (BU-P5-101, BU-P5-103 — post-write verify and restore-on-
   failure) are dispositioned RETIRE/ABSORBED but their absorption claim
   does not survive independent re-check; they should split out as plain
   RETIRE, separate from BU-P5-097/098/099 in the same row, which are
   genuinely absorbed.
2. The J-column citations on every pure-RETIRE row misapply the Bounded-
   Judgment Ladder to units where no actor decision survives; they should
   read `N/A`, not `J0`. This is cosmetic to the outcome but is exactly
   the "rung order" defect class §8.11 exists to catch, and record-
   shapes.md's own citation-discipline rule applies at package granularity
   here.
3. The RETIRE-labeled rows whose destination prose actually describes an
   absorbing analog (BU-P5-090/091/BU-P1-132, BU-P5-108, BU-P5-096/
   BU-P6-021) should be relabeled ABSORBED to match their own stated
   destination, so the Final disposition is traceable from the table
   rather than asserted past it.

None of these three findings changes the retirement conclusion or the
recommended ABSORBED Final disposition; they are findings about whether
the record, as currently drafted, actually proves what it concludes.
