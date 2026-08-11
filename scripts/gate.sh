#!/usr/bin/env bash
# Shipping-gate runner: ensures the no-mistakes daemon is alive with the
# environment this host needs, then starts the run.
#
# Portable across every measured host (docs/environments/) rather than
# hardcoded to the cloud container:
#   - repo root is resolved from this script's own location, not a fixed
#     checkout path (cloud container: /home/user/sergeant-rs; Cerberus:
#     /home/miztertea/sergeant-rs).
#   - no-mistakes's state dir is $HOME/.no-mistakes (root's is
#     /root/.no-mistakes; a normal user's is ~/.no-mistakes — same
#     layout, different HOME).
#   - IS_SANDBOX=1 is set only under root: it exists solely because
#     `claude --dangerously-skip-permissions` refuses to run under
#     root/sudo unless it's set (src/backend/claude.rs module docs); a
#     non-root host like Cerberus doesn't hit that refusal and shouldn't
#     opt in to it.
#   - how the daemon's env is actually set differs by supervision
#     mechanism (measured on Cerberus, 2026-08-11): the cloud container's
#     `no-mistakes daemon start` forks directly and inherits the invoking
#     shell's env, so prefixing the command works there. A host with a
#     systemd --user session has `daemon start` install/refresh a
#     systemd unit that bakes in only HOME and PATH — an inline env
#     prefix on `daemon start` is silently discarded. Detect which
#     applies and, when a unit exists, route env through `systemctl
#     --user set-environment` + restart instead.
#
# Usage: scripts/gate.sh "<intent>" [extra axi run flags...]
set -euo pipefail
intent="$1"; shift

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NO_MISTAKES_HOME="${NO_MISTAKES_HOME:-$HOME/.no-mistakes}"

[ -z "$(git -C "$REPO_ROOT" status --porcelain)" ] || { echo "gate.sh: tree not clean - commit first" >&2; exit 1; }

# CARGO_TARGET_DIR shared with the main checkout: the pipeline's disposable
# worktree reuses dependency artifacts (duckdb's ~2 GB rlib included) instead
# of a multi-GB cold build per run — this container hit ENOSPC during M5.
export CARGO_TARGET_DIR="$NO_MISTAKES_HOME/cargo-target"  # pipeline-private persistent cache: warm across ITS runs, isolated from the main checkout (shared-with-main caused binary cross-contamination - worktree-baked env! paths in reused test binaries, diagnosed 2026-08-09)

is_root() { [ "$(id -u)" = "0" ]; }

# Empty stdout when systemctl is absent or no unit is installed yet — never
# lets the probe itself trip `set -e`/`pipefail`.
systemd_unit() {
  command -v systemctl >/dev/null 2>&1 || return 0
  systemctl --user list-units --all --no-legend --plain 'no-mistakes-daemon-*.service' 2>/dev/null \
    | awk '{print $1}' | head -n1
  return 0
}

wait_for_pid_file() {
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    [ -f "$NO_MISTAKES_HOME/daemon.pid" ] && return 0
    sleep 0.5
  done
  echo "gate.sh: no-mistakes daemon.pid never appeared after daemon start" >&2
  exit 1
}

start_daemon() {
  # A stale daemon.pid from a previous daemon would satisfy
  # wait_for_pid_file before the new daemon writes its own; the freshly
  # written file is the readiness signal, so clear the old one first.
  rm -f "$NO_MISTAKES_HOME/daemon.pid"
  if is_root; then
    IS_SANDBOX=1 CARGO_TARGET_DIR="$CARGO_TARGET_DIR" no-mistakes daemon start
  else
    CARGO_TARGET_DIR="$CARGO_TARGET_DIR" no-mistakes daemon start
  fi
  local unit
  unit="$(systemd_unit)"
  if [ -n "$unit" ]; then
    # systemd-managed: `daemon start` only baked HOME/PATH into the unit.
    # Route the rest through the user manager's environment block instead.
    if is_root; then
      systemctl --user set-environment CARGO_TARGET_DIR="$CARGO_TARGET_DIR" IS_SANDBOX=1
    else
      systemctl --user set-environment CARGO_TARGET_DIR="$CARGO_TARGET_DIR"
    fi
    rm -f "$NO_MISTAKES_HOME/daemon.pid"
    systemctl --user restart "$unit"
  fi
  wait_for_pid_file
}

daemon_env_ok() {
  local pid="$1"
  grep -qz CARGO_TARGET_DIR "/proc/$pid/environ" || return 1
  if is_root; then
    grep -qz IS_SANDBOX "/proc/$pid/environ" || return 1
  fi
  return 0
}

# Empty stdout (rather than a pipefail abort) when the pid file is missing,
# empty, or not in the expected shape.
daemon_pid() {
  local pid_file="$NO_MISTAKES_HOME/daemon.pid"
  [ -s "$pid_file" ] || return 0
  { grep -oE '"pid":[0-9]+' "$pid_file" 2>/dev/null || true; } | cut -d: -f2 | head -n1
  return 0
}

no-mistakes daemon status >/dev/null 2>&1 || start_daemon
pid="$(daemon_pid)"
if [ -z "$pid" ] || ! daemon_env_ok "$pid"; then
  no-mistakes daemon stop
  start_daemon
  pid="$(daemon_pid)"
  [ -n "$pid" ] || { echo "gate.sh: no readable pid in $NO_MISTAKES_HOME/daemon.pid after restart" >&2; exit 1; }
  daemon_env_ok "$pid" || {
    wanted="CARGO_TARGET_DIR"
    is_root && wanted="CARGO_TARGET_DIR and IS_SANDBOX"
    echo "gate.sh: daemon (pid $pid) still missing $wanted in its env after restart - check how its supervisor sets env before running the gate" >&2
    exit 1
  }
fi

exec no-mistakes axi run --intent "$intent" --skip push,pr,ci "$@"
