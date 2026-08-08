#!/usr/bin/env bash
# Fake-CLI regression suite for the Claude background harness lifecycle
# (Claude Background Harness PRD, CH-4).  Every scenario in the PRD's
# Negative Test Matrix that can be exercised without a real API call is
# covered here, against a fake `claude` binary whose behavior is driven by
# env vars and small state files — no real network or subscription needed.
#
# What this file does NOT cover: CH-5 (the real-Claude contract test, which
# needs the genuine `stop`-causes-`attach`-to-exit behavior and is documented
# separately for manual/CI re-run before trusting a Claude Code version), and
# full tmux-based integration of every one of the eight termination paths
# across sgt-cleanup/sgt-watch/sgt-recover/sgt-respond/sgt-dispatch/
# sgt-validate/sgt-drain-force (tests/sgt-claude-stop-bg-session-test.sh
# covers the shared backstop helper's own correctness directly; a
# representative subset of the eight call sites — sgt-drain-force, sgt-watch,
# and both sgt-cleanup sites — are proven behaviorally in their own dedicated
# test files, per that file's header comment).

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v tmux >/dev/null 2>&1 || {
  printf 'sgt-claude-worker: skipped (tmux unavailable)\n'
  exit 0
}
command -v jq >/dev/null 2>&1 || {
  printf 'sgt-claude-worker: skipped (jq unavailable)\n'
  exit 0
}

TEST_ROOT="$(mktemp -d)"
TMUX_SESSION="sgt-claude-worker-test-$$"
export TMUX_TMPDIR="$TEST_ROOT/tmux"
mkdir -p "$TMUX_TMPDIR"
trap 'tmux kill-session -t "$TMUX_SESSION" 2>/dev/null || true; rm -rf "$TEST_ROOT"' EXIT

tmux new-session -d -s "$TMUX_SESSION" -n keepalive "while :; do sleep 1; done"

# _fake_claude <dir> <body>
# Writes a fake `claude` binary at <dir>/claude whose behavior for each
# subcommand is the caller-supplied case-statement body.  --help, stop, and
# respawn get sane defaults the body can override by matching first.
_fake_claude() {
  local dir="$1" body="$2"
  mkdir -p "$dir"
  {
    printf '#!/usr/bin/env bash\n'
    printf 'case "$1" in\n'
    printf '%s\n' "$body"
    printf '  --help) echo "--bg attach stop agents respawn"; exit 0 ;;\n'
    printf '  stop|respawn) exit 0 ;;\n'
    printf '  *) exit 1 ;;\n'
    printf 'esac\n'
  } > "$dir/claude"
  chmod +x "$dir/claude"
}

_launch() {
  local name="$1" fake_dir="$2"
  local state="$TEST_ROOT/$name/state" worktree="$TEST_ROOT/$name/worktree"
  mkdir -p "$state" "$worktree"
  printf 'in_progress\n' > "$worktree/.sergeant-status"
  printf 'launch-%s\n' "$name" > "$state/notification_id"
  cat > "$worktree/.sergeant-notification" <<EOF
notification_id=launch-$name
kind=initial
instruction=Read the .sergeant-brief.md file and execute the mission.
EOF
  tmux new-window -d -t "$TMUX_SESSION:" -n "$name" \
    "env SGT_NOTIFICATION_RETRY_INTERVAL=0.02 \
    SGT_CLAUDE_LIVENESS_INTERVAL=0.02 SGT_CLAUDE_LIVENESS_ATTEMPTS=10 \
    SGT_PROGRESS_INTERVAL=600 SGT_DRAIN_CHECK_INTERVAL=600 \
    SGT_TERMINATION_WATCHER_INTERVAL=0.05 \
    '$ROOT_DIR/bin/sgt-interactive-worker' '$state' '$worktree' '$fake_dir/claude'"
}

_field() { sed -n "s/^$2=//p" "$1"; }

