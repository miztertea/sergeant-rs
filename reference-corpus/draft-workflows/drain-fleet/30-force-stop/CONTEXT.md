# 30-force-stop: set drain, await convergence, worker-side checkpoint, force-stop, then undrain

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Force-stop is refused unless a drain is already active; requires explicit confirmation or dry-run; displays exact identity. This is the workflow's sole judgment-bearing checkpoint (N1 adjudication A4): every other behavior this package owns — setting the drain, waiting for cooperative convergence, the worker-side cooperative-drain checkpoint, and undraining afterward — is deterministic machinery that folds in here as ordered helper invocations, because none of it carried a judgment argument that survives §6.3's reimplementation test.

Trigger (workflow-level): An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## What must become true here (durable outcome)

A drain is set (admission refused instantly, race closed by an explicit lock); the workflow waits, bounded, for in-scope workers to converge, counting a worker drained only when its exit is provable; each worker publishes its handoff and settles its lease before any termination; force-stop — refused unless a drain is already active, requiring explicit confirmation or dry-run, with exact identity displayed — is applied only to what cooperative draining left unresolved; and the drain is then lifted, idempotently, with mutually exclusive scopes.

## Behavior contract

- **Force-stopping workers is refused unless a cooperative drain is already active for the targeted scope, and it always requires explicit confirmation (--yes) or is limited to a --dry-run preview; it never runs automatically as a side effect of anything else.**
  (trigger: cooperative drain has failed to stop some workers within a bounded wait; outcome: a destructive force-stop only ever happens as an explicit, confirmed, drain-scoped operator action with full identity disclosed first)
  — `BU-P6-039`, `reference/sergeant-upstream/bin/sgt-drain-force` (L1-4, L45-46, L58-62)
- **sgt-drain-force must require an active drain and an explicit `--yes` (or offer `--dry-run`) before force-stopping any drain-eligible worker, and it must display the exact worker identity before stopping it, and it invokes a harness-specific backstop (e.g. a Claude background-session stop call) as part of the force-stop loop.**
  (trigger: cooperative drain fails to stop a worker and an operator must force-stop it; outcome: a destructive force-stop is never accidental: it requires both an active drain state and explicit operator confirmation, with the exact target identity shown first)
  — `BU-P7-083`, `reference/sergeant-upstream/tests/sgt-drain-force-test.sh` (line 2 and source-inventory description)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim. The helper invocations below run before and after this judgment: setting the drain and waiting for convergence come first; undraining comes last, once force-stop (if needed) is resolved.

## Helper invocations (folded stages, N1 adjudication A4)

Four stages extracted as their own candidates (ladder §6.5, "deterministic-machinery candidate") carried no "Additional note" arguing they survive §6.3's reimplementation test — swapping each operation's implementation would leave this stage's force-stop checkpoint unchanged. They fold in here as ordered helper invocations:

**1. set drain** (formerly `00-set-drain`) — admission is refused the instant the drain is set, scope global or per-project, race closed by an explicit lock.

- **Whether admission (new dispatch or relaunch) is currently allowed is decided purely by the presence of a drain file — global or project-scoped — and an empty or unparseable project name is treated as absent, checking only the global drain rather than erroring.**
  (trigger: a new dispatch or relaunch is about to happen; outcome: admission is blocked exactly when a global or a matching project drain exists, and never ambiguously blocked or admitted by a malformed project name)
  — `BU-P6-057`, `reference/sergeant-upstream/bin/_sgt-drain.sh` (L93-107)
- **A concurrent 'read drain state, then start new work' race is closed by an explicit admission lock that every dispatch/relaunch procedure and every drain-set/undrain procedure must acquire before reading or writing drain state, so a drain set mid-dispatch is never silently missed.**
  (trigger: a dispatch/relaunch and a drain-set could happen concurrently; outcome: admission decisions are always made against a consistent, lock-serialized view of drain state — never a stale read that lets new work slip through a just-activated drain)
  — `BU-P6-058`, `reference/sergeant-upstream/bin/_sgt-drain.sh` (L109-114)
- **A drain-lock acquisition failure that stems from the filesystem itself being unable to create hard links (e.g. FAT/exFAT, some CIFS/FUSE mounts) is distinguished from ordinary contention, because spinning to the deadline and reporting 'contended' would send an operator chasing a holder that does not exist.**
  (trigger: the lock filesystem does not support hard links; outcome: an environment-incompatibility failure is reported immediately and correctly, never masquerading as ordinary lock contention)
  — `BU-P6-062`, `reference/sergeant-upstream/bin/_sgt-drain.sh` (L458-467)
