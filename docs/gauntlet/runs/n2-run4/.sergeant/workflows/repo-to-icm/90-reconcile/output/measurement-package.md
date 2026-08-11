# Measurement package — run 01KZQRGZE32RQ79KT82XTB9MV2

Per `references/reconciliation-method.md` §2. This run's actors never
opened `reference-corpus/`, and this stage does not either — the blindness
rule (`../_config/run-discipline.md` §1) applies for the whole run. Every
number below is pulled directly from this run's own artifacts, named by
stage, not estimated.

Corpus size, for scale: 21 harvested partitions, 1333 classification
records, 44 materialized draft workflow packages — larger than any prior
attempt of this workflow (run 3 stopped at 6 partitions). Nothing below was
sampled down for volume on the mechanical counts; where the corpus's own
upstream stages licensed a bound (`80-adversarial-review`'s judgment-based
checks used deliberate sampling, as its own method allows), that is stated
in place, not silently applied.

## Proposal §9.9 — the ten measurement dimensions, stated by name

### Internally computable (reported below, from this run's own artifacts)

**1. Source coverage.** `10-inventory/output/inventory.md`: 179 files
enumerated (`find reference/sergeant-upstream -type f`), disposed
`decompose` 83, `helper-evidence` 80, `reference-only` 16,
`obsolete-candidate` 0 (83+80+16+0=179 ✓), across 19 partitions (A–S, this
run's own `10-inventory` scheme). Harvest itself proceeded on **run 3's own
21-partition scheme (P1–P21)**, per `20-harvest/output/partition-ledger.md`
"Scheme provenance (run 4)" — this is a resumed run reconciling at the file
level, not the partition-label level (see AF-0005 in
`output/adjudication-log.md`; partitioning is actor judgment, not
deterministic, and the two schemes are not expected to name-match). **Every
partition reached `done`: 21/21, 0 `pending`** — no unreached partition
this run, so this dimension does not itself surface a meta-level grammar-pressure
moment (contrast the CONTEXT.md's own named example of a `pending` row
doing so). The consequence-class sweep (`20-harvest/output/
consequence-class-sweep.md`) covers **82 of the 83 `decompose`-census
files, 0 blank cells among its 82 rows** — the one missing row is
`.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`, excluded
from harvest by a ruled disposition disagreement (AF-0001/AF-0003/AF-0004
in the adjudication log); the gap is a known, ruled exclusion, not an
uncovered file the sweep failed to reach.

**2. Behavioral precision** (citations *this run* sampled and reverified).
`80-adversarial-review` ran an **exhaustive** (not sampled) re-verification
of all 1333 normalized behavior units' citations against
`reference/sergeant-upstream/`: **1331/1333 citations verified cleanly**
(quote exists, contiguous, at the cited location). The other 2
(BU-0040, BU-0041) had a locator that recovered only half the actual quoted
span — a real precision defect (AF-0006 in the adjudication log), not
fabrication: the quote text and its `quote_hash` were both genuine and
internally consistent for both records. Both locators are corrected as
part of this stage's own adjudication (see `output/adjudication-log.md`
AF-0006), so a citation re-verification run against this run's *current*
committed evidence would find 1333/1333. Reported here as the review's own
measured value (1331/1333), the repair noted separately, per the rule that
this package reports what was measured, not a post-repair re-estimate.

**3. Provenance completeness.** `70-lint` ran `validate-structure.py`'s
`[S8]` provenance check against all 44 candidates: **0 `[S8]` findings** —
every materialized candidate's `provenance.md` cites at least one real
`behavior_id`. Cross-checked independently by `80-adversarial-review`
against `40-classify/output/classifications.ndjson`'s actual 1333-id set:
**1159/1159** `BU-####` citations in the 44 candidates' `provenance.md`
files resolve to a real id, and **958/958** citations in every stage
`CONTEXT.md`'s "Guidance" section do too — zero citations to a
non-existent id in either file class. **Provenance completeness: 44/44
candidates (100%).**