# _force_kill_worker <state-dir> <window>
# `tmux kill-window` only removes the pane; it does not reliably terminate the
# worker's own background loops (_deliver_notifications, _watch_progress,
# _watch_drain, _watch_termination) or a fake harness's own infinite loop —
# measured directly: those processes were still running seconds after
# `tmux kill-window`, and stayed running until this whole suite's tmux
# SESSION was torn down at exit.  A case that force-ends a worker that never
# reaches its own natural exit (e.g. one built to hang deliberately) must kill
# the worker's recorded process GROUP directly, the same way sgt-cleanup and
# _drain_terminate do in production — otherwise every such case leaks
# processes that accumulate for the rest of the suite and degrade later
# cases' own timing.
_force_kill_worker() {
  local state="$1" window="$2" pgid
  pgid="$(cat "$state/worker_process_group" 2>/dev/null || true)"
  tmux kill-window -t "$TMUX_SESSION:$window" 2>/dev/null || true
  if [[ "$pgid" =~ ^[1-9][0-9]*$ ]]; then
    kill -TERM -- "-$pgid" 2>/dev/null || true
    sleep 0.1
    kill -KILL -- "-$pgid" 2>/dev/null || true
  fi
}

# ── 1. Happy path: --bg, agents --json liveness, then attach ─────────────────
# Normal launch: --bg returns a background id, the liveness check finds
# state=working, base_argv is repointed to `attach <id>`, and the mission
# completes normally through the attached call.

_fake_claude "$TEST_ROOT/happy" '
  --bg) echo "happy-bg-1"; exit 0 ;;
  agents) echo "[{\"id\":\"happy-bg-1\",\"state\":\"working\",\"sessionId\":\"happy-session-1\"}]"; exit 0 ;;
  attach)
    printf "done\n" > .sergeant-status
    printf "https://example.invalid/pr/1\n" > .sergeant-result
    exit 0
    ;;'
_launch happy "$TEST_ROOT/happy"
for _ in $(seq 1 500); do
  [[ -f "$TEST_ROOT/happy/state/result" ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/happy/state/status" 2>/dev/null || true)" == "done" ]] || {
  printf 'FAIL: happy-path claude worker did not reach done: %s\n' \
    "$(cat "$TEST_ROOT/happy/state/status" 2>/dev/null || true)" >&2
  cat "$TEST_ROOT/happy/state/diagnostic" 2>/dev/null >&2 || true
  exit 1
}
[[ "$(_field "$TEST_ROOT/happy/state/launch_record" base_argv)" == "attach happy-bg-1" ]] || {
  printf 'FAIL: base_argv was not repointed to attach: %q\n' \
    "$(_field "$TEST_ROOT/happy/state/launch_record" base_argv)" >&2
  exit 1
}
[[ -f "$TEST_ROOT/happy/state/claude_background_id" ]] || {
  printf 'FAIL: claude_background_id was not persisted\n' >&2
  exit 1
}
[[ "$(tr -d '\n' < "$TEST_ROOT/happy/state/claude_background_id")" == "happy-bg-1" ]]
[[ "$(tr -d '\n' < "$TEST_ROOT/happy/state/claude_session_id")" == "happy-session-1" ]]
tmux kill-window -t "$TMUX_SESSION:happy" 2>/dev/null || true

# ── 2. Launch failure: claude --bg returns non-zero ───────────────────────────
# Dispatch fails closed — no phantom Claude session is assumed.

_fake_claude "$TEST_ROOT/bg-fails" '
  --bg) exit 1 ;;'
