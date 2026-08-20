#!/usr/bin/env bash
# probe-env-selftest.sh — regression coverage for scripts/probe-env.sh's
# honesty invariants: never a blank/unmeasured cell, never a fabricated
# fact, never a nonzero exit because the HOST lacks a tool. Pure bash +
# coreutils; no cargo, no network required (network probes fail soft and
# are not asserted on here).
#
# This is R-S0-12 "code is code": probe-env.sh changes executable behavior
# and needs a test that fails when a fix is reverted (LESSONS L7). Each
# check below reproduces one fault-injection scenario from the blind-critic
# review that shipped alongside this test (see the commit that added it).
set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET="$SCRIPT_DIR/probe-env.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/probe-env-selftest.XXXXXX")"
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

# Build a stub PATH dir preloaded with real coreutils (symlinked) so a test
# can override just the one or two binaries it cares about.
mk_stub_path() {
  local dir="$1"
  shift
  mkdir -p "$dir"
  local t
  for t in "$@"; do
    local p
    p="$(command -v "$t" 2>/dev/null || true)"
    [ -n "$p" ] && ln -sf "$p" "$dir/$t"
  done
}

ALL_TOOLS=(bash mktemp chmod cat tr sed awk df uname nproc python3 curl docker stat hostname grep cut date rm timeout printf id chattr)

no_blank_cells() {
  # A data row (not the header/separator) with an empty Measured-value or
  # Evidence cell is a bug: "| Fact |  |  |" or "| Fact | val |  |".
  grep -nE '^\| [^|]+ \| *\| ' <<<"$1" | grep -v '^| Fact |' \
    && return 1
  grep -nE '^\| [^|]+ \| [^|]+ \| *\|$' <<<"$1" \
    && return 1
  return 0
}

# ---------------------------------------------------------------------------
# Test 1 (baseline): plain run on this host must exit 0, produce a dated
# header, no blank cells, and the paste-destination line.
# ---------------------------------------------------------------------------
out="$(bash "$TARGET" 2>/tmp/probe-env-selftest-baseline.err)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "baseline run: expected exit 0, got $rc"
else
  pass "baseline run: exit 0"
fi
if ! grep -q '^Facts measured .* on host' <<<"$out"; then
  fail "baseline run: missing dated header line"
else
  pass "baseline run: dated header present"
fi
if ! grep -q '^Paste destination: docs/environments/' <<<"$out"; then
  fail "baseline run: missing paste destination line"
else
  pass "baseline run: paste destination present"
fi
if ! no_blank_cells "$out"; then
  fail "baseline run: blank cell(s) in table"
else
  pass "baseline run: no blank cells"
fi

# ---------------------------------------------------------------------------
# Test 2 (E1 regression): uid 0 with claude ABSENT from PATH must not print
# the fabricated "IS_SANDBOX / root skip-flag refusal" measured-value row;
# the imported claude.rs claim must appear only in Notes, if at all.
# ---------------------------------------------------------------------------
d="$WORK/e1"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/claude" "$d/id"
cat >"$d/id" <<'EOF'
#!/bin/sh
case "$1" in
  -u) echo 0 ;;
  -un) echo root ;;
  -Gn) echo root ;;
  -nG) echo root ;;
  *) echo "uid=0(root) gid=0(root) groups=0(root)" ;;
esac
EOF
chmod +x "$d/id"
out="$(PATH="$d" bash "$TARGET" 2>/dev/null)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "E1 (uid0, no claude): expected exit 0, got $rc"
elif grep -qF 'cannot be used with root/sudo privileges' <<<"$out" && \
     ! sed -n '/^Notes/,$p' <<<"$out" | grep -qF 'cannot be used with root/sudo privileges'; then
  fail "E1 (uid0, no claude): imported claude.rs claim appears outside Notes section"
elif grep -qE '^\| IS_SANDBOX / root skip-flag refusal \|' <<<"$out"; then
  fail "E1 (uid0, no claude): stale fabricated fact row still present"
else
  pass "E1 (uid0, no claude): no fabricated measured-value row"
fi

# ---------------------------------------------------------------------------
# Test 3 (E2 regression): chmod always fails (exit 1, mode unchanged) must
# yield "unmeasurable", never a confident "NOT enforced" claim.
# ---------------------------------------------------------------------------
d="$WORK/e2"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/chmod"
cat >"$d/chmod" <<'EOF'
#!/bin/sh
exit 1
EOF
chmod +x "$d/chmod"
out="$(PATH="$d" bash "$TARGET" 2>/dev/null)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "E2 (chmod always fails): expected exit 0, got $rc"
elif grep -qE '^\| DAC / permission-bit enforcement \| NOT enforced' <<<"$out"; then
  fail "E2 (chmod always fails): wrongly reported NOT enforced when chmod couldn't change the mode"
