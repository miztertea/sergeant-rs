# C3 — Provenance Completeness, Draft Validity, Engine-Gap Quality, Review Convergence

Scope: `docs/gauntlet/contracts/N2.md` §9.8–9.9 measurement dimensions
"provenance completeness," "draft validity," "engine-gap quality," and
"review convergence," defined in
`reference/proposal-next-iteration-icm-workflows.md` §9.9. Generated
artifacts: the 3 packages under `docs/gauntlet/runs/n2-run2/drafts/
workflows/{dispatch-mode,standard-task-workflow,ship-with-no-mistakes}/`
and their `provenance.md`; `docs/gauntlet/runs/n2-run2/workflows/
repo-to-icm/{40-classify,70-lint,80-adversarial-review,90-reconcile}/
output/*`. Reference: `reference-corpus/engine-pressure.md`,
`reference-corpus/draft-workflows/*`, `reference-corpus/FROZEN.md` v1.

**Scope discipline.** Per `comparison-c1-coverage-recall.md` §0, this run's
108 units come from **16** distinct files, not the 18 its own
`measurement-package.md` and `grammar-pressure.ndjson` claim (that
arithmetic error is itself relevant to §4 below and is not re-derived
here — see C1 §0 for the three-way independent verification). Nothing in
this document depends on the file count, but any quote from this run's own
artifacts repeating "18" is reproduced verbatim, not corrected, and
flagged where it recurs.

---

## 1. Provenance completeness

**Method.** Read all three `provenance.md` files in full. Cross-checked
every `behavior_id` each one cites against `40-classify/output/
classifications.ndjson` programmatically — not just "does the id exist"
but "does the cited `representation`/`workflow`/`stage` match what
`provenance.md` claims for it."

**Result: 16/16 citations across all three packages resolve exactly.**

| `behavior_id` | Package claims | `classifications.ndjson` actually has | Match |
|---|---|---|---|
| `BU-0003` | workflow / `dispatch-mode` | `workflow`, `dispatch-mode`, — | yes |
| `BU-0041` | stage / `dispatch-mode`/`dispatch-worker` | `stage`, `dispatch-mode`, `dispatch-worker` | yes |
| `BU-0011`–`BU-0013`,`BU-0016`,`BU-0019` | stage / `standard-task-workflow`/named stage | same, 5/5 | yes |
| `BU-0014`,`BU-0015` | unattached stage-context / `standard-task-workflow` | `stage-context`, `standard-task-workflow`, `confirm-decisions`/`monitor-progress` | yes |
| `BU-0028`–`BU-0034` (7) | unattached stage-context / `ship-with-no-mistakes` | `stage-context`, `ship-with-no-mistakes`, 4 distinct stage names | yes |

Zero fabricated citations, zero mismatched representation/workflow/stage
fields. `70-lint/output/lint-report.md`'s independent structural pass
(`[S8]`, "every workflow/stage has provenance") reports **zero `[S8]`
defects** across all four validator runs (three packages plus the
workflow's own tree) — confirmed by re-running the validator myself (§2
below): none of the three failures it reproduces is an `[S8]` failure.

**Uncovered/unattached ground is recorded, not invented.** All three
packages are explicit about the evidence they *don't* have:

- `dispatch-mode/provenance.md`'s "Uncovered ground" section states
  `BU-0003`'s own completion-condition clause ("progress is monitored
  through to reconciliation of merge order and cross-repo implications")
  has **no** corresponding stage evidence anywhere in the 108-unit corpus
  — no stage is invented to cover it.
- `standard-task-workflow/provenance.md`'s "Unattached ground" section
  names `BU-0014`/`BU-0015` (steps 5, 7) as guidance content with no
  member-stage home, and separately flags that steps 3, 6, 8 have no
  classified evidence in this corpus at all.
- `ship-with-no-mistakes/provenance.md` is the extreme case: **zero**
  `representation: stage` records name this workflow anywhere in the
  corpus — the entire package rests on 7 unattached `stage-context`
  records, and its own provenance file says so under a heading titled
  "the entirety of this candidate's direct evidence," not disguised as
  something stronger.

