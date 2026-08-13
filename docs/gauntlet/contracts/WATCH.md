# WATCH — `sgt watch` — **ADJUDICATED (owner ruling on auto-spawn 2026-08-13; remaining dispositions orchestrator-recommended under standing flag-and-proceed, owner overrules by exception)**

Derived from `reference/proposal-sgt-watch-v1.md` (vendored 2026-08-13, audit
basis 5756b5d — audit verified accurate by the review panel) as amended by the
proposal-review gauntlet (wf_56a68808: 22 panel findings + 5 orchestrator
findings → 21 confirmed / 3 plausible / 3 refuted-as-duplicates; tree clean)
and the owner's rulings. The proposal is cited, never restated: where this
contract is silent, the proposal's text governs; where they disagree, this
contract wins.

Governing, cited never restated: the proposal (§ by number); `NORTH-STAR.md`
(R-NS-4, R-NS-6); `AGENTS.md`; `GAUNTLET.md` deviation register; `LESSONS.md`
(L7, L8, L13, L19 bind hardest); `docs/DEVELOPMENT.md`.

## Outcome

One small vertical slice (proposal §19): a read-only `sgt watch [WORK_ID]
[--follow]` verb over the existing SSE + Work surfaces — the harness's return
path after `sgt run`. No new daemon route, event family, crate, or durable
subscription state (WATCH-07/-08/-11/-12 stand as proposed). The panel
confirmed every §2 audit claim against source (SSE attach-live-before-replay,
seam dedup, lag refill, `Last-Event-ID`/`?from=`, `stream_events`,
`journal_head`, event-kind names) and confirmed no existing surface already
serves a headless blocking consumer (L18 check: the TUI is the only SSE
consumer today).

## RULINGS

### R-WATCH-1 — The watch set is six states, not five (V1, error)

**Ruled.** `waiting` joins the watch set: `needs_input`, `blocked`, `waiting`,
`failed`, `completed`, `canceled`. Excluded: `pending`, `active` only.

**Why.** The proposal's own justification for watching `failed` ("Sergeant
permits an explicit retry from that state", §6.4) applies verbatim to
`waiting`: `begin_retry` gates on `Failed | Blocked | Waiting`
(`src/runtime/engine.rs:1763-1773`), `BackendSignal::Waiting` routes to
`Next::Parked` exactly as Blocked does (`engine.rs:2157-2171`), and nothing
auto-resumes it. A Work parked in `waiting` would otherwise sit invisible —
the polling this verb exists to kill. `work.waiting` exists as a real kind
(`src/domain/work.rs:126-143`). Scoped `--follow`: `waiting` emits and
continues, like `blocked`.

**Pin.** W-test: Work parked `waiting` → exactly one notice; `pending`/`active`
transitions emit nothing.

### R-WATCH-2 — The fingerprint carries attention identity (F1, error)

**Ruled.** §8.4's fingerprint gains the attention payload's identity:
`{state, stage_id, attempt, stage_status, detail_identity}` where
`detail_identity` is a hash of the snapshot's current question/blocking-detail
text (empty for states that carry none).

**Why.** A needs_input → respond → needs_input cycle inside one stage attempt
changes none of the four proposed fields (`begin_input` reuses
stage_id/index/attempt, `engine.rs:1632-1700`; attempt increments only on
retry, `src/domain/workflow.rs:346`) — question B would be silently swallowed:
the proposal's own §17 falsifier.

**Pin.** W-test: two different questions in the same stage attempt → two
notices; the same snapshot re-triggered by queued duplicate events → one.

### R-WATCH-3 — `watch` never auto-spawns a daemon (owner-ruled 2026-08-13)