elif ! grep -qE '^\| DAC / permission-bit enforcement \| unmeasurable' <<<"$out"; then
  fail "E2 (chmod always fails): DAC row did not degrade to unmeasurable"
else
  pass "E2 (chmod always fails): DAC row correctly unmeasurable"
fi

# ---------------------------------------------------------------------------
# Test 4 (W2 regression): chattr +i succeeds but the inline -i cleanup
# fails; the EXIT trap must retry it (leak guard must not be cleared on a
# failed cleanup attempt).
# ---------------------------------------------------------------------------
d="$WORK/w2"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/chattr"
log="$WORK/w2-chattr.log"
: >"$log"
cat >"$d/chattr" <<EOF
#!/bin/sh
echo "\$*" >> "$log"
case "\$1" in
  +i) exit 0 ;;
  -i) exit 1 ;;
esac
exit 0
EOF
chmod +x "$d/chattr"
PATH="$d" bash "$TARGET" >/dev/null 2>/dev/null
calls="$(wc -l <"$log" | tr -d ' ')"
if [ "$calls" -lt 3 ]; then
  fail "W2 (chattr -i fails): expected at least 3 chattr calls (+i, failing inline -i, trap retry -i), got $calls"
else
  pass "W2 (chattr -i fails): EXIT trap retried cleanup ($calls chattr calls)"
fi

# ---------------------------------------------------------------------------
# Test 5 (W3 regression): dockerd stderr warnings must not poison the JSON
# parse of docker info's stdout.
# ---------------------------------------------------------------------------
d="$WORK/w3"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/docker"
cat >"$d/docker" <<'EOF'
#!/bin/sh
if [ "$1" = "--version" ]; then echo "Docker version 29.7.2, build stub"; exit 0; fi
if [ "$1" = "info" ]; then
  echo "WARNING: No swap limit support" >&2
  echo "WARNING: bridge-nf-call-iptables is disabled" >&2
  echo '{"Driver":"overlay2","CgroupVersion":"2"}'
  exit 0
fi
exit 0
EOF
chmod +x "$d/docker"
out="$(PATH="$d" bash "$TARGET" 2>/dev/null)"
if grep -qE '^\| Docker:.*storage driver: unmeasurable' <<<"$out"; then
  fail "W3 (docker stderr warnings): storage driver wrongly downgraded to unmeasurable"
elif ! grep -qE '^\| Docker:.*storage driver: overlay2' <<<"$out"; then
  fail "W3 (docker stderr warnings): storage driver not correctly parsed as overlay2"
else
  pass "W3 (docker stderr warnings): stdout/stderr split correctly, JSON parsed"
fi

# ---------------------------------------------------------------------------
# Test 6 (W4a regression): HOME unset must not trigger an unbound-variable
# abort and must not produce blank cells.
# ---------------------------------------------------------------------------
out="$(env -u HOME PATH="$PATH" bash "$TARGET" 2>"$WORK/w4a.err")"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "W4a (HOME unset): expected exit 0, got $rc"
elif grep -q 'unbound variable' "$WORK/w4a.err"; then
  fail "W4a (HOME unset): unbound-variable error on stderr"
elif ! no_blank_cells "$out"; then
  fail "W4a (HOME unset): blank cell(s) in table"
else
  pass "W4a (HOME unset): no crash, no blank cells"
fi

# ---------------------------------------------------------------------------
# Test 7 (W4b regression): empty PATH (no tr/sed/awk/coreutils at all) must
# still exit 0 with no blank cells — mdcell/oneline must not depend on
# external tools.
# ---------------------------------------------------------------------------
out="$(env -i PATH= HOME="$HOME" "$(command -v bash)" "$TARGET" 2>/dev/null)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "W4b (empty PATH): expected exit 0, got $rc"
elif ! no_blank_cells "$out"; then
  fail "W4b (empty PATH): blank cell(s) in table"
else
  pass "W4b (empty PATH): no crash, no blank cells"
fi

# ---------------------------------------------------------------------------
# Test 8 (E3 regression): O_DIRECT rows must distinguish open-refused /
# unaligned-accepted / unaligned-rejected, not just report "open SUCCEEDS"
# for both outcomes.
# ---------------------------------------------------------------------------
out="$(bash "$TARGET" 2>/dev/null)"
if have_python3="$(command -v python3 2>/dev/null)"; [ -n "$have_python3" ]; then
  if grep -qE '^\| O_DIRECT open\+unaligned-write .*\| open SUCCEEDS \|' <<<"$out"; then
    fail "E3 (O_DIRECT): row still reports bare 'open SUCCEEDS' with no write-alignment outcome"
  elif ! grep -qE '^\| O_DIRECT open\+unaligned-write .*(ACCEPTED|REJECTED|open FAILS)' <<<"$out"; then
    fail "E3 (O_DIRECT): row missing a distinguishing alignment outcome"
  else
    pass "E3 (O_DIRECT): row distinguishes open vs write-alignment outcome"
  fi
