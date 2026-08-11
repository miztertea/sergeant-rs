# N-series Run B2 — settle driver measured live (closes evidence gap on #19 / R-N0-6, continued)

Governing: `docs/gauntlet/notes/v2-measurement-and-migration-plan.md` §Run B;
`docs/gauntlet/contracts/N2.md`; issue #19 (real-Claude soak evidence). This
run follows the original `docs/gauntlet/runs/runB/run-manifest.md` (both
attempts of which never advanced past stage `00-contract`, canceled) and
measures the **settle driver** — the fix for that run's core finding — live
against the real `claude` CLI. Same evidentiary standard as Run B: full raw
journal + decoded turn transcripts + artifacts, nothing extrapolated.

## Setup summary

- **Binary:** `/home/miztertea/sergeant-runb/target/debug/sgt`, built from
  `sergeant-runb` at **`08a173a`** ("GAUNTLET: round-2 fixer ledger entry —
  #44 close-out follow-through") — post-Bug-Sprint-2. Non-root Cerberus
  container (not the root-refusal environment that broke Run B attempt 1).
- **Data dir:** `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/runB2/data`
- **Subject/workspace:** `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/runB2/subject`,
  workspace name `runb2-subject`, HEAD `6c87cfb8d8adf5d1b5f8988ceb950f13bdf43eb3`.
- **Workflow:** `repo-to-icm` v2, 10 stages (`00-contract` … `90-reconcile`).
- **Backend:** `claude` (real CLI, `claude 2.1.227`).
- **Work id:** `01KZRBQF79YND346STVPWVVE5S`, branch
  `sergeant/01KZRBQF79YND346STVPWVVE5S`.
- **Intent:** identical to Run B's — decompose `reference/sergeant-upstream`'s
  root docs and `bin/` partition, pinned at upstream SHA
  `f430cfd4f90174a98adbd7abebbece6303817929`, into draft ICM workflows. A
  bounded measurement run, all other partitions out of scope by contract.
- **`permission_mode` mechanism — quoted from the file the setup agent
  wrote**, `subject/sergeant.toml`:

  ```toml
  [workspace]
  name = "runb2-subject"

  [[repository]]
  name = "runb2-subject"
  path = "."

  [[profile]]
  name = "runb2"
  backend = "claude"

  [profile.options]
  permission_mode = "bypassPermissions"
  ```

  Subject commit history confirms this was deliberate setup, not an
  accident: `5cdd3d2 runB2: declare runb2 profile (permission_mode=bypass
  Permissions, #47)` followed by `6c87cfb runB2: fix sergeant.toml shape
  (workspace + repository tables)`.
- **Doctor's read-only capability view of `permission_mode`** (run from
  inside the subject workspace, where profiles are discovered — running it
  from a directory with no declared profiles instead reports "no profiles
  declared", which is a workspace-discovery fact, not evidence anything was
  misconfigured):

  ```
  sergeant doctor — .../runB2/data
    [ok  ] git          git version 2.53.0
    [ok  ] claude       claude: claude 2.1.227 (Claude Code); all 8 required flags present
    [ok  ] data_dir     .../runB2/data is writable
    [ok  ] journal      101 events replay cleanly (head seq 101)
    [ok  ] projection   rebuilds from the journal to seq 101
    [ok  ] daemon       serving http://127.0.0.1:36585 (pid 2776268, api v1)
    [ok  ] permission_mode runb2=bypassPermissions
  healthy
  ```

## Verified timeline (from the full journal, `journal_full.ndjson`)

