#!/usr/bin/env bash
# Prove the generated sergeant-rs installer actually verifies what it downloads.
#
# Two tests, run against a local origin so nothing here touches the network or a
# published release:
#   positive — the real archive installs, and the installer does NOT take its
#              "no checksums to verify" path;
#   negative — a structurally valid but WRONG archive served under the right
#              filename (the supply-chain substitution a checksum defends
#              against) is rejected with a non-zero exit and nothing installed.
#
# A single-byte corruption is deliberately NOT used: xz's frame CRC rejects it
# before the installer's checksum ever runs, which would prove the wrong thing.
#
# Usage: verify-installer-checksums.sh <installer.sh> <artifacts-dir> <tag>
#   e.g. verify-installer-checksums.sh target/distrib/sergeant-rs-installer.sh \
#          target/distrib v0.1.2
set -euo pipefail

installer=${1:?usage: verify-installer-checksums.sh <installer.sh> <artifacts-dir> <tag>}
artifacts_dir=${2:?usage: verify-installer-checksums.sh <installer.sh> <artifacts-dir> <tag>}
tag=${3:?usage: verify-installer-checksums.sh <installer.sh> <artifacts-dir> <tag>}

# This script runs on a linux x86_64 runner, so the installer will select the
# x86_64 archive; the darwin archive is the substitute for the negative test.
host_archive="sergeant-rs-x86_64-unknown-linux-gnu.tar.xz"
other_archive="sergeant-rs-aarch64-apple-darwin.tar.xz"
port="${INSTALLER_SMOKE_PORT:-8917}"

for f in "$installer" "$artifacts_dir/$host_archive" "$artifacts_dir/$other_archive"; do
  if [ ! -f "$f" ]; then
    echo "::error::installer smoke test needs $f, which is not present" >&2
    exit 1
  fi
done

work=$(mktemp -d)
server_pid=""
cleanup() {
  if [ -n "$server_pid" ]; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
  rm -rf "$work"
}
trap cleanup EXIT

# Mirror GitHub's release-download URL layout; the installer appends
# /miztertea/sergeant-rs/releases/download/<tag> to the base URL it is given.
download_dir="$work/srv/miztertea/sergeant-rs/releases/download/$tag"
mkdir -p "$download_dir"
cp "$artifacts_dir/$host_archive" "$artifacts_dir/$other_archive" "$download_dir/"

python3 -m http.server "$port" --bind 127.0.0.1 --directory "$work/srv" \
  >"$work/http.log" 2>&1 &
server_pid=$!

ready=0
for _ in $(seq 1 50); do
  if curl -fsS -I -o /dev/null \
    "http://127.0.0.1:$port/miztertea/sergeant-rs/releases/download/$tag/$host_archive"; then
    ready=1
    break
  fi
  sleep 0.2
done
if [ "$ready" -ne 1 ]; then
  echo "::error::local origin did not come up on 127.0.0.1:$port" >&2
  cat "$work/http.log" >&2
  exit 1
fi

# SERGEANT_RS_INSTALL_DIR / SERGEANT_RS_NO_MODIFY_PATH / ..._GITHUB_BASE_URL are
# the generated installer's own env vars — nothing here edits the script.
run_installer() {
  local dest=$1 log=$2 rc=0
  mkdir -p "$dest"
  SERGEANT_RS_INSTALLER_GITHUB_BASE_URL="http://127.0.0.1:$port" \
  SERGEANT_RS_INSTALL_DIR="$dest" \
  SERGEANT_RS_NO_MODIFY_PATH=1 \
    sh "$installer" >"$log" 2>&1 || rc=$?
  return "$rc"
}

# ── positive ────────────────────────────────────────────────────────────────
if ! run_installer "$work/good" "$work/good.log"; then
  echo "::error::installer failed against a VALID archive" >&2
  cat "$work/good.log" >&2
  exit 1
fi
if grep -q 'no checksums to verify' "$work/good.log"; then
  echo "::error::installer took its unverified path ('no checksums to verify') — the generated installer carries no checksum values" >&2
  cat "$work/good.log" >&2
  exit 1
fi
if [ ! -x "$work/good/bin/sgt" ]; then
  echo "::error::installer reported success but did not install $work/good/bin/sgt" >&2
  cat "$work/good.log" >&2
  exit 1
fi
"$work/good/bin/sgt" --version

# ── negative ────────────────────────────────────────────────────────────────
# Serve a different, structurally valid archive under the host archive's name.
cp "$download_dir/$other_archive" "$download_dir/$host_archive"
if run_installer "$work/bad" "$work/bad.log"; then
  echo "::error::installer ACCEPTED a substituted archive — checksum verification is not effective" >&2
  cat "$work/bad.log" >&2
  exit 1
fi
if ! grep -q 'checksum mismatch' "$work/bad.log"; then
  echo "::error::installer rejected the substituted archive, but not for a checksum mismatch — the negative test is not proving what it claims" >&2
  cat "$work/bad.log" >&2
  exit 1
fi
if [ -e "$work/bad/bin/sgt" ]; then
  echo "::error::installer aborted on checksum mismatch but still left a binary behind" >&2
  exit 1
fi

echo "installer smoke test: valid archive installed and verified; substituted archive rejected on checksum mismatch"
