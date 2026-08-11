# C3 — Provenance Completeness, Draft Validity, Review Convergence, Protocol Mechanics

Scope: `reference/proposal-next-iteration-icm-workflows.md` §9.9's
"provenance completeness," "draft validity," and "review convergence"
dimensions, plus the run-3-specific "protocol mechanics" axis (the
partition-checkpoint capability v1 lacked, per
`.sergeant/workflows/repo-to-icm/20-harvest/references/
partition-checkpoint-protocol.md`). Generated artifacts: the 18 packages
under `docs/gauntlet/runs/n2-run3/.sergeant/drafts/workflows/*/`;
`docs/gauntlet/runs/n2-run3/.sergeant/workflows/repo-to-icm/{20-harvest,
40-classify,70-lint,80-adversarial-review,90-reconcile}/output/*`.
Reference: `reference-corpus/FROZEN.md` v1. Prior run for delta:
`docs/gauntlet/runs/n2-run2/comparison-c3-validity.md`. Companion
documents from this run: `comparison-c1-coverage-recall.md`,
`comparison-c2-boundaries.md` — both already exist and are cited here by
section rather than re-derived.

**Ground rule applied throughout.** Per the run's own `partition-ledger.md`,
6 of 21 partitions (28 of 82 `decompose` files, `BU-0001`–`BU-0312`) are
`done`; the other 15 are honestly `pending`, not silently dropped. Every
verdict below about *quality* (provenance, review, sweep completeness) is
scoped to that 28-file covered surface. Raw coverage (34% of the corpus by
file count) is a separate axis, already scored in
`comparison-c1-coverage-recall.md` §0–§1 and not re-litigated as a defect
here — except where the covered-scope review process is asked to notice it
(§3 below), which is where it belongs on this axis.

---

## 1. Provenance completeness

**Method.** Sampled 7 of the 18 packages' `provenance.md` — exceeding the
≥5 requested — chosen to span package shapes: staged with high fan-in
(`dispatch-worker`, 47 supporting records), staged with a single stage
(`installation-and-setup`, `project-graphify`, `fleet-cleanup`,
`dag-orchestration`), a multi-stage sequential package
(`task-intake-and-execution`, 8 stages), and a zero-stage,
`representation: workflow`-anchored package (`sergeant-help-query`). For
each, traced the full chain: `provenance.md`'s cited `BU-####` ids →
`40-classify/output/classifications.ndjson` (representation/workflow/stage
fields match the claim) → `20-harvest/output/behavior-units.ndjson`
(`source.path`/`source.locator`/`source.quote`) → the actual file on disk
at that locator.

**Layer 1 — provenance.md ↔ classifications.ndjson.** All 7 stage
citations checked resolve exactly:

| Package | Stage citation | `representation`/`workflow`/`stage` in classifications.ndjson |
|---|---|---|
| `dispatch-worker` | `BU-0140` → `01-intent-file-gate` | `stage` / `dispatch-worker` / `intent-file-gate` — match |
| `dispatch-worker` | `BU-0284` → `02-create-td-tasks` | `stage` / `dispatch-worker` / `create-td-tasks` — match |
| `installation-and-setup` | `BU-0129` → `01-dependency-check` | `stage` / `installation-and-setup` / `dependency-check` — match |
| `project-graphify` | `BU-0250` → `01-publish-graph` | `stage` / `project-graphify` / `publish-graph` — match |
| `fleet-cleanup` | `BU-0171` → `01-cleanup-preconditions` | `stage` / `fleet-cleanup` / `cleanup-preconditions` — match |
| `dag-orchestration` | `BU-0203` → `01-stage-dependency-gate` | `stage` / `dag-orchestration` / `stage-dependency-gate` — match |
| `sergeant-help-query` | `BU-0124` → workflow-as-a-whole | `workflow` / `null` / `null` — match (record's own `workflow` field is `null` "per convention"; sourced by topic, correctly not claimed as a `stage` record) |

No mismatched representation, workflow, or stage field found in the
sample.

**Layer 2 — classifications.ndjson ↔ harvest source.** For the same 7 ids
plus `BU-0129`, resolved each to its `20-harvest/output/behavior-units.ndjson`
record's `source.path`/`source.locator`, then confirmed the source file is
one of the 28 covered files and sits inside the partition
`partition-ledger.md` marks `done`:

| Unit | `source.path` | Locator | Partition (ledger) |
|---|---|---|---|
| `BU-0140` | `docs/using-sergeant.md` | L112-117 | P3, done |
| `BU-0171` | `docs/using-sergeant.md` | L403-408 | P3, done |
| `BU-0250` | `bin/sgt-graphify` | L426-433 | P5, done |
| `BU-0129` | `docs/getting-started.md` | L51-53 | P3, done |
| `BU-0124` | `skills/sergeant-help/SKILL.md` | L30-44 | P2, done |
| `BU-0284` | `skills/dispatch/SKILL.md` | L198 | P6, done |
| `BU-0203` | `schema/project.yaml.example` | L117-120 | P3, done |

**Layer 3 — harvest ↔ actual file.** Spot-verified the strongest case
directly against the subject repository
(`/workspace/miztertea/sergeant/docs/using-sergeant.md`, the checkout this
run's `reference/UPSTREAM.md`-pinned subtree derives from): lines 112-117
read *"`--intent-file` is required when the objective names auth/OAuth,
security, secrets or credentials, payments, databases or migrations,
stateful/production work, destructive work, persistent state, or state
transitions..."* — the locator range is correct and the quote is a close
paraphrase of `BU-0140`'s `statement` field (not a fabricated citation; the
`quote` field itself, separately hash-checked at scale by
`80-adversarial-review`'s Axis 2 sample of 20 units across 12 files with
zero mismatches, is the field held to byte-exact reproduction — see §3).

**Verdict.** Provenance completeness holds up end-to-end on every sampled
package: no orphaned `BU-####` citation, no representation/workflow/stage
disagreement between a package's own `provenance.md` and the ledger it
claims to summarize, and no citation pointing outside the run's own
28-file covered scope. This matches `80-adversarial-review`'s own
independent, exhaustive check (not a sample): "parsed every `BU-####` id
referenced across all 18 candidates' `provenance.md` files (204 distinct
ids cited) and confirmed every one exists in
`../40-classify/output/classifications.ndjson`. Zero unresolved
citations" (`80-adversarial-review/output/review-summary.md`). The 7-package
hand trace corroborates that exhaustive machine check rather than merely
repeating it — it additionally confirms the *field-level* agreement
(representation/workflow/stage), which the exhaustive check's "id exists"
test does not by itself guarantee.

---

## 2. Draft validity

**Method.** Reran `.sergeant/workflows/repo-to-icm/scripts/
validate-structure.py` against all 18 packages under
`docs/gauntlet/runs/n2-run3/.sergeant/drafts/workflows/` (exceeding the
≥6 requested, including both zero-stage packages named in the prompt as a
floor). Draft mode (`PATH` argument): `index.md` status must be `draft`,
tree must live under `.sergeant/drafts/workflows/`, and (draft-only) every
workflow/stage must carry provenance.

**Result: 11/18 PASS, 7/18 FAIL — all 7 failures are `[S3]`, none is `[S12]`.**

