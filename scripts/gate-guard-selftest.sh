#!/usr/bin/env bash
# gate-guard-selftest.sh — regression coverage for #120's shipping-gate
# false-green: scripts/gate.sh's `guard_against_empty_diff_base` must refuse
# to start a `no-mistakes axi run` when this checkout's diff against
# origin's detected default branch is empty (the exact precondition that
# made no-mistakes silently fast-path to `outcome: passed` without running
# review/test/document/lint — see issue #120's thread), and must not
# false-positive when the diff is genuinely non-empty.
#
# This drives the real scripts/gate.sh end to end (not a reimplementation
# of the guard) against real git fixtures — a manufactured "origin" working
# copy and a "work" clone of it, exactly the shape #120 describes (origin is
# a live, non-bare working copy, not a fixed-default-branch host). Since
# gate.sh resolves REPO_ROOT from its own script path (`$SCRIPT_DIR/..`),
# not from cwd or an argument, each fixture gets its own copy of gate.sh
# under `<fixture>/scripts/` so REPO_ROOT resolves to the fixture, not this
# checkout. The no-mistakes daemon plumbing above the guard (not under test
# here) is satisfied with a stub `no-mistakes` binary and a pre-seeded
# daemon.pid pointing at a live sleeper process, so the daemon-health checks
# pass without a real daemon and execution reaches the guard.
#
# R-S0-12 "code is code": gate.sh changed executable behavior and needs a
# test that fails when the fix is reverted (LESSONS L7). Same pattern as
# scripts/coverage/selftest.sh and scripts/perf/common-selftest.sh: a shell
# script outside the gate is a script whose checks are claims.
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REAL_GATE="$SCRIPT_DIR/gate.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/gate-guard-selftest.XXXXXX")"
FAIL=0
SLEEPER_PIDS=()

cleanup() {
  local p
  for p in "${SLEEPER_PIDS[@]:-}"; do
    [ -n "$p" ] && kill "$p" >/dev/null 2>&1
  done
  rm -rf "$WORK"
}
trap cleanup EXIT

fail() {
  echo "FAIL: $1" >&2
  FAIL=1
}
pass() {
  echo "PASS: $1"
}

# Stub `no-mistakes`: satisfies gate.sh's daemon-health checks without a
# real daemon, and records whether `axi run` (the pipeline the guard must
# block) was actually reached.
mk_stub_no_mistakes() {
  local dir="$1"
  mkdir -p "$dir"
  cat >"$dir/no-mistakes" <<'EOF'
#!/usr/bin/env bash
set -u
case "${1:-}" in
  daemon) exit 0 ;;
  axi)
    if [ "${2:-}" = "run" ] && [ -n "${AXI_RUN_MARKER:-}" ]; then
      : > "$AXI_RUN_MARKER"
    fi
    exit 0
    ;;
  *) exit 0 ;;
esac
EOF
  chmod +x "$dir/no-mistakes"
}

# A live process whose /proc/<pid>/environ carries CARGO_TARGET_DIR, seeded
# as the daemon.pid gate.sh will read — satisfies daemon_env_ok() without a
# real no-mistakes daemon.
start_stub_daemon() {
  local no_mistakes_home="$1"
  mkdir -p "$no_mistakes_home"
  env CARGO_TARGET_DIR="$no_mistakes_home/cargo-target" sleep 300 &
  local pid=$!
  SLEEPER_PIDS+=("$pid")
  printf '{"pid":%s}\n' "$pid" >"$no_mistakes_home/daemon.pid"
}

# Builds an "origin" working copy (main, one commit carrying f.txt and a
# copy of the real gate.sh) and a "work" clone of it — the exact live,
# non-bare-origin shape #120 describes. Echoes the work checkout's path.
mk_fixture_pair() {
  local base="$1" origin work
  origin="$base/origin_repo"
  work="$base/work"

  git init -q -b main "$origin"
  git -C "$origin" config user.email t@t.com
  git -C "$origin" config user.name t
  mkdir -p "$origin/scripts"
  cp "$REAL_GATE" "$origin/scripts/gate.sh"
  chmod +x "$origin/scripts/gate.sh"
  echo a >"$origin/f.txt"
  git -C "$origin" add f.txt scripts/gate.sh
  git -C "$origin" commit -qm "c1"

  git clone -q "$origin" "$work"
  git -C "$work" config user.email t@t.com
  git -C "$work" config user.name t
  git -C "$work" checkout -q -b workbranch

  echo "$work"
}

