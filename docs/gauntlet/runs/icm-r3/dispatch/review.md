# Independent adversarial review: dispatch

ICM-R3 reviewer pass, `reference/proposal-icm-r-procedure-authority.md`
§8.11 (source fidelity; rung order; Captain/workflow boundary;
stage/helper boundary; authority grants and missing J0 cases; package
identity/naming; duplicated or drift-prone content; false pairing
assumptions; unjustified engine gaps), applied against the producer's
draft at `docs/gauntlet/runs/icm-r3/dispatch/adjudication-draft.md` and
independently re-checked against the live package content at
`.sergeant/workflows/dispatch/` (six stage `CONTEXT.md`/`output/README.md`
pairs, `CONTEXT.md`, `index.md`, `workflow.toml`), `.sergeant/index.md`'s
20-package catalog, `docs/icm/re-homing-record-2026-08-12.md`,
`docs/icm/convention.md` §6, and `.sergeant/workflows/repo-to-icm/
workflow.toml`. This reviewer wrote nothing to the live package; findings
below are recorded for reconcile-and-publish, per `docs/adr/
0013-icm-r0-owner-rulings.md` decisions 6-7 and `convention.md` §6.3
(independent review lives in the execution boundary — fresh read, no edit
authority over the subject reviewed).

### BU-DISP-01 -- verdict: CONFIRMED

`00-check-queue-and-plan/CONTEXT.md` matches the disposition exactly: task
-vs-brief mode determination and plan confirmation, PL-5/J2, no dangling
reference, `## Judgment required` boilerplate present (not yet `##
Bounded judgment` — covered by BU-DISP-13 below). Independently re-read;
no discrepancy from the package content.

### BU-DISP-02 -- verdict: CONFIRMED

`05-classify-risk/CONTEXT.md` re-read directly: the fixed keyword set
(auth, security, secrets, payments, databases, migrations, production,
destructive, persistent state, state transitions) is stated as a hard gate
onto `--intent-file`, matching the draft's J5-governing/J2-residual split.
The "Alternatives considered" rejection of PL-6 (deterministic machinery)
is independently sound: applying a keyword list to free objective text
that may imply an unlisted-verbatim risk category (the draft's own
"stateful" example) is judgment, not mechanical matching.

### BU-DISP-03 -- verdict: NEEDS-REVISION

The disposition itself (fold into `15-check-admission` as a helper,
PL-6/J5) is independently re-confirmed: swapping the harness/model-tuple/
identity probe implementations leaves the admission checkpoint unchanged,
satisfying §6.3's reimplementation test exactly as the package's own
"Additional note" already concedes.

What the producer's draft missed: `15-check-admission/CONTEXT.md`'s
"Deterministic-machinery candidate" section (line 134, read directly)
states, as part of *why* this remains an ordinary actor stage rather than
a `kind = "execute"` stage: **"No `kind = \"execute\"` stage exists in the
current engine (N1 is content-only; the governing proposal's Phase B is
not adopted at this milestone)."** This is false as of this branch.
`.sergeant/workflows/repo-to-icm/workflow.toml` (read directly, lines
7, 26-33, 44) shows `65-self-check` is a live `kind = "execute"` stage,
added at the MVP-2/N4 pass on 2026-08-12 — four days before this ICM-R3
pass. This is not a hypothetical concern: it is the **identical false
claim, in the identical words**, that this same ICM-R3 reconciliation wave
already found and corrected in the `research` package's own
`00-investigate/CONTEXT.md` ("Rung-rationale correction (ICM-R2 pilot
review, 2026-08-16): the prior text here claimed 'no `kind = \"execute\"`
stage exists in the current engine' as part of this fold's justification.
That is false as of this branch..."). The `dispatch` package carries the
same stale premise, uncorrected, and the producer's draft — despite citing
`research`'s `B9` finding by name for the drain-fleet/respond-to-worker
check — did not extend the same scrutiny to this sibling claim sitting in
the same stage file it was independently re-deriving.

