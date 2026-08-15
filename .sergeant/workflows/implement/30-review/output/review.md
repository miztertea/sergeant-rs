# Review output — 30-review

Work: WA · gate-script portability (#130)
Branch: sergeant/01M03JS0FTRMWVMTVR99FJ6ZDM
Date: 2026-08-15

## Change reviewed

`scripts/gate.sh` — `daemon_env_ok()` narrowed to skip the `/proc/<pid>/environ` check on the direct-fork (non-systemd) path on hosts where `/proc` is not available (macOS), while keeping it as a hard requirement on Linux (where `/proc` is authoritative) and failing with a clear diagnostic on any host where systemd manages the daemon without `/proc` being available.

Diff base: `4e7e21b` (the implementing commit, "scripts/gate.sh: narrow daemon_env_ok() to skip /proc check on direct-fork path")

## Review method

`/code-review --effort high` — synthesized from multiple review angles (correctness A–C, cleanup/efficiency/altitude D–H).

## Findings

### Finding 1 — CONFIRMED, FIXED (correctness, HIGH)
**File:** `scripts/gate.sh`, line 95 (original implementation commit)
**Summary:** `daemon_env_ok()` was fail-open on Linux when the daemon died in the narrow window between `daemon_pid()` reading the PID file and the `[ -f "/proc/$pid/environ" ]` test.

**Failure scenario:** On Linux, `/proc/<pid>/environ` disappears atomically when the process exits. With the original fix, a dead daemon's absent `/proc` entry fell through to the macOS path: `systemd_unit()` returned empty on a direct-fork host → `return 0`. The old code's unconditional `grep` on the missing file returned 1, which was fail-closed (correct). The new code's structural change inadvertently changed this to fail-open on Linux.

**Fix applied (this review stage):** Added `if [ -d /proc ]; then return 1; fi` immediately after the `[ -f "/proc/$pid/environ" ]` check fails. On Linux, `/proc` is always a mounted filesystem; its presence means the OS supports `/proc/<pid>/environ` and the PID's absence from it means the daemon is dead → caller's restart block fires. On macOS, `/proc` does not exist at all → `[ -d /proc ]` is false → the code falls through to the systemd/direct-fork logic as intended.

**Verified:** under `/bin/bash` (3.2.57, arm64-macOS) — macOS direct-fork path returns 0; `/proc` not present confirms the guard does not fire on this host.

### Findings 2 & 3 — OUT OF SCOPE
`scripts/perf/common.sh:56,130` — `|| true` missing on perl-clock fallback assignments. Not in WA's file scope (`scripts/gate.sh` only). Filed for separate attention.

### Finding 4 — OUT OF SCOPE
`src/domain/workspace.rs:1776` — vacuous `assert_eq!(None, None)` in workspace test. Not in WA's file scope.

## Disposition

One in-scope finding confirmed and fixed in this review stage. No remaining open findings in `scripts/gate.sh`. The change is correct, bash-3.2-clean (`/bin/bash -n` passes), and resolves #130's hard-block on macOS without weakening the guard's protection on Linux or systemd hosts.

Commit trailer: `Fixes #130` — justified: the check now correctly handles all three cases (Linux live daemon, Linux dead daemon, macOS direct-fork). The "no platform-independent verification on macOS+systemd" path remains, but that is a host that does not currently exist in this repo's measured environment set, and the failure there is an explicit diagnostic, not a silent pass.
