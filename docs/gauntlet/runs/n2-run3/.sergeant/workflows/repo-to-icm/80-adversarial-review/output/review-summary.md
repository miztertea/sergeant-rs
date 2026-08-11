# Review summary — `80-adversarial-review`

Fresh execution: no earlier stage's conversation was seen. Everything below
comes from `../CONTEXT.md`, the files it names in its `## Inputs` table, and
the worktree as it stands.

Step 0 check: none of `../00-contract/output/contract.md` or any other
Inputs-table artifact opens with `# AMBIGUOUS — NOT RESOLVED`. Ordinary
review work proceeded.

## Candidates reviewed

All 18 candidate packages `../60-draft/output/draft-report.md`'s manifest
names were checked under Axis 1 (structural checks — publication/layer/
name-collision boundaries) and, for the 11 with `NN-*/` stage directories,
under Axis 2's hidden-translation check (structural comparison of materialized
stage names/ordering against `../40-classify/output/classifications.ndjson`'s
`stage` records — sampled 7 of the 11: `task-intake-and-execution`,
`dispatch-worker`, `dag-orchestration`, `review-finding-routing`,
`validation-gate`, `worker-response-and-recovery`, `project-registration`; all
7 matched their classification records exactly, in the same order, no
splits/merges/promotions found). All 7 zero-stage candidates
(`fleet-monitor-and-reconcile`, `callback-delivery`, `skill-adoption`,
`sergeant-help-query`, `troubleshoot-td-identity`, `cross-repo-planning`,
`undocumented-failure-escalation`) were checked for `stages = []` and no
stray `output/` population — clean, matches `../70-lint/output/lint-report.md`'s
already-recorded substantive `[S3]` findings on these 7 (not re-litigated
here; lint already attributed them correctly to the zero-stage design
choice `../60-draft/output/draft-report.md` records).

## Axes applied

**Axis 1 — Boundary honesty.** Applied to every candidate and every run
output.
- Publication boundary: all 18 candidates live only under
  `.sergeant/drafts/workflows/`, each `index.md` declares `status: draft`,
  no candidate directory exists under `.sergeant/workflows/` (only
  `repo-to-icm` itself is there). Clean.
- Layer boundary: every `NN-*/output/` directory across all 26 materialized
  stage directories contains exactly `README.md`, nothing else. Every `L4`
  Inputs-table row sampled across the candidates points at that same
  candidate's own prior stage `output/`, never another candidate's. Clean.
- Blindness boundary: grepped the literal string `reference-corpus` across
  every run output directory (`00-contract` through `70-lint`'s `output/`)
  and every materialized draft package. Exactly one hit outside the
  expected location: `00-contract/output/contract.md`'s own exclusion
  record (the expected hit, not a finding) and one additional hit in
  `10-inventory/output/inventory.md` that is prose paraphrasing
  `contract.md`'s own conclusion ("no `reference-corpus/` exists...") rather
  than an independent citation into the directory — checked against the
  checklist's "prose repeating the exclusion policy's own wording" carve-out
  and classified as not a finding. No draft package contains any hit. Clean.
- Name-collision boundary: no duplicate names among the 18 candidates, no
  collision with `repo-to-icm` or any pre-existing `.sergeant/workflows/`
  entry. Clean.

**Axis 2 — Invention.** Applied via sampling.
- Citation re-verification: sampled 20 behavior units spanning 12 of the 28
  distinct source files actually present in the corpus (not just the first
  few records) from `../30-normalize/output/behavior-units.normalized.ndjson`.
  For each, re-opened the cited file/locator in `reference/sergeant-upstream`
  at the byte range indicated, confirmed the quote appears contiguously, and
  recomputed `sha256` over the exact stored `quote` bytes. All 20 matched
  both the re-located span and the stored `quote_hash` exactly. No invention
  found in this sample.
- Provenance citation re-verification: parsed every `BU-####` id referenced
  across all 18 candidates' `provenance.md` files (204 distinct ids cited)
  and confirmed every one exists in `../40-classify/output/classifications.ndjson`.
  Zero unresolved citations.
- Rationale discrimination: sampled 12 `helper`/`shared-helper` records and
  10 `stage` records. All 22 gave rung-specific reasoning tied to the
  individual behavior's mechanics (not a generic sentence that would read
  the same pasted onto an adjacent representation) — each `helper`/
  `shared-helper` record's rationale explicitly states the required §6.3
  answer before its §6.5 language, satisfying the ordering rule.
- Over-staging: reimplementation test re-applied by hand to the 10 sampled
  `stage` records (of 31 total). All 10 held up — none read as "a script
  someone reaches for."
- Hidden translation: see "Candidates reviewed" above — 7 of 11 stage-bearing
  candidates checked structurally, all clean.
- One finding recorded (AF-0004): the `worker-response-and-recovery` /
  `recover-worker` stage-context cluster's own rationale chain is circular —
  see `output/findings.ndjson`.

