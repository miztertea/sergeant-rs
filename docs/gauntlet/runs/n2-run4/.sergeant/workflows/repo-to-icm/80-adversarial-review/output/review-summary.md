# 80-adversarial-review — review summary

Fresh-execution adversarial review of this run's corpus: 21 harvested
partitions, 1333 classification records, 44 materialized draft workflow
packages under `.sergeant/drafts/workflows/`. All four axes in
`references/challenge-checklist.md` were applied against **every**
candidate package and **every** upstream artifact named in this stage's own
Inputs table — nothing was sampled down for volume on the mechanical
checks (citation/hash verification, count recomputation, structure
scanning, provenance-id resolution all ran over the full corpus, not a
subset); the genuinely judgment-based checks (rationale discrimination,
over-staging, hidden translation, engine-gap re-attempt) used deliberate,
diverse sampling as the checklist itself expects for that kind of check,
recorded per-check below.

6 findings recorded in `output/findings.ndjson`: 1 high, 4 medium, 1 low.
The high-severity finding (AF-0001) is a genuine blindness-boundary
contamination and is the most consequential result of this review — it
also explains three of the four medium/low findings (AF-0002–AF-0004),
which are its downstream, independently-checkable symptoms.

## Axis 1 — Boundary honesty

**Publication boundary.** Checked all 44 candidates: every one lives only
under `.sergeant/drafts/workflows/`, none exists identically under
`.sergeant/workflows/` (only `repo-to-icm` exists there), and every
`index.md` declares `status: draft` (verified by direct grep across all 44
— zero non-`draft` hits). Clean.

**Layer boundary.** Checked all 44 candidates' 182 `NN-.../output/`
directories: every one contains only `README.md` (`find
.sergeant/drafts/workflows -path '*/output/*' -type f ! -name README.md`
returns nothing) — no Layer 4 material fabricated at draft time. Spot-swept
every stage `CONTEXT.md`'s Inputs table for L3 rows pointing at an
`output/` path or L4 rows pointing at `_config/`/`references/` paths — zero
mistagged rows found across the corpus. Stage-directory counts for all 44
candidates were independently recounted from the filesystem and compared
against `60-draft/output/draft-report.md`'s manifest table — all 44 match
exactly. Clean.

**Blindness boundary.** Grepped the literal string `reference-corpus`
across every artifact `00-contract` through `70-lint` produced, plus every
materialized draft package (drafts: zero hits). Every hit outside
`_config/`, `references/`, and `CONTEXT.md` files (Layer 1/3 authored text
restating the exclusion policy's own wording — not findings, per the
checklist) was individually read and classified:

- `00-contract/output/contract.md`, `10-inventory/output/inventory.md`,
  `20-harvest/output/run3-inventory.md`: all "no `reference-corpus/`
  directory was found; the blindness rule is vacuous here" statements —
  legitimate, not findings.
- `20-harvest/output/partition-ledger.md`: **contamination** —
  see **AF-0001** (high). This is the one hit that names actual
  reference-corpus *content* (a source-inventory line number, a
  disposition, and an N1 adjudication id) rather than the exclusion
  policy's own wording, and that content was used to decide this run's own
  harvest scope.
- Cross-checked this against `00-contract/output/contract.md`'s own explicit
  ruling that this run carries no measurement framing — see **AF-0002**
  (medium), an internal contradiction between the contract and the harvest
  stage's actual behavior that no stage in between reconciles.

**Name-collision boundary.** Checked all 44 candidate names against each
other (zero duplicates), against `.sergeant/workflows/` (only
`repo-to-icm` present, no candidate is named `repo-to-icm`), and confirmed
`draft-report.md`'s own claim of a clean collision check. Clean.

## Axis 2 — Invention

**Citation re-verification.** Ran an exhaustive (not sampled) re-verification
over all 1333 normalized behavior units: for every record, re-opened the
cited `source.path` at `source.locator` in the actual
`reference/sergeant-upstream/` subtree and confirmed the stored `quote`
appears contiguously there (substring match against the reconstructed line
range, to allow for a locator that legitimately spans more context than the
quote itself). Result: 1331/1333 contiguous and correctly located; 2 records
(BU-0040, BU-0041) have a locator that is short by exactly half the actual
quoted span — **AF-0006** (medium). No case of a quote that does not exist
in the file at all, and no case of a quote existing but at a materially
wrong location.

