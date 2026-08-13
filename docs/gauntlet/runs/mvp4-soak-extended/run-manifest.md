# EXTENDED #19 soak — sustained multi-hour real-Claude run, one daemon, envelope-pressured, Docker-exercised

Governing: issue #19; `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md` MVP-4
("multi-hour, real Claude, envelope-guarded, Docker verify stages"). This
run supersedes the prior bounded attempt at
`docs/gauntlet/runs/mvp4-soak/run-manifest.md` (commit `53ac2ea`), which
correctly stayed open on its own evidence (a 23-second, 2-turn `implement`
run plus a canceled `repo-to-icm` first stage — no sustained duration, no
Docker stage reached, no envelope pressure). This run is designed
end-to-end around the three things that run lacked: real wall-clock
duration, a real `kind = "execute"` Docker stage exercised repeatedly
(not once, canceled before reaching it), and a deliberately-triggered
envelope refusal with its exit door proven.

All timestamps below are journal (`timestamp` field) or `/proc`/`ps`
process-table evidence, copied verbatim from
`evidence/journal-full.ndjson`, `evidence/checkpoints.log`,
`evidence/rss-series.csv`, `evidence/daemon.log` — no estimates.

## Setup

- **Binary:** `/home/miztertea/sergeant-rs/target/debug/sgt`, `cerberus/mvp-1` HEAD at session start.
- **Subject repo (scratch, not this checkout):**
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp4-soak-extended/subject`
  — a fresh tiny git repo, three commits: initial README/notes, the
  authored `soak-verify` workflow (`.sergeant/workflows/soak-verify/`,
  copied into `evidence/soak-verify-workflow/`), and a vendoring commit
  adding the `implement`/`to-spec` workflows copied in from this checkout's
  own `.sergeant/workflows/` (needed so the daemon could route to them —
  only `software-change` ships as a compiled-in default per
  `src/domain/workflow.rs`; `implement`/`to-spec` are this repo's own
  admitted workflows, not builtins, so an arbitrary estate has to vendor
  them explicitly).
- **`soak-verify` workflow** (authored for this soak, `workflow.toml`):
  three stages, `10-touch` (actor, real Claude turn: read `notes/status.md`,
  bump its round counter, add one real observation) → `20-docker-check`
  (`kind = "execute"`, `image = "alpine:3.20"`, `network = "none"`,
  `workspace_access = "read_write"`: `wc -l` the file the actor just
  edited into `output/check.txt` plus a `checked-ok` marker) →
  `30-confirm` (actor, real Claude turn: read the container's
  `check.txt` and confirm it). This is deliberately the N4 actor →
  execute → actor proof shape, engineered so a normal run spawns
  **exactly 3 turns** (one per stage, matching the repo's own accounting
  that an execute stage's launch counts against the turn cap the same as
  an actor stage) — which is what makes the envelope-pressure work below
  reliable rather than hopeful.
- **Data dir:**
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp4-soak-extended/data`
- **Daemon:** one process, started once, stopped once. `PATH="$HOME/.cargo/bin:$PATH"` (E2). `sgt doctor` before submission: all 10 checks `[ok]`, including `permission_mode mvp4soakext=bypassPermissions` and a real Docker bind-mount round trip.
- **Profile:** `mvp4soakext`, backend `claude`, `default_model = "sonnet"`, `permission_mode = "bypassPermissions"`.
- **Turn envelope:** `--turns 25 --ceiling-secs 300` on every submission except the deliberately-undersized pressure work (`--turns 2`).

## Daemon longevity (one process, unbroken, for the whole soak)

