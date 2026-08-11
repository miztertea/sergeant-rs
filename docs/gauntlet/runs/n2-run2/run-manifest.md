# N2 run2 — evidence manifest

Collected 2026-08-10. This directory is a copy of the run's deliverables,
pulled out of the disposable worktree/journal so they survive after the
data dir is cleaned up. The worktree and journal themselves remain at
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run2/`
as the primary evidence; this manifest and the copied files are a
convenience layer on top.

## Identity

| Field | Value |
|---|---|
| Subject | `reference/sergeant-upstream` (upstream `main`, fork `miztertea/sergeant`) |
| Subject SHA (base = head, no upstream commits during run) | `85568a0f52500537826c7010756dc5bfa558d576` |
| Work id | `01KZNW46C3Y2W890DE7S8M94NZ` |
| Work branch | `sergeant/01KZNW46C3Y2W890DE7S8M94NZ` |
| Workflow | `repo-to-icm` v1, 10 stages |
| Backend | `fake` (engine bookkeeping only — see "How to read the timings" below; substantive stage work was actor-driven, per N2's "actor-only" design) |
| Final work state | **`completed`** (`sgt work show --json`: `work.state = "completed"`, `stage.status = "completed"` at index 9/10 = `90-reconcile`) |
| Teardown disposition | `retained_dirty` at capture time, but the worktree's `git status` is clean — all listed changes were already committed to the work branch (commit `b1546e9`, "repo-to-icm finalize: apply output/ dispositions (D9)", on top of the base commit `85568a0f`) |

## Stage timing table

Derived from `stage.entered` / `stage.needs_input` / `stage.input_received` /
`stage.completed` / `stage.blocked` events in the journal, filtered to this
work id (76 of the journal's 82 total events carry `work_id =
01KZNW46C3Y2W890DE7S8M94NZ`).

**How to read the timings:** this is an *actor-only* workflow (N2's stated
scope — task #13). The engine's fake backend does not execute stage work
itself; it holds each stage open with a `stage.needs_input` prompt until an
external actor (a Claude session working directly in the worktree) finishes
that stage's artifacts and calls `sgt respond`, which the engine records as
`stage.input_received` immediately followed by `stage.completed`. So for
stages `00-contract`, `10-inventory`, and `20-harvest`'s first attempt, the
`entered → completed` gap **is** real actor wall-clock work time. From
`20-harvest`'s retry (attempt 2) onward, stages `30-normalize` through
`90-reconcile` have no `needs_input` hold configured and complete in
single-digit milliseconds — because by the time the engine record was
unblocked, the actor had already produced every remaining stage's output
files directly in the worktree during the same session; the fast completions
are the engine catching its bookkeeping up to work already done, not stages
executing instantaneously.

| Stage | Attempt | Entered (UTC) | Ended (UTC) | Duration | Note |
|---|---:|---|---|---:|---|
| `00-contract` | 1 | 12:59:54.217 | 13:03:07.900 | 193.7s (~3m 14s) | actor work time (entered→input_received→completed) |
| `10-inventory` | 1 | 13:03:07.903 | 13:11:25.439 | 497.5s (~8m 18s) | actor work time |
| `20-harvest` | 1 | 13:11:25.440 | 14:54:11.025 | 6165.6s (~1h 42m 46s) | actor work time, ended in **`stage.blocked`**, not completion — see below |
| `20-harvest` | 2 (retry) | 14:55:15.147 | 14:55:15.149 | 0.002s | engine catch-up; artifacts already on disk from attempt 1 |
| `30-normalize` | 1 | 14:55:15.151 | 14:55:15.151 | ~0s | engine catch-up |
| `40-classify` | 1 | 14:55:15.152 | 14:55:15.152 | ~0s | engine catch-up |
| `50-synthesize` | 1 | 14:55:15.153 | 14:55:15.154 | 0.001s | engine catch-up |
| `60-draft` | 1 | 14:55:15.154 | 14:55:15.154 | ~0s | engine catch-up |
| `70-lint` | 1 | 14:55:15.156 | 14:55:15.156 | ~0s | engine catch-up |
| `80-adversarial-review` | 1 | 14:55:15.157 | 14:55:15.158 | 0.001s | engine catch-up |
| `90-reconcile` | 1 | 14:55:15.159 | 14:55:15.159 | ~0s | engine catch-up |

Wall clock, `work.submitted` (12:59:54.115) → `work.completed`
(14:55:15.160): **6921.0s (~1h 55m 21s)**.

### The `20-harvest` blocked/retry episode

`20-harvest` attempt 1's actor work (writing `emit.py`, `emit_batch.py`,
`quote.sh`, `behavior-units.ndjson`, `coverage-note.md`) finished and
`stage.input_received` fired at 14:54:11.024, but delivering that input hit
`stage.blocked` / `work.blocked` one millisecond later: *"backend `fake`
does not recognise execution `01KZNWS9G19T83S9WXEENP9856`"* — the daemon
(and presumably its in-memory fake-backend execution table) had been
restarted between when attempt 1 started (13:11:25) and when the actor
finally responded (14:54:11), so the execution id no longer resolved.
`work.resumed` (`reason: retry`) at 14:55:15.143 opened a fresh attempt 2,
which completed instantly since the actual stage artifacts were already
committed to the worktree from attempt 1. No content was lost or redone —
this is a clean recovery from a daemon-restart / execution-id mismatch, not
a workflow defect.

### Post-completion noise

After `work.completed` (14:55:15.160) and `surface.torn_down`
(14:55:15.193), six further `work.respond` commands arrived roughly every
10–15 minutes (15:06, 15:18, 15:29, 15:38, 15:45, 15:54, 16:01 UTC) — all
correctly rejected with `not_awaiting_input` since the work was already
`completed`. Consistent with an external driver (e.g. the overnight
send_later continuation chain) polling a work item it didn't yet know had
finished; harmless, evidence of idempotent-rejection working as designed.

## Journal statistics

| Metric | Value |
|---|---:|
| Journal file | `data/journal/00000001.ndjson` (single segment) |
| Journal bytes | 92,374 |
| Total events (journal-wide) | 82 |
| Events for this work id | 76 |
| Non-work events | 6 (`daemon.started` ×2, `backend.probed` ×4 — one daemon restart mid-run, both `claude` and `fake` backends re-probed each time) |
| `sgt status --json` `journal_head` | 82 (matches event count) |

Event-kind breakdown for this work id:

| Kind | Count |
|---|---:|
| `execution.started` | 11 |
| `execution.stopped` | 10 |
| `stage.entered` | 11 |
| `stage.completed` | 10 |
| `command.rejected` | 7 |
| `command.accepted` | 5 |
| `work.resumed` | 4 |
| `stage.input_received` | 3 |
| `stage.needs_input` | 3 |
| `work.needs_input` | 3 |
| `stage.blocked` | 1 |
| `surface.materialized` | 1 |
| `surface.materializing` | 1 |
| `surface.torn_down` | 1 |
| `work.blocked` | 1 |
| `work.completed` | 1 |
| `work.started` | 1 |
| `work.submitted` | 1 |
| `workflow.bound` | 1 |
| **Total** | **76** |

## Artifact inventory (copied into this directory)

### `workflows/repo-to-icm/*/output/` — 23 files across all 10 stages

```
00-contract/output/{README.md, contract.md}
10-inventory/output/{README.md, inventory.md}
20-harvest/output/{README.md, behavior-units.ndjson}
  (coverage-note.md was produced but not promoted — see "Finalize" below)
30-normalize/output/{README.md, behavior-units.normalized.ndjson}
40-classify/output/{README.md, classifications.ndjson}
50-synthesize/output/{README.md, candidates.md}
60-draft/output/{README.md, draft-report.md}
70-lint/output/{README.md, lint-report.md}
80-adversarial-review/output/{README.md, findings.ndjson, review-summary.md}
90-reconcile/output/{README.md, adjudication-log.md, grammar-pressure.ndjson, measurement-package.md}
```

**Finalize:** `90-reconcile` ran
`scripts/finalize.py` against the `output/` dispositions declared in each
stage's own README. The one non-`keep` action was removing
`20-harvest/output/coverage-note.md` (not declared `promote` by
`20-harvest/output/README.md`, which names only `behavior-units.ndjson`).
That removal was actually applied and committed (`git log` on the work
branch shows `b1546e9 repo-to-icm finalize: apply output/ dispositions
(D9)`) — the worktree's copy of `20-harvest/output/` at capture time
already reflects this (README.md + behavior-units.ndjson only), which this
run-manifest's copy preserves as-is. `coverage-note.md`'s content is not
lost: it's quoted/summarized in `90-reconcile/output/grammar-pressure.ndjson`
and in `measurement-package.md`'s "Extraction coverage" section, and
remains recoverable from Work-branch history per the D9 convention.

### `drafts/workflows/*` — 24 files, 3 draft ICM workflow packages

Copied from the worktree's `.sergeant/drafts/workflows/` (60-draft's
output), each with `status: draft` in its `index.md`:

| Candidate | Member stages | Notes |
|---|---:|---|
| `dispatch-mode` | 1 (`10-dispatch-worker`) | |
| `standard-task-workflow` | 5 (`10-load-context`, `20-check-queue`, `30-reconcile-existing-state`, `40-validate`, `50-reconcile-and-deliver`) | |
| `ship-with-no-mistakes` | 0 (deliberate empty `stages` array) | per its own `provenance.md`/`CONTEXT.md` |

## Ambiguity roll-up

Grepped every stage output artifact under `workflows/repo-to-icm/*/output/`
for `AMBIGUOUS` (the workflow's fail-closed marker is the literal string
`# AMBIGUOUS — NOT RESOLVED`, per `_config/run-discipline.md`):

