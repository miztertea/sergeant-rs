# Measurement package — run 01KZQ32J2BAD4P8WJA9SWXRMZ9

Per `references/reconciliation-method.md` §2. This run found no
`reference-corpus/` anywhere in the worktree (`../00-contract/output/
contract.md` §3), so `../_config/run-discipline.md` §1's blindness rule is
**vacuous for this run** — there is nothing to be blind to — but its
underlying discipline (never source a citation from anywhere but the
subject subtree) still held throughout, and this stage still reports the
five §9.9 dimensions below as not covered *from inside this run*, exactly
as it would for a measurement run: they require comparison against an
already-adjudicated reference decomposition, and none exists for this run
to compare against or be contaminated by.

Step 0 check (`../_config/run-discipline.md` §2): none of this stage's
Inputs-table artifacts open with `# AMBIGUOUS — NOT RESOLVED` — checked
directly, first line of each: `00-contract/output/contract.md`,
`10-inventory/output/inventory.md`, `20-harvest/output/partition-ledger.md`,
`20-harvest/output/consequence-class-sweep.md`,
`50-synthesize/output/candidates.md`, `60-draft/output/draft-report.md`,
`70-lint/output/lint-report.md`, `80-adversarial-review/output/
review-summary.md` (the two `20-harvest`/`40-classify`/`30-normalize`
NDJSON ledgers have no heading at all by construction — checked their first
record instead, ordinary `BU-####`/`behavior_id` records, not an ambiguity
marker). Ordinary reconciliation work proceeded.

---

## INTERNALLY COMPUTABLE

**Source coverage.** 178 files enumerated in scope; disposition split 82
`decompose` / 80 `helper-evidence` / 0 `obsolete-candidate` / 16
`reference-only` (`../10-inventory/output/inventory.md` "Totals",
82+80+0+16=178). Of the 82 `decompose` files across 21 partitions, only 6
partitions (P1–P6, 28 files) reached `done` in
`../20-harvest/output/partition-ledger.md`; the other 15 (P7–P21, 54 files)
are still `pending`. The consequence-class sweep
(`../20-harvest/output/consequence-class-sweep.md`) covers exactly those 28
`done`-partition files with **no blank cells among them** (every cell is a
`behavior_id` or the literal `swept, none found`, verified by direct
inspection) — but has **no row at all**, not even a `swept, none found`
row, for any of the 54 `pending`-partition files. This is the run's single
largest defect: adjudicated as AF-0001/AF-0002/AF-0003 in
`output/adjudication-log.md` (both parked as unrepairable from inside
`90-reconcile`, one partially repaired by adding a coverage caveat to the
two prose artifacts capable of carrying one). **Every count below this
point describes the 28-file (34%) corpus this run actually harvested, not
the 82-file corpus it was scoped to cover.**

**Behavioral precision.** Of the citations *this run itself* sampled and
reverified (`../80-adversarial-review`'s Axis 2 citation-reverification
sample): **20/20 sampled citations verified** — 20 behavior units spanning
12 of the 28 distinct source files present in the corpus, each re-located
at its cited byte range in `reference/sergeant-upstream` with the quote
appearing contiguously and `sha256(quote)` matching the stored
`quote_hash` exactly. No invention found in this sample
(`../80-adversarial-review/output/review-summary.md` Axis 2). This is a
20-unit sample of the 312-record corpus, not a full-corpus figure — do not
read it as "34/34" or extrapolate a full-corpus percentage from it.

