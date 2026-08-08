#!/usr/bin/env bash
# Regression for the shared Claude session helpers in bin/_sgt-drain.sh:
# _sgt_claude_stop_bg_session (the backstop every one of the Claude Background
# Harness PRD's eight independent termination paths calls) and
# _sgt_claude_bg_session_is_live (the liveness check sgt-recover's stall-
# recovery relaunch and sgt-respond's superseded-worker relaunch both use
# before dispatching a replacement worker, PRD CH-8).  Three things matter
# here: each helper is correct on its own (idempotent, resolves the binary
# through fleet state's own `agent` field rather than a hardcoded literal
# `claude`, never fails, and — for the liveness check — true only for a
# genuinely working/blocked session).
#
# A representative subset of the eight termination call sites named in the
# PRD's Recovery Semantics section is proven behaviorally — a real invocation
# of the actual script, with a fake `claude` binary observing the genuine
# `stop <id>` call — rather than by a source-text grep for the call site,
# which would pass just as well for a dead, commented-out, or reordered call:
#   * bin/sgt-drain-force's force-stop loop (tests/sgt-drain-force-test.sh)
#   * bin/sgt-watch's _recycle_terminal_worker (tests/sgt-watch-recycle-test.sh)
#   * bin/sgt-cleanup's _stop_local_worker and _stop_validation_pane
#     (tests/sgt-cleanup-test.sh, Issue #21 remain-on-exit fixture)
# Full behavioral coverage of the remaining sites (sgt-recover, sgt-respond,
# sgt-dispatch, sgt-validate, sgt-interactive-worker's _drain_terminate) is
# not attempted here: each additionally requires standing up that script's own
# relaunch/dispatch/validation preconditions (response-lock state, td state,
# a live worktree, etc.) for this one narrow property, which is prohibitively
# expensive relative to what it would additionally prove beyond what the
# representative subset above already demonstrates about the shared helper
# being reachable and correct at real call sites.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

# shellcheck source=bin/_sgt-drain.sh
source "$ROOT_DIR/bin/_sgt-drain.sh"

# ── 1. No-op when claude_background_id is absent ──────────────────────────────

repo_dir="$TEST_ROOT/no-id"
mkdir -p "$repo_dir"
_sgt_claude_stop_bg_session "$repo_dir" || {
  printf 'FAIL: the helper returned non-zero for an absent background id\n' >&2
  exit 1
}

# ── 2. No-op when the resolved binary does not exist ──────────────────────────

repo_dir="$TEST_ROOT/no-binary"
mkdir -p "$repo_dir"
printf 'some-bg-id\n' > "$repo_dir/claude_background_id"
printf '%s\n' "$TEST_ROOT/no-binary/definitely-not-a-real-binary" > "$repo_dir/agent"
_sgt_claude_stop_bg_session "$repo_dir" || {
  printf 'FAIL: the helper returned non-zero for an unresolvable binary\n' >&2
  exit 1
}

# ── 3. Calls "<agent> stop <id>", resolved from fleet state's own agent field ─
# (not a hardcoded literal `claude`), and is idempotent across repeats.

repo_dir="$TEST_ROOT/happy"
mkdir -p "$repo_dir" "$TEST_ROOT/fake-bin"
cat > "$TEST_ROOT/fake-bin/claude" <<EOF
#!/usr/bin/env bash
if [[ "\$1" == "stop" ]]; then
  printf '%s\n' "\$2" >> "$TEST_ROOT/stop-calls.log"
  exit 0
fi
exit 1
EOF
chmod +x "$TEST_ROOT/fake-bin/claude"
printf 'happy-bg-id\n' > "$repo_dir/claude_background_id"
printf '%s\n' "$TEST_ROOT/fake-bin/claude" > "$repo_dir/agent"

_sgt_claude_stop_bg_session "$repo_dir"
[[ -f "$TEST_ROOT/stop-calls.log" ]] || {
  printf 'FAIL: the resolved agent binary was never called\n' >&2
  exit 1
}
[[ "$(cat "$TEST_ROOT/stop-calls.log")" == "happy-bg-id" ]] || {
  printf 'FAIL: stop was called with the wrong background id: %q\n' \
    "$(cat "$TEST_ROOT/stop-calls.log")" >&2
  exit 1
}

# Repeat: idempotent — a second call is a no-op success, not an error.
_sgt_claude_stop_bg_session "$repo_dir"
[[ "$(wc -l < "$TEST_ROOT/stop-calls.log" | tr -d ' ')" == "2" ]] || {
  printf 'FAIL: a repeat stop was not observed as an idempotent no-op call\n' >&2
  exit 1
}

# ── 4. Falls back to a bare "claude" lookup when fleet state predates the ────
#      recorded agent field (no repo_dir/agent file at all)