_launch bg-fails "$TEST_ROOT/bg-fails"
for _ in $(seq 1 800); do
  [[ -s "$TEST_ROOT/bg-fails/state/status" ]] && \
    [[ "$(cat "$TEST_ROOT/bg-fails/state/status")" == failed:* ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/bg-fails/state/status" 2>/dev/null || true)" == failed:* ]] || {
  printf 'FAIL: a failed --bg launch did not fail closed: %s\n' \
    "$(cat "$TEST_ROOT/bg-fails/state/status" 2>/dev/null || true)" >&2
  exit 1
}
[[ ! -f "$TEST_ROOT/bg-fails/state/claude_background_id" ]] || {
  printf 'FAIL: a background id was recorded for a launch that never returned one\n' >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:bg-fails" 2>/dev/null || true

# ── 3. Launch failure: claude --bg returns no parseable background id ────────

_fake_claude "$TEST_ROOT/bg-empty" '
  --bg) echo ""; exit 0 ;;'
_launch bg-empty "$TEST_ROOT/bg-empty"
for _ in $(seq 1 800); do
  [[ -s "$TEST_ROOT/bg-empty/state/status" ]] && \
    [[ "$(cat "$TEST_ROOT/bg-empty/state/status")" == failed:* ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/bg-empty/state/status" 2>/dev/null || true)" == failed:* ]] || {
  printf 'FAIL: an empty --bg id did not fail closed: %s\n' \
    "$(cat "$TEST_ROOT/bg-empty/state/status" 2>/dev/null || true)" >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:bg-empty" 2>/dev/null || true

# ── 3b. Launch failure: claude --bg returns a non-empty but malformed id ─────
# Whitespace is stripped before the charset check runs, so a bare space would
# not exercise it; a leading dash is outside the id's own charset and survives
# stripping, so it does.

_fake_claude "$TEST_ROOT/bg-malformed" '
  --bg) echo "-bad/id"; exit 0 ;;'
_launch bg-malformed "$TEST_ROOT/bg-malformed"
for _ in $(seq 1 800); do
  [[ -s "$TEST_ROOT/bg-malformed/state/status" ]] && \
    [[ "$(cat "$TEST_ROOT/bg-malformed/state/status")" == failed:* ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/bg-malformed/state/status" 2>/dev/null || true)" == failed:* ]] || {
  printf 'FAIL: a malformed --bg id did not fail closed: %s\n' \
    "$(cat "$TEST_ROOT/bg-malformed/state/status" 2>/dev/null || true)" >&2
  exit 1
}
[[ ! -f "$TEST_ROOT/bg-malformed/state/claude_background_id" ]] || {
  printf 'FAIL: a malformed background id was persisted despite failing closed\n' >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:bg-malformed" 2>/dev/null || true

# ── 4. Model verification layer 2: bounded post-launch liveness catches a ────
#      shaped-but-invalid pin (state reaches "failed" within the bound)

_fake_claude "$TEST_ROOT/liveness-fail" '
  --bg) echo "livefail-bg-1"; exit 0 ;;
  agents) echo "[{\"id\":\"livefail-bg-1\",\"state\":\"failed\",\"sessionId\":\"\"}]"; exit 0 ;;
  attach) while :; do sleep 1; done ;;'
_launch liveness-fail "$TEST_ROOT/liveness-fail"
for _ in $(seq 1 800); do
  [[ -s "$TEST_ROOT/liveness-fail/state/status" ]] && \
    [[ "$(cat "$TEST_ROOT/liveness-fail/state/status")" == failed:* ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/liveness-fail/state/status" 2>/dev/null || true)" == failed:* ]] || {
  printf 'FAIL: a post-launch failed state was not caught within the bounded window: %s\n' \
    "$(cat "$TEST_ROOT/liveness-fail/state/status" 2>/dev/null || true)" >&2
  exit 1
}
grep -qF 'livefail-bg-1' "$TEST_ROOT/liveness-fail/state/status" || {
  printf 'FAIL: liveness failure diagnostic does not name the background id\n' >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:liveness-fail" 2>/dev/null || true

# ── 5. Termination watcher: done + non-empty result stops the session ────────
# The watcher must call `claude stop <id>` only once .sergeant-status is
# terminal AND .sergeant-result is non-empty — never on status=done alone.

_fake_claude "$TEST_ROOT/term-normal" '
  --bg) echo "term-bg-1"; exit 0 ;;
  agents) echo "[{\"id\":\"term-bg-1\",\"state\":\"working\",\"sessionId\":\"term-session-1\"}]"; exit 0 ;;
  stop) touch "'"$TEST_ROOT"'/term-normal/stop-called"; exit 0 ;;
  attach)
    sleep 0.3
    printf "https://example.invalid/pr/1\n" > .sergeant-result
    printf "done\n" > .sergeant-status
    # Real claude measured behavior: an external stop causes attach to exit on
    # its own.  This fake polls for the same stop-called marker its own "stop"
    # case writes, since two separate fake-claude invocations share no other
    # process state — without this, attach would block forever and the outer
    # loop in sgt-interactive-worker could never observe the terminal status.
    while [[ ! -e "'"$TEST_ROOT"'/term-normal/stop-called" ]]; do sleep 0.02; done
    exit 0
    ;;'
