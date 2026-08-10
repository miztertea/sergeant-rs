# Measurement package — `repo-to-icm` run

Per `references/reconciliation-method.md` §2. This reports what this run can
honestly measure **without** `reference-corpus/` — this run's actors never
opened it, and this stage does not either (`../CONTEXT.md`'s blindness
note, `../_config/run-discipline.md` §1). Proposal §9.9 names ten
measurement dimensions; five are internally computable from this run's own
artifacts, five require a separate blind comparison against
`reference-corpus/` this run does not and cannot perform.

## Internally computable (§9.9)

### Source coverage

From `../10-inventory/output/inventory.md`:

- Files found by enumeration: **179**
- Excluded (build-dependency-output category): **1**
  (`bin/__pycache__/sgt-callbackcpython-312.pyc`)
- Files inventoried: **178**

| Disposition | Count |
|---|---:|
| decompose | 136 |
| helper-evidence | 27 |
| obsolete-candidate | 0 |
| reference-only | 15 |
| **Total** | **178** |

**Partition count:** 5 explicitly named partitions over the `decompose` set
(`inventory.md`'s own "partition **name**" markers): `bin: the sgt-* CLI`
(itself split into 5 named sub-groups: fleet dispatch & lifecycle,
validation & review gates, project & config commands, td task-tracker
integration, callback/graph/wiki utilities), `agents-skills: vendored
skill definitions`, `agents-skills: vendored skill supporting references`,
`skills: Sergeant-owned project skills`, and `tests: behavioral regression
& contract suite` (itself split into 7 named sub-groups: dispatch & worker
launch, drain lifecycle, recovery & lease/notification finalization,
respond/wake/callback delivery, watch/graphify/fleet observability,
validation & review gates, environment/policy/setup guardrails). Root
files, `docs/`, and root config/task-automation rows are not partitioned
(each row stands alone).

### Behavioral precision

Of the citations **this run itself** sampled and reverified
(`../80-adversarial-review/output/review-summary.md`, Axis 2 "Citation
re-verification"): **29/29** sampled citations verified cleanly (hash
matched, quote appeared contiguously at the cited locator; 0 invention
findings against this sample). The sample spanned all 16 distinct
`source.path` values in `../30-normalize/output/behavior-units.normalized.ndjson`
(not just the first few), out of 108 total behavior units.

Separately, provenance citations (every `behavior_id` cited in all three
candidates' `provenance.md` files): **16/16** confirmed to exist in
`../40-classify/output/classifications.ndjson`, 0 invention findings.

Report as sampled fractions, not extrapolated percentages: **29/29** (source
citation sample) and **16/16** (provenance citation sample).

### Provenance completeness

From `../70-lint/output/lint-report.md`'s four validator runs (`dispatch-mode`,
`standard-task-workflow`, `ship-with-no-mistakes`, this workflow's own
tree): zero `[S8]` defects were reported in any run. `[S8]` fails when a
draft package has no `provenance.md`, an empty one, or one that cites no
real `behavior_id` (`scripts/validate-structure.py`). No `[S8]` failure
means all three materialized candidates passed it: **3/3** materialized
candidates have a non-empty `provenance.md` citing at least one real
`behavior_id`. Cross-checked against Axis-2 invention findings above: zero
fabricated-citation findings against any of the 16 distinct `behavior_id`s
those `provenance.md` files cite, so this 3/3 reflects genuine citations,
not merely "a citation exists."

### Draft validity

From `../70-lint/output/lint-report.md`, validator pass/fail per candidate
(plus this workflow's own tree), and mechanical vs. substantive defect
counts:

| Tree | Initial | Mechanical fixes applied | Final | Substantive defects remaining |
|---|---|---:|---|---|
| `dispatch-mode` | FAIL (1) | 0 | FAIL (1) | `[S12]` no finalize step named |
| `standard-task-workflow` | FAIL (1) | 0 | FAIL (1) | `[S12]` no finalize step named |
| `ship-with-no-mistakes` | FAIL (1) | 0 | FAIL (1) | `[S3]` deliberate empty `stages` array |
| This workflow's own tree | FAIL (1) | 0 | FAIL (1) | `[S10]` `20-harvest/quote.sh` unclassified executable |

Totals: 4/4 trees FAIL at both initial and final validator runs; 0
mechanical defects found or fixed anywhere; 4 substantive defects total (1
per tree), each explicitly classified by `70-lint` as requiring judgment
that stage was not given authority to exercise on its own, and none
adjudicated away this stage (see `output/adjudication-log.md` — 0 findings
were raised against any of these four defects by `80-adversarial-review`,
so none was accepted/rejected/parked here; they remain open substantive
defects for a human reviewer).

### Review convergence

From `output/adjudication-log.md` (this stage's own): `../80-adversarial-review/output/findings.ndjson`
contained **0** findings. `review-summary.md`'s own finding-counts table
independently confirms this:

| Axis | high | medium | low | total |
|---|---:|---:|---:|---:|
| boundary-honesty | 0 | 0 | 0 | 0 |
| invention | 0 | 0 | 0 | 0 |
| engine-gap-refutation | 0 | 0 | 0 | 0 |
| **Total** | **0** | **0** | **0** | **0** |

Adjudication disposition counts (this stage): accept 0, reject 0, park 0 —
there was nothing to dispose. 0 repairs were applied to any file this run
as a result (the adjudication mechanism was never triggered, not skipped).

## NOT covered here

This package does **not** and cannot cover the following five `§9.9`
dimensions from inside this run — they require comparing this run's output
against `reference-corpus/`, performed later by independent comparers this
run's blindness rule does not bind (`docs/gauntlet/contracts/N2.md` Outcome
§3):

- behavioral recall
- workflow-boundary agreement
- stage-boundary agreement
- representation agreement
- engine-gap quality

No attempt was made to estimate any of these five from inside this run.

## Supporting run statistics

(Context for the above, not `§9.9` dimensions in their own right.)

**Run identity:** subject `reference/sergeant-upstream` @
`f430cfd4f90174a98adbd7abebbece6303817929` (upstream `main`, fork
`miztertea/sergeant`) — `../00-contract/output/contract.md`.

**Extraction coverage:** 108 units recorded (`BU-0001`–`BU-0108`), spanning
18 of 136 `decompose` files (`AGENTS.md`, `README.md`, and all 14 files of
the `bin: fleet dispatch & lifecycle` partition). 118 of 136 `decompose`
files/partition-members explicitly recorded as **not reached** this run —
this is the source of the `20-harvest` meta-level grammar-pressure record
in `output/grammar-pressure.ndjson` (`../20-harvest/output/coverage-note.md`).
0 zero-unit files among the 18 reached; 0 units carry `confidence: low` or a
disputed citation.

**Normalization outcome:** 108 units in, 108 units out (0 splits, 0 units
lost or gained — `id` sets identical between `../20-harvest/output/behavior-units.ndjson`
and `../30-normalize/output/behavior-units.normalized.ndjson`, verified
programmatically). 41 of 108 `statement` fields were rewritten during
normalization; 51 of 108 gained a `notes` field (typically a "mechanism:"
annotation); 0 `confidence` shifts (every unit's `confidence` value is
identical before and after normalization).

**Representation mix** (`../40-classify/output/classifications.ndjson`,
108 records, counted programmatically):

| representation | count |
|---|---:|
| `agents-invariant` | 13 |
| `workflow` | 1 |
| `stage` | 6 |
| `stage-context` | 9 |
| `shared-helper` | 79 |
| `shared-context` | 0 |
| `obsolete-mechanism` | 0 |
| `engine-gap` | 0 |
| **Total** | **108** |

**Candidate yield** (`../50-synthesize/output/candidates.md`):

- Workflow candidates: **3** (`dispatch-mode`, `standard-task-workflow`,
  `ship-with-no-mistakes`)
- Stage candidates: **6** (`dispatch-worker`; `load-context`,
  `check-queue`, `reconcile-existing-state`, `validate`,
  `reconcile-and-deliver`)
- Shared-helper candidates: **15** named groupings (`dispatch-contract`,
  `dag-dispatch-hook`, `dag-run`, `cleanup`, `drain`, `drain-force`,
  `undrain`, `recover`, `respond`, `ack-response`, `watch`, `wake`,
  `interactive-worker`, `notify`, `findings-router`), covering all 79
  `shared-helper` records
- Shared-context candidates: **0**
- Permanent-instruction (`agents-invariant`) candidates: **13** records,
  listed but not grouped into named candidates (bucket 4 does not group)
- Obsolete-mechanism candidates: **0**
- Engine-pressure (`engine-gap`) candidates: **0**
- Unattached `stage-context` records (bucket 1/3 defect appearance,
  neither dropped nor force-attached): **9**

**Draft materialization** (`../60-draft/output/draft-report.md`): 3
candidate packages materialized under `.sergeant/drafts/workflows/`:

| Candidate | Path | Member stages |
|---|---|---:|
| `dispatch-mode` | `.sergeant/drafts/workflows/dispatch-mode/` | 1 |
| `standard-task-workflow` | `.sergeant/drafts/workflows/standard-task-workflow/` | 5 |
| `ship-with-no-mistakes` | `.sergeant/drafts/workflows/ship-with-no-mistakes/` | 0 (deliberate) |

## Note folded back from `10-inventory`

`../10-inventory/output/inventory.md` flagged that `../00-contract/output/contract.md`
§3's supporting "checked" sentence ("no build/dependency-output directory
is currently present under it") is inaccurate — `bin/__pycache__/sgt-callbackcpython-312.pyc`
exists and was correctly excluded from the inventory by applying the
contract's own exclusion *category* rather than trusting that inaccurate
supporting sentence. Recorded here per `10-inventory`'s explicit request;
see `output/adjudication-log.md`'s "Other observation" section for why this
stage records rather than edits `contract.md` (it was never raised as an
`80-adversarial-review` finding, so it falls outside the accept/reject/park
mechanism this run's adjudication step applies).

## Finalize

`python3 .sergeant/workflows/repo-to-icm/scripts/finalize.py --dry-run`
(from the repository root), verbatim output:

```text
keep    00-contract/output/contract.md  (promote)
keep    10-inventory/output/inventory.md  (promote)
keep    20-harvest/output/behavior-units.ndjson  (promote)
would remove  20-harvest/output/coverage-note.md
keep    30-normalize/output/behavior-units.normalized.ndjson  (promote)
keep    40-classify/output/classifications.ndjson  (promote)
keep    50-synthesize/output/candidates.md  (promote)
keep    60-draft/output/draft-report.md  (promote)
keep    70-lint/output/lint-report.md  (promote)
keep    80-adversarial-review/output/findings.ndjson  (promote)
keep    80-adversarial-review/output/review-summary.md  (promote)
keep    90-reconcile/output/adjudication-log.md  (promote)
keep    90-reconcile/output/grammar-pressure.ndjson  (promote)
keep    90-reconcile/output/measurement-package.md  (promote)

dry-run: would finalize 1 file(s); nothing modified.
```

Exit code: `0` (clean apply plan, no ambiguity). `20-harvest/output/coverage-note.md`
is the one file this plan removes — `20-harvest/output/README.md` declares
only `behavior-units.ndjson` as `promote`, so `coverage-note.md` is
evidence-class/undeclared by that stage's own disposition and is correctly
scheduled for removal, not an error in this plan. Its content (the
partition-coverage accounting) survives independently: it is quoted and
summarized in `output/grammar-pressure.ndjson`'s `20-harvest` record and in
this file's "Extraction coverage" section above, so removal at finalize
does not lose the fact — only the per-run evidence file that first recorded
it (recoverable from Work-branch history regardless, per D9 convention).
