# N2 measurement run — n2-run3

- **Subject SHA:** `e7ef97cc465bc38ce713766fe2258dcf1fb53930` (base_sha == head_sha; no commits made in the surface — all deliverables are untracked worktree writes)
- **Work id:** `01KZQ32J2BAD4P8WJA9SWXRMZ9`
- **Workflow:** `repo-to-icm` v2, 10 stages (`00-contract` … `90-reconcile`)
- **Backend:** `fake` (deterministic in-process backend; each stage held for external input via `sgt respond`, consistent with the overnight-driver continuation pattern — see intent below)
- **Intent:** "Decompose the repository subtree `reference/sergeant-upstream` — pinned per `reference/UPSTREAM.md` at upstream SHA `f430cfd4f90174a98adbd7abebbece6303817929` — into draft ICM workflows per `.sergeant/workflows/repo-to-icm`. Scope: the subtree only; exclude `reference/UPSTREAM.md` itself, `.sergeant/`, and `AGENTS.md`."
- **Data dir:** `/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run3/data`
- **Final `work.state` (via `sgt work show --json`, queried live against the still-running daemon):** **`completed`**
  - Final stage: `90-reconcile`, index 9 of 10, `status: completed`, `detail: advanced-9`
  - `execution.stop_requested: true` on the final (10th) execution — consistent with the fake backend's hold/respond/advance/stop-per-stage pattern, not a mid-run abort
  - `teardown.clean: false` — worktree left `retained_dirty` by design (deliverables are all untracked adds; see teardown binding `changes` diff, 155 files added, all under `.sergeant/drafts/workflows/**` and `.sergeant/workflows/repo-to-icm/**/output/**`)

Note on the CLI surface: `sgt work show` has no separate "event list" verb — work-scoped events were extracted directly from the journal (filtering `work_id == 01KZQ32J2BAD4P8WJA9SWXRMZ9`), which is the only source of truth per this repo's architecture invariant ("the journal is the only truth").

## Stage timings (`stage.entered` → `stage.completed`, from the journal)

| # | Stage | Entered (UTC) | Completed (UTC) | Duration |
|---|---|---|---|---|
| 0 | 00-contract | 2026-08-11T00:20:35.084Z | 2026-08-11T00:23:22.368Z | 2m 47s (167.3s) |
| 1 | 10-inventory | 2026-08-11T00:23:22.369Z | 2026-08-11T00:34:00.594Z | 10m 38s (638.2s) |
| 2 | 20-harvest | 2026-08-11T00:34:00.596Z | 2026-08-11T01:11:25.034Z | 37m 24s (2244.4s) |
| 3 | 30-normalize | 2026-08-11T01:11:25.036Z | 2026-08-11T01:12:56.125Z | 1m 31s (91.1s) |
| 4 | 40-classify | 2026-08-11T01:12:56.125Z | 2026-08-11T01:32:34.878Z | 19m 39s (1178.8s) |
| 5 | 50-synthesize | 2026-08-11T01:32:34.880Z | 2026-08-11T01:43:46.246Z | 11m 11s (671.4s) |
| 6 | 60-draft | 2026-08-11T01:43:46.248Z | 2026-08-11T01:57:27.117Z | 13m 41s (820.9s) |
| 7 | 70-lint | 2026-08-11T01:57:27.119Z | 2026-08-11T02:03:15.307Z | 5m 48s (348.2s) |
| 8 | 80-adversarial-review | 2026-08-11T02:03:15.308Z | 2026-08-11T02:13:23.724Z | 10m 8s (608.4s) |
| 9 | 90-reconcile | 2026-08-11T02:13:23.727Z | 2026-08-11T02:23:21.656Z | 9m 58s (597.9s) |