_launch term-normal "$TEST_ROOT/term-normal"
for _ in $(seq 1 500); do
  [[ -e "$TEST_ROOT/term-normal/stop-called" ]] && break
  sleep 0.02
done
[[ -e "$TEST_ROOT/term-normal/stop-called" ]] || {
  printf 'FAIL: termination watcher never called stop after done+result\n' >&2
  exit 1
}
# Let the fake attach's own stop-called poll observe the marker and exit
# before killing the window, so the worker's EXIT trap runs cleanly rather
# than being cut off mid-_finish.
for _ in $(seq 1 300); do
  tmux list-panes -t "$TMUX_SESSION:term-normal" >/dev/null 2>&1 || break
  sleep 0.02
done
tmux kill-window -t "$TMUX_SESSION:term-normal" 2>/dev/null || true

# ── 6. Termination watcher: done with EMPTY result does NOT stop the session ──
# Orphaned-result handling takes precedence — matches _finish's own existing
# empty-result-means-orphaned rule (templates/worker-brief.md:131-132 ordering).

_fake_claude "$TEST_ROOT/term-empty-result" '
  --bg) echo "term-bg-2"; exit 0 ;;
  agents) echo "[{\"id\":\"term-bg-2\",\"state\":\"working\",\"sessionId\":\"term-session-2\"}]"; exit 0 ;;
  stop) touch "'"$TEST_ROOT"'/term-empty-result/stop-called"; exit 0 ;;
  attach)
    sleep 0.3
    printf "done\n" > .sergeant-status
    while :; do sleep 1; done
    ;;'
_launch term-empty-result "$TEST_ROOT/term-empty-result"
sleep 1.5
[[ ! -e "$TEST_ROOT/term-empty-result/stop-called" ]] || {
  printf 'FAIL: termination watcher stopped the session on done with an EMPTY result\n' >&2
  exit 1
}
# This worker is designed to hang forever (status stays "done" with an empty
# result, so the termination watcher never fires and the fake attach never
# exits) — there is no natural exit to wait for.  Kill its process group
# directly; see _force_kill_worker.
_force_kill_worker "$TEST_ROOT/term-empty-result/state" term-empty-result

# ── 7. Unexpected death: session found stopped while status is non-terminal ──
# treated as a runtime failure, not mission completion: respawn + re-attach,
# no prompt re-delivered, no false "mission complete" interpretation.