**Axis 3 — Engine-gap refutation.** Applied to both surviving `engine-gap`
records (BU-0137, BU-0227) — the entire population, not a sample. For each:
independently re-attempted rungs 6.1-6.6; `why_each_fails` gives a
mechanically distinct reason per rung for both records (no repeated generic
reason); `observable_acceptance_test` in both describes a checkable
kill/restart/confirm scenario, not a restated feature; no "would be
convenient"/"could be more elegant" language found in any field of either
record. No downgrade found for either record; no findings recorded under
this axis.

**Axis 4 — Structural self-consistency.** All three checks run, blind, no
`reference-corpus/` access.
1. *Self-consistency of counts*: recomputed the distinct `source.path` count
   in `../20-harvest/output/behavior-units.ndjson` directly (28), and
   compared it against `../10-inventory/output/inventory.md`'s 82-file
   `decompose` total, `../20-harvest/output/partition-ledger.md`'s
   partition-`done` membership, and the headline counts stated in
   `../30-normalize`, `../50-synthesize`, and `../60-draft`'s own artifacts.
   This is a severe mismatch — see AF-0001/AF-0002/AF-0003, the three
   highest-severity findings this run produced. Also confirmed
   `consequence-class-sweep.md`'s row count (28, not 82) and
   `partition-ledger.md`'s status column (6 `done`, 15 `pending` — a third
   state never appears, but 15 `pending` rows exist this late in the run).
2. *Hash-vs-stored-quote verification*: recomputed `sha256` directly over
   every one of the 312 stored `quote` fields in
   `../30-normalize/output/behavior-units.normalized.ndjson` (full corpus,
   not just Axis 2's sample) and compared to each record's own stored
   `quote_hash`. Zero mismatches across all 312 records — no
   hash-truncation-style defect found (the class of defect N2 run 2 missed).
3. *Representation-distribution sanity*: computed the full distribution
   over `../40-classify/output/classifications.ndjson` myself: 106
   agents-invariant (34.0%), 87 helper (27.9%), 73 stage-context (23.4%), 31
   stage (9.9%), 8 shared-helper (2.6%), 3 workflow (1.0%), 2 shared-context
   (0.6%), 2 engine-gap (0.6%). `helper`+`shared-helper` = 30.4% of the
   corpus, not a large-majority skew against `stage`+`stage-context`
   (33.3%) — the promotion-only over-staging re-check this skew would
   trigger was not warranted by the numbers. Over-promotion tell: verified
   directly against `../50-synthesize/output/candidates.md`'s own Bucket 5
   grouping (the only grouping that matters for this check, since Bucket 5
   clusters by contract, not file) — none of its 5 groups maps one-to-one
   onto a single source file; the run's own check here is correct and not
   re-litigated. Unused tiers: `obsolete-mechanism` is the only zero-count
   representation in this corpus (`shared-context`=2, `engine-gap`=2, both
   nonzero); `../50-synthesize/output/candidates.md`'s Bucket 6 asserts it
   empty by count alone, without the required per-candidate-unit
   "why it doesn't clear this rung" reasoning — recorded as AF-0005.

## Findings by axis and severity

| Axis | high | medium | low | total |
|---|---|---|---|---|
| boundary-honesty | 0 | 0 | 0 | 0 |
| invention | 0 | 1 | 0 | 1 |
| engine-gap-refutation | 0 | 0 | 0 | 0 |
| structural-self-consistency | 3 | 1 | 0 | 4 |
| **Total** | **3** | **2** | **0** | **5** |

Full records: `output/findings.ndjson` (AF-0001 through AF-0005).

## Headline

The single dominant finding this review surfaced is corpus incompleteness
(AF-0001/AF-0002/AF-0003): `20-harvest` completed only 6 of 21 partitions
(28 of 82 decompose files, 34%) before every later stage — `30-normalize`
through `70-lint` — proceeded as if the resulting 312-record corpus were the
whole subject repository, with no stage anywhere in that chain flagging the
15 `pending` partition rows still sitting in `20-harvest/output/partition-ledger.md`.
This was not caught by `70-lint`'s structural validator (which checks
package shape, not corpus coverage) and does not fit the
`# AMBIGUOUS — NOT RESOLVED` fail-closed propagation path (that path is for
an unresolved *contract*, not an incomplete *harvest* — `00-contract` itself
resolved cleanly). It is, however, exactly the kind of self-checkable
defect Axis 4 exists to catch: nothing here required opening
`reference-corpus/` or knowing a "right" answer, only recomputing counts
this run's own artifacts already contain and noticing they disagree with
each other. Everything downstream of `20-harvest` is honest and internally
consistent *given the 28-file corpus it was actually handed* — citations
verify, hashes verify, provenance resolves, engine-gap records hold up,
representation distribution is sane, hidden-translation checks are clean —
the defect is entirely about scope, not about the quality of what work was
actually done on the (incomplete) scope it covered.
