# 80-adversarial-review summary

Fresh execution, per `../CONTEXT.md` and `references/challenge-checklist.md`.
No upstream Inputs-table artifact opened with `# AMBIGUOUS — NOT RESOLVED`
(checked `../00-contract/output/contract.md`,
`../30-normalize/output/behavior-units.normalized.ndjson`,
`../40-classify/output/classifications.ndjson`,
`../50-synthesize/output/candidates.md`,
`../60-draft/output/draft-report.md`, `../70-lint/output/lint-report.md`
directly), so step 0's fail-closed propagation check does not apply this
run — ordinary review proceeded.

**Result: zero findings.** `output/findings.ndjson` is empty. This reflects
real effort under all three axes, detailed below, not a skipped check.

## Candidates reviewed

All three candidate packages `../60-draft` materialized, per
`../60-draft/output/draft-report.md`'s manifest:

- `dispatch-mode` (`.sergeant/drafts/workflows/dispatch-mode/`) — 1 stage
- `standard-task-workflow` (`.sergeant/drafts/workflows/standard-task-workflow/`) — 5 stages
- `ship-with-no-mistakes` (`.sergeant/drafts/workflows/ship-with-no-mistakes/`) — 0 stages (deliberate, per its own `provenance.md`/`CONTEXT.md`)

## Axis 1 — Boundary honesty (applied to all three candidates + this run's own output tree)

- **Publication boundary.** Confirmed all three candidates live only under
  `.sergeant/drafts/workflows/`, each `index.md` declares `status: draft`,
  and no identically-named directory exists under `.sergeant/workflows/`
  (only `repo-to-icm` itself is there). No violation.
- **Layer boundary.** Every member stage's `output/` directory (6 total,
  across `dispatch-mode` and `standard-task-workflow`) contains exactly
  `README.md` and nothing else — no populated L4 artifact at draft time.
  Every stage `CONTEXT.md`'s `Inputs` table row is correctly tagged: L1
  for the workflow orientation file (first stage only), L4 for an
  upstream same-candidate stage's own `output/`. No violation.
- **Blindness boundary.** Grepped the literal string `reference-corpus`
  across every artifact `00-contract` through `70-lint` produced and every
  materialized draft package. Three hits total in this run's own output
  artifacts, all classified:
  - `00-contract/output/contract.md:67` — the expected single hit in the
    exclusion record (`00-contract` searched, found no such directory
    under this worktree, and recorded that search — prose, not a
    citation). Matches `references/challenge-checklist.md`'s stated
    expectation exactly; not a finding.
  - `10-inventory/output/inventory.md:10` — prose stating the same search
    result under the same blindness rule (`../_config/run-discipline.md`
    §1); not a citation field; not a finding.
  - `20-harvest/output/coverage-note.md:76` — prose asserting this run's
    citations were all hashed directly from `reference/sergeant-upstream`
    and that no `reference-corpus/` file was opened/grepped/referenced;
    not a citation field; not a finding.
  - No hit anywhere is inside a `source.path`/`source.locator`/`quote`
    field, a `provenance.md` citation, or a finding's own `target`/
    `evidence` — no contamination. Also confirmed: this worktree has no
    `reference-corpus/` directory at all (it exists only in the outer
    checkout at `/home/user/sergeant-rs/reference-corpus`, outside this
    worktree's tree), consistent with `00-contract`'s own claim.
- **Name-collision boundary.** `dispatch-mode`, `standard-task-workflow`,
  `ship-with-no-mistakes` are mutually distinct, distinct from
  `repo-to-icm`, and distinct from every name under `.sergeant/workflows/`
  (only `repo-to-icm` exists there). No violation.

## Axis 2 — Invention (applied against the full 108-record corpus and all three candidates' `provenance.md`)

- **Citation re-verification.** Sampled 29 of 108 behavior units spanning
  all 16 distinct `source.path` values in
  `../30-normalize/output/behavior-units.normalized.ndjson` (not just the
  first few — at least one record per source file, two for files with
  more than 3 units). For each: recomputed `quote_hash` against the exact
  bytes at the cited location in `reference/sergeant-upstream` (handling
  the `span_bytes`-truncated long-quote case per
  `../_config/evidence-policy.md` by re-locating the full span from the
  500-char prefix and hashing the complete span, not just the stored
  prefix). All 29 verified: hash matched, quote appeared contiguously at
  the cited locator. No invention found in this sample.
- **Provenance citation re-verification.** Every `behavior_id` cited in
  all three candidates' `provenance.md` files (16 distinct ids across
  `BU-0003`, `BU-0011`–`BU-0016`, `BU-0019`, `BU-0028`–`BU-0034`,
  `BU-0041`) confirmed to exist in
  `../40-classify/output/classifications.ndjson`. No invention found.
