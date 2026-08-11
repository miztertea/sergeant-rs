# Generator v2 vs v1 — Delta Verdict

Run 3 (`docs/gauntlet/runs/n2-run3/`, generator v2) against run 2
(`docs/gauntlet/runs/n2-run2/`, generator v1), both measured against the
frozen reference corpus (`reference-corpus/`, `FROZEN.md` v1) at the same
pinned subject SHA `f430cfd4f90174a98adbd7abebbece6303817929`. Companion:
`comparison-scorecard.md` (this run), which carries the §9.9 dimensions and
the §22.2 answer in full; this document answers the narrower question of
whether the v1→v2 changes worked.

**Headline verdict: v2 is a real improvement, not a rescoping artifact — and
the improvement is concentrated exactly where v2's changes were aimed.**
Every v1→v2 change targeted a named run-2 finding, and each landed:
the consequence-class sweep closed the §22.2 failure, the §6.3-before-§6.5
ladder ordering collapsed the helper skew, finalize-aware drafting closed
`[S12]`, the adversarial-review axis went from null to real, and the
partition-checkpoint protocol produced honest resumable state. What v2 did
*not* fix is the class of defect that requires reading source prose back
against the corpus — and one v1 failure mode (the `bin/`-script
under-promotion) survives with the question now honestly asked and the same
wrong answer reached.

---

## 1. Per-turn coverage: 16 → 28 files

| | v1 (run 2) | v2 (run 3) |
|---|---|---|
| Files decomposed in one harvest turn | 16 | **28** |
| Behavior units | 108 | **312** |
| Corpus coverage | 16/136 (11.8%) | 28/82 (34.1%) |
| Draft packages | 16 | **18** |
| Unreached files | 120, **record destroyed by the run's own finalize step** | 54, **named per-partition in a durable ledger** |
| Harvest stage duration | — | 37m 24s of the run's 2h 2m 47s (`run-manifest.md`) |

**Verdict: the per-turn wall moved, but it is still a wall.** 28 files in one
`20-harvest` execution is 1.75× v1's throughput at 2.9× the unit count — the
denominators differ because the inventory was re-partitioned (82 `decompose`
files in 21 named partitions, versus v1's 136-file flat list), so the file
counts are comparable per-turn but the percentages are not directly
comparable. The material change is not the count: it is that v2 stops at a
declared boundary and leaves a ledger, where v1 stopped wherever it ran out
and then lost the note saying so.

The run stopped after 6 of 21 partitions. `20-harvest` did not fail; it
completed its stage having done what one turn allows and recorded the rest as
`pending`. That is the designed behavior, and it is the precondition for
everything in §7.

---

## 2. Recall within comparable scope

The two runs' file sets overlap on only three files (`AGENTS.md`,
`README.md`, `bin/sgt-dispatch`), so run-level recall figures are not a
controlled comparison. The controlled slice is:

| Scope | v1 recall | v2 recall |
|---|---:|---:|
| **`AGENTS.md` + `README.md` (103 ref units, both runs read both files)** | **51.5%** (53/103) | **≥ 78.6%** (81/103) |
| Whole in-scope corpus (different denominators: 165 vs 385 ref units) | 47.3% (78/165) | ≥ 56.9% (219/385), floor |
| Guardrail-class hand sample of the lexically-unmatched set | not performed | 56/63 (88.9%) |

**Verdict: recall improved on identical input, by ~27 points on the two files
both runs read.** This is the number that survives the rescoping objection —
same files, same reference units, different generator. The corpus-wide
figures are floors on both sides and the v2 floor is loose: independent
re-matching with a stricter tokenizer produced 158/385 (41.0%), and hand
review of 63 lexically-unmatched guardrail candidates found 56 genuinely
present (`BU-P8-094`→`BU-0172`, `BU-P8-098`→`BU-0178`,
`BU-P5-115`→`BU-0123`, `BU-P6-088`→`BU-0245`, `BU-P1-048`→`BU-0048` are all
near-verbatim matches the matcher scored below threshold). No point estimate
is claimed.

Two confirmed within-scope recall holes remain, both narrow:
`schema/project.yaml.example` **0/4** (only its `dag:` block was read →
`BU-0201`–`BU-0203`) and `docs/what-is-sergeant.md` **7/14** (the
Project/Repository/Task/Fleet core-concept definitions `BU-P1-101`–`104` are
absent). A third is a repeat from v1: `AGENTS.md`'s procedural-skills routing
table (L107-121, ref `BU-P1-132`–`BU-P1-137`) again produced nothing of its
own, with only `BU-P1-136` partially recovered via `BU-0123`.