else
  echo "SKIPPED-ENV: E3 check needs python3, absent on this host"
fi

# ---------------------------------------------------------------------------
# Test 9: `pgrep`-adjacent hygiene — the script must not leave its scratch
# dirs behind (this also indirectly checks the EXIT trap runs on the happy
# path, complementing test 4's failure-path check).
# ---------------------------------------------------------------------------
before="$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'probe-env.*' 2>/dev/null | wc -l | tr -d ' ')"
bash "$TARGET" >/dev/null 2>/dev/null
after="$(find "${TMPDIR:-/tmp}" -maxdepth 1 -name 'probe-env.*' 2>/dev/null | wc -l | tr -d ' ')"
if [ "$after" -gt "$before" ]; then
  fail "cleanup: probe-env scratch dir(s) leaked under \$TMPDIR ($before -> $after)"
else
  pass "cleanup: no leaked scratch dirs under \$TMPDIR"
fi

# ---------------------------------------------------------------------------
# Test 10 (#143 bounded-fallback regression): PATH lacking both timeout and
# gtimeout must not leak the old sentinel string into a probe's measured
# value, and must surface the condition as a measured "Probe bounds" row.
# ---------------------------------------------------------------------------
d="$WORK/b1"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/timeout" "$d/gtimeout"
out="$(PATH="$d" bash "$TARGET" 2>/dev/null)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "#143 (no timeout/gtimeout): expected exit 0, got $rc"
elif grep -qF 'no timeout(1) available' <<<"$out"; then
  fail "#143 (no timeout/gtimeout): stale sentinel string leaked into a fact value"
elif ! grep -qE '^\| Probe bounds \| unenforced:' <<<"$out"; then
  fail "#143 (no timeout/gtimeout): Probe bounds row missing or not reporting unenforced"
else
  pass "#143 (no timeout/gtimeout): unenforced state measured, no sentinel leak"
fi

# ---------------------------------------------------------------------------
# Test 11 (#143 gtimeout-fallback regression): PATH lacking timeout but with
# a working gtimeout must use it — no Probe bounds row, since bounding is
# still enforced via the fallback.
# ---------------------------------------------------------------------------
d="$WORK/b2"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/timeout"
cat >"$d/gtimeout" <<'EOF'
#!/bin/sh
shift
exec "$@"
EOF
chmod +x "$d/gtimeout"
out="$(PATH="$d" bash "$TARGET" 2>/dev/null)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "#143 (gtimeout fallback): expected exit 0, got $rc"
elif grep -qE '^\| Probe bounds \|' <<<"$out"; then
  fail "#143 (gtimeout fallback): Probe bounds row present even though gtimeout was on PATH"
else
  pass "#143 (gtimeout fallback): gtimeout used, no unenforced row"
fi

# ---------------------------------------------------------------------------
# Test 12 (#143 cores-sysctl-fallback regression): PATH lacking nproc but
# with a stub sysctl reporting hw.ncpu must use it for the Cores row, with
# evidence naming sysctl, not nproc.
# ---------------------------------------------------------------------------
d="$WORK/b3"
mk_stub_path "$d" "${ALL_TOOLS[@]}"
rm -f "$d/nproc"
cat >"$d/sysctl" <<'EOF'
#!/bin/sh
if [ "$1" = "-n" ] && [ "$2" = "hw.ncpu" ]; then
  echo 12
  exit 0
fi
exit 1
EOF
chmod +x "$d/sysctl"
out="$(PATH="$d" bash "$TARGET" 2>/dev/null)"
rc=$?
if [ "$rc" -ne 0 ]; then
  fail "#143 (sysctl cores fallback): expected exit 0, got $rc"
elif ! grep -qE '^\| Cores \| 12 \| sysctl -n hw.ncpu' <<<"$out"; then
  fail "#143 (sysctl cores fallback): Cores row did not report 12 via sysctl -n hw.ncpu"
else
  pass "#143 (sysctl cores fallback): Cores measured via sysctl fallback"
fi

echo
if [ "$FAIL" -ne 0 ]; then
  echo "probe-env-selftest: FAILURES ABOVE"
  exit 1
fi
echo "probe-env-selftest: all checks passed"
exit 0
