# Comparison C1 — Source Coverage, Behavioral Recall, Behavioral Precision (run 3 vs frozen reference, and vs run 2)

Run 3 (`docs/gauntlet/runs/n2-run3/`, generator v2, `.sergeant/workflows/repo-to-icm/`)
vs the frozen reference corpus (`reference-corpus/`, `FROZEN.md` v1: 979
units, 34 draft packages) vs run 2 (`docs/gauntlet/runs/n2-run2/`, generator
v1, `comparison-scorecard.md`). Dimension definitions:
`reference/proposal-next-iteration-icm-workflows.md` §9.9. Both runs are
pinned to the identical subject SHA, `f430cfd4f90174a98adbd7abebbece6303817929`
(`reference/UPSTREAM.md`), so unit-for-unit comparison across runs is valid.

**Ground rule enforced throughout:** every number below is reported *within
run 3's covered scope only* — the 28 files `partition-ledger.md` marks
`done` (partitions P1–P6). The other 54 `decompose` files (P7–P21, `pending`)
are excluded from every recall/precision figure; run 3's own artifacts
(`run-manifest.md`, `80-adversarial-review` findings AF-0001–AF-0003) already
disclose that gap honestly and it is not re-litigated here. Raw coverage is
reported once, separately, and never blended into quality numbers.

---

## 0. Covered scope, restated from `partition-ledger.md`

28 files, 312 generated units (`20-harvest/output/behavior-units.ndjson`,
carried unchanged through `30-normalize/output/behavior-units.normalized.ndjson`):

| Partition | Files | Gen. units |
|---|---|---:|
| P1 Root agent policy | `AGENTS.md` | 60 |
| P2 Product overview & docs index | `README.md`, `docs/README.md`, `docs/what-is-sergeant.md`, `docs/skills.md`, `docs/repo-scoped-skills.md`, `skills/sergeant-help/SKILL.md` | 45+3+7+5+2+6 = 68 |
| P3 Install/usage/troubleshoot/schema | `docs/getting-started.md`, `docs/using-sergeant.md`, `docs/troubleshooting.md`, `docs/schema.md`, `schema/project.yaml.example`, `mise.toml` | 6+37+21+8+3+10 = 85 |
| P4 Durable callback protocol | `docs/callbacks.md` | 21 |
| P5 Project resolution/status/sync/graphify | `bin/sgt-list`, `bin/sgt-context`, `bin/sgt-status`, `bin/sgt-sync`, `bin/sgt-td-list`, `bin/sgt-graphify`, `skills/load-project/SKILL.md` | 2+2+2+2+2+10+12 = 32 |
| P6 Cross-repo planning & dispatch | `skills/cross-repo-work/SKILL.md`, `skills/dispatch/SKILL.md`, `bin/sgt-dispatch`, `bin/sgt-td-create`, `bin/sgt-treehouse-init`, `bin/_sgt-review-axes.sh`, `templates/worker-brief.md` | 6+14+10+2+1+3+10 = 46 |
| **Total** | **28 files** | **312** |

Independently re-derived reference slice (filter `reference-corpus/behavior-units/P*.ndjson` on `source.path` ∈ these 28 paths under `reference/sergeant-upstream/`): **385 reference units**. Per-file counts below.

One methodology note, not a defect: `bin/_sgt-review-axes.sh` is disposed
`decompose` in run 3's own `10-inventory/output/inventory.md` (part of P6)
but `helper-evidence` in the reference's `source-inventory.md` (line 137) —
so the reference contributes **0** units for it. Run 3's 3 units from that
file (`BU-0300`–`BU-0302`, review-axis single-source-of-truth, missing-guidance
fail-closed, `error`-vs-`high` severity mapping) are real, source-supported
behaviors regardless — a legitimate alternate disposition, not an invention.

---

## 1. Source coverage

*What proportion of behavior-bearing source regions produced at least one traceable behavior unit?*

