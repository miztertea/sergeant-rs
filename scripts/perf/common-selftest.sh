#!/usr/bin/env bash
# common-selftest.sh — regression coverage for #95: `perf_now_ns` and
# `perf_mark` in common.sh must fail loudly when neither clock branch works,
# rather than feed a malformed value (macOS's BSD `date +%s%N` prints the
# epoch seconds followed by a literal "N", e.g. "1786728341N") into
# arithmetic. This estate has no macOS host, so the real failure mode
# (`EPOCHREALTIME` unset because bash is 3.2, `date` lacking `%N`) is
# reproduced here on Linux by unsetting `EPOCHREALTIME` — which forces the
# same fallback branch macOS always takes — and shadowing `date` with a
# BSD-shaped stub on `PATH`.
#
# R-S0-12 "code is code": common.sh's clock functions changed executable
# behavior and need a test that fails when the fix is reverted (LESSONS L7).
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
# Test 2 (#95 regression, perf_now_ns): EPOCHREALTIME unset + BSD-shaped
# `date` must make perf_now_ns fail loudly — nonzero exit, a stderr message
# naming the cause — never a value containing a stray "N".
# ---------------------------------------------------------------------------
stub="$WORK/bsd-date"
mk_bsd_date_stub "$stub"
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
