#!/usr/bin/env bash
# A small synthetic bash fixture for the chunker corpus. ASCII-only.
set -euo pipefail

TMPDIR="${TMPDIR:-/tmp}"
LOG_FILE="${LOG_FILE:-/dev/null}"

log() {
    local message="$1"
    echo "[$(date +%s)] ${message}" >> "${LOG_FILE}"
}

require_command() {
    local name="$1"
    if ! command -v "${name}" >/dev/null 2>&1; then
        echo "missing required command: ${name}" >&2
        exit 1
    fi
}

build_one() {
    local target="$1"
    log "building ${target}"
    require_command cargo
    CARGO_BUILD_JOBS=6 cargo build --release -p "${target}"
}

build_all() {
    local targets=("$@")
    for target in "${targets[@]}"; do
        build_one "${target}"
    done
}

run_tests() {
    local target="$1"
    log "testing ${target}"
    require_command cargo
    cargo test -p "${target}" --no-fail-fast
}

main() {
    local targets=("fixture-one" "fixture-two" "fixture-three")
    build_all "${targets[@]}"
    for target in "${targets[@]}"; do
        run_tests "${target}"
    done
    log "done"
}

main "$@"
