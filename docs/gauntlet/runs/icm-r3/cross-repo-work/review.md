# Independent adversarial review: cross-repo-work

ICM-R3 reviewer pass, `reference/proposal-icm-r-procedure-authority.md`
§8.11 (source fidelity; rung order; Captain/workflow boundary;
stage/helper boundary; authority grants and missing J0 cases; package
identity/naming; duplicated or drift-prone content; false pairing
assumptions; unjustified engine gaps), applied against the producer's
draft at `docs/gauntlet/runs/icm-r3/cross-repo-work/adjudication-draft.md`
and independently re-checked against the live package content at
`.sergeant/workflows/cross-repo-work/` (five stage `CONTEXT.md`/`output/
README.md` pairs, `CONTEXT.md`, `index.md`, `workflow.toml`),
`.sergeant/index.md`'s catalog, `docs/icm/re-homing-record-2026-08-12.md`,
`docs/icm/convention.md` §§2a, 4, 6, and `reference-corpus/
engine-pressure.md`'s G6 entry. This reviewer wrote nothing to the live
package; findings below are recorded for reconcile-and-publish, per
`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7 and `convention.md`
§6.3 (independent review lives in the execution boundary — fresh read, no
edit authority over the subject reviewed).

### BU-CRW-01 -- verdict: CONFIRMED

`10-assign-ownership/CONTEXT.md` re-read directly: fixed per-repo record
shape (repo/role/deliverable/acceptance), ambiguity resolved from the
project graph first with the user asked only for genuinely contested
ownership. PL-5/J2-with-narrow-J0-carve-out is independently re-derived
and matches the live text exactly (`BU-P5-041`/`042`/`043`, lines 21-29).

### BU-CRW-02 -- verdict: CONFIRMED

`20-define-dependency-order/CONTEXT.md` re-read directly: the evidence
vocabulary for edges and the "cycle never reaches dispatch" fixed rule
match the draft's J2/J5 split exactly (`BU-P5-044`/`045`/`046`, lines
21-29). The Alternatives-considered rejection of PL-6 for the cycle logic
is independently sound — "how to break a genuine cycle" is judgment, only
"never let one reach dispatch" is mechanical.

### BU-CRW-03 -- verdict: CONFIRMED

The fold of `30-inspect-repository-state` into `40-define-delivery-gates`
as a preceding helper invocation is independently re-derived against
§6.3's reimplementation test: swapping the status-inspection command
leaves the checkpoint (a read-only repo-state record) unchanged. Matches
N1 adjudication A4 and the live `40-define-delivery-gates/CONTEXT.md`
"Helper invocations" section (lines 32-41) verbatim. PL-6/J5
(strictly-read-only) is correct.

### BU-CRW-04 -- verdict: DISPUTED

The disposition (PL-5, STAND) is confirmed. The **J-boundary is
incomplete**, and so is the resulting remediation plan.

The stage's own behavior contract (`BU-P5-049`, `40-define-delivery-gates/
CONTEXT.md` line 21, read directly) requires each delivery gate to record
"any already-approved or still-missing data/security/destructive
decisions." A **still-missing** security or destructive decision is
exactly the class of fact the Bounded-Judgment Ladder puts at J0: §6.7
names "security/privacy posture, destructive effects" explicitly as
risk-changing, and `convention.md` §6.1 requires every actor stage's
eventual `## Bounded judgment` section to state "what must become
`needs_input` at J0." The draft's J-boundary column for this unit states
only "J2 (delegated: defining each repository's concrete gate content...)
with J5 (governing: the plan's completion condition... is fixed)" — it
never names the missing-security/destructive-decision case as a J0
trigger, and the "Surviving package design" remediation list (item 1)
explicitly names a J0 clause for `10-assign-ownership` and a J5 clause for
`20-define-dependency-order` but is silent on any J0 clause for
`40-define-delivery-gates`, even though this stage's own cited behavior
contract is the one unit in the package that names security/destructive
decisions by name.

**Required correction:** add "a required data/security/destructive
decision for a repository's gate is still missing or unresolved" as an
explicit J0 clause to `40-define-delivery-gates`'s future `## Bounded
judgment` section, and add it to remediation item 1's list alongside the
already-named `10-assign-ownership`/`20-define-dependency-order` clauses.
This does not change PL-5/STAND for this unit — it is a completeness gap
in the authority-grants finding the draft's own `BU-CRW-08`/`09` already
correctly opened, not a placement or disposition dispute.

### BU-CRW-05 -- verdict: CONFIRMED

The `dispatch` delegation citation (`50-handoff-or-stop/CONTEXT.md`
"Delegation", line 34) is independently re-verified against the live
`.sergeant/workflows/` listing and `dispatch`'s own ICM-R3 verdict
(`docs/gauntlet/runs/icm-r3/dispatch/review.md`: "STAND is confirmed...
The draft's claim that this record's STAND verdict is load-bearing for
[the] wave-2 package is accurate. No false-pairing issue found on
`dispatch`'s side of that citation.") `dispatch` is live, unchanged, and
the "context composition, not nested-workflow invocation" framing matches
`convention.md` §4 rule 1 exactly (read directly). PL-5 with J5-governing/
J2-residual is correct.

### BU-CRW-06 -- verdict: CONFIRMED

Re-deriving `60-reconcile`'s checkpoint independent of the dangling
`reconcile-and-cleanup-fleet` characterization: reconciling PR/CI/thread/
merge/deploy state and terminal status strictly for the repos this plan
named is real, self-contained judgment (`BU-P5-052`/`053`, A8 scope note,
read directly in `60-reconcile/CONTEXT.md` lines 17-28) that does not
depend on the disputed reference. PL-5/STAND survives independently, as
the draft argues. The J-boundary (J5 completeness-fact requirement + J2
residual for which fact applies where) matches the live text.

### BU-CRW-07 -- verdict: NEEDS-REVISION

The core finding — `reconcile-and-cleanup-fleet` does not exist as a live
package or draft, and the `CONTEXT.md`/`60-reconcile/CONTEXT.md`
characterization of it as an "adjacent, owned procedure" is inaccurate —
is independently confirmed: it is absent from `.sergeant/workflows/`,
absent from `.sergeant/index.md`'s catalog, and `docs/icm/
re-homing-record-2026-08-12.md` line 25 (read directly) confirms its
multi-repo cleanup half is doctrinally foreclosed ("NOT-EVER per North
Star's 'fleet as a domain object' line"). FOLD (in-place prose correction,
no placement change) is the correct disposition.

**Source-fidelity defect (§8.11):** the draft's validation evidence
states the absence check was "confirmed against the current **17-entry**
directory listing" (`adjudication-draft.md` line 161-162, and repeated at
line 378). The live `.sergeant/workflows/` directory has **20** entries
(`ls -1 .sergeant/workflows/ | wc -l`, verified directly), matching the
sibling `dispatch` review's own independently-counted "20-package
catalog" from the same reconciliation wave. This is the same class of
uncorrected-stale-count error the wave already exists to catch — it does
not change the conclusion (`reconcile-and-cleanup-fleet` is absent from
either count), but a source-fidelity claim in an adjudication record must
be accurate, not merely directionally right. **Required correction:**
change "17-entry" to "20-entry" (or drop the specific count and just cite
the catalog check, matching `dispatch`'s own review's phrasing) in both
occurrences before this record is treated as settled.

### BU-CRW-08 -- verdict: CONFIRMED

Independently re-read all five stage `CONTEXT.md` files directly: every
one carries the uniform `## Judgment required` boilerplate paragraph and
none carries a `## Bounded judgment` section in the shape `convention.md`
§6.1 requires (J2 delegations by name, J1 local choices, J0 escalation
triggers, completion boundary, decision-recording location). The draft's
"uniform... unlike `dispatch`'s reviewed `BU-DISP-13`" claim is accurate
as written — no stage in this package is an exception. J5-governing
citation to ADR 0013 decision 4 / `convention.md` §6.1 is correct.

### BU-CRW-09 -- verdict: CONFIRMED

`CONTEXT.md` (L1, read directly) has no `## Authority envelope` section —
confirmed by direct heading scan of the file. `convention.md` §6.1's
requirement ("Every workflow's Layer-1 `CONTEXT.md` carries an
`## Authority envelope` section") applies and is unmet. J5/STAND-with-
amendment is correct.

## Additional checks

- **Package identity and naming:** `cross-repo-work` collides with no
  other name in the 20-package catalog or `skills/` root (checked
  directly). No rehome trigger from naming.
- **Duplicated/drift-prone content:** the parenthetical distinguishing
  "context composition today" from "true nested-workflow invocation,
  which does not exist yet" (`50-handoff-or-stop/CONTEXT.md` line 34,
  `60-reconcile/CONTEXT.md` line 11) is independently checked against
  `convention.md` §4 rule 1 and `reference-corpus/engine-pressure.md`'s
  G6 entry (lines 419-440, read directly) and found accurate and
  narrowly scoped — it is not the same stale claim `dispatch`'s own
  ICM-R3 review found and corrected (`BU-DISP-03`'s "no `kind =
  \"execute\"` stage exists in the current engine" is a blanket claim
  about execute stages; this package's claim is specifically about a
  child-workflow-invocation primitive, which still does not exist even
  though `repo-to-icm/65-self-check` is a live `execute` stage). No
  correction needed here; the draft's own reasoning on this point
  (distinguishing the two claims) is independently confirmed sound.
- **False pairing assumptions:** the `dispatch`/`reconcile-and-cleanup
  -fleet` pairing is correctly split — `dispatch` accurate,
  `reconcile-and-cleanup-fleet` not. No further false pairing found
  elsewhere in the package's five stage files.
- **Unjustified engine gaps:** none. The package files no new engine-gap
  claim; it reuses the existing G6 entry as evidence, which is the
  correct move (`convention.md` §4 rule 1 forbids smuggling a
  child-workflow invocation through `@@name` or prose; G6 already
  records the underlying pressure).
- **Structural check:** `workflow.toml`'s declared stage order
  (`10-assign-ownership`, `20-define-dependency-order`,
  `40-define-delivery-gates`, `50-handoff-or-stop`, `60-reconcile`) agrees
  with the directory listing and every stage's Inputs table names exactly
  the immediately preceding stage's `output/README.md` with no forward
  reference (verified directly against all five `CONTEXT.md` files) —
  matches the draft's structural-validity claim.

## Overall verdict on Final disposition

**STAND is confirmed** — no unit's independent re-derivation produces a
different placement rung or a different package-level disposition.
`10`/`20`/`40`/`50`/`60` remain PL-5 actor stages, the folded
`30-inspect-repository-state` remains PL-6, and no new REHOME/SPLIT/
HARVEST/ABSORBED trigger was found anywhere in the package.

Two items need correction before this record is treated as settled,
neither of which changes STAND or any unit's PL rung:

1. **BU-CRW-04** — the remediation plan for `40-define-delivery-gates`'s
   future `## Bounded judgment` section omits an explicit J0 clause for a
   still-missing data/security/destructive decision, even though the
   stage's own cited behavior contract names that category directly. Add
   it to remediation item 1's list.
2. **BU-CRW-07** — the "17-entry directory listing" citation is wrong;
   the live catalog has 20 entries (confirmed directly and by the sibling
   `dispatch` review's independent count in the same wave). Correct the
   count in both occurrences.

The draft's "Authority-valid: not yet" self-assessment already correctly
anticipates that this package is not yet ready to publish pending the
`## Bounded judgment`/`## Authority envelope` amendments; this review adds
one more line item (the J0 clause above) to that already-open remediation
list rather than opening a new authority gap. Per
`reference/proposal-icm-r-procedure-authority.md` §8.12, both findings
above should be merged into the producer's record before it is treated as
settled.