repo_dir="$TEST_ROOT/legacy-fallback"
mkdir -p "$repo_dir"
printf 'legacy-bg-id\n' > "$repo_dir/claude_background_id"
# No repo_dir/agent file: the helper must fall back to a bare "claude" lookup
# on PATH rather than fail.  Point PATH at the fake bin above so the fallback
# resolves to something and the call is observable without depending on a
# real claude install existing on this machine.
PATH="$TEST_ROOT/fake-bin:$PATH" _sgt_claude_stop_bg_session "$repo_dir"
[[ "$(wc -l < "$TEST_ROOT/stop-calls.log" | tr -d ' ')" == "3" ]] || {
  printf 'FAIL: the legacy fallback (no recorded agent field) did not resolve a binary and call stop\n' >&2
  exit 1
}
[[ "$(tail -1 "$TEST_ROOT/stop-calls.log")" == "legacy-bg-id" ]] || {
  printf 'FAIL: the legacy fallback called stop with the wrong background id\n' >&2
  exit 1
}

# ── 5. Rejects a background id containing shell metacharacters ───────────────
# The recorded id is validated against an alphanumeric-safe charset before
# ever reaching a command line.

repo_dir="$TEST_ROOT/malformed-id"
mkdir -p "$repo_dir"
printf '%s\n' 'bg;rm -rf /' > "$repo_dir/claude_background_id"
printf '%s\n' "$TEST_ROOT/fake-bin/claude" > "$repo_dir/agent"
before_lines="$(wc -l < "$TEST_ROOT/stop-calls.log" | tr -d ' ')"
_sgt_claude_stop_bg_session "$repo_dir" || {
  printf 'FAIL: the helper returned non-zero for a malformed background id\n' >&2
  exit 1
}
after_lines="$(wc -l < "$TEST_ROOT/stop-calls.log" | tr -d ' ')"
[[ "$before_lines" == "$after_lines" ]] || {
  printf 'FAIL: a background id containing shell metacharacters reached the stop call\n' >&2
  exit 1
}

# ── 5b. _sgt_claude_bg_session_is_live: true only for working/blocked ────────
# The core state-determination logic sgt-recover's stall-recovery relaunch and
# sgt-respond's superseded-worker relaunch both depend on before dispatching a
# replacement worker (PRD CH-8 / Recovery Semantics).  Full tmux-based
# integration of the relaunch flows themselves is out of scope for this file
# (same cost-vs-value tradeoff as the structural check below); this proves the
# helper's own decision is correct for every state value that matters.

command -v jq >/dev/null 2>&1 || {
  printf 'sgt-claude-stop-bg-session: skipped remaining cases (jq unavailable)\n'
  exit 0
}

_fake_claude_reporting_state() {
  local dir="$1" state="$2"
  mkdir -p "$dir"
  cat > "$dir/claude" <<EOF
#!/usr/bin/env bash
if [[ "\$1" == "agents" ]]; then
  echo '[{"id":"live-check-bg","state":"$state","sessionId":"live-check-session"}]'
  exit 0
fi
exit 1
EOF
  chmod +x "$dir/claude"
}

for state in working blocked; do
  repo_dir="$TEST_ROOT/live-$state"
  mkdir -p "$repo_dir"
  _fake_claude_reporting_state "$TEST_ROOT/fake-bin-$state" "$state"
  printf 'live-check-bg\n' > "$repo_dir/claude_background_id"
  printf '%s\n' "$TEST_ROOT/fake-bin-$state/claude" > "$repo_dir/agent"
  _sgt_claude_bg_session_is_live "$repo_dir" || {
    printf 'FAIL: a session reporting state=%s was not treated as live\n' "$state" >&2
    exit 1
  }
done

for state in stopped failed totally-unrecognized-value; do
  repo_dir="$TEST_ROOT/notlive-$state"
  mkdir -p "$repo_dir"
  _fake_claude_reporting_state "$TEST_ROOT/fake-bin-notlive-$state" "$state"
  printf 'live-check-bg\n' > "$repo_dir/claude_background_id"
  printf '%s\n' "$TEST_ROOT/fake-bin-notlive-$state/claude" > "$repo_dir/agent"
  if _sgt_claude_bg_session_is_live "$repo_dir"; then
    printf 'FAIL: a session reporting state=%s was incorrectly treated as live\n' "$state" >&2
    exit 1
  fi
done

# No recorded background id at all: never live, never calls out to any binary.
repo_dir="$TEST_ROOT/notlive-no-id"
mkdir -p "$repo_dir"
if _sgt_claude_bg_session_is_live "$repo_dir"; then
  printf 'FAIL: a repo with no recorded background id was treated as live\n' >&2
  exit 1
fi

printf 'sgt-claude-stop-bg-session: ok\n'