**4. Draft validity.** `70-lint` ran the structural validator against all
44 candidates (none sampled): **every one of the 44 fails with exactly one
defect, `[S12]`** (closing stage names no finalize step) — **0** `[S1]`–`[S11]`,
`[S13]`–`[S15]` findings anywhere, **0** `[S7]` repository-wide findings.
**Mechanical defects found/fixed this run: 0** (none were mechanical — the
one defect present in every candidate is substantive, per
`references/mechanical-vs-substantive.md`'s test, and is left unfixed for
`80`/`90` rather than papered over with a string-match-only repair).
**Substantive defects: 1 systemic root cause, manifesting identically in
all 44 candidates** (traced to `60-draft`'s own template never mentioning
"finalize" — one template gap, not 44 independent defects). This run's own
workflow tree (`.sergeant/workflows/repo-to-icm`) was independently
validated too: **PASS, clean**, `engine-gap records checked: 0` (matches a
direct recount of `classifications.ndjson`). This same `[S12]` root cause
is consolidated into `output/grammar-pressure.ndjson` as a meta-level
engine-gap claim (see that file, `source: meta`, `stage: 60-draft`).

**5. Review convergence.** `80-adversarial-review` recorded **6 findings**
(1 high, 4 medium, 1 low) across 3 of 4 challenge axes (boundary-honesty 2,
structural-self-consistency 3, invention 1; Axis 3 — engine-gap refutation
— had 0 records to re-attempt, since 0 `engine-gap` classifications exist
in this corpus). This stage's own adjudication
(`output/adjudication-log.md`): **5 accept, 1 reject, 0 park**. No accepted
finding was an Axis-3 engine-gap refutation, so no classification record's
`representation` changed as a result of this run's adjudication.

### Not covered here (require `reference-corpus/` comparison)

