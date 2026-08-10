# Provenance — Drain Fleet

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W12** `drain-fleet`.

## Stages

### `00-set-drain` (folded into `30-force-stop`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-057` | Whether admission (new dispatch or relaunch) is currently allowed is decided purely by the presence of a drain file — global or project-scoped — and an empty or unparseable project name is treated as absent, checking only the global drain rather than erroring. | `reference/sergeant-upstream/bin/_sgt-drain.sh` (L93-107) |
| `BU-P6-058` | A concurrent 'read drain state, then start new work' race is closed by an explicit admission lock that every dispatch/relaunch procedure and every drain-set/undrain procedure must acquire before reading or writing drain state, so a drain set mid-dispatch is never silently missed. | `reference/sergeant-upstream/bin/_sgt-drain.sh` (L109-114) |
| `BU-P6-062` | A drain-lock acquisition failure that stems from the filesystem itself being unable to create hard links (e.g. FAT/exFAT, some CIFS/FUSE mounts) is distinguished from ordinary contention, because spinning to the deadline and reporting 'contended' would send an operator chasing a holder that does not exist. | `reference/sergeant-upstream/bin/_sgt-drain.sh` (L458-467) |
| `BU-P8-077` | A drain refuses new worker starts within its scope while still storing incoming responses generation-safely for later delivery, --wait activates the drain and then waits for in-scope live workers to finish their current turn and exit, and on timeout it leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them. | `reference/sergeant-upstream/docs/using-sergeant.md` (L231-243 (Pause admission with a drain)) |

### `10-await-convergence` (folded into `30-force-stop`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-064` | A worker is only ever counted as having genuinely finished draining when its recorded process is provably gone; absence of recorded identity is explicitly not treated as proof of exit, so an unverifiable worker blocks a drain wait rather than being silently counted as resolved. | `reference/sergeant-upstream/bin/sgt-drain` (L147-152) |
| `BU-P8-077` | A drain refuses new worker starts within its scope while still storing incoming responses generation-safely for later delivery, --wait activates the drain and then waits for in-scope live workers to finish their current turn and exit, and on timeout it leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them. | `reference/sergeant-upstream/docs/using-sergeant.md` (L231-243 (Pause admission with a drain)) |

### `20-worker-side-checkpoint` (folded into `30-force-stop`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-111` | A cooperative drain of one worker publishes every durable fact it can before terminating anything — a handoff, settlement of the outstanding action lease, and the drained status — and only after everything durable is published does it begin terminating processes, because a drain must never be a way to discard unfinished work. | `reference/sergeant-upstream/bin/sgt-interactive-worker` (L219-234) |
| `BU-P7-084` | A cooperative drain must actually terminate the worker's entire process group — not merely the backgrounded watcher subshell that detects the drain signal — and it must publish its durable handoff and finalize the action lease BEFORE terminating, leaving no live execution context and no surviving process behind. | `reference/sergeant-upstream/tests/sgt-drain-terminate-test.sh` (lines 1-14) |
| `BU-P7-107` | A cooperative drain checkpoint inside the interactive worker must, on detecting drain, produce a clean exit with a `td` handoff written — including verifying the worktree it hands off from (per the same worktree-verification contract sgt-td-memory enforces elsewhere) — rather than exiting as if orphaned. | `reference/sergeant-upstream/tests/sgt-worker-drain-test.sh` (lines 20-25) |
| `BU-P7-108` | Cooperative drain detection inside the worker must be idempotent: an already-drained marker file present on disk must prevent a redundant re-drain, and it must distinguish global-drain, project-drain-match, project-drain-no-match, and no-drain-signal cases correctly, preserving all other worktree files across the drain transition. | `reference/sergeant-upstream/tests/sgt-drain-worker-test.sh` (lines 10-18) |

### `30-force-stop`

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-039` | Force-stopping workers is refused unless a cooperative drain is already active for the targeted scope, and it always requires explicit confirmation (--yes) or is limited to a --dry-run preview; it never runs automatically as a side effect of anything else. | `reference/sergeant-upstream/bin/sgt-drain-force` (L1-4, L45-46, L58-62) |
| `BU-P7-083` | sgt-drain-force must require an active drain and an explicit `--yes` (or offer `--dry-run`) before force-stopping any drain-eligible worker, and it must display the exact worker identity before stopping it, and it invokes a harness-specific backstop (e.g. a Claude background-session stop call) as part of the force-stop loop. | `reference/sergeant-upstream/tests/sgt-drain-force-test.sh` (line 2 and source-inventory description) |

### `40-undrain` (folded into `30-force-stop`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P6-015` | Removing a drain is explicitly idempotent: undraining a scope that is not currently drained still exits successfully, and --global and a named project are mutually exclusive scopes that cannot both be targeted in one invocation. | `reference/sergeant-upstream/bin/sgt-undrain` (L8-9, L47) |

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02):

| Stage | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|
| `00-set-drain` | none | Swapping the drain-file/lock implementation leaves the checkpoint — admission refused the instant the drain is set — unchanged. | **Demoted** — folded into `30-force-stop` as a preceding helper invocation (first). |
| `10-await-convergence` | none | Swapping the wait/liveness-check implementation leaves the checkpoint — bounded wait, provable exit only — unchanged. | **Demoted** — folded into `30-force-stop` (second). |
| `20-worker-side-checkpoint` | none | Swapping the handoff/lease-settlement implementation leaves the checkpoint — durable facts published before termination — unchanged. | **Demoted** — folded into `30-force-stop` (third, immediately preceding its judgment). |
| `30-force-stop` | n/a (extracted as actor-stage, §6.4, already judgment-bearing) | n/a | **Kept.** |
| `40-undrain` | none | Swapping the undrain implementation leaves the checkpoint — idempotent, mutually exclusive scopes — unchanged. | **Demoted** — folded into `30-force-stop` as a following helper invocation. Its `promote` output disposition is inherited by the merged stage output (see `30-force-stop/output/README.md`). |

Stage count: 5 extracted → 1 surviving. No behavior unit was deleted; all nine citations above remain cited, under `30-force-stop`'s own contract or its "Helper invocations" section (see that stage's `CONTEXT.md`).

## Notes

**Synthesis notes:** Raises engine-gap **G4** (operator-declared, durable, scope-qualified admission block) — survives, ranked high-evidence/low-cost. See `reference-corpus/synthesis.md` §5.

