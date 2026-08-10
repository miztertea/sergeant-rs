#!/usr/bin/env bash
# The §39 first-user-experience walkthrough, executable.
#
# A developer clones a repository, asks for work, sergeant materializes a
# surface, runs a workflow, stops to ask a question, is answered, finishes, and
# retires the surface — and at every step this script prints *where the
# evidence lives*, because "sergeant recorded it" is only a claim until you can
# go and look.
#
# It runs against the deterministic fake backend (§37) in a throwaway data dir
# and a throwaway git repo, so it burns no tokens and leaves nothing behind.
# Exit 0 means the walkthrough held: every state transition happened in order
# and every evidence pointer resolved.
#
#   scripts/demo.sh                 # fake backend, deterministic, no tokens
#   scripts/demo.sh --real-claude   # same shape through the real adapter
#
# `--real-claude` requires SERGEANT_CLAUDE_TESTS=1 (the budget discipline the
# rest of this repo uses for anything that spends model tokens) and an
# installed, entitled `claude`. Two things differ on that path, both because
# of what the real adapter is rather than what the walkthrough wants:
#
#   * a stage runs inside the submit/respond HTTP request, so the client's
#     ten-second default timeout is raised (SGT_CLIENT_TIMEOUT_SECS);
#   * P0's Claude adapter emits no ask-the-human signal, so the needs_input
#     pause is not required — only §37's fake backend can script one. The run
#     then proves submit → surface → stages → completed → retire, and says so
#     out loud rather than quietly skipping a step.
#
# Setting SGT_FAKE_SCRIPT before calling takes the same "the pause is not
# required" path with the fake backend, which is how that branch is exercised
# without spending tokens.
#
# With KEEP_DEMO_DIR=1 the throwaway directory survives, holding the artifacts
# every evidence pointer below was resolved from: graph.json, dashboard.html,
# analytics.json, and the data dir itself.

set -euo pipefail

REAL_CLAUDE=0
for arg in "$@"; do
  case "$arg" in
    --real-claude) REAL_CLAUDE=1 ;;
    -h|--help) sed -n '2,41p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "demo.sh: unknown argument $arg" >&2; exit 2 ;;
  esac
done

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SGT="${SGT_BIN:-}"
if [ -z "$SGT" ]; then
  echo "building sgt…"
  cargo build --quiet --manifest-path "$REPO_ROOT/Cargo.toml" --bin sgt
  SGT="$REPO_ROOT/target/debug/sgt"
fi
[ -x "$SGT" ] || { echo "demo.sh: no sgt binary at $SGT" >&2; exit 1; }

WORKDIR="$(mktemp -d "${TMPDIR:-/tmp}/sgt-demo-XXXXXX")"
DATA_DIR="$WORKDIR/data"
REPO="$WORKDIR/service"
mkdir -p "$DATA_DIR"

# How long the daemon gets to shut down on SIGTERM before SIGKILL, and then
# to leave the process table. The same ten and five seconds as
# `tests/support/mod.rs`'s reaper: two teardown paths that disagreed about
# "too slow" would report a slow shutdown differently depending on which one
# ran, and this script's daemon is reaped by nothing else — its data dir is a
# bare `mktemp -d`, not a `DataDir`, so no test guard can see it.
DEMO_TERM_GRACE_TENTHS=100
DEMO_KILL_GRACE_TENTHS=50

# Wait up to <tenths> tenths of a second for <pid> to disappear.
wait_gone() { # wait_gone <pid> <tenths>
  local pid="$1" tenths="$2" i=0
  while [ "$i" -lt "$tenths" ]; do
    kill -0 "$pid" 2>/dev/null || return 0
    sleep 0.1
    i=$((i + 1))
  done
  ! kill -0 "$pid" 2>/dev/null
}