run_gate() {
  local work="$1" no_mistakes_home="$2" stub_bin="$3" marker="$4"
  env \
    NO_MISTAKES_HOME="$no_mistakes_home" \
    AXI_RUN_MARKER="$marker" \
    PATH="$stub_bin:$PATH" \
    bash "$work/scripts/gate.sh" "selftest intent" \
    >"$WORK/out.$$" 2>"$WORK/err.$$"
  local rc=$?
  cat "$WORK/out.$$" "$WORK/err.$$" >"$WORK/combined.$$"
  echo "$rc"
}

# ---------------------------------------------------------------------------
# Test 1: work checkout cut from origin's exact tip, zero unique commits —
# the precise #120 precondition. The guard must refuse before the stub's
# `axi run` is ever reached.
# ---------------------------------------------------------------------------
work="$(mk_fixture_pair "$WORK/t1")"
nmh="$WORK/t1-nmh"
stub="$WORK/t1-stub"
marker="$WORK/t1-axi-run-marker"
mk_stub_no_mistakes "$stub"
start_stub_daemon "$nmh"
rc="$(run_gate "$work" "$nmh" "$stub" "$marker")"

if [ "$rc" -eq 0 ]; then
  fail "empty-diff fixture: expected nonzero exit, got 0"
else
  pass "empty-diff fixture: exited nonzero"
fi
if [ -e "$marker" ]; then
  fail "empty-diff fixture: guard did not block — 'no-mistakes axi run' was reached"
else
  pass "empty-diff fixture: 'no-mistakes axi run' was never reached"
fi
if ! grep -qF '#120' "$WORK/combined.$$"; then
  fail "empty-diff fixture: refusal message does not name #120"
else
  pass "empty-diff fixture: refusal message names #120"
fi

# ---------------------------------------------------------------------------
# Test 2: work checkout with one real commit ahead of origin's tip — the
# guard must not false-positive; the run must proceed to the stub's `axi
# run` exactly as gate.sh's pre-#120-guard behavior did.
# ---------------------------------------------------------------------------
work="$(mk_fixture_pair "$WORK/t2")"
echo b >>"$work/f.txt"
git -C "$work" commit -qam "real work commit"
nmh="$WORK/t2-nmh"
stub="$WORK/t2-stub"
marker="$WORK/t2-axi-run-marker"
mk_stub_no_mistakes "$stub"
start_stub_daemon "$nmh"
rc="$(run_gate "$work" "$nmh" "$stub" "$marker")"

if [ "$rc" -ne 0 ]; then
  fail "non-empty-diff fixture: expected exit 0, got $rc ($(cat "$WORK/combined.$$"))"
else
  pass "non-empty-diff fixture: exited 0"
fi
if [ ! -e "$marker" ]; then
  fail "non-empty-diff fixture: 'no-mistakes axi run' was never reached — guard false-positived"
else
  pass "non-empty-diff fixture: 'no-mistakes axi run' was reached, guard did not false-positive"
fi
if grep -qF '#120' "$WORK/combined.$$"; then
  fail "non-empty-diff fixture: unexpected #120 refusal message on a genuinely non-empty diff"
else
  pass "non-empty-diff fixture: no #120 refusal message"
fi

# ---------------------------------------------------------------------------
# Test 3 (two-dot vs three-dot regression): origin's default branch has
# advanced independently after the work branch was cut, and the work branch
# itself carries zero unique commits — the same "genuinely cut from that
# tip" shape #120 names, just with origin moving afterward instead of
# before. A two-dot diff against the fetched base would wrongly report
# non-empty here (it shows origin's own unique changes, not this branch's),
# which would silently defeat the guard. The three-dot form gate.sh actually
# uses must still refuse.
# ---------------------------------------------------------------------------
work="$(mk_fixture_pair "$WORK/t3")"
echo c >"$WORK/t3-origin-advance-marker"
(
  cd "$WORK/t3/origin_repo" || exit 1
  echo b >>f.txt
  git commit -qam "origin advances after the branch was cut"
)
nmh="$WORK/t3-nmh"
stub="$WORK/t3-stub"
marker="$WORK/t3-axi-run-marker"
mk_stub_no_mistakes "$stub"
start_stub_daemon "$nmh"
rc="$(run_gate "$work" "$nmh" "$stub" "$marker")"

if [ "$rc" -eq 0 ]; then
  fail "origin-advanced fixture: expected nonzero exit, got 0 — two-dot-shaped false negative"
else
  pass "origin-advanced fixture: exited nonzero"
fi
if [ -e "$marker" ]; then
  fail "origin-advanced fixture: guard did not block — 'no-mistakes axi run' was reached"
else
  pass "origin-advanced fixture: 'no-mistakes axi run' was never reached"
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "gate-guard-selftest: FAILURES ABOVE"
  exit 1
fi
echo "gate-guard-selftest: all checks passed"
exit 0