This is exactly the "duplicated or drift-prone content" class this
reviewer pass is asked to check (§8.11): the same boilerplate claim was
copied into multiple packages during the same extraction era and only one
copy (`research`) has been corrected so far.

**Required correction, mirroring `research`'s own fix:** `15-check-
admission/CONTEXT.md`'s "Deterministic-machinery candidate" section should
state that `repo-to-icm`'s `65-self-check` is a live `kind = "execute"`
stage, and that whether this checkpoint's harness/model/identity probes
should become a mechanical `execute`-stage check (riding after this
actor stage, analogous to `65-self-check`'s validator) rather than trusted
to this stage's own self-check is an open question this pass raises but
does not resolve — parked as a follow-on finding, not silently re-asserted
as settled (same shape as `research`'s own parked follow-on). This is an
addition to "Surviving package design"'s in-place-amendment list, not a
placement or disposition change: PL-6/J5/FOLD-into-helper for this unit is
unaffected.

### BU-DISP-04 -- verdict: DISPUTED

The disposition (STAND, PL-5, with the required drain-fleet correction) is
confirmed. The **J-rung classification is internally inconsistent**
between the main table and the draft's own full record.

The table row states J2 ("delegated: this is the workflow's real
judgment-bearing checkpoint — deciding the lock's narrow hold window
against the specific side effect it protects"). But the behavior contract
this cites (`BU-P6-128`, read directly in `15-check-admission/CONTEXT.md`)
does not leave the hold window to the acting stage's discretion: "the
admission (drain) lock is held only through that first side effect and
released immediately afterward" is a fixed rule, not a class of decision
the stage chooses among alternatives for. The draft's own "drain-fleet"
full record effectively re-derives this as a governing constraint, not a
delegation — "the lock-then-release rule is this stage's own contract"
and "narrow-lock-then-release is a choice this stage makes about its own
execution, not a delegation to another procedure" reads as J5 (a fixed
protocol the stage must follow), with at most J1 residual for exactly
*how* the release is mechanically triggered. J2 requires "the active skill
or stage explicitly delegates this class of decision within named bounds"
(§6.5) — but there is no named class of decision being delegated here; the
window is fixed, not chosen.

Recommendation: reclassify BU-DISP-04's J boundary to J5 (governing: the
lock is held across exactly one side effect and released immediately,
`BU-P6-128`) with J1 residual for local sequencing, not J2. This does not
change PL-5, STAND, or the required drain-fleet correction — only the
J-column of the table and the "Rungs checked" note in the drain-fleet full
record, which should read J5 as governing rather than "J2 — this producer
is not authorized..." (that J2 disclaimer is about *rewriting the
package*, a different question, and should not be conflated with the
checkpoint's own J-rung).

### BU-DISP-05 -- verdict: CONFIRMED

`20-prepare-intent/CONTEXT.md` re-read directly: one canonical
`.sergeant-intent.md` revision written identically to fleet state and
every worktree, treated as canonical downstream. J5-governing (correctness
invariant) with J1 for mechanical write order is the right split — no
actor discretion changes *which* revision is canonical, only the order
operations touch already-decided content.

### BU-DISP-06 -- verdict: CONFIRMED

`80-monitor/CONTEXT.md` Helper item 1 (bulk fleet reconciliation) re-read:
grace-period bound and the never-sweep needs_input/blocked/orphaned rule
are stated as fixed, matching J5/PL-6. Independently re-checked against
§6.3: swapping the reconciliation implementation leaves the fold's
governing invariants (bounded grace period, protected states) unchanged —
correctly a helper, not a stage.

### BU-DISP-07 -- verdict: CONFIRMED

Helper item 2 (all-or-nothing tracked-work creation, rollback on failure)
re-read; matches PL-6/J5 as stated. The ordering note ("Order preserves
N1 adjudication A3 (BH-01): fleet reconciliation runs before tracked-work
creation") is independently verified against `80-monitor/CONTEXT.md`'s own
"Helper invocations" numbering (reconcile fleet listed as item 1, create
tracked work as item 2) — the fix from A3 is actually present in the
current file, not merely claimed.

### BU-DISP-08 -- verdict: CONFIRMED

Helper item 3 (isolated work surface, treehouse-pool preference,
unpushed-work refusal with non-destructive `--adopt-branch` escape)
re-read; the safety behavior (refusal message must never suggest deleting
the branch) is stated as fixed in the package content, matching J5/PL-6.

### BU-DISP-09 -- verdict: CONFIRMED

Helper item 4 (brief rendering, defaults→group→repo merge order, explicit
override always wins) re-read and matches. The draft's characterization of
the BU-P5-075..089/150-153 range as "authored into the brief but not
executed by this stage" is independently consistent with `80-monitor/
CONTEXT.md`'s own text ("input to `worker-mission` and
`route-review-findings`, not a claim that this stage performs that
content's behavior") — correctly scoped, not an overclaim.

### BU-DISP-10 -- verdict: CONFIRMED

Helper item 5 (`intended`→`confirmed` launch evidence, per-repo orphaned
failure before loop abort) re-read and matches PL-6/J5 as stated.

### BU-DISP-11 -- verdict: CONFIRMED

`80-monitor/CONTEXT.md`'s escalation-handling text (read full escalation,
obtain explicit human decision without inferring consequential intent,
deliver to the exact task/repo pair) matches PL-5/J2-with-J0-carve-out.
The respond-to-worker correction is independently re-verified below
(same finding as the producer's, confirmed against primary sources). The
J0 carve-out for "without inference" is real: the stage's own text uses
that exact phrase as a hard prohibition, which is the same shape as an
explicit stop-and-ask trigger, not merely an aspirational quality bar.

### BU-DISP-12 -- verdict: CONFIRMED

`90-reconcile-fleet/CONTEXT.md` re-read: itemized per-repo gate list (pinned
scope, validation, review artifacts, zero blocking findings, CI, threads,
dependency order) plus the explicit "never complete merely because PRs
exist" rule matches J2-itemized-verification/J5-governing-PR-rule exactly
as stated.

### BU-DISP-13 -- verdict: NEEDS-REVISION

The disposition (STAND, in-place amendment required — add `## Bounded
judgment` to every stage `CONTEXT.md`, per `convention.md` §6.1 and ADR
0013 decision 4) is confirmed and independently re-derived: none of the
six stage files carries a `## Bounded judgment` heading (checked directly
by grep across all six).