cleanup() {
  local status=$?
  # The daemon outlives this shell by design (it is detached), so it is asked
  # to stop explicitly, escalated if it will not, and *verified gone before
  # anything is deleted*. The order is the point: `rm -rf` under a live daemon
  # leaves it running on a directory that no longer exists — the exact shape
  # that accumulated 89 orphans on one container — and, because SIGKILL runs
  # nothing registered at exit, it also throws away whatever the daemon would
  # have flushed on the way out (its coverage profile, when this script is run
  # under instrumentation). TERM → 10s grace → KILL → verify-gone, matching
  # the reaper in tests/support/mod.rs.
  #
  # PIN, AND ITS LIMITS (R-S0-5, recorded honestly). The check below is
  # self-pinning only in part: `tests/m6_surfaces.rs` t4 runs this script and
  # requires exit 0, so a daemon that survived teardown fails that test rather
  # than passing unnoticed. What that does *not* exercise is everything past
  # the happy path — on a healthy run the daemon always stops on SIGTERM, so
  # the escalation branch, the survival branch and this early `exit 1` are
  # never taken by the suite, and no test asserts which signal was needed.
  # They were exercised by hand in a disposable worktree (S1 phase 1) by
  # pointing the teardown at a live unrelated pid and confirming the script
  # exited nonzero with the directory still on disk. Two further gaps stay
  # open by construction: `kill -0` cannot tell a reused pid from the original
  # daemon, and a daemon that never wrote `runtime.json` is invisible here.
  local pid=""
  if [ -f "$DATA_DIR/runtime.json" ]; then
    pid="$(sed -n 's/.*"pid": *\([0-9]*\).*/\1/p' "$DATA_DIR/runtime.json" | head -1)"
  fi
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    kill -TERM "$pid" 2>/dev/null || true
    if ! wait_gone "$pid" "$DEMO_TERM_GRACE_TENTHS"; then
      printf 'demo.sh: daemon %s ignored SIGTERM for %ss — escalating to SIGKILL (nothing registered at exit runs)\n' \
        "$pid" "$((DEMO_TERM_GRACE_TENTHS / 10))" >&2
      kill -KILL "$pid" 2>/dev/null || true
      wait_gone "$pid" "$DEMO_KILL_GRACE_TENTHS" || true
    fi
  fi
  if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
    # Fail closed: never delete a data dir out from under a live daemon, and
    # never let the run be reported as clean when it was not.
    printf 'demo.sh: daemon %s survived SIGTERM and SIGKILL — leaving %s on disk\n' \
      "$pid" "$WORKDIR" >&2
    exit 1
  fi

  if [ "${KEEP_DEMO_DIR:-0}" = "1" ]; then
    echo "demo directory kept at $WORKDIR"
  else
    rm -rf "$WORKDIR"
  fi
  exit "$status"
}
trap cleanup EXIT

sgt() { "$SGT" --data-dir "$DATA_DIR" "$@"; }

step()     { printf '\n\033[1m== %s\033[0m\n' "$*"; }
say()      { printf '   %s\n' "$*"; }
evidence() { printf '   \033[2m→ evidence: %s\033[0m\n' "$*"; }
fail()     { printf '\n\033[31mdemo failed: %s\033[0m\n' "$*" >&2; exit 1; }

