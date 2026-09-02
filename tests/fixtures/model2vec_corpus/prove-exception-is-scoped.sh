#!/usr/bin/env bash
# F5 (S5 W3b) — proof that deny.toml's RUSTSEC-2024-0436 exception is scoped
# to that one advisory ID, not to `paste`, not to the model2vec subtree, and
# not to the crate that carries it.
#
# Directly modelled on tests/fixtures/anydoc_corpus/prove-exception-is-scoped.sh
# (R2 — the mechanism this repo already owns and already reviewed), retargeted
# from the anydoc subtree to the model2vec one.
#
# What this does: takes cargo-deny's already-fetched, real advisory-db cache
# (the same one `cargo deny check` reads, per deny.toml's `db-path`), copies
# it (never mutates the real cache), drops in one SYNTHETIC advisory against
# `tokenizers` — the crate sitting directly between `paste` and
# `model2vec-rs` on the very edge the real exception covers — under a fresh
# RUSTSEC id this repo's deny.toml has never heard of, then runs `cargo deny
# check advisories --offline` against THIS repo's real deny.toml (the file is
# read as-is; nothing here edits it) with db-path repointed at the doctored
# copy.
#
# Expected, and asserted below:
#   - RUSTSEC-2024-0436 (real, paste) does NOT appear as an error —
#     deny.toml's exception still covers it.
#   - RUSTSEC-2098-9998 (synthetic, tokenizers) DOES appear as an error — a
#     different advisory ID reaching the graph through the same subtree is
#     not swept in by the same entry. This is the owner ruling's condition
#     "The next advisory must still fail the gate", made checkable.
#   - The command's exit code is 1 (advisories check fails overall) because
#     of the synthetic advisory.
#
# The anydoc entry (RUSTSEC-2026-0192) is untouched by this proof and by the
# W3b change; it keeps working exactly as its own proof asserts.
#
# Requires the real advisory-db already fetched once (`cargo deny fetch` or
# any prior `cargo deny check` populates `~/.cargo/advisory-db`, per this
# repo's deny.toml `db-path`). Network-free otherwise: `--offline` is passed
# explicitly so this script never re-fetches or phones out.
#
# Usage: tests/fixtures/model2vec_corpus/prove-exception-is-scoped.sh
# Evidence from the last recorded run: F5-exception-narrowness-proof.md
# (beside this script).

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
tmpdir="${TMPDIR:-/var/tmp/sgt-test-tmp}"
work="$tmpdir/f5-narrowness-proof-$$"
mkdir -p "$work"
trap 'rm -rf "$work"' EXIT

real_db_path="$(grep -m1 '^db-path' "$repo_root/deny.toml" | sed -E 's/^db-path = "(.*)"/\1/')"
real_db_path="${real_db_path/#\~/$HOME}"
cache_dir="$(find "$real_db_path" -maxdepth 1 -type d -name 'advisory-db-*' | head -n1)"
if [[ -z "$cache_dir" ]]; then
  echo "FAIL: no cached advisory-db found under $real_db_path — run 'cargo deny fetch' once first" >&2
  exit 2
fi
cache_name="$(basename "$cache_dir")"

echo "== copying cached advisory-db ($cache_name) — the real cache is never touched =="
copy_db_path="$work/db-path"
mkdir -p "$copy_db_path"
cp -r "$cache_dir" "$copy_db_path/$cache_name"

echo "== dropping a SYNTHETIC advisory against tokenizers (the crate between paste and model2vec-rs) =="
target_dir="$copy_db_path/$cache_name/crates/tokenizers"
mkdir -p "$target_dir"
cat > "$target_dir/RUSTSEC-2098-9998.md" <<'EOF'
```toml
[advisory]
id = "RUSTSEC-2098-9998"
package = "tokenizers"
date = "2026-08-30"
url = "https://example.invalid/synthetic-advisory-for-gate-proof"
informational = "unmaintained"

[versions]
patched = []
```

# SYNTHETIC advisory — gate-narrowness proof only, not a real vulnerability

Fabricated by `tests/fixtures/model2vec_corpus/prove-exception-is-scoped.sh`
to prove deny.toml's `RUSTSEC-2024-0436` ignore entry does not suppress a
*different* advisory ID reaching the graph through the same `model2vec-rs ->
tokenizers` subtree.
EOF

echo "== running cargo deny check advisories against the repo's real deny.toml, db-path repointed at the doctored copy =="
proof_config="$work/deny-proof.toml"
sed "s#^db-path = .*#db-path = \"$copy_db_path\"#" "$repo_root/deny.toml" > "$proof_config"

set +e
( cd "$repo_root" && TMPDIR="$tmpdir" cargo deny --offline --config "$proof_config" check advisories ) \
  > "$work/output.txt" 2>&1
status=$?
set -e

echo "----- cargo deny output -----"
cat "$work/output.txt"
echo "------------------------------"

fail=0

if grep -q 'RUSTSEC-2024-0436' "$work/output.txt" && grep -B2 'RUSTSEC-2024-0436' "$work/output.txt" | grep -q '^error'; then
  echo "FAIL: RUSTSEC-2024-0436 (real, paste) still surfaces as an error — the ignore entry is broken" >&2
  fail=1
fi

if ! grep -q 'RUSTSEC-2098-9998' "$work/output.txt"; then
  echo "FAIL: the synthetic RUSTSEC-2098-9998 advisory did not appear at all — the proof did not exercise what it claims to" >&2
  fail=1
fi

if ! grep -q '^error\[unmaintained\]: SYNTHETIC advisory' "$work/output.txt"; then
  echo "FAIL: the synthetic advisory did not surface as an error[] — the exception is broader than one advisory ID" >&2
  fail=1
fi

if [[ $status -ne 1 ]]; then
  echo "FAIL: expected cargo deny check advisories to exit 1 (still failing, on the synthetic advisory), got $status" >&2
  fail=1
fi

if [[ $fail -eq 0 ]]; then
  echo "PASS: RUSTSEC-2024-0436 stays ignored; RUSTSEC-2098-9998 (a different advisory, same model2vec subtree) still fails the gate."
  exit 0
else
  exit 1
fi
