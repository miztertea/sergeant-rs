# N2 run-2 — Adjudicated comparison scorecard

Adjudicator over C1 (`comparison-c1-coverage-recall.md`), C2
(`comparison-c2-boundaries.md`), C3 (`comparison-c3-validity.md`).
Generated run: `docs/gauntlet/runs/n2-run2/`. Reference:
`reference-corpus/` (`FROZEN.md` v1 — 979 units, 34 draft packages).
Framing: `docs/gauntlet/contracts/N2.md` Outcome §3 / Gate;
`reference/proposal-next-iteration-icm-workflows.md` §9.8 (disagreement
classification), §9.9 (dimension definitions), §22.2 (success criterion).

Per §9.9 this scorecard publishes **no single accuracy number**.

---

## 0. Scope discipline — the one rule that governs every number below

The generated run covered **16 of 136** `decompose`-dispositioned files.
Everything is reported twice where it matters: as **raw coverage** (honest
absence-of-attempt over the whole 136-file set) and as **within-covered-scope
quality** (the 16 files both sides actually hold). These are never averaged
and never conflated.

| Scale | Generated | Reference |
|---|---:|---:|
| `decompose` files reached | 16 / 136 (11.8%) | 136 / 136 (by construction) |
| Behavior units, whole corpus | 108 | 979 |
| Behavior units, **16-file covered scope** | 108 | **165** |

The covered-scope reference slice was re-derived independently for this
adjudication (filter `reference-corpus/behavior-units/P*.ndjson` on
`source.path` ∈ the 16 paths in
`20-harvest/output/behavior-units.ndjson`): **165 units**, per-file counts
identical to C1 §2's table (`AGENTS.md` 67, `README.md` 36,
`bin/sgt-interactive-worker` 10, `bin/sgt-cleanup` 8, `bin/sgt-dispatch` 6,
`bin/sgt-respond` 6, `bin/sgt-watch` 6, `bin/sgt-recover` 5, `bin/sgt-wake`
5, `bin/sgt-ack-response` 4, `bin/sgt-notify` 4, `bin/sgt-drain-force` 3,
`bin/sgt-drain` 2, `bin/sgt-dag-dispatch-hook` 1, `bin/sgt-dag-run` 1,
`bin/sgt-undrain` 1). C1 and C2 are numerically reconciled: they analyzed the
identical 165-unit slice; only the file *label* differed (see D-1).

---

## 1. Adjudication of comparer disagreements

Six points where C1, C2 and C3 disagree, hedge, or where one is simply
wrong. §9.8's six categories describe generated-vs-reference disagreements;
where a disagreement is instead *between comparers*, it is labelled
**comparer error** and marked as outside §9.8's taxonomy.

### D-1 — "16 files" (C1, C3) vs "18 files" (C2). **C1 is correct: 16.**

Verified four independent ways:

1. `20-harvest/output/behavior-units.ndjson` — 108 records, exactly **16**
   distinct `source.path` values (re-enumerated for this adjudication).
2. `30-normalize/output/behavior-units.normalized.ndjson` — identical
   16-path / 108-unit breakdown.
3. `10-inventory/output/inventory.md` — `bin: fleet dispatch & lifecycle
   (14)` plus exactly two `decompose` root files (`AGENTS.md`, `README.md`;
   `.gitignore`/`Dockerfile.test` are `helper-evidence`, `LICENSE` is
   `reference-only`) = 16.
4. **The run's own adversarial-review stage counted 16.**
   `80-adversarial-review/output/review-summary.md` Axis 2: "spanning all
   **16** distinct `source.path` values." So does
   `90-reconcile/output/measurement-package.md`'s own "Behavioral precision"
   section — in the *same document* whose "Extraction coverage" section says
   18.

C2 inherited "18" from `measurement-package.md` without checking. C2's
substantive analysis is unaffected (it filtered on the correct 16-path list
and reached the correct 165-unit slice); only its labels are wrong.

**§9.8 classification of the underlying run defect: generator invention, at
the measurement layer.** A headline coverage figure (18, and its complement
118) is asserted where the run's own printed file list and its own review
stage both say 16 / 120. It is not a behavior-unit invention — no citation
is fabricated — but it is an unsupported number in the run's principal
measurement artifact.

### D-2 — C1 §4.1's "16/108 records carry a data-integrity defect". **Overturned. C1 is wrong; the run is conformant.**