---

## 3. The 11-miss list: disposition

Run 2's §22.2 failure was 11 reference behaviors with confirmed safety,
identity, recovery, delivery or human-decision consequence, silently absent
from files it had actually read.

| Disposition | Count | Detail |
|---|---:|---|
| **Captured in v2** | **3 / 3 that entered scope** | `BU-P1-050` → **`BU-0050`** (standing authorization never authorizes risk acceptance, gate skipping, force ops, secret exposure, or destruction of preserved state — near word-for-word, double-filed in the sweep under Safety and Human-decision); `BU-P1-016` → **`BU-0018`** (direct mode never edits several repos in one checkout nor bypasses instructions/ownership/review independence/shipping gates — verbatim); `BU-P6-127` → **`BU-0288`** + **`BU-0296`** (rollback scoped to a pane this invocation created; disarmed on full success — correctly split into two independently-true halves, both swept) |
| **Out of scope, ledger-declared** | 8 | `BU-P6-040`, `BU-P6-073`, `BU-P6-079`, `BU-P6-098`, `BU-P6-103`, `BU-P6-108`, `BU-P6-113`, `BU-P6-114`, `BU-P6-115`, `BU-P6-138`, `BU-P6-139`, `BU-P6-140` — every source file (`bin/sgt-drain-force`, `sgt-cleanup`, `sgt-watch`, `sgt-interactive-worker`, `sgt-recover`, `sgt-respond`, `sgt-wake`) is in partitions P7–P10, `pending` in `partition-ledger.md` and named by `AF-0001` |
| **Newly silent in v2** | **0** | See §3.1 |

### 3.1 The mechanism worked, and it is worth being precise about why

