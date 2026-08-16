# Independent adversarial review: validate-and-ship package adjudication

ICM-R2 pilot, `reference/proposal-icm-r-procedure-authority.md` §8.11 (Step
10 — Independent adversarial review). Reviewer position, independent of the
producer who wrote `adjudication-draft.md`. No edit authority over the live
package or the producer's draft — this record only. Checked against
`docs/adr/0013-icm-r0-owner-rulings.md`, §5/§6 (Placement and
Bounded-Judgment ladders), §8.10-8.11 (self-check / adversarial checklist),
and `docs/icm/record-shapes.md` §6 (record shape). Every disposition below
was independently re-derived by reading the actual package content under
`.sergeant/workflows/validate-and-ship/` and its cited primary sources
(`reference/sergeant-upstream/...`, `scripts/gate.sh`,
`docs/DEVELOPMENT.md`, the two named retrospectives, `docs/icm/convention.md`,
`docs/icm/retriage-2026-08-11.md`, `docs/icm/re-homing-record-2026-08-12.md`)
directly, not from the producer's own citations.

## Behavior-unit dispositions

### BU-VAS-01 -- verdict: CONFIRMED

Independent re-derivation: `CONTEXT.md`'s Purpose line and `index.md`'s
description both state the package identity verbatim: "validate a committed
change through the pipeline to a terminal outcome, routing every finding,
without the validating actor ever editing the code." PL-4 (package-level,
whole-lifecycle-from-admitted-intent-to-terminal-result) and J5 are both
correctly cited.

One precision note, not a disagreement: read literally against the rest of
the package, "the actor never edits the code" is true only from the point
validation begins (`20-select-intent-transport` onward) — `10-do-the-work`
explicitly has the actor *make and commit* the task's changes
(`BU-P2-061`) before validation starts. The producer's own full-text intent
("the validating actor") already scopes this correctly and is consistent
with `40-drive-gates`'s narrower, textually exact J5 restatement
(BU-VAS-08, "the actor never edits the pipeline-owned worktree"). The
one-line table cell is compressed enough that a future reader could
misapply it package-wide; worth tightening in-place ("the actor never
edits the pipeline-owned worktree once validation has begun") but this
does not change the disposition.

### BU-VAS-02 -- verdict: CONFIRMED

`00-check-scope/CONTEXT.md` was read in full. `BU-P2-058`/`059` citations
verified against `reference/sergeant-upstream/.agents/skills/no-mistakes/
SKILL.md` lines 20-31 — exact match, both content and line numbers. PL-5 /
J2 both hold: distinguishing validate-only from task-first and translating
an ambiguous flag request are genuinely different-response judgment calls,
not lookup-table mechanics.

### BU-VAS-03 -- verdict: CONFIRMED

`10-do-the-work/CONTEXT.md` read in full; `BU-P2-060`/`061` verified against
source lines 36-42 — exact match. PL-5/J2 correctly applied: isolating
task-scoped changes from unrelated pre-existing uncommitted changes is
judgment, not a mechanical diff, as the stage's own "Judgment required"
text argues.

### BU-VAS-04 -- verdict: CONFIRMED

`20-select-intent-transport/CONTEXT.md` read in full; `BU-P6-134`,
`BU-P8-085` verified against `reference/sergeant-upstream/bin/sgt-validate`
L274-281 and `docs/using-sergeant.md` L333-339 — both resolve and match.
The J5 (never expose intent via argv) + J4 (operator's per-run consent)
pairing is correctly distinguished: the prohibition itself is a governing
constraint, but *choosing to use* the argv-exposing option at all requires
the operator's own explicit act, which is J4 rather than a second J5.

### BU-VAS-05 -- verdict: CONFIRMED

The "Helper: verify readiness, acquire launch reservation, reserve isolated
snapshot" section was read in full and its five citations
(`BU-P6-130`, `BU-P8-082`, `BU-P8-084`, `BU-P6-143`, `BU-P6-133`, `BU-P6-044`)
independently checked for resolution against `sgt-validate` and
`using-sergeant.md`; all found governing, deterministic, and correctly
scoped to the coordinator-launched entry only, as the section's own opening
sentence states. PL-6/J5 hold: none of this content requires choosing among
alternatives — every clause is "refuse the launch if X," which is exactly
PL-6's "output follows mechanically from declared inputs" test.

### BU-VAS-06 -- verdict: DISPUTED

The producer's PL-6/J5/STAND classification treats this as settled, correctly-
executed re-homing. Independently checking the underlying claim — "Its
behavior is this repository's own git pre-push hook, which mechanically
gates every push in this repository" (`20-select-intent-transport/CONTEXT.md`
line 65) — finds it is **not currently true of this repository**:

- `reference/sergeant-upstream/scripts/hooks/pre-push` (the cited source,
  `BU-P6-007`/`008`) lives under `reference/sergeant-upstream/`, which
  `docs/DEVELOPMENT.md`'s own development record explicitly calls "frozen
  evidence: don't edit it to change behavior" — it is the *source project*
  being reconciled, not this repository's live tooling.
- This repository's actual git hooks directory
  (`.git/worktrees/.../hooks/` at the shared `.git-common-dir`) contains no
  `pre-push` hook at all — checked directly; only the stock `.sample` files
  are present.
- No `scripts/hooks/pre-push` exists anywhere under this repository's own
  tree (checked directly); `scripts/gate.sh` is the only shipping-adjacent
  script under `scripts/`, and it is a manually-invoked wrapper, not a git
  hook.

So the behavior this row cites is aspirational/not-yet-installed in
sergeant-rs, not a live PL-6 mechanism "mechanically gating every push in
this repository" today. This is a source-fidelity defect (§8.11 checklist
item 1): the producer's row conflates the upstream reference corpus
(what the behavior unit was extracted *from*) with this repository's
current, actually-installed state (what the re-homing claims to be true
*of*). The correct disposition is not necessarily different in kind — PL-6
may still be right once the hook is actually installed — but "STAND,
re-homing already correctly executed; no further placement change needed"
overstates what exists. Recommend: either (a) reclassify as an open
implementation gap (install the hook per the cited upstream source, then
re-verify PL-6), or (b) if a pre-push safeguard is deliberately not wanted
in this repository, correct the CONTEXT.md prose so it no longer asserts a
live mechanism that isn't there. Either way this is a live discrepancy, not
a closed re-homing.