**No stage output ever opened with `# AMBIGUOUS — NOT RESOLVED`.** Every
match found is a stage's own explicit confirmation that the marker did
*not* apply and ordinary work proceeded — `00-contract/output/contract.md`,
`10-inventory/output/inventory.md`, `50-synthesize/output/candidates.md`,
`60-draft/output/draft-report.md`, `70-lint/output/lint-report.md`, and
`80-adversarial-review/output/review-summary.md` each contain one such
negative-confirmation sentence. This run never hit the workflow's
fail-closed escalation path.

Two substantive things *were* flagged, neither as an `AMBIGUOUS` escalation:

1. **Contract inaccuracy, folded back but not corrected.**
   `10-inventory/output/inventory.md` found `bin/__pycache__/
   sgt-callbackcpython-312.pyc` present under the subject subtree,
   contradicting `00-contract/output/contract.md` §3's claim that "no
   build/dependency-output directory is currently present." `10-inventory`
   excluded the file correctly by applying the contract's own exclusion
   *category* (not escalated as ambiguous) and asked `90-reconcile` to
   fold the discrepancy back into the contract record.
   `90-reconcile/output/adjudication-log.md` records the fact plainly but
   left `contract.md` unedited — it was never raised as an
   `80-adversarial-review` finding, so it falls outside that stage's
   accept/reject/park mechanism, and `90-reconcile`'s own contract doesn't
   name `contract.md` as an editable file this run.

