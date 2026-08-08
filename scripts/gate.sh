#!/usr/bin/env bash
# Shipping-gate runner: ensures the no-mistakes daemon is alive WITH the
# IS_SANDBOX env this root container requires (the daemon dies between
# long-gapped runs — observed twice on 2026-08-08), then starts the run.
# Usage: scripts/gate.sh "<intent>" [extra axi run flags...]
set -euo pipefail
intent="$1"; shift
no-mistakes daemon status >/dev/null 2>&1 || IS_SANDBOX=1 no-mistakes daemon start
pid=$(grep -oE '"pid":[0-9]+' /root/.no-mistakes/daemon.pid | cut -d: -f2)
grep -qz IS_SANDBOX "/proc/$pid/environ" || { no-mistakes daemon stop; IS_SANDBOX=1 no-mistakes daemon start; }
exec no-mistakes axi run --intent "$intent" --skip push,pr,ci "$@"
