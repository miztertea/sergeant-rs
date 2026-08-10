#!/usr/bin/env bash
# One authenticated GET, timed, as a standalone process for `xargs -P` fan-out.
#
#   _get-one.sh <endpoint> <token> <path> <out-csv>
#
# Writes one CSV line: t_start_ns,wall_ns,curl_total_s,http_code,bytes
set -uo pipefail

endpoint="$1"; token="$2"; path="$3"; out="$4"

now_ns() {
  local t=${EPOCHREALTIME:-}
  if [ -n "$t" ]; then
    t=${t/,/.}
    local sec=${t%.*} frac=${t#*.}
    while [ ${#frac} -lt 9 ]; do frac="${frac}0"; done
    printf '%s%s' "$sec" "${frac:0:9}"
  else
    date +%s%N
  fi
}

t0="$(now_ns)"
line="$(curl -sS -m "${PERF_HTTP_TIMEOUT:-300}" -o /dev/null \
  -w '%{http_code} %{size_download} %{time_total}' \
  -H "Authorization: Bearer $token" "$endpoint$path" 2>/dev/null)"
t1="$(now_ns)"
read -r code bytes total <<< "$line"
printf '%s,%s,%s,%s,%s\n' "$t0" "$((t1 - t0))" "${total:-0}" "${code:-0}" "${bytes:-0}" > "$out"
