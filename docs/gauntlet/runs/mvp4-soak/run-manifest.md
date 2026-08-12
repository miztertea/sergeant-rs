# MVP-4 — #19 real-Claude soak, redirected mid-run to a small multi-stage proof

Governing: issue #19 (real-Claude soak evidence); `docs/gauntlet/notes/
mvp-bucketing-2026-08-11.md` (defines "the real #19 soak" as multi-hour,
real Claude, envelope-guarded, with the MVP-2 Docker verify stage
exercised); prior evidence `docs/gauntlet/runs/runB2/run-manifest.md`
(settle driver fired live twice on `repo-to-icm`, 13 turns, canceled) and
`docs/gauntlet/runs/mvp1-selfhost/run-manifest.md` (`implement` 2/2 stages
completed autonomously, $10.73, 14 min).

**This session started as the originally-briefed heavy soak
(`repo-to-icm`, 11 stages including the real `kind = "execute"` Docker
stage `65-self-check`) and was redirected mid-run by the orchestrator**
after run 1's `00-contract` turn hit its 300s ceiling: "repo-to-icm is the
heaviest workflow in the library and the soak's information doesn't need
its volume — the adapter facts... come from any multi-stage run." Run 1
was canceled (kept as interrupt/ceiling evidence, not discarded) and
replaced with a small `implement` run. Both runs are real `claude`
backend, `sonnet`-pinned, `bypassPermissions`, same turn envelope
(`--turns 25 --ceiling-secs 300`).

## Setup summary (shared by both runs)

- **Binary:** `/home/miztertea/sergeant-rs/target/debug/sgt`, `cargo
  build` at `cerberus/mvp-1` HEAD `29332d8` ("MVP-4: perf re-baseline of
  assembled MVP-1..3 product + coverage measurement").
- **Subject clone:**
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp4-soak/subject`,
  cloned from `/home/miztertea/sergeant-rs` at `cerberus/mvp-1`. Three
  fixture commits added on top (final `8e97472`):
  - `90aa2c0` — `vendor/tiny-deploy-tool/`, a small, genuinely-agreeing
    vendored fixture (a runbook, a `deploy.sh` matching it step-for-step,
    a `test_deploy.sh` smoke test, both verified to pass by hand before
    submission) — bounded subject matter for `repo-to-icm`, chosen so the
    workflow would have real procedural knowledge to decompose without an
    unbounded scope.
  - `2f3f288` — `vendor/tiny-deploy-tool/UPSTREAM.md`, provenance record
    pinning the vendored subtree to `90aa2c0` per `00-contract`'s
    vendored-subtree resolution rule.
  - `8e97472` — `sergeant.toml` estate config (below).
- **Data dir:**
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp4-soak/data`
- **Daemon:** started with `PATH="$HOME/.cargo/bin:$PATH"` in its
  environment (E2), pid `1695302`, endpoint `http://127.0.0.1:36939`.
  `sgt doctor` before submission: all checks `[ok]`, including
  `permission_mode mvp4soak=bypassPermissions`.
- **Profile (`sergeant.toml`):**

  ```toml
  [estate]
  name = "mvp4soak-subject"

  [[repo]]
  name = "mvp4soak-subject"
  path = "."

  [[profile]]
  name = "mvp4soak"
  backend = "claude"
  default_model = "sonnet"

  [profile.options]
  permission_mode = "bypassPermissions"
  ```

