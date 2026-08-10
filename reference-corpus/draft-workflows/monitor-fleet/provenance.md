# Provenance — Monitor Fleet

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W13** `monitor-fleet`.

## Stages

### `00-snapshot`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-101` | A bounded, side-effect-free activity snapshot answers exactly one narrow question — is Sergeant verifiably doing work right now — as constant-size versioned JSON, and reports busy:true only when ALL of a stable in_progress status, an exact live worker execution-instance identity match, and recent progress attributable to that exact instance hold together; every other outcome is busy:null, because absence of a verified witness is never treated as proof of idleness. | `reference/sergeant-upstream/bin/sgt-watch` (L36-49) |
| `BU-P7-101` | `sgt-watch --snapshot` must be strictly read-only, constant-size, and versioned, and must only report the fleet as busy when it has a verified active witness — unlike `--list` (human-oriented, embeds free-form brief text) or `--sync`/`--sync-all` (which mutate lifecycle state), giving a coordinator or bridge a safe machine-readable answer to 'is Sergeant verifiably doing work right now?'. | `reference/sergeant-upstream/tests/sgt-watch-snapshot-test.sh` (lines 1-12) |

### `10-evaluate-liveness`

| Unit | Statement | Source |
|---|---|---|
| `BU-P8-072` | Worker health must never be equated with the in_progress status alone; it requires exact live-process identity plus recent, meaningful progress evidence, using a defined fallback chain (session activity, then recorded progress timestamp, then file mtime only as a last resort), and once that evidence exceeds the grace window the worker stays in_progress but a nonterminal 'live worker stalled' diagnostic is recorded rather than an automatic failure or kill. | `reference/sergeant-upstream/docs/using-sergeant.md` (L161-172 (Worker states)) |

## Moved at N1 adjudication A7 (BH-07)

The two mutating stages formerly here — `20-reconcile-terminal` and
`30-background-watch`, citing `BU-P6-103`, `BU-P6-104`, `BU-P6-105`,
`BU-P7-100`, `BU-P7-099` — moved to `reconcile-and-cleanup-fleet`, which
already owns fleet mutation and cleanup (A7). Both further fold into that
package's sole surviving actor stage, `00-require-terminal`, under N1
adjudication A4 (see `reconcile-and-cleanup-fleet/provenance.md` §
"Adjudication A4" for the full disposition of each unit). This package's
own provenance keeps only its strictly read-only stages, below.