| Package | Stages (per `provenance.md`) | `validate-structure.py` |
|---|---:|---|
| `dag-orchestration` | 1 | PASS |
| `dispatch-worker` | 6 | PASS |
| `fleet-cleanup` | 1 | PASS |
| `installation-and-setup` | 1 | PASS |
| `project-graphify` | 1 | PASS |
| `project-registration` | 1 | PASS |
| `review-finding-routing` | 2 | PASS |
| `shipping-gate-driving` | 1 | PASS |
| `task-intake-and-execution` | 8 | PASS |
| `validation-gate` | 2 | PASS |
| `worker-response-and-recovery` | 2 | PASS |
| `callback-delivery` | 0 | FAIL — `[S3] callback-delivery: workflow.toml has no non-empty \`workflow.stages\` array` |
| `cross-repo-planning` | 0 | FAIL — `[S3]` (same message) |
| `fleet-monitor-and-reconcile` | 0 | FAIL — `[S3]` |
| `sergeant-help-query` | 0 | FAIL — `[S3]` |
| `skill-adoption` | 0 | FAIL — `[S3]` |
| `troubleshoot-td-identity` | 0 | FAIL — `[S3]` |
| `undocumented-failure-escalation` | 0 | FAIL — `[S3]` |

The correlation is exact and total: every package with 0 materialized
stages fails `[S3]` (empty `workflow.stages`), every package with ≥1 stage
passes clean. No `[S1]`/`[S2]`/`[S4]`–`[S11]` defect fired anywhere across
all 18 runs (confirmed by grepping all 18 outputs for `\[S\d+\]`: the only
tag that appears, 7 times, is `[S3]`).

**Did v2's finalize-aware drafting close run 2's `[S12]`/`[S3]` failures?**