_fake_claude "$TEST_ROOT/unexpected-death" '
  --bg) echo "death-bg-1"; exit 0 ;;
  agents)
    if [[ -e "'"$TEST_ROOT"'/unexpected-death/respawned" ]]; then
      echo "[{\"id\":\"death-bg-1\",\"state\":\"working\",\"sessionId\":\"death-session-1\"}]"
    else
      echo "[{\"id\":\"death-bg-1\",\"state\":\"stopped\",\"sessionId\":\"death-session-1\"}]"
    fi
    exit 0
    ;;
  respawn) touch "'"$TEST_ROOT"'/unexpected-death/respawned"; exit 0 ;;
  stop) touch "'"$TEST_ROOT"'/unexpected-death/stop-called"; exit 0 ;;
  attach)
    if [[ -e "'"$TEST_ROOT"'/unexpected-death/respawned" ]]; then
      printf "https://example.invalid/pr/1\n" > .sergeant-result
      printf "done\n" > .sergeant-status
      # Same stop-causes-exit simulation as case 5: the termination watcher
      # runs "$AGENT" stop as a separate fake-claude invocation, so this
      # attach polls for the marker that call writes instead of blocking
      # forever.
      while [[ ! -e "'"$TEST_ROOT"'/unexpected-death/stop-called" ]]; do sleep 0.02; done
      exit 0
    else
      # The unexpected death itself: attach exits with nothing written,
      # simulating the underlying runtime having already crashed.
      exit 0
    fi
    ;;'
_launch unexpected-death "$TEST_ROOT/unexpected-death"
for _ in $(seq 1 500); do
  [[ -e "$TEST_ROOT/unexpected-death/respawned" ]] && break
  sleep 0.02
done
[[ -e "$TEST_ROOT/unexpected-death/respawned" ]] || {
  printf 'FAIL: an unexpectedly stopped session was never respawned\n' >&2
  exit 1
}
for _ in $(seq 1 500); do
  [[ "$(cat "$TEST_ROOT/unexpected-death/state/status" 2>/dev/null || true)" == "done" ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/unexpected-death/state/status" 2>/dev/null || true)" == "done" ]] || {
  printf 'FAIL: worker did not reach done after respawn+reattach: %s\n' \
    "$(cat "$TEST_ROOT/unexpected-death/state/status" 2>/dev/null || true)" >&2
  exit 1
}

# ── 8. Identity persistence across respawn ────────────────────────────────────
# _record_worker_identity's persisted claude_background_id/claude_session_id
# remain valid and correctly associated with the same worker after respawn.

[[ "$(tr -d '\n' < "$TEST_ROOT/unexpected-death/state/claude_background_id")" == "death-bg-1" ]] || {
  printf 'FAIL: background id changed identity across respawn\n' >&2
  exit 1
}
for _ in $(seq 1 300); do
  tmux list-panes -t "$TMUX_SESSION:unexpected-death" >/dev/null 2>&1 || break
  sleep 0.02
done
tmux kill-window -t "$TMUX_SESSION:unexpected-death" 2>/dev/null || true

# ── 9. Unknown/unexpected agents --json state values fail closed, never crash ─

_fake_claude "$TEST_ROOT/unknown-state" '
  --bg) echo "unknown-bg-1"; exit 0 ;;
  agents) echo "[{\"id\":\"unknown-bg-1\",\"state\":\"totally-unrecognized-value\",\"sessionId\":\"x\"}]"; exit 0 ;;
  attach)
    printf "done\n" > .sergeant-status
    printf "https://example.invalid/pr/1\n" > .sergeant-result
    exit 0
    ;;'
_launch unknown-state "$TEST_ROOT/unknown-state"
for _ in $(seq 1 500); do
  [[ -f "$TEST_ROOT/unknown-state/state/result" ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/unknown-state/state/status" 2>/dev/null || true)" == "done" ]] || {
  printf 'FAIL: an unrecognized agents --json state value crashed or blocked the worker: %s\n' \
    "$(cat "$TEST_ROOT/unknown-state/state/status" 2>/dev/null || true)" >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:unknown-state" 2>/dev/null || true

# ── 10. Model pin, out-of-shape value: rejected before any --bg call ─────────
# A syntactically invalid pinned model fails via the pre-flight regex before
# claude --bg is ever invoked — no bg-launch marker file is created.

_fake_claude "$TEST_ROOT/preflight-reject" '
  --bg) touch "'"$TEST_ROOT"'/preflight-reject/bg-called"; echo "should-not-run"; exit 0 ;;'
