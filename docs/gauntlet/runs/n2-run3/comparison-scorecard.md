# N2 run 3 — Comparison Scorecard (generator v2 vs frozen reference corpus v1)

Adjudication over `comparison-c1-coverage-recall.md`,
`comparison-c2-boundaries.md`, and `comparison-c3-validity.md`.
Dimension definitions: `reference/proposal-next-iteration-icm-workflows.md`
§9.9. Disagreement taxonomy: §9.8. Success criterion: §22.2.

- **Run 3:** `docs/gauntlet/runs/n2-run3/` — generator v2, workflow
  `repo-to-icm` v2 (10 stages), Work `01KZQ32J2BAD4P8WJA9SWXRMZ9`, fake
  backend, 312 behavior units, 18 draft packages.
- **Reference:** `reference-corpus/`, `FROZEN.md` v1 — 979 units, 34 packages.
- **Run 2 (v1 baseline):** `docs/gauntlet/runs/n2-run2/comparison-scorecard.md`
  — 108 units, 16 files, 47.3% in-scope recall, 11 silent §22.2 misses.
- Both runs decompose the same pinned subject subtree
  (`reference/sergeant-upstream`, upstream SHA
  `f430cfd4f90174a98adbd7abebbece6303817929`), so unit-for-unit comparison
  across runs is valid.

## 0. The one ground rule, and how it is applied here

**Within-covered-scope quality and raw coverage are separate numbers and are
never blended.** Covered scope is exactly the 28 files
`20-harvest/output/partition-ledger.md` marks `done` (partitions P1–P6,
`BU-0001`–`BU-0312`). The other 54 `decompose` files (P7–P21) are `pending`
in that same ledger, and the run's own review raised the gap as its dominant
defect (`AF-0001`, `AF-0002`, `AF-0003`, all severity `high`). A behavior
whose source file was never opened is a coverage number. It is not a
recall failure and — critically for §22.2 — it is not *silent*, because the
ledger names the partition it lives in.

**What this adjudication re-derived independently** rather than accepting
from the comparers (all against `reference/sergeant-upstream/` at the pinned
SHA):

| Check | Method | Result |
|---|---|---|
| Precision | recomputed `sha256(quote)` and literal-substring containment for all 312 units | 312/312 clean, 0 `span_bytes` records |
| Representation distribution | joined `20-harvest/output/behavior-units.ndjson` → `40-classify/output/classifications.ndjson` | reproduces C2 §4.1 exactly |
| Draft validity | re-ran `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py` over all 18 packages | 11 PASS / 7 FAIL, every failure `[S3]`, no other tag |
| Provenance | parsed every `BU-####` in all 18 `provenance.md` (204 distinct) and all `CONTEXT.md` (69 distinct) against `classifications.ndjson` | 0 unresolved |
| §22.2 hunt | lexically matched all 385 in-scope reference units against the 312 generated units, then hand-reviewed a 63-unit stratified sample of the unmatched set filtered on guardrail language | see §3 |

---

## 1. Adjudication of comparer disagreements and hedges (§9.8)

### D-1 — C1 §2: "`BU-P7-003`/`BU-P7-004` are absent everywhere." **Overturned. Both are present in the corpus.**

C1 correctly finds that `schema/project.yaml.example` scored 0/4 — run 3's
three units from that file (`BU-0201`–`BU-0203`) are all `dag:`-block
content and none of the reference's four units' source regions were touched.
That per-file finding stands. The *corpus-wide* claim attached to it does
not:

