#!/usr/bin/env bash
# Shipping-gate runner: ensures the no-mistakes daemon is alive WITH the
# IS_SANDBOX env this root container requires (the daemon dies between
# long-gapped runs — observed twice on 2026-08-08), then starts the run.
# Usage: scripts/gate.sh "<intent>" [extra axi run flags...]
set -euo pipefail
intent="$1"; shift
[ -z "$(git -C /home/user/sergeant-rs status --porcelain)" ] || { echo "gate.sh: tree not clean - commit first" >&2; exit 1; }
# CARGO_TARGET_DIR shared with the main checkout: the pipeline's disposable
# worktree reuses dependency artifacts (duckdb's ~2 GB rlib included) instead
# of a multi-GB cold build per run — this container hit ENOSPC during M5.
export CARGO_TARGET_DIR=/root/.no-mistakes/cargo-target  # pipeline-private persistent cache: warm across ITS runs, isolated from the main checkout (shared-with-main caused binary cross-contamination - worktree-baked env! paths in reused test binaries, diagnosed 2026-08-09)
no-mistakes daemon status >/dev/null 2>&1 || IS_SANDBOX=1 CARGO_TARGET_DIR="$CARGO_TARGET_DIR" no-mistakes daemon start
pid=$(grep -oE '"pid":[0-9]+' /root/.no-mistakes/daemon.pid | cut -d: -f2)
grep -qz CARGO_TARGET_DIR "/proc/$pid/environ" && grep -qz IS_SANDBOX "/proc/$pid/environ" || { no-mistakes daemon stop; IS_SANDBOX=1 CARGO_TARGET_DIR="$CARGO_TARGET_DIR" no-mistakes daemon start; }
exec no-mistakes axi run --intent "$intent" --skip push,pr,ci "$@"