mkdir -p "$TEST_ROOT/preflight-reject/state" "$TEST_ROOT/preflight-reject/worktree"
printf 'anthropic/not a valid model!!\n' > "$TEST_ROOT/preflight-reject/state/agent_model"
printf 'flag\n' > "$TEST_ROOT/preflight-reject/state/agent_model_source"
printf 'in_progress\n' > "$TEST_ROOT/preflight-reject/worktree/.sergeant-status"
tmux new-window -d -t "$TMUX_SESSION:" -n preflight-reject \
  "'$ROOT_DIR/bin/sgt-interactive-worker' '$TEST_ROOT/preflight-reject/state' \
  '$TEST_ROOT/preflight-reject/worktree' '$TEST_ROOT/preflight-reject/claude'"
for _ in $(seq 1 800); do
  [[ -s "$TEST_ROOT/preflight-reject/state/status" ]] && \
    [[ "$(cat "$TEST_ROOT/preflight-reject/state/status")" == failed:* ]] && break
  sleep 0.02
done
# This tuple is malformed against Sergeant's own pinned-tuple grammar
# (_SGT_AGENT_MODEL_RE), so it is rejected by _sgt_resolve_agent_launch itself,
# before sgt-interactive-worker's Claude-specific pre-flight regex ever runs —
# both are "reject before any --bg call" layers, and this proves the earlier one.
[[ "$(cat "$TEST_ROOT/preflight-reject/state/status" 2>/dev/null || true)" == failed:* ]] || {
  printf 'FAIL: a malformed pinned tuple was not rejected: %s\n' \
    "$(cat "$TEST_ROOT/preflight-reject/state/status" 2>/dev/null || true)" >&2
  exit 1
}
[[ ! -e "$TEST_ROOT/preflight-reject/bg-called" ]] || {
  printf 'FAIL: claude --bg was invoked despite a malformed pinned model\n' >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:preflight-reject" 2>/dev/null || true

# ── 10b. Model pin shaped enough for Sergeant's shared tuple grammar but not ──
#         for Claude's own pre-flight regex (Claude Harness Lifecycle layer 1)
# "not-a-claude-model" clears _SGT_AGENT_MODEL_RE (alnum/dot/dash/underscore)
# and the provider-scope check (anthropic), so it reaches sgt-interactive-
# worker's own _SGT_CLAUDE_MODEL_RE — which must still reject it, distinguishing
# this layer from case 10's rejection at the shared-grammar layer.
#
# Spawns the real worker, mirroring case 10's structure: the observable
# rejection (a failed status, no --bg call ever made) is what's asserted, not
# the regex text itself.

_fake_claude "$TEST_ROOT/claude-preflight-reject" '
  --bg) touch "'"$TEST_ROOT"'/claude-preflight-reject/bg-called"; echo "should-not-run"; exit 0 ;;'
mkdir -p "$TEST_ROOT/claude-preflight-reject/state" "$TEST_ROOT/claude-preflight-reject/worktree"
printf 'anthropic/not-a-claude-model\n' > "$TEST_ROOT/claude-preflight-reject/state/agent_model"
printf 'flag\n' > "$TEST_ROOT/claude-preflight-reject/state/agent_model_source"
printf 'in_progress\n' > "$TEST_ROOT/claude-preflight-reject/worktree/.sergeant-status"
tmux new-window -d -t "$TMUX_SESSION:" -n claude-preflight-reject \
  "'$ROOT_DIR/bin/sgt-interactive-worker' '$TEST_ROOT/claude-preflight-reject/state' \
  '$TEST_ROOT/claude-preflight-reject/worktree' '$TEST_ROOT/claude-preflight-reject/claude'"