**Reopenability of cited source.** §9.9 also asks "can every source-cited
behavior be reopened at the pinned revision?" `comparison-c1-coverage-
recall.md` §4.1 already established the relevant fact for this question:
108/108 generated units' `quote` field is a literal, verbatim substring of
the named file at the pinned SHA — every citation these three packages
ultimately rest on is genuinely reopenable. Separately (not conflatable
with reopenability), 16/108 (14.8%) of the underlying behavior-unit
records fail their own `hash(quote) == quote_hash` self-check due to a
500-character truncation defect (C1 §4.1) — none of the 16 affected ids
(`BU-0017`, `BU-0023`, `BU-0024`, `BU-0026`, `BU-0027`, `BU-0036`,
`BU-0037`, `BU-0038`, `BU-0068`, `BU-0070`, `BU-0083`, `BU-0086`,
`BU-0089`, `BU-0090`, `BU-0092`, `BU-0093`) happens to be among the 16
ids cited directly by any `provenance.md` file, so this defect does not
degrade *these three packages'* provenance completeness specifically, but
it is a live defect in the corpus the packages are drawn from and is
counted against the run in C1, not silently dropped here.

**Depth, not conflated with completeness.** Every citation that exists is
real and correctly typed — completeness, in the narrow §9.9 sense, is
clean. But the citation *count* per package is thin against the
reference's own provenance depth for the same territory: `dispatch-mode`
rests on 2 citations (`BU-0003`, `BU-0041`) where `comparison-c2-
boundaries.md` §2.1 shows the reference's `dispatch` package alone carries
69 provenance citation-rows for the corresponding territory (19 of them
inside this run's reached scope). That is a coverage/boundary finding
already made in C1/C2, restated here only to keep it from being read as a
provenance-completeness defect — it isn't one; every row present is
genuine.

---

## 2. Draft validity

**Method.** Re-ran `.sergeant/workflows/repo-to-icm/scripts/
validate-structure.py` against each of the three materialized packages
directly (not trusting `70-lint/output/lint-report.md`'s transcript —
independently reproduced):

```
$ python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py \
    docs/gauntlet/runs/n2-run2/drafts/workflows/dispatch-mode
FAIL — 1 defect:
 - [S12] dispatch-mode: outputs are declared but the closing stage
   `10-dispatch-worker` names no finalize step (docs/icm/convention.md
   §1a, D9)

$ python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py \
    docs/gauntlet/runs/n2-run2/drafts/workflows/standard-task-workflow
FAIL — 1 defect:
 - [S12] standard-task-workflow: outputs are declared but the closing
   stage `50-reconcile-and-deliver` names no finalize step
   (docs/icm/convention.md §1a, D9)

$ python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py \
    docs/gauntlet/runs/n2-run2/drafts/workflows/ship-with-no-mistakes
FAIL — 1 defect:
 - [S3] ship-with-no-mistakes: workflow.toml has no non-empty
   `workflow.stages` array
