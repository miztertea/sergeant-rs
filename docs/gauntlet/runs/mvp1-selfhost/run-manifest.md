# MVP-1 self-hosting checkpoint — issue #50 fixed via a sergeant Work

Contract acceptance item: at least one real fix executed AS a sergeant Work
against this repo, operated as a human user would. This run submits
`implement` on issue #50 (the perf harness's `commit` field pinning bug)
against the MVP-1 binary (`target/debug/sgt`, R-MVP1-7's turn envelope
included), real `claude` backend, and observes it to completion read-only.

## Setup summary

- **Binary:** `/home/miztertea/sergeant-rs/target/debug/sgt` (debug build,
  `cargo build`, HEAD `71bb638` at clone time — "N4 adjudicated
  flag-and-proceed").
- **Subject clone:**
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp1-selfhost/subject`,
  branch `cerberus/mvp-1`, cloned from `/home/miztertea/sergeant-rs`.
  `ISSUE-50.md` added (`c672d6d`), `sergeant.toml` added declaring the
  `mvp1selfhost` profile (`87eddd8`, then schema-corrected in `00f943f` —
  see Friction below).
- **Data dir:**
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp1-selfhost/data`
- **Daemon:** started with `PATH="$HOME/.cargo/bin:$PATH"` in its
  environment (E2 workaround, cerberus.md), data dir above, pid `4065997`,
  endpoint `http://127.0.0.1:45147`.
- **Profile (`sergeant.toml`, final form):**

  ```toml
  [estate]
  name = "mvp1selfhost-subject"

  [[repo]]
  name = "mvp1selfhost-subject"
  path = "."

  [[profile]]
  name = "mvp1selfhost"
  backend = "claude"

  [profile.options]
  permission_mode = "bypassPermissions"
  ```

  `sgt doctor` confirms: `[ok  ] permission_mode mvp1selfhost=bypassPermissions`.
- **Workflow:** `implement` v2 (`10-implement-with-tdd`, `30-review`), the
  repo's own admitted workflow, discovered from the cloned `.sergeant/workflows/implement`.
- **Backend:** `claude` (real CLI, `claude 2.1.228`).
- **Turn envelope:** no CLI flag or API field exists to set a per-submission
  turn cap (`sgt run --help` has no such option; `POST /v1/work` has no such
  field — grep-confirmed against `src/api.rs`). The daemon-wide default,
  `runtime::engine::DEFAULT_TURN_CAP = 12`, was used unmodified — it already
  equals the requested cap, so no `SGT_TURN_CAP` override was needed. Noted
  as friction (below): the cap is a daemon-startup knob, not a
  submission-time one, and no client surface reports it back at all.
- **Submit command:**

  ```
  sgt run "Fix issue #50 (see ISSUE-50.md): pin the perf summary commit
  field to the tested binary's identity captured once at
  perf_require_binary time, not a live rev-parse per scenario.
  Scripts-only change in scripts/perf/common.sh; update any scenario that
  reads it; keep the fix minimal." \
    --workflow implement --backend claude --profile mvp1selfhost --json \
    --data-dir "$DATA"
  ```

- **Work id:** `01KZTDYAK8WYGCK8D95NTV6CGZ`, branch
  `sergeant/01KZTDYAK8WYGCK8D95NTV6CGZ`.
- **Observation:** read-only polling of `sgt work show --json` plus
  journal-derived cumulative cost, intended every ~60s, with a stated
  operator budget of $4 / 45 min wall (cancel-on-breach). See "Budget
  arithmetic" below for why the $4 guard never got a chance to fire.

## Timeline (from `journal_full.ndjson`)

| seq | kind | timestamp | note |
|---|---|---|---|
| 1 | `daemon.started` | 07:27:33.946Z | |
| 4 | `work.submitted` | 07:28:13.929Z | |
| 8 | `work.started` | 07:28:14.051Z | |
| 9 | `stage.entered` (`10-implement-with-tdd`) | 07:28:14.051Z | |
| 12 | `execution.started` (turn 1) | 07:28:14.052Z | |
| 42 | `conversation.turn.grammar_unmeasured` | 07:30:15.619Z | ask-withdrawal, same shape as runB2 |
| 43 | `conversation.turn.ended` | 07:30:15.619Z | turn 1 ends, `result_envelope: true` |
| 44 | `usage.updated` | 07:30:15.619Z | turn 1 cost **$1.6763739999999998** |
| 45 | `stage.completed` (`10-implement-with-tdd`) | 07:30:15.772Z | **autonomous — no client command** |
| 47 | `stage.entered` (`30-review`) | 07:30:15.773Z | |
| 50 | `execution.started` (turn 2) | 07:30:15.776Z | |
| 305 | `conversation.turn.ended` | 07:42:22.640Z | turn 2 ends, `result_envelope: true` |
| 306 | `usage.updated` | 07:42:22.640Z | turn 2 cost **$9.056980500000002** |
| 307 | `stage.completed` (`30-review`) | 07:42:22.795Z | **autonomous — no client command** |
| 309 | `work.completed` | 07:42:22.796Z | `{"stages": 2}` |
| 310 | `surface.torn_down` | 07:42:22.840Z | `disposition: removed`, `clean: true` |

**No client command exists anywhere between seq 13 (`command.accepted`,
the original submit) and the end of the journal at seq 310.** Both stages
advanced, and the whole Work completed, entirely under the settle driver —
zero operator intervention, matching runB2's headline finding on the same
engine.

Total wall clock, submit to completion: **848.9s (≈14m9s)** — well inside
the 45-minute operator budget.

## Cost and turn-envelope evidence

Two turns spawned, both completing normally (`is_error: false`,
`stop_reason: "end_turn"`):

| Turn | Stage | `usage.updated` seq | `total_cost_usd` | internal `num_turns` | internal `duration_api_ms` |
|---|---|---|---|---|---|
| 1 | `10-implement-with-tdd` | 44 | $1.6763739999999998 | 14 | 120,678 |
| 2 | `30-review` | 306 | $9.056980500000002 | 12 | 1,616,190 |
| **Total** | | | **$10.7334** | | |

**Turn count vs. envelope: 2 of 12.** Nowhere near the cap — no
`turn envelope exhausted` blocking, no `Engine::extend_turn_envelope` call,
no ceiling interrupt (both turns finished well inside the 15-minute
`DEFAULT_TURN_CEILING`; turn 2's engine-visible wall time was ~12m7s).

**Envelope client-visibility gap (new finding, not previously logged in
the dogfood run):** the engine's own turn counter,
`WorkRun::turns_spawned` (`src/runtime/projection.rs:408`), is **never
read anywhere in `src/api.rs`** (grep-confirmed) — it does not appear in
`sgt work show --json`, `sgt status`, or any other client-facing surface.
The only way to learn "this Work spawned 2 of its 12 allotted turns" is to
open the raw journal and count `execution.started`/`stage.resumed`
(or `conversation.turn.ended`) events by hand, exactly as this report had
to. The engine enforces the envelope correctly and journals every fact
needed to reconstruct it, but **no client can ask "how much envelope is
left?"** — a sharper, envelope-specific version of the dogfood run's E7
(transcript legibility) finding.

## Budget arithmetic — why the $4 operator guard never fired

Stated operator budget: cancel if cumulative journaled cost exceeds $4, or
wall clock exceeds 45 minutes. Actual outcome: the Work completed at
**$10.7334 total — 2.68x over the $4 guard** — before any cancellation
could happen. This is not a bug in the observation loop; it reproduces
runB2's L16 finding exactly, one level up (an *external* operator guard
this time, not the engine's):

- Cost is only observable at **per-turn granularity** — one `usage.updated`
  event per completed `conversation.turn.ended`, nothing in between.
- Turn 1 landed at $1.68 (seq 44, 07:30:15.619Z) — under budget, so the
  poll loop's next 60s tick correctly continued observing.
- Turn 2 then ran for **~12 minutes with zero incremental cost signal**.
  Its `usage.updated` (seq 306) and the Work's `work.completed` (seq 309)
  landed **156ms apart**, both inside a single burst of journal writes at
  07:42:22.6–.8Z. The poll loop's next scheduled tick (60s cadence)
  landed at 07:42:53Z — by which point `state` was already `completed`
  and `cost` was already `$10.73` in the very same read. **There was no
  wall-clock moment at which the journal showed cost over $4 while the
  Work was still cancelable** — the overrun and the completion arrived
  as the same atomic fact.
- A tighter poll interval would not have helped: the signal that would
  need to fire *mid-turn* (turn 2's cost climbing past $4 partway through
  its ~12 minutes) does not exist. The adapter reports cost once, at
  turn end, never incrementally. This is the same conclusion runB2 drew
  for the engine's own (nonexistent) sub-turn budget guard, now confirmed
  to bind an *external* operator's guard just as tightly.
- Secondary finding: turn 2's internal `num_turns: 12` (its own nested
  agentic steps — the reviewing actor delegated to a code-review pass
  covering "eight finder angles" per its own stage-completion report) cost
  $9.06 by itself, well past the $3.21 single-turn precedent
  `DEFAULT_TURN_CAP`'s docstring reasons from (`src/runtime/engine.rs:787-793`,
  "~$38 bound"). One measured turn here alone is 2.8x that precedent,
  meaning the doc comment's illustrative $38 figure for a 12-turn cap is
  itself an underestimate on this evidence — worth a follow-up note in
  the engine source, out of scope for this checkpoint to fix.