- **`[S12]` (declared outputs with no finalize step): closed, 0/18.** Run
  2 hit `[S12]` on 2 of its 3 packages (`dispatch-mode`,
  `standard-task-workflow` — `comparison-c3-validity.md` §2.1 in
  `n2-run2`, "outputs are declared but the closing stage that would
  finalize them is missing"). Across all 18 of this run's packages, `[S12]`
  fires zero times. `90-reconcile/output/adjudication-log.md`'s companion
  file, `output/measurement-package.md`, independently names this same
  gap as **closed by design**, not by luck: `60-draft`'s own template now
  requires a finalize-aware closing stage wherever a package declares
  outputs, and `70-lint/output/lint-report.md`'s per-package sections show
  every staged package's declared outputs resolving to a disposition. This
  is a genuine, structural fix — not merely an artifact of this run's
  smaller per-package output surface, since the staged packages here
  range from 1 stage (`fleet-cleanup`) to 8 (`task-intake-and-execution`)
  and all pass clean.
- **`[S3]` (empty `workflow.stages` for a zero-stage design): NOT closed,
  7/18 (39%), same defect class as run 2's 1/3.** Every one of the 7 is
  the *same* honest failure mode run 2's single `[S3]` case was: a package
  correctly declines to invent a stage boundary the corpus doesn't support
  (`.sergeant/workflows/repo-to-icm/_config/icm-ladder.md` bucket 3), and
  self-discloses this in its own `CONTEXT.md` — e.g. `callback-delivery/
  CONTEXT.md` §"Zero materialized stages": *"No `representation: stage`
  record ... carries this candidate's `workflow` value ... this is not
  resolved by inventing a stage: this package has no `NN-*/` directories
  and `workflow.toml` declares `stages = []`."* `70-lint/output/
  lint-report.md` correctly classifies all 7 as **substantive**, not
  mechanical (`grep`-confirmed: "1×S3 (zero-stage design)" recorded 7
  times, once per package), consistent with `70-lint`'s explicit,
  unfixed-by-repair verdict for each. Structurally this is unsurprising:
  fixing `[S12]` was a template/process change to how a *staged* package
  finalizes its own declared outputs; `[S3]` is a disagreement between the
  validator's baked-in assumption ("a workflow package has ≥1 stage") and
  the ICM ladder's own bucket-3 rule (a workflow boundary can be real and
  correctly drawn with zero materialized stages) — a different bug class
  entirely, and finalize-awareness in `60-draft` was never going to touch
  it.

**Verdict.** §9.9's literal question — "do generated packages pass the
structural validator without manual syntax repair?" — is now **yes for
11 of 18 (61%)**, up from 0 of 3 (0%) in run 2, and the specific defect
class run 2 failed on twice (`[S12]`) is fully closed. But the validator's
literal verdict on the full population is still **FAIL, 7/18**, for a
defect class (`[S3]`) that is honestly disclosed, correctly triaged by
`70-lint` as substantive, and — per §2.3 below — a validator/ladder
disagreement rather than a generation defect, but still an `[S3]` on the
delivered artifact by the same literal bar run 2 was held to.

---

## 3. Review convergence

**What `80-adversarial-review` found.** `findings.ndjson` records 5
findings (`AF-0001`–`AF-0005`), against run 2's 0. `90-reconcile/output/
adjudication-log.md` disposed all 5, independently re-verifying each
before dispositioning: 3 `park` (`AF-0001`, `AF-0002`, `AF-0005` — real,
correct, but requiring an upstream re-run or new judgment `90-reconcile`
cannot perform in place), 1 `accept` (`AF-0004`, rationale rewritten in
place), 1 `accept (partial)` (`AF-0003`, caveat added to the 2 of 4 named
targets capable of carrying one). **0 rejected — every one of the 5 held
up on independent re-verification**, confirmed here by re-deriving three
of them directly:

- `AF-0001`/`AF-0002` (corpus incompleteness): confirmed — `partition-ledger.md`
  rows P7-P21 are `pending`, `consequence-class-sweep.md` has exactly 28
  data rows (`grep -c '^|'` = 30, minus header/separator = 28), one per
  P1-P6 file, no row at all for the other 54 files (§1's method, cross-checked
  against `comparison-c1-coverage-recall.md` §0).
- `AF-0004` (circular `recover-worker` classification): confirmed —
  `grep '"stage": "recover-worker"'` combined with
  `"representation": "stage"` against `classifications.ndjson` returns
  zero matches, so `BU-0039`/`BU-0146`/`BU-0159`/`BU-0174`/`BU-0286`'s
  mutual "established more specifically elsewhere" rationale chain never
  terminates in an actual stage record.

**Grading the axis addition.** The 5 findings are real, non-duplicative,
correctly severity-ranked (3 high on the dominant scope defect, 2 medium),
and `90-reconcile` treated them honestly rather than rubber-stamping:
`AF-0003`'s repair explicitly declines to patch the two NDJSON ledgers
("inserting a non-conforming line would itself violate that shape"), and
`AF-0004`'s repair explicitly declines the more invasive fix its own
finding gestures at (promoting the cluster to `representation: stage`,
which would require re-running `50-synthesize`/`60-draft`) — both are
principled boundary calls about what a reconcile stage may touch, not
under-effort. This is a genuine capability run 2 did not have (0
findings, 0 unresolved — see `n2-run2/comparison-c3-validity.md` §"Review
convergence") and it correctly surfaced the run's single dominant defect
(corpus incompleteness) with severity matching its actual impact.

**What it still missed — C1/C2-class defects surviving inside the
reviewed 28-file scope**, none named in `findings.ndjson`:

1. **A confirmed false "swept, none found" cell**
   (`comparison-c1-coverage-recall.md` §4). `consequence-class-sweep.md`'s
   `schema/project.yaml.example` row records `swept, none found` for
   Delivery, but the file's own `graphify:` block comment (lines 88-93,
   *"sgt-graphify only replaces it after a complete run"*) is exactly the
   atomic-replace pattern `consequence-class-checklist.md`'s Hunt
   Question 4 asks for, and is precisely what the reference's `BU-P7-003`
   cites from this same file. This is the sweep step's own declared
   contract failing on a file that *was* read (§1's Layer 2/3 confirm
   `schema/project.yaml.example` sits in `done` partition P3) — a defect
   `80-adversarial-review`'s Axis 4 structural-self-consistency checks
   (which produced `AF-0001`/`AF-0002`/`AF-0003`/`AF-0005`, all
   arithmetic/count-based) had no method for catching, since it requires
   reading the swept file's own prose, not recomputing a count.
2. **A silent under-citation in the same sweep artifact**
   (`comparison-c1-coverage-recall.md` §3, item #10). `BU-0018`
   (`AGENTS.md` L38-41 — *"Never use direct mode to edit several
   repositories in one checkout, or to bypass repository instructions,
   task ownership, review independence, or shipping gates"*) is captured
   verbatim as a behavior unit, but `consequence-class-sweep.md`'s
   `AGENTS.md` row cites it in neither its Safety nor its Human-decision
   column, despite being a textbook fit for both.
3. **A representation-tier misclassification in `cross-repo-planning`**
   (`comparison-c2-boundaries.md` §2.1). `skills/cross-repo-work/SKILL.md`
   produced 6 units (`BU-0267`–`BU-0272`); only `BU-0267` was tied to the
   `cross-repo-planning` workflow (`stage-context`), and the other 5
   (`BU-0268`–`BU-0272`) were filed `agents-invariant` with rationale text
   that pattern-matches the sentence to an *existing* AGENTS.md-level rule
   ("the same shape as `BU-0029`," "the same shape as `BU-0018`" — the
   very unit missed in item 2 above) rather than treating it as
   procedure-specific judgment. This left `cross-repo-planning`'s own
   `provenance.md` citing exactly one record for the entire package — the
   same under-promotion failure shape as `AF-0004`'s `recover-worker`
   finding, in the same run, never caught.
4. **A missing dispatch-time unit in `bin/sgt-dispatch`**
   (`comparison-c1-coverage-recall.md` §3, "one new in-scope near-twin").
   The reference's `BU-P6-124` (dispatch-time model-tuple resolution
   precedence) has no counterpart anywhere in this run's `BU-0287`–`BU-0296`
   (`bin/sgt-dispatch`), and `consequence-class-sweep.md`'s Identity column
   for that file (`BU-0292, BU-0295, BU-0296`) doesn't cover it either —
   `consequence-class-checklist.md`'s own Identity hunt question
   explicitly warns against exactly this confusion (mistaking the
   resume-time check for the dispatch-time one).

**Why these survived: a real, quantifiable sampling blind spot, not bad
luck.** `review-summary.md`'s Axis 2 (invention) sampled 12
`helper`/`shared-helper` and 10 `stage` records for "rationale
discrimination" and re-applied the reimplementation test to 10 of the 31
`stage` records for over-staging — every sampled tier is one that was
already *promoted*. `agents-invariant` is the corpus's single largest
tier (106 of 312 records, 34% — `40-classify/output/
classifications.ndjson` representation counts:
`agents-invariant: 106, helper: 87, stage-context: 73, stage: 31,
shared-helper: 8, workflow: 3, shared-context: 2, engine-gap: 2`) and
received **zero** sampled checks for *under*-promotion in this review —
every finding-4-class defect above (items 2 and 3) is exactly an
`agents-invariant` record that should plausibly have been
`stage-context`. The review's own axes check "did a promoted record
deserve its rung" (over-staging, invention) and "is the corpus internally
consistent" (Axis 4, arithmetic-based); nothing checks "did a
*demoted*-to-invariant record actually belong one rung up," which is
precisely `AF-0004`'s own finding shape — the review caught one instance
of it (`recover-worker`, because its own rationale text was internally
circular and self-flagged) and missed the shape's other instances
(`cross-repo-planning`), which required reading the source file's content
against the corpus rather than checking the corpus's citations against
itself.

**Verdict.** Review convergence went from a null capability (0 findings,
untested against a corpus with a known-nonzero defect population — run
2's 3-package corpus turned out to have exactly 3 latent structural
defects and found none of them) to a real one: 5 correct, well-scoped,
honestly-adjudicated findings, 0 false positives, on a corpus roughly 3×
larger. But it is not close to exhaustive even within its own reduced
scope — the 4 items above are all silent, all inside the reviewed 28
files, and all in dimensions (`schema/project.yaml.example`'s sweep,
`agents-invariant`-tier under-promotion) the review's own axis design does
not reach.

---

## 4. Protocol mechanics — the partition-checkpoint capability, as run

**Is the resumable state real and correctly formed?** Yes, on every check
performed:

- **Contiguous, non-overlapping unit-id ranges matching the ledger.**
  Recomputed directly: `behavior-units.ndjson`'s 312 `id` values are
  exactly `BU-0001`–`BU-0312`, contiguous with no gaps or duplicates
  (`python3 -c "..."` over the file, verified). `partition-ledger.md`'s 6
  `done` rows (`BU-0001–BU-0060`, `BU-0061–BU-0128`, `BU-0129–BU-0213`,
  `BU-0214–BU-0234`, `BU-0235–BU-0266`, `BU-0267–BU-0312`) partition that
  exact range with no gap or overlap, and sum to 312.
  `consequence-class-sweep.md`'s file rows follow the identical order
  (`AGENTS.md`, then P2's 6 files, then P3's 6, ... through P6's 7),
  matching the ledger's declared partition order, and its row count (28)
  matches the ledger's declared file count for P1-P6 exactly.
- **Halted at a partition boundary, not mid-partition**, per the
  protocol's central rule ("Stop at a partition boundary, never
  mid-partition ... there is no `half-done`" —
  `partition-checkpoint-protocol.md` "Stopping honestly"). P6's row is
  fully `done` with a complete unit range and no partial notes; P7's row
  is fully `pending` with both `Unit id range` and `Notes` blank — no row
  shows a partial range or a "started, not finished" annotation, which is
  what a mid-partition stop would look like.
- **A single-attempt run, consistent with the protocol's single-turn
  path.** `run-manifest.md`'s journal event counts show exactly one
  `execution.started`/`execution.stopped` pair per stage (10/10, no
  retries anywhere in the run) — `20-harvest` reached `stage.completed`
  after one execution, having done 6 of 21 partitions' worth of work and
  stopped honestly rather than truncating coverage to force the ledger to
  look complete (the protocol's explicit anti-pattern: "do not silently
  truncate coverage by skipping straight to the last partition"). Because
  no retry was issued in this run, the ledger's fitness for an *actual*
  resume was not exercised end-to-end here — but its shape is exactly
  what `partition-checkpoint-protocol.md`'s "On stage entry" step 1 needs
  from a prior attempt ("read it as the authoritative record of what is
  already done... do not re-read or re-extract a `done` partition's
  files"): unambiguous `done`/`pending` status per row, no third state
  anywhere in the 21 rows, and exact unit-id ranges a resuming attempt
  could use to know where `behavior-units.ndjson` already stands.

**Are consequence-sweep rows complete for done partitions?** Yes,
structurally: 28 rows for 28 done-partition files, no row missing, no
blank cell (`swept, none found` used explicitly where a class was checked
and nothing matched, per the sweep's own header rule — confirmed by
inspection, no empty cell found across all 28×5 = 140 cells). This is a
*structural* completeness claim only, not a quality one: §3 above (items 1
and 2) found the sweep's *content* wrong in two places within this
same 28-row, 140-cell surface — a confirmed false `swept, none found`
(`schema/project.yaml.example`, Delivery) and a silent under-citation
(`AGENTS.md`, `BU-0018`). The protocol's structural contract ("a blank
cell is never left") held; the sweep's substantive contract (correctly
identifying which class each cited behavior belongs to, and citing every
behavior that belongs) did not, in at least these two cells.

**One process-design gap the protocol itself does not close, correctly
self-disclosed rather than hidden.** `partition-checkpoint-protocol.md`
gives `20-harvest` a mechanism to stop honestly mid-corpus, but nothing in
the workflow enforces that `30-normalize` onward must not proceed against
an incomplete ledger — `00-contract/output/contract.md` §5's own success
criteria ("bounded by, and done when, each stage's own CONTEXT.md-declared
durable outcome has been reached in sequence") was not met by `20-harvest`
(15 of 21 rows still `pending`), yet every subsequent stage ran as
ordinary work rather than treating the corpus as incomplete — exactly the
gap `AF-0001` names and `90-reconcile` correctly `park`s rather than
silently accepting. This is a real gap in the *protocol's* enforcement
reach (a checkpoint mechanism inside one stage, with no cross-stage gate
tied to it), not a bug in how this run executed the checkpoint mechanism
itself — the ledger this run produced is exactly the honest, correctly
shaped record the protocol asks for; what's missing is something to *read*
that ledger before `30-normalize` starts.

**Verdict.** The partition-checkpoint protocol, as run, produced real,
correctly-formed, honestly-halted resumable state — every structural
guarantee the protocol document promises (contiguous ranges, boundary-only
stops, no third status, sweep rows keyed exactly to done-partition files)
held on direct re-verification. Its untested edge is an actual resume
(no retry occurred in this run to exercise "read it as authoritative and
skip re-work"), and its known limitation — silently proceeding downstream
of an incomplete harvest — is a cross-stage gate the checkpoint protocol
was never designed to provide, correctly caught by `80-adversarial-review`
and `park`ed rather than swept under a "the ledger says pending, so it's
fine" reading.

---

## 5. Summary

| Dimension | Run 2 (v1) | Run 3 (v2) | Delta |
|---|---|---|---|
| Provenance completeness | Not separately measured at this scale | 7/7 sampled packages fully traced (unit → classification fields → harvest source → covered partition); 204/204 citations resolve corpus-wide (`80-adversarial-review`, exhaustive) | New, and clean on every check performed |
| Draft validity | 0/3 pass `validate-structure.py` (`[S12]`×2, `[S3]`×1) | 11/18 pass (61%); `[S12]` fully closed (0/18); `[S3]` persists at the same rate/reason (7/18, all honest zero-stage designs) | `[S12]` closed by finalize-aware drafting; `[S3]` is a different bug class (validator/ladder disagreement) the fix was never targeted at |
| Review convergence | 0 findings, 0 unresolved (untested capability) | 5 findings, 0 false positives (independently re-verified), 3 `park`/1 `accept`/1 `accept-partial`, all correctly scoped and severity-ranked | Real new capability, but misses 4 concrete in-scope C1/C2-class defects (a confirmed false "none found," a silent under-citation, an under-promoted workflow package, a missing dispatch-time unit) — all fall in the 106-record `agents-invariant` tier or the sweep's own prose-reading step, neither of which any review axis samples for under-promotion |
| Protocol mechanics | No equivalent capability (v1 had no way to checkpoint/resume a partial harvest) | Real: contiguous unit ranges, boundary-only halts, correctly-keyed sweep rows, honest `pending` disclosure — all structurally verified | New capability, structurally sound as run; untested for an actual resume (no retry occurred); does not by itself gate downstream stages on ledger completeness (correctly caught by `AF-0001`, `park`ed, not silently accepted) |

**Overall read.** Every dimension shows the same shape: v2 added a real
capability where v1 had none or failed outright, and each new capability
is honest rather than papered-over where it still falls short. The
recurring pattern worth carrying into the next round is that self-checking
mechanisms in this run (lint, adversarial review, the checkpoint protocol)
excel at *arithmetic* self-consistency (counts, ids, ranges all agree with
each other) and still miss *content* defects (a sentence a sweep should
have caught, a unit that should have been promoted a rung) that require
reading source prose against a corpus rather than recomputing the corpus
against itself.
