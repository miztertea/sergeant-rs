# Provenance — Monitor Fleet

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W13** `monitor-fleet`.

## Stages

### `00-observe-and-interpret`

Own citations (actor-stage judgment — interpreting the two helper readings below and reporting the result, never acting on it):

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-101` | A bounded, side-effect-free activity snapshot answers exactly one narrow question — is Sergeant verifiably doing work right now — as constant-size versioned JSON, and reports busy:true only when ALL of a stable in_progress status, an exact live worker execution-instance identity match, and recent progress attributable to that exact instance hold together; every other outcome is busy:null, because absence of a verified witness is never treated as proof of idleness. | `reference/sergeant-upstream/bin/sgt-watch` (L36-49) |
| `BU-P7-101` | `sgt-watch --snapshot` must be strictly read-only, constant-size, and versioned, and must only report the fleet as busy when it has a verified active witness — unlike `--list` (human-oriented, embeds free-form brief text) or `--sync`/`--sync-all` (which mutate lifecycle state), giving a coordinator or bridge a safe machine-readable answer to 'is Sergeant verifiably doing work right now?'. | `reference/sergeant-upstream/tests/sgt-watch-snapshot-test.sh` (lines 1-12) |
| `BU-P8-072` | Worker health must never be equated with the in_progress status alone; it requires exact live-process identity plus recent, meaningful progress evidence, using a defined fallback chain (session activity, then recorded progress timestamp, then file mtime only as a last resort), and once that evidence exceeds the grace window the worker stays in_progress but a nonterminal 'live worker stalled' diagnostic is recorded rather than an automatic failure or kill. | `reference/sergeant-upstream/docs/using-sergeant.md` (L161-172 (Worker states)) |

Prior to N1 adjudication A4 these three units were split across two
separate §6.5 "deterministic-machinery candidate" stages (`00-snapshot`:
`BU-P6-101`, `BU-P7-101`; `10-evaluate-liveness`: `BU-P8-072`). Both folded
into this single actor stage — see "Adjudication A4" below.

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02) to this package's two stages, left in place by A7's transfer of the mutating stages elsewhere:

| Stage | Extraction rung | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|---|
| `00-snapshot` | stage (§6.5 candidate) | none | Swapping the snapshot-generation implementation leaves the checkpoint — a verified busy/idle answer, never a guess — unchanged. | **Demoted** — folded into `00-observe-and-interpret` as helper invocation 1. |
| `10-evaluate-liveness` | stage (§6.5 candidate) | none | Swapping the fallback-chain implementation (session activity → progress timestamp → file mtime) leaves the checkpoint — a correctly distinguished healthy/stalled worker — unchanged. | **Demoted** — folded into `00-observe-and-interpret` as helper invocation 2. |

Judgment call (recorded here per this finding's instruction): unlike every
other A4 sweep in this corpus, neither extracted stage in this package was
already an actor-stage for the demoted stage(s) to fold into — both were
§6.5 candidates with no surviving judgment argument. A4's default rule
(demote to helper of "its adjacent judgment-bearing stage") presupposes
such a stage exists; here it does not, and the package cannot correctly
end with zero judgment-bearing stages, since dispatch's `80-monitor` and
any operator caller need a caller-facing interpretation, not two raw
machinery outputs. Resolution: both stages fold into one new actor stage,
`00-observe-and-interpret`, whose own judgment is the interpretation these
two mechanical readings require (verified busy/idle; healthy/stalled) and
the read-only reporting/escalation-flagging of that interpretation — never
mutating fleet state, consistent with this package's read-only purpose
under A7. Stage count: 2 extracted (post-A7) → 1 surviving. No behavior
unit was deleted; all three citations above remain cited, now under
`00-observe-and-interpret`'s own contract, with `00-snapshot`'s and
`10-evaluate-liveness`'s content preserved as its "Helper invocations"
(see that stage's `CONTEXT.md`).

## Moved at N1 adjudication A7 (BH-07)

The two mutating stages formerly here — `20-reconcile-terminal` and
`30-background-watch`, citing `BU-P6-103`, `BU-P6-104`, `BU-P6-105`,
`BU-P7-100`, `BU-P7-099` — moved to `reconcile-and-cleanup-fleet`, which
already owns fleet mutation and cleanup (A7). Both further fold into that
package's sole surviving actor stage, `00-require-terminal`, under N1
adjudication A4 (see `reconcile-and-cleanup-fleet/provenance.md` §
"Adjudication A4" for the full disposition of each unit). This package's
own provenance keeps only its strictly read-only stage, `00-observe-and-interpret`, above.