- **Backend:** `claude` (real CLI, `claude 2.1.229`).
- **Turn envelope:** `--turns 25 --ceiling-secs 300` on every submission
  (MVP-3's per-submission override, not the daemon-wide default).

## Run 1 — `repo-to-icm` (11 stages incl. the Docker `65-self-check` stage), canceled at stage 1/11

Evidence: `run1-repo-to-icm-canceled/` (`submit-response.json`,
`journal.ndjson` — journal state at cancel, `cancel-response.json`,
`transcript.json`, `poll.log`, `wip-output/contract.md`).

- **Work id:** `01KZVYA5RZYAZF5M7CTFPAH0ZW`, branch
  `sergeant/01KZVYA5RZYAZF5M7CTFPAH0ZW`.
- **Intent:** decompose `vendor/tiny-deploy-tool/` (scope explicitly
  bounded to that subtree) through all 11 stages into a draft ICM
  package.
- **Measured timeline (journal timestamps, `journal.ndjson`):**

  | seq | kind | timestamp |
  |---|---|---|
  | 5 | `work.submitted` | 2026-08-12T21:33:33.855Z |
  | 12 | `execution.started` (turn 1, `00-contract`) | 2026-08-12T21:33:33.980Z |
  | 128 | `execution.turn_ceiling_interrupted` | 2026-08-12T21:38:34.069Z |
  | 129 | `conversation.turn.ended` | 2026-08-12T21:38:34.091Z |
  | 130 | `work.canceled` (client `sgt cancel`) | 2026-08-12T21:40:57.916Z |
  | 131 | `stage.canceled` | 2026-08-12T21:40:57.916Z |
  | 132 | `execution.stopped` | 2026-08-12T21:40:57.917Z |
  | 133 | `surface.torn_down` | 2026-08-12T21:40:57.978Z |

- **Ceiling-secs enforcement fired live and correctly**, unprompted:
  `execution.turn_ceiling_interrupted` at seq 128 (21:38:34.069Z),
  `ceiling_secs: 300.0`, **300.089s** after turn 1's `execution.started`
  (seq 12, 21:33:33.980Z) — the `--ceiling-secs 300` flag is not
  decorative; the engine killed the turn itself, within 0.089s of the
  configured bound. The
  work then sat interruptible (no automatic retry had fired yet) until
  the orchestrator's `sgt cancel` at 21:40:57Z, ~2m23s later.
- **Process evidence:** `poll.log` shows the real `claude -p ... --model
  sonnet --permission-mode bypassPermissions --session-id
  a8289e80-...` subprocess (pid 1695538, parented by daemon pid 1695302)
  live across three samples (21:35–21:38Z), then `proc=none` at
  21:39:40Z — consistent with the ceiling interrupt at 21:38:34Z having
  already killed it. (First sample, 21:34:18Z, is a false positive from
  an early cut of the poll script matching its own `grep` invocation
  against the scratch path's `claude-1001` substring — caught and fixed
  before the next sample by keying evidence off child-of-daemon-pid +
  session-id match instead of a bare `claude` substring; left in
  `poll.log` for transparency rather than scrubbed.)
- **Cancel/interrupt teardown, measured clean:** `sgt cancel` response
  (`cancel-response.json`) shows `stage.status: "canceled"`, `detail:
  "work canceled"`, `teardown.clean: false` / `disposition:
  "retained_dirty"` — the worktree was kept, not silently discarded, with
  its one WIP file: `.sergeant/workflows/repo-to-icm/00-contract/output/
  contract.md`. That file (`wip-output/contract.md`) is genuinely good
  partial work: it correctly identified the vendored-subtree case (vs.
  live-checkout), correctly read the pinned revision from
  `UPSTREAM.md` rather than attempting `git rev-parse` inside a `.git`-
  less subtree, and cross-checked it against the outer repo's own commit
  log. **Envelope at cancel: 1/25 turns spawned** — nowhere near the cap;
  the ceiling, not the cap, is what fired.
- **Cost-visibility gap reconfirmed (matches dogfood-2026-08-11's E1 and
  Run B2's same finding):** zero `usage.updated` events exist anywhere in
  this run's journal — the interrupted turn made real, billable API calls
  (five `tool.requested`/`tool.completed` pairs, one
  `conversation.assistant.completed`) but the ceiling interrupt killed it
  before a `usage.updated` could be recorded. Telemetry-only note, not a
  guard failure: nothing in this soak's brief guards on cost.
- **Process table clean after cancel:** no `claude -p` subprocess remained
  (verified immediately after the cancel response).

**Docker `65-self-check` stage: not reached** (canceled at stage 1/11,
the execute stage is stage 8/11) — see "Docker-stage verdict" below.

## Run 2 — `implement` (2 stages), completed autonomously

Evidence: `run2-implement-completed/` (`submit-response.json`,
`journal_full.ndjson` — full daemon-lifetime journal at completion,
`journal-this-work-only.ndjson` — this work's events only,
`timeline.jsonl`, `transcript.json`, `transcript.txt`, `final.diff`,
`poll.log`, `work-show-final.json`).

- **Work id:** `01KZVYSC0JE7WYWBTGHN82KDSS`, branch
  `sergeant/01KZVYSC0JE7WYWBTGHN82KDSS`.
- **Intent:** add a short "Version note" section to
  `vendor/tiny-deploy-tool/README.md` pointing at `UPSTREAM.md`, scoped to
  that one file, verifying `tests/test_deploy.sh` still passes.
- **Workflow:** `implement` v2, stages `10-implement-with-tdd` →
  `30-review` (the repo's own admitted 2-stage workflow — same one
  `mvp1-selfhost` used).

### Measured timeline (journal `timestamp` field, `journal-this-work-only.ndjson`)

| seq | kind | timestamp |
|---|---|---|
| 135 | `work.submitted` | 2026-08-12T21:41:51.762Z |
| 140 | `stage.entered` (`10-implement-with-tdd`) | 2026-08-12T21:41:51.876Z |
| 142 | `execution.started` (turn 1) | 2026-08-12T21:41:51.876Z |
| 159 | `usage.updated` (turn 1) | 2026-08-12T21:42:03.417Z |
| 160 | `stage.completed` (`10-implement-with-tdd`) | 2026-08-12T21:42:03.497Z |
| 162 | `stage.entered` (`30-review`) | **2026-08-12T21:42:03.497Z** |
| 164 | `execution.started` (turn 2) | 2026-08-12T21:42:03.499Z |
| 176 | `usage.updated` (turn 2) | 2026-08-12T21:42:14.748Z |
| 177 | `stage.completed` (`30-review`) | 2026-08-12T21:42:14.781Z |
| 179 | `work.completed` | 2026-08-12T21:42:14.781Z |
| 180 | `surface.torn_down` | (immediately after) |

**Total wall clock, submit to completion: 23.019s** (21:41:51.762Z →
21:42:14.781Z, both journal timestamps).

### Settle-driver-live verdict: CONFIRMED, a third independent measurement

`stage.entered` for `30-review` (seq 162) carries the **identical
timestamp** to `stage.completed` for `10-implement-with-tdd` (seq 160,
both `21:42:03.497Z`) — the engine advanced the moment the first turn's
result landed, with no gap. **The only `command.accepted` event in this
work's entire journal is seq 143, the original `work.submit` at
21:41:51.877Z.** No `command.accepted` for anything else appears between
submission and `work.completed` — grep-confirmed against
`journal-this-work-only.ndjson`. Zero client commands drove the stage 1 →
stage 2 → completion cascade; it is the same settle-driver mechanism
Run B2 measured on `repo-to-icm` (13 turns, 2 cascades, canceled before
completion) and `mvp1-selfhost` measured on this same `implement`
workflow (2/2 stages, 14 min, $10.73) — now a third, independent,
**completed** confirmation, on the fastest and cheapest of the three.

### Envelope evidence

**2 of 25 turns spawned** (`work-show-final.json`: `envelope.turns_spawned:
2`) — one `execution.started`/`execution.reserved` pair per stage (seq
141–142, 163–164), confirmed by direct count against
`journal-this-work-only.ndjson`. Nowhere near the cap; the 300s ceiling
was also never approached (both turns finished in under 12 seconds each —
turn 1: 21:41:51.876Z → 21:42:03.417Z ≈ 11.5s; turn 2: 21:42:03.499Z →
21:42:14.748Z ≈ 11.2s). Across this daemon's whole lifetime, including run
1's canceled turn, **3 turns were spawned in total** (1 + 2) — the figure
the orchestrator's own journal read cited.

### Model-pin and cost evidence (telemetry only)

Both `usage.updated` events carry `"model_pin":{"model":"claude-sonnet-5",
"verdict":"honored"}` — the actor ran on **sonnet**, not fable, confirmed
by the adapter's own pin-verification field, not assumed from the launch
flag.

| Turn | Stage | `usage.updated` seq | `total_cost_usd` |
|---|---|---|---|
| 1 | `10-implement-with-tdd` | 159 | $0.1538452 |
| 2 | `30-review` | 176 | $0.1378297 |
| **Total (run 2)** | | | **$0.2916749** |

Run 1's interrupted turn recorded **$0** journaled cost (the E1 gap
above) despite making real billable calls — reported here as a known
measurement blind spot, not as "run 1 was free."

### Work quality (readable transcript, no tangle)

`final.diff` — the only change made anywhere in the worktree:

```diff
diff --git a/vendor/tiny-deploy-tool/README.md b/vendor/tiny-deploy-tool/README.md
+## Version note
+
+This fixture has no version scheme of its own. Its provenance and pinned
+revision are tracked in `UPSTREAM.md` — see that file for details.
```

Exactly the file requested, nothing else touched (`git diff --stat`
confirms one file). The `30-review` stage's own recorded `detail` field
(`work-show-final.json`): *"The 'Version note' section was already
present in the working tree, correctly scoped to
`vendor/tiny-deploy-tool/README.md` only (confirmed via `git status`/`git
diff` — no other files touched), and it points to `UPSTREAM.md` as
requested. `tests/test_deploy.sh` passes (PASS, exit 0)."* — the review
stage independently re-ran the test rather than trusting the implement
stage's say-so. `transcript.txt`/`transcript.json` decode both turns'
full conversations from the journal's blob store via `sgt work
transcript` (MVP-3), human-readable end to end.

**Teardown:** `disposition: "retained_dirty"` — branch
`sergeant/01KZVYSC0JE7WYWBTGHN82KDSS` retained with the one-file diff
uncommitted in the worktree (finalize/promote to the base branch is not
this workflow's job, matching E6 from `dogfood-2026-08-11`).

## Docker-stage verdict

**Not exercised in this soak.** Run 1 was canceled at stage 1/11, three
stages before `65-self-check` (stage 8/11, the only `kind = "execute"`
stage in the library — confirmed by grep across every
`.sergeant/workflows/*/workflow.toml`); run 2's workflow (`implement`) has
no execute stage at all (2 actor stages only). This is a real gap in
*this specific soak's* evidence, not a gap in the product's overall
Docker record — cited instead of re-measured, per the redirect:

- **`tests/m7_docker_executor.rs`** — 16 `#[test]`-annotated cases against
  a real Docker daemon (not the fake backend): exit-code capture,
  workspace read/write access both ways, isolation-escape-hatch refusal,
  `network = "none"` enforcement, name-collision/foreign-container
  fail-closed handling, resume/observe/stop/interrupt lifecycle, pinned-
  image-identity on retry, host-worktree-ownership of container-written
  files/directories (the MVP-2 D3 fixer-pass fix), and a real bind-mount
  round-trip probe.
- **MVP-5's own exit bar** (`docs/gauntlet/notes/
  mvp-bucketing-2026-08-11.md` §A7): *"the assembled-product ship gate is
  MVP-5's exit: fresh clone, documented install, two repos registered,
  fresh harness context, intent through AGENTS.md, **actor + Docker
  verify**, detach, restart, return via status/show/transcript, find
  branch + outputs"* — Docker-in-a-Work is a named, explicit item in the
  next milestone's own ship gate, not an unaddressed gap.

`docker ps -a` at the end of this session shows zero `sergeant`-named
containers — consistent with the execute stage never launching one.

## #19 verdict

Argued both ways, on the evidence actually collected this session plus
the two prior real-Claude sessions it corroborates:

**For closure:** this session, `runB2`, and `mvp1-selfhost` are now
**three independent real-Claude measurements** of the same core claim —
the settle driver autonomously cascades a multi-stage Work from submit to
completion (or a clean, evidence-preserving interrupt) with **zero
client commands** in between. Run 2 here passes every item in the
walk-away checklist this soak was scored against: finished work
(`work.completed`, 2/2 stages), a retained branch with a real, minimal,
on-scope diff, a fully readable decoded transcript, turns bounded well
inside the cap (2/25), and no tangle (one file changed, exactly as
asked). Run 1 adds new, clean evidence for the *unhappy* path: the
300-second ceiling fired live and correctly with no operator
intervention, and cancel/interrupt teardown preserved good partial work
rather than losing it. Sonnet pinning is now adapter-confirmed
(`model_pin.verdict: "honored"`), not just flag-asserted.

**Against closure:** the project's own stated bar for "the real #19 soak,"
recorded before this session ran
(`docs/gauntlet/notes/mvp-bucketing-2026-08-11.md` line 78 — *"multi-hour,
real Claude, envelope-guarded, the MVP-2 Docker verify stage
exercised"*), is not met here. Run 2's total wall clock was **23
seconds**, using 2 of a 25-turn envelope with no pressure on either the
turn cap or the 300s ceiling; the Docker execute stage was not reached in
either run. `docs/gauntlet/notes/north-star-arbitration-2026-08-11.md`
(line 123) already ruled on a materially larger prior data point —
Run B2's 13 turns and two live cascades — as insufficient to "support
'walk away for hours' from anyone's sequencing." Adding one 23-second,
2-turn, no-Docker run and one canceled 1-turn run does not change that
arithmetic; if 13 turns wasn't enough, 2 more turns is not either.

**Verdict: issue #19 stays open.** This session is genuine, valuable
adapter evidence — a third clean confirmation of the settle-driver
mechanism, the first live measurement of `--ceiling-secs` actually firing,
and a reconfirmation of the interrupted-turn cost-visibility gap — but it
does not constitute *the* soak the issue asks for, by the project's own
prior, explicit definition of what that soak requires. No `Fixes #19`
trailer on this commit.

## Friction (max 12 lines)

1. Orchestrator redirect arrived mid-run-1, after the ceiling interrupt
   had already fired but before any retry — good timing to cancel without
   losing evidence, but it means the originally-briefed Docker-stage
   exercise never happened this session.
2. `pgrep`-style process evidence needs care: a bare `claude` substring
   match false-positived against this scratch tree's own
   `/tmp/claude-1001/...` path (caught mid-run-1, fixed by keying off
   child-of-daemon-pid + session-id instead).
3. `execution.turn_ceiling_interrupted` produces **no** `usage.updated`
   — a real, billable turn's cost is unrecoverable from the journal once
   the ceiling kills it (E1, reconfirmed a third time across three
   separate sessions now).
4. `sgt work show`'s `envelope.turns_spawned` is per-Work; nothing
   surfaces a daemon-lifetime total — the "3 turns" cumulative figure had
   to be hand-summed from two separate `work show` calls.
5. Run 1's `00-contract` turn spent ~5 minutes on real tool calls before
   the ceiling cut it off, for a subject with exactly 5 small files — the
   300s ceiling is tight for even a bounded `repo-to-icm` first stage.

## Teardown

Daemon stopped, `pgrep -f "debug/sgt [-]-data-dir"` empty afterward,
`docker ps -a` clean of `sergeant`-named containers. Scratch preserved
under `/tmp/claude-1001/.../scratchpad/mvp4-soak/` (not deleted).