This is the most consequential adjudication in the exercise, because C1's
verdict propagates into C3 §1 (provenance caveat), C3 §4 (first "missed
defect"), and both comparers' summary tables.

C1 checked `sha256(source.quote) == source.quote_hash` and found 16 failures,
all with a `quote` of exactly 500 characters, and called it "a systematic
500-character cap applied … sometime after `quote_hash` was computed."

`.sergeant/workflows/repo-to-icm/_config/evidence-policy.md` line 44 states
the rule the run was authored against, verbatim:

> `source.quote` — The quoted source text, **verbatim**, ≤500 characters. **A
> longer span records its first 500 characters here plus a `span_bytes`
> count; the hash still covers the full span.**

and line 45: "hash the literal bytes you are about to put in `quote` (**or the
literal bytes of the full span, if `quote` is a 500-char prefix**)."

Re-verified directly for this adjudication against
`reference/sergeant-upstream`:

- All **16** flagged records — and only those 16 of the 108 — carry the
  required `source.span_bytes` field (values 502–1041).
- Locating each record's stored 500-char prefix in its cited file, taking
  `span_bytes` bytes from that offset, and hashing gives an exact match to
  the stored `quote_hash` for **16/16**, zero failures.
- The other 92 records hash directly from `quote`. Total: **108/108 records
  verify against `_config/evidence-policy.md`.**
- Independently confirmed alongside C1's own uncontested result: **108/108
  quotes are literal contiguous substrings** of their cited file at the
  pinned SHA. Zero fabricated citations.

C1 applied a check its own subject explicitly rules out for long spans, then
criticized `80-adversarial-review` for using the *correct* method — the
review's stated procedure ("re-locating the full span from the 500-char
prefix and hashing the complete span, not just the stored prefix") is exactly
what the policy prescribes and exactly what reproduces here. C1's
"14.8% data-integrity defect", its claim that the run's `29/29` precision
self-report "overstates", and C3 §4's first missed-defect bullet are all
**struck**.

**Classification: comparer error (outside §9.8). No generated-run defect
exists here.** Behavioral precision is clean on both axes.

### D-3 — Over-staging vs A4 (C2 §3). **Downgraded from C2's framing.**

C2 finds `standard-task-workflow`'s `20-check-queue` and
`30-reconcile-existing-state` over-staged, measured against
`reference-corpus/adjudication-round1.md` ruling A4 ("demote by default";
verified — A4 folds `02-check-queue`→`03-choose-mode` and
`04-reconcile-state`→`05-confirm-decisions` in `task-intake-and-route`, and
the same two checkpoint types in `direct-implementation` and
`route-review-findings`). C2's finding is factually right.

But A4 is a *post-hoc adjudication that cut the reference's own first draft*.
The reference's original drafters, reading the same `AGENTS.md` nine-step
list, made the identical call the generator made. A generator reproducing the
gold's pre-adjudication shape is not making an error the gold avoided — it is
making an error the gold made and then caught.

**§9.8 classification: legitimate alternate decomposition, refuted by a
documented ruling.** No safety/identity/recovery/delivery consequence. Action
for the next workflow version: the demote-by-default rule belongs in
`_config/icm-ladder.md` as authoring guidance, not in a comparison scorecard
as a generator defect.

### D-4 — C2 §2.1 (`dispatch-mode` boundary gap "mostly coverage") vs C1 §3 (recall misses in the same files). **Both correct; not a conflict.**

C2 measures whether the reference's *package* shape was reachable (it largely
was not — 50 of `dispatch`'s 69 provenance citation-rows come from
`skills/dispatch/SKILL.md`, `docs/`, `tests/`, `bin/_sgt-intent.sh` etc., all
outside the 16-file scope). C1 measures whether the *behaviors inside the
reached files* were extracted (47.3% were). A run can hold 14 fleet-lifecycle
files, extract half their behaviors, and still be unable to reconstruct
7 reference packages whose defining evidence lives elsewhere. Both are
recorded; neither is discounted by the other.

### D-5 — §22.2. Only C1 addresses it; C2 and C3 are silent. **Adjudicated in §3 below.**

### D-6 — C3 §4's second "missed defect" (coverage arithmetic). **Upheld, and sharpened.**

The review stage *had the right number* (16, Axis 2) and never reconciled it
against the upstream artifact's 18. That is a genuine, cheap,
reference-independent miss — and now the only one, since D-2 removes the
other. C3's review-convergence verdict stands on one leg instead of two, but
it stands.