| seq | kind | timestamp | note |
|---|---|---|---|
| 1 | `daemon.started` | 12:10:48.902Z | |
| 4 | `work.submitted` | 12:11:03.273Z | |
| 8 | `work.started` | 12:11:03.311Z | |
| 13 | `command.accepted` (`work.submit`) | 12:11:03.314Z | **the only client command until the cancel** — everything from here to seq 96 ran unattended |
| 38 | `conversation.turn.grammar_unmeasured` | 12:13:07.481Z | ask-withdrawal firing live (quoted below) |
| 39 | `conversation.turn.ended` | 12:13:07.481Z | turn 1 ends, `result_envelope: true`, usage $1.409999 |
| 41 | `stage.completed` (`00-contract`) | 12:13:07.537Z | **no client command between seq 39 and seq 41 — the settle driver's first live firing** |
| 43 | `stage.entered` (`10-inventory`) | 12:13:07.538Z | |
| 88 | `conversation.turn.ended` | 12:17:25.836Z | turn 2 ends, `result_envelope: true`, usage $3.2126830 |
| 90 | `stage.completed` (`10-inventory`) | 12:17:25.869Z | **again, no client command — second autonomous firing** |
| 92 | `stage.entered` (`20-harvest`) | 12:17:25.870Z | |
| 96 | `work.canceled` | (immediately after) | from `active` |
| 99 | `conversation.turn.ended` | (same) | `interrupted: true`, `result_envelope: false` — the interrupt path |
| 101 | `command.accepted` (`work.cancel`) | (same) | the cancel command itself, arriving **after** `work.canceled` at seq 96 |

## Headline verdict 1 — the settle driver fired live, twice, with no client command

The original Run B's core finding was a discrepancy: the adapter's tested
signal-derivation logic said a clean, question-free turn should complete
its stage, but live the stage sat `active` indefinitely with zero
subsequent journal events until manually canceled. In this run, under the
fixed engine, both of the first two stages transition
`conversation.turn.ended` → `stage.completed` with **no intervening
`command.accepted`** — i.e., no CLI/API client drove the transition; the
engine's own settle driver polled the finished turn and advanced the stage
on its own:

- Stage 1 (`00-contract`): seq 39 (`turn.ended`) → seq 41
  (`stage.completed`) with only an intervening `usage.updated` (seq 40).
  Elapsed: 0.056s.
- Stage 2 (`10-inventory`): seq 88 (`turn.ended`) → seq 90
  (`stage.completed`) with only an intervening `usage.updated` (seq 89).
  Elapsed: 0.033s.

Both stages produced real, on-scope, committed artifacts (below), not
stubs — this is genuine cascade behavior on real actor output, not an
artifact of a trivial/degenerate turn.

## Headline verdict 2 — ask-withdrawal (`conversation.turn.grammar_unmeasured`) fired live

At seq 38, immediately before turn 1's `conversation.turn.ended`, the
journal recorded:

```json
{"seq":38,"kind":"conversation.turn.grammar_unmeasured","payload":{
  "capability":"ask",
  "detail":"a turn completed with a result envelope and no post_turn_summary line; the evidence `Capabilities::ask` rests on is not present in this CLI's stream, so the claim is withdrawn rather than left failing open",
  "expected":"system/post_turn_summary",
  "raw":"b3:19f866c59a8b95dd70394fae1d261af29da9ddfe3a8c48d2948aa6db671b46f8",
  "session_id":"f3b926c3-71c3-40b5-a232-4a2f08cd82cf"
}}
```

Verified directly against the decoded transcript
(`turn1_00-contract.stream-json.ndjson`): the only `system` subtypes
present are `init`, `success`, `thinking_tokens`, and
`vcs_state_changed` — no `post_turn_summary` line anywhere in the 74-line
transcript, confirming the withdrawal's premise. The `result` line shows
`is_error: false`, `stop_reason: "end_turn"`, `num_turns: 12` — a normal,
successful turn that simply never emitted the `post_turn_summary` shape the
`ask` capability's evidence depends on, so the engine withdrew the claim
rather than assert it on absent evidence or fail closed.