| | Value |
|---|---|
| Raw | **28 / 82** `decompose` files (34.1%), per `partition-ledger.md`/`run-manifest.md`. 54 files (P7–P21) never opened, disclosed by the run itself (AF-0001/AF-0002, `80-adversarial-review/output/review-summary.md`). |
| vs. run 2 | Run 2 reached 16/136 files (11.8%) under the old inventory's finer file split. Run 2's 16 files are a subset of run 3's 28: all 16 (`AGENTS.md`, `README.md`, 14 `bin/sgt-*` fleet-lifecycle files) are **outside** run 3's P1–P6 scope except `AGENTS.md` and `README.md` — run 3 has not yet re-reached run 2's `bin/` fleet-lifecycle set (`sgt-drain-force`, `sgt-cleanup`, `sgt-watch`, `sgt-interactive-worker`, `sgt-recover`, `sgt-respond`, `sgt-wake`, `sgt-drain`, `sgt-undrain`, `sgt-notify`, `sgt-ack-response`, `sgt-dag-run`, `sgt-dag-dispatch-hook`) — those live in P7/P8/P9/P10, still `pending`. Run 3 instead pushed *breadth* into new territory (docs, schema, skills, cross-repo/dispatch) rather than depth into run 2's partially-covered fleet set. |
| Within covered scope | **28 / 28** files yield ≥1 unit — no zero-unit file. |
| Within-file evenness | Uneven, and the unevenness recurs from run 2 in one exact spot: `AGENTS.md`'s "Procedural skills" routing table (lines 107–121, six trigger→skill rows) still produced **nothing** as its own units — only `BU-0022` touches the adjacent lines 120–124 (the "read the file directly" mechanic), not the six-row mapping itself. This is the *same* gap C1 flagged in run 2 (there, `BU-P1-132`–`BU-P1-137`), unresolved across generator versions. `schema/project.yaml.example` is the sharpest new instance: run 3 extracted only its `dag:` block (`BU-0201`–`BU-0203`) and never touched the file's `repos:`, `groups:`, or `graphify:` sections at all (§3 below). |

---

## 2. Behavioral recall — within covered scope

**Method:** every generated unit was matched to a reference unit sharing the
same source file by (a) hand verification for all 15 files with ≤23
reference units each (feasible to check exhaustively), and (b) lexical
(stop-word-filtered Jaccard, threshold 0.30) matching for the four largest
files (`AGENTS.md`, `README.md`, and — used as a labeled floor only —
`docs/callbacks.md`, `docs/getting-started.md`, `docs/skills.md`,
`docs/troubleshooting.md`, `docs/using-sergeant.md`,
`skills/cross-repo-work/SKILL.md`, `skills/dispatch/SKILL.md`,
`skills/load-project/SKILL.md`, `skills/sergeant-help/SKILL.md`,
`templates/worker-brief.md`). The lexical method **undercounts** on
paraphrase — confirmed directly: it scored `bin/sgt-dispatch` at 0/6 matched
even though hand verification (below, and §3) finds 4/6; `mise.toml` at 2/5
where hand verification finds 5/5; `bin/sgt-graphify` at 1/4 where hand
verification finds 4/4; `docs/schema.md` at 5/12 where hand verification
finds 10/12. **Where a file is marked "auto (floor)" below, treat its number
as a lower bound, not the true value** — the pattern above suggests true
recall on those ten files is meaningfully higher than shown.

| File | Ref units | Matched | Method | Recall |
|---|---:|---:|---|---:|
| `AGENTS.md` | 67 | 53 | auto, spot-checked (incl. `BU-0018`, `BU-0050`, see §3) | 79.1% |
| `README.md` | 36 | 28 | auto (floor) | 77.8% |
| `docs/README.md` | 6 | 5 | hand | 83.3% |
| `docs/what-is-sergeant.md` | 14 | 7 | hand | 50.0% |
| `docs/skills.md` | 16 | 4 | auto (floor) | 22.2%+ |
| `docs/repo-scoped-skills.md` | 4 | 2 | hand | 50.0% |
| `skills/sergeant-help/SKILL.md` | 17 | 4 | auto (floor) | 23.5%+ |
| `docs/getting-started.md` | 11 | 5 | auto (floor) | 45.5%+ |
| `docs/using-sergeant.md` | 44 | 25 | auto (floor) | 56.8%+ |
| `docs/troubleshooting.md` | 16 | 12 | auto (floor) | 75.0%+ |
| `docs/schema.md` | 12 | 10 | hand | 83.3% |
| `schema/project.yaml.example` | 4 | **0** | hand | **0.0%** |
| `mise.toml` | 5 | 5 | hand | 100.0% |
| `docs/callbacks.md` | 22 | 13 | auto (floor) | 59.1%+ |
| `bin/sgt-list` | 2 | 2 | hand | 100.0% |
| `bin/sgt-context` | 3 | 2 | hand | 66.7% |
| `bin/sgt-status` | 1 | 1 | hand | 100.0% |
| `bin/sgt-sync` | 2 | 2 | hand | 100.0% |
| `bin/sgt-td-list` | 1 | 1 | hand | 100.0% |
| `bin/sgt-graphify` | 4 | 4 | hand | 100.0% |
| `skills/load-project/SKILL.md` | 23 | 9 | auto (floor) | 39.1%+ |
| `skills/cross-repo-work/SKILL.md` | 16 | 6 | auto (floor) | 37.5%+ |
| `skills/dispatch/SKILL.md` | 40 | 10 | auto (floor) | 25.0%+ |
| `bin/sgt-dispatch` | 6 | 4 | hand (see §3) | 66.7% |
| `bin/sgt-td-create` | 1 | 1 | hand | 100.0% |
| `bin/sgt-treehouse-init` | 2 | 1 | hand | 50.0% |
| `bin/_sgt-review-axes.sh` | 0 | — | n/a (reference disposes helper-evidence) | n/a |
| `templates/worker-brief.md` | 10 | 3 | auto (floor) | 30.0%+ |
| **Total** | **385** | **219** | mixed, floor | **≥ 56.9%** |