**Provenance completeness.** 18/18 materialized candidates (100%) have a
`provenance.md` citing at least one real `behavior_id`; across all 18,
**204/204 distinct `BU-####` citations resolve** to a real record in
`../40-classify/output/classifications.ndjson` — zero unresolved citations
(`../80-adversarial-review/output/review-summary.md` Axis 2, "Provenance
citation re-verification"). Cross-checked against Axis 2's invention
findings: none of the 5 adversarial-review findings (`../80-adversarial-review/
output/findings.ndjson`) allege a fabricated citation — AF-0004 is a
circular-rationale finding, not an invented-citation finding — so this
100% figure stands unqualified by any recorded invention.

**Draft validity.** 18/18 candidates validated by
`scripts/validate-structure.py` (`../70-lint/output/lint-report.md`). 15
mechanical (`[S4]`) defects found and fixed across 5 candidates (one
systemic pattern: a stale `../CONTEXT.md` Inputs row on every non-first
stage, removed per `../_config/run-discipline.md`'s L1-delivery rule); 0
mechanical defects remain. 7 substantive (`[S3]`) defects remain across 7
candidates (`fleet-monitor-and-reconcile`, `callback-delivery`,
`skill-adoption`, `sergeant-help-query`, `troubleshoot-td-identity`,
`cross-repo-planning`, `undocumented-failure-escalation`) — each is the
same design choice (`workflow.toml` with `stages = []`, deliberately zero
materialized stages per `../60-draft/output/draft-report.md` "Zero-stage
candidates": no `representation: stage` record exists for these workflows,
and inventing one to satisfy the validator's ≥1-stage assumption is exactly
the invention this run's discipline forbids). Final split: **11/18 PASS
clean, 7/18 FAIL with the same recorded substantive defect.** These 7 are
not in `../80-adversarial-review/output/findings.ndjson` (review confirmed
they match lint's own attribution and did not re-litigate them) and were
not adjudicated as separate findings here for the same reason — they are
counted here as-is, unresolved, per this dimension's own definition.

**Review convergence.** 5 findings total from `../80-adversarial-review`,
by axis and severity (`review-summary.md`'s own table, reproduced):

| Axis | high | medium | low | total |
|---|---|---|---|---|
| boundary-honesty | 0 | 0 | 0 | 0 |
| invention | 0 | 1 | 0 | 1 |
| engine-gap-refutation | 0 | 0 | 0 | 0 |
| structural-self-consistency | 3 | 1 | 0 | 4 |
| **Total** | **3** | **2** | **0** | **5** |

Adjudication (`output/adjudication-log.md`, this stage): **1 accept**
(AF-0004 — rationale corrected in place), **1 accept-partial** (AF-0003 —
caveat added to 2 of its 4 named targets, the other 2 being NDJSON ledgers
with no field for a document-level caveat), **3 park** (AF-0001, AF-0002,
AF-0005 — all real and correct, none repairable from inside `90-reconcile`
without re-running an upstream stage or inventing new judgment), **0
reject**. Every one of the 5 findings held up on this stage's own
independent re-verification.

---

## NOT COVERED HERE

This run's own artifacts cannot establish, and this stage does not
estimate: **behavioral recall, workflow-boundary agreement, stage-boundary
agreement, representation agreement, engine-gap quality.** Each requires
comparing this run's output against an already-adjudicated reference
decomposition of the same subject. No such reference exists in this
worktree for this run (`../00-contract/output/contract.md` §3's exclusion
check found none), so there is nothing to compare against from inside this
run — these five dimensions are not "unknown," they are structurally
outside what a single run's own artifacts can self-report, per
`references/reconciliation-method.md` §2 and
`docs/gauntlet/contracts/N2.md` Outcome §3. A later, separate comparison
process — performed by comparers this run's blindness rule does not bind —
is what would produce them, whether or not a reference corpus happens to
exist yet.

---

## SUPPORTING RUN STATISTICS

**Run identity.** Subject: `reference/sergeant-upstream` (vendored
subtree, no live `.git` of its own). Revision:
`f430cfd4f90174a98adbd7abebbece6303817929`, resolved from
`reference/UPSTREAM.md`'s provenance row (vendored-subtree case; not
derived via `git rev-parse` inside the subtree, which would have returned
the *outer* repository's own HEAD — a false signal
`../00-contract/output/contract.md` §1 specifically checked for and
avoided). Task-claimed SHA and resolved SHA agree exactly; no discrepancy.

**Extraction coverage.** 312 behavior units extracted
(`../20-harvest/output/behavior-units.ndjson`), spanning 28 distinct source
files (independently recomputed: `len({source.path for each record})` =
28) — the P1–P6 partitions only. 54 files across the remaining 15
partitions (P7–P21) were never reached; see "Source coverage" above for
the full accounting.

**Normalization outcome.** 312 units in, 312 units out
(`../30-normalize/output/behavior-units.normalized.ndjson`) — same id set
as the harvest ledger exactly, zero splits (no non-standard `BU-####` id
appeared), zero merges. 0 confidence shifts (every record's `confidence`
field is byte-identical to its harvest-stage value). 49 of the 312 records
carry a `notes` field beginning "Mechanism (pre-normalization wording):",
recording that this stage rewrote the unit's `statement` to abstract away
an implementation-specific name (e.g. `sgt-context` → "the
context-resolution command") while preserving the original mechanism name
in `notes` for traceability.

**Representation mix** (`../40-classify/output/classifications.ndjson`,
312 records, independently recomputed):

| Representation | Count | % |
|---|---|---|
| agents-invariant | 106 | 34.0% |
| helper | 87 | 27.9% |
| stage-context | 73 | 23.4% |
| stage | 31 | 9.9% |
| shared-helper | 8 | 2.6% |
| workflow | 3 | 1.0% |
| shared-context | 2 | 0.6% |
| engine-gap | 2 | 0.6% |
| **Total** | **312** | **100%** |

Matches `../80-adversarial-review/output/review-summary.md` Axis 4's own
recomputation exactly.

**Candidate yield** (`../50-synthesize/output/candidates.md`): 18 workflow
candidates. Buckets 1–3 (workflow/stage/stage-context/helper): 194 records
(3 + 31 + 73 + 87). Bucket 4 (agents-invariant, listed not drafted): 106.
Bucket 5 (shared-helper/shared-context): 10 (8 + 2). Bucket 6
(obsolete-mechanism): 0 — flagged in adjudication (AF-0005, parked) for
insufficient per-unit justification of the zero count. Bucket 7
(engine-gap/engine-pressure): 2. 194+106+10+0+2 = 312.

**Draft materialization** (`../60-draft/output/draft-report.md`): 18/18
candidates materialized under `.sergeant/drafts/workflows/`. 26 `NN-*/`
stage directories total across the 11 candidates with any stages, carrying
31 `representation: stage` `behavior_id`s between them (matches the 31
`stage` count above exactly). 7 candidates materialize with zero stages
(`stages = []`), a deliberate design choice per
`../../_config/icm-ladder.md` bucket 3, not an omission — see "Draft
validity" above for how this surfaces at lint.

---

## Grammar pressure — pointer

Every surviving engine-gap claim (2 behavior-level, 1 meta-level) is
consolidated as a full six-field record in `output/grammar-pressure.ndjson`
per `references/reconciliation-method.md` §3. Not restated here.

---

## Finalize

`python3 .sergeant/workflows/repo-to-icm/scripts/finalize.py --dry-run`,
run from the repository root after steps 1–3 above were written and staged
(`git add .sergeant/workflows/repo-to-icm/*/output/`), verbatim:

```text
keep    00-contract/output/contract.md  (promote)
keep    10-inventory/output/inventory.md  (promote)
keep    20-harvest/output/behavior-units.ndjson  (promote)
keep    20-harvest/output/consequence-class-sweep.md  (promote)
keep    20-harvest/output/partition-ledger.md  (promote)
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

nothing to finalize (no evidence-class or undeclared files present)
```

All fifteen of this workflow's own `output/`-resident artifacts across all
ten stages are `promote` and none is evidence-class or undeclared — the
real, run-scoped defect this run surfaced (54 of 82 subject files never
harvested) lives in what `20-harvest`'s own artifacts honestly recorded as
`pending`/missing, not in any stray or misclassified file needing removal
here. The real finalize run (no `--dry-run`) follows this preview
immediately below, per `../CONTEXT.md` step 4.4.