### BU-VAS-07 -- verdict: CONFIRMED

`30-start-run/CONTEXT.md` read in full, all fourteen citations spot-checked
against source (`BU-P2-062` through `BU-P2-074`, `BU-P1-042/069/070`) —
resolve and match. The stage's own "Additional note" argues the PL-5
re-rung (from a demoted §6.3 boilerplate classification at extraction) via
the reimplementation test directly, and independently re-applying that test
here agrees: swapping the deterministic precondition checks for different
tooling would not change this checkpoint's outcome, but the in-flight-run
disambiguation (reattach / leave alone / never bypass via abort) and intent
composition would. PL-5/J2 confirmed.

### BU-VAS-08 -- verdict: CONFIRMED

`40-drive-gates/CONTEXT.md` read in full — the largest behavior-contract
section in the package (18 units). Spot-checked `BU-P2-079/080` (gate
finding classification), `BU-P2-098/099` (ask-user escalation) against
source lines 122-137 and 223-238 — exact match. The proposal's own §3.4
Finding ICMR-F4 was independently located (`reference/proposal-icm-r-
procedure-authority.md` line 299-309) and does name this exact stage
("validate-and-ship/40-drive-gates already distinguishes three kinds of
gate finding... That is exactly the concept needed") as the Bounded-
Judgment Ladder's own worked precedent — the producer's citation of this as
"the canonical worked precedent this whole ladder generalizes from" is
accurate, not embellished. The J2 (auto-fix/no-op) + J0 carve-out (ask-user)
pairing is the correct rung pair per §6.5/§6.7: ask-user findings are
exactly the "conflicting/risk-changing, not delegated" case J0 exists for.

### BU-VAS-09 -- verdict: CONFIRMED

The "Helper: route findings" section (`BU-P6-023/026`, `BU-P1-080`,
`BU-P7-065`) resolves against `bin/sgt-no-mistakes-finding` and its test
file. PL-6/J5 correctly applied: severity/kind deterministically fixing td
routing eligibility is mechanical, not a judgment call, matching the
section's own framing.

### BU-VAS-10 -- verdict: CONFIRMED, with one additional defect the producer's table did not catch

The dangling-reference finding itself is independently confirmed: no
package or draft named `route-review-findings` exists anywhere under
`.sergeant/workflows/` or (nonexistent) `.sergeant/drafts/workflows/` in
this repository. `docs/icm/retriage-2026-08-11.md` line 52 and
`docs/icm/re-homing-record-2026-08-12.md` line 29 both confirm it was
retriaged to CLI-SURFACE/NET-NEW-SURFACE as unbuilt verb candidates
(`sgt review route-findings` / `sgt gate clear`), independently verified by
reading both cited lines directly. `convention.md` §4 rule 1 is correctly
applied: even though the current text does not literally use `@@name`
syntax, "delegates part of its outcome to **route-review-findings**"
(`CONTEXT.md` line 34) and "running **route-review-findings** to its own
completion" (`40-drive-gates/CONTEXT.md` line 96) both imply exactly the
kind of sub-workflow invocation §4 rule 1 forbids, for a package that no
longer exists to be invoked. FOLD is a defensible modifier here since the
behavior is already folded in-package as BU-VAS-09's helper — though this
row is really a citation defect on an already-correctly-placed behavior
rather than a unit earning its own placement, so FOLD is being used loosely
(there is no actual "unit" moving anywhere).

**Additional defect found independently, not on the producer's table at
all:** this package's own `CONTEXT.md` and every one of its seven stage
`CONTEXT.md` files repeatedly cite a co-located `provenance.md` file that
**does not exist anywhere in the package** (confirmed by listing every file
under `.sergeant/workflows/validate-and-ship/` directly — only `CONTEXT.md`,
`index.md`, `workflow.toml`, and the seven stage directories exist; no
`provenance.md`). Four separate citations point to it:

- `CONTEXT.md` line 30: "'s 'Adjudication A4' section", "'s 'Adjudication A5'
  section"
- `CONTEXT.md` line 46 ("## Provenance"): "See `provenance.md` for the
  complete stage-to-behavior-unit mapping and workflow-level citations"
- `20-select-intent-transport/CONTEXT.md` line 61: "`BU-P6-129`... is cited
  at workflow level in `provenance.md`"
- `20-select-intent-transport/CONTEXT.md` line 65: "See `provenance.md`'s
  'Re-homed from repo-release-verification (A6)' section"

This is exactly the "citations resolve" self-check §8.10 requires and the
"source fidelity" §8.11 challenge targets — a broken in-package reference
present in every stage of the workflow, missed entirely by the producer's
pass. (A file with adjacent content exists at
`docs/gauntlet/promoted-provenance/validate-and-ship.md` — the producer's
own draft cites that path correctly elsewhere — but nothing in the package
itself points there; the package's own prose asserts a same-directory
`provenance.md` that was apparently never carried over when the package was
promoted out of its draft-workflow location.) This should be added to the
package's required remediation list (see Final disposition below).

### BU-VAS-11 -- verdict: CONFIRMED

`50-reconcile-custody/CONTEXT.md` read in full; all seven citations
(`BU-P2-089` through `BU-P2-094`, `BU-P1-079`) verified against source lines
188-196 and 300 — resolve and match. PL-5 correctly applied; the J5 (never
improvise git surgery) + J2 (choose among the three structured remediation
paths) pairing is accurate — the prohibition on freelance git operations is
absolute, but which of `sync`/`continue_active_run`/`recover_custody` to
follow is read from state, which is delegated judgment within named bounds.

### BU-VAS-12 -- verdict: CONFIRMED

`60-close-out/CONTEXT.md` read in full, including the folded "Helper:
handover log" section (`BU-P8-089`, `BU-P7-104`). Citations spot-checked
against source lines 169-221 and 293-296 (`README.md`) and the handover
lines in `using-sergeant.md`/`sgt-validate-test.sh` — resolve and match.
PL-5/J2 (diagnose-and-fix on failed/cancelled) + J5 (never poll for merge)
pairing correctly reflects the actual stage content.

### BU-VAS-13 -- verdict: CONFIRMED

Independently read all seven stage `CONTEXT.md` files: every one carries a
`## Judgment required` section with near-identical boilerplate text ("This
is an actor stage (ladder §6.4)...") and none names a J2 decision class, a
J1 local choice, or a J0 escalation trigger in the shape ADR 0013 decision 4
requires ("every actor stage carries an explicit local `## Bounded
judgment` section always... omission is never ambiguous"). This is a real,
package-wide compliance gap, not a paraphrase issue — the section header
itself is the wrong one throughout.

### BU-VAS-14 -- verdict: CONFIRMED

Read `CONTEXT.md` in full: no `## Authority envelope` heading exists
anywhere in the file (confirmed by direct section-header scan). `convention.md`
§6.1 "Required sections" was independently located and does require this
section at the workflow Layer-1 level. Gap confirmed exactly as stated.

### BU-VAS-15 -- verdict: CONFIRMED

Independently re-checked all six rungs (J5 down through J0) against the
actual stage content of `30-start-run/CONTEXT.md` and
`40-drive-gates/CONTEXT.md`: neither names `push`, `pr`, `ci`, `--skip`, or
any autonomous-publication authority anywhere in either file (confirmed by
direct text search across both). `scripts/gate.sh:202` was read directly
and does say `exec no-mistakes axi run --intent "$intent" --skip
push,pr,ci "$@"` — exact match to the producer's citation. `docs/
DEVELOPMENT.md:105` was read directly and does say `` `scripts/gate.sh
"<intent>"` runs the no-mistakes pipeline (`--skip push,pr,ci`; push/PR
handled manually) `` — exact match. Both cited retrospectives were read in
full at the cited sections:

- `docs/gauntlet/runs/path-to-mac-2026-08-15/retrospective.md` §3.1
  confirms the push happened live via a dispatched Work that bypassed
  `gate.sh`, and its own §7 item 1 independently names this "a product
  gap," not resolved. (Note: that retrospective cites `scripts/gate.sh:122`
  for the `--skip` flag, while the producer's draft cites the current
  `scripts/gate.sh:202` — this is not a discrepancy, just the file having
  grown between 2026-08-15 and now; both point to the same
  `--skip push,pr,ci` clause, verified by direct read of the current file.)
- `docs/gauntlet/runs/macbook-arrival-2026-08-15/retrospective.md` §3
  confirms the second, independent live materialization (PR #141 opened
  autonomously against `main`) exactly as quoted.

The rung-by-rung record itself was independently re-walked: no J5 governing
text exists in-package; no J4 grant is visible to either stage; no J3
settled record answers it; no J2 delegation names this decision class; and
J1 does not apply because autonomous push/PR/CI is exactly the kind of
public-facing, not-locally-reversible-once-CI-notifies choice §6.6
excludes. J0 is the correct, honestly-arrived-at conclusion, and the
producer's refusal to invent the missing J-clause on its own authority is
the right application of the J0 procedure (§6.7 steps 1-3; a recommendation
was optionally offered and appropriately withheld here since the underlying
policy question is a live owner call, not a drafting gap this producer is
positioned to resolve).

## Overall verdict on Final disposition

**Confirmed: STAND**, with two corrections required before this record's
Final disposition can be called authority-valid or source-valid without
qualification (the producer's own "Validation evidence" section already
concedes authority-validity is "not yet" — this review agrees, and adds a
second, package-wide source-fidelity gap the producer missed entirely):

1. **BU-VAS-06's re-homing claim needs correction, not just placement
   confirmation.** The behavior the row cites (a live, repository-own
   pre-push drain-suite hook) does not currently exist in this
   repository's `.git/hooks/` or under `scripts/`. Either install the hook
   the citation describes, or rewrite the prose so it stops asserting a
   live mechanism that is not present. Until one of those happens, "STAND
   — re-homing already correctly executed" is not accurate.
2. **The package-wide `provenance.md` reference is broken everywhere it
   appears** (four citations across `CONTEXT.md` and
   `20-select-intent-transport/CONTEXT.md`), missed by the producer's own
   self-check pass. This should be corrected in the same remediation batch
   as BU-VAS-13/14's Bounded-judgment/Authority-envelope additions and
   BU-VAS-10's dangling-reference fix — either restore a co-located
   `provenance.md`, or repoint every citation to the actual archived file
   at `docs/gauntlet/promoted-provenance/validate-and-ship.md`.

Neither correction changes the package's identity, ownership, or stage
structure — both are in-place content fixes of the same kind the producer's
own "Surviving package design" section already schedules for BU-VAS-13/14/
15, so neither triggers ADR 0013 decision 6's REHOME/SPLIT/HARVEST
draft-and-rehome step. The seven-stage linear sequence, both entry
variants, and all fifteen behavior-unit placements (PL-4 package / PL-5
each stage / PL-6 each identified helper) are independently re-derived and
confirmed correct. No unjustified engine-gap claim was found; no missed J0
case was found beyond the one the producer already surfaced (BU-VAS-15); no
false Captain/workflow-boundary pairing was found (the execution-surface
test genuinely discriminates cleanly for both entries, independently
re-applied here); the one false-pairing-shaped issue this review did find
(BU-VAS-06) is a source-fidelity defect inside an already-correct rung, not
a placement error.

This record itself is reviewer output under this pilot's own procedure and
is not self-promoting; Captain's reconcile-and-publish pass (§8.12) still
decides what happens with the two corrections named above.