for _ in $(seq 1 800); do
  [[ -s "$TEST_ROOT/claude-preflight-reject/state/status" ]] && \
    [[ "$(cat "$TEST_ROOT/claude-preflight-reject/state/status")" == failed:* ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/claude-preflight-reject/state/status" 2>/dev/null || true)" == failed:* ]] || {
  printf 'FAIL: a shaped-but-non-claude model was not rejected: %s\n' \
    "$(cat "$TEST_ROOT/claude-preflight-reject/state/status" 2>/dev/null || true)" >&2
  exit 1
}
[[ ! -e "$TEST_ROOT/claude-preflight-reject/bg-called" ]] || {
  printf 'FAIL: claude --bg was invoked despite a shaped-but-non-claude pinned model\n' >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:claude-preflight-reject" 2>/dev/null || true

# ── 11. Model pin, valid alias reaching the Claude-specific regex directly ────
# Exercise sgt-interactive-worker's OWN pre-flight regex (Claude Harness
# Lifecycle layer 1), not Sergeant's shared tuple grammar from case 10: a
# tuple shaped enough to clear _sgt_resolve_agent_launch (anthropic/sonnet)
# but whose model segment sgt-interactive-worker's own _SGT_CLAUDE_MODEL_RE
# would still need to accept.  Also proves a genuinely valid alias launches
# normally through this same code path.

_fake_claude "$TEST_ROOT/valid-alias" '
  --bg) echo "valid-bg-1"; exit 0 ;;
  agents) echo "[{\"id\":\"valid-bg-1\",\"state\":\"working\",\"sessionId\":\"valid-session-1\"}]"; exit 0 ;;
  attach)
    printf "done\n" > .sergeant-status
    printf "https://example.invalid/pr/1\n" > .sergeant-result
    exit 0
    ;;'
mkdir -p "$TEST_ROOT/valid-alias/state" "$TEST_ROOT/valid-alias/worktree"
printf 'anthropic/sonnet\n' > "$TEST_ROOT/valid-alias/state/agent_model"
printf 'flag\n' > "$TEST_ROOT/valid-alias/state/agent_model_source"
printf 'in_progress\n' > "$TEST_ROOT/valid-alias/worktree/.sergeant-status"
printf 'launch-valid-alias\n' > "$TEST_ROOT/valid-alias/state/notification_id"
cat > "$TEST_ROOT/valid-alias/worktree/.sergeant-notification" <<'EOF'
notification_id=launch-valid-alias
kind=initial
instruction=Read the .sergeant-brief.md file and execute the mission.
EOF
tmux new-window -d -t "$TMUX_SESSION:" -n valid-alias \
  "env SGT_NOTIFICATION_RETRY_INTERVAL=0.02 SGT_CLAUDE_LIVENESS_INTERVAL=0.02 \
  '$ROOT_DIR/bin/sgt-interactive-worker' '$TEST_ROOT/valid-alias/state' \
  '$TEST_ROOT/valid-alias/worktree' '$TEST_ROOT/valid-alias/claude'"
for _ in $(seq 1 500); do
  [[ -f "$TEST_ROOT/valid-alias/state/result" ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/valid-alias/state/status" 2>/dev/null || true)" == "done" ]] || {
  printf 'FAIL: a validly pinned alias did not launch and complete: %s\n' \
    "$(cat "$TEST_ROOT/valid-alias/state/status" 2>/dev/null || true)" >&2
  cat "$TEST_ROOT/valid-alias/state/diagnostic" 2>/dev/null >&2 || true
  exit 1
}
[[ "$(_field "$TEST_ROOT/valid-alias/state/launch_record" model_transport)" == "argv-bare" ]] || {
  printf 'FAIL: model_transport was not argv-bare: %q\n' \
    "$(_field "$TEST_ROOT/valid-alias/state/launch_record" model_transport)" >&2
  exit 1
}
tmux kill-window -t "$TMUX_SESSION:valid-alias" 2>/dev/null || true

# ── 12. Transcript substitution-warning scan ──────────────────────────────────
# A valid-but-unentitled model is caught only by scanning the transcript for
# the substitution-warning line, even though the mission completes
# successfully and agents --json reports a normal, non-failed state.