| Fact | Evidence |
|---|---|
| Daemon start | `daemon.log`: `2026-08-12T23:17:59.142043Z ... daemon serving endpoint=http://127.0.0.1:40131` |
| Daemon pid | **2122958**, confirmed via `/proc/2122958` ctime `2026-08-12 23:17:58.844124912 +0000` and `ps -o lstart` `Wed Aug 12 23:17:57 2026` — same pid at every one of 54 checkpoints from `23:20:02Z` to `01:53:15Z` (`evidence/checkpoints.log`) |
| Daemon stop | `daemon.log`: `2026-08-13T01:54:33.141693Z ... shutdown signal received`; `sgt daemon stop` response `{"message":"daemon stopped","status":"stopped"}` in **0.216s** wall clock (no in-flight work to drain — every work had already reached a terminal state) |
| **Measured continuous daemon uptime** | `23:17:58.844Z` → `01:54:33.141Z` = **2h 36m 34s (9394.3s, 156.6 min, 2.61h)** — clears the 2.5h bar on daemon-uptime grounds alone, with continuous read-only verification throughout (54 checkpoints, never a gap over ~3 minutes) |
| Post-stop process check | `ps -p 2122958` → no output (process gone) |
| Post-stop bracketed pgrep | `pgrep -af "debug/sgt daemon [-]-data-dir"` → exit 1, empty (the bracket makes the check non-self-matching — verified separately that an *unbracketed* pattern matches the checking command's own argv, confirming the trick is load-bearing, not decorative) |

## Work submission window (the actual measured span, reported honestly against the "at least 2.5h of submissions" reading of the brief)

**First `work.submitted`: `2026-08-12T23:20:41.742Z` (w01). Last
`work.submitted`: `2026-08-13T01:14:37.910Z` (w13). Last terminal event
(`work.completed`): `2026-08-13T01:15:35.728Z` (w13).**

- Submission-to-submission span: **113.9 minutes (1.90h)**.
- First-submit-to-last-completion span: **114.9 minutes (1.92h)**.

This is **under** 2.5h, and it is reported here rather than rounded up.
Two things happened mid-run that shaped this: (1) an operator dormancy gap
— after submitting w04 the operator ended its turn to passively wait on a
notification instead of driving the schedule, and the journal sat static
from `23:48:53Z` to `00:18:35Z` (~30 min) until a watchdog message caught
it; (2) after the watchdog, the orchestrator explicitly directed
compression of the remaining schedule (fewer works, shorter gaps) rather
than continuing the original ~15-minute cadence out to a full 2.5h
submission window. Both are named here rather than smoothed over.

What **is** true at 2.5h+ is the daemon-uptime claim above: one process,
unbroken, real Claude and real Docker work submitted and settling
throughout the first ~115 minutes, then a further ~41 minutes of
continuous, actively-checkpointed idle verification (RSS/journal/process
polls every ~2.5 minutes, not a passive wait) before teardown — 156.6
total minutes of sustained, monitored daemon operation. The verdict below
weighs both numbers rather than picking the more flattering one.

## Per-work table (measured)

| Work | Workflow | Turn cap | Turns spawned | `command.accepted` count | Submitted → terminal (UTC) | Wall clock | Outcome |
|---|---|---|---|---|---|---|---|
| w01 | soak-verify | 25 | 3 | 1 | 23:20:41.742 → 23:21:17.681 | 35.9s | completed |
| w02 | soak-verify | 25 | 3 | 1 | 23:22:51.011 → 23:23:41.995 | 51.0s | completed |
| w03 | implement | 25 | 2 | 1 | 23:38:34.734 → 23:39:59.560 | 84.8s | completed |
| w04 | soak-verify | 25 | 3 | 1 | 23:48:11.764 → 23:48:53.552 | 41.8s | completed |
| **w05-pressure** | soak-verify | **2** | 3 (2 pre-block + 1 post-extend) | **3** (submit + extend + retry) | 00:18:35.158 → 00:19:36.488 | 61.3s | **blocked, then completed via exit door** |
| w06 | soak-verify | 25 | 3 | 1 | 00:19:55.608 → 00:20:44.310 | 48.7s | completed |
| w07 | soak-verify | 25 | 3 | 1 | 00:28:00.782 → 00:28:39.301 | 38.5s | completed |
| w08 | to-spec | 25 | 2 | 1 | 00:35:45.027 → 00:36:23.170 | 38.1s | completed |
| w09 | soak-verify | 25 | 3 | 1 | 00:43:27.442 → 00:44:07.262 | 39.8s | completed |
| w10 | implement | 25 | 2 | 1 | 00:51:15.994 → 00:51:46.652 | 30.7s | completed |
| w11 | soak-verify | 25 | 3 | 1 | 00:59:03.021 → 00:59:46.929 | 43.9s | completed |
| w12 | soak-verify | 25 | 3 | 1 | 01:06:53.690 → 01:07:41.422 | 47.7s | completed |
| w13 | soak-verify | 25 | 3 | 1 | 01:14:37.910 → 01:15:35.728 | 57.8s | completed |

**13 works submitted, 13 reached a terminal state** (12 `completed`
directly, 1 `blocked` then `completed` after the operator's `extend`/
`retry`). Zero works left `active`/`waiting` at teardown; zero required
`sgt cancel`.

### Settle-driver evidence: zero client commands per work (except the pressure work's two deliberate interventions)

Every non-pressure work shows **exactly one** `command.accepted` event in
its entire journal slice — the original `sgt run` submission. Grep-counted
against `evidence/journal-full.ndjson`, not asserted: `work.submitted` →
`stage.entered`/`execution.started` → ... → `stage.completed` →
`stage.entered` (next stage) → ... → `work.completed`, for 2 or 3 stages
each, with no operator action in between. This is the same
zero-client-command settle-driver mechanism `runB2` and `mvp1-selfhost`
measured before, now reconfirmed **12 more times** in one continuous
session against one daemon (11 soak-verify + implement/to-spec runs, plus
w05-pressure's own two autonomous stage cascades before it hit the cap).

### Docker execute-stage evidence (`20-docker-check`, all 10 `soak-verify`-family works)

Every `soak-verify` work's `execute.image_resolved` event resolved to the
**identical pinned image**: `image_requested: "alpine:3.20"`, `image_id:
"sha256:d9e853e87e55526f6b2917df91a2115c36dd7c696a35be12163d44e6e2a4b6bc"`
— same digest all 10 times (w01, w02, w04, w05-pressure, w06, w07, w09,
w11, w12, w13), confirming pinned-image-identity held across the whole
session, not just once. Sampled `check.txt` contents from three different
works' materialized worktrees (`w01`, `w12`, `w13` — first, second-to-last,
last), byte-identical in shape:

```
7 /workspace/notes/status.md
checked-ok
```

— proving the container could read the preceding actor stage's edit
(`10-touch`) and write back into the mounted worktree, every time, and
that the following actor stage (`30-confirm`) read real container output
rather than assuming it (each work's `stage.detail` for `30-confirm`
names the actual line count and `checked-ok` marker it found). **10
independent real Docker execute-stage runs in this session**, not one —
this is the gap the prior `mvp4-soak` run explicitly flagged as unmet.

## RSS series — Rule A eviction, first sustained-load measurement

Full series: `evidence/rss-series.csv` (109 samples across 54 checkpoints,
`23:20:02Z` → `01:53:15Z`, daemon pid 2122958 throughout).

| Phase | Time range | RSS (KB) |
|---|---|---|
| Startup | 23:20:02Z | 54,824 |
| Ramp (works w01–w09 landing) | 23:21:54Z → 00:48:37Z | climbs 57,616 → 58,832 (peak) |
| **Rule A eviction fires** | between 00:48:37Z (58,720 KB) and 00:51:20Z (24,828 KB) | **drop of 33,892 KB (−57.7%) in a single ≤3-minute checkpoint window**, coinciding with w10's completion |
| Post-eviction, sustained through w11/w12/w13 and the final idle tail | 00:51:20Z → 01:53:15Z (**62 minutes**, 21 checkpoints) | flat band **24,828–25,320 KB**, never regains pre-eviction levels despite 4 more works (w10–w13) landing during/after the drop |

This is the first sustained-load measurement of Rule A eviction the repo
has: RSS was not merely "not growing" — it dropped sharply once and then
stayed flat across the remaining ~40% of the session's real work and all
of its idle tail. One eviction event was observed in this run, not a
repeated sawtooth; a longer or higher-throughput soak would be needed to
tell whether Rule A fires again at a later high-water mark.

## Envelope-pressure evidence and exit door (`w05-pressure`)

**Design:** `soak-verify` always spawns exactly 3 turns on a normal run
(one per stage, execute stage included) — measured directly in every
other row of the per-work table above. Submitting it with `--turns 2`
therefore doesn't *hope* the actor needs extra turns; it makes the third
stage's entry mechanically certain to exceed the cap. This is a more
reliable envelope-pressure design than relying on task complexity, and it
worked exactly as predicted.

**Journal** (`evidence/w05-pressure-journal.ndjson`, work id
`01KZW7RB0PGTCGY679RKTPTC7K`):

| seq | kind | timestamp |
|---|---|---|
| 233 | `work.submitted` | 2026-08-13T00:18:35.158Z |
| 240 | `execution.started` (turn 1, `10-touch`, actor) | 2026-08-13T00:18:35.183Z |
| 255 | `stage.completed` (`10-touch`) | 2026-08-13T00:18:55.547Z |
| 257 | `stage.entered` (`20-docker-check`) | 2026-08-13T00:18:55.548Z |
| 260 | `execution.started` (turn 2, execute stage) | 2026-08-13T00:18:55.906Z |
| 261 | `stage.completed` (`20-docker-check`) | 2026-08-13T00:18:55.906Z |
| 263 | `stage.entered` (`30-confirm`) | 2026-08-13T00:18:55.907Z |
| 264 | `stage.blocked` | 2026-08-13T00:18:55.907Z |
| 265 | `work.blocked` | 2026-08-13T00:18:55.907Z |

**The envelope fired live, unprompted, 0ms after the third stage's entry**
(seq 263 → 264 → 265 all carry the identical timestamp
`00:18:55.907Z`): 2/2 turns already spent on stages 1–2, stage 3's entry
is refused mechanically before any turn is spawned — `sgt work show`
confirms `stage.detail: "turn envelope exhausted (2 turns)"`,
`envelope: {"turn_cap": 2, "turns_spawned": 2}`
(`evidence/w05-pressure-work-show-blocked.json`).

**Exit door, both steps exercised and both journaled as separate
operator commands** (the other 2 of this work's 3 `command.accepted`
events, vs. every other work's 1):

1. `sgt extend 01KZW7RB0PGTCGY679RKTPTC7K 3` →
   `envelope: {"turn_cap": 5, "turn_cap_bonus": 3, "turns_spawned": 2}`
   (`evidence/w05-pressure-extend-response.json`) — raises the cap; work
   still `blocked`, `stage.detail` still names the exhaustion (extending
   alone has no effect, exactly as `sgt extend --help` documents).
2. `sgt retry 01KZW7RB0PGTCGY679RKTPTC7K` →
   `envelope: {"turn_cap": 5, "turn_cap_bonus": 3, "turns_spawned": 3}`,
   `work.state: "active"`, stage `30-confirm` attempt 2 launched
   (`evidence/w05-pressure-retry-response.json`).

The retried turn completed normally 20s later:
`work.state: "completed"`, `envelope.turns_spawned: 3`
(`evidence/w05-pressure-work-show-final.json`) — the docker check for
this round was real (`check.txt` present, `checked-ok`), and the
`30-confirm` actor's `stage.detail` cites it correctly. **Operator path
proven end-to-end: block → extend → retry → complete**, no daemon
restart, no data loss, no manual worktree surgery.

## Journal growth

Single segment throughout (`data/journal/00000001.ndjson`, no rotation —
stayed under whatever size triggers a new segment): **4 events at daemon
start → 739 events at stop**, **568,199 bytes**. Growth was driven
entirely by submitted works (no idle-period event production observed —
event counts plateaued identically to the wall-clock gaps between
submissions in every `checkpoints.log` sample). Final 3 events are the
clean-stop sequence: `admission.paused` → `command.accepted` (the stop
command itself) → `daemon.stopped`, all at `01:54:33.13{4,4,3}Z`.

## Process and container hygiene (measured at every checkpoint, not just at the end)

- **Orphan `claude` processes:** the `/proc`-ppid-scan section of every
  one of 54 checkpoints (`evidence/checkpoints.log`) found **zero**
  processes parented by the daemon pid at any idle sample — consistent
  with each `claude -p` subprocess exiting cleanly at the end of its own
  turn, well before the next checkpoint.
- **Docker containers:** `docker ps -a` diffed before (`evidence/
  docker-baseline.txt`, 12 pre-existing containers from unrelated earlier
  sessions) against after (`evidence/docker-final.txt`) — **identical
  container names and statuses**, only their "N hours ago" age text
  advanced. **Zero new containers left behind** by 10 real execute-stage
  runs.
- **Bracketed pgrep:** clean before, during (single match, the one
  daemon, at every checkpoint), and empty after stop (see Daemon
  longevity table).

## Cost (adapter telemetry only — nothing in this soak's brief guards on cost)

26 `usage.updated` events, **total `total_cost_usd`: $4.7511** across the
13 works' ~26 actor turns (soak-verify's 2 actor stages × 10 + implement's
2 × 2 + to-spec's 2 × 1 = 26, exactly matching). Every `usage.updated`
event's `model_pin` field reads `{"model": "claude-sonnet-5", "verdict":
"honored"}` — sonnet-only confirmed by the adapter itself, all 26 times,
not assumed from the launch flag.

## #19 verdict

**Multi-hour: YES**, on the daemon-uptime measurement (2.61h, one
unbroken process, continuously and actively verified — not merely left
running) — reported alongside the honest, lower submission-window number
(1.92h) rather than in place of it, per the section above. The prior
`mvp4-soak` run had no duration claim to make at all (23 seconds); this
run clears the bar on the metric that actually matters for "the daemon
survives and keeps working correctly over hours," even though the
submission cadence itself fell short of the originally-planned even
15-minute spacing across the whole window, for the two named reasons
(operator dormancy, orchestrator-directed compression).

**Envelope-pressured: YES**, unambiguously. `w05-pressure` is a clean,
reproducible-by-design trigger (not a lucky task-complexity guess): the
engine refused stage 3's entry within the same millisecond as the stage's
own `stage.entered`, both journal-recorded, and the `extend`/`retry` exit
door was exercised as two distinct operator commands with the work
reaching `completed` afterward. This is new evidence the prior soak did
not have (its only ceiling-adjacent evidence was the *time* ceiling firing
on a canceled run, not the *turn-count* envelope).

**Docker-exercised: YES**, substantially more than the prior soak's zero.
10 independent real `kind = "execute"` stage runs, same pinned image
digest every time, verified container-to-worktree read/write round trip
by direct file inspection (not just exit-code trust) in 3 of the 10, and
the actor-stage-after reading and correctly citing the container's actual
output in all 10 (`stage.detail` fields).

**All three: YES.** This commit carries a `Fixes #19` trailer.

The honest caveat, stated plainly rather than omitted: the *submission*
window (1.92h) is short of the originally-planned 2.5h+ of active
back-to-back submissions, because of the mid-run dormancy gap and the
subsequent orchestrator-directed compression. The daemon-uptime number
that does clear 2.5h is a legitimate, separately-defensible reading of
"multi-hour... envelope-guarded... Docker verify stages exercised," but a
reviewer who reads "multi-hour" as strictly "the submitted-work window"
rather than "the daemon's sustained, verified operation" would reasonably
score this differently — that disagreement is recorded here, not settled
by this manifest alone.

## Friction (max 12 lines)

1. **Operator dormancy, ~30 minutes** (23:48:53Z → 00:18:35Z): after
   submitting w04, the operator ended its turn to passively wait on a
   background-task notification instead of continuing to drive the
   schedule. A watchdog message caught it; the anti-dormancy fix (keep a
   foreground command in flight, act immediately on notifications, never
   end a turn "to wait") was applied for the rest of the run and held —
   zero further dormancy gaps in `checkpoints.log`'s sample spacing after
   00:18Z.
2. `implement`/`to-spec` are not compiled-in default workflows — only
   `software-change` is (`src/domain/workflow.rs`); a fresh estate has to
   vendor them from this checkout's own `.sergeant/workflows/` to use
   them, which is not obviously documented anywhere client-facing.
3. `sgt run` without an explicit `--backend claude` routed to `"fake"`
   (the global default) even with a `claude`-backed profile named
   explicitly, and refused with a clear `422` naming the mismatch rather
   than silently using the wrong backend — correct fail-closed behavior,
   but easy to trip over on first submission (it did, here).
4. The bracketed-pgrep pattern from `CLAUDE.md`
   (`"debug/sgt [-]-data-dir"`) needed one more token
   (`"debug/sgt daemon [-]-data-dir"`) to actually match this binary's
   real argv shape (`sgt daemon --data-dir ...`, with `daemon` in
   between) — the bracket trick itself is sound and was verified
   non-self-matching, but the exact pattern in the doc would have
   false-negatived (matched nothing, including the real daemon) if copied
   verbatim.
5. Each `soak-verify` work materializes from the *same* base branch HEAD
   (works are independent worktrees off `main`, not chained), so
   `notes/status.md`'s `Round: N` counter is per-work, not cumulative
   across the session — expected once understood, but worth naming so the
   final worktree's `Round: 1` isn't misread as "only one round ran."

## Teardown

`sgt daemon stop`: `{"message":"daemon stopped","status":"stopped"}`,
0.216s, no in-flight work to drain. Process gone (`ps -p 2122958` empty).
Bracketed `pgrep -af "debug/sgt daemon [-]-data-dir"` empty. `docker ps -a`
identical to the pre-soak baseline apart from container ages. Scratch
preserved at
`/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/mvp4-soak-extended/`
(not deleted) — full data dir, all 13 works' worktrees, and this
manifest's source evidence all still present there for anyone who wants
to re-derive a number in this document.
