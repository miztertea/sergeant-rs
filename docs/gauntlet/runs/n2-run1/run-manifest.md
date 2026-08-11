# N2 measurement run — run manifest (n2-run1)

Evidence capture for the N2 actor-only `repo-to-icm` workflow run on the
current engine. Written by the collection pass after the daemon was stopped;
source data remains under
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run/`
(not deleted — retained as evidence).

## Identity

| Field | Value |
|---|---|
| Subject repository | `reference/sergeant-upstream` (vendored subtree) — see ambiguity roll-up: this identification is the actor's own inference, never confirmed by the Work's task text |
| Subject base/head SHA (surface binding) | `d27227d9b8203705998f4c79370440def577b619` (`master`) |
| Work id | `01KZNT2Y5BX7S26PJB3B1QVADW` |
| Workflow | `repo-to-icm` v1, 10 stages |
| Backend | `fake` (scripted: `needs_input:hold-stage-N` → `complete:advanced-stage-N` for N=0..9) |
| Data dir | `.../n2-run/data` |
| Worktree | `.../n2-run/data/surfaces/01KZNT2Y5BX7S26PJB3B1QVADW/subject` |
| Daemon endpoint | `http://127.0.0.1:38457` (pid 22449, stopped via `SIGTERM`) |

## 1. Final Work state

`sgt work show 01KZNT2Y5BX7S26PJB3B1QVADW --json`:

- **`work.state`: `completed`**
- Final stage: `90-reconcile`, index 9 of 10, `status: completed`, `detail: advanced-stage-9`
- Execution: backend `fake`, `execution_id 01KZNT2Y8A7KM60M36Q6M8EBDM` → final execution `01KZNV88MX5PJD29GQVD21T6AJ`, `stop_requested: true`
- Teardown: `clean: false`, disposition `retained_dirty` — 13 files added under `.sergeant/workflows/repo-to-icm/*/output/` (listed under Artifact inventory below), branch `sergeant/01KZNT2Y5BX7S26PJB3B1QVADW`

The work reached `completed` — but every stage from `10-inventory` through
`90-reconcile` completed by **cleanly relaying an unresolved-ambiguity
marker**, not by doing its ordinary work. See §4 (Ambiguity roll-up) — this
is the headline finding of the run, not an incidental detail.

Event list: 98 events carry `work_id = 01KZNT2Y5BX7S26PJB3B1QVADW` (fetched
via `GET /v1/events?work_id=...&limit=1000` against the loopback API, since
the CLI has no `work events`/`work show --events` subcommand — `work_events`
is an `ApiViews`-only method). Event kind histogram for this work:

| kind | count |
|---|---|
| command.accepted | 11 |
| execution.started | 10 |
| execution.stopped | 10 |
| stage.completed | 10 |
| stage.entered | 10 |
| stage.input_received | 10 |
| stage.needs_input | 10 |
| work.needs_input | 10 |
| work.resumed | 10 |
| surface.materializing | 1 |
| surface.materialized | 1 |
| surface.torn_down | 1 |
| work.completed | 1 |
| work.started | 1 |
| work.submitted | 1 |
| workflow.bound | 1 |
| **total** | **98** |

## 2. Trajectory stats from the journal

Journal file: `data/journal/00000001.ndjson`, single segment.

- **Journal bytes: 106,447**
- **Total events in journal: 101** (101 lines, one JSON object per line)
- **Events for this work: 98** (matches the API query above)
- **Non-work events: 3** — `daemon.started` (1), `backend.probed` (2); these carry `work_id: null` (daemon/backend-lifecycle events, not attributable to any one work item)

### Per-stage timings (`stage.entered` → `stage.completed`)

Every stage in this run paused for `needs_input` immediately after
`stage.entered` and only resumed once a `stage.input_received` / `respond`
command landed (scripted by `fake_script.txt`), so the interval below is
"time the stage held its turn open," not raw compute time. `execution.started`
/ `execution.stopped` bracket the identical interval in every stage (their
timestamps match `stage.entered`/`stage.completed` to the millisecond), so no
separate execution-only figure is reported.

| # | stage_id | entered | completed | duration (s) |
|---|---|---|---|---|
| 0 | 00-contract | 12:24:15.882 | 12:28:19.196 | 243.31 |
| 1 | 10-inventory | 12:28:19.196 | 12:29:13.464 | 54.27 |
| 2 | 20-harvest | 12:29:13.465 | 12:30:29.833 | 76.37 |
| 3 | 30-normalize | 12:30:29.834 | 12:31:29.939 | 60.11 |
| 4 | 40-classify | 12:31:29.940 | 12:32:28.342 | 58.40 |
| 5 | 50-synthesize | 12:32:28.343 | 12:33:31.341 | 63.00 |
| 6 | 60-draft | 12:33:31.342 | 12:34:44.752 | 73.41 |
| 7 | 70-lint | 12:34:44.753 | 12:35:47.079 | 62.33 |
| 8 | 80-adversarial-review | 12:35:47.080 | 12:44:38.940 | 531.86 |
| 9 | 90-reconcile | 12:44:38.941 | 12:52:07.623 | 448.68 |

