#!/usr/bin/env bash
# Real-Claude contract test (Claude Background Harness PRD, CH-5).
#
# Reproduces the original defect end to end against the GENUINE `claude` CLI —
# not a fake — and proves the fix holds: a mission written, `.sergeant-status=done`
# set after a non-empty `.sergeant-result`, the worker exits unattended, and no
# live `claude` process or monitoring loop remains.
#
# This is NOT part of the fake-CLI suite (tests/sgt-claude-worker-test.sh) and is
# never wired into CI: it needs a real Claude Code install, a real account with
# the bypass-permissions disclaimer already accepted on this machine
# (`claude --dangerously-skip-permissions`, run interactively once — PRD CH-7),
# and it consumes a real, small amount of API usage. It is meant to be re-run by
# hand — or by a deliberately scheduled job — against every future Claude Code
# version bump before that version is trusted in production, since the whole
# fix depends on the measured `stop`-causes-`attach`-to-exit behavior the
# research spike recorded for Claude Code 2.1.220 continuing to hold.
#
# Usage: SGT_REAL_CLAUDE_CONTRACT_TEST=1 bash tests/sgt-claude-real-contract-test.sh

set -euo pipefail

if [[ "${SGT_REAL_CLAUDE_CONTRACT_TEST:-0}" != "1" ]]; then
  printf 'sgt-claude-real-contract: skipped (set SGT_REAL_CLAUDE_CONTRACT_TEST=1 to run against a real claude install)\n'
  exit 0
fi

command -v claude >/dev/null 2>&1 || {
  printf 'sgt-claude-real-contract: skipped (claude not installed)\n'
  exit 0
}
command -v tmux >/dev/null 2>&1 || {
  printf 'sgt-claude-real-contract: skipped (tmux unavailable)\n'
  exit 0
}
command -v jq >/dev/null 2>&1 || {
  printf 'sgt-claude-real-contract: skipped (jq unavailable)\n'
  exit 0
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_ROOT="$(mktemp -d)"
TMUX_SESSION="sgt-claude-real-contract-$$"
export TMUX_TMPDIR="$TEST_ROOT/tmux"
mkdir -p "$TMUX_TMPDIR"
trap 'tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true; rm -rf "$TEST_ROOT"' EXIT

STATE="$TEST_ROOT/state"
WORKTREE="$TEST_ROOT/worktree"
mkdir -p "$STATE" "$WORKTREE"

# ── 1. Capability gate (PRD Compatibility and Rollout) ────────────────────────
_sgt_help="$(claude --help 2>&1 || true)"
for _sgt_cap in --bg attach stop agents respawn; do
  printf '%s\n' "$_sgt_help" | grep -qF -- "$_sgt_cap" || {
    printf 'sgt-claude-real-contract: skipped (installed claude is missing required capability: %s)\n' \
      "$_sgt_cap"
    exit 0
  }
done

# ── 2. Write a mission the worker can complete deterministically without ─────
#      relying on model judgement: publish the result, then done, exactly in
#      the order templates/worker-brief.md:131-132 mandates, from a trivial,
#      unambiguous instruction.
printf 'in_progress\n' > "$WORKTREE/.sergeant-status"
printf 'launch-real-contract\n' > "$STATE/notification_id"
cat > "$WORKTREE/.sergeant-notification" <<'EOF'
notification_id=launch-real-contract
kind=initial
instruction=This is a Sergeant contract-test mission with no real work to do. Immediately run exactly these two shell commands in order and nothing else: `printf 'contract-test-ok\n' > .sergeant-result` then `printf 'done\n' > .sergeant-status`. Do not ask for confirmation, do not explore the repository, do not write anything else.
EOF

tmux new-session -d -s "$TMUX_SESSION" -n keepalive "while :; do sleep 1; done"
tmux new-window -d -t "$TMUX_SESSION:" -n worker \
  "'$ROOT_DIR/bin/sgt-interactive-worker' '$STATE' '$WORKTREE' claude"

# ── 3. Wait for the mission to genuinely complete ────────────────────────────
_sgt_deadline_attempts=1800  # up to 3 minutes at 0.1s/attempt for a real model turn
for _ in $(seq 1 "$_sgt_deadline_attempts"); do
  [[ -s "$WORKTREE/.sergeant-result" && "$(cat "$WORKTREE/.sergeant-status" 2>/dev/null || true)" == "done" ]] && break
  sleep 0.1
done
if [[ ! -s "$WORKTREE/.sergeant-result" || "$(cat "$WORKTREE/.sergeant-status" 2>/dev/null || true)" != "done" ]]; then
  printf 'FAIL: real Claude mission did not reach done with a result within the bound\n' >&2
  printf 'status=%s diagnostic=%s\n' \
    "$(cat "$WORKTREE/.sergeant-status" 2>/dev/null || true)" \
    "$(cat "$STATE/diagnostic" 2>/dev/null || true)" >&2
  exit 1
fi

# ── 4. CH-1: the worker process exits within the termination watcher's poll ──
#      interval, with no manual intervention.
_sgt_pid="$(cat "$STATE/worker_pid" 2>/dev/null || true)"
[[ "$_sgt_pid" =~ ^[1-9][0-9]*$ ]] || {
  printf 'FAIL: no worker_pid was recorded\n' >&2
  exit 1
}
_sgt_exited=false
for _ in $(seq 1 200); do  # up to 20s: the watcher's own default poll interval is 2s
  kill -0 "$_sgt_pid" 2>/dev/null || { _sgt_exited=true; break; }
  sleep 0.1
done
"$_sgt_exited" || {
  printf 'FAIL: the worker process (pid %s) is still alive after the mission completed — this is the original defect (GH: unattended claude worker never exits)\n' \
    "$_sgt_pid" >&2
  exit 1
}

# ── 5. No live claude process remains for this session ───────────────────────
_sgt_bg_id="$(cat "$STATE/claude_background_id" 2>/dev/null || true)"
if [[ -n "$_sgt_bg_id" ]]; then
  _sgt_state="$(claude agents --json 2>/dev/null | \
    jq -r ".[] | select(.id == \"$_sgt_bg_id\") | .state" 2>/dev/null || true)"
  [[ "$_sgt_state" != "working" && "$_sgt_state" != "blocked" ]] || {
    printf 'FAIL: claude background session %s is still %s after the mission completed\n' \
      "$_sgt_bg_id" "$_sgt_state" >&2
    exit 1
  }
fi

printf 'sgt-claude-real-contract: ok (worker pid %s exited unattended; background session %s stopped; result=%s)\n' \
  "$_sgt_pid" "${_sgt_bg_id:-none}" "$(cat "$WORKTREE/.sergeant-result")"
