#!/usr/bin/env bash
# One submit, timed, as a standalone process so `xargs -P` can fan it out.
#
#   _submit-one.sh <endpoint> <token> <cwd> <intent> <out-csv> [repo]
#
# Writes exactly one CSV line:
#   t_start_ns,wall_ns,curl_total_s,http_code,work_id,state
#
# The request body (including its ULID command_id) is built *before* the clock
# starts, so the measured interval is the HTTP call and nothing else. curl's
# own %{time_total} is recorded alongside the shell's wall delta: the gap
# between them is this harness's per-call process cost, which is what tells a
# reader whether the client or the daemon saturated first.
#
# `repo`, when given, becomes explicit `scope.repos` (work submission now
# requires a declared scope — estate-root proposal §7.1); an empty `repo`
# submits no scope field, relying on the daemon's single-repository inference
# for a one-repository estate.
set -uo pipefail

endpoint="$1"; token="$2"; cwd="$3"; intent="$4"; out="$5"; repo="${6:-}"

alphabet=0123456789ABCDEFGHJKMNPQRSTVWXYZ
ulid() {
  local s="${alphabet:$((RANDOM % 8)):1}" i
  for ((i = 0; i < 25; i++)); do s+="${alphabet:$((RANDOM % 32)):1}"; done
  printf '%s' "$s"
}

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

if [ -n "$cwd" ]; then
  body="$(jq -nc --arg cid "$(ulid)" --arg intent "$intent" --arg cwd "$cwd" --arg r "$repo" \
    '{command_id:$cid, intent:$intent, backend:"fake", origin:{client:"cli", cwd:$cwd}}
     + (if $r == "" then {} else {scope:{repos:[$r]}} end)')"
else
  body="$(jq -nc --arg cid "$(ulid)" --arg intent "$intent" --arg r "$repo" \
    '{command_id:$cid, intent:$intent, backend:"fake"}
     + (if $r == "" then {} else {scope:{repos:[$r]}} end)')"
fi

t0="$(now_ns)"
resp="$(curl -sS -m "${PERF_HTTP_TIMEOUT:-300}" -X POST \
  -H "Authorization: Bearer $token" -H 'content-type: application/json' \
  -w '\n%{http_code} %{time_total}' -d "$body" "$endpoint/v1/work" 2>/dev/null)"
t1="$(now_ns)"

tail_line="$(printf '%s' "$resp" | tail -1)"
code="${tail_line%% *}"
total="${tail_line##* }"
json="$(printf '%s' "$resp" | head -n -1)"
work_id="$(printf '%s' "$json" | jq -r '.work.id // empty' 2>/dev/null)"
state="$(printf '%s' "$json" | jq -r '.work.state // .error.code // empty' 2>/dev/null)"

printf '%s,%s,%s,%s,%s,%s\n' "$t0" "$((t1 - t0))" "${total:-0}" "${code:-0}" "${work_id:-}" "${state:-}" > "$out"
