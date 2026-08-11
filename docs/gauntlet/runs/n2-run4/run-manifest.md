# N2 measurement run — n2-run4

- **Subject SHA:** `2d86381669372a7ce18a75f5a0d8a668c3d71189` (base_sha == head_sha; the surface's own git log shows two further commits on top — `finalize.py`'s capture commit and its disposition-apply commit — the *subject repo* content itself, i.e. `reference/sergeant-upstream` + `UPSTREAM.md` + `.sergeant` + `AGENTS.md`, is unchanged from this SHA)
- **Work id:** `01KZQRGZE32RQ79KT82XTB9MV2`
- **Workflow:** `repo-to-icm` v2, 10 stages (`00-contract` … `90-reconcile`)
- **Backend:** `fake` (deterministic in-process backend; each stage held for external input via `sgt respond`, same overnight-driver continuation pattern as run 3)
- **Intent:** "Decompose the repository subtree `reference/sergeant-upstream` — pinned per `reference/UPSTREAM.md` at upstream SHA `f430cfd4f90174a98adbd7abebbece6303817929` — into draft ICM workflows per `.sergeant/workflows/repo-to-icm`. Scope: the subtree only; exclude `reference/UPSTREAM.md` itself, `.sergeant/`, and `AGENTS.md`."
- **Data dir:** `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/n2-run4/data`
- **Final `work.state` (via `sgt work show --json`, queried live against the still-running daemon):** **`completed`**
  - Final stage: `90-reconcile`, index 9 of 10, `status: completed`
  - `execution.stop_requested: true` on the final (10th) execution — consistent with the fake backend's per-stage hold pattern, not a mid-run abort
  - `teardown.clean: false` — worktree left `retained_dirty` by design (deliverables are untracked/committed writes inside the surface's own git history; see the surface's own `git log`, 3 commits: the subject seed at `2d86381`, `finalize.py`'s capture commit `23f5ede`, and its disposition-apply commit `a5fcabb` — worktree itself is clean, `git status --porcelain` empty)

Note on the CLI surface: `sgt work show` has no separate "event list" verb — work-scoped events were extracted directly from the journal (filtering `work_id == 01KZQRGZE32RQ79KT82XTB9MV2`), which is the only source of truth per this repo's architecture invariant ("the journal is the only truth").

## SEEDING provenance — this is a resumed run, not a fresh harvest

Run 4 is a resume of run 3's stalled `20-harvest`. Run 3 stopped after reaching only 6 of 21 partitions (P1–P6, 312 units, 28/82 `decompose` files, 34%) — recorded as run 3's single largest defect (AF-0001/AF-0002/AF-0003, all `park`, in run 3's own `90-reconcile/output/adjudication-log.md`). Run 4's mandate was to resume at P7 and drive the corpus to completion.

- **At harvest attempt 1** (the sole execution record against `20-harvest`, `attempt: 1` — see Journal stats below), run 3's own committed `20-harvest` outputs were copied into run 4's `20-harvest/output/` as the seed for the resume, from:
  - **Source path:** `/home/miztertea/sergeant-rs/docs/gauntlet/runs/n2-run3/.sergeant/workflows/repo-to-icm/20-harvest/output`
  - **Files copied verbatim:** `behavior-units.ndjson` (BU-0001–BU-0312, partitions P1–P6, all marked `done`), `consequence-class-sweep.md`, `partition-ledger.md` (re-labeled with a "Scheme provenance (run 4)" section explaining the seed), plus `run3-inventory.md` — a copy of run 3's own `10-inventory/output/inventory.md`, taken for **scheme provenance only** (run 3 partitioned by P1–P21; this run's own independent `10-inventory` stage partitioned the same file census by its own A–S scheme — 19 partitions, zero name overlap, reconciled at the **file level** per `partition-ledger.md`'s scheme-provenance note, not the partition-label level).
  - **Fail-closed census check (per the seeding instruction):** before harvesting P7 onward, run 4 diffed the union of run 3's P1–P21 decompose files (82) against its own A–S decompose census (83) and found exactly one mismatch — `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`, dispositioned `decompose` by run 4's own `10-inventory` but `helper-evidence` by run 3. This is a genuine, independently-arising disposition disagreement between two runs' judgment, not a defect in either; the harvest actor ruled in run 3's favor (file stays excluded from harvest) — this ruling is itself the subject of adjudication findings AF-0001/AF-0002 below (both `accept`, repair applied to the *citation*, not the disposition).
  - `run3-inventory.md` was **not** promoted to the final committed evidence: `finalize.py`'s dry-run correctly flagged it `would remove` (a scheme-provenance working copy, not a declared stage output per `20-harvest/output/README.md`'s manifest) and it was removed as part of the finalize disposition — it is **not** present in the copy under this directory (consistent with the surface's own post-finalize state).
  - Partitions P7–P21 (15 partitions, BU-0313–BU-1333, 1,021 new units across 54 files) were harvested fresh by this run's own actor, continuing the same P-numbering, before the fail-closed census check above and the rest of the pipeline ran.

## Stage timings (`stage.entered` → `stage.completed`, from the journal)

| # | Stage | Entered (UTC) | Completed (UTC) | Duration |
|---|---|---|---|---|
| 0 | 00-contract | 2026-08-11T06:35:27.595Z | 2026-08-11T06:38:13.944Z | 2m 46s (166.3s) |
| 1 | 10-inventory | 2026-08-11T06:38:13.944Z | 2026-08-11T06:49:13.094Z | 10m 59s (659.2s) |
| 2 | 20-harvest | 2026-08-11T06:49:13.094Z | 2026-08-11T09:26:55.260Z | 2h 37m 42s (9462.2s) |
| 3 | 30-normalize | 2026-08-11T09:26:55.261Z | 2026-08-11T09:57:09.888Z | 30m 15s (1814.6s) |
| 4 | 40-classify | 2026-08-11T09:57:09.889Z | 2026-08-11T10:51:07.804Z | 53m 58s (3237.9s) — attempt 1 entered → attempt 2 completed; see note below |
| 5 | 50-synthesize | 2026-08-11T10:51:07.805Z | 2026-08-11T10:51:07.805Z | <1ms |
| 6 | 60-draft | 2026-08-11T10:51:07.805Z | 2026-08-11T10:51:07.805Z | <1ms |
| 7 | 70-lint | 2026-08-11T10:51:07.806Z | 2026-08-11T10:51:07.806Z | <1ms |
| 8 | 80-adversarial-review | 2026-08-11T10:51:07.806Z | 2026-08-11T10:51:07.807Z | 1ms |
| 9 | 90-reconcile | 2026-08-11T10:51:07.807Z | 2026-08-11T10:51:07.807Z | <1ms |

- **Total stage-active time (sum of the 10 durations):** 15,340.2s ≈ 4h 15m 40s
- **Wall clock, `work.submitted` → `work.completed`:** 2026-08-11T06:35:27.555Z → 2026-08-11T10:51:07.808Z = 15,340.3s — matches the sum above (no idle time between stages, all elapsed time is inside stages).
- Stages 0–4 each held for external input (fake backend's hold-per-stage mechanism: `stage.needs_input` → `work.needs_input` → `stage.input_received` → `work.resumed` → `stage.completed`), so their durations reflect real wall-clock time doing work outside the daemon between the hold and the `sgt respond` that released it (the 20-harvest and 40-classify partitions' actual harvest/reconciliation work), not raw backend compute time. Stages 5–9 (`50-synthesize` through `90-reconcile`) ran back-to-back with no hold, immediately after `40-classify`'s attempt-2 retry — the fake backend advanced them synthetically in one burst (all within the same millisecond), consistent with how the driver loop was operated for this closing segment.

**40-classify note — daemon restart mid-hold, one retry:** `40-classify` attempt 1 entered `needs_input` normally at 09:57:09.889Z. Before it was answered, the daemon was stopped (`daemon.stopped`, 10:42:12.745Z) and restarted (`daemon.started`, 10:50:34.803Z — 502.1s / 8m 22s down). On restart, delivering the queued answer (`stage 40-classify complete`) failed: `stage.blocked` / `work.blocked`, reason `` backend "fake" does not recognise execution 01KZR42A40672QSV4SZ3NZ9PZP `` — the fake backend's execution registry is in-process state that does not survive a daemon restart (consistent with this repo's "work state ≠ process state" invariant: the durable work identity survived, the process-scoped execution handle did not). A `work.retry` command (32.7s later, 10:51:07.803Z) opened `40-classify` **attempt 2**, which completed immediately (fake backend, no further hold) and cascaded straight through `50-synthesize`–`90-reconcile` to `work.completed`.

## Journal stats

- **Journal directory:** `data/journal/` — single segment, `00000001.ndjson`
- **Journal bytes:** 186,458
- **Total events in journal (all work + daemon-scoped):** 125
- **Events for this work (`work_id` filter):** 98 (the other 27: 11 daemon-scoped events with no `work_id` — 3× `daemon.started`, 2× `daemon.stopped`, 6× `backend.probed`, across the one mid-run restart — plus the false-start work `01KZQRF6TP6113W82B7CE3XHP6`'s own 16 events, submitted and canceled before the real work was submitted)
- **Retry / attempt count on `20-harvest` (execution records):** **1** — a single `execution.reserved` / `execution.started` pair, `attempt: 1`. Two `work.retry` commands submitted during the stage's `needs_input` hold (07:14:32Z, 07:25:59Z) were both **rejected** (`not_retryable`, HTTP 409 — `work.retry` only applies to `failed`/`blocked`/`waiting` work, and this work was correctly `needs_input`, not one of those). The stage was actually released the intended way, ~2h1m later, by `stage.input_received` (`input: "stage 20-harvest complete"`) at 09:26:55.260Z — the harvest itself (P7–P21) happened as out-of-band actor work during that hold, then the driver answered the stage once it was done, matching this repo's fake-backend hold/respond pattern. Net: **1 harvest attempt, 0 harvest retries, 2 retry commands rejected as inapplicable.**
- **Event kind breakdown for this work (`work_id` filter, 98 events):**

| kind | count |
|---|---|
| stage.entered | 11 |
| execution.reserved | 11 |
| execution.started | 11 |
| stage.completed | 10 |
| execution.stopped | 10 |
| command.rejected | 8 |
| command.accepted | 7 |
| work.resumed | 6 |
| work.needs_input | 5 |
| stage.needs_input | 5 |
| stage.input_received | 5 |
| workflow.bound | 1 |
| work.submitted | 1 |
| work.started | 1 |
| work.completed | 1 |
| work.blocked | 1 |
| surface.torn_down | 1 |
| surface.materializing | 1 |
| surface.materialized | 1 |
| stage.blocked | 1 |

(11 `stage.entered`/`execution.*` rather than 10, because `40-classify` entered twice — attempt 1 and attempt 2, per the daemon-restart note above; `stage.needs_input`/`work.needs_input`/`stage.input_received` are 5 rather than 10 because attempt 2 of `40-classify` and stages 5–9 never held.)

## Artifact inventory (copied out of the worktree into this directory)

All paths below are relative to this manifest's directory. Copied from the surface at `data/surfaces/01KZQRGZE32RQ79KT82XTB9MV2/subject/`, which remains present and untouched (its own git history — 3 commits — is the durable evidence copy; this directory is a flat snapshot of its `output/` and `.sergeant/drafts/workflows/` trees for review convenience).

### `.sergeant/workflows/repo-to-icm/*/output/` — 25 files (10 `README.md` + 15 content files)

| Stage | File | Bytes |
|---|---|---|
| 00-contract | contract.md | 5,676 |
| 10-inventory | inventory.md | 24,509 |
| 20-harvest | behavior-units.ndjson | 1,500,298 |
| 20-harvest | consequence-class-sweep.md | 20,402 |
| 20-harvest | partition-ledger.md | 20,021 |
| 30-normalize | behavior-units.normalized.ndjson | 1,531,581 |
| 40-classify | classifications.ndjson | 499,013 |
| 50-synthesize | candidates.md | 410,926 |
| 60-draft | draft-report.md | 62,561 |
| 70-lint | lint-report.md | 26,767 |
| 80-adversarial-review | findings.ndjson | 9,824 |
| 80-adversarial-review | review-summary.md | 17,368 |
| 90-reconcile | adjudication-log.md | 8,706 |
| 90-reconcile | grammar-pressure.ndjson | 3,552 |
| 90-reconcile | measurement-package.md | 14,029 |

(plus one `output/README.md` per stage, 10 total, not itemized above; `20-harvest`'s working-copy `run3-inventory.md` was removed by `finalize.py`'s disposition run — see SEEDING provenance above — and is correctly absent here.)

### `.sergeant/drafts/workflows/` — 44 draft ICM workflow packages, 549 files

| Package | Files | Package | Files |
|---|---|---|---|
| adopt-external-skill | 6 | list-projects | 6 |
| callback-protocol | 19 | list-tasks | 6 |
| check-repo-status | 6 | load-project | 10 |
| ci-verification | 6 | no-mistakes-finding-routing | 8 |
| code-review | 10 | notify-primary-session | 9 |
| cross-repo-work | 12 | prototype | 18 |
| dag-run | 13 | record-recovery-pointer | 6 |
| design-it-twice | 10 | register-project | 6 |
| diagnose-bug | 14 | research | 6 |
| direct-mode | 12 | resolve-merge-conflict | 10 |
| dispatch-mode | 43 | review-findings-routing | 10 |
| domain-modeling | 9 | sergeant-help | 8 |
| fleet-status-listing | 6 | sergeant-setup | 24 |
| graphify | 8 | standard-workflow | 20 |
| grilling | 6 | sync-project-repos | 8 |
| implement | 10 | tdd | 8 |
| install-sergeant | 12 | to-spec | 10 |
| invoke-grill-with-docs | 6 | to-tickets | 16 |
| — | — | treehouse-init | 6 |
| — | — | triage | 21 |
| — | — | troubleshoot-failure | 6 |
| — | — | validation-pipeline-gate | 33 |
| — | — | wayfinder | 13 |
| — | — | wiki-maintenance | 8 |
| — | — | worker-contract | 8 |
| — | — | worker-lifecycle | 57 |

44 draft packages matches `60-draft/output/draft-report.md`'s own count ("44/44 candidates" materialized from `50-synthesize`'s Buckets 1–3); 182 stage directories total (174 directly evidenced + 8 single-stage design inferences), per `90-reconcile/output/measurement-package.md`'s "Draft materialization" line.

**Total copied files: 574** (25 repo-to-icm outputs + 549 draft-package files).

## Final unit/package counts vs. run 3

| Metric | Run 3 (stalled) | Run 4 (this run, resumed to completion) |
|---|---|---|
| Partitions harvested | 6 of 21 (P1–P6) | **21 of 21 (P1–P21)** |
| `decompose` files covered | 28 of 82 (34%) | **82 of 82 (100%)** — the 83rd file (`hitl-loop.template.sh`) is a ruled exclusion (see SEEDING provenance), not an uncovered gap |
| Behavior units | 312 | **1,333** (4.27×) |
| Classification records | 312 | **1,333** |
| Draft workflow candidates/packages | 18 | **44** (2.44×) |
| `agents-invariant` candidates (listed, not drafted) | not separately reported at run-3 scale | 126 |
| `shared-helper` candidates | not separately reported at run-3 scale | 23 (9 contract groups) |

Run 3's own `measurement-package.md` disclosed its 312/18 figures as describing only the 34%-coverage corpus, not the full 82-file scope `00-contract` had set — "this run's single largest defect," parked for a follow-up resume. Run 4 is that follow-up: closes the coverage gap identified in run 3 (21/21 partitions, 82/82 in-scope files, 0 `pending`), and its 1,333/44 figures describe the full corpus.

## Ambiguity roll-up

The workflow's fail-closed convention is a literal `# AMBIGUOUS — NOT RESOLVED` first line on an Inputs-table artifact, checked by each downstream stage before proceeding. **That marker was never triggered anywhere in this run** — every stage from `10-inventory` through `90-reconcile` checked for it on its declared inputs and recorded that it was absent.

`80-adversarial-review` recorded **6 findings** (1 high, 4 medium, 1 low) across 3 of 4 challenge axes (boundary-honesty 2, structural-self-consistency 3, invention 1; Axis 3 — engine-gap refutation — had 0 records to re-attempt, since 0 `engine-gap` classifications exist in this corpus). `90-reconcile`'s adjudication: **5 accept, 1 reject, 0 park**.

| Finding | Severity | Axis | Disposition |
|---|---|---|---|
| AF-0001 | high | boundary-honesty | **accept** — `20-harvest/output/partition-ledger.md`'s census-mismatch ruling quoted specific `reference-corpus/` content (a blindness-boundary violation) to justify the `hitl-loop.template.sh` exclusion; the specific citation is redacted, the exclusion **disposition itself is unchanged** (file stays excluded, per L9) |
| AF-0002 | medium | boundary-honesty | **accept** — the same ledger section framed the run as an actively graded comparison, contradicting `00-contract/output/contract.md` §3's explicit no-measurement-framing ruling; the framing language is removed, cross-referenced to §3 |
| AF-0003 | medium | structural-self-consistency | **accept** — `inventory.md`'s 83-file `decompose` count was never annotated to reflect that harvest only reached 82 (for the AF-0001-ruled reason); a cross-reference note was added, the 83 count itself is unchanged (it is independently correct) |
| AF-0004 | medium | structural-self-consistency | **accept** — `consequence-class-sweep.md`'s 82-row table has no note explaining why it's one short of the 83-file census; a cross-reference note was appended |
| AF-0005 | low | structural-self-consistency | **reject** — `partition-ledger.md` (run 3's P1–P21 naming) vs. `inventory.md` (this run's own A–S naming) diverge with zero label overlap, but both files disclose and explain it (see SEEDING provenance above); requiring identical partition names across an independently-repartitioned resume boundary is not this workflow's method — no repair applied |
| AF-0006 | medium | invention | **accept** — BU-0040/BU-0041's `source.locator` recovered less than half of the actual quoted span (`AGENTS.md L150-153` vs. the true `L150-157`); not fabrication (quote + hash both genuine), a locator defect — corrected in both `20-harvest` and `30-normalize` ndjson |

**Bottom line:** 5 of 6 findings were real defects in this run's own committed evidence, all repaired in place (citation redactions, cross-reference notes, one locator correction); none required reopening a stage or re-deriving a count. No accepted finding was an Axis-3 engine-gap refutation, so no classification record's `representation` changed as a result of this adjudication. This run's own remaining disclosed caveat (not itself an adjudicated finding — observed directly while assembling `measurement-package.md`): `00-contract`/`10-inventory` header themselves with this run's own work id (`01KZQRGZE32RQ79KT82XTB9MV2`), while the seeded `20-harvest` artifacts (`partition-ledger.md`, `consequence-class-sweep.md`) still header themselves with run 3's work id (`01KZQ32J2BAD4P8WJA9SWXRMZ9`) — every number in `measurement-package.md` was independently recomputed from each file's own content, not copied via the header, so this does not affect any reported count, but a reader relying on the header alone to confirm run identity should know the two headers disagree.

**Coverage gap closed, per run 3's parked findings:** run 3's AF-0001/AF-0002 (park — "requires re-running `20-harvest` for P7–P21 and everything downstream") and AF-0003 (accept-partial, same root cause) are the reason run 4 exists. Run 4's own `10-inventory/output/inventory.md` and `20-harvest/output/partition-ledger.md` both report 21/21 partitions `done`, 0 `pending` — the gap run 3 disclosed but could not close is closed in this run.

## Daemon shutdown

`sgt` has no `daemon stop` subcommand (`daemon` only runs in the foreground until signaled); graceful shutdown is `SIGTERM` to the daemon pid, per its own `--help` text ("Run the daemon in the foreground until SIGINT/SIGTERM"). Ran `kill -TERM 2610841` after evidence collection completed; `daemon.log` recorded `shutdown signal received` and the process exited.

`pgrep -f "debug/sgt [-]-data-dir"` — **empty** (verified after shutdown, exit code 1 / no matches).

Evidence directory `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/n2-run4` was **not** deleted, per instructions.
