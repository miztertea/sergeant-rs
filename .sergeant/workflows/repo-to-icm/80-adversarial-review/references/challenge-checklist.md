# Adversarial review — the four challenge axes

Layer 3 (stable across runs), local to `80-adversarial-review`. You are a
fresh execution: you have not seen any earlier stage's reasoning, only this
file, `../CONTEXT.md`, the named Inputs, and the worktree as it now stands.
Your job is to find what is wrong, not to confirm what looks plausible —
approach every artifact you read assuming it might contain an error until
you have specifically checked for one.

## Axis 1 — Boundary honesty

- **Publication boundary.** Every candidate package named in
  `../60-draft/output/draft-report.md` actually lives under
  `.sergeant/drafts/workflows/`, never `.sergeant/workflows/`. Its
  `index.md` declares `status: draft`. No candidate directory exists
  identically in both trees (`docs/icm/convention.md` §2 rules 2–3).
- **Layer boundary.** Every candidate's own `NN-.../output/` directory
  contains only `README.md` — a populated artifact there at draft time is a
  layer violation (Layer 4 material fabricated as if a real run happened).
  Every `Inputs` table row is tagged with the layer it actually is
  (`docs/icm/record-shapes.md` §1a rule 2) — an L4 row pointed at a
  same-candidate earlier stage's `output/`, an L3 row at genuinely
  run-independent material, never the reverse.
- **Blindness boundary.** Grep every artifact this run has produced so far
  (`../00-contract` through `../70-lint` outputs, and every materialized
  draft package) for the literal string `reference-corpus`. This is a
  first pass, not the finding itself — **a correctly-executed `00-contract`
  is expected to produce exactly one such hit, in its own `contract.md`'s
  exclusion record** (`00-contract/CONTEXT.md`: "name it in `contract.md`"),
  and that expected hit is not a finding. Do not open `reference-corpus/`
  itself to perform this check; you do need to read enough of *this run's
  own text* around every other hit to classify it, because the grep alone
  cannot tell a citation from a mention:
  - A hit inside a **citation field** — a behavior unit's `source.path`,
    `source.locator`, or a quoted span backing a `quote`/`quote_hash`; a
    `provenance.md` entry; a finding's own `target`/`evidence` naming a
    location inside `reference-corpus/` — means this run's evidence is not
    actually independent of the answer key. That is a contamination
    finding, highest severity.
  - A hit that is **prose repeating the exclusion policy's own wording**
    (e.g. a summary that copies `contract.md`'s exclusion line, or restates
    this checklist's or `../_config/run-discipline.md`'s own text) is not
    itself a finding — it did not have to open the directory to write that.
  - If you cannot place a hit in either bucket from context alone, do not
    guess which it is; record it as a `medium`-severity finding describing
    exactly what you found and why it was ambiguous, rather than silently
    dropping it or inflating it to `high` on suspicion alone.
- **Name-collision boundary.** No candidate workflow name collides with
  `repo-to-icm` itself, with any other candidate this run produced, or with
  any name already under `.sergeant/workflows/`.

## Axis 2 — Invention

- **Re-verify a sample of citations.** Pick a genuine, not cherry-picked-easy
  sample of behavior units (aim for coverage across different source files,
  not just the first few records) from
  `../30-normalize/output/behavior-units.normalized.ndjson`. For each: does
  the quoted span exist, contiguous, at the cited `source.locator` in
  `source.path` in the subject repository at the revision
  `../00-contract/output/contract.md` pins? Recompute `quote_hash` yourself
  against the exact bytes (`printf '%s' "$QUOTE" | sha256sum`, per
  `../_config/evidence-policy.md`) for at least several of them. A hash that
  does not verify, or a quote that does not appear contiguously where cited,
  is invention — record it as a finding regardless of the unit's
  `confidence` value.
- **Re-verify provenance citations.** For each candidate package's
  `provenance.md`, confirm every cited `behavior_id` actually exists in
  `../40-classify/output/classifications.ndjson`. A citation to a
  non-existent id is invention.