- **Total stage-active time (sum of the 10 durations):** 7,366.6s ≈ 2h 2m 47s
- **Wall clock, `work.submitted` → last work event:** 2026-08-11T00:20:35.019Z → 2026-08-11T02:23:21.667Z = 7,366.6s (matches the sum above; each stage's `stage.completed` is immediately followed by the next `stage.entered`, no idle gaps between stages — all elapsed time is inside stages, none between them)
- Every stage entered `needs_input`/held (fake backend's hold-per-stage mechanism: `stage.needs_input` → `work.needs_input` → `stage.input_received` → `work.resumed` → `stage.completed`, 10/10 for each of the 10 kinds), so per-stage duration reflects real wall-clock work done outside the daemon between the hold and the `sgt respond` that released it — not raw backend compute time.

## Journal stats

- **Journal directory:** `data/journal/` — single segment, `00000001.ndjson`
- **Journal bytes:** 141,258
- **Total events in journal (all work + daemon-scoped):** 111
- **Events for this work (`work_id` filter):** 108
- **Event kind breakdown for this work:**

| kind | count |
|---|---|
| command.accepted | 11 |
| stage.entered | 10 |
| execution.reserved | 10 |
| execution.started | 10 |
| stage.needs_input | 10 |
| work.needs_input | 10 |
| stage.input_received | 10 |
| work.resumed | 10 |
| stage.completed | 10 |
| execution.stopped | 10 |
| work.submitted | 1 |
| surface.materializing | 1 |
| surface.materialized | 1 |
| workflow.bound | 1 |
| work.started | 1 |
| work.completed | 1 |
| surface.torn_down | 1 |

(3 non-work-scoped events also in the journal: `daemon.started` ×1, `backend.probed` ×2 — daemon startup, before this work was submitted.)

## Artifact inventory (copied out of the worktree into this directory)

All paths below are relative to this manifest's directory. Copied from the surface at `data/surfaces/01KZQ32J2BAD4P8WJA9SWXRMZ9/subject/`, which remains `retained_dirty` (untouched) as the durable evidence copy.

### `.sergeant/workflows/repo-to-icm/*/output/` — 25 files (10 `README.md` + 15 content files)

| Stage | File | Bytes |
|---|---|---|
| 00-contract | contract.md | 5,938 |
| 10-inventory | inventory.md | 35,234 |
| 20-harvest | behavior-units.ndjson | 375,142 |
| 20-harvest | consequence-class-sweep.md | 4,870 |
| 20-harvest | partition-ledger.md | 2,344 |
| 30-normalize | behavior-units.normalized.ndjson | 385,547 |
| 40-classify | classifications.ndjson | 144,957 |
| 50-synthesize | candidates.md | 56,218 |
| 60-draft | draft-report.md | 23,493 |
| 70-lint | lint-report.md | 11,621 |
| 80-adversarial-review | findings.ndjson | 8,799 |
| 80-adversarial-review | review-summary.md | 9,912 |
| 90-reconcile | adjudication-log.md | 6,394 |
| 90-reconcile | grammar-pressure.ndjson | 8,062 |
| 90-reconcile | measurement-package.md | 12,449 |

(plus one `output/README.md` per stage, 10 total, not itemized above)

### `.sergeant/drafts/workflows/` — 18 draft ICM workflow packages, 124 files

| Package | Files |
|---|---|
| callback-delivery | 4 |
| cross-repo-planning | 4 |
| dag-orchestration | 6 |
| dispatch-worker | 16 |
| fleet-cleanup | 6 |
| fleet-monitor-and-reconcile | 4 |
| installation-and-setup | 6 |
| project-graphify | 6 |
| project-registration | 6 |
| review-finding-routing | 8 |
| sergeant-help-query | 4 |
| shipping-gate-driving | 6 |
| skill-adoption | 4 |
| task-intake-and-execution | 20 |
| troubleshoot-td-identity | 4 |
| undocumented-failure-escalation | 4 |
| validation-gate | 8 |
| worker-response-and-recovery | 8 |

18 draft packages matches `60-draft/output/draft-report.md`'s own count ("18/18 candidates" materialized from `50-synthesize`'s Buckets 1–3).

**Total copied files: 149** (25 repo-to-icm outputs + 124 draft-package files).

## Ambiguity roll-up

The workflow's fail-closed convention is a literal `# AMBIGUOUS — NOT RESOLVED` first line on an Inputs-table artifact, checked by each downstream stage before proceeding. **That marker was never triggered anywhere in this run** — every stage from `10-inventory` through `90-reconcile` explicitly checked for it on its declared inputs and recorded that it was absent, so all 10 stages proceeded with ordinary work rather than the fail-closed path.

However, `80-adversarial-review` surfaced a real, severe **coverage gap** that is functionally the same class of problem (downstream stages silently treating an incomplete corpus as complete) even though it never tripped the literal marker:

- **Root cause (AF-0001/AF-0002, both `severity: high`):** `20-harvest` only reached 6 of the 21 partitions `10-inventory` scoped in (P1–P6, 28 of 82 `decompose`-dispositioned files = 34%). `partition-ledger.md` still shows P7–P21 as `pending`, and `consequence-class-sweep.md` has zero rows (not even "swept, none found") for the 54 unreached files.
- **Propagation (AF-0003, `severity: high`):** `30-normalize`, `40-classify`, `50-synthesize`, and `60-draft` all treated the resulting 312-record ledger as the full corpus without disclosing the 28/82-file shortfall anywhere.
- **Secondary findings:** AF-0004 (`medium`) — a 5-record circular-justification chain in `40-classify` claiming a `recover-worker` stage checkpoint is "established elsewhere" when no such record exists anywhere in the corpus. AF-0005 (`medium`) — an empty classification bucket (`obsolete-mechanism`) justified by count alone instead of per-candidate reasoning.
- **90-reconcile disposition (all 5 findings adjudicated, 0 rejected — every finding held up on independent re-verification):**
  - AF-0001 — **park** (unrepairable in `90-reconcile`; requires re-running `20-harvest` for P7–P21 and everything downstream)
  - AF-0002 — **park** (same root cause as AF-0001)
  - AF-0003 — **accept (partial)** — coverage caveat added in place to `50-synthesize/output/candidates.md` and `60-draft/output/draft-report.md`; the two NDJSON ledgers were left unedited (no field for a document-level caveat without violating their declared one-record-per-line shape)
  - AF-0004 — **accept** — rewrote the 5 records' `rationale` fields in `classifications.ndjson` to state plainly that no attached stage record exists, rather than claiming one exists elsewhere; did **not** promote any record to `representation: stage` (flagged as an open question for a follow-up/dedicated review, since restructuring an already-lint-validated, already-reviewed package is outside `90-reconcile`'s mandate)
  - AF-0005 — **park** (unrepairable in `90-reconcile`; requires `50-synthesize` to re-run its Unused-tiers check with actual per-candidate reasoning)
- **Bottom line (per `90-reconcile/output/measurement-package.md`):** "This run's single largest defect" — every count and every one of the 18 draft workflow packages in this run's deliverables describes/derives from the 28-file (34%) corpus actually harvested, not the 82-file corpus `00-contract` scoped the run to cover. The gap is disclosed (in the two prose artifacts capable of carrying a caveat, plus the adjudication log and measurement package) but **not closed** — closing it requires a follow-up run that resumes `20-harvest` at P7 and re-runs everything downstream.

## Daemon shutdown

Command run: `sgt --data-dir <data_dir> daemon stop` (or SIGTERM to pid 5164 per docs) after evidence collection completed.

`pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` — **empty** (verified after shutdown; see task completion notes).

Evidence directory `/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run3` was **not** deleted, per instructions.