## Fix correctness — grade: CORRECT

The Work's own stage-completion report (`stage.completed`, seq 307)
describes reviewing and repairing its own first cut before committing —
verified independently against the actual diff and against a live
reproduction of issue #50's exact failure mode.

**The diff** (`fix.diff`, `scripts/perf/common.sh` only, +42/−2, commit
`916650b` "perf harness: pin the summary commit field once per run, with
the binary's mtime (issue #50)", `Fixes #50` trailer):

- `perf_require_binary` now captures `git rev-parse HEAD` into `PERF_COMMIT`
  **exactly once** (guarded by `[ -z "${PERF_COMMIT:-}" ]`) and **exports**
  it, so scenario scripts spawned as subprocesses by `run-all.sh` inherit
  the same pin instead of each re-reading `HEAD` at their own start time —
  this is the literal cross-scenario drift issue #50 reported.
- A failed `rev-parse` warns and is never cached/exported as `unknown`, so
  a transient failure doesn't poison the whole matrix (an edge the
  original bug didn't have, introduced correctly rather than regressed).
- Per the issue's explicit prescription, the binary's own mtime is now
  captured alongside (`PERF_BIN_MTIME`, written to every summary JSON as
  `binary_mtime` and to `environment.txt`), with a startup warning when the
  binary predates `HEAD`'s commit time — the exact condition that made the
  original anomaly need manual mtime forensics to diagnose.
- `perf_init`'s `perf_kv commit` and `perf_environment`'s `commit:` line
  both switched from live `git rev-parse` calls to reading the pinned
  `$PERF_COMMIT` — the two sites the issue named.
- No scenario script (`s1`–`s7`, `idle-baseline`) reads `PERF_COMMIT`
  directly (grep-confirmed, both by the Work's own report and
  independently here), so "update any scenario that reads it" required no
  changes — correctly recognized as already satisfied via `perf_init`.

**Independent live verification** (this operator, not the Work): checked
out the fix commit into a throwaway worktree, sourced the fixed
`common.sh`, and reproduced issue #50's exact scenario —

```
$ . scripts/perf/common.sh; perf_require_binary   # simulates run-all.sh's one call
parent PERF_COMMIT=916650bb3c080737b2666f4205d813fb0f18fadf
$ bash -c '. scripts/perf/common.sh; perf_require_binary; echo scenario1 sees $PERF_COMMIT'
scenario1 sees PERF_COMMIT=916650bb3c080737b2666f4205d813fb0f18fadf
$ git commit --allow-empty -m "simulated concurrent commit mid-matrix"   # the confound
new HEAD is now: 6aca34575e95fb2637fefd1bb6edf10fec0e534b
$ bash -c '. scripts/perf/common.sh; perf_require_binary; echo scenario2 sees $PERF_COMMIT'
scenario2 sees PERF_COMMIT=916650bb3c080737b2666f4205d813fb0f18fadf
```

Scenario 2 (spawned after a concurrent commit landed mid-"matrix," exactly
the 2026-08-11 baseline-run confound) still reports the **original pinned
commit**, not the new `HEAD` — the precise failure mode in issue #50 is
gone. `bash -n` syntax-checked clean on the fixed file.

(Operator note: this verification worktree briefly polluted the work
branch with the throwaway "simulated concurrent commit" test commit while
checked out on the same ref — caught immediately via `git log`, corrected
with `git branch -f sergeant/01KZTDYAK8WYGCK8D95NTV6CGZ 916650b…` before
anything was cherry-picked. `fix.diff` and `fix-commit.txt` in this
directory were captured from the corrected ref and are the real fix only.)

**Scope discipline:** the diff touches only `scripts/perf/common.sh`,
matching "scripts-only change" and "keep the fix minimal." Two things the
Work explicitly declined to do, and said so in its report: it didn't add a
test (correctly citing that the task scoped it to `common.sh`, and
CLAUDE.md's L7 test-requirement would need a new test file, which is a
real, self-identified tension the Work flagged rather than silently
worked around); and it left `docs/perf/baseline-cerberus-2026-08-11.md`'s
present-tense description of the bug unedited, correctly calling that a
docs change outside scripts-only scope.

**Verdict: the fix is correct, minimal, scoped as instructed, and
independently reproducible as closing the exact issue.**

## Envelope / process evidence for the MVP-1 acceptance item itself

- The Work ran through both `implement` stages autonomously, with real
  committed artifacts at every stage (not stubs) — same shape as runB2's
  headline finding, now on a *scripts-only, real backlog issue* rather
  than an internal N-series measurement task.
- `conversation.turn.grammar_unmeasured` fired once (seq 42, turn 1 only),
  same withdrawal shape as runB2/cerberus.md's a5 finding (`post_turn_summary`
  absent on this host's CLI auth mode) — consistent, not a new anomaly.
- Teardown was clean (`surface.torn_down`, `clean: true`,
  `disposition: removed`); the fix commit survives independently on
  `sergeant/01KZTDYAK8WYGCK8D95NTV6CGZ` in the subject's git object
  database.

## Friction log (operator's stumbles, in order encountered)

1. **`sergeant.toml` schema drift from the runB2 precedent.** Copying
   runB2's exact `[workspace]` / `[[repository]]` shape verbatim produced
   two sequential `sgt doctor` warnings: `[workspace]` is legacy (now
   `[estate]`), and `[[repository]]` is legacy (now `[[repo]]`). Both
   remedies were named directly in the warning text and fixed in one
   iteration each, but a doc precedent barely a day old (runB2,
   2026-08-11) was already stale — this is the exact "config/profile
   discoverability" gap (E5) the 2026-08-11 dogfood run flagged, now
   measured as concrete schema churn rather than "undiscoverable."
2. **No submission-time turn-envelope control.** Neither `sgt run --help`
   nor the `POST /v1/work` body accepts a turn-cap override; the only
   knobs are daemon-startup (`SGT_TURN_CAP` env var / `DaemonConfig`) or
   post-hoc (`sgt extend <id> <n>` after a block). The task asked to "set
   turn cap 12" — satisfied only because the daemon-wide default already
   equals 12; a task wanting a *different* cap for one submission has no
   CLI/API path today.
3. **The turn envelope is invisible after submission.** `WorkRun::turns_spawned`
   exists, is correctly maintained by the engine, and journals everything
   needed to reconstruct it — but no client surface (`sgt work show`,
   `sgt status`) ever reports it. "How many turns has this Work used, out
   of its cap?" is answerable only by hand-counting raw journal events,
   which is what this report had to do.
4. **An external operator's dollar budget guard is structurally
   unenforceable at 60s-poll granularity**, for the same reason the
   engine's own (nonexistent) sub-turn guard is: cost lands once per turn,
   atomically, often together with the Work's own completion. A ~$9 single
   turn made the stated $4 guard moot before the first over-budget poll
   could even happen — not a mistake in the polling design, a hard
   ceiling on what *any* per-turn-granularity observer can do (matches
   runB2's L16 finding, now reproduced from the outside).
5. **Transcript/cost evidence lives only in blob-hash-addressed journal
   payloads.** Decoding the two turns' full transcripts required grep-ing
   the journal for `usage.updated`/`conversation.turn.ended`, resolving
   `raw: "b3:<hash>"` fields, and reading straight from
   `data/blobs/b3/<hash>` — the same E7 (no `sgt work transcript`) gap the
   2026-08-11 dogfood run already logged, reproduced here identically.

## Files in this directory

- `run-manifest.md` — this file
- `journal_full.ndjson` — full journal, daemon start (seq 1) through
  `surface.torn_down` (seq 310)
- `submit-response.json` — the `sgt run --json` submission response
- `work-show-final.json` — final `sgt work show --json` at completion
- `poll.log` — the operator's read-only observation log (60s cadence)
- `turn1_10-implement-with-tdd.stream-json.ndjson` — turn 1's raw
  stream-json transcript (79 lines), decoded from blob
  `b3:d0e4117b…`
- `turn2_30-review.stream-json.ndjson` — turn 2's raw stream-json
  transcript (614 lines), decoded from blob `b3:430e4ec4…`
- `fix.diff` — the fix, `cerberus/mvp-1..sergeant/01KZTDYAK8WYGCK8D95NTV6CGZ`
  (`scripts/perf/common.sh` only)
- `fix-commit.txt` — `git log --format=fuller` of the fix commit `916650b`
- `worktree-output/common.sh` — the fixed file in full, recovered from the
  work branch (the worktree itself was torn down cleanly on completion)