- **Total wall clock, `work.submitted` → last event (`command.accepted`):
  1671.85 s (~27.9 min)**, from `2026-08-10T12:24:15.787Z` to
  `2026-08-10T12:52:07.634Z`.
- Stages 1–7 (the pure AMBIGUOUS-relay stages) each took roughly
  54–76 s — consistent with a short mechanical relay turn.
- **`80-adversarial-review` (531.9 s) and `90-reconcile` (448.7 s) are
  6–10x longer** than the relay stages — consistent with those two stages
  doing genuine, substantive work (a real adversarial review producing two
  findings; a real adjudication pass reasoning about disposition) rather
  than a one-line relay, even though the corpus they had to review was
  itself the empty/ambiguous chain.
- `00-contract` (243.3 s) is the second-longest stage — consistent with it
  being the stage that actually performed the subject/revision
  investigation (multiple `find`/`git`/`grep` checks enumerated in its
  "What was checked" list) before concluding AMBIGUOUS.

## 3. Deliverables copied

Copied from the worktree's `.sergeant/workflows/repo-to-icm/*/output/` into
`docs/gauntlet/runs/n2-run1/<stage>/output/`, preserving stage structure —
**21 files**, all of the `git status --short` additions in the worktree
(13 tracked-content files + 8 stage `README.md` mirrors that
`git status` doesn't list separately as new content but exist as the
stage's own output-summary alongside the tracked artifact).

No `.sergeant/drafts/workflows/` directory exists anywhere in the worktree
— `60-draft` never placed anything there (confirmed by `find` and
corroborated by `80-adversarial-review`'s own finding AF-0001). Nothing to
copy from that path.

### Artifact inventory

| Stage | File | Bytes | Lines | Note |
|---|---|---|---|---|
| 00-contract | output/README.md | 1063 | 21 | |
| 00-contract | output/contract.md | 5167 | 86 | **AMBIGUOUS — NOT RESOLVED** (origin of the marker) |
| 10-inventory | output/README.md | 987 | 20 | |
| 10-inventory | output/inventory.md | 1346 | 25 | AMBIGUOUS relay |
| 20-harvest | output/README.md | 916 | 19 | |
| 20-harvest | output/behavior-units.ndjson | 2097 | 36 | AMBIGUOUS relay (no NDJSON records) |
| 30-normalize | output/README.md | 990 | 20 | |
| 30-normalize | output/behavior-units.normalized.ndjson | 1834 | 32 | AMBIGUOUS relay (no NDJSON records) |
| 40-classify | output/README.md | 876 | 19 | |
| 40-classify | output/classifications.ndjson | 1922 | 34 | AMBIGUOUS relay (no NDJSON records) |
| 50-synthesize | output/README.md | 1089 | 21 | |
| 50-synthesize | output/candidates.md | 2027 | 36 | AMBIGUOUS relay |
| 60-draft | output/README.md | 1279 | 24 | |
| 60-draft | output/draft-report.md | 3315 | 58 | AMBIGUOUS relay; no draft package materialized |
| 70-lint | output/README.md | 884 | 19 | |
| 70-lint | output/lint-report.md | 3406 | 60 | AMBIGUOUS relay |
| 80-adversarial-review | output/README.md | 971 | 23 | |
| 80-adversarial-review | output/findings.ndjson | 3327 | 2 | **Real content** — 2 findings, AF-0001 (high) / AF-0002 (medium) |
| 80-adversarial-review | output/review-summary.md | 6432 | 111 | **Real content** — genuine review, not a relay |
| 90-reconcile | output/README.md | 1461 | 31 | |
| 90-reconcile | output/adjudication-log.md | 4244 | 72 | AMBIGUOUS relay (records but does not dispose AF-0001/AF-0002) |
| 90-reconcile | output/grammar-pressure.ndjson | 3183 | 56 | AMBIGUOUS relay |
| 90-reconcile | output/measurement-package.md | 4161 | 76 | AMBIGUOUS relay; none of the §9.9 dimensions reported (would misrepresent an absent corpus) |

**21 files, 901 total lines** (`README.md` count included).

## 4. Ambiguity roll-up

**Headline: this run never produced an ICM decomposition.** `00-contract`
determined the subject/revision was unresolvable and wrote the fail-closed
`# AMBIGUOUS — NOT RESOLVED` marker; every downstream stage through
`90-reconcile` relayed that marker rather than inventing content in its
place, per `_config/run-discipline.md` §2's fail-closed rule. Two stages —
`80-adversarial-review` and `90-reconcile` — did real, substantive work
*about* that unresolved state (reviewing/adjudicating the propagation
itself), which is why their durations stand out in §2.

### Root cause (`00-contract/output/contract.md`)

- The Work's initiating task ("Decompose this repository's procedural
  knowledge into draft ICM workflows per
  `.sergeant/workflows/repo-to-icm`") names **no subject repository path
  and no revision**.
- The worktree contains exactly one candidate subject:
  `reference/sergeant-upstream`, a vendored subtree with no `.git` of its
  own. `_config/run-discipline.md`'s worked example and the stage's own
  `CONTEXT.md` both use it as their paradigm example, corroborating but not
  confirming the inference.
- Vendored-subtree subjects must have their pinned revision recorded in a
  provenance document (e.g. `UPSTREAM.md`) — none exists. The one
  provenance-shaped file that does exist,
  `reference/sergeant-upstream/.agents/skills/PROVENANCE.md`, records a
  narrower, unrelated fact (per-skill import hashes from a different
  upstream), and using it as the answer would be inventing a resolution
  method the subject doesn't have — explicitly forbidden by `CONTEXT.md`.
- Six checks were run and logged (`work show` intent text, worktree tree,
  `git`/`ls` vendored-subtree checks, `find -iname UPSTREAM.md`,
  README/AGENTS.md/.gitignore header scan, and a grep for
  vendor/provenance/pinned/revision/commit strings) — all came back
  negative for an explicit pin.
- Meta-level note recorded in `contract.md`: the engine gives no actor
  stage a way to pause its own turn and ask a human a disambiguating
  question mid-run — `needs_input`/`waiting` is runtime-driven from
  outside the actor's turn, never actor-requested. The only fail-closed
  action available was to write the marker and stop.

### Propagation (`10-inventory` → `70-lint`)

All six intermediate stages relayed the marker cleanly: no invented
behavior units, classification records, synthesis candidates, draft
packages, or lint findings were substituted in its place. Confirmed
independently by `80-adversarial-review`'s own investigation (finding
AF-0001, high severity) and by this collection pass's own `grep` above.

### Real findings surfaced despite the empty corpus (`80-adversarial-review`)

- **AF-0001** (`boundary-honesty`, high) — confirms the clean AMBIGUOUS
  propagation described above is itself the finding worth recording, not a
  defect to review past.
- **AF-0002** (`invention`, medium) — `contract.md`'s "What was checked"
  list claims `git -C reference/sergeant-upstream rev-parse
  --is-inside-work-tree` fails; re-running it does not fail (`git`
  resolves to the *outer* worktree's `.git` pointer file and returns
  `true`, exit 0). The sibling check that actually controls the
  vendored-subtree classification, `ls
  reference/sergeant-upstream/.git`, does correctly fail and does
  reproduce — so AF-0002 does not overturn the AMBIGUOUS determination,
  but the specific `git rev-parse` claim as written is not reproducible.

### Adjudication (`90-reconcile`)

Both findings are recorded but explicitly **left undisposed** (no
accept/reject/park) — `_config/run-discipline.md` §2 forecloses ordinary
adjudication once any Inputs-table artifact carries the marker, and
`CONTEXT.md` step 0 names adjudication (step 1) as one of the steps not to
proceed with, without a carve-out for findings whose substance doesn't
touch the subject/revision question. `measurement-package.md` reports none
of proposal §9.9's ten dimensions (rather than reporting hollow zeros),
and `scripts/finalize.py` (step 4) was never run — both omissions are
explained in-document rather than left silent.

### AMBIGUOUS-marker file count

- 10 of 13 tracked content artifacts (00-contract through 70-lint, plus
  90-reconcile's 3 files) open with `# AMBIGUOUS — NOT RESOLVED`.
- 2 files (`80-adversarial-review/output/findings.ndjson` and
  `review-summary.md`) contain genuine, non-relayed content.
- 0 files contain fabricated/invented decomposition content in place of
  the marker.

## 5. Daemon shutdown and leak check

- Graceful stop: `kill -TERM 22449` (pid recorded in `runtime.json`).
  `daemon.log` recorded `shutdown signal received` at
  `2026-08-10T12:55:20.322068Z`.
- Post-stop check: `ps -p 22449` — no such process.
- Leak check: `pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` —
  **empty** (only the invoking shell wrapper itself matches the raw,
  unfiltered `pgrep`, which is excluded by the `bash -c` filter as
  instructed).
- Evidence directory retained (not deleted):
  `/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-run/`