The unit's own **description is not fully accurate to the package
content**: it states "All six stage `CONTEXT.md` files — uniform
`## Judgment required` boilerplate paragraph." Independently re-checked:
five of six (`00-check-queue-and-plan`, `05-classify-risk`,
`20-prepare-intent`, `80-monitor`, `90-reconcile-fleet`) do carry that
boilerplate. `15-check-admission/CONTEXT.md` does **not** — it has no
`## Judgment required` section at all; it substitutes "Deterministic
-machinery candidate" and "Additional note" sections instead (verified by
direct heading grep). "Uniform... across all six" overclaims by one file.
This does not change the required remediation (all six still need `##
Bounded judgment` added), but the unit's evidence should say "five of six
carry the boilerplate; `15-check-admission` carries neither the boilerplate
nor the required section" rather than asserting uniformity that does not
hold — the same self-check discipline (§8.10: "citations resolve") that
caught the drain-fleet/respond-to-worker defects should have caught this
smaller inaccuracy too.

### BU-DISP-14 -- verdict: CONFIRMED

Independently re-checked: `CONTEXT.md` (L1, read in full) has no `##
Authority envelope` heading. Matches `convention.md` §6.1's requirement
exactly as cited. No discrepancy found.

### BU-DISP-15 -- verdict: CONFIRMED