**quote_hash recomputation against the actual file span.** For every
non-truncated record (1145 of 1333; the other 188 carry `span_bytes` and
quote a 500-character prefix of a longer span, per evidence-policy.md),
recomputed `sha256` over the exact byte range identified by the locator.
5 records showed a raw span/hash mismatch purely because their locator's
line range is wider or narrower than the record's own quote (not itself
invention — see next check, which isolates this). BU-0040/BU-0041 are the
two already recorded as AF-0006; the other 3 (BU-1263, BU-1282, BU-1322)
have a locator only 1 line wider than the quote (an adjacent blank or
label line included in the cited range) — checked individually and judged
within evidence-policy.md's "precise enough" bar, not filed as findings.

**Provenance citation re-verification.** Checked every `BU-####` id cited
in all 44 candidates' `provenance.md` files (1159 distinct citations) and
every stage `CONTEXT.md`'s "Guidance" section (958 citations) against
`40-classify/output/classifications.ndjson`'s actual `behavior_id` set
(1333 valid ids). Zero citations to a non-existent id, in either file
class. Clean.

**Rationale discrimination.** Sampled across every representation value
(4 records each of `helper`, `shared-helper`, `stage`, `stage-context`,
`agents-invariant`, `workflow` — 24 records) and read each rationale
against its neighbors. Every sampled rationale is specific to the actual
question answered for that record (explicit §6.3 language, or the
6.1/6.2/6.4 equivalent), not boilerplate that would read the same pasted
onto an adjacent representation. Went further than sampling for the one
check the ladder itself calls out by name (N2 run 2's own failure mode,
`icm-ladder.md` §6.3): searched **all 1025** `helper`/`shared-helper`/
`stage-context` records for a rationale that never mentions "6.3" at all —
zero found. This is a genuinely strong, corpus-wide result on exactly the
defect class this workflow's own governing text names as the highest-risk
one; recorded here as a real, clean outcome, not left implicit.

**Over-staging (reimplementation test).** Sampled 14 of the 174 `stage`
records across different candidates (`BU-0295`, `BU-0153`, `BU-0595`,
`BU-1293`, `BU-0031`, `BU-0060`, `BU-1055`, `BU-0103`, `BU-0380`, `BU-1148`,
plus the 4 initially flagged by an automated sweep for missing
"checkpoint"/"durable" language, `BU-0006`/`BU-0008`/`BU-0010`/`BU-0028`,
which turned out to phrase the same reimplementation-test substance
differently). Applied the test personally to each: in every sampled case,
the checkpoint plausibly survives a hypothetical reimplementation and isn't
"just a script someone reaches for." One record (`BU-1148`,
`triage/01-operate-state-machine`) was checked specifically because it
looked like it might be a sub-detail promoted to its own stage; confirmed
via `50-synthesize/output/candidates.md` that it is in fact the sole
stage-anchoring record for that checkpoint, with the genuinely
sub-checkpoint transition rules (`BU-1147`, `BU-1149`, `BU-1150`) correctly
attached as `stage-context`. No over-staging found in the sample.

**Hidden translation.** For all 44 candidates, extracted the stage-name
list `50-synthesize/output/candidates.md` records for each workflow and
compared it (exact names, exact order) against the drafted
`NN-<stage-name>/` directories under `.sergeant/drafts/workflows/`. 36 of
44 candidates have a stage list in `candidates.md` to compare against, and
all 36 match exactly — no split, merge, or reorder. The remaining 8
(`adopt-external-skill`, `check-repo-status`, `fleet-status-listing`,
`invoke-grill-with-docs`, `list-projects`, `list-tasks`,
`register-project`, `treehouse-init`) have no `candidates.md` stage entry
by construction — these are exactly the 8 design-inference candidates
`draft-report.md` §2 discloses (5 no-`stage`-member workflow clusters plus
3 standalone single-behavior candidates), each given one design-inference
stage per that disclosed, licensed bound. Spot-checked 3 of the 36 in full
detail (`code-review`, `dispatch-mode`, `worker-lifecycle` — 3, 19, and 26
stages respectively) by reading the actual `candidates.md` numbered list
against the actual drafted directory names side by side; all three match
exactly. No hidden translation found.