---

## 2. The §9.9 dimensions

### Source coverage

*What proportion of behavior-bearing source regions produced at least one
traceable behavior unit?*

| | Value |
|---|---|
| Raw | **16 / 136** `decompose` files (11.8%). 120 files never opened. |
| Within covered scope | **16 / 16** files yield ≥1 traceable unit — no zero-unit file. |
| Within-file evenness | Uneven. Ref-behavior recovery ranges 0% (`bin/sgt-dag-run`, 1 ref unit) and 13% (`bin/sgt-cleanup`) to 100% (`bin/sgt-ack-response`, `bin/sgt-dag-dispatch-hook`, `bin/sgt-undrain`). |

**Honesty of the coverage gap:** the 120 unreached files were recorded as
not-reached, not silently dropped — but see §4: the artifact that recorded
them (`20-harvest/output/coverage-note.md`) was destroyed by the run's own
finalize step and is **not** recoverable, contrary to what
`measurement-package.md` and `run-manifest.md` both assert.

Sub-file coverage is also uneven in a way the file count hides:
`README.md`'s entire orientation/genesis region (`BU-P1-062`–`BU-P1-067`) and
`AGENTS.md`'s six-rule procedural-skills routing table
(`BU-P1-132`–`BU-P1-137`) produced nothing, while later sections of the same
files were extracted — the turn budget ran out *inside* files, not only
between them.

### Behavioral recall

*How many reference behaviors were recovered, regardless of exact workflow
grouping?*

**Within covered scope: 78 / 165 (47.3%). 87 missed.** C1's matching method
(max of quote-text and statement word-overlap, Jaccard ≥ 0.25, hand-checked
at the boundary, many-to-one counted as recovered per §9.9's "regardless of
exact grouping") is accepted; its per-file totals reproduce against the
independently re-derived 165-unit slice.

| Sub-scope | Ref units | Recovered | Recall |
|---|---:|---:|---:|
| `AGENTS.md` + `README.md` | 103 | 53 | 51.5% |
| 14-file `bin` fleet partition | 62 | 25 | 40.3% |
| **All 16 covered files** | **165** | **78** | **47.3%** |

Raw recall against the full corpus (78/979 = 8.0%) is reported once here and
never used again — it measures coverage, not decomposition quality.

**§9.8 classification of the 87:** generator miss. Not ambiguous source (the
reference units cite the same passages the generator read), not legitimate
alternate (these are absences, not different boundaries). The turn-budget
wall is the proximate cause for the two root files; for the `bin` partition
the cause is representational (see below) — units were extracted but the
finer safety guarantees inside those scripts were not.

### Behavioral precision

*How many generated units were supported by source evidence rather than
invented from generic priors?*

**Clean, on every axis measured.**

| Check | Result |
|---|---|
| Quote is a literal contiguous substring of the cited file at the pinned SHA | **108 / 108** |
| `quote_hash` verifies per `_config/evidence-policy.md` (92 direct + 16 full-span with `span_bytes`) | **108 / 108** |
| Hand-read `statement`-vs-`quote` fidelity (C1 §4.2 + §4.3, 38 units incl. 6 unmatched "extras") | **0 inventions** |
| Provenance `behavior_id` citations resolving with matching representation/workflow/stage | **16 / 16** (C3 §1, re-confirmed) |

**The 45 generated units unmatched by any reference unit** (per-file counts
in C1 §3.3) are, on inspection, real guard clauses and preconditions the
reference simply did not extract at that granularity — e.g. `BU-0059`
(cleanup task-id rejected if empty / `.` / `..` / absolute / containing a
separator, restating the literal `case "$TASK_ID" in ""|"."|".."|/*|*/*)`
guard), `BU-0060`–`BU-0062`.

**§9.8 classification of the 45: legitimate alternate decomposition**, not
generator invention. Zero units in this run meet the invention bar.

### Workflow-boundary agreement

*Similar reusable outcomes, or mirrored source-file boundaries?*

| Generated | Reference | Verdict |
|---|---|---|
| `standard-task-workflow` | `task-intake-and-route` | **Strong genuine agreement.** Both independently recovered `AGENTS.md`'s nine-step structure by step number (`BU-0011`/`0012`/`0013`/`0016` ↔ `BU-P1-026`/`027`/`029`/`034`). No file-boundary shortcut available — both sides read one document. |
| `ship-with-no-mistakes` | `validate-and-ship` (+ `route-review-findings`) | **Partial, real.** From `README.md` prose alone, blind, it named `start-run`, `drive-gates`, `finish-run` — three of `validate-and-ship`'s own post-adjudication stage names — and independently identified `route-findings` as a separate concern, which the reference spins out as its own package. The other two-thirds of `validate-and-ship` is evidenced entirely outside the 16-file scope. |
| `dispatch-mode` (1 package, 1 stage) | `dispatch` + 6 sibling packages | **Major under-recognition.** Mostly coverage (D-4). Within-scope residue, attributable to the run's own actors: having read all 14 fleet-lifecycle files, it never separated "dispatch a worker" from "operate / recover / drain / reconcile an already-dispatched worker". |

One further boundary failure sits a tier down and is *not* coverage-explained:
`50-synthesize`'s 15 `shared-helper` groupings map essentially one-to-one onto
the 14 covered `bin/` files (`respond`: `BU-0079`–`BU-0084`, all from
`bin/sgt-respond`; `drain`: `BU-0064`–`BU-0070`, all from `bin/sgt-drain`).
That is precisely the "merely mirror source file boundaries" failure mode
§9.9 names, reappearing at the helper tier downstream of the
misclassification below.

**§9.8:** `dispatch-mode`'s under-recognition — generator miss (within-scope
portion) + out-of-scope coverage gap (the rest). The two name-level
convergences — genuine agreement, and the strongest positive result in the
run.

