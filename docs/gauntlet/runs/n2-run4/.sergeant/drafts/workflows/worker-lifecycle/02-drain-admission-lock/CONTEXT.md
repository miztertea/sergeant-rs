# 02-drain-admission-lock

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** dispatch or respond needs to take the drain admission lock

**Outcome:** locking either succeeds via hard link or the operation fails closed — it never proceeds without the lock

**Statement (the operative rule):** Drain admission locking uses an atomic hard link rather than requiring `flock`; the drain state directory must be writable by the invoking user and on a filesystem that supports hard links, otherwise dispatch and respond fail closed rather than proceeding unlocked.

## What must become true here (durable outcome)

Locking either succeeds via hard link or the operation fails closed — it never proceeds without the lock — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0521`: A project name that would collide with the admission-lock's own artifact filename is refused outright, so no drain or undrain can ever target the live lock.
- `BU-0528`: The drain step --status, when scanning all project drains, explicitly excludes lock records and their staging/quarantine/temp-file artifacts from being reported as active drains — reporting them would invent drains nobody set, and the obvious operator response (the drain step) would delete a lock a live dispatch is holding.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0543`: The drain admission lock uses a hard link, not flock(1): flock is absent from macOS system installs and limited on BusyBox, and guarding it with `command -v flock` previously meant the whole lock silently degraded to a no-op; link(2) fails atomically when the target already exists and is available everywhere Sergeant runs.
- `BU-0544`: The lock's owner record is written and fully staged before it becomes the lock (via link), so the lock can never exist in an unattributable state that no later contender is able to reclaim.
- `BU-0545`: Every lock instance carries a nonce, and both reclamation and release are bound to that exact nonce, so a contender can neither destroy a lock that was reacquired while it was deciding to reclaim it, nor release a lock it no longer owns.
- `BU-0546`: No EXIT trap is installed by this locking library on purpose, because release is explicit rather than kernel-backed and the library is sourced by scripts that own their own traps; a holder killed mid-flight simply leaves its record behind, which is safe because the record always names a verifiable owner the next contender can reclaim from.
- `BU-0547`: Process liveness is checked via /proc when available (authoritative on Linux and BusyBox), because `kill -0` alone is unreliable — it also fails with EPERM for a live process owned by another user; without /proc and without a usable `ps`, liveness is reported as undeterminable rather than assumed dead.
- `BU-0548`: Callers of the process-liveness check must treat an undeterminable (return code 2) result as 'still alive', so an unverifiable lock/worker owner is never displaced by a false claim of death.
- `BU-0549`: PID-reuse detection prefers reading a process's start time from /proc/<pid>/stat field 22 (Linux/BusyBox) and falls back to `ps -o lstart=` (macOS) when /proc is unavailable.
- `BU-0550`: Every value written into a lock record is sanitized — newlines and carriage returns stripped, truncated to 256 characters — before being written, because a newline embedded in USER, in `uname -n`, or in a process-start token could otherwise inject additional record fields; specifically, an injected owner_nonce read first by `grep -m1` would prevent the true owner from ever releasing its own lock.
- `BU-0551`: A lock record is read in one snapshot rather than field-at-a-time, because a competing reclaimer renaming the record mid-check could otherwise hand back fields from different record generations — including an empty nonce, which previously made reclamation unbounded.
- `BU-0552`: An unidentified host is never treated as matching another unidentified host when deciding whether a lock's recorded owner is running on this machine.
- `BU-0553`: A lock record with no nonce is never eligible for reclamation, because treating it as reclaimable would let a contender delete a lock that was legitimately acquired during the very race being adjudicated.
- `BU-0554`: Reclaiming a lock proven stale renames the record first (an atomic move, so at most one contender can move it), then re-checks the observed nonce against the one that justified the reclaim; if a different lock instance is found — meaning the lock was legitimately reacquired between the staleness decision and the rename — the quarantine copy is restored via `ln` (which refuses to clobber) instead of being discarded.
- `BU-0555`: When restoring a wrongly-reclaimed lock record fails and no other copy of it has appeared, the quarantine copy is deliberately preserved (not deleted) and an error is printed, because it is the only remaining copy of a live lock record and must survive for the next contender to find.
- `BU-0556`: Leftover quarantine and staging lock artifacts are swept only when the process id embedded in their own filename is provably gone, so a concurrent reclamation or acquisition already in progress is never disturbed by the sweep.
- `BU-0557`: A drain-admission-lock acquisition timeout reports the current holder's pid, user, host, purpose, and age, plus explicit recovery guidance (retry; a dead owner's lock is reclaimed automatically), rather than an undiagnosable generic failure.
- `BU-0558`: `ln` failing while the lock's target path does not exist is treated as a filesystem incapable of hard links (e.g. FAT/exFAT, some CIFS/FUSE mounts) rather than as contention, and fails immediately rather than spinning to the timeout deadline and reporting a nonexistent holder.
- `BU-0559`: Releasing a drain-admission lock re-verifies the on-disk owner_nonce first, so release can never remove a lock that now belongs to a different process — for example after an operator manually removed the record and someone else has since acquired it; when the nonce no longer matches, an error is printed and the record is deliberately left in place.
- `BU-0560`: When the drain admission lock cannot be acquired, the wrapped command is never invoked at all — the lock-acquisition outcome (timeout or unavailable) is returned in its place, and an empty invocation (no command given) is itself refused rather than silently reported as success.
- `BU-0561`: Because a wrapped command may itself exit with status 2 or 3 (the same numeric sentinels the lock wrapper uses for timeout/unavailable), SGT_DRAIN_LOCK_STATE — not the numeric return code alone — is the authoritative signal a caller must use to distinguish a lock-acquisition failure from the wrapped command's own exit status.