json() { python3 -c '
import json,sys
value = json.load(sys.stdin)
for key in sys.argv[1].split("."):
    if value is None: break
    value = value[int(key)] if isinstance(value, list) else value.get(key)
print("" if value is None else value)
' "$1"; }

# The §39 workflow: two stages, so stage progression is visible. The first
# stage stops to ask a question, exactly as §39's review loop does.
#
# PAUSE_REQUIRED says whether this run is entitled to insist on that pause.
# It is, for the default fake script — the pause is scripted, so its absence
# would be a real regression. It is not for a caller-supplied script, and it
# is not for the real adapter: P0's Claude backend emits no ask-the-human
# signal at all (only §37's fake backend scripts one), so a run through it
# proves the arc without its pause rather than failing on a state the backend
# cannot produce.
if [ -n "${SGT_FAKE_SCRIPT:-}" ]; then
  PAUSE_REQUIRED=0
else
  export SGT_FAKE_SCRIPT="needs_input:Should the retry budget be 3 attempts?;complete:retry handling implemented"
  PAUSE_REQUIRED=1
fi
if [ "$REAL_CLAUDE" = "1" ]; then
  [ "${SERGEANT_CLAUDE_TESTS:-0}" = "1" ] || fail "--real-claude needs SERGEANT_CLAUDE_TESTS=1 (token budget discipline)"
  command -v claude >/dev/null || fail "--real-claude needs an installed claude CLI"
  BACKEND=claude
  PAUSE_REQUIRED=0
  unset SGT_FAKE_SCRIPT
  # `sgt run` and `sgt respond` drive the stage inside the HTTP request, so
  # against a real model they outlast the client's ten-second default. A
  # submit that times out would be reported as "submit returned no work id",
  # which is a lie about what went wrong.
  export SGT_CLIENT_TIMEOUT_SECS="${SGT_CLIENT_TIMEOUT_SECS:-1800}"
else
  BACKEND=fake
fi

# ---------------------------------------------------------------------------
step "a developer clones a repository and prepares it for sergeant"

git init -q -b main "$REPO"
cd "$REPO"
git config user.email demo@example.invalid
git config user.name  "sergeant demo"
cat > payments.py <<'PY'
def settle(payment):
    return gateway.settle(payment)
PY
mkdir -p .sergeant/workflows/software-change/{10-implement,20-review}
cat > .sergeant/workflows/software-change/workflow.toml <<'TOML'
[workflow]
name = "software-change"
version = "1"
stages = ["10-implement", "20-review"]
TOML
echo "Implement the change." > .sergeant/workflows/software-change/10-implement/CONTEXT.md
echo "Review it independently." > .sergeant/workflows/software-change/20-review/CONTEXT.md
git add -A && git commit -qm "payment settlement worker"

say "repository: $REPO (branch main)"
say "workflow:   software-change — 10-implement → 20-review"
evidence "the workflow is content, not code: $REPO/.sergeant/workflows/software-change/"

# ---------------------------------------------------------------------------
step "they ask for work — no daemon is running yet"

say "\$ sgt run \"Add retry handling to the payment settlement worker…\""
SUBMIT="$(SGT_ORIGIN_CLIENT=claude sgt --json run \
  "Add retry handling to the payment settlement worker. Have another agent independently review it." \
  --backend "$BACKEND" 2>/dev/null)"
WORK_ID="$(printf '%s' "$SUBMIT" | json work.id)"
[ -n "$WORK_ID" ] || fail "submit returned no work id: $SUBMIT"
say "the client auto-spawned a daemon, then submitted work $WORK_ID"
ENDPOINT="$(sed -n 's/.*"endpoint": *"\([^"]*\)".*/\1/p' "$DATA_DIR/runtime.json")"
TOKEN="$(sed -n 's/.*"token": *"\([^"]*\)".*/\1/p' "$DATA_DIR/runtime.json")"
evidence "runtime descriptor (0600): $DATA_DIR/runtime.json → $ENDPOINT"

# ---------------------------------------------------------------------------
step "sergeant routed it, cut a work surface, and bound the workflow"

SHOW="$(sgt --json work show "$WORK_ID")"
STATE="$(printf '%s' "$SHOW" | json work.state)"
WORKTREE="$(printf '%s' "$SHOW" | json surface.bindings.0.worktree_path)"
BRANCH="$(printf '%s' "$SHOW" | json surface.bindings.0.work_branch)"
STAGE="$(printf '%s' "$SHOW" | json stage.stage_id)"
RESOLVED="$(printf '%s' "$SHOW" | json backend)"
say "state $STATE · stage $STAGE · backend $RESOLVED (origin affinity: claude → $RESOLVED)"
say "worktree $WORKTREE on branch $BRANCH"
[ -n "$WORKTREE" ] || fail "no worktree was recorded for $WORK_ID"
[ -n "$BRANCH" ] || fail "no work branch was recorded for $WORK_ID"
if [ "$STATE" = "completed" ]; then
  # Nothing paused, so the whole workflow ran inside the submit and the
  # surface has already been retired. Its existence is checked where it is
  # still checkable — at "the surface was retired", below.
  say "the workflow already ran to the end inside the submit, so this surface is retired"
else
  [ -d "$WORKTREE" ] || fail "the recorded worktree does not exist: $WORKTREE"
  git -C "$REPO" worktree list | grep -q "$BRANCH" || fail "git does not know about branch $BRANCH"
  evidence "git itself agrees: git -C $REPO worktree list"
fi

# ---------------------------------------------------------------------------
step "the first stage runs and stops to ask a question"

PAUSED=0
if [ "$STATE" = "needs_input" ]; then
  PAUSED=1
  PROMPT="$(sgt --json work show "$WORK_ID" | json stage.detail)"
  say "work $WORK_ID is waiting: \"$PROMPT\""
  say "the foreground client may exit here — sergeant is the one holding the work"
elif [ "$PAUSE_REQUIRED" = "1" ]; then
  fail "expected needs_input after the first stage, got $STATE"
else
  say "this backend ran the stage straight through (state $STATE) without asking."
  say "P0's real Claude adapter has no ask-the-human signal — the pause in the"
  say "default walkthrough is scripted by §37's fake backend — so this run proves"
  say "the arc without it. Everything below is unchanged."
fi
evidence "journal: $DATA_DIR/journal/ (append-only NDJSON, one line per fact)"

# ---------------------------------------------------------------------------
step "the developer answers"

if [ "$PAUSED" = "1" ]; then
  say "\$ sgt respond $WORK_ID \"yes, 3 attempts with exponential backoff\""
  sgt respond "$WORK_ID" "yes, 3 attempts with exponential backoff" >/dev/null
  say "the answer resumed the stage; the workflow ran on to the end"
else
  say "nothing was asked, so there is nothing to answer."
fi
FINAL="$(sgt --json work show "$WORK_ID")"
STATE="$(printf '%s' "$FINAL" | json work.state)"
say "state $STATE"
[ "$STATE" = "completed" ] || fail "expected completed, got $STATE"

# ---------------------------------------------------------------------------
# §39's arc is "stages", plural, and the second one is the part that makes it
# so: the review is dispatched as *another execution*, not as a later phase of
# the same agent's turn. It ran in every version of this walkthrough and was
# never printed, so the one element that distinguishes a workflow from a single
# prompt was invisible in the walkthrough that narrates it.
step "the review ran as a second, independent execution"

STAGE_RUNS="$(python3 - "$DATA_DIR/journal" "$WORK_ID" <<'PY'
import glob, json, os, sys

journal, work_id = sys.argv[1], sys.argv[2]
stages = []
for path in sorted(glob.glob(os.path.join(journal, "*.ndjson"))):
    with open(path) as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            event = json.loads(line)
            if event.get("work_id") != work_id:
                continue
            payload = event.get("payload") or {}
            if event["kind"] == "stage.entered":
                stages.append([payload.get("stage_id", "?"), "-", "-", event["seq"]])
            elif event["kind"] == "execution.started" and stages:
                execution = payload.get("execution") or {}
                stages[-1][1] = execution.get("execution_id", "?")
                stages[-1][2] = execution.get("backend", "?")
                stages[-1][3] = event["seq"]
for stage in stages:
    print("\t".join(str(field) for field in stage))
PY
)"
[ -n "$STAGE_RUNS" ] || fail "the journal records no stages for $WORK_ID"

STAGE_COUNT=0
EXECUTIONS=""
while IFS=$'\t' read -r stage execution backend seq; do
  [ -n "$stage" ] || continue
  STAGE_COUNT=$((STAGE_COUNT + 1))
  say "stage $STAGE_COUNT: $stage — execution $execution on $backend (journal seq $seq)"
  EXECUTIONS="$EXECUTIONS $execution"
done <<< "$STAGE_RUNS"

[ "$STAGE_COUNT" -ge 2 ] || fail "the workflow has two stages but only $STAGE_COUNT ran"
DISTINCT="$(printf '%s' "$EXECUTIONS" | tr ' ' '\n' | grep -c '[^[:space:]-]' || true)"
UNIQUE="$(printf '%s' "$EXECUTIONS" | tr ' ' '\n' | grep '[^[:space:]-]' | sort -u | wc -l | tr -d ' ')"
[ "$UNIQUE" = "$DISTINCT" ] \
  || fail "the stages reused an execution — the review must be its own ($EXECUTIONS)"
say "different execution ids: the review is independent work, not a later turn of the first agent"
evidence "journal: stage.entered + execution.started per stage, at the seqs above"

# ---------------------------------------------------------------------------
step "the surface was retired"

TEARDOWN="$(printf '%s' "$FINAL" | json teardown.reason)"
say "teardown: ${TEARDOWN:-recorded}"
[ -d "$WORKTREE" ] && fail "the worktree survived teardown: $WORKTREE"
say "the worktree is gone; the work branch $BRANCH is kept, because it is the work"
git -C "$REPO" rev-parse --verify --quiet "$BRANCH" >/dev/null \
  || fail "teardown removed the work branch $BRANCH"
evidence "branch retained: git -C $REPO log --oneline $BRANCH"

# ---------------------------------------------------------------------------
step "where the evidence lives"

SEGMENTS="$(find "$DATA_DIR/journal" -name '*.ndjson' | wc -l | tr -d ' ')"
EVENTS="$(cat "$DATA_DIR"/journal/*.ndjson | wc -l | tr -d ' ')"
say "journal:   $DATA_DIR/journal/ — $SEGMENTS segment(s), $EVENTS events"
[ "$EVENTS" -gt 0 ] || fail "the journal is empty"

BLOBS=0
[ -d "$DATA_DIR/blobs" ] && BLOBS="$(find "$DATA_DIR/blobs" -type f | wc -l | tr -d ' ')"
say "blobs:     $DATA_DIR/blobs/ — $BLOBS content-addressed object(s) (BLAKE3; the fake backend archives none)"

say "analytics: sgt analytics blocked_time_per_work"
sgt analytics blocked_time_per_work | sed 's/^/     /'
# Exit 0 is not an answer: the canned question prints ahead of the rows, so a
# projection that returned nothing would still look like it had spoken. The
# rows are what must exist.
sgt --json analytics blocked_time_per_work > "$WORKDIR/analytics.json" \
  || fail "the analytics query did not answer"
ANALYTICS_ROWS="$(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["rows"]))' \
  "$WORKDIR/analytics.json")"
[ "$ANALYTICS_ROWS" -gt 0 ] \
  || fail "the analytics query answered with zero rows — the projection is not being fed"
say "           $ANALYTICS_ROWS row(s) — answer kept at $WORKDIR/analytics.json"

say "graph:     GET /v1/graph/work/$WORK_ID"
GRAPH_CODE="$(curl -s -o "$WORKDIR/graph.json" -w '%{http_code}' \
  -H "Authorization: Bearer $TOKEN" "$ENDPOINT/v1/graph/work/$WORK_ID")"
[ "$GRAPH_CODE" = "200" ] || fail "the graph endpoint answered $GRAPH_CODE"
EDGES="$(json edges.0.relation < "$WORKDIR/graph.json")"
say "           200 — $(python3 -c 'import json,sys; print(len(json.load(open(sys.argv[1]))["edges"]))' "$WORKDIR/graph.json") edges, each carrying the journal seq that justifies it (first: $EDGES)"

DASHBOARD="$(sgt --json web | json url)"
DASH_CODE="$(curl -s -o "$WORKDIR/dashboard.html" -w '%{http_code}' "$DASHBOARD")"
[ "$DASH_CODE" = "200" ] || fail "the dashboard answered $DASH_CODE"
grep -q "$WORK_ID" "$WORKDIR/dashboard.html" || fail "the dashboard does not show work $WORK_ID"
say "dashboard: $DASHBOARD"
say "           200 — the fleet page names work $WORK_ID"
say "tui:       sgt   (no subcommand — same daemon, same state, no private shortcuts)"

# ---------------------------------------------------------------------------
step "the walkthrough held"
if [ "$PAUSED" = "1" ]; then
  say "submit → surface → stages → needs_input → respond → completed → retire"
else
  say "submit → surface → stages → completed → retire"
fi
say "every pointer above resolved."
