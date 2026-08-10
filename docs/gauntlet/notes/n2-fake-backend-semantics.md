# N2 Unknown U1 — fake-backend hold-open/advance semantics (measured)

Governing: `docs/gauntlet/contracts/N2.md` Unknown U1. Starting point:
`scripts/perf/README.md`'s "What the fake script means" section (P1 measured
FIFO semantics) and `src/backend/fake.rs`. This note re-measures those
semantics against a live daemon, specifically for the shape N2's
`repo-to-icm` workflow needs: a **10-stage** run in which *every* stage is
held open and then advanced by a command issued from outside the stage's own
execution (the actor-per-stage model `repo-to-icm` must run under).

Method: `target/debug/sgt` (already-built debug binary, unmodified), driven
against scratch data dirs under
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-measure/`.
Two probe workflows were authored purely for this measurement (not part of
the deliverable workflow): a 10-stage `n2-probe` and a 2-stage
`n2-waiting-probe`. Both daemons were killed (SIGTERM, graceful) at the end
of the session; `pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` was
empty on exit. No file under `src/` was modified.

Everything below marked "measured" was produced by literally running the
commands shown and reading the resulting `--json` output and the raw journal
(`data/journal/*.ndjson`). Items marked "read from source" come directly from
`src/backend/fake.rs` / `src/runtime/engine.rs` but were not separately
live-fired in this session (they follow the identical code path as something
that *was* fired live, noted per item).

---

## (a) `SGT_FAKE_SCRIPT` step → stage lifecycle mapping

`SGT_FAKE_SCRIPT` is parsed once, at daemon startup, into a `VecDeque<FakeStep>`
shared by the whole process (`FakeBackend::from_env`, read at
`Registry`/backend construction time — **not** re-read per work or per
stage). One step is popped:

- by `Backend::start` (`FakeBackend::start`) — i.e. every time the engine
  calls `enter_stage` (a stage's first attempt **or** a `retry`'s fresh
  attempt of the same stage);
- by `Backend::send` (`FakeBackend::send`) — i.e. every time `sgt respond`
  delivers input to a `needs_input` execution.

When the queue is empty, both call sites fall back to `FakeStep::complete()`
(`Self::next_step`'s `.unwrap_or_else(FakeStep::complete)`) — every stage
after the script runs out just completes.

Verb → `BackendSignal` → engine reaction (`src/runtime/engine.rs`, the match
on `observation.signal` in the stage-observation path):

| Script verb | `FakeStep`/`BackendSignal` | `stage.*` event | `work.*` event | Resulting `Work.state` | Advances via |
|---|---|---|---|---|---|
| `complete` / `complete:<summary>` | `StageCompleted` | `stage.completed` | (none directly; if a next stage exists, immediately cascades into `stage.entered`/`execution.started` for it) | `needs_input`/`waiting`/etc. of the **next** stage, or `completed` if this was the last stage | automatic — no external command needed |
| `needs_input:<prompt>` | `NeedsInput` | `stage.needs_input` | `work.needs_input` | `needs_input` | `sgt respond <id> <text>` (measured) |
| `waiting:<reason>` | `Waiting` | `stage.waiting` | `work.waiting` | `waiting` | `sgt retry <id>` (measured) — **not** `respond` |
| `blocked:<reason>` | `Blocked` | `stage.blocked` | `work.blocked` | `blocked` | `sgt retry <id>` (read from source: identical `retry` match arm as `waiting`, not separately fired live) |
| `fail:<reason>` | `Failed` | `stage.failed` | `work.failed` | `failed` (terminal for this attempt; surface torn down) | `sgt retry <id>` re-enters the stage (read from source, same `retry` match arm) |
| `hang` | `Running`, `ignores_stop: true` | — | — | stays `running`; `stop`/`interrupt` cannot kill it (native stays `Running`) | not part of the hold/advance question — this models a stuck native process, not a checkpoint |

Unknown verbs are silently dropped by `parse_script` (`_ => None`), so a typo
in a workflow author's script does not raise an error, it just shortens the
FIFO — the extraction/lint discipline in `convention.md`/`record-shapes.md`
has no equivalent guard for this, worth remembering if a later milestone
scripts `repo-to-icm`'s own fake-backend acceptance tests.

## (b) Holding a stage open and advancing it externally

There are **two structurally different mechanisms**, keyed off which signal
the stage is holding on — they are not interchangeable, and using the wrong
one is refused, not silently absorbed:

**1. `needs_input` → `sgt respond <id> <text>` (`POST /v1/work/{id}/input`)**

- Delivers input to the **same execution** that is already running
  (`Backend::send`, same `execution_id`/`native_id` throughout).
- `Engine::provide_input` requires `Work.state == NeedsInput` exactly; any
  other state is refused with a structured `EngineError::NotAwaitingInput`
  (measured below as HTTP 409, `not_awaiting_input`), never a silent no-op.
- One `respond` call both answers the hold *and* — if the popped step is
  `complete` — closes out the stage and (synchronously, inside the same
  call) opens the next one, which itself may immediately re-enter
  `needs_input` on its own hold. This is what makes a whole 10-stage chain
  driveable by exactly 10 `respond` calls (see (c)).

**2. `waiting` (and, unverified live, `blocked`/`failed`) → `sgt retry <id>` (`POST /v1/work/{id}/retry`)**

- `retry` is refused unless `Work.state` is `Failed`, `Blocked`, or
  `Waiting` (`EngineError::NotRetryable` otherwise) — it is not a general
  "unstick" command, and `respond` is refused the same way in the other
  direction (measured: a `respond` against a `waiting` work returns 409
  `not_awaiting_input`, exactly as it would against any non-`needs_input`
  state).
- `retry` re-enters the **current stage index** at `attempt + 1`, which
  calls `Backend::start` again — this **starts a brand-new execution**
  (fresh `execution_id`, fresh `native_id` for the fake backend, and for a
  real backend a fresh process/turn — not a resumed conversation). Measured
  journal evidence: `stage.entered {attempt:2, index:0}` /
  `execution.started {execution_id: "...NA3VB9EKNKQ1QYYGJP69"}`, a
  *different* execution id from the attempt-1 execution
  (`...MVK1CP8146PEZYV6MXKT`) that was holding on `waiting`.
- Because `retry`'s fresh attempt pops the *next* FIFO step, if that step is
  `complete` the stage finishes immediately inside the same `retry` call —
  in the 2-stage probe below, one `retry` call carried the work all the way
  from `waiting` at stage 0 to `completed` at stage 1, because both
  remaining script steps were `complete`.

**Consequence for `repo-to-icm`'s design:** if a stage needs to "hold open
while external work happens, then resume with that work folded back into
the *same* conversation/turn", only `needs_input`/`respond` gives that — the
execution and its accumulated context survive. `waiting`/`retry` holds the
stage open too, but resuming means literally starting the stage over as a
new attempt/new execution; whatever the first attempt had produced only
survives if it was written to the worktree (Layer 4 `output/`), not in
conversational state. This is a real semantic difference, not just two
names for the same thing, and workflow authors reaching for `waiting` should
expect a fresh actor turn on retry, not a continued one.

## (c) Exact commands: a 10-stage run, every stage held open and advanced externally

Setup (measured, this session):

```sh
SGT=/home/user/sergeant-rs/target/debug/sgt
WORKDIR=/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-measure
DATA_DIR="$WORKDIR/data"
REPO="$WORKDIR/repo"

# A 10-stage workflow named after §9.3's shape, content irrelevant to the
# probe (each CONTEXT.md is a one-line stub).
mkdir -p "$REPO/.sergeant/workflows/n2-probe"/{00-contract,10-inventory,20-harvest,30-normalize,40-classify,50-synthesize,60-draft,70-lint,80-adversarial-review,90-reconcile}
# ... workflow.toml lists all 10 stage ids in order; each dir gets a CONTEXT.md.

# One needs_input/complete pair per stage — 20 steps total, one FIFO
# consumption per START (needs_input) and one per SEND (complete).
export SGT_FAKE_SCRIPT="needs_input:hold-stage-0;complete:advanced-stage-0;\
needs_input:hold-stage-1;complete:advanced-stage-1;\
needs_input:hold-stage-2;complete:advanced-stage-2;\
needs_input:hold-stage-3;complete:advanced-stage-3;\
needs_input:hold-stage-4;complete:advanced-stage-4;\
needs_input:hold-stage-5;complete:advanced-stage-5;\
needs_input:hold-stage-6;complete:advanced-stage-6;\
needs_input:hold-stage-7;complete:advanced-stage-7;\
needs_input:hold-stage-8;complete:advanced-stage-8;\
needs_input:hold-stage-9;complete:advanced-stage-9"
```

Run (11 commands total drive all 10 stages to completion — one submit, then
exactly one `respond` per stage):

```sh
$SGT --data-dir "$DATA_DIR" --json run "…intent…" --workflow n2-probe --backend fake
# -> work.state=needs_input, stage=00-contract, stage.detail="hold-stage-0"
#    (stage 0 is HELD OPEN here — this is the external hold point)

WORK_ID=<id from above>
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 0"
# -> work.state=needs_input, stage=10-inventory, detail="hold-stage-1"
#    (stage 0 closed AND stage 1 opened+held, inside this one call)

$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 1"
# -> stage=20-harvest, detail="hold-stage-2"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 2"
# -> stage=30-normalize, detail="hold-stage-3"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 3"
# -> stage=40-classify, detail="hold-stage-4"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 4"
# -> stage=50-synthesize, detail="hold-stage-5"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 5"
# -> stage=60-draft, detail="hold-stage-6"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 6"
# -> stage=70-lint, detail="hold-stage-7"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 7"
# -> stage=80-adversarial-review, detail="hold-stage-8"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 8"
# -> stage=90-reconcile, detail="hold-stage-9"
$SGT --data-dir "$DATA_DIR" --json respond "$WORK_ID" "external answer for stage 9"
# -> work.state=completed, stage=90-reconcile, detail="advanced-stage-9"
```

**Measured result:** all 10 `respond` calls in order transitioned
`stage_id` 00-contract → 10-inventory → 20-harvest → 30-normalize →
40-classify → 50-synthesize → 60-draft → 70-lint → 80-adversarial-review →
90-reconcile, each held on `needs_input` between calls, with the final
`respond` landing `Work.state = completed`. Full raw transcript:
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-measure/transcript.txt`.

Journal confirmation (excerpt, one stage's full cycle — every later stage is
structurally identical, just with incremented ids/indices):

```
 9 stage.entered           {"attempt":1,"index":0,"stage_id":"00-contract"}
10 execution.started       {"execution":{"execution_id":"01KZNPJN15MSGPGC8XGZB09KPN", ...}}
11 stage.needs_input       {"detail":"hold-stage-0","stage_id":"00-contract"}
12 work.needs_input        {"prompt":"hold-stage-0","stage_id":"00-contract"}
13 command.accepted        {"operation":"work.submit", ...}
14 stage.input_received    {"input":"external answer for stage 0", "stage_id":"00-contract"}
15 work.resumed            {"reason":"input_received"}
16 stage.completed         {"detail":"advanced-stage-0","index":0,"stage_id":"00-contract"}
17 execution.stopped       {"execution_id":"01KZNPJN15MSGPGC8XGZB09KPN", "reason":"stage completed"}
18 stage.entered           {"attempt":1,"index":1,"stage_id":"10-inventory"}
19 execution.started       {"execution":{"execution_id":"01KZNPK1Z1T4JFXZKJX56B6ZH0", ...}}
20 stage.needs_input       {"detail":"hold-stage-1","stage_id":"10-inventory"}
...
99 work.completed          {"stages":10}
100 surface.torn_down      {...}
```

Note each stage gets its **own** `execution_id`/`native_id` even though the
hold-and-answer within one stage reuses the same execution — the engine
always retires the stage's execution on `stage.completed`
(`stop_execution`, `execution.stopped`) and starts a fresh one for the next
stage (`enter_stage`). "One work in flight" (per `scripts/perf/README.md`)
is required for this to be deterministic: the FIFO is global to the daemon
process, shared across every work it runs, so a second concurrent work
submitted against the same daemon would steal steps out of this sequence.

### The `waiting`/`retry` alternative (measured separately, 2-stage probe)

```sh
export SGT_FAKE_SCRIPT="waiting:hold-via-waiting;complete:stage0-done-after-retry;complete:stage1-done"
$SGT --data-dir "$DATA_DIR2" --json run "…" --workflow n2-waiting-probe --backend fake
# -> work.state=waiting, stage=00-first, detail="hold-via-waiting"

$SGT --data-dir "$DATA_DIR2" --json respond "$WORK_ID2" "trying to answer a waiting stage"
# -> sgt: 409: work <id> is waiting, not needs_input; nothing is waiting for an answer
#    (measured refusal — respond does NOT work on a waiting hold)

$SGT --data-dir "$DATA_DIR2" --json retry "$WORK_ID2"
# -> work.state=completed  (one retry call ran BOTH remaining `complete` steps:
#    attempt 2 of stage 00-first popped "complete:stage0-done-after-retry" and
#    completed immediately, cascading straight into stage 10-second, which
#    popped "complete:stage1-done" and finished the work — all inside this
#    single retry call)
```

Journal confirmation:

```
11 stage.waiting        {"detail":"hold-via-waiting","stage_id":"00-first"}
12 work.waiting         {"reason":"hold-via-waiting","stage_id":"00-first"}
14 command.rejected     {"operation":"work.respond","result":{"error":{"code":"not_awaiting_input", ...}}}
15 work.resumed         {"reason":"retry","stage_id":"00-first"}
16 stage.entered        {"attempt":2,"index":0,"stage_id":"00-first"}
17 execution.started    {"execution":{"execution_id":"01KZNPNA3VB9EKNKQ1QYYGJP69", ...}}
                          # different execution_id from attempt 1
                          # (01KZNPMVK1CP8146PEZYV6MXKT) — a fresh native
                          # context, not a resumed one
18 stage.completed       {"detail":"stage0-done-after-retry", ...}
20 stage.entered         {"attempt":1,"index":1,"stage_id":"10-second"}
22 stage.completed       {"detail":"stage1-done", ...}
24 work.completed        {"stages":2}
```

Full raw transcript:
`/tmp/claude-0/-home-user-sergeant-rs/fff58c4e-2990-5de3-a53f-f5e2c669c45f/scratchpad/n2-measure/transcript-waiting.txt`.

## Summary for `repo-to-icm` workflow design

- For every stage `repo-to-icm` wants a fresh per-run agent turn to hold
  open pending an external decision and then be answered *in the same
  conversation*, script/drive it as `needs_input` + `sgt respond`. This is
  the only mechanism that preserves the execution/native context across the
  hold.
- `waiting`/`blocked`/`failed` + `retry` also hold the work open externally
  and are advanced externally, but "advance" there means "start the stage
  over as a new attempt, new execution" — appropriate for "come back later
  and try again from scratch", not for "here is the answer to your
  question."
- A scripted multi-stage `SGT_FAKE_SCRIPT` run is only deterministic with
  exactly one work in flight per daemon (confirmed consistent with
  `scripts/perf/README.md`); this matters for any acceptance test the N2
  workflow's own lint/CI harness writes against the fake backend.
- Neither mechanism is specific to stage *count* — the FIFO/queue model
  scales to 10 stages (or any N) exactly as it does to 2, provided the
  script supplies the right number of steps per stage (one per `start`, one
  more per `send` if the stage is held on `needs_input`).

## Session hygiene

Both daemons spawned for this measurement (`data/`, `data2/`) were sent
SIGTERM and confirmed exited before this note was written.
`pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` returned empty at end
of session. No file under `src/` was read for anything but citation, and
none was modified. `reference/` and `reference-corpus/` were not touched.