- **Rationale discrimination.** Sampled the 6 `stage` records (all of
  them — small population), plus 4 `agents-invariant` and 4
  `stage-context` records and several `shared-helper` records. Each
  `rationale` cited behavior-specific mechanics (e.g. BU-0011 vs BU-0012
  vs BU-0013 each name a different concrete reason tied to their own
  trigger/outcome, not interchangeable boilerplate). No same-text-pasted-
  onto-a-different-record pattern found.
- **Over-staging (reimplementation test).** Independently re-applied to
  all 6 `stage` records. Five of six (`BU-0011`, `BU-0012`, `BU-0013`,
  `BU-0016`, `BU-0019`) map to explicitly numbered steps ("Standard
  workflow" steps 1, 2, 4, 9, and the validate paragraph) in `AGENTS.md`
  itself — the source document already treats these as an ordered
  procedure's named checkpoints, which is strong direct support for
  `stage` over `helper`. `BU-0041` (dispatch-mode's sole stage) similarly
  names a boundary later steps causally depend on. No over-staging found.
- **Hidden translation.** Compared each candidate's actual materialized
  shape (`workflow.toml` `stages` list, `NN-.../` directories,
  `provenance.md`) against `../50-synthesize/output/candidates.md` bucket
  1/2's own candidate descriptions. All three match exactly: `dispatch-
  mode` (1 stage), `standard-task-workflow` (5 stages, same names and
  order, including the same unresolved `40-validate` step-number
  ambiguity carried through transparently rather than silently resolved),
  `ship-with-no-mistakes` (0 stages, explicitly and consistently recorded
  as such in `index.md`, `CONTEXT.md`, `provenance.md`, and
  `workflow.toml`'s own comment). No hidden translation found.
- Also checked: every `workflow`/`stage` record's `alternatives_considered`
  is non-empty (`icm-ladder.md` rule), across all 7 such records
  (`BU-0003`, `BU-0011`, `BU-0012`, `BU-0013`, `BU-0016`, `BU-0019`,
  `BU-0041`). No violation.

## Axis 3 — Engine-gap refutation

Zero `representation: engine-gap` records exist anywhere in
`../40-classify/output/classifications.ndjson` (representation counts:
`agents-invariant` 13, `workflow` 1, `stage` 6, `stage-context` 9,
`shared-helper` 79 — no `engine-gap`, no `obsolete-mechanism`, no
`shared-context`), confirmed independently by counting records
programmatically rather than trusting `../60-draft/output/draft-report.md`'s
bucket-7 claim at face value. This matches both `../60-draft`'s and
`../70-lint`'s own independent counts. This axis is therefore applied
vacuously this run: there is nothing to independently re-attempt rungs
against, no `why_each_fails`/`observable_acceptance_test` to check, and no
"would be convenient" language to catch, because no surviving `engine-gap`
record exists to check. This is stated as a genuine outcome of applying
the axis, not a skipped step.

## Not re-litigated

`[S12]` (no finalize step named, `dispatch-mode` and
`standard-task-workflow`), `[S3]` (`ship-with-no-mistakes`'s deliberate
empty `stages` array), and `[S10]` (`20-harvest/quote.sh` unclassified
executable) are already recorded as substantive, unfixed findings in
`../70-lint/output/lint-report.md` and are not re-recorded here.

## Finding counts by axis and severity

| Axis | high | medium | low | total |
|---|---|---|---|---|
| boundary-honesty | 0 | 0 | 0 | 0 |
| invention | 0 | 0 | 0 | 0 |
| engine-gap-refutation | 0 | 0 | 0 | 0 |
| **Total** | **0** | **0** | **0** | **0** |