- **A drain refuses new worker starts within its scope while still storing incoming responses generation-safely for later delivery, --wait activates the drain and then waits for in-scope live workers to finish their current turn and exit, and on timeout it leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them.**
  (trigger: an operator wants to pause admission of new work, optionally waiting for a graceful stop; outcome: admission is refused immediately, in-flight responses are never lost, and a timed-out graceful wait never silently force-stops anything)
  — `BU-P8-077`, `reference/sergeant-upstream/docs/using-sergeant.md` (L231-243 (Pause admission with a drain))

**2. await convergence** (formerly `10-await-convergence`) — a bounded wait; a worker counts as drained only when its exit is provable; timeout leaves the drain active, exits non-zero, and names the unresolved.

- **A worker is only ever counted as having genuinely finished draining when its recorded process is provably gone; absence of recorded identity is explicitly not treated as proof of exit, so an unverifiable worker blocks a drain wait rather than being silently counted as resolved.**
  (trigger: a bounded drain wait is evaluating whether the scope is fully drained; outcome: a drain wait can never falsely report success because a worker's identity happened to be unrecordable)
  — `BU-P6-064`, `reference/sergeant-upstream/bin/sgt-drain` (L147-152)

**3. worker-side checkpoint** (formerly `20-worker-side-checkpoint`) — idempotent drain detection; publish handoff and settle the lease before terminating anything.

- **A cooperative drain of one worker publishes every durable fact it can before terminating anything — a handoff, settlement of the outstanding action lease, and the drained status — and only after everything durable is published does it begin terminating processes, because a drain must never be a way to discard unfinished work.**
  (trigger: an active global or project drain is detected while a worker is running; outcome: a drained worker's true, honest state (never a fabricated result) is fully durable before any process is terminated)
  — `BU-P6-111`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L219-234)
- **A cooperative drain must actually terminate the worker's entire process group — not merely the backgrounded watcher subshell that detects the drain signal — and it must publish its durable handoff and finalize the action lease BEFORE terminating, leaving no live execution context and no surviving process behind.**
  (trigger: a project or global drain signals a worker to stop cooperatively; outcome: a worker marked 'drained' is actually and fully stopped — no live execution context, no live agent process, no live background loop — with its handoff durably recorded first)
  — `BU-P7-084`, `reference/sergeant-upstream/tests/sgt-drain-terminate-test.sh` (lines 1-14)
- **A cooperative drain checkpoint inside the interactive worker must, on detecting drain, produce a clean exit with a `td` handoff written — including verifying the worktree it hands off from (per the same worktree-verification contract sgt-td-memory enforces elsewhere) — rather than exiting as if orphaned.**
  (trigger: a drained-status signal reaches a running interactive worker; outcome: a cooperatively drained worker leaves durable, worktree-verified recovery evidence behind, exactly like every other clean-exit path, rather than looking like an unexplained crash)
  — `BU-P7-107`, `reference/sergeant-upstream/tests/sgt-worker-drain-test.sh` (lines 20-25)
- **Cooperative drain detection inside the worker must be idempotent: an already-drained marker file present on disk must prevent a redundant re-drain, and it must distinguish global-drain, project-drain-match, project-drain-no-match, and no-drain-signal cases correctly, preserving all other worktree files across the drain transition.**
  (trigger: the worker evaluates whether it should cooperatively drain; outcome: drain detection is scoped precisely (global vs. this-project vs. other-project vs. none), preserves all worktree state, and never re-triggers drain handling once already drained)
  — `BU-P7-108`, `reference/sergeant-upstream/tests/sgt-drain-worker-test.sh` (lines 10-18)

**4. undrain** (formerly `40-undrain`) — undrain is idempotent, with mutually exclusive scopes. Runs after force-stop resolves (or is skipped, if cooperative draining already converged), lifting the drain this stage set.

- **Removing a drain is explicitly idempotent: undraining a scope that is not currently drained still exits successfully, and --global and a named project are mutually exclusive scopes that cannot both be targeted in one invocation.**
  (trigger: operator runs sgt-undrain for a project or --global; outcome: admission for the given scope is restored, or was already restored, with the same successful outcome either way)
  — `BU-P6-015`, `reference/sergeant-upstream/bin/sgt-undrain` (L8-9, L47)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