_fake_claude "$TEST_ROOT/substituted" '
  --bg) echo "subst-bg-1"; exit 0 ;;
  agents) echo "[{\"id\":\"subst-bg-1\",\"state\":\"working\",\"sessionId\":\"subst-session-1\",\"transcript\":\"Model \\\"opus\\\" is restricted by your organization'"'"'s settings. Using claude-sonnet-5 instead.\"}]"; exit 0 ;;
  attach)
    printf "done\n" > .sergeant-status
    printf "https://example.invalid/pr/1\n" > .sergeant-result
    exit 0
    ;;'
mkdir -p "$TEST_ROOT/substituted/state" "$TEST_ROOT/substituted/worktree"
printf 'anthropic/opus\n' > "$TEST_ROOT/substituted/state/agent_model"
printf 'flag\n' > "$TEST_ROOT/substituted/state/agent_model_source"
printf 'in_progress\n' > "$TEST_ROOT/substituted/worktree/.sergeant-status"
printf 'launch-substituted\n' > "$TEST_ROOT/substituted/state/notification_id"
cat > "$TEST_ROOT/substituted/worktree/.sergeant-notification" <<'EOF'
notification_id=launch-substituted
kind=initial
instruction=Read the .sergeant-brief.md file and execute the mission.
EOF
tmux new-window -d -t "$TMUX_SESSION:" -n substituted \
  "env SGT_NOTIFICATION_RETRY_INTERVAL=0.02 SGT_CLAUDE_LIVENESS_INTERVAL=0.02 \
  '$ROOT_DIR/bin/sgt-interactive-worker' '$TEST_ROOT/substituted/state' \
  '$TEST_ROOT/substituted/worktree' '$TEST_ROOT/substituted/claude'"
for _ in $(seq 1 500); do
  [[ -f "$TEST_ROOT/substituted/state/result" ]] && break
  sleep 0.02
done
[[ "$(cat "$TEST_ROOT/substituted/state/status" 2>/dev/null || true)" == "done" ]] || {
  printf 'FAIL: substitution-warning case did not complete the mission successfully: %s\n' \
    "$(cat "$TEST_ROOT/substituted/state/status" 2>/dev/null || true)" >&2
  exit 1
}
for _ in $(seq 1 800); do
  grep -qiF 'substitution warning' \
    "$TEST_ROOT/substituted/state/model_substitution_warning" 2>/dev/null && break
  sleep 0.02
done
grep -qiF 'substitution warning' \
  "$TEST_ROOT/substituted/state/model_substitution_warning" 2>/dev/null || {
  printf 'FAIL: substitution warning was not scanned/recorded despite a successful mission:\n%s\n' \
    "$(cat "$TEST_ROOT/substituted/state/model_substitution_warning" 2>/dev/null || true)" >&2
  exit 1
}
grep -qF 'subst-bg-1' "$TEST_ROOT/substituted/state/model_substitution_warning" || {
  printf 'FAIL: substitution warning does not name the background id\n' >&2
  exit 1
}

# ── 13. A message queued mid-turn is delivered after the current turn ────────
# NOT independently covered here or by tests/sgt-worker-handshake-test.sh: that
# file's fake harness body performs a sequential ack/accept/complete handshake
# with no simulated "busy/generating" state, so it does not actually model a
# message arriving while a turn is in progress.  What IS covered — by every
# other case in this file plus the handshake test's own claude fake — is that
# nothing Claude-specific was added to the notification loop itself, which is
# what makes reusing that loop unmodified a reasonable bet for this specific
# scenario too.  The scenario itself was only measured manually against a real
# claude session (docs/research/claude-background-harness-spike.md's "A
# message sent via tmux send-keys to an attached session..." finding); it is
# not exercised by an automated fake-CLI regression.

printf 'sgt-claude-worker: ok\n'