Per this run's blindness rule, the following five §9.9 dimensions are
**not measured by this run** — they require comparison against an
already-adjudicated reference decomposition, performed later by
independent comparers this run's actors are not (`docs/gauntlet/
contracts/N2.md` Outcome §3):

- behavioral recall
- workflow-boundary agreement
- stage-boundary agreement
- representation agreement
- engine-gap quality

Naming them here, once, plainly: this package does not estimate any of the
five from inside this run to fill the gap — reporting the boundary
honestly is itself part of the "generator preserves uncertainty instead of
inventing confidence" gate item (proposal §22.1 item 9).

## Supporting run statistics (context, not §9.9 dimensions in their own right)

**Run identity.** Subject: `reference/sergeant-upstream` (vendored
subtree, no `.git` of its own). Revision:
`f430cfd4f90174a98adbd7abebbece6303817929`, read from
`reference/UPSTREAM.md`'s provenance table (`00-contract/output/
contract.md` §1) — agrees exactly with the Work's initiating task, no
discrepancy. **Caveat, observed directly while assembling this package
(not itself adjudicated as a finding — it was not named in
`findings.ndjson` and this stage does not originate new adversarial
findings):** `00-contract/output/contract.md` and `10-inventory/output/
inventory.md` both header themselves with this run's own Work id,
`01KZQRGZE32RQ79KT82XTB9MV2`; `20-harvest/output/partition-ledger.md` and
`20-harvest/output/consequence-class-sweep.md` instead header themselves
`01KZQ32J2BAD4P8WJA9SWXRMZ9`, a different id neither file explains. It does
not appear to affect this package's own counts (every number above was
independently recomputed from each file's own content, not copied via the
header), but a reader relying on the header alone to confirm run identity
should know the two headers disagree.

**Extraction coverage.** 1333 behavior units, from 82 distinct source
files (of 83 `decompose`-census files — one ruled exclusion, see Source
coverage above), across 21 done partitions, 0 pending.

**Normalization outcome.** 1333 units in, 1333 out — **0 net split or
merge**. **410/1333 statements rewritten**, 923/1333 carried through
unchanged (verbatim `statement` match against `20-harvest`'s own text).
**Confidence distribution unchanged, record for record**: `high` 1293,
`medium` 39, `low` 1 in both `20-harvest` and `30-normalize` — independently
diffed per-id; 0 records show a confidence shift in either direction.

**Representation mix.** From `40-classify/output/classifications.ndjson`
(1333 records, independently recounted, matches `50-synthesize`'s own
table field-for-field): `stage-context` 795 (59.6%), `helper` 207 (15.5%),
`stage` 174 (13.1%), `agents-invariant` 126 (9.5%), `shared-helper` 23
(1.7%), `workflow` 8 (0.6%), `shared-context` 0, `obsolete-mechanism` 0,
`engine-gap` 0.

**Candidate yield.** 44 workflow candidates (41 clustered by shared
`workflow` field value + 3 standalone single-behavior candidates), 126
`agents-invariant` candidates (listed, not drafted — an `AGENTS.md` change
is the promotion reviewer's call), 23 `shared-helper` candidates (9
contract groups, over-promotion tell checked and not triggered), 0
`shared-context`, 0 `obsolete-mechanism`, 0 `engine-pressure`
(`engine-gap`). `50-synthesize`'s own coverage accounting: all 1333 records
appear in exactly one bucket, 0 missing, 0 double-counted. **A separate,
delegated adjudication:** `50-synthesize` also surfaced an "Unattached
records" defect (4 `helper` records with `workflow=null`/`stage=null`; 16
records naming a `workflow`+`stage` pair, `standard-workflow` /
`monitor-progress`, with no matching `stage`-rung candidate) — `60-draft`
§4.2 carried this forward verbatim and explicitly left the "is this
grammar pressure" call to this stage. **Ruling:** this is a `40-classify`-stage
data-quality defect (incomplete field population on 20 of 1333 records),
not grammar pressure — nothing about the grammar itself prevented these
records from carrying a `workflow`/valid `stage` value (1313 of 1333
records do so correctly), so no rung was "tried and failed for a reason
specific to that rung's own mechanics" (`icm-ladder.md` §6.7's own bar).
Excluded from `output/grammar-pressure.ndjson` on that basis; recorded here
so the delegated call is visible, not silently dropped.

**Draft materialization.** 44 packages materialized under
`.sergeant/drafts/workflows/` (never `.sergeant/workflows/`), 182 stage
directories (174 directly evidenced by a `stage`-rung record + 8
single-stage design inferences for the 5 no-`stage`-member workflow
clusters and 3 standalone candidates — the one licensed bound this stage's
own method allows, recorded per-candidate in each package's own
`provenance.md`, not applied silently). Name-collision check against
`.sergeant/workflows/` (only `repo-to-icm` exists there) and against every
other candidate: clean, 0 collisions.

## Finalize

**Preview** — `python3 .sergeant/workflows/repo-to-icm/scripts/finalize.py --dry-run`, run after `git add .sergeant/workflows/repo-to-icm/*/output/` (step 1), verbatim stdout, exit 0:

```
keep    00-contract/output/contract.md  (promote)
keep    10-inventory/output/inventory.md  (promote)
keep    20-harvest/output/behavior-units.ndjson  (promote)
keep    20-harvest/output/consequence-class-sweep.md  (promote)
keep    20-harvest/output/partition-ledger.md  (promote)
would remove  20-harvest/output/run3-inventory.md
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

would first create a capture commit for 1 currently-staged file(s) not yet in any commit (evidence-preservation guard, GP-5b); nothing modified (dry-run)

dry-run: would finalize 1 file(s); nothing modified.
```

**Reviewed, not merely run:** every declared-`promote` artifact this run's
stages actually wrote is `keep`t; the one `would remove` is
`20-harvest/output/run3-inventory.md` — the copy of run 3's committed
inventory that `20-harvest` pulled in for scheme-provenance only (per
`partition-ledger.md`'s "Scheme provenance (run 4)" section, not this
stage's own declared `output/README.md` manifest for any stage), so its
removal from the working tree (after being captured into history first,
per GP-5b below) is the expected disposition, not a defect. Nothing named
`REFUSED`; nothing slated for removal that was not already staged.

**Script defect observed while running this, noted for completeness (not
itself a `[S12]`-class or grammar-pressure finding — a plain script bug):**
`finalize.py` line 162 raises a `SyntaxWarning` on stderr
(`` "\`" is an invalid escape sequence ``) from an unescaped backtick
inside a Python string literal in its own source. It did not affect the
dry-run's result (exit 0, correct disposition plan) and is recorded here
only because this stage's method requires reviewing the helper's result
rather than trusting it silently.

This preview is deliberately the only `finalize.py` output committed inside
this file: per this stage's own `CONTEXT.md` steps 2–3, this section is
written and re-staged (`git add output/measurement-package.md`) *before*
the real (non-`--dry-run`) invocation runs, so the finalize record lands
inside `finalize.py`'s own capture commit rather than being left as an
uncommitted dirty file afterward. The real invocation's own two commits
(capture, then removal — GP-5b) are this run's actual closing act and are
themselves the durable record of what `finalize.py` did; their commit
messages and `git log` are the place to verify that, not a further edit to
this file.