2. **Zero findings from adversarial review — confirmed genuine, not a
   skip.** `80-adversarial-review/output/findings.ndjson` is empty (0
   records) across all three axes (boundary-honesty, invention,
   engine-gap-refutation) and all three severities.
   `90-reconcile/output/adjudication-log.md` cross-checked this
   independently (review-summary.md's own finding-counts table agrees) and
   disposed 0 findings (0 accept / 0 reject / 0 park) — there was nothing
   to adjudicate.

3. **Four open substantive lint defects, not adjudicated as findings.**
   `70-lint/output/lint-report.md` reports all 4 trees checked (3 draft
   candidates + this workflow's own tree) FAIL at both initial and final
   validator runs, one substantive defect each: `dispatch-mode` and
   `standard-task-workflow` both missing a named finalize step (`[S12]`);
   `ship-with-no-mistakes` has a deliberate empty `stages` array (`[S3]`,
   by its own design); this workflow's own tree has
   `20-harvest/quote.sh` present but unclassified as an executable
   (`[S10]`). None of these 4 defects was raised as an
   `80-adversarial-review` finding, so `90-reconcile` recorded but did not
   adjudicate them — they remain open for a human reviewer.

## Supporting run statistics (from `measurement-package.md`)

Reproduced here for convenience — full detail and methodology in
`workflows/repo-to-icm/90-reconcile/output/measurement-package.md`.

- **Source coverage:** 179 files enumerated, 1 excluded
  (build-dependency-output category), 178 inventoried — 136 `decompose`,
  27 `helper-evidence`, 0 `obsolete-candidate`, 15 `reference-only`.
- **Behavioral precision:** 29/29 sampled source citations verified
  cleanly (0 invention findings); 16/16 provenance citations confirmed to
  exist in `classifications.ndjson`.
- **Provenance completeness:** 3/3 materialized draft candidates have a
  non-empty `provenance.md` citing at least one real `behavior_id`, 0
  fabricated-citation findings against any of the 16 distinct
  `behavior_id`s cited.
- **Draft validity:** 4/4 trees FAIL the structural validator (see item 3
  in the ambiguity roll-up above), 0 mechanical defects, 4 substantive
  defects (1 per tree).
- **Review convergence:** 0 findings, 0 adjudications, 0 repairs applied.
- **Extraction coverage:** 108 behavior units recorded (`BU-0001`–`BU-0108`)
  spanning 18 of 136 `decompose` files/partition-members; 118 explicitly
  recorded as not reached this run (not silently dropped).
- **Normalization:** 108 in, 108 out (0 splits, 0 lost/gained); 41/108
  statements rewritten, 51/108 gained a `notes` field, 0 confidence shifts.
- **Representation mix (108 classified records):** `shared-helper` 79,
  `agents-invariant` 13, `stage-context` 9, `stage` 6, `workflow` 1,
  `shared-context` 0, `obsolete-mechanism` 0, `engine-gap` 0.
- **Candidate yield:** 3 workflow candidates, 6 stage candidates, 15
  shared-helper groupings (covering all 79 `shared-helper` records), 13
  `agents-invariant` records (listed, not grouped), 0 `engine-gap`
  candidates.
- Five of the ten §9.9 measurement dimensions (behavioral recall,
  workflow-boundary agreement, stage-boundary agreement, representation
  agreement, engine-gap quality) are explicitly **not** covered — they
  require blind comparison against `reference-corpus/`, which this run's
  actors never opened and which a separate, independent comparer must
  perform later.

## Daemon shutdown

Daemon (pid 2184, data dir
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run2/data`)
was stopped with `SIGTERM` (no `sgt daemon stop` subcommand exists — verified
via `sgt daemon --help`, which only runs the daemon in the foreground). Post-stop
`pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` was empty — see the
task's final output for the literal command/result pair.

The worktree, journal, and full data dir at
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run2/`
were left in place as primary evidence (not deleted).
