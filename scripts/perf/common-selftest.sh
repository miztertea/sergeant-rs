#!/usr/bin/env bash
# common-selftest.sh — regression coverage for #95: `perf_now_ns` and
# `perf_mark` in common.sh must fail loudly when no clock tier works, rather
# than feed a malformed value (macOS's BSD `date +%s%N` prints the epoch
# seconds followed by a literal "N", e.g. "1786728341N") into arithmetic.
# The real macOS failure mode (`EPOCHREALTIME` unset because bash is 3.2,
# `date` lacking `%N`) is reproduced here by unsetting `EPOCHREALTIME` and
# shadowing `date` with a BSD-shaped stub on `PATH` — originally written on a
# Linux host with no macOS to measure against; #95's clock (`perl
# -MTime::HiRes`, chosen 2026-08-15 on a real MacBook Pro M3 Pro after
# measuring it against python3 and bare `date`) is now a third real tier, so
# tests 2 and 3 below also shadow `perl` to still exercise the genuine
# "nothing works" case, and tests 2b/3b confirm the new tier itself.
#
# R-S0-12 "code is code": common.sh's clock functions changed executable
# behavior and need a test that fails when the fix is reverted (LESSONS L7).
#
# Ponytail rung R2 (new file, existing pattern): this is a new file, but it
# reuses scripts/coverage/selftest.sh's already-established shape verbatim —
# a standalone bash self-test wired into `cargo test` via a subprocess check
# (see tests/m6_surfaces.rs's the_coverage_harness_grades_its_own_accounting)
# rather than a new harness invented for this one guard.
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMMON="$SCRIPT_DIR/common.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/perf-common-selftest.XXXXXX")"
FAIL=0

cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT

fail() {
  echo "FAIL: $1" >&2
  FAIL=1
}
pass() {
  echo "PASS: $1"
}

# A BSD/macOS-shaped `date`: honors every flag common.sh actually uses
# (`+%s%N`, plus whatever else a caller passes) except `%N`, which it passes
# through literally — exactly what real BSD date does, because it has no
# idea what `%N` means. Delegates everything else to the *real* date
# binary, resolved to an absolute path now, before this stub goes on
# `PATH` — otherwise the fallback line would exec itself.
mk_bsd_date_stub() {
  local dir="$1" real_date
  real_date="$(command -v date)"
  mkdir -p "$dir"
  cat >"$dir/date" <<EOF
#!/bin/sh
case "\$1" in
  +%s%N) printf '%sN\n' "\$($real_date +%s)" ;;
  *) exec "$real_date" "\$@" ;;
esac
EOF
  chmod +x "$dir/date"
}

# A `perl` that is present on `$PATH` but cannot actually time anything —
# simulates a host where #95's chosen clock is unavailable too, so tests 2/3
# below still exercise the genuine "no clock at all" fail-loud path rather
# than quietly succeeding through perl once it becomes a real tier.
mk_broken_perl_stub() {
  local dir="$1"
  mkdir -p "$dir"
  cat >"$dir/perl" <<'EOF'
#!/bin/sh
exit 1
EOF
  chmod +x "$dir/perl"
}

# ---------------------------------------------------------------------------
# Test 1 (baseline): on this host, with EPOCHREALTIME available, perf_now_ns
# and perf_mark must still work exactly as before — the guard must not
# false-positive on the fast path.
# ---------------------------------------------------------------------------
out="$(bash -c '
  set -u
  source "'"$COMMON"'"
  perf_now_ns
' 2>"$WORK/baseline.err")"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "baseline (EPOCHREALTIME present): expected exit 0, got $rc: $(cat "$WORK/baseline.err")"
elif ! [[ "$out" =~ ^[0-9]+$ ]]; then
  fail "baseline (EPOCHREALTIME present): expected a plain integer, got '$out'"
else
  pass "baseline (EPOCHREALTIME present): perf_now_ns returns a plain integer"
fi

# ---------------------------------------------------------------------------
# Test 2 (#95 regression, perf_now_ns): EPOCHREALTIME unset, BSD-shaped
# `date`, AND a non-functional `perl` (every clock tier exhausted) must make
# perf_now_ns fail loudly — nonzero exit, a stderr message naming the cause
# — never a value containing a stray "N".
# ---------------------------------------------------------------------------
stub="$WORK/bsd-date"
mk_bsd_date_stub "$stub"
mk_broken_perl_stub "$stub"
out="$(PATH="$stub:$PATH" bash -c '
  set -u
  unset EPOCHREALTIME
  source "'"$COMMON"'"
  perf_now_ns
' 2>"$WORK/t2.err")"
rc=$?
if [ "$rc" -eq 0 ]; then
  fail "#95 (perf_now_ns, no working clock): expected nonzero exit, got 0 (output: '$out')"