## Axis 3 — Engine-gap refutation

Zero `representation: engine-gap` records exist in this corpus (confirmed
by direct count over `40-classify/output/classifications.ndjson`, not
copied from `candidates.md`'s or `draft-report.md`'s own stated "0" — see
Axis 4 below for the independent recount). There is therefore nothing to
re-attempt lower rungs against under this axis's literal instructions. This
null result is cross-checked, not accepted silently — see the "Unused
tiers" check under Axis 4, which specifically searches for behavior units
that came close to needing `engine-gap` and confirms why each one correctly
resolved lower on the ladder instead (two of the three named candidates
there, `BU-0427`/`BU-0494`, are exactly the kind of record — cross-process
concurrency exclusion — that could plausibly need runtime-owned ordering;
both are correctly classified `stage`/`stage-context` because a
deterministic stop-then-dispatch script ordering already achieves the
guarantee).

## Axis 4 — Structural self-consistency

**Count self-consistency.** Recomputed the number of distinct `source.path`
values in `20-harvest/output/behavior-units.ndjson` directly (82) and
compared against every stated count this run's own artifacts assert about
it:
- `10-inventory/output/inventory.md`'s stated `decompose` total: **83** —
  mismatch, **AF-0003** (medium).
- `20-harvest/output/consequence-class-sweep.md`'s row count: **82** —
  agrees with the recount, but see AF-0004 below for why that's still a
  finding relative to inventory.md.
- `30-normalize/output/behavior-units.normalized.ndjson`,
  `50-synthesize/output/candidates.md`, `60-draft/output/draft-report.md`:
  all state **1333** total classification/behavior-unit records — this
  matches a direct recount of `40-classify/output/classifications.ndjson`
  (1333 lines) exactly. Clean on this number.
- `50-synthesize/output/candidates.md`'s own representation-count table
  (workflow 8, stage 174, stage-context 795, agents-invariant 126,
  shared-helper 23, shared-context 0, helper 207, obsolete-mechanism 0,
  engine-gap 0, total 1333) was independently recomputed from
  `40-classify/output/classifications.ndjson` directly (not copied) and
  matches exactly, field for field. Clean.

`consequence-class-sweep.md` row-count-per-decompose-file check: 82 rows,
no blank cells, no duplicate rows (recomputed directly) — but inventory.md
names 83 decompose files, so one is missing a row — **AF-0004** (medium),
the same root file as AF-0003 (`.agents/skills/diagnosing-bugs/scripts/
hitl-loop.template.sh`, excluded per the AF-0001 ruling).