Independently re-derived from primary sources, not the producer's
citations: `drain-fleet` does not appear under `.sergeant/workflows/`
(directory listing checked directly — 20 entries, `dispatch` present,
`drain-fleet` absent), does not appear in `.sergeant/index.md`'s 20
-package catalog (read in full), and `.sergeant/drafts/workflows/` does
not exist at all in this working tree (checked directly) — so trivially
absent there too. `docs/icm/re-homing-record-2026-08-12.md` line 28 (read
directly) confirms retirement: "CLI-SURFACE, NET-NEW-SURFACE — no
admission-block primitive exists; engine-gap **G4**." The producer's
re-derivation of the checkpoint independent of the broken claim (fleet
-wide admission lock held across exactly one side effect, released
immediately) is sound and matches the actual stage content. FOLD (correct
the reference in place) is the right disposition — no placement change.

### BU-DISP-16 -- verdict: CONFIRMED

Independently re-derived: `respond-to-worker` is absent from the same
20-package catalog and directory listing (checked directly, same method
as BU-DISP-15). `docs/icm/re-homing-record-2026-08-12.md` line 22 (read
directly) confirms: "CLI-SURFACE, ABSORBED... 'collides with shipped `sgt
respond` (`src/cli.rs:89`)'... Nowhere new — the shipped `sgt respond` /
`POST /v1/work/{id}/input` already is this." The producer's re-derivation
(the escalation-without-inference judgment is real and self-contained; only
the stated delivery mechanism is wrong) is independently sound. FOLD is
correct — no placement change.

## Additional check: package identity and cross-references not in the table

- **`cross-repo-work` delegation citation** — independently re-verified
  that `cross-repo-work/CONTEXT.md` line 30 ("`50-handoff-or-stop`
  delegates to **dispatch**") is live and current (read directly). The
  draft's claim that this record's STAND verdict is load-bearing for that
  wave-2 package is accurate. No false-pairing issue found on `dispatch`'s
  side of that citation.
- **A parallel dangling reference the draft did not flag, because it
  belongs to a different package**: `cross-repo-work/60-reconcile/
  CONTEXT.md` (read directly) names `reconcile-and-cleanup-fleet` as an
  adjacent owned procedure alongside `dispatch`. `reconcile-and-cleanup
  -fleet` is also absent from the 20-package catalog. This is out of scope
  for `dispatch`'s own adjudication (the reference lives in
  `cross-repo-work`'s file, not `dispatch`'s, and `dispatch` does not cite
  it) — noted here only so `cross-repo-work`'s own ICM-R3 pass does not
  miss it, per the same defect class this brief asked `dispatch` to check
  for its own two citations.
- **Package naming**: `dispatch` does not collide with any other name in
  the current 20-package catalog or `skills/` root (checked directly).
  PL-4/in-Work admission boundary re-derivation (§5.6, §2a's "would a human
  type `sgt run '<intent>' --workflow dispatch`?" test) is independently
  confirmed: the package receives an already-scoped objective and repo set
  and does not itself decide whether work should exist — correctly PL-4,
  not PL-2.

## Overall verdict on Final disposition

**STAND is confirmed** — no unit's independent re-derivation produces a
different placement rung, and no new REHOME/SPLIT/HARVEST/ABSORBED
trigger was found. The package's six-stage structure, its PL-5/PL-6
rungs, and both FOLD dispositions (BU-DISP-15/16) all survive independent
re-derivation from the live package content.

However, this reviewer's pass surfaces **one confirmed remediation item
the producer's draft missed** (BU-DISP-03: the stale "no `kind =
\"execute\"` stage exists" claim in `15-check-admission/CONTEXT.md`,
already known-false as of `repo-to-icm`'s `65-self-check` and already
corrected in the sibling `research` package four days before this pass)
and **one J-rung classification this reviewer disputes** (BU-DISP-04:
J5-governing, not J2-delegated, for the admission lock's fixed hold
-and-release window). Neither changes STAND or PL-5/PL-6 placement for any
unit. Both should be folded into "Surviving package design"'s remediation
list (a sixth in-place amendment item alongside the five already there)
and the table's BU-DISP-04 J-column before this record is treated as
settled, per `reference/proposal-icm-r-procedure-authority.md` §8.12
("every finding is accepted, rejected, merged, or parked with rationale").
Until then, this record's own "Authority-valid: not yet" caveat (already
correctly stated in the draft) should be read as also covering these two
items, not only BU-DISP-13/14.