**Ruled.** `sgt watch` joins `sgt doctor` and `sgt daemon stop` in the
deliberate no-auto-spawn set (`src/cli.rs:1266-1268`, `:1382` are the
precedent). No running daemon → stderr names the state and the remedy
("no daemon is running for <data_dir>; start one with any dispatching verb or
`sgt daemon`"), exit nonzero, **no daemon spawned**. Supersedes §11.1's first
clause; the no-restart-on-closure half of §11.1 stands. Observation must not
materialize the thing observed — fail-closed at both ends of the process's
life.

**Pin.** W-test: `sgt watch` against a data dir with zero daemons → refusal,
remedy text, and a process-table check proving nothing was spawned (inverse of
m2's `t7` auto-spawn proof).

**Flagged follow-on (not in WATCH scope):** the owner's consistency direction
— observation surfaces generally shouldn't materialize the daemon — touches
`status`/`work`/`analytics`/`web`, whose auto-spawn is pinned by m2/m6/m8
contract tests and load-bearing in AGENTS.md's standard loop step 2. That
sweep is its own unit with its own adjudication; a backlog issue records it.

### R-WATCH-4 — Estate-wide gap closed by ordering, not grammar (V2)

**Ruled.** No `--from <seq>` in v1 (WATCH-03 minimalism stands). The blind
window between fleet reconciliation and watch attach is closed by doctrine:
**attach first, then reconcile** — start the estate-wide watcher (or a scoped
one), *then* run `sgt status`/`sgt work list`; anything landing after the
watcher's head H arrives on the stream. This is the daemon's own
subscribe-before-history pattern (`src/api.rs:2482-2496`) applied one level
up, at zero code cost. §15.1's AGENTS.md text and README must state the order
explicitly. Residual stated honestly: a bare one-shot estate watch invoked
*after* reconciliation still carries the gap — the docs say so instead of
pretending §6.6 closes it.

### R-WATCH-5 — §15.1 amends AGENTS.md without dropping doctrine (V3, orch:F2)

**Ruled.** The step-6 rewrite folds in, verbatim or strengthened, the
journal-not-liveness sentence and its five BU tags (BU-0036/0038/0047/0111/
0115) — a literal "replace" that drops them is refused. The added guidance
steers long waits to `--follow` under the harness's background facility:
foreground tool calls in the pilot harness cap at ~10 minutes, so one-shot
foreground `watch` is for short expected waits; the §16.4 pilot gate records
which facility the harness chose as environment evidence, per the proposal.

### R-WATCH-6 — W3/W4 get a real seam, not a wall-clock race (F2, orch:F4)

**Ruled.** The watch loop carries a test-only, env-gated rendezvous:
`SGT_WATCH_TEST_HOLD=<path>` — when set, the client touches `<path>.ready`
after reading journal head + attaching the stream, then blocks until `<path>`
exists, before its first Work read. Unset (production): zero-cost no-op.
Precedent: `SGT_FAKE_SCRIPT`'s env-knob pattern (`src/backend/fake.rs:493`)
extended to the client process, where fake.rs's in-process `Gate` cannot
reach. W3 forces its transition inside the held window; W4 uses the same hold
to land respond-then-complete before the first read. The seam is documented in
the module and excluded from user-facing help.

**Pin.** W3/W4 as specified in §16.2, now deterministic. A test that passes
with the hold proven never engaged is vacuous (TH-05's class) — the test
asserts the `.ready` handshake occurred.

### R-WATCH-7 — Malformed-frame tightening revises a pinned contract, deliberately (F3)

**Ruled.** §11.2 requires `decode_frame`/`EventStream` to distinguish
keep-alive/comment, valid event, malformed `data:` frame, and transport end —
today malformed and keep-alive both collapse to `None`
(`src/api.rs:3036-3072`), and the existing test
`decode_frame_coalesces_data_lines_with_a_real_newline` (`src/api.rs:3182`)
**pins the very silent-skip §11.2 forbids**. The builder revises that test's
assertion in the same commit that changes the semantics, cites this ruling in
the commit body, and audits the TUI — the other `EventStream` consumer
(`src/tui.rs:1073`) — for deliberate handling of the new variant. This is a
ruled exception to "existing pins don't move": the pin moves because the
contract that produced it changed, with provenance.

### R-WATCH-8 — The exit table states the abandoned-failure row (F4, orch:F3)

**Ruled.** Scoped `--follow` exit-on-terminal means `completed | canceled`
only. A Work that fails and is never retried or canceled leaves a `--follow`
watcher attached indefinitely *after having emitted the `failed` notice* —
stated in `--help`, README, and §11.4's table as a row, not discovered. A
supervising harness that wants an exit on `failed` uses one-shot mode and
re-arms.

### R-WATCH-9 — The snapshot is the verbatim Work view; the example matches it (F5)

**Ruled.** §9.3's prose is the contract: `snapshot` is the complete
`GET /v1/work/{id}` body forwarded verbatim — today eleven top-level keys
(`work, stage, surface, execution, reservation, workflow, backend,
route_source, teardown, output, envelope`, `src/api.rs:1489-1535`), including
`envelope.turn_cap_bonus` and `work.created_by/created_at`. The §9.2 worked
example is corrected to the real shape before any builder sees it; a test
asserts the notice's `snapshot` deep-equals the endpoint body at emission
time, so the wrapper can never drift into a reduced parallel schema (R-NS-4's
teeth).

### R-WATCH-10 — Test-plan deltas (F6, F7, orch:F5)

**Ruled.** In addition to W1–W8 (as amended above): (a) a signal W-test —
SIGINT/SIGTERM to a live watcher → native signal exit, no journal event, no
Work state change; (b) all live-daemon W-tests run through the existing
`tests/support` DataDir isolation rigs like every other stateful CLI surface;
(c) the R-WATCH-3 no-spawn refusal test replaces the proposal's implied
auto-spawn-then-attach case. §16.1's decoder bullets cite the existing pinned
tests (`decode_frame_skips_comment_and_keep_alive_lines`, and the F3-revised
malformed-frame test) rather than re-deriving them. §16.3's structural
import checks are precedented in this repo and in scope. Any contract text
derived from §2.2 uses the real CLI grammar (`run`, `work
show|list|transcript`), not the proposal's shorthand.

## Acceptance

Proposal §17 stands with these substitutions: criterion 1 reads six states;
criterion 9 is satisfied via the R-WATCH-6 seam; criterion 12's "never
auto-restarts" extends to "never auto-spawns" (R-WATCH-3); criterion 13
unchanged and load-bearing. The §17 falsifier stands verbatim — it is the
sharpest sentence in the proposal. The §16.4 product pilot (this harness, real
Work, no polling) is the human-facing gate before the PR is offered for merge.