`partition-ledger.md` "every partition done, same names/order as
inventory.md" check: **fails** — the ledger uses run 3's P1–P21 naming
throughout, inventory.md uses this run's own A–S naming, and the two label
sets are entirely disjoint — **AF-0005** (low; the ledger discloses and
explains the discrepancy, but the checklist's literal comparison still
finds zero correspondence between the two files' partition columns).

**Hash-vs-stored-quote verification.** For every one of the 1145
non-truncated (no `span_bytes`) records, recomputed `sha256` directly over
the record's own **stored** `quote` field's raw bytes (JSON-unescaped) and
compared to the record's own stored `quote_hash` — independent of and in
addition to the span-reopening check under Axis 2. **Zero mismatches**
across all 1145 records. This is the exact check `references/
challenge-checklist.md` §Axis 4.2 calls out as the one Axis 2's
span-reopening structurally cannot perform (a record whose `quote` drifted
from what its `quote_hash` was actually computed over) — run exhaustively,
not sampled, and clean.

**Representation-distribution sanity vs the ladder.**
Recomputed the full distribution directly from
`40-classify/output/classifications.ndjson` (not copied from
`candidates.md`'s summary): `stage-context` 795 (59.6%), `helper` 207
(15.5%), `stage` 174 (13.1%), `agents-invariant` 126 (9.5%), `shared-helper`
23 (1.7%), `workflow` 8 (0.6%).

- *Skew toward machinery* (the specific N2-run-2 failure shape): does
  **not** apply here — `helper`+`shared-helper` is 17.3% of the corpus,
  while `stage`+`stage-context` is 72.7%, the opposite direction from the
  pattern the checklist warns about. Combined with the corpus-wide §6.3
  mention check above (Axis 2), this is a genuinely strong signal the
  rung-ordering defect that caused N2 run 2's miss was avoided here.
- *Over-promotion tell*: `draft-report.md` §4.1 reports its own check (9
  contract-clustered `shared-helper` groups, none mapping onto one whole
  source file's full unit set). Independently re-derived this a different
  way: scanned every stage `CONTEXT.md`'s "Guidance" section for
  `stage-context` `BU-####` ids reused verbatim across more than one
  candidate's stages (the file-mirroring tell would also show up as
  identical judgment content copy-pasted across workflows). Zero ids
  reused across more than one candidate. Clean by both methods.
- *Unused tiers*: `shared-context`, `obsolete-mechanism`, and `engine-gap`
  are all 0. Per the checklist, this null result must be tested, not
  accepted silently. Named and checked specific near-miss candidates for
  each:
  - `obsolete-mechanism`: `BU-0205` (`mise.toml`, "Install removes legacy
    `oc-inject` links (a deleted feature)") was the strongest candidate
    found by keyword sweep (`legacy`/`deprecated`/`obsolete` across all
    normalized statements/notes). Checked its actual classification
    (`stage-context`, correctly attached to `install-symlinks`): the
    behavior itself — safely cleaning up traces of a *different*,
    already-dead feature — is not itself an obsolete mechanism; the
    *current* symlink-guard mechanism it describes is live and unreplaced.
    Correctly does not clear the rung.
  - `engine-gap`: `BU-0427`/`BU-0494` ("two Claude processes are never
    running concurrently against the same worktree") is the kind of
    cross-process exclusion guarantee that could plausibly need
    runtime-owned ordering. Checked: both are correctly classified
    `stage`/`stage-context` because a deterministic stop-then-dispatch
    script ordering (verified present in `bin/sgt-respond` and
    `bin/sgt-recover`) already achieves the guarantee at a lower rung — no
    runtime-owned capability is actually required.
  - `shared-context`: systematic cross-file scan (above) found zero
    stage-context judgment content reused verbatim across candidates.
    Manually compared the two closest candidates found by topic
    (`dispatch-mode/19-detect-model-substitution` vs
    `worker-lifecycle/01-resume-model-pin-reverification`, both
    model-pin-related): confirmed they encode genuinely distinct
    contracts (silent-substitution detection vs. pin-honoring-on-resume),
    not the same judgment duplicated under two names.
  - All three are recorded here as a clean result (a rung that produced
    zero because it genuinely did not fit, not because nothing was routed
    to its question) — no finding filed for this bullet.

## Out-of-scope note carried forward, not re-litigated

`60-draft/output/draft-report.md` §5 and `70-lint/output/lint-report.md`
both already flag a systemic, unfixed `[S12]` defect (every one of the 44
candidates' closing stages names no finalize step, traced to a gap in
`60-draft`'s own template) and explicitly hand it to `80`/`90`. It is noted
here for visibility but not duplicated into `findings.ndjson`: it does not
fit any of this stage's four challenge axes (it is a template-completeness
gap, not a boundary/invention/engine-gap/count-consistency defect), and
`references/mechanical-vs-substantive.md` already gave it its disposition
of substantive-and-unfixed. `90-reconcile` has both artifacts naming it
directly.

## Candidates and axes actually applied

All 44 materialized candidates were covered by the corpus-wide mechanical
checks (publication/layer boundary, stage-count/name reconciliation,
provenance-id resolution). 3 candidates (`code-review`, `dispatch-mode`,
`worker-lifecycle`) received full manual side-by-side hidden-translation
review; all 44 received the automated exact-match version of the same
check. 14 `stage` records across 10+ distinct candidates received the
manual reimplementation test. Axis 3 had zero records to re-attempt; its
null result is cross-validated by the Axis 4 unused-tiers check.