```

**Result: 3/3 packages FAIL the structural validator. This exactly
reproduces `70-lint/output/lint-report.md`'s own transcript** — same
codes, same messages, same 1-defect-per-package count — so the run's
self-report on this dimension is accurate, not the source of a new
discrepancy the way §9.9's coverage self-report was (C1 §0).

### 2.1 The `[S12]` finding, examined

`[S12]` fires because `docs/icm/convention.md` §1a's D9 rule ("a workflow
that declares any output ends with a deterministic finalize step") is
checked mechanically: does any stage's `output/README.md` declare an
expected artifact, and if so, does the *closing* stage's `CONTEXT.md`
contain the literal word "finalize"? Both `dispatch-mode`'s
`10-dispatch-worker` and `standard-task-workflow`'s
`50-reconcile-and-deliver` declare real expected artifacts (`output/
README.md`: "**Expected artifact:** a delivery record — the verified
terminal state and preserved evidence checked before cleanup, and the
cleanup outcome itself," `**Disposition:** promote`, for the latter) —
this is not a spurious trigger; these packages genuinely intend to
produce a durable artifact with nowhere named to finalize it.

`70-lint`'s own handling of this is not silence or a mechanical patch: its
`lint-report.md` classifies the finding as **substantive**, quoting each
stage's own `CONTEXT.md` back at itself — both closing stages' "How to do
it" sections state plainly that they are "orientation-level ... not a
finished, ready-to-run stage contract" and that "detailed method content
is a promotion-time task for a human reviewer." Naming a finalize step
would mean inventing operational content these stages were not given the
authority to invent, so `70-lint` correctly declines to force-fix it and
carries the defect forward unresolved. This is the right call under
`references/mechanical-vs-substantive.md`'s own test — but it does not
change the fact that the validator's verdict on the delivered artifact is
FAIL, twice, for the same reason across the run's two non-trivial
packages.

### 2.2 The `[S3]` finding, examined

`ship-with-no-mistakes` fails differently and for a reason its own
`provenance.md`/`CONTEXT.md`/`workflow.toml` comment all state up front:
`stages = []` is deliberate, because this corpus supports zero classified
`stage` records for this candidate and `60-draft`'s own contract forbids
inventing stage boundaries from the 7 unattached `stage-context` records
just to produce a passing shape. This is the honest failure mode, not the
sloppy one — but it is still a validator FAIL, and per `references/
mechanical-vs-substantive.md`'s framing, a package that cannot pass with
zero stages is "not promotable as-is" by the package's own admission.

### 2.3 Verdict

§9.9 asks the question in its plainest form: *"Do generated packages pass
the structural validator without manual syntax repair?"* The literal
answer is **no — 0 of 3**. All three failures are correctly classified
substantive rather than mechanical (none is a syntax slip a script could
safely auto-fix; `70-lint` found and applied **zero** mechanical repairs
across all four validator runs, including its own tree), and all three are
honestly self-disclosed inside the very files that fail (not hidden, not
silently worked around). That distinguishes this from a LESSONS-L7-class
"sloppy, untested" failure — but it does not change the dimension's
literal verdict. A package the workflow itself, by design, marks
"orientation-level, not ready-to-run" is not yet a draft that clears its
own structural bar.

---

## 3. Engine-gap quality

**Reference side (`reference-corpus/engine-pressure.md`, `FROZEN.md`):**
16 first-pass `engine-gap` behavior units → 9 distinct claims → **4
domain-level survivors** after full §6.7 template application and
round-1 re-adjudication:

| Claim | Behavior | Source files | Rank |
|---|---|---|---|
| G1 | Runtime-owned durable wait/wake scheduling | `bin/sgt-wake`, `templates/worker-brief.md`, `tests/sgt-wake-test.sh` | 1 |
| G2 | Fleet identity + dependency-edge advance | `skills/dispatch/SKILL.md`, `bin/sgt-dag-dispatch-hook`, `bin/sgt-dag-run` | 2 |
| G3 | Durable outbound notification queue + ack gate | `docs/callbacks.md` | 3 |
| G4 | Operator-declared durable admission block | `bin/sgt-drain` | 4 |

(G5 rejected on re-derivation at round-1 adjudication — its "never
attempted" lower rung, a re-enterable `needs_input` stage, is already
shipped; G6 survives only partially, downgraded to grammar pressure, not
counted among FROZEN.md's "4 surviving"; G7–G9 rejected, absorbed at
lower rungs.) Every surviving claim carries multiple independently-found
citations across different artifact classes (G1: a command, a template,
and a test, plus 7 corroborating units) and passed an explicit
adversarial-narrowing pass — G1's own scope was cut down from "a
first-class waiting state" to "just the scheduler" during that pass, and
round-1 adjudication *reversed* G5 outright after re-checking its claimed
lower rung against the shipped engine rather than the proposal text.

**Generated side:** `40-classify/output/classifications.ndjson` carries
**zero** `representation: engine-gap` records (representation mix, C2
§4.1: 13 `agents-invariant`, 1 `workflow`, 6 `stage`, 9 `stage-context`,
79 `shared-helper`, 0 `shared-context`, 0 `obsolete-mechanism`, 0
`engine-gap` — confirmed independently by `60-draft`, `70-lint`, and
`80-adversarial-review`, all three counting programmatically and
agreeing). `90-reconcile/output/grammar-pressure.ndjson` separately
carries **2 meta-level** `engine_gap` records — about the `repo-to-icm`
workflow's own mechanics, not about any behavior in the subject
repository:

1. `20-harvest`'s own turn-budget/fan-out limit (the coverage gap C1
   documents at length, reframed here as engine pressure: "the runtime
   would need to own a durable fan-out ... a bounded set of actor
   sub-turns over a partitioned input set").
2. `60-draft`'s disposition/finalize mechanism not reaching content
   materialized outside any stage's own `output/` — precisely the gap
   `[S12]` above is a symptom of at the structural-lint layer.

### 3.1 The exact source material for 3 of the reference's 4 survivors was inside this run's reach — and yielded zero engine-gap findings

`bin/sgt-wake` (G1's primary citation), `bin/sgt-dag-dispatch-hook` and
`bin/sgt-dag-run` (G2's two `bin/` citations), and `bin/sgt-drain` (G4's
sole citation) are **all four** inside this run's own 16-file reached
scope (C1 §1) — this run read every one of them and extracted units from
every one of them:

| File | Generated units | Classified as |
|---|---|---|
| `bin/sgt-wake` | `BU-0095`–`BU-0099` (5) | `shared-helper`, all 5 |
| `bin/sgt-dag-dispatch-hook` | `BU-0051`–`BU-0053` (3) | `shared-helper`, all 3 |
| `bin/sgt-dag-run` | `BU-0054`–`BU-0058` (5) | `shared-helper`, all 5 |
| `bin/sgt-drain` | `BU-0064`–`BU-0070` (7) | `shared-helper`, all 7 |

All 20 of these units were classified `shared-helper` — not one raised an
engine question. The rationale text is uniform in shape across all 20
(`40-classify/output/classifications.ndjson`): e.g. `BU-0095`
("Evaluating a wake condition through a read-only adapter keyed on its
declared kind is deterministic machinery `sgt-wake` applies the same way
to every waiting worker"), `BU-0096` ("Failing terminally once a wake
condition's deadline passes is deterministic, uniform timeout handling in
`sgt-wake`"). Each is a §6.5 machinery-subordination judgment applied to
content the reference's own G1 treats as exactly the seam requiring a new
durable runtime fact (backoff/deadline/attempt-count bookkeeping owned by
the runtime, not "loose files" or a helper re-invoked externally — G1's
own §4 explicitly rejects "helper re-invoked by scheduler" as a lower rung
that already ships and already fails). This is the same §6.3-vs-§6.5
pattern C2 §4.2 documents for `sgt-recover`/`sgt-respond`/`sgt-watch`,
recurring here at the point where it costs the run an entire dimension:
none of the four files whose exact content the reference used to derive
3 of its 4 surviving engine-gap claims produced even a candidate
engine-gap flag in this run.

### 3.2 Template quality of the 2 claims actually produced

Checked against `docs/icm/record-shapes.md` §5's six required fields and
its five numbered rules: both `grammar-pressure.ndjson` records carry all
six fields (`behavior`, `source_evidence`, `lower_rungs_attempted`,
`why_each_fails`, `minimum_runtime_capability_required`,
`observable_acceptance_test`), name real §6 ladder rungs (`stage`,
`helper`) rather than restating the gap, give a mechanics-specific reason
per rung (not "not powerful enough" repeated), contain no "would be
convenient" language, and each closes with a checkable acceptance test
("the run's own journal shows more than one actor turn was used ... with
the daemon — not the actor — tracking partition completion"). On pure
template conformance, both are well-formed — not a defect on that axis.

What is thinner: `source_evidence` for each claim is exactly **one**
self-generated document (`20-harvest/output/coverage-note.md` for claim 1,
`60-draft/output/draft-report.md`'s own prose for claim 2) — evidence
about this run's own execution, not independently-found evidence from
multiple artifact classes in the subject repository the way every
reference G-claim is built (G1: a command file, a template, and a test,
found independently by three partitions). This is not a template
violation, but it is a different, weaker evidentiary genre: the reference
claims argue "the subject system needs this," these two argue "this
workflow's own current run needed this."

### 3.3 Process: neither meta-claim was ever adversarially reviewed

`80-adversarial-review` runs *before* `90-reconcile`. Its own Axis 3
("engine-gap refutation") explicitly checked `classifications.ndjson` —
correctly found 0 `engine-gap` records there — and states the axis
"is therefore applied vacuously this run: there is nothing to
independently re-attempt rungs against." `grammar-pressure.ndjson`'s two
meta-claims do not exist yet at that point in the pipeline — they are
authored by `90-reconcile` itself, the same stage that would otherwise
adjudicate them. No fresh execution ever challenged either claim's named
lower rungs, evidence sufficiency, or scope, the way the reference's own
process did explicitly and repeatedly (round-1 adjudication reopened and
reversed G5's verdict; G9's absorption reasoning was independently revised
on separate grounds). Both of this run's engine-gap claims are real,
well-templated, and plausible — but by pipeline construction, neither
one has survived anything; nothing tried to refute them.

### 3.4 Verdict

§9.9's actual question is "how many proposed engine features survived
lower-rung refutation and independent review?" On the domain-behavior
axis the reference's 4 survivors were built on, this run produced **zero**
candidates despite holding the exact source material for 3 of the 4 (§3.1)
— a clean miss, not a near-miss. On the workflow's-own-mechanics axis,
this run produced 2 template-complete claims that are real findings about
`repo-to-icm` itself (and the second directly explains the `[S12]`
structural defect in §2), but **zero of the two have been reviewed by
anything** — the pipeline's own ordering placed their authorship after
the one stage designed to challenge them. These are two different kinds
of engine-gap claim (tool-mechanics vs. subject-domain) and should not be
averaged into one number; reported separately, both counts are worse than
their face-value "2 produced, well-formed" reading suggests.

---

## 4. Review convergence

**Literal count.** `80-adversarial-review/output/findings.ndjson`: 0
records. `review-summary.md`'s own table: 0/0/0 across all three
axes×severities. `90-reconcile/output/adjudication-log.md`: 0 accepted, 0
rejected, 0 parked — "there was nothing to dispose."

**This is a real, effortful pass, not a skipped one.** The transcript
supports that reading: 29 of 108 citations independently hash-reverified
(spanning all 16 covered `source.path` values), all 16 distinct
`behavior_id`s cited across the three `provenance.md` files independently
confirmed to exist, the §6.3 reimplementation test independently
re-applied to all 6 `stage` records, a literal grep for
`reference-corpus` across every artifact with all 3 hits individually
classified as expected blindness-attestation prose rather than
contamination. Grading it a "lazy zero" would be inaccurate.

**But real defects sat inside its own declared, reference-independent
scope and were missed:**

- **Hash self-consistency (C1 §4.1), 16/108 records.** The review's own
  Axis-2 "Citation re-verification" methodology explicitly works around
  this: it recomputes the hash by "re-locating the full span from the
  500-char prefix and hashing the complete span, **not the stored
  prefix**" — sound for confirming the cited span is genuinely real (it
  is), but by that same design it never checks `hash(stored quote) ==
  stored quote_hash`, so it structurally cannot and did not catch the
  exact defect present in 14.8% of the corpus it was sampling from. This
  check needs nothing from `reference-corpus/` — it is pure internal
  consistency between two fields already present in this run's own
  `behavior-units.normalized.ndjson`.
- **Coverage-claim arithmetic (C1 §0).** `20-harvest/output/
  coverage-note.md` (quoted verbatim into both `grammar-pressure.ndjson`
  and `measurement-package.md`) asserts 18 files reached and 118 not
  reached; the same document's own named-file list sums to 16, and
  `16 + 120 = 136` while `18 + 118 = 136` — both arithmetically closed,
  but only one matches the file list actually printed. The review's own
  "Blindness boundary" check *opened and quoted from this exact file*
  (`coverage-note.md:76`) for an unrelated purpose (confirming no
  `reference-corpus/` grep-contamination) and did not notice the
  self-contradiction sitting in the document it had open. Also purely
  self-checkable, no reference needed.
- **Asymmetric over-staging check (C2 §3, §4).** Axis 2's "over-staging
  (reimplementation test)" was applied to all 6 `stage` records — checking
  each for whether it was wrongly *promoted*. It was never applied in the
  other direction: none of the 79 `shared-helper` records was checked for
  whether it was wrongly *demoted* (i.e., whether §6.3's own
  "would this checkpoint survive reimplementation" question was ever
  asked of them, rather than jumping straight to §6.5's subordination
  question, as C2 §4.2 shows happened uniformly). Confirming the specific
  *reference* stage boundary each demoted item should have matched
  requires the reference corpus and is legitimately out of reach — but
  asking the bidirectional question at all ("did we only test promotions,
  never demotions, with the same test we already trust enough to apply
  one direction?") is a pure methodology check on this run's own
  artifacts. This gap is exactly what let the run's single largest
  representation-agreement disagreement (`shared-helper` ~9x
  overrepresented, C2 §4.1) pass through adversarial review with zero
  findings.

**Legitimately out of scope, and correctly declared so.**
`measurement-package.md`'s own "NOT covered here" section lists
behavioral recall, workflow-boundary agreement, stage-boundary agreement,
representation agreement, and engine-gap quality as five of ten §9.9
dimensions "this package does not and cannot cover ... from inside this
run" — an accurate and honest limitation, not an omission to hold against
review convergence specifically. The A4 over-staging precedent (C2 §3),
the `dispatch`-cluster boundary gap (C2 §2.1), and the bulk of C1's recall
misses fall here: a blind, single-pass review cannot check its
decomposition against adjudication rulings or ground truth it is
forbidden to open.

**The lint-layer findings never received any disposition at all.**
`[S12]` (×2), `[S3]`, and `[S10]` were correctly generated by `70-lint`,
correctly classified substantive (not force-fixed), and correctly *not*
re-litigated by `80-adversarial-review` (its own "Not re-litigated"
section says so explicitly, citing exactly these four codes). But
`90-reconcile/output/adjudication-log.md`'s accept/reject/park mechanism
applies, by its own stated scope, strictly to `findings.ndjson`'s
contents — and these four never entered that file. The result: 4 known,
named, correctly-classified structural defects exit this run's pipeline
with **no** disposition record anywhere — not accepted, not rejected, not
parked, just present in `lint-report.md` and otherwise unowned by the
reconciliation step whose entire job is disposing of findings. `[S10]`
(`20-harvest/quote.sh` unclassified executable) is a fifth item in the
same unowned state, on the workflow's own admitted tree.

### 4.1 Verdict

The literal review-convergence metric (findings unresolved after
reconciliation) is 0/0 — vacuously clean, because reconciliation never
received anything to resolve. That number is technically accurate and
also close to meaningless as a signal of draft quality: within the
review's own declared, reference-independent reach, at least two
concrete, cheaply-checkable defects (hash truncation, coverage
arithmetic) were missed outright, and one methodology asymmetry
(promotion-only over-staging checks) left the run's largest single
disagreement with the reference (§4.2's shared-helper skew) completely
unflagged. Outside that reach, the review's restraint — not guessing at
recall or boundary agreement it cannot check blind — is correct
discipline, not a defect. Grade: **the review stage's methodology is
real and its restraint on reference-dependent dimensions is correct; but
"zero findings" measurably undercounts what was checkable without the
reference, and should not be read as evidence the three drafts are
clean** — §1–§3 above, and C1/C2 in full, are the evidence that they are
not.

---

## 5. Summary

| Dimension | Finding |
|---|---|
| Provenance completeness | Clean: 16/16 citations across all 3 packages resolve exactly (representation, workflow, stage all match); uncovered/unattached ground recorded honestly, never invented; 108/108 underlying quotes verbatim-reopenable. Thin relative to reference depth (2 citations for `dispatch-mode` vs. `dispatch`'s 69), which is a boundary/coverage finding (C1/C2), not a completeness defect. |
| Draft validity | 0/3 packages pass `validate-structure.py` (`[S12]` ×2 — no finalize step named despite declared outputs; `[S3]` — deliberate empty stage list). All three failures self-reproduced independently, correctly classified substantive not mechanical, and honestly disclosed inside the failing files themselves — but the literal §9.9 bar ("passes without manual repair") is failed by all three. |
| Engine-gap quality | 0 domain-level engine-gap claims generated, despite this run holding the exact source files (`bin/sgt-wake`, `bin/sgt-dag-dispatch-hook`, `bin/sgt-dag-run`, `bin/sgt-drain`) behind 3 of the reference's 4 surviving domain claims (G1, G2, G4) — all 20 units from those files classified `shared-helper` instead. 2 meta-level claims about the workflow's own mechanics were generated, template-complete but built on single self-generated citations and authored downstream of the one stage designed to refute them, so neither has been adversarially reviewed. |
| Review convergence | Literal 0 findings / 0 unresolved — a real, effortful pass within its declared 3-axis scope, not fabricated. But it missed 2 concrete, reference-independent defects squarely inside that scope (hash-truncation self-consistency, coverage-arithmetic self-contradiction) and its over-staging check ran promotion-only, letting the run's largest representation disagreement through unflagged. 5 known structural defects (`[S12]`×2, `[S3]`, `[S10]`×1 on the admitted tree) exit the pipeline with no accept/reject/park disposition at all. |