- **Check rationale discrimination.** For a sample of classification
  records in `../40-classify/output/classifications.ndjson`, ask: does
  `rationale` actually explain why *this* rung and not an adjacent one, or
  would it read the same pasted onto a different `representation` value? The
  latter is a violation (`docs/icm/record-shapes.md` §4).
- **Over-staging.** For every `representation: stage` record, re-apply the
  reimplementation test from `../_config/icm-ladder.md` §6.3 yourself: if
  the mechanism behind this behavior were replaced tomorrow, would the
  checkpoint still exist? If the honest answer is "no, this is just a
  script someone reaches for," that is an over-staging finding — the record
  should have classified as `helper`, not `stage`.
- **Hidden translation.** Compare each materialized draft package's actual
  `NN-.../CONTEXT.md` content against the classification record(s) and
  synthesis candidate that supposedly justify it
  (`../50-synthesize/output/candidates.md`). Did `60-draft` silently draw a
  different boundary than what was classified/synthesized — split a stage
  that was one candidate, merged two that were separate, or promoted a
  `stage-context` attachment into its own stage — without recording why?
  That is hidden translation even if the resulting shape looks reasonable;
  the violation is the silence, not necessarily the shape.

## Axis 3 — Engine-gap refutation

For every `representation: engine-gap` record surviving through
`../40-classify` and `../50-synthesize`'s engine-pressure candidates:

- Independently re-attempt every rung 6.1–6.6 in
  `../_config/icm-ladder.md` yourself, including ones the record already
  lists as attempted — try harder than the classifying stage did. If you
  find a lower rung that would actually work, write that up as a finding
  that downgrades the claim (name the rung and how it would work).
- Check `why_each_fails`: is each listed rung's failure reason specific to
  that rung's actual mechanics, or is the same reason repeated across every
  rung? A repeated generic reason ("not powerful enough") across every rung
  is itself disqualifying (`docs/icm/record-shapes.md` §5 rule 3), not
  merely weak.
- Check `observable_acceptance_test`: does it describe a checkable scenario
  after the capability would exist, or does it just restate the desired
  feature ("the engine supports nested workflows" does not qualify, per
  rule 5)?
- Check every field for "would be convenient" / "could be more elegant" in
  any form — this disqualifies the claim outright per rule 4, regardless of
  how the other fields read.

## Axis 4 — Structural self-consistency (blind, no reference needed)

N2 run 2's own review pass was genuinely effortful under axes 1–3 and still
missed three cheap, purely self-checkable defects sitting inside its own
declared scope — one hash-truncation self-consistency defect in 14.8% of
the corpus, a headline coverage count contradicted by the same document's
own file list two sentences earlier, and a representation skew (`shared-
helper` ~9x overrepresented) that a promotion-only over-staging check could
never see (`docs/gauntlet/runs/n2-run2/comparison-c1-coverage-recall.md`
§4.1, `comparison-scorecard.md` D-1/D-6, `grammar-pressure-report.md` GP-6).
None of the three checks below requires opening `reference-corpus/` or
knowing the "right" answer — they check this run's own artifacts against
each other and against internal arithmetic.

### 1. Self-consistency of counts

Every count one of this run's own artifacts asserts *about another of this
run's own artifacts* must be independently recomputed from the artifact
itself, not copied from whatever stated it. A mismatch is a finding
regardless of which side turns out to be right — the point is that nothing
downstream should have to guess which number to trust.

- Recompute the number of distinct `source.path` values in
  `../20-harvest/output/behavior-units.ndjson` yourself (a plain count over
  the file, not a number quoted from any other artifact) and compare it
  against: `../10-inventory/output/inventory.md`'s total `decompose` file
  count, `../20-harvest/output/partition-ledger.md`'s `done` partition
  membership, and any headline coverage number stated in
  `../30-normalize/output/behavior-units.normalized.ndjson`,
  `../50-synthesize/output/candidates.md`, or `../60-draft/output/
  draft-report.md`. Every one of these should agree with your own recount;
  any that doesn't is a finding, naming both the recomputed value and the
  contradicting stated value and exactly where each was found.