This event fired **exactly once** in the whole run (verified: `grep -c
grammar_unmeasured journal_full.ndjson` → `1`), even though turn 2 also
lacks a `post_turn_summary` line (`turn2_10-inventory.stream-json.ndjson`:
same four `system` subtypes, no `post_turn_summary`, `result` shows
`is_error: false`, `stop_reason: "end_turn"`, `num_turns: 20`). Consistent
with a once-withdrawn capability staying withdrawn for the work rather than
re-firing every turn — recorded as observed, not independently traced
further into the engine (out of scope for this measurement run).

## Stage 3 (`20-harvest`) — interrupted, and the `retained_dirty` question

Stage `20-harvest` was entered at seq 92, and the third execution started
(seq 94) and received the user turn (seq 95) — but the cancel (seq 96)
landed almost immediately after, well before the actor produced any tool
calls: there are **zero `tool.requested`/`tool.completed` events** between
seq 92 and the `turn.ended` at seq 99. The decoded transcript for this turn,
`turn3_20-harvest-interrupted.stream-json.ndjson`, is only **2 lines**
(`rate_limit_event` + `system/init`) — the process was killed before any
real work happened.

`surface.torn_down` at seq 100 recorded:

```json
{"report":{"bindings":[{"disposition":"removed", "repository":"runb2-subject",
  "work_branch":"sergeant/01KZRBQF79YND346STVPWVVE5S",
  "worktree_path":".../runB2/data/surfaces/01KZRBQF79YND346STVPWVVE5S/runb2-subject"}],
  "clean":true, "work_id":"01KZRBQF79YND346STVPWVVE5S"}}
```

**No `retained_dirty` teardown evidence exists in this run** — `disposition`
is `"removed"` and `clean` is `true`, matching `sgt work show --json`'s
`teardown.clean: true`. This is consistent with the transcript evidence
above: the interrupted turn never got far enough to make any uncommitted
worktree writes, so there was nothing dirty for teardown to retain. (Commits
from the two *completed* stages, `eb31aba` and `78c0bf4`, survive
independently in the subject's shared git object database — see artifacts
below — because they were committed on the work branch before teardown,
not because teardown retained anything.)

## Usage evidence (verbatim `usage.updated` fields)

**Turn 1** (`00-contract`, seq 40):

| Field | Value |
|---|---|
| `total_cost_usd` | **1.409999** |
| `is_error` | `false` |
| `model_pin.verdict` | `unpinned` |
| `usage.input_tokens` | 15 |
| `usage.output_tokens` | 9212 |
| `usage.cache_creation_input_tokens` | 33144 |
| `usage.cache_read_input_tokens` | 283943 |

Per-model (`model_usage`): `claude-fable-5` — input 15, output 9212, cache
creation 33144, cache read 283943, cost `1.4075729999999995`;
`claude-haiku-4-5-20251001` — input 2341, output 17, cost `0.002426`.

**Turn 2** (`10-inventory`, seq 89):

| Field | Value |
|---|---|
| `total_cost_usd` | **3.2126830000000006** |
| `is_error` | `false` |
| `model_pin.verdict` | `unpinned` |
| `usage.input_tokens` | 28 |
| `usage.output_tokens` | 18278 |
| `usage.cache_creation_input_tokens` | 76661 |
| `usage.cache_read_input_tokens` | 763362 |

Per-model (`model_usage`): `claude-fable-5` — input 28, output 18278, cache
creation 76661, cache read 763362, cost `3.2107620000000003`;
`claude-haiku-4-5-20251001` — input 1821, output 20, cost `0.001921`.

**Turn 3** (`20-harvest`, interrupted): no `usage.updated` event exists —
the turn was killed before usage was reported, matching Run B attempt 1's
precedent that a dead/interrupted turn produces no usage record.

**Total spend: $1.409999 + $3.2126830000000006 = $4.622682000000001 ≈
$4.62**, against the orchestrator's recorded **$2.50** stretch guard for
this run — an overrun of roughly 1.85x.

## Cancel provenance and the budget-granularity finding

