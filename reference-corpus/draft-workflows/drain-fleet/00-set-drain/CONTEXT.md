# 00-set-drain: set drain

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

Admission is refused the instant the drain is set, scope global or per-project, race closed by an explicit lock.

Trigger (workflow-level): An operator needs to freeze new stage/turn admission — globally or for one project — before a disruptive operation.

## What must become true here (durable outcome)

Admission is refused the instant the drain is set, scope global or per-project, race closed by an explicit lock.

## Behavior contract

- **Whether admission (new dispatch or relaunch) is currently allowed is decided purely by the presence of a drain file — global or project-scoped — and an empty or unparseable project name is treated as absent, checking only the global drain rather than erroring.**
  (trigger: a new dispatch or relaunch is about to happen; outcome: admission is blocked exactly when a global or a matching project drain exists, and never ambiguously blocked or admitted by a malformed project name)
  — `BU-P6-057`, `reference/sergeant-upstream/bin/_sgt-drain.sh` (L93-107)
- **A concurrent 'read drain state, then start a new pane' race is closed by an explicit admission lock that every dispatch/relaunch path (sgt-dispatch, sgt-respond) and every drain-set/undrain path (sgt-drain) must acquire before reading or writing drain state, so a drain set mid-dispatch is never silently missed.**
  (trigger: a dispatch/relaunch and a drain-set could happen concurrently; outcome: admission decisions are always made against a consistent, lock-serialized view of drain state — never a stale read that lets new work slip through a just-activated drain)
  — `BU-P6-058`, `reference/sergeant-upstream/bin/_sgt-drain.sh` (L109-114)
- **A drain-lock acquisition failure that stems from the filesystem itself being unable to create hard links (e.g. FAT/exFAT, some CIFS/FUSE mounts) is distinguished from ordinary contention, because spinning to the deadline and reporting 'contended' would send an operator chasing a holder that does not exist.**
  (trigger: the lock filesystem does not support hard links; outcome: an environment-incompatibility failure is reported immediately and correctly, never masquerading as ordinary lock contention)
  — `BU-P6-062`, `reference/sergeant-upstream/bin/_sgt-drain.sh` (L458-467)
- **A drain refuses new worker starts within its scope while still storing incoming responses generation-safely for later delivery, --wait activates the drain and then waits for in-scope live workers to finish their current turn and exit, and on timeout it leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them.**
  (trigger: an operator wants to pause admission of new work, optionally waiting for a graceful stop; outcome: admission is refused immediately, in-flight responses are never lost, and a timed-out graceful wait never silently force-stops anything)
  — `BU-P8-077`, `reference/sergeant-upstream/docs/using-sergeant.md` (L231-243 (Pause admission with a drain))

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P6-058` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