- Confirm `../20-harvest/output/consequence-class-sweep.md` has exactly one
  row per `decompose` file named in `../10-inventory/output/inventory.md`,
  with no blank cells (per `../20-harvest/references/
  consequence-class-checklist.md`) — a missing row or a blank cell is a
  finding under this axis, not a silent gap.
- Confirm `../20-harvest/output/partition-ledger.md` shows every partition
  `done`, in the same order and under the same names
  `../10-inventory/output/inventory.md` recorded — a `pending` row this far
  into the run, or a partition named in one file but not the other, is a
  finding.

### 2. Hash-vs-stored-quote verification (not just span re-verification)

Axis 2's citation re-verification (re-locating the full span from the
source file and hashing it) confirms the cited span is *real* — but by
design it never checks whether the record's own **stored** `quote` field
still hashes to its own stored `quote_hash`, so it structurally cannot
catch a record whose `quote` has silently drifted from what its
`quote_hash` was actually computed over (e.g., truncation applied after the
hash, or a re-typed span). For the same sample Axis 2 draws for citation
re-verification, **additionally** recompute `sha256` directly over the
stored `quote` field's raw bytes (JSON-unescape it first — a literal
newline byte and the two characters `\`+`n` in the JSON text must hash
identically, per `../_config/evidence-policy.md`) and compare to the
record's own stored `quote_hash`. A record can pass span-reopening and
still fail this — that mismatch is itself a finding, independent of and in
addition to whatever Axis 2 found for the same record.

### 3. Representation-distribution sanity vs the ladder

Compute the full distribution of `representation` values across
`../40-classify/output/classifications.ndjson` — counts and percentages,
computed by you, not copied from `../50-synthesize/output/candidates.md`'s
own summary (that would just be re-checking count self-consistency, not
this). Sanity-check the shape against the ladder itself, blind — no
reference numbers needed for any of these:

- **Skew toward machinery.** If `helper` + `shared-helper` together account
  for a large majority of the corpus while `stage` + `stage-context`
  together account for a small minority, that skew is itself a finding
  requiring you to re-examine a sample of the `helper`/`shared-helper`
  records for the §6.3-before-§6.5 rung-ordering error
  (`../_config/icm-ladder.md` §6.3's "the question must actually be
  answered" note): does each sampled record's `rationale` actually state
  an explicit §6.3 answer before its §6.5 reasoning, or does it jump
  straight to "this is deterministic machinery"? The latter, found on
  re-check, is an `invention`-or-`structural-self-consistency` finding
  either way — record it under this axis since the trigger was the
  distribution shape, not an individual citation.
- **The over-promotion tell.** Group the `helper`/`shared-helper` records
  by the `source.path` of their originating behavior unit (from
  `../30-normalize/output/behavior-units.normalized.ndjson`, joined by
  `behavior_id`). If a group's membership maps one-to-one onto a single
  source file — this file's helper records, and only this file's, form
  their own cluster, with every other covered file doing the same — that
  is file-shape mirroring, not behavior-shape clustering
  (`../_config/icm-ladder.md` §6.6), and is a finding on its own even
  before reading any individual `rationale`.
- **Unused tiers.** If `shared-context`, `obsolete-mechanism`, or
  `engine-gap` are all zero across a corpus of any real size, that is
  itself the thing to challenge, not a null result to accept silently —
  name at least a few candidate units (from any representation) whose
  underlying behavior comes closest to each unused rung's own question and
  record, for each, why it does not actually clear that rung. A rung that
  produced zero records because you tried and it genuinely never fit is a
  clean result; a rung that produced zero because nothing was ever
  routed to its question is a finding.

## Recording findings

Write every finding to `output/findings.ndjson` (one per line) per
`../CONTEXT.md`'s record shape, tagged with the axis it came from. A
candidate/record you genuinely checked under an axis and found nothing wrong
with is not itself a finding — but `output/review-summary.md` must state
which candidates and which axes were actually applied, so a later reader can
tell diligence happened rather than an axis being silently skipped.