The cancel that ended this run (`work.canceled` at seq 96, `command.accepted
work.cancel` at seq 101) **did not come from any in-engine budget guard.**
There is no budget-check event, no `stage.blocked`/`stage.failed` for a
budget reason, and no automated cost-threshold logic anywhere in the
journal between the two stage completions and the cancel. Per the verified
orchestration record for this session, the cancel came from **a stopped
orchestration workflow's collector agent** — an orchestration-harness
artifact sitting outside the engine, not a decision the daemon or the
`claude` adapter made. This should be recorded exactly as that: an
external stop, not an engine-level guard firing.

The reason no engine-level guard *could* have caught this at $2.50: usage is
only observable at **per-turn granularity** (one `usage.updated` event per
completed `conversation.turn.ended`). Turn 2 alone cost **$3.2126830000000006
— already past the $2.50 guard on its own**, and there is no sub-turn
usage signal the engine could poll to stop a turn mid-flight before it
finishes. A sub-turn budget cap is not implementable against this adapter's
current telemetry; the only enforceable granularity is "cap total spend
after each turn commits," which would have caught the overrun only *after*
turn 2 had already exceeded it. **The $4.62-vs-$2.50 overrun is the
orchestrator's accountability** — the guard was a target for the
orchestrator's own stop discipline, not a mechanism the engine enforced,
and the orchestrator's collector correctly stopped the run once the overrun
was noticed, just after turn-2 granularity had already made the guard
unenforceable in advance.

## Stage reached / timings

| Stage | Entered | Ended | Outcome |
|---|---|---|---|
| `00-contract` | 12:11:03.311Z (work.started) | 12:13:07.537Z (`stage.completed`, autonomous) | completed, real artifact (`contract.md`, commit `eb31aba`) |
| `10-inventory` | 12:13:07.538Z | 12:17:25.869Z (`stage.completed`, autonomous) | completed, real artifact (`inventory.md`, commit `78c0bf4`) |
| `20-harvest` | 12:17:25.870Z | canceled almost immediately (seq 96) | interrupted before any tool call; teardown clean, nothing retained |
| `30-normalize` … `90-reconcile` | not reached | — | — |

**Two of ten stages completed autonomously; a third was entered and cleanly
interrupted mid-startup.** This is a materially different — and better —
shape than Run B, where zero stages ever transitioned past `00-contract`.

## #19 verdict

This run is **substantial adapter evidence**: it measured the settle driver
firing live twice on real, non-trivial actor turns (real committed
artifacts each time, not stubs), and measured the interrupt/cancel path
cleanly tearing down a stage that hadn't produced any writes yet. It is
**not** a completed soak — only 2 of 10 workflow stages were reached, the
run ended via external cancel rather than natural completion or a
deliberately engineered failure/retry scenario, and total real wall-clock
coverage is a few minutes of two turns. **Issue #19 stays open.** This
manifest is additional measured evidence toward it, not its closure.

## Files in this directory

- `run-manifest.md` — this file
- `journal_full.ndjson` — full journal, daemon start (seq 1) through
  `daemon.stopped` recorded separately at teardown (101 in-run events; see
  teardown note below for the final stop)
- `turn1_00-contract.stream-json.ndjson` — turn 1's raw stream-json
  transcript, decoded from blob `b3:19f866c5…` (74 lines)
- `turn2_10-inventory.stream-json.ndjson` — turn 2's raw stream-json
  transcript, decoded from blob `b3:65c2517d…` (127 lines)
- `turn3_20-harvest-interrupted.stream-json.ndjson` — turn 3's raw
  stream-json transcript, decoded from blob `b3:1b2685d5…` (2 lines —
  killed at startup)
- `worktree-output/contract.md` — the `00-contract` artifact, recovered
  from the subject's shared git object database at commit `eb31aba`
  (the linked worktree was removed by clean teardown, but its commits
  survive independently on branch `sergeant/01KZRBQF79YND346STVPWVVE5S`)
- `worktree-output/inventory.md` — the `10-inventory` artifact, recovered
  the same way at commit `78c0bf4`