- **`BU-P7-003`** (*"A published cross-repo knowledge graph is replaced only
  after a graphify run completes in full…"*, Delivery class) is captured
  three times over: **`BU-0245`** (`bin/sgt-graphify` L7-10) — *"Publication
  of the merged project graph is atomic: concurrent readers observe either
  the complete old output or the complete new output throughout the run,
  never a partial…"* — which is a near-verbatim match for the reference's
  own `bin/sgt-graphify` twin **`BU-P6-088`**; **`BU-0198`**
  (`docs/schema.md`) — symlink preservation plus *"Sergeant only replaces
  the published output after a complete run"*; and **`BU-0254`**
  (`bin/sgt-graphify` L513-536) — the single-rename (`mv -T`) mechanism.
  `BU-0253` adds the failure-path restore.
- **`BU-P7-004`** (per-repo `agent_instructions` override group-level) is
  captured as **`BU-0053`** (`AGENTS.md`) — *"`sgt-context` resolves
  instructions in order — `defaults.agent_instructions`, then group
  instructions, then repo instructions — with later layers overriding
  earlier ones"* — and corroborated by `BU-0200` (`docs/schema.md`).

§9.8 classification: **comparer error** (outside the taxonomy), corrected
here. The underlying per-file gap is a **generator miss** at file
granularity only.

### D-2 — C1 §3 / C3 §3 item 4: `BU-P6-124` (dispatch-time model-tuple precedence) is "missing too, not just the resume-time half." **Overturned at corpus level; upheld as a per-file attribution gap.**

C1's literal statement — that run 3's `bin/sgt-dispatch` units
(`BU-0287`–`BU-0296`) contain no model-tuple-precedence unit — is true and
verified. The conclusion drawn from it is not. `BU-P6-124`'s three
constituent claims are each captured, from `AGENTS.md` and `README.md`:

| `BU-P6-124` clause | Run 3 counterpart |
|---|---|
| "a per-invocation flag beats an environment variable, which beats no pin at all" | **`BU-0058`** — *"`sgt-dispatch --model` or `SERGEANT_MODEL` pins the harness model as `provider/model[:variant]`, resolved with precedence `--model` > `SERGEANT_MODEL` > the harness's ambient default"* |
| "no project-level or per-repo default in the precedence chain by deliberate decision, not by omission" | **`BU-0074`** — *"resolved from the flag, the environment, or the unpinned default, explicitly only — there is deliberately no project-level model default"* |
| "validated before any intent file, task, or worktree exists" | **`BU-0059`** — *"A model tuple the selected harness cannot honor fails before any intent file, td task, worktree, or fleet state is created"* |

Further, `consequence-class-sweep.md`'s **`AGENTS.md` Identity column cites
`BU-0058` and `BU-0059`** — the sweep *did* catch this Identity-class
behavior, at the file where it was extracted. The reference itself
redundantly sources the same fact from `AGENTS.md` (`BU-P1-058`,
`BU-P1-060`), so run 3's attribution is not even eccentric.

§9.8 classification: **comparer error**, corrected. What survives is a real
but much narrower defect — the `bin/sgt-dispatch` extraction did not
re-derive the precedence from the script's own L180-190 comment block, and
that file's Identity sweep column (`BU-0292, BU-0295, BU-0296`) does not
cross-reference the units that do carry it.

### D-3 — C2 §2.4 left the `dag-orchestration` boundary explicitly unresolved, pending "a source-citation check C1/C3 would run." Neither ran it. **Resolved here: run 3 is source-supported; this is a gold-side omission.**

Verified directly against
`reference/sergeant-upstream/schema/project.yaml.example`: lines 105–120
contain a real `dag:` block (`name`, `stages`, `repos:`, `td:`, `brief:`,
`after: [foundation]`) plus the header comment *"sgt-watch auto-advances the
DAG when fleet tasks complete."* `BU-0201`/`BU-0202`/`BU-0203` quote it
verbatim (hash-verified, §0). The reference's four units from this file cite
lines 5, 31-32, and 90-92 — it never extracted the `dag:` block from this
file at all, routing DAG-advance territory to `engine-pressure.md` G2 via
`bin/sgt-dag-*` instead (files in `pending` partition P13).

§9.8 classification: **gold miss** (narrow — the reference's own
`schema/project.yaml.example` extraction is incomplete) **plus legitimate
alternate decomposition** (run 3 files DAG-stage dependency as an ordinary
workflow; the reference treats the runtime-advance half as an engine gap).
Not generator invention. The two positions are compatible: a declarative
`after:` field is expressible; runtime auto-advance is what G2 is about.
No action against run 3.

### D-4 — C1 §4 / C3 §3 item 1: the "confirmed false `swept, none found`" cell. **Upheld as a sweep defect; downgraded in consequence.**

`consequence-class-sweep.md` row 22 reads
`| schema/project.yaml.example | swept, none found | BU-0201 | swept, none
found | swept, none found | swept, none found |`. The file's own lines 90-92
state the atomic-replace guarantee, which is a textbook Hunt-Question-4
Delivery pattern. The cell is wrong. **Upheld.**

But its consequence is smaller than C1 and C3 both imply, because the very
same sweep table's **`bin/sgt-graphify` Delivery column cites `BU-0245,
BU-0250, BU-0251, BU-0254`** — the atomic-publish behavior *was* hunted,
found, and filed under Delivery, one row down. This is a per-file
cross-referencing failure inside the sweep, not a class of behavior escaping
the sweep. It is the same defect shape as C1's item #10 (`BU-0018` captured
but not cited in `AGENTS.md`'s Safety or Human-decision columns — verified:
Safety lists `BU-0050, BU-0054, BU-0055, BU-0057, BU-0059`; Human-decision
lists `BU-0029, BU-0050`).

§9.8 classification: **generator miss**, artifact-level, two confirmed
instances. Real, and the correct target for a v3 fix — but not a §22.2
absence.

### D-5 — C1's recall figure is hedged as a floor ("≥56.9%"). **Upheld as a floor; the floor is loose, and this adjudication tightens the read without inventing a point estimate.**

C1 is right that its lexical method undercounts paraphrase and right to
label ten files "auto (floor)". An independent re-run here with a different
tokenizer over the full record blob produced an even lower floor (158/385 =
41.0%), confirming the method — not the corpus — is what varies.

The tightening evidence: of a **63-unit stratified hand sample** drawn from
the lexically-unmatched set and filtered on hard guardrail language
(`never`, `refuses`, `exactly once`, `orphan`, `escalate`, `atomic`,
`before any`, `fail closed`, …), **56 turned out to be genuinely captured
under paraphrase** and only 7 absent. Examples of matcher failure at
J<0.30 that are in fact near-verbatim: `BU-P8-094`→`BU-0172`,
`BU-P8-098`→`BU-0178`, `BU-P5-115`→`BU-0123`, `BU-P8-108`→`BU-0191`,
`BU-P6-088`→`BU-0245`, `BU-P1-048`→`BU-0048`. **The true within-scope recall
is materially above 56.9%; on guardrail-class content specifically it is
near 89%.** No point estimate is asserted for the whole 385 — full hand
verification was not performed and the sample is deliberately
guardrail-biased.

### D-6 — C2 §4.3 pairs `BU-0251` against `BU-P6-088` as its "strongest disagreement." **Mis-paired; the finding survives in weakened form.**

`BU-P6-088` is the atomicity property, and its true counterpart is
**`BU-0245`**, not `BU-0251`. `BU-0251` is the staged-artifact completeness
check (`graph.json`/`manifest.json`/`GRAPH_REPORT.md` nonempty), whose
reference counterpart is nearer `BU-P6-091`. C2's substantive point — that
`bin/sgt-graphify` gets 9 `helper` + 1 `stage` from run 3 against 2 `stage`
+ 2 `stage-context` from the reference (both recomputed and confirmed here)
— is unaffected. The named example should be `BU-0245` (`helper`, vs
`BU-P6-088` `stage`), which makes the point more sharply, not less.

### D-7 — C1 verifies precision over all 312 units; C3 leans on `80-adversarial-review`'s 20-unit sample. **No conflict; C1's method is the correct one and reproduces.**

Recomputed here at full population: 312/312 literal substrings, 312/312
hash matches, 0 records carrying `span_bytes` — so run 2's D-2
long-quote-convention dispute cannot recur in this run.

---

## 2. §9.9 dimensions

*No single accuracy number is reported. Coverage is stated once and never
averaged into a quality figure.*

### Source coverage

| | Value |
|---|---|
| Raw, run 3 | **28 / 82 `decompose` files (34.1%)**. 54 files never opened (P7–P21), disclosed by `partition-ledger.md` and raised as `AF-0001`/`AF-0002`/`AF-0003`. |
| Raw, run 2 | 16 / 136 files (11.8%) under the older, finer inventory split. |
| Within covered scope | **28 / 28 files yield ≥1 unit.** No zero-unit file. |
| Within-file evenness | Uneven, with two named blind spots. `AGENTS.md`'s "Procedural skills" routing table (L107-121) again produced nothing of its own — reference `BU-P1-132`–`BU-P1-137`, only `BU-P1-136` partially recovered via `BU-0123`. This is the *same* region run 2 missed, unresolved across generator versions. `schema/project.yaml.example` is the sharpest new instance: only the `dag:` block was read (`BU-0201`–`BU-0203`). |

Run 3 and run 2 overlap on only three files (`AGENTS.md`, `README.md`,
`bin/sgt-dispatch`); run 3 spent its breadth on new territory (docs, schema,
skills, cross-repo/dispatch) rather than finishing run 2's `bin/` fleet set.

### Behavioral recall

*Separated from coverage. Denominator is the 385 reference units whose
`source.path` falls in the 28 covered files.*

| Sub-scope | Ref units | Recovered | Recall |
|---|---:|---:|---:|
| C1's mixed hand/lexical total | 385 | 219 | **≥ 56.9% (floor)** |
| Independent lexical re-run (stricter) | 385 | 158 | 41.0% (looser floor) |
| Guardrail-class hand sample of the unmatched set (§1 D-5) | 63 | 56 | 88.9% |
| **Controlled same-file comparison vs run 2** (`AGENTS.md` + `README.md`) | 103 | 81 | **≥ 78.6%**, against run 2's **51.5%** (53/103) on the identical two files |

Fully hand-verified files cluster high: `mise.toml` 5/5, `bin/sgt-graphify`
4/4, `bin/sgt-list` 2/2, `bin/sgt-sync` 2/2, `bin/sgt-status` 1/1,
`bin/sgt-td-list` 1/1, `bin/sgt-td-create` 1/1, `docs/schema.md` 10/12,
`docs/README.md` 5/6. Two confirmed sharp exceptions:
`schema/project.yaml.example` **0/4** and `docs/what-is-sergeant.md` **7/14**
(its Project/Repository/Task/Fleet core-concept definitions,
`BU-P1-101`–`BU-P1-104`, are absent; only Worker and Decision-request
survived as `BU-0111`/`BU-0112`).

§9.8 classification of the residue: predominantly **generator miss**, with a
material **legitimate alternate decomposition** component — reference units
the run recovered from a *different* source file than the reference cited
(D-1, D-2 are both of this shape) count as recovered under §9.9's "regardless
of exact workflow grouping."

### Behavioral precision

**Clean — 312/312, verified at full population, not sampled.**

| Check | Result |
|---|---:|
| `quote` is a literal contiguous substring of the cited file at the pinned SHA | **312 / 312** |
| `sha256(quote)` equals stored `quote_hash` | **312 / 312** |
| Records carrying `span_bytes` (run 2's D-2 failure mode) | **0** |
| Invented behaviors found in any hand-verified file | **0** |

Unmatched *generated* units are consistently finer-grained real guard
clauses the reference collapsed — e.g. `BU-0246` (repo-name charset
rejection) and `BU-0247` (`graphify.output` never equal to a source repo
path), both real `bin/sgt-graphify` guards not separately itemized in the
reference's 4 units for that file. §9.8: **legitimate alternate
decomposition**, zero invention. Precision held across the v1→v2 change:
the recall gain was not bought with evidence discipline.

### Workflow-boundary agreement

Mixed and file-dependent; not uniformly better than run 2.

- **Strongest positive, repeating run 2's pattern.**
  `task-intake-and-execution` independently recovers `AGENTS.md`'s nine
  numbered steps blind to the reference — `BU-0025`/`BU-0026`/`BU-0028`/
  `BU-0031`/`BU-0034`/`BU-0035` land on real `stage` records matching
  `task-intake-and-route`'s `01-load-context`/`03-choose-mode`/
  `05-confirm-decisions`/`06-execute`/`08-handle-decisions`/
  `09-reconcile-deliver` one-for-one by step number. Step 7 is correctly
  left unattached (`BU-0033`, `stage-context`), matching the reference's own
  A4 fold.
- **Sharpest negative, and a genuinely new failure mode.**
  `cross-repo-planning` (C2 §2.1): `skills/cross-repo-work/SKILL.md` was
  read in full and produced 6 units (`BU-0267`–`BU-0272`), of which only
  `BU-0267` carries a workflow tie; `BU-0268`–`BU-0272` were filed
  `agents-invariant` on rationales that pattern-match to an existing
  AGENTS.md rule ("the same shape as `BU-0018`"). The reference extracts 16
  units from this file, **all 16 workflow-tied, zero `agents-invariant`**.
  The boundary collapsed one rung earlier in the ladder than any run-2
  failure. Partially self-disclosed: `60-draft/output/draft-report.md` names
  the package among its 7 zero-stage candidates, but as "no stage record,"
  not as "this file is under-extracted."
- **Three workflow-level matches with no stages.** `skill-adoption`
  (`BU-0119`), `sergeant-help-query` (`BU-0124`),
  `undocumented-failure-escalation` (`BU-0192`) are each anchored by a real
  `representation: workflow` record — the strongest available form of
  boundary agreement — against reference counterparts with 7, 3, and 1
  stages respectively. Honest under-staging, correctly declining to invent
  checkpoints. §9.8: **legitimate alternate decomposition** under the
  evidence held.
- **One representation-family disagreement**, `dag-orchestration`, resolved
  at D-3 above.

### Stage-boundary agreement

More symmetric than run 2, which found over-staging only.

- **One reasoned over-staging disagreement against precedent A4.**
  `task-intake-and-execution` keeps `02-resolve-task` (`BU-0026`) and
  `03-reconcile-before-start` (`BU-0028`) as stages where A4 demoted their
  equivalents. Unlike run 2's identically-shaped finding, `40-classify`
  states a genuine §6.3 checkpoint argument for both before descending —
  `BU-0026`: *"exactly the kind of durable, evidence-bearing outcome
  operators want tracked as its own boundary"*; `BU-0028`: *"a real,
  evidence-bearing gate operators would want measured on its own."*
  §9.8: **legitimate alternate decomposition**, a live adjudication
  candidate rather than a defect.
- **One under-staging case, self-caught.** `worker-response-and-recovery`'s
  `recover-worker` cluster (`BU-0039`, `BU-0146`, `BU-0159`, `BU-0174`,
  `BU-0286`) demotes every member on the claim that the checkpoint "is
  established more specifically elsewhere," and the chain never terminates
  in a `stage` record — independently confirmed here: no record in
  `classifications.ndjson` carries both `"stage": "recover-worker"` and
  `"representation": "stage"`. `AF-0004` caught it. The reference
  materializes exactly this territory as real stages
  (`recover-stalled-worker`'s `00-collect-signals`,
  `40-escalate-on-second-attempt`, `50-escalate-undocumented`). §9.8:
  **generator miss**, self-detected.

### Representation agreement

Recomputed independently; C2 §4.1 reproduces exactly.

| Representation | Run 3 (312) | Reference, same 28 files (385) |
|---|---:|---:|
| `agents-invariant` | 106 (34.0%) | 84 (21.8%) |
| `helper` | 87 (27.9%) | 28 (7.3%) |
| `stage-context` | 73 (23.4%) | 103 (26.8%) |
| `stage` | 31 (9.9%) | 68 (17.7%) |
| `shared-helper` | 8 (2.6%) | 15 (3.9%) |
| `workflow` | 3 (1.0%) | 26 (6.8%) |
| `shared-context` | 2 (0.6%) | 43 (11.2%) |
| `obsolete-mechanism` | 0 | 14 (3.6%) |
| `engine-gap` | 2 (0.6%) | 4 (1.0%) |

The `helper`+`shared-helper` skew collapsed from run 2's **73.1%** to
**30.4%**; `stage`+`stage-context` rose from 13.9% to 33.3% against a
same-scope reference figure of 44.5%. The ladder's §6.3-before-§6.5 ordering
rule (added in direct response to run 2) is honestly applied — `review-summary.md`
Axis 2 sampled 12 `helper`/`shared-helper` rationales and found every one
states the §6.3 answer first, and the file-shape-mirroring tell the rule was
written to catch is absent (`dispatch-worker`'s helper clusters group by
contract across files, unlike run 2's `dispatch-mode` clusters which were
one-cluster-per-script).

**But the improvement is compositional, not a changed verdict.** On the one
file both runs share and that drove most of run 2's skew, `bin/sgt-dispatch`
is **8 `helper` / 1 `stage-context` / 1 `stage`** (recomputed) — run 2
reached 9 `shared-helper` of 10, the reference 5 of 6 at checkpoint tier or
higher. `bin/sgt-graphify` is **9 `helper` / 1 `stage`** against 4/4
checkpoint-tier in the reference. Same content type, same under-promoted
answer, now with the question honestly asked. Two tiers stayed unused:
`shared-context` (2 vs 43) and `obsolete-mechanism` (0 vs 14), the latter
justified by count alone and flagged as insufficiently reasoned by
`AF-0005`.

§9.8: the aggregate movement is real improvement; the residual `bin/`-script
skew is **generator miss**, and the unused `shared-context`/`obsolete-mechanism`
tiers are **generator miss** with the same shape run 2 exhibited.

### Engine-gap quality

Run 3 produced **3 engine-gap claims** — 2 behavior-sourced
(`BU-0137` mission-handoff exactly-once; `BU-0227` callback claim-lease with
backoff and bounded batch) and 1 meta (`60-draft`: per-run content written
outside a stage's own `output/` cannot receive a D9 disposition). Against
run 2's **0 domain claims from 4 reachable sources** this is the dimension's
first non-null result.

All three carry the full §6.7 apparatus: behavior, source evidence,
`lower_rungs_attempted` enumerating all six rungs 6.1–6.6, a distinct
`why_each_fails` per rung, and a minimum runtime capability. The refutations
are substantive, not boilerplate — e.g. `BU-0227`'s 6.5 refutation is *"a
helper script invoked per drain has no durable claim state that survives
across separate invocations from separate processes."* Both behavior-sourced
claims map onto reference `engine-pressure.md` survivors (`BU-0227` → G3,
the durable outbound notification queue with an acknowledgement gate;
`BU-0137` → the crash-safe-publication family). **Agreement: 2 of 2
behavior-sourced claims land on reference-surviving territory; 0 rejected
claims; 0 claims made without a lower-rung refutation.** None were
independently reviewed — `80-adversarial-review` has no engine-gap axis —
so "survived independent review" is untested, not passed.

### Provenance completeness

**Clean at full population.** All 204 distinct `BU-####` ids cited across the
18 `provenance.md` files, and all 69 cited across `CONTEXT.md` files, resolve
in `classifications.ndjson` — zero unresolved, verified here directly rather
than taken from `80-adversarial-review`. C3's 7-package hand trace adds what
the id-existence check cannot: field-level agreement between each package's
claimed `representation`/`workflow`/`stage` and the ledger's (7/7 match), and
that no citation points outside the 28-file covered scope. Every cited unit's
`source.path`/`source.locator`/`quote` reopens at the pinned SHA (§0
precision check covers all 312).

### Draft validity

**11/18 PASS (61%), 7/18 FAIL — every failure `[S3]`, re-verified here by
re-running the validator.**

- **`[S12]` (declared outputs, no finalize step): fully closed, 0/18**,
  against run 2's 2/3. Structural, not lucky: `60-draft`'s template now
  requires a finalize-aware closing stage wherever a package declares
  outputs, and the passing packages span 1 stage (`fleet-cleanup`) to 8
  (`task-intake-and-execution`).
- **`[S3]` (empty `workflow.stages`): 7/18, not closed.** The correlation is
  exact — every zero-stage package fails, every ≥1-stage package passes. All
  7 are honest bucket-3 designs that self-disclose in their own `CONTEXT.md`
  and are triaged `substantive` by `70-lint`. This is a validator/ladder
  disagreement (the validator assumes ≥1 stage; the ICM ladder permits a
  correctly-drawn zero-stage workflow boundary), not a generation defect —
  and finalize-awareness was never aimed at it. But by the literal §9.9
  question, the delivered population still fails 7 times.

### Review convergence

From a **null capability** (run 2: 0 findings raised against a 3-package
corpus later shown to hold 3 latent structural defects) to a **real one**:
5 findings (`AF-0001`–`AF-0005`), 3 `high` / 2 `medium`, **0 false
positives** — every one independently re-verified, three re-derived again
here (P7–P21 all `pending`; sweep has exactly 28 data rows; no
`representation: stage` record for `recover-worker`). `90-reconcile` disposed
all five without rubber-stamping: 3 `park`, 1 `accept`, 1 `accept (partial)`,
with principled refusals to patch NDJSON ledgers with non-conforming lines or
to re-run upstream stages from a reconcile step.

**Unresolved after reconciliation: 0 rejected, 3 parked** (`AF-0001`,
`AF-0002`, `AF-0005` — each requiring an upstream re-run the reconcile stage
cannot perform).

**Recurring kind the review does not reach.** Its axes test whether a
*promoted* record deserved its rung (over-staging, invention) and whether the
corpus is arithmetically self-consistent (Axis 4). Nothing samples the
106-record `agents-invariant` tier for *under*-promotion, and nothing reads a
swept file's prose back against its sweep row. Both artifact-level sweep
defects (D-4) and the `cross-repo-planning` collapse are exactly those two
shapes; the review caught the one instance whose own rationale text was
internally circular (`AF-0004`) and missed the rest.

### Protocol mechanics (run-3-specific axis; v1 had no equivalent)

Structurally sound on every check: `BU-0001`–`BU-0312` contiguous with no
gaps or duplicates; the ledger's six `done` ranges partition that range
exactly and sum to 312; the halt is at a partition boundary (P6 fully `done`,
P7 fully `pending`, no third state and no partial range anywhere in 21 rows);
28 sweep rows keyed one-per-`done`-file with no blank cell across 140 cells.
**Untested edge: an actual resume.** `run-manifest.md`'s journal shows exactly
one `execution.started`/`execution.stopped` pair per stage — 10/10, no retry
anywhere — so the ledger's fitness for "read as authoritative, skip re-work"
was never exercised. **Known limitation, correctly parked not hidden:**
nothing gates `30-normalize` onward on ledger completeness, which is
precisely `AF-0001`.

---

## 3. §22.2 success criterion — answered explicitly

> *No reference behavior with a confirmed safety, identity, recovery,
> delivery or human-decision consequence is silently absent from the final
> adjudicated output.*

**Answer: YES — the criterion is MET for run 3's covered scope.** This
inverts run 2's answer, which was an unambiguous NO.

**What "silent" requires**, per the standing run-2 reading: the behavior is
absent from the final adjudicated output, *and* nothing in the run records
the absence. An honestly-pending partition fails the second test — the
ledger names it. The criterion is also scoped to the *output*, not to any
one file's extraction: a behavior recovered from a different source file
than the reference cited is present.

**Disposition of run 2's 11 silent misses.** Of the 11, three fell inside
run 3's newly-covered files and **all three are now captured**:

| Run-2 miss | Reference id | Run 3 status |
|---|---|---|
| #9 standing authorization never authorizes risk acceptance / gate skipping / force ops / secret exposure / destruction of preserved state | `BU-P1-050` | **`BU-0050`**, near word-for-word; double-filed in the sweep under Safety and Human-decision |
| #10 direct mode never edits several repos in one checkout, nor bypasses instructions / ownership / review independence / shipping gates | `BU-P1-016` | **`BU-0018`**, verbatim (`AGENTS.md` L38-41). Captured; *under-cited* in the sweep (D-4) |
| #11 dispatch rollback kills only a pane this invocation created, disarms on full success | `BU-P6-127` | **`BU-0288`** + **`BU-0296`**, split into its two independently-true halves, both swept |

The remaining **8 of 11 are all scope-excluded, not silent** — every source
file (`bin/sgt-drain-force`, `bin/sgt-cleanup`, `bin/sgt-watch`,
`bin/sgt-interactive-worker`, `bin/sgt-recover`, `bin/sgt-respond`,
`bin/sgt-wake`) sits in P7–P10, `pending` in `partition-ledger.md` and named
in `AF-0001`.

**Independent hunt inside the covered scope.** Rather than only re-checking
the comparers' candidates, all 385 in-scope reference units were matched
against all 312 generated units, and the 227 lexically-unmatched were
filtered on hard guardrail language, yielding 63 candidates for hand review.
**56 of 63 proved captured under paraphrase.** The 7 genuine absences:

| Ref id | Source (partition) | Behavior | Consequence class | Verdict |
|---|---|---|---|---|
| `BU-P5-049` | `skills/cross-repo-work/SKILL.md` (P6 done) | required contents of a per-repository delivery gate (owning td task, fixed point + preserved source state, repo-native test/lint/typecheck/build, Standards/Spec review sources, PR dependency + deployment order, outstanding data/security/destructive decisions) | Delivery, **soft** | **Sharpest surviving candidate.** A gate-completeness checklist, not an exactly-once / atomicity / readiness guarantee — it does not answer Hunt Question 4 as posed. Reference files it `stage-context`, not a guardrail. Partially disclosed: `cross-repo-planning` is named a zero-stage package in `draft-report.md`. |
| `BU-P6-012` | `bin/sgt-status` (P5 done) | status walk reports per-repo state "never mutating anything" | Safety-adjacent, **soft** | A read-only guarantee, not a gate on a destructive action. Reference files it `workflow`. `bin/sgt-status`'s sweep row is all `swept, none found`. |
| `BU-P5-038` | `skills/cross-repo-work/SKILL.md` | decompose outcome and define dependency/merge order before any dispatch | none | Procedural; partially covered by `BU-0006`, `BU-0269` (cycles rejected before dispatch) |
| `BU-P5-052` | `skills/cross-repo-work/SKILL.md` | post-dispatch reconciliation checklist | none | Partially covered by `BU-0035` |
| `BU-P5-104` | `skills/load-project/SKILL.md` | `docs/schema.md` is the single source of truth for YAML fields | none | Documentation authority |
| `BU-P8-041` | `docs/getting-started.md` | first-install required/optional tool set | none | Install prerequisites |
| `BU-P8-051` | `docs/getting-started.md` | nine-item completed-installation checklist | none | Install checklist |

**No behavior carrying a confirmed safety, identity, recovery, delivery or
human-decision consequence is silently absent.** Every candidate the
comparers nominated as such resolves otherwise: `BU-P7-003` is present
(`BU-0245`/`BU-0198`/`BU-0254`) and correctly swept under `bin/sgt-graphify`
Delivery; `BU-P7-004` is present (`BU-0053`); `BU-P6-124` is present
(`BU-0058`/`BU-0059`/`BU-0074`) and correctly swept under `AGENTS.md`
Identity. The two soft candidates above are recorded here as visible
lower-impact measurements for v3, per §22.2's own instruction.

**Two real defects that do not meet the "silently absent" bar but must not
be lost**, both in `consequence-class-sweep.md` rather than in the corpus:
one confirmed false `swept, none found` (`schema/project.yaml.example`,
Delivery) and one under-citation (`AGENTS.md`, `BU-0018` absent from both
Safety and Human-decision). The behaviors exist; the sweep's own
cross-referencing failed. Left unfixed, this class degrades into genuine
silent absence as soon as the only source of a consequence-class behavior is
the file whose cell is wrong.

---

## 4. §22.2 reporting checklist

| Item | Value (within covered scope unless noted) |
|---|---|
| Reference behaviors total | 979 corpus-wide; **385** in covered scope |
| Matched behaviors | **≥ 219** (C1 floor); guardrail-class hand sample 56/63 (88.9%) |
| Missed behaviors | ≤ 166, of which **0 carry a confirmed §22.2 consequence**; 2 soft candidates (`BU-P5-049`, `BU-P6-012`) |
| Extra unsupported behaviors | **0 invented.** Unmatched generated units (e.g. `BU-0246`, `BU-0247`) are finer-grained and source-supported → legitimate alternate |
| Workflow-boundary agreements / disagreements | 1 strong independent match (`task-intake-and-execution` ↔ nine numbered steps) + 3 `representation: workflow`-anchored matches (`BU-0119`, `BU-0124`, `BU-0192`) / 1 new-shape collapse (`cross-repo-planning`, 5 of 6 units mis-tiered) + 1 representation-family disagreement (`dag-orchestration`, resolved as gold miss + legitimate alternate) |
| Stage-boundary agreements / disagreements | 6 of 9 `AGENTS.md` steps agree one-for-one; step-7 fold matches A4 / 1 reasoned over-staging vs A4 (`BU-0026`, `BU-0028`), 1 self-caught under-staging (`recover-worker`, `AF-0004`) |
| Representation agreements / disagreements | `helper`+`shared-helper` 30.4% vs 11.2% (was 9.3× over in run 2, now 2.7×); `stage`+`stage-context` 33.3% vs 44.5% (was 0.31×, now 0.75×); `shared-context` 2/43; `obsolete-mechanism` 0/14; `workflow` 3/26 |
| Engine-gap agreements / disagreements | 3 claims generated (2 behavior-sourced, 1 meta), all with complete §6.7 lower-rung refutations; both behavior-sourced claims land on reference-surviving territory (G3 and the crash-safe-publication family); 0 rejected; 0 independently reviewed |
| Unresolved review findings | 5 raised, 5 disposed, **0 rejected, 3 parked** (`AF-0001`, `AF-0002`, `AF-0005`) |
| Coverage (reported, never averaged in) | **28 / 82 files (34.1%)**; 54 unreached, all ledger-declared |

### §9.8 classification roll-up

| Category | Count | Instances |
|---|---:|---|
| Generator miss | 6 classes | ≤166 unmatched in-scope ref units (0 with confirmed §22.2 consequence); `schema/project.yaml.example` read only at its `dag:` block (0/4); `AGENTS.md` procedural-skills routing table again unextracted (`BU-P1-132`–`137`); 2 `consequence-class-sweep.md` cross-referencing defects; `recover-worker` circular demotion (`AF-0004`, self-caught); `shared-context`/`obsolete-mechanism` tiers unused |
| Generator invention | **0** | Zero at the behavior-unit level (312/312 hash-verified) and zero at the artifact level — no unsupported headline found, unlike run 2's D-1 |
| Gold miss | 1 | The reference's `schema/project.yaml.example` extraction omits the file's `dag:` block entirely (D-3) |
| Legitimate alternate decomposition | 5 | `bin/_sgt-review-axes.sh` dispositioned `decompose` (→`BU-0300`–`BU-0302`) where the reference calls it `helper-evidence`; `BU-0026`/`BU-0028` over-staging vs A4 with a genuine §6.3 argument; the three zero-stage `representation: workflow` packages; finer-grained unmatched generated units; DAG-declaration-as-workflow vs engine-gap (D-3) |
| Ambiguous source | 0 | Every disagreement examined resolved to another category |
| Genuine engine pressure | 3 claimed, 0 refuted | `BU-0137`, `BU-0227`, and the `60-draft` meta claim, each with all six lower rungs attempted and refuted. Not independently reviewed — provisional |
| *Comparer error (outside §9.8)* | 3 | C1's "`BU-P7-003`/`BU-P7-004` absent everywhere" (D-1); C1/C3's "no model-tuple-precedence unit at all" (D-2); C2's `BU-0251`↔`BU-P6-088` mis-pairing (D-6) |

---

## 5. Verdict

**Within covered scope, run 3 is honest, precise, and no longer shallow —
and for the first time it clears the §22.2 bar.** Precision is flawless at
full population (312/312, zero inventions, zero `span_bytes` records).
Provenance resolves completely (204/204 + 69/69). Recall on the two files
both runs read rose from 51.5% to ≥78.6%. Every consequence-class behavior
the comparers nominated as missing turned out to be present in the corpus —
the run's own consequence-class sweep, the mechanism added specifically in
response to run 2's failure, is doing the work it was built for.

**What run 3 is not** is complete, and it says so: 54 of 82 files were never
opened, its own review made that its top three findings, and its ledger names
every pending partition. Two-thirds of the corpus remains unmeasured, and the
run's structural self-checks are demonstrably strong at arithmetic
consistency and weak at content — the two surviving sweep defects and the
`cross-repo-planning` tier collapse all required reading source prose against
the corpus, which no review axis does.

**The single most important carry-forward** is that both remaining
consequence-class defects are *cross-referencing* failures in
`consequence-class-sweep.md`, not extraction failures. That is a cheap fix
and a fragile safety margin: the criterion is met today partly because the
behaviors happened to be extracted from a second file. Making the sweep
cite every unit that covers a class for a file — not merely some — is what
keeps the margin from evaporating on the next partition.
