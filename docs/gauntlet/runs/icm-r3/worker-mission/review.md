# Independent review: `worker-mission` (ICM-R3, package adjudication)

Fresh execution, explicit inputs read directly (not inherited from the
producer's characterization of them), review-only contract, no edit
authority over the producer's draft, per `docs/adr/0013-icm-r0-owner-
rulings.md` decision 7. Inputs read: `docs/adr/0013-icm-r0-owner-
rulings.md`; `reference/proposal-icm-r-procedure-authority.md` §§5, 6,
8.10-8.11; `docs/icm/record-shapes.md` §6; the producer's draft
(`adjudication-draft.md`); every file under
`.sergeant/workflows/worker-mission/`; the five delegate packages'
ICM-R3 records in full, including `tdd/review.md`,
`tdd/adjudication-draft.md`, `tdd/draft/fold-notes/
worker-mission-20-implement.md`, `implement/adjudication-draft.md`,
`implement/review.md`; `.sergeant/common/contexts/` (grepped);
`.sergeant/workflows/dispatch/CONTEXT.md`, `80-monitor/CONTEXT.md`,
`code-review/CONTEXT.md`; `docs/gauntlet/runs/icm-r3/
recover-stalled-worker/adjudication-draft.md`; `.sergeant/index.md`;
`docs/icm/convention.md` §4.

Per §8.11, this review challenges source fidelity, rung order,
Captain/workflow and stage/helper boundaries, authority grants and
missing J0 cases, package identity, drift-prone/duplicated content,
false pairing assumptions, and unjustified engine gaps — checked
directly against the live package content, not against the producer's
own restatement of it.

## Package identity, driver, admission boundary — verdict: CONFIRMED

Independently re-applied §5.6's PL-4 test to `worker-mission`'s own four
live stages (`workflow.toml`: `10-triage-and-route`, `20-implement`,
`30-independent-review`, `40-escalate-or-continue`, all `driver:
stage-actor`). The package has a recognizable trigger after intent
shaping (a rendered brief), a bounded outcome, durable checkpoints, and a
result meaningful independent of the originating conversation continuing.
Nothing in the five delegate records, or in `recover-stalled-worker`'s
own independent check of its isolation from `dispatch`/`worker-mission`,
challenges this. The producer's PL-4/STAND conclusion holds.

### BU-WM-01 -- verdict: CONFIRMED
### BU-WM-02 -- verdict: CONFIRMED
### BU-WM-05 -- verdict: CONFIRMED

Directly read `30-independent-review/CONTEXT.md`: its `## Behavior
contract` cites `BU-P7-013` verbatim as claimed, correctly states the
brief-authoritative-axis coverage requirement independent of whatever a
loaded review skill would otherwise cover. PL-5/J5 classification is
correct — this is a governing constraint, not a J2 narrowing choice.

### BU-WM-06 -- verdict: CONFIRMED
### BU-WM-07 -- verdict: CONFIRMED

Directly read `40-escalate-or-continue/CONTEXT.md`: both `BU-P7-009`
(ack/accept/act-once handshake) and `BU-P7-012` (monotonic
gate-generation counter, persist-before-write) are cited and quoted
accurately. PL-5/J5 is the correct rung for both — non-negotiable
correctness constraints, not local implementation choices.

### BU-WM-08 -- verdict: CONFIRMED

The folded `50-publish-result` content (`BU-P7-066`, `BU-P7-110`) is
present verbatim in `40-escalate-or-continue/CONTEXT.md`'s "Helper
invocations" section, correctly framed as PL-6 deterministic machinery
with mixed J5 (worktree-source verification is non-negotiable) / J1
(bounded-wait mechanics) — matches the source content exactly.

### BU-WM-03 -- verdict: CONFIRMED

`10-triage-and-route/CONTEXT.md`'s `## Behavior contract` correctly
states the five-category classification (`BU-P7-007`) as PL-5. The
finding under "10-triage-and-route's missing J0 for straddling
candidates" is independently checked against `deepen-module/review.md`
(its straddling-candidate finding for `00-classify-dependencies` is real
and confirmed there, not fabricated), and the analogy to
`worker-mission`'s own five-way, mutually-exclusive branching point
holds structurally. The J-rung elimination (J5 through J1, landing at
J0) is correctly reasoned and matches the ladder's own method (§6.3-6.7).

### BU-WM-04a -- verdict: CONFIRMED

Directly read `20-implement/CONTEXT.md`'s `## Delegation` section: the
"context composition today... does not exist yet" hedge is present
verbatim for all five branches, identical in kind to `implement`'s own
pre-revision `10-implement-with-tdd` wording (independently
cross-checked by reading `implement/10-implement-with-tdd/CONTEXT.md`
directly — the phrasing matches near-verbatim). This is the same defect
class `docs/icm/convention.md` §4 rule 1 names (misdescribing a
checkpointed, separately-admitted workflow invocation as mere context
pull-in), and the fix direction (state plainly that these four are
separately-admitted PL-4 workflows dispatched as their own Work, per
`proposal-next-iteration-icm-workflows.md` §7.7's "an agent... could
even submit another `sgt run`") is sound and correctly scoped to only
the four settled branches, since `diagnose-bug`/`prototype`/`implement`/
`deepen-module` are all independently confirmed STAND by their own
ICM-R3 records (re-verified directly here, not taken on the producer's
word: all four show `STAND` in their own `## Final disposition` and
their own independent reviewers' overall verdicts).

### BU-WM-09 -- verdict: CONFIRMED

Grepped all four stage `CONTEXT.md` files directly: all four use
`## Judgment required`, none uses `## Bounded judgment` with named
J2/J1/J0 subsections. Matches `docs/icm/convention.md` §6.1 /
`docs/adr/0013` decision 4's requirement exactly as the producer states.

### BU-WM-10 -- verdict: CONFIRMED

Directly read the workflow `CONTEXT.md`: no `## Authority envelope`
section exists. Confirmed gap.

### BU-WM-11 -- verdict: CONFIRMED

Confirmed directly: `CONTEXT.md`'s `## Provenance` section says "See
`provenance.md`"; `ls .sergeant/workflows/worker-mission/` shows no such
file (only the four stage directories, `CONTEXT.md`, `index.md`,
`workflow.toml`). The real archive is
`docs/gauntlet/promoted-provenance/worker-mission.md`, exactly as
claimed. Same defect class as `diagnose-bug`'s `BU-DB-12` and
`implement`'s `BU-IMPL-08` — plausible as a systemic pattern, not
independently re-verified against those two records' own text here
(out of this review's assigned package scope), but the `worker-mission`
instance itself is directly confirmed.

### BU-WM-12 -- verdict: CONFIRMED

`10-triage-and-route/CONTEXT.md`'s own "Additional note" self-flags
engine-gap G6 in language matching the producer's quotation exactly.
Correctly scoped as "not a new claim, additional source evidence" rather
than a fresh engine-gap record — the producer does not edit
`implement/draft/engine-gap-nested-workflow-invocation.md` itself, which
is the right restraint (out of this pass's assigned package scope,
mirroring `tdd`'s own restraint toward `test-quality.md`'s other
consumers).

### BU-WM-13 -- verdict: CONFIRMED

`grep -l "^Draft workflow package" .sergeant/workflows/*/CONTEXT.md`
independently re-run: returns 19 files, matching the producer's claim
and `deepen-module/review.md`'s own systemic-boilerplate finding
(confirmed by directly reading that finding, not merely trusting the
producer's citation of it). Correctly recorded as non-actionable at this
pass rather than silently dropped.

## The `tdd` citation and the `## Delegation` split (BU-WM-04b) -- verdict: NEEDS-REVISION

The producer's bottom-line handling of the `tdd` dispute is right on
both of the two questions this review was specifically asked to check:

1. **Did the producer correctly decline to adopt `tdd`'s disputed fold
   note?** Yes. Directly read
   `tdd/draft/fold-notes/worker-mission-20-implement.md`: it is
   explicitly conditioned on REHOME being accepted, and its proposed
   `@@tdd`/`@@test-quality` replacement text presupposes shared contexts
   that do not exist (`ls .sergeant/common/contexts/` independently
   re-run: only `bounded-judgment.md` exists). `tdd`'s own reviewer
   disputed REHOME rather than confirming it
   (`tdd/review.md`: "Final disposition from REHOME to DISPUTED... Do
   not promote REHOME as drafted"). Adopting the fold note now would be
   premature, and the producer correctly did not adopt it — directly
   confirmed by reading `20-implement/CONTEXT.md`'s live `##
   Delegation` text, which still carries the original, unrevised `tdd`
   wording.
2. **Is `worker-mission`'s own delegation citation to `tdd` left
   accurately flagged rather than silently resolved either way?** Yes.
   `BU-WM-04b`'s disposition ("FOLD — revise prose only on the
   dispute-independent point; do not adopt the disputed REHOME fold
   note's replacement text") and the "Surviving package design" section
   both state plainly that the `tdd` branch "should be revised again
   once `tdd`'s own ICM-R3 dispute resolves," pointing at
   `tdd/adjudication-draft.md` and `review.md` by path. This is an
   accurate flag, not a silent resolution toward either STAND or
   REHOME.

However, the producer's supporting argument for *why* the `implement`
precedent "applies with at least equal force" rests on a citation that
does not check out, and this is a real defect under §8.11's "source
fidelity" and "false pairing assumptions" challenge criteria:

> "...`tdd`'s own reviewer explicitly treats `worker-mission` as one of
> `tdd`'s "two genuinely independent direct parents" (`tdd/
> adjudication-draft.md` "Driver and admission boundary" point 2,
> confirmed again from the `implement` side at `implement/
> adjudication-draft.md` `BU-IMPL-09`)."

Checked directly, both halves of this citation are wrong:

- **The phrase is not `tdd`'s reviewer's.** Grepped `tdd/review.md` for
  "direct parent" and "genuinely independent": no hits. `tdd`'s own
  independent reviewer never uses this framing anywhere in its review.
  The phrase "two genuinely independent direct parents" appears only in
  `implement`'s own package — both in `implement/adjudication-draft.md`
  (`BU-IMPL-09`'s row) and in `implement/review.md` (`BU-IMPL-09`'s
  verdict, line 187: "This gives `tdd` two genuinely independent direct
  parents and `code-review` one"). It is `implement`'s producer and
  `implement`'s independent reviewer who make this claim about `tdd`,
  not `tdd`'s own reviewer.
- **The first citation ("`tdd/adjudication-draft.md` `Driver and
  admission boundary` point 2") is also internally inconsistent with its
  own attribution** ("reviewer" vs. a citation to the producer's own
  draft), **and does not contain the quoted phrase regardless.** Read
  `tdd/adjudication-draft.md`'s "Driver and admission boundary" section
  directly: point 2 states only that `worker-mission` and `implement`
  are "the only two references to `tdd` as something 'run'" — true, but
  not phrased as "two genuinely independent direct parents." That exact
  language appears in a different point (point 4: "two independent
  current consumers (`implement`, `worker-mission`)") in `tdd`'s own
  producer draft, and in full ("two genuinely independent direct
  parents") only in `implement`'s package, as above.

The parenthetical's second half ("confirmed again from the `implement`
side at `implement/adjudication-draft.md` `BU-IMPL-09`") is itself
accurate — that row does contain the exact phrase, independently
verified by direct read. So the producer did cite one correct source,
but framed it as corroboration of a first citation that is fabricated
in substance (misattributed to the wrong document and the wrong role —
"reviewer" rather than "producer," and to `tdd`'s package rather than
`implement`'s).

This does not overturn the producer's underlying, separately-verifiable
fact that `worker-mission`'s `20-implement/CONTEXT.md` names `tdd`
directly (confirmed independently here by reading the live file, not
from any of the disputed citations above) — so `worker-mission` is in
fact one of `tdd`'s two direct citing stages, and the argument that
`implement`'s precedent transfers "with at least equal force" is likely
still reachable on a corrected citation. But as drafted, the specific
evidentiary claim attributed to "`tdd`'s own reviewer" is not true, and
should not stand as written. **NEEDS-REVISION**: correct the citation to
attribute "two genuinely independent direct parents" to `implement`'s
own producer (`BU-IMPL-09`) and independent reviewer (`implement/
review.md`, `BU-IMPL-09`), not to `tdd`'s reviewer or to `tdd/
adjudication-draft.md` point 2. The `20-implement/CONTEXT.md` live
content and the FOLD/tracking-note disposition for the `tdd` branch
(`BU-WM-04b` itself) do not need to change; only the "Driver and
admission boundary" narrative's sourcing does.

## Overall verdict on Final disposition -- verdict: DISPUTED (narrow)

**Package-level disposition STAND is CONFIRMED** — independently
re-derived directly against the live package content, not assumed from
the producer's own restatement. Twelve of thirteen behavior-unit
dispositions (`BU-WM-01` through `BU-WM-03`, `BU-WM-04a`,
`BU-WM-05` through `BU-WM-13`) hold up against direct inspection of the
live files and the cited delegate/precedent records, with no citation
gap found. The `10-triage-and-route` straddling-candidate J0 finding
holds up. The decision to leave the `tdd` branch of `20-implement`'s
delegation partially unrevised — tracking `tdd`'s own dispute rather
than adopting either side — is the correct call and is accurately
flagged, not silently resolved, satisfying the specific dispute-handling
question this review was asked to check.

The dispute is narrow and does not reach the package verdict or the
remediation list: `BU-WM-04b`'s supporting citation for *why*
`worker-mission`'s status as a direct (not second-hand) `tdd` citation
strengthens the `implement` precedent's applicability misattributes its
key quote — "two genuinely independent direct parents" — to `tdd`'s own
independent reviewer and to a specific point in `tdd`'s producer draft
that does not contain it, when the phrase in fact originates entirely
from `implement`'s own producer and reviewer. This is a source-fidelity
defect (§8.11) that should be corrected before promotion, but it is a
citation-attribution error in the supporting narrative, not an error in
the underlying fact (`worker-mission` does cite `tdd` directly,
independently confirmed here) or in the resulting disposition (FOLD,
dispute-independent-only revision, tracking note) that fact supports.

**Recommendation for Captain's reconcile-and-publish pass:** accept the
package verdict (STAND) and the full remediation list as-is, with one
correction folded in — fix the misattributed "two genuinely independent
direct parents" citation in the "Driver and admission boundary" /
"The `tdd` citation" narrative section to point at `implement`'s own
producer and reviewer records rather than `tdd`'s. No change to any
`BU-WM-*` table row's disposition or destination is required.