**Headline: recall within covered scope is at least 56.9% (219/385), a
floor — the true figure is higher because ten of the largest files are
scored by an undercounting method.** Where fully hand-verified, recall
clusters much higher (all seven single/two-unit `bin/*` files at 100% or
50-66% off a denominator of 1-3; `mise.toml` 100%; `docs/schema.md` 83.3%),
with two sharp, confirmed exceptions: `schema/project.yaml.example` (0%, see
§3) and `docs/what-is-sergeant.md` (50%, its six "core concepts" definitions
— Project/Repository/Task/Fleet, `BU-P1-101`–`BU-P1-104` — are entirely
absent; only Worker and Decision-request survived, as `BU-0111`/`BU-0112`).

This is a clear, real improvement over run 2's 47.3% in-scope recall
(`n2-run2/comparison-scorecard.md` §2) even taken at the floor value, and
almost certainly larger once the ten floor-scored files are hand-verified.

### `schema/project.yaml.example` — 0/4, worth stating plainly

Run 3 generated 3 units from this file (`BU-0201`–`BU-0203`), and all three
are exclusively about the `dag:` block (name uniqueness, per-stage brief
source, `after:` ordering). The reference's 4 units for this file —
project-identity-is-filename (`BU-P7-001`), gh-auth identity precedence
(`BU-P7-002`), atomic graph republish (`BU-P7-003`), and per-repo
instruction override (`BU-P7-004`) — are all sourced from the file's
`repos:`/`graphify:` comment blocks, none of which run 3 touched. (The
project-identity fact is separately captured from `docs/schema.md` as
`BU-0195`, so it is not corpus-wide-absent — but it is absent from *this*
file's own extraction, and `BU-P7-003`/`BU-P7-004` are absent everywhere.)

---

## 3. THE question — the 11 consequence-class behaviors run 2 silently missed

Per `n2-run2/comparison-scorecard.md` §3, cross-referenced against source
file to determine whether each now lives in run 3's covered scope (P1–P6).

| # | Ref id(s) | Source file | In run 3 scope? | Status |
|---|---|---|---|---|
| 1 | `BU-P6-040`, `BU-P6-138` | `bin/sgt-drain-force`, `bin/sgt-cleanup` | No — P9/P10, `pending` | Still missing (out of scope, not silent-in-scope) |
| 2 | `BU-P6-139` | `bin/sgt-cleanup` | No — P10, `pending` | Still missing (out of scope) |
| 3 | `BU-P6-140` | `bin/sgt-cleanup` | No — P10, `pending` | Still missing (out of scope) |
| 4 | `BU-P6-103` | `bin/sgt-watch` | No — P10, `pending` | Still missing (out of scope) |
| 5 | `BU-P6-115` | `bin/sgt-interactive-worker` | No — P7, `pending` | Still missing (out of scope) |
| 6 | `BU-P6-108` | `bin/sgt-interactive-worker` | No — P7, `pending` | Still missing (out of scope) |
| 7 | `BU-P6-073`, `BU-P6-079`, `BU-P6-113`, `BU-P6-114` | `bin/sgt-recover`, `bin/sgt-respond`, `bin/sgt-interactive-worker` | No — P7/P8, `pending` | Still missing (out of scope) |
| 8 | `BU-P6-098` | `bin/sgt-wake` | No — P8, `pending` | Still missing (out of scope) |
| 9 | `BU-P1-050` | `AGENTS.md` | **Yes — P1, done** | **Captured: `BU-0050`** |
| 10 | `BU-P1-016` | `AGENTS.md` | **Yes — P1, done** | **Captured: `BU-0018`** |
| 11 | `BU-P6-127` | `bin/sgt-dispatch` | **Yes — P6, done** | **Captured: `BU-0288` + `BU-0296`** |

**8 of 11 remain unresolved, but every single one of them is now
scope-excluded, not silently absent** — their source files sit in P7–P10,
still `pending`, and run 3's own artifacts disclose that gap
(`partition-ledger.md`, `run-manifest.md` AF-0001/AF-0002). Per the ground
rule (§9.9, and the adjudicated distinction in run 2's own D-4/§3), a miss
whose source file was never read is a coverage gap, not a silent-recall
failure — it cannot be scored against §22.2 here.

**All 3 of the items that fell inside run 3's newly-covered scope are now
captured, essentially verbatim:**

- **#9** — reference `BU-P1-050`: *"Standing authorization may remove
  repetitive dispatch confirmation, but never authorizes risk acceptance,
  gate skipping, force operations, secret exposure, or destruction of
  preserved state."* Run 3's `BU-0050` (source `AGENTS.md` L173-175,
  quote hash `sha256:7e97c0c9d7addaff19d5c1c443b0e621cb6ed4be55b67d8b41cb64ca360b3d7d`,
  verified below) reproduces this near word-for-word, and is correctly
  double-filed in `consequence-class-sweep.md`'s `AGENTS.md` row under both
  Safety and Human-decision.
- **#10** — reference `BU-P1-016`: *"Never use direct mode to edit several
  repositories in one checkout, or to bypass repository instructions, task
  ownership, review independence, or shipping gates."* Run 3's `BU-0018`
  (source `AGENTS.md` L38-41, quote hash
  `sha256:3905de42bbbfa463807140e215c8e4b0aeabeb5222a4906c6658048e653bb7c2`)
  reproduces this verbatim (the run's `statement` field is a literal copy of
  the reference's). **However: `consequence-class-sweep.md`'s `AGENTS.md`
  row does not cite `BU-0018` in either its Safety or Human-decision
  column** (Safety lists `BU-0050, BU-0054, BU-0055, BU-0057, BU-0059`;
  Human-decision lists `BU-0029, BU-0050`) — the behavior is captured, but
  the sweep artifact itself under-cites it against the exact class it was
  written to hunt for. Not a "none found" failure (the columns aren't
  empty), but a real completeness gap in the sweep's own cross-referencing —
  see §4.
- **#11** — reference `BU-P6-127`: *"Rolling back an interrupted dispatch is
  scoped precisely: a managed coordinator pane this exact invocation created
  is killed on failure, but a coordinator pane the invocation merely
  selected … is never touched, and once every repo has been dispatched
  successfully the rollback trap is disarmed…"* Run 3 splits this
  correctly into its two independently-true halves: `BU-0288` (source
  `bin/sgt-dispatch` L324-335, the scoped-kill/never-touch-adopted-pane half,
  installed as an EXIT trap) and `BU-0296` (the trap-disarm-on-full-success
  half). Both are cited in `consequence-class-sweep.md`'s `bin/sgt-dispatch`
  row (`BU-0288` under Safety, `BU-0296` under Identity) — correctly swept,
  unlike #10.

**One new in-scope near-twin discovered while verifying #11:** reference
`BU-P6-124` (*"Which pinned model tuple a dispatched worker will run is
resolved with a fixed, explicit-only precedence … no project-level or
per-repo default in the precedence chain"*, `bin/sgt-dispatch`) is the
**dispatch-time** counterpart to miss #6 above (`BU-P6-108`, the
**resume-time** re-honoring of the same pinned tuple). The
`consequence-class-checklist.md`'s own Identity hunt question explicitly
warns against this exact confusion ("do not let finding the creation-time
check satisfy the hunt for the resume-time one"). Run 3's `bin/sgt-dispatch`
units (`BU-0287`–`BU-0296`) cover worktree/pane/task validation in detail
but contain **no unit about model-tuple resolution precedence at all** — the
dispatch-time half is missing too, not just the resume-time half inherited
from run 2. `consequence-class-sweep.md`'s Identity column for
`bin/sgt-dispatch` (`BU-0292, BU-0295, BU-0296`) doesn't cover it either.

---

## 4. Verifying `consequence-class-sweep.md`'s "swept, none found" cells

The sweep table (`20-harvest/output/consequence-class-sweep.md`) has ~65
"swept, none found" cells across the 28 covered files. A false "none found"
— a cell claiming nothing was found in a file whose own text plainly states
a Safety/Identity/Recovery/Delivery/Human-decision behavior per
`consequence-class-checklist.md`'s five hunt questions — is a sweep failure.
Verified against the reference's per-file unit lists and, for the strongest
candidate, the file's own raw text.

### Confirmed sweep failure: `schema/project.yaml.example`, Delivery = "swept, none found"

The file's own `graphify:` block comment (lines 88-93 of
`reference/sergeant-upstream/schema/project.yaml.example`) reads:

> `# Required if graphify block is present. Published output directory for the`
> `# merged project graph; trailing / is fine, directory symlinks are preserved,`
> `# and sgt-graphify only replaces it after a complete run.`

This is exactly the pattern Hunt Question 4 asks for — *"Is there a
guarantee that some action happens exactly once … a readiness gate that
must be satisfied before delivering"* — and is precisely what the
reference's own `BU-P7-003` cites from this file (*"A published cross-repo
knowledge graph is replaced only after a graphify run completes in
full…"*). The sentence sits in the file that was swept, was not extracted as
a behavior unit at all (§2, the 0/4 finding), and the sweep row records
"swept, none found" for Delivery anyway. This traces directly to the
extraction gap in §2 — the actor never read past the `dag:` block-adjacent
material it did extract from — but the sweep step is specifically supposed
to be a second, deliberate pass "against the five hunt questions" over text
"already open," and it missed a sentence sitting in a six-line comment
block. **Confirmed false "none found."**

### Related, not a "none found" case: `AGENTS.md`'s Safety/Human-decision under-citation of `BU-0018` (§3, item #10)

As detailed above — the cell isn't empty, but it omits a unit that is a
textbook fit for both columns it's filed under nowhere.

### Spot-checked and found accurate ("none found" holds)

- `docs/README.md` (Identity/Recovery/Delivery/Human-decision): its 6
  reference units (`BU-P8-001`–`BU-P8-006`) are documentation-authority and
  credential-hygiene rules with no destructive action, identity-pinning,
  resumability, or escalation content. Accurate.
- `docs/skills.md`, `docs/repo-scoped-skills.md`, `skills/sergeant-help/SKILL.md`
  (Identity/Recovery/Delivery): skill-vetting and documentation-routing
  procedures; no candidate content in the reference units for these
  classes. Accurate.
- `docs/getting-started.md` (Safety/Human-decision): install-checklist
  content; no destructive-action gating or escalation language. Accurate.
  (Delivery is a softer call — `BU-P8-048`, "a project graph is only
  considered successfully published when both `graph.json` and
  `GRAPH_REPORT.md` exist," is borderline Delivery-class but is a much
  weaker fit than the `schema/project.yaml.example` case above, since it's
  about a completeness check rather than an atomicity/exactly-once
  guarantee; flagged as a soft candidate, not counted as confirmed.)
- `mise.toml` (Recovery/Human-decision), `docs/schema.md` (Recovery/Human-decision),
  `docs/callbacks.md` (Human-decision only unfilled cell), `bin/sgt-list`,
  `bin/sgt-status`, `bin/sgt-td-list`, `bin/sgt-treehouse-init`,
  `skills/dispatch/SKILL.md` (Identity only unfilled cell): spot-checked
  against their reference unit lists — no clear counter-example found.

Given the volume of cells (~65), this is a spot-check, not an exhaustive
audit — but the one file with the smallest reference-unit count and the
most obviously-skipped content (`schema/project.yaml.example`) produced a
confirmed failure, which is consistent with §2's independent finding that
this same file has 0% recall.

---

## 5. Behavioral precision

*How many generated units are supported by source evidence rather than invented?*

**Clean — 312/312, verified by direct recomputation, not by trusting the
run's self-report.**

| Check | Result |
|---|---:|
| `quote` is a literal contiguous substring of the cited file, read fresh from `reference/sergeant-upstream/` at the pinned SHA `f430cfd4f90174a98adbd7abebbece6303817929` | **312 / 312** |
| `sha256(quote)` (as UTF-8 bytes) equals the stored `quote_hash` | **312 / 312** |
| Units carrying a `span_bytes` field (the long-quote convention that caused run 2's D-2 dispute) | **0** — no long quotes in this run, so that failure mode cannot recur here |

This was run programmatically over the full 312-record
`behavior-units.normalized.ndjson`, i.e. it is a complete check, not merely
the ≥15-sample spot-check requested. Twenty are named here for direct
inspection (locator column lets each be reopened by hand):

| Unit | File | Locator | Quote hash (verified) |
|---|---|---|---|
| `BU-0058` | `AGENTS.md` | L187 | matches |
| `BU-0013` | `AGENTS.md` | L22-36 | matches |
| `BU-0141` | `docs/using-sergeant.md` | L149-155 | matches |
| `BU-0126` | `skills/sergeant-help/SKILL.md` | L51-52 | matches |
| `BU-0115` | `docs/what-is-sergeant.md` | L79-82 | matches |
| `BU-0072` | `README.md` | L217-225 | matches |
| `BU-0053` | `AGENTS.md` | L181 | matches |
| `BU-0280` | `skills/dispatch/SKILL.md` | L168-173 | matches |
| `BU-0045` | `AGENTS.md` | L163-164 | matches |
| `BU-0303` | `templates/worker-brief.md` | L39 | matches |
| `BU-0217` | `docs/callbacks.md` | L41-49 | matches |
| `BU-0017` | `AGENTS.md` | L38-41 | matches |
| `BU-0048` | `AGENTS.md` | L169-170 | matches |
| `BU-0112` | `docs/what-is-sergeant.md` | L54-58 | matches |
| `BU-0120` | `docs/skills.md` | L142-144 | matches |
| `BU-0259` | `skills/load-project/SKILL.md` | L39 | matches |
| `BU-0309` | `templates/worker-brief.md` | L102 | matches |
| `BU-0014` | `AGENTS.md` | L22-36 | matches |
| `BU-0288` | `bin/sgt-dispatch` | L324-335 | matches |
| `BU-0018` | `AGENTS.md` | L38-41 | matches (cited in full, §3) |

**Extra/unmatched generated units** (present in run 3, no reference unit
matched them in the same file): under the hand-verified files these are
consistently finer-grained real behaviors the reference collapsed at
coarser granularity — e.g. `bin/sgt-graphify`'s `BU-0246`/`BU-0247`
(repo-name validation, output-path-not-equal-source-path) are real,
source-supported guard clauses not separately itemized in the reference's 4
units for that file. No invented behavior was found in any hand-verified
file. **Precision verdict: clean, consistent with run 2's clean precision
finding — the improvement across generator versions is entirely on the
recall/coverage side, not by relaxing evidence discipline.**

---

## 6. Summary

| Dimension | Run 2 (in-scope) | Run 3 (in-scope) |
|---|---:|---:|
| Raw coverage | 16/136 files (11.8%) | 28/82 files (34.1%) |
| Covered-scope reference units | 165 | 385 |
| Behavioral recall | 47.3% (78/165) | **≥56.9% (219/385), floor** |
| Behavioral precision | 108/108 clean | **312/312 clean** |
| §22.2 items resolved (of run 2's original 11) | 0/11 (all silent) | **3/3 of the items that entered scope; 8/11 remain — now correctly out-of-scope, not silent** |

Run 3 is a real improvement on every axis measured here: coverage nearly
tripled, recall improved even at a conservative floor, precision stayed
perfect, and all three of run 2's silent §22.2 misses that fell inside the
newly-covered files are now captured near-verbatim. The two clearest
remaining defects are narrow and specific rather than systemic: (1)
`schema/project.yaml.example` was almost entirely unread (0/4 recall, one
confirmed false "none found" sweep cell for its Delivery-class atomic-publish
sentence), and (2) the dispatch-time model-tuple-precedence behavior
(`BU-P6-124`) joins its already-known resume-time twin (`BU-P6-108`, still
out of scope) as a second, in-scope, still-uncaptured half of the same
Identity-class fact family the consequence-class checklist was written
specifically to catch.