The consequence-class sweep (`20-harvest/references/consequence-class-checklist.md`,
written explicitly against run 2's §22.2 finding) is a mandatory second lens
over already-open text, one row per file, five columns, never blank. It
produced 28 rows × 5 cells with no empty cell. Independent hunting over all
385 in-scope reference units found **no confirmed-consequence behavior
silently absent** — the §22.2 criterion is **MET**, inverting run 2's NO.

All three candidates the comparers nominated as new silent misses dissolve on
inspection, and in each case the sweep itself is part of why:

- `BU-P7-003` (atomic graph republish, Delivery) → present as **`BU-0245`**
  (`bin/sgt-graphify` L7-10, near-verbatim against the reference's own twin
  `BU-P6-088`), plus `BU-0198` and `BU-0254`; **and cited in the sweep's
  `bin/sgt-graphify` Delivery column.**
- `BU-P6-124` (dispatch-time model-tuple precedence, Identity) → present as
  **`BU-0058`** (precedence chain), **`BU-0074`** (deliberately no
  project-level default), **`BU-0059`** (validated before any intent file,
  task, worktree, or fleet state); **and cited in the sweep's `AGENTS.md`
  Identity column.**
- `BU-P7-004` (per-repo instruction override) → present as **`BU-0053`**.

**But the sweep has two confirmed cross-referencing defects**, and they are
the fragile edge of this result:

1. `schema/project.yaml.example`, Delivery = `swept, none found` — false. The
   file's own lines 90-92 state the atomic-replace guarantee. The criterion
   survives only because the same behavior was extracted from
   `bin/sgt-graphify`.
2. `AGENTS.md` cites `BU-0018` in neither its Safety nor its Human-decision
   column, despite `BU-0018` being one of the two sharpest misses the entire
   checklist was written to prevent.

The behaviors exist; the sweep's own cross-referencing failed. Left unfixed,
this degrades into genuine silent absence the moment a consequence-class
behavior's *only* source is the file whose cell is wrong.

---

## 4. Precision: held?

**Yes, exactly — and verified at full population rather than sampled.**

| Check | v1 | v2 |
|---|---|---|
| Quote is a literal contiguous substring at the pinned SHA | 108/108 | **312/312** |
| `sha256(quote)` matches stored `quote_hash` | 108/108 | **312/312** |
| Records carrying `span_bytes` (v1's D-2 dispute) | 16 | **0** |
| Invented behaviors | 0 | **0** |
| Artifact-level invention | **1** (run 2's D-1: an unsupported "18 files / 118 not reached" coverage headline) | **0** |

The 2.9× unit increase was not bought by relaxing evidence discipline.
Unmatched *generated* units are, as in v1, finer-grained real guard clauses —
`BU-0246` (repo-name charset rejection), `BU-0247` (`graphify.output` never
equal to a source repo path) — classified **legitimate alternate
decomposition**, not invention. v2 additionally eliminated the long-quote
convention that produced run 2's only precision *dispute*: zero records use
`span_bytes`, so that failure mode cannot recur.

---

## 5. Representation skew: moved?

**Yes, substantially — but compositionally, not by a changed verdict on
shared content.**

| Tier | v1 (own scope) | v2 (28-file scope) | Reference, same 28 files |
|---|---:|---:|---:|
| `helper` + `shared-helper` | 73.1% | **30.4%** | 11.2% |
| `stage` + `stage-context` | 13.9% | **33.3%** | 44.5% |
| `shared-context` | 0 of 22 | 2 of 43 | — |
| `obsolete-mechanism` | 0 of 13 | 0 of 14 | — |
| `workflow` | — | 3 of 26 | — |

The over-helper ratio fell from **9.3×** to **2.7×**; the stage-tier deficit
narrowed from **0.31×** to **0.75×**.

**The ladder change was honestly applied.** `_config/icm-ladder.md` §6.3
(added citing run 2's C2 §4.2 by name) requires the checkpoint question to be
*answered* before a lower-rung classification is legal. Two independent
checks confirm compliance: `review-summary.md` Axis 2 sampled 12
`helper`/`shared-helper` rationales and found every one states its §6.3
answer before any §6.5 language (v1's defect was 73% of its corpus skipping
straight to §6.5); and the file-shape-mirroring tell the rule was designed to
catch is absent — `dispatch-worker`'s helper clusters group by contract
across files ("model/variant resolution & validation" spanning `AGENTS.md`,
`README.md`, `docs/schema.md`) rather than one-cluster-per-script, unlike
v1's `dispatch-mode` clusters (`BU-0079`–`0084` all from `bin/sgt-respond`;
`BU-0064`–`0070` all from `bin/sgt-drain`).

**And yet the shared-file verdict did not change.** On `bin/sgt-dispatch`,
the one file both runs read that drove most of v1's skew, v2 classifies
**8 of 10 `helper`** (1 `stage-context`, 1 `stage`) where v1 reached 9 of 10
`shared-helper` and the reference puts 5 of 6 at checkpoint tier or higher.
`bin/sgt-graphify` is starker: **9 `helper` / 1 `stage`** against 4/4
checkpoint-tier in the reference — including `BU-0245`, the atomic-publish
guarantee, filed `helper` where the reference's `BU-P6-088` is a `stage`.
Both carry real §6.3 rationale text. The question is being asked; on
single-purpose `bin/` command scripts the *answer* is still wrong.

**So: the aggregate improvement is real but traces mostly to composition.**
The newly-read file types (prose docs, `skills/*.md`, small read-only `bin/`
commands) carry more inherently judgment-bearing content. That is a genuine
gain — v2 reached content v1 never would have — but it is not evidence that
v2 would re-classify v1's fleet-machinery scripts correctly. `shared-context`
(2 vs 43) and `obsolete-mechanism` (0 vs 14) remain effectively unused, the
latter justified by count alone and flagged as insufficiently reasoned by
`AF-0005`.

---

## 6. Adversarial review: real findings?

**Yes. From a null capability to a functioning one, with zero false
positives.**

| | v1 | v2 |
|---|---|---|
| Findings raised | 0 | **5** (`AF-0001`–`AF-0005`; 3 `high`, 2 `medium`) |
| False positives | n/a | **0** — all five held on independent re-verification |
| Dispositions | n/a | 3 `park`, 1 `accept`, 1 `accept (partial)`, 0 rejected |
| Latent defects the stage failed to catch | **3 of 3** (`[S12]`×2, `[S3]`) | 4 in-scope C1/C2-class defects (below) |

v1's review stage raised nothing against a 3-package corpus later shown to
contain exactly 3 latent structural defects. v2's found the run's actual
dominant defect (corpus incompleteness — `AF-0001`/`AF-0002`/`AF-0003`, all
`high`) with severity matching real impact, plus a genuine circular-reasoning
defect (`AF-0004`: the `recover-worker` cluster's five records each defer to
a sibling's checkpoint claim, and the chain never terminates in a
`representation: stage` record — re-confirmed independently) and an
insufficiently-justified empty tier (`AF-0005`).

`90-reconcile` treated them honestly rather than rubber-stamping, declining
to patch NDJSON ledgers with non-conforming lines and declining `AF-0004`'s
more invasive fix on the principled ground that promoting the cluster would
require re-running `50-synthesize`/`60-draft`. Those are boundary calls, not
under-effort.

**What it structurally cannot reach.** Its axes ask whether a *promoted*
record deserved its rung (over-staging, invention) and whether the corpus is
arithmetically self-consistent (Axis 4 — counts, ids, ranges). Nothing
samples the 106-record `agents-invariant` tier — the corpus's largest — for
*under*-promotion, and nothing reads a swept file's prose back against its
sweep row. Both surviving sweep defects (§3.1) and the `cross-repo-planning`
tier collapse (§8) are exactly those two shapes. `AF-0004` was caught only
because its own rationale text was internally circular and thus visible to a
citation-graph check.

---

## 7. Checkpoint / retry mechanics: proven?

**Half proven. The state is real and correctly formed; the resume was never
exercised.**

**Proven, by direct re-verification:**

- `BU-0001`–`BU-0312` are contiguous with no gaps or duplicates.
- The ledger's six `done` rows partition that exact range with no gap or
  overlap and sum to 312.
- The halt is at a partition boundary: P6 fully `done` with a complete range,
  P7 fully `pending` with blank range and notes. No third status, no partial
  range, no "started, not finished" annotation anywhere in 21 rows — the
  protocol's central rule ("there is no `half-done`") held.
- `consequence-class-sweep.md`'s 28 rows are keyed one-per-`done`-file in
  ledger order, with no blank cell across 140 cells.
- Coverage was not silently truncated to make the ledger look complete — the
  protocol's named anti-pattern.

**Not proven:** `run-manifest.md`'s journal shows exactly one
`execution.started`/`execution.stopped` pair per stage — **10/10, no retry
anywhere in the run**. The ledger's fitness for an actual resume ("read it as
authoritative; do not re-read or re-extract a `done` partition's files") was
never tested end-to-end. Its *shape* is exactly what a resuming attempt
needs; that it will be honored is inference, not measurement.

**One gap the protocol does not close, correctly parked rather than hidden:**
nothing gates `30-normalize` onward on ledger completeness.
`00-contract/output/contract.md` §5's success criteria were not met by
`20-harvest` (15 of 21 rows `pending`), yet every downstream stage ran as
ordinary work. That is `AF-0001`, `park`ed — a cross-stage gate the
checkpoint protocol was never designed to provide, not a bug in how the
checkpoint executed.

---

## 8. Remaining gaps for v3

Ordered by leverage. **§6.7 discipline applied throughout: nothing below is
filed as an engine-gap claim, because for every item a lower rung is
available and unattempted.**

### 8.1 Finish the corpus — a caller-side orchestration loop, not engine pressure

54 of 82 files remain unread (P7–P21). Run 2 filed this phenomenon as GP-1
(*harvest turn-budget / volume wall*) and the adjudication **rejected the
engine-gap claim** while accepting the phenomenon: *"Lower rung
(`needs_input`/retry loop, per-partition stages) never attempted; this run's
own journal shows it working. Re-file only with evidence from a run that
tried it."*

**GP-1's remaining lower rung is an orchestration retry loop in the caller,
and run 3's own harvest demonstrates every piece it needs:**

- `20-harvest` already stops honestly at a partition boundary and writes a
  ledger whose `done`/`pending` rows and exact unit-id ranges are precisely
  what a resuming attempt reads (§7).
- The engine already holds a stage for external input and resumes it: the
  journal records `stage.needs_input` → `work.needs_input` →
  `stage.input_received` → `work.resumed` → `stage.completed`, **10 times, once
  per stage**, with each stage's wall-clock duration being real work done
  outside the daemon between the hold and the `sgt respond` that released it.
- Therefore re-entering `20-harvest` until `partition-ledger.md` shows 21
  `done` rows requires **no engine change whatsoever** — it is a loop in
  whatever drives `sgt respond`.

This is not engine pressure. It is a caller that has not yet been written.
Re-filing GP-1 as an engine gap remains barred until a run actually attempts
the loop and it fails.

The adjacent, genuinely-missing piece is the cross-stage gate (`AF-0001`):
`30-normalize` should refuse to proceed against a ledger with `pending` rows.
Lower rung available and unattempted — this is a stage `CONTEXT.md`
precondition check on an input artifact, exactly the fail-closed pattern the
workflow already uses for the `# AMBIGUOUS — NOT RESOLVED` marker. Not an
engine ask.

### 8.2 Make the consequence-class sweep cite *every* covering unit, not *some*

The two confirmed defects (§3.1) are both under-citation, and they are the
thin margin on which the §22.2 result rests. Two concrete lower-rung fixes,
both stage-context edits:

- Require the sweep to be filled **after** extraction is complete for the
  file and to name, for each class, every unit that covers it — a cell
  listing one id when three qualify is currently indistinguishable from a
  correct cell.
- Require `swept, none found` to carry the hunt question it was tested
  against, so a false negative on a six-line comment block
  (`schema/project.yaml.example` lines 90-93) becomes visible as a claim
  rather than a bare assertion.

### 8.3 Add an under-promotion axis to `80-adversarial-review`

The review has no method for the corpus's largest tier. A sampling axis over
`agents-invariant` records asking "does this record's rationale merely
pattern-match an existing invariant, or does it establish that the behavior
is procedure-independent?" would have caught `cross-repo-planning` —
where `BU-0268`–`BU-0272` were filed `agents-invariant` on rationales reading
"the same shape as `BU-0018`" and "the same 'ask only when genuinely
necessary' shape as `BU-0029`," leaving the package's `provenance.md` citing
one record for the whole workflow, against 16 workflow-tied reference units
for the same file. Stage-context change; not an engine ask.

### 8.4 Re-test the `bin/`-script under-promotion with the ladder change in force

v2's distribution gain does not extend to the content type v1's skew was
built from (§5). The cleanest experiment is to run the pending fleet
partitions (P7–P10) — v1's exact file set — and compare tier-by-tier. Until
then, "the ladder fix worked" is supported for prose and unproven for
command scripts.

### 8.5 Resolve `[S3]`: validator vs ladder, not generator vs validator

7 of 18 packages fail `[S3]` (empty `workflow.stages`), all of them honest
bucket-3 designs that self-disclose in their own `CONTEXT.md` and are triaged
`substantive` by `70-lint`. `[S12]` — the defect finalize-aware drafting was
built for — is fully closed (0/18, from 2/3 in v1). `[S3]` is a disagreement
between `validate-structure.py`'s baked-in assumption that a workflow package
has ≥1 stage and the ICM ladder's rule that a boundary can be correctly drawn
with zero materialized stages. One of the two is wrong and the adjudication
has not been made. Until it is, the delivered population fails the §9.9
draft-validity question 7 times.

### 8.6 Exercise an actual resume

§7's untested edge. The cheapest proof is a deliberate two-attempt harvest:
stop after P1–P6, re-enter, and confirm the second attempt reads the ledger,
skips the `done` partitions, and continues unit ids from `BU-0313`. This is
also the acceptance test for 8.1.

---

## 9. Delta verdict

**v2 is a genuine generational improvement over v1, and the evidence is
causal rather than circumstantial: each v1→v2 change was aimed at a named
run-2 finding and each hit its target.**

| Change | Aimed at | Landed? |
|---|---|---|
| Consequence-class sweep | run 2's 11 silent §22.2 misses | **Yes.** §22.2 criterion MET; all 3 in-scope misses captured; 0 new silent misses |
| §6.3-before-§6.5 ladder ordering | run 2's 73% helper skew | **Partly.** Skew 73.1%→30.4%, rule honestly applied — but the shared-file verdict is unchanged |
| Finalize-aware drafting | `[S12]`×2 | **Yes.** 0/18 |
| Adversarial-review axes | 0 findings against 3 real defects | **Yes.** 5 findings, 0 false positives, correct severities |
| Partition-checkpoint protocol | v1's destroyed coverage record | **Yes structurally, untested for resume** |

**The safety criterion is met.** No reference behavior with a confirmed
safety, identity, recovery, delivery or human-decision consequence is
silently absent from run 3's covered scope — a clean inversion of run 2's
answer, and the single most important thing v2 changed.

**The dominant remaining defect is coverage, and it is now a caller problem,
not a generator problem or an engine problem.** v2 built the mechanism to
stop honestly and resume; nobody has yet written the loop that resumes it.
That loop is GP-1's unattempted lower rung, and run 3's own journal —
ten hold/respond/advance cycles, one per stage — is the demonstration that
the engine already supports it.

**The most fragile result is the §22.2 pass itself.** It holds partly because
two consequence-class behaviors happened to be extracted from a second file
after their primary file's sweep cell was filled in wrong. §8.2 is the
cheapest work on this list and the one that keeps the margin from
evaporating.