### Stage-boundary agreement

*Meaningful durable checkpoints, over-fragmentation, or collapse into prose?*

- **Under-fragmentation: none found.** No case of independently measurable
  boundaries collapsed into prose.
- **Over-staging: 2 of the 6 generated `stage` records** (`20-check-queue`,
  `30-reconcile-existing-state`) — measured against A4, adjudicated at D-3 as
  legitimate alternate, refuted by ruling.
- **The other 4 stages hold.** `10-load-context`, `50-reconcile-and-deliver`,
  `40-validate`, and `dispatch-mode`'s `10-dispatch-worker` all name
  checkpoints the reference keeps as real stages after A4.
- **One clean cross-workflow leak, self-flagged:** `40-validate` is
  `validate-and-ship` material (`BU-P1-042`/`043`) placed inside
  `standard-task-workflow`. `50-synthesize/output/candidates.md` recorded the
  uncertainty at the time ("cannot pin an exact step number … a judgment call
  `80-adversarial-review` should be free to challenge") and `60-draft`
  carried it through unresolved rather than hiding it. **§9.8: legitimate
  alternate, honestly flagged.**

### Representation agreement

*Did it distinguish permanent instructions, workflows, stages, actor context,
helpers, shared content, obsolete mechanisms, and engine gaps?*

**The dominant disagreement in the run.** Both distributions re-counted
independently for this adjudication; C2 §4.1's table reproduces exactly.

| Representation | Generated (108) | Reference, same 16 files (165) | Ratio |
|---|---:|---:|---|
| `agents-invariant` | 13 (12.0%) | 33 (20.0%) | 0.6× |
| `workflow` | 1 (0.9%) | 5 (3.0%) | 0.3× |
| `stage` | 6 (5.6%) | 34 (20.6%) | 0.27× |
| `stage-context` | 9 (8.3%) | 41 (24.8%) | 0.34× |
| `shared-context` | **0** | 22 (13.3%) | — |
| `shared-helper` + `helper` | **79 (73.1%)** | 13 (7.9%) | **9.3×** |
| `obsolete-mechanism` | **0** | 13 (7.9%) | — |
| `engine-gap` | **0** | 4 (2.4%) | — |

**Mechanism, identified precisely by C2 §4.2 and confirmed here.** The
classify stage answered §6.5's question ("is this deterministic machinery
subordinate to some checkpoint?") on the fleet-lifecycle scripts *without
first clearing §6.3's* ("would this checkpoint survive reimplementation?").
Same sentence, same file, same documented ladder, opposite answer:

- `BU-0075` (`bin/sgt-recover`, `shared-helper`, "deterministic precondition
  check … independent of caller workflow") ↔ `BU-P6-071` (`stage`,
  `stall-recovery`/`attempt-recovery`, "operators would want measured … matches
  §6.3").
- Same pattern at `BU-0079` ↔ `BU-P6-078`, `BU-0090` ↔ `BU-P6-101`,
  `BU-0064`/`0071`/`0072` ↔ `BU-P6-039`/`BU-P6-015`.

**§9.8: generator miss — a single systematic rung-ordering error, not 79
independent judgments.** It is the root cause of four other dimension
results: the `stage`/`stage-context` deficit, the file-shaped helper
groupings, `dispatch-mode`'s failure to split operate-from-dispatch, and the
entire engine-gap result below.

`shared-context` 0 and `obsolete-mechanism` 0 are separate, additive misses —
two representation tiers never used at all. The `obsolete-mechanism` zero is
partly upstream: `10-inventory` dispositioned **0** files
`obsolete-candidate`, so the classify stage was never primed to look.

### Engine-gap quality

*How many proposed engine features survived lower-rung refutation and
independent review?*

Two axes, deliberately not averaged.

**Subject-domain claims: 0 produced, against 4 reachable.** The reference's
4 in-scope `engine-gap` units are `BU-P6-099` (`bin/sgt-wake` → G1),
`BU-P6-016` (`bin/sgt-dag-dispatch-hook` → G2), `BU-P6-017` (`bin/sgt-dag-run`
→ G2), `BU-P6-063` (`bin/sgt-drain` → G4) — i.e. the source material behind
**3 of the reference's 4 surviving domain claims** sat inside this run's
reach. The run read all four files and extracted 20 units from them
(`BU-0095`–`0099`, `BU-0051`–`0053`, `BU-0054`–`0058`, `BU-0064`–`0070`) — and
classified **all 20** `shared-helper`.

Sharper than C3 stated: this is a **classification** failure, not an
extraction failure. Two of the four underlying behaviors *were recovered as
behaviors* — C1's match list pairs `BU-P6-016→BU-0052` and `BU-P6-099→BU-0097`
(the latter: "once that cap is exceeded, the worker escalates to needs_input
instead of continuing to retry indefinitely" — G1's exact seam, described
accurately and then filed as machinery). The §6.7 question was simply never
asked of them.

**Workflow-mechanics claims: 2 produced, 0 adversarially reviewed.** Both
`90-reconcile/output/grammar-pressure.ndjson` records carry all six
`docs/icm/record-shapes.md` §5 fields, name real ladder rungs, give
rung-specific failure reasons, and contain no "would be convenient" language —
template-conformant. But each rests on a single self-generated citation, and
`80-adversarial-review` runs *before* `90-reconcile`, which authored them. Its
Axis 3 correctly reported itself "applied vacuously." Nothing has tried to
refute either. `grammar-pressure-report.md` does that; claim 1 does not
survive.

**§9.8: generator miss** (domain axis) — the specific miss being the failure
to route any unit to §6.7 at all.

### Provenance completeness

*Can every artifact be traced to units, and every source-cited behavior be
reopened at the pinned revision?*

**Clean — the run's strongest dimension.**

- 16 / 16 `behavior_id` citations across all three `provenance.md` files
  resolve, with `representation`, `workflow` and `stage` fields matching
  what `40-classify/output/classifications.ndjson` actually holds.
- 0 `[S8]` defects across all four validator runs.
- 108 / 108 quotes reopenable verbatim at the pinned SHA
  (`f430cfd4f90174a98adbd7abebbece6303817929`).
- Uncovered ground is *declared*, never papered over:
  `dispatch-mode/provenance.md` states its workflow unit's monitoring clause
  has no stage evidence anywhere in the corpus;
  `standard-task-workflow/provenance.md` names steps 3, 6, 8 as unevidenced;
  `ship-with-no-mistakes/provenance.md` states under its own heading that 7
  unattached `stage-context` records are "the entirety of this candidate's
  direct evidence."

Depth is thin against the reference (2 citations for `dispatch-mode` vs 69
provenance rows for `dispatch`) — that is the coverage/boundary finding
already booked above, not a completeness defect. **No double-counting.**

With D-2 struck, C3 §1's hash caveat is withdrawn: nothing degrades
reopenability.

### Draft validity

*Do generated packages pass the structural validator without manual repair?*

**0 / 3 pass. 4 / 4 trees FAIL** (the three candidates plus `repo-to-icm`'s
own tree). C3 independently re-ran `validate-structure.py` and reproduced
`70-lint/output/lint-report.md` exactly — code for code, message for message.
The run's self-report on this dimension is accurate.

| Tree | Defect |
|---|---|
| `dispatch-mode` | `[S12]` closing stage `10-dispatch-worker` names no finalize step, though outputs are declared |
| `standard-task-workflow` | `[S12]` same, `50-reconcile-and-deliver` |
| `ship-with-no-mistakes` | `[S3]` empty `workflow.stages` — deliberate, corpus supports zero `stage` records |
| `repo-to-icm` (own tree) | `[S10]` `20-harvest/quote.sh` unclassified executable |

All four are correctly classified **substantive, not mechanical** (0
mechanical repairs found or applied anywhere), and all are disclosed inside
the files that fail. `[S3]` is the *right* failure — refusing to invent stage
boundaries to obtain a passing shape.

**`[S12]` is not a generator defect and not an engine gap** — see
`grammar-pressure-report.md` GP-5. It is an authoring-guidance hole: the
exemplar `repo-to-icm` tree satisfies S12 (`90-reconcile/CONTEXT.md` step 4
names `finalize.py`), while
`60-draft/references/draft-package-template.md` — the template handed to the
drafting actor — instructs it to declare artifacts and dispositions (the half
that *triggers* S12) and never mentions the finalize step (the half that
*satisfies* it). The validator is also absent from `60-draft`'s Inputs table.

### Review convergence

*How many adversarial findings remain unresolved, and what recurs?*

**Literal: 0 findings, 0 accepted, 0 rejected, 0 parked.** Vacuously clean —
reconciliation received nothing.

The pass was real, not skipped: 29 citations re-hashed across all 16 source
files (using the *correct* long-span method — see D-2), 16 provenance ids
re-confirmed, §6.3 re-applied to all 6 `stage` records, a literal
`reference-corpus` grep over every artifact with all 3 hits individually
classified as blindness-attestation prose.

Three structural limits let a zero pass through:

1. **Axis 2's over-staging test is one-directional.** The checklist reads
   "For every `representation: stage` record, re-apply the reimplementation
   test" — promotions only. The 79 `shared-helper` records were never asked
   the same question in the demotion direction. That is a pure methodology
   check needing nothing from `reference-corpus/`, and its absence is exactly
   what let the run's largest disagreement pass unflagged.
2. **Axis 3 is refutation-only.** "For every `representation: engine-gap`
   record…" — with zero such records it is structurally vacuous. There is no
   dual check asking whether anything *should* have raised §6.7.
3. **No cross-artifact consistency axis.** The review's own Axis 2 wrote
   "16 distinct `source.path` values" while the upstream artifact it had open
   said 18; nothing required it to reconcile the two (D-6).

**Unowned findings: 5.** `[S12]`×2, `[S3]`, `[S10]`, plus the
`10-inventory`→`contract.md` accuracy discrepancy
(`bin/__pycache__/sgt-callbackcpython-312.pyc` present, contradicting
`contract.md` §3). All were correctly generated, correctly classified, and
correctly not re-litigated by `80-adversarial-review` — and then fell outside
`90-reconcile`'s accept/reject/park mechanism, which by its own scope applies
only to `findings.ndjson`. They exit the pipeline with **no disposition
record anywhere.** That is a workflow-shape defect: the only channel into
adjudication is a file the lint stage cannot write to.

---

## 3. §22.2 success criterion — answered explicitly

> *No reference behavior with a confirmed safety, identity, recovery,
> delivery or human-decision consequence is silently absent from the final
> adjudicated output.*

**Answer: NO — the criterion is NOT met within covered scope.**

Two things must both hold for a miss to be "silent": the behavior is absent,
and nothing in the run records the absence. Both hold here. The run's
ambiguity roll-up confirms **no** stage output ever carried
`# AMBIGUOUS — NOT RESOLVED`; no miss below is recorded as a gap, a `gold
miss`, a low-confidence unit, or a disputed citation; and
`measurement-package.md` reports "0 zero-unit files among the reached" with no
per-file completeness caveat, which reads as covered.

Verified absences with §22.2-class consequence, keyword-scanned across all
108 generated `statement`/`trigger`/`outcome`/`scope`/`authority`/`notes`
fields (not only the threshold match):

| # | Reference behavior | Consequence class | Ref id(s) | Evidence of absence |
|---|---|---|---|---|
| 1 | Force-stop / cleanup verify the recorded process **start time** still matches before signalling, so a reused PID is treated as already-gone rather than killed | safety, identity | `BU-P6-040`, `BU-P6-138` | 0 hits for start-time / PID-reuse in the 2 `sgt-drain-force` + 5 `sgt-cleanup` units. *Mitigating:* `BU-0068` (`bin/sgt-drain`) does capture the identity concept for drain-wait classification — the kill-safety guarantee itself is absent |
| 2 | Cleanup refuses to retire fleet state while an unfinishable response is outstanding, so retiring never silently discards evidence | safety, delivery | `BU-P6-139` | absent from `BU-0059`–`BU-0063` |
| 3 | A durable, resumable cleanup-phase record is published *before* removal begins, so an interrupted cleanup is safely retryable | recovery | `BU-P6-140` | absent from the same 5 units |
| 4 | A `done` transition with an empty worktree result file is refused and reclassified `orphaned` | recovery, delivery | `BU-P6-103` | 0 orphan-classification hits in `BU-0090`–`BU-0094` |
| 5 | Interactive-worker exit is `orphaned` unless genuinely terminal with substantiating evidence | recovery | `BU-P6-115` | 0 hits in `BU-0100`–`BU-0104` |
| 6 | A worker resumed by response / recovery / wake runs the **exact pinned model-variant tuple**; a tuple the resuming harness cannot honor is terminal failure, never silent fallback to the ambient default | identity | `BU-P6-108` | `BU-0100` covers harness-registry validation at *dispatch*, not the resume-time re-honoring guarantee |
| 7 | Notification action-lease convergence through the one shared finalizer, on the agent's own completion proof, before recovery/response proceeds; settled at a single unified exit boundary across **every** terminal status | delivery | `BU-P6-073`, `BU-P6-079`, `BU-P6-113`, `BU-P6-114` | *Partially* present: `BU-0102` covers lease settlement on the cooperative-drain exit path only. The "single unified boundary across every terminal status" and the pre-action convergence gate on `sgt-recover`/`sgt-respond` are absent |
| 8 | Wake conditions distinguish "unmet" (may still resolve) from "escalate" (permanently unsatisfiable — retrying would be dishonest) | human-decision, recovery | `BU-P6-098` | 0 hits for `unmet` anywhere in the 108 |
| 9 | Standing authorization removes repetitive dispatch confirmation but **never** authorizes risk acceptance, gate skipping, force operations, secret exposure, or destruction of preserved state | safety, human-decision | `BU-P1-050` | 0 hits for any of the five named terms across all 108 units |
| 10 | Direct mode never edits several repositories in one checkout, nor bypasses repository instructions, task ownership, review independence, or shipping gates | safety, human-decision | `BU-P1-016` | 0 hits; `BU-0014` (confirm-on-risk) is adjacent but covers *when to ask*, not the scope guardrail |
| 11 | Dispatch rollback kills only a coordinator pane the invocation itself created, never a pre-existing selected pane, and disarms once every repo has dispatched | safety | `BU-P6-127` | 0 hits for rollback / disarm / pre-existing-pane in `bin/sgt-dispatch`'s 10 units |

**Eleven reference behaviors across all five named consequence classes are
silently absent inside the 16 files this run actually read.** Items 9 and 10
are the sharpest: both are one-sentence `AGENTS.md` guardrails in a file the
run extracted 23 units from.

Scope note held: **the 120 unreached files are excluded from this finding.**
Their absence is coverage, declared as such, and is not counted here.

Adjacent §22.2-relevant finding, recorded separately: the run's honest
coverage record was **destroyed by its own pipeline** — see §4. That does not
make a *reference behavior* silently absent, but it removes the primary
evidence that the coverage gap was declared at all.

---

## 4. Finding outside the comparers' scope: the coverage note is gone

All three comparers cite `20-harvest/output/coverage-note.md`. It does not
exist. `scripts/finalize.py` removed it because
`20-harvest/output/README.md` declares only `behavior-units.ndjson` as
`promote`.

`measurement-package.md` says its content is "recoverable from Work-branch
history regardless, per D9 convention"; `run-manifest.md` repeats it. **Both
are false.** Verified on the run's own branch
(`sergeant/01KZNW46C3Y2W890DE7S8M94NZ` in the retained work repo): the branch
holds exactly two commits, and the finalize commit `b1546e9` is
`40 files changed, 2721 insertions(+)` with **zero deletions** —
`coverage-note.md` appears in no tree at any revision. It was deleted from an
uncommitted working tree, so the "removable from history" story never applied.

Consequence: the file that recorded which 120 files were not reached, and that
is the **sole `source_evidence` citation** for grammar-pressure claim 1, is
unrecoverable. Full detail and the §6.7 treatment are in
`grammar-pressure-report.md` GP-5.

---

## 5. §22.2's reporting checklist

| Item | Value (within-covered-scope unless noted) |
|---|---|
| Reference behaviors total | 979 corpus-wide; **165** in covered scope |
| Matched behaviors | **78** (47.3% of in-scope) |
| Missed behaviors | **87**, of which **11** carry §22.2 consequence |
| Extra unsupported behaviors | **0** invented. 45 unmatched, all source-supported → legitimate alternate |
| Workflow-boundary agreements / disagreements | 2 genuine (1 strong, 1 partial) / 1 major under-recognition (7 ref packages → 1), mostly coverage-explained |
| Stage-boundary agreements / disagreements | 4 of 6 agree / 2 over-staged vs A4 (legitimate alternate, refuted by ruling); 0 under-fragmentation; 1 self-flagged cross-workflow leak |
| Representation agreements / disagreements | `shared-helper` **9.3×** over (73.1% vs 7.9%); `stage`+`stage-context` **0.31×** (13.9% vs 45.4%); `shared-context` 0/22; `obsolete-mechanism` 0/13; `engine-gap` 0/4 |
| Engine-gap agreements / disagreements | Domain: **0 generated / 4 reachable** (3 of the reference's 4 survivors). Mechanics: 2 generated, 0 reviewed, **1 rejected on adjudication** (GP-1) |
| Unresolved review findings | 0 raised. **5 known defects exit with no disposition** (`[S12]`×2, `[S3]`, `[S10]`, contract/inventory discrepancy) |
| Coverage (reported, never averaged in) | **16/136 files (11.8%)**; 120 unreached |

### §9.8 classification roll-up

| Category | Count | Instances |
|---|---:|---|
| Generator miss | 4 classes | 87 unmatched ref units (11 with §22.2 consequence); the systematic §6.3-before-§6.5 rung-order error behind 79 `shared-helper` records; `shared-context`+`obsolete-mechanism` tiers unused; 0 domain engine-gap claims from 4 reachable sources |
| Generator invention | 1 | Coverage headline "18 files / 118 not reached" unsupported by the run's own file list and its own review stage (D-1). **Zero at the behavior-unit level** |
| Gold miss | 0 | None found. The 45 unmatched generated units are finer-grained, not corrections to the gold |
| Legitimate alternate decomposition | 3 | The 45 unmatched generated units; `20-check-queue` + `30-reconcile-existing-state` over-staging (refuted by A4, no consequence); `40-validate`'s cross-workflow placement (self-flagged) |
| Ambiguous source | 0 | Every disagreement examined resolved to one of the above |
| Genuine engine pressure | 1 of 2 claimed | GP-2 (no actor-initiated ask) confirmed. GP-1 (harvest fan-out) **rejected** — unattempted rung. GP-5b (finalize destroys uncommitted evidence) is a helper-semantics defect, not a new durable fact. See `grammar-pressure-report.md` |
| *Comparer error (outside §9.8)* | 1 | C1 §4.1's 16/108 "hash defect" — struck at D-2; the records are policy-conformant |

---

## 6. Verdict

**Within covered scope**, this run is **honest and precise but shallow and
systematically mis-tiered.** Precision is flawless (108/108 citations
verbatim and hash-valid, zero inventions); provenance is exact (16/16);
uncovered ground is declared rather than invented; two workflow boundaries
were independently rediscovered blind. Against that: fewer than half the
reference behaviors in the files it read were recovered, one rung-ordering
error routed 73% of the corpus to `shared-helper` and took the entire
engine-gap dimension with it, no draft passes its own validator, and its
zero-finding review could not see any of it.

**§22.2 is not met.** Eleven reference behaviors with safety, identity,
recovery, delivery or human-decision consequence are silently absent from
inside the covered scope — including two one-sentence `AGENTS.md` guardrails
in a file the run extracted 23 units from.

**Coverage (16/136) is a separate, honestly-declared measurement** and is not
counted against decomposition quality anywhere above. Its own honesty record,
however, did not survive the pipeline (§4).
