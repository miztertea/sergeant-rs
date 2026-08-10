# Provenance — Monitor Fleet

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W13** `monitor-fleet`.

## Stages

### `00-snapshot`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-101` | A bounded, side-effect-free activity snapshot answers exactly one narrow question — is Sergeant verifiably doing work right now — as constant-size versioned JSON, and reports busy:true only when ALL of a stable in_progress status, an exact live worker pane identity match, and recent progress attributable to that exact pane hold together; every other outcome is busy:null, because absence of a verified witness is never treated as proof of idleness. | `reference/sergeant-upstream/bin/sgt-watch` (L36-49) |
| `BU-P7-101` | `sgt-watch --snapshot` must be strictly read-only, constant-size, and versioned, and must only report the fleet as busy when it has a verified active witness — unlike `--list` (human-oriented, embeds free-form brief text) or `--sync`/`--sync-all` (which mutate lifecycle state), giving a coordinator or bridge a safe machine-readable answer to 'is Sergeant verifiably doing work right now?'. | `reference/sergeant-upstream/tests/sgt-watch-snapshot-test.sh` (lines 1-12) |

### `10-evaluate-liveness`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-072` | Worker health must never be equated with the in_progress status alone; it requires exact live-process identity plus recent, meaningful progress evidence, using a defined fallback chain (pane activity, then recorded progress timestamp, then file mtime only as a last resort), and once that evidence exceeds the grace window the worker stays in_progress but a nonterminal 'live worker stalled' diagnostic is recorded rather than an automatic failure or kill. | `reference/sergeant-upstream/docs/using-sergeant.md` (L161-172 (Worker states)) |

### `20-reconcile-terminal`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-103` | Fleet reconciliation recognizes a specific hazardous case — a status transitioning to done while the worktree's actual result file is empty — and refuses to accept it as a genuine completion, instead marking the Work orphaned with a diagnostic requiring a result before done can be trusted. | `reference/sergeant-upstream/bin/sgt-watch` (L561-567) |
| `BU-P6-104` | Retiring a terminal (done, failed, or drained) worker's pane recycling evidence is bound to the exact pane identity being retired, not merely stamped as a permanent task-level marker — because binding to the wrong scope (any prior recycling ever) permanently suppressed recycling of every later relaunched pane once one pane had ever been recycled. | `reference/sergeant-upstream/bin/sgt-watch` (L286-292) |
| `BU-P6-105` | Recycling a terminal worker's pane first settles its accepted notification action-lease before the pane is taken away, because recycling used to stop the only process that could ever publish completion, which is exactly how a completed turn became permanently unrecoverable. | `reference/sergeant-upstream/bin/sgt-watch` (L322-326) |
| `BU-P7-100` | Terminal-worker recycling must trigger for every terminal-adjacent status including `drained`, not only `done`/`failed:*`, and the recycling-suppression marker must be per-pane/identity-bound and clearable, not a permanent task-level flag — a marker stamped merely because a pane went absent must not permanently suppress recycling of every later relaunched pane. | `reference/sergeant-upstream/tests/sgt-watch-recycle-test.sh` (lines 5-11) |

### `30-background-watch`

| Unit | Statement | Source |
|---|---|---|
| `BU-P7-099` | `sgt-watch --background` must be idempotent (a duplicate start is detected, not double-started), must detect and report a failed background start, must recognize and clean up a stale systemd unit, and must handle platforms without systemd support gracefully, in addition to covering ordinary active/terminal transitions. | `reference/sergeant-upstream/tests/sgt-watch-background-test.sh` (lines 1-4) |