elif [[ "$out" == *N* ]]; then
  fail "#95 (perf_now_ns, no working clock): malformed value leaked to stdout: '$out'"
elif ! grep -qF 'no working nanosecond clock' "$WORK/t2.err"; then
  fail "#95 (perf_now_ns, no working clock): stderr did not name the cause: $(cat "$WORK/t2.err")"
else
  pass "#95 (perf_now_ns, no working clock): fails loudly instead of emitting a malformed value"
fi

# ---------------------------------------------------------------------------
# Test 3 (#95 regression, perf_mark): same fault, through the fork-free
# nameref-style helper every scenario actually calls.
# ---------------------------------------------------------------------------
out="$(PATH="$stub:$PATH" bash -c '
  set -u
  unset EPOCHREALTIME
  source "'"$COMMON"'"
  perf_mark t0
  printf "%s" "$t0"
' 2>"$WORK/t3.err")"
rc=$?
if [ "$rc" -eq 0 ]; then
  fail "#95 (perf_mark, no working clock): expected nonzero exit, got 0 (t0='$out')"
elif [[ "$out" == *N* ]]; then
  fail "#95 (perf_mark, no working clock): malformed value leaked into the marked variable: '$out'"
elif ! grep -qF 'no working nanosecond clock' "$WORK/t3.err"; then
  fail "#95 (perf_mark, no working clock): stderr did not name the cause: $(cat "$WORK/t3.err")"
else
  pass "#95 (perf_mark, no working clock): fails loudly instead of emitting a malformed value"
fi

# ---------------------------------------------------------------------------
# Test 2b/3b (#95's chosen clock): EPOCHREALTIME unset, BSD-shaped `date`,
# but a *working* `perl` — the macOS shape this repo actually has now, per
# `docs/environments/macbook.md` — must resolve through the perl tier rather
# than fail. Uses the real `perl` on `$PATH` (not a stub): the point is that
# the chosen tier actually works, not just that its absence is caught.
# ---------------------------------------------------------------------------
if command -v perl >/dev/null 2>&1; then
  # A separate stub dir from test 2/3's: the BSD `date` shape, but no broken
  # `perl` shadowing it, so the real system perl resolves through the rest
  # of `$PATH`.
  stub2b="$WORK/bsd-date-real-perl"
  mk_bsd_date_stub "$stub2b"
  out="$(PATH="$stub2b:$PATH" bash -c '
    set -u
    unset EPOCHREALTIME
    source "'"$COMMON"'"
    perf_now_ns
  ' 2>"$WORK/t2b.err")"
  rc=$?
  if [ "$rc" -ne 0 ]; then
    fail "#95 (perf_now_ns, perl tier): expected exit 0, got $rc: $(cat "$WORK/t2b.err")"
  elif ! [[ "$out" =~ ^[0-9]+$ ]]; then
    fail "#95 (perf_now_ns, perl tier): expected a plain integer, got '$out'"
  else
    pass "#95 (perf_now_ns, perl tier): resolves through perl -MTime::HiRes when date lacks %N"
  fi

  out="$(PATH="$stub2b:$PATH" bash -c '
    set -u
    unset EPOCHREALTIME
    source "'"$COMMON"'"
    perf_mark t0
    printf "%s" "$t0"
  ' 2>"$WORK/t3b.err")"
  rc=$?
  if [ "$rc" -ne 0 ]; then
    fail "#95 (perf_mark, perl tier): expected exit 0, got $rc: $(cat "$WORK/t3b.err")"
  elif ! [[ "$out" =~ ^[0-9]+$ ]]; then
    fail "#95 (perf_mark, perl tier): expected a plain integer, got '$out'"
  else
    pass "#95 (perf_mark, perl tier): resolves through perl -MTime::HiRes when date lacks %N"
  fi
else
  echo "SKIPPED-ENV: no perl on this host, cannot exercise #95's chosen clock tier"
fi

# ---------------------------------------------------------------------------
# Test 4: EPOCHREALTIME unset but a GNU-shaped `date` (this host's real
# fallback shape) must still work — the guard must accept a genuinely
# numeric fallback, not just reject everything once EPOCHREALTIME is gone.
# ---------------------------------------------------------------------------
out="$(bash -c '
  set -u
  unset EPOCHREALTIME
  source "'"$COMMON"'"
  perf_now_ns
' 2>"$WORK/t4.err")"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "GNU date fallback: expected exit 0, got $rc: $(cat "$WORK/t4.err")"
elif ! [[ "$out" =~ ^[0-9]+$ ]]; then
  fail "GNU date fallback: expected a plain integer, got '$out'"
else
  pass "GNU date fallback: perf_now_ns still works when EPOCHREALTIME is unset but date supports %N"
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "common-selftest: FAILURES ABOVE"
  exit 1
fi
echo "common-selftest: all checks passed"
exit 0
