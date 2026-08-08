# Product Requirements: Claude Background Harness

Status: Draft

Pinned source baseline: `efcc96639ab83caf908e651bbedc0790487620a0`

Related research: `docs/research/claude-background-harness-spike.md`

---

## Summary

A Claude worker's foreground harness invocation never returns once its mission is complete: Claude's
interactive TUI can write `.sergeant-status=done` and sit at its own prompt indefinitely, so
`sgt-interactive-worker`'s installed `EXIT` trap never fires. Every Claude-harness dispatch therefore
requires a human to find and kill the idle process by hand before drain, watch-recycle, or cleanup
can proceed — Sergeant's automation for this harness is blocked on manual intervention at the end of
every mission.

This PRD specifies fixing that by launching Claude with its native background-session support
(`claude --bg`) and holding the worker's persistent slot on `claude attach <id>` instead of a bare
invocation. A Sergeant-owned watcher ends the mission with an external `claude stop <id>`, which is
measured to make the attached call exit on its own — restoring normal `_finish`/drain/cleanup
behavior with no change to the existing notification, ACK, or pane-identity protocol.

## Users

- **Coordinator agent:** dispatches `--agent claude` workers exactly as it dispatches OpenCode or
  Goose workers today; must see no behavioral difference in dispatch, status, or recovery commands.
- **Worker agent (Claude):** runs the assigned mission inside the same durable-file protocol
  (`.sergeant-status`, `.sergeant-result`, notifications) every other harness already uses.
- **Human operator:** dispatches, monitors, and troubleshoots Claude workers through existing
  commands (`sgt-watch`, `tmux attach -t sgt-<task-id>`); must not need a harness-specific procedure.
- **Maintainer:** extends the harness registry or launch contract without reintroducing
  harness-specific branches in shared code (`bin/_sgt-harness.sh`, `_deliver_notifications`).

## Problem

**A Claude worker's own completion cannot end its process.** Claude's plain interactive TUI has no
concept of "the mission is over, exit now" — it finishes a turn and returns to its own prompt,
indefinitely. Measured directly: a live, idle `claude` process was found 4 hours 25 minutes after its
task genuinely completed, with over 5 minutes of accumulated CPU time and two monitoring loops still
running, for a single task.

**`_sgt_harness_launch_contract`'s Claude row is entirely unmeasured.** `bin/_sgt-lib.sh` records
Claude's model transport, variant transport, provider scope, and base argv as `unmeasured unmeasured
unmeasured -` — a pinned `--model` fails closed for Claude today, not because the harness lacks the
capability, but because it has never been measured.

**The harness registry's readiness probe already assumes the pane's live process is the harness
itself.** This holds for OpenCode and Goose because they run directly in the pane's foreground for
the whole mission. It must continue to hold for Claude without a Claude-specific fork of
`_sgt_harness_ready`, `_deliver_notifications`, or the pane-identity checks that already gate
ACK/accept/complete.

## Product Principle

Sergeant owns the mission; Claude's native background supervisor owns only the Claude runtime.
Sergeant's durable state (`.sergeant-status`, `.sergeant-result`, notification and lease protocol)
remains the sole authority on mission completion; Claude's own reported state is an auxiliary signal,
never a substitute. The fix reuses the existing worker architecture wherever it already applies
rather than introducing a parallel one for a single harness.

## Outcomes

1. `sgt-dispatch --agent claude` produces a worker that exits on its own once Sergeant's durable
   state reaches a terminal value — no manual process termination required.
2. Existing `_deliver_notifications`, `_watch_progress`, `_watch_drain`'s own polling logic,
   `_finish`, pane-identity, and ACK/accept/complete logic require zero Claude-specific branching
   beyond five named exceptions, none of which touch the steady-state delivery/notification loops
   themselves:
   1. The launch-assembly step (Claude Harness Lifecycle — a value known only after a preceding
      command returns).
   2. The three-layer pinned-model verification logic (Claude Harness Lifecycle; CH-3) — syntactic,
      liveness, and transcript-substitution checks that must run before and immediately after launch.
   3. The unexpected-death respawn+reattach recovery loop (Claude Harness Lifecycle) — restoring the
      worker's foreground slot after a runtime crash Sergeant's own durable state did not record as
      terminal.
   4. A `claude stop` backstop at each of the nine independent termination paths an exhaustive sweep
      found — eight external call sites plus `_finish`'s own in-process backstop, since `_finish`
      runs as the terminal step of every exit path, including a clean `done` exit none of the eight
      external scripts ever observe (Recovery Semantics).
   5. The prior-session-liveness check added to `sgt-recover`/`sgt-respond`'s relaunch flow (Recovery
      Semantics).

   All five are enumerated, not implied away by a general "no branching" claim; this list closing at
   five rather than three was itself a documentation gap in an earlier draft of this PRD, found and
   corrected during implementation — the underlying behavior (CH-1, CH-3, CH-8) was never in
   question.
3. A pinned `--model` works for Claude through the existing harness launch contract, and a launch
   whose pinned model was silently substituted is distinguishable from one that was honored.
4. OpenCode and Goose dispatch, delivery, recovery, and cleanup behavior is unchanged.

## Non-Goals

- Building a generic multi-harness "background session" abstraction. Only Claude's launch mechanics
  differ from OpenCode/Goose today; a second consumer would justify generalizing, not this PRD.
- Delivering notifications through any mechanism other than the existing `tmux send-keys` loop —
  measured to work unchanged once the pane is attached.
- Waking a stopped session with new content. `claude respawn <id>` is scoped to recovery from an
  unexpected runtime death, not to routine delivery — see Terminology and Recovery Semantics.
- Any change to `.sergeant-status`, `.sergeant-result`, notification ID, nonce, ACK, acceptance, or
  action-lease file formats or semantics.
- A Claude-specific permission-bypass policy beyond what OpenCode's `--dangerously-skip-permissions`
  posture already establishes.

## Terminology

- **Background session:** a Claude Code process launched with `--bg`, owned by a per-user supervisor
  independent of the launching terminal, addressable by a short background ID and a full session
  UUID (`sessionId`).
- **Attach:** `claude attach <id>`, which blocks the calling process in a full interactive view of a
  background session's live conversation, rendering exactly as a foreground session would.
- **External stop:** `claude stop <id>`, run from a process other than the one holding `attach`
  open. Measured to cause the attached process to exit on its own.
- **Respawn:** `claude respawn <id>`, which restarts a session (running or stopped) on a new process
  with its conversation and `sessionId` intact, and accepts no new prompt.
- **Terminal durable state:** any value of `.sergeant-status` this codebase already treats as
  terminal (`done` with non-empty `.sergeant-result`, or `failed: <reason>`).

## Claude Harness Lifecycle

`sgt-interactive-worker` calls `_sgt_resolve_agent_launch` once, near the top of the script, to
populate `SGT_LAUNCH_BASE_ARGV`/`SGT_LAUNCH_MODEL_ARGV`/`SGT_LAUNCH_MODEL_ENV` from the static
per-harness contract table. It then starts `_deliver_notifications &`, `_watch_progress &`, and
`_watch_drain &` — all three begin polling and probing the pane *before* the launch argv is even
assembled — and only afterward builds the final argv and makes one call to `"$AGENT" "$@"`. Claude
needs a value in `base_argv` — the background ID — that does not exist until a separate command has
already run and returned, which the static table-driven, single-invocation shape cannot express on
its own. Where that pre-step runs matters as much as what it does: `_sgt_harness_ready_tui` has no
process-name or presentation-string comparison at all (deliberately, per its own cited GH #175 fix)
— it reports ready the moment the pane renders stable, non-blank content twice in a row, with no way
to tell "the `--bg` launch command's own echoed confirmation" apart from "the real attached Claude
TUI." If the bg-launch step ran inside the launch-assembly block, after the loops already started,
that echoed confirmation text is exactly the kind of stable non-blank content the probe would accept
as ready — and a notification delivered at that moment would be typed into whatever the pane shows
next (possibly a bare shell prompt, if `--bg` has already returned and `attach` hasn't run yet), not
into Claude. This requires one narrow, explicit addition to `sgt-interactive-worker`, scoped to the
Claude branch only, placed **before** the three background loops start, not inside the
launch-assembly block below them:

- **Launch.** When `harness == claude`, immediately after `_sgt_resolve_agent_launch` returns and
  *before* `_deliver_notifications &`/`_watch_progress &`/`_watch_drain &` are started, run `claude
  --bg --name <name> "${SGT_LAUNCH_MODEL_ARGV[@]}" "<initial brief>"` as its own explicit command —
  consuming the model argv here, not at the final invocation. Capture the returned background ID and
  `sessionId` from its output and from `claude agents --json`. Only now write the `intended`
  launch-record evidence (moved to after this step, since it did not previously need to wait on
  anything) with the background ID included. Set `SGT_LAUNCH_BASE_ARGV=(attach "$claude_bg_id")` and
  clear `SGT_LAUNCH_MODEL_ARGV` to empty, so the existing generic assembly appends nothing further to
  it. This reordering *narrows* the readiness-probe race but does not by itself close it:
  `_sgt_harness_ready_tui` has no content-stability check, only "alive, non-blank, not the first
  sighting" — so the `--bg` command's own echoed confirmation, still sitting in the pane when the
  loops start their first poll, could itself satisfy readiness before `attach` has painted anything.
  Closing it requires one more explicit step: clear the pane (e.g. a literal screen-clear sent to it)
  immediately after capturing the background ID and immediately before running `claude attach`, so
  the first content either loop can ever observe is `attach`'s own rendering, never the `--bg`
  confirmation text. With both changes together, the three background loops start against a pane
  that has not yet rendered anything Claude-related at all — exactly the same starting condition
  OpenCode/Goose already have — and the existing, unmodified launch-assembly block's final call
  becomes `claude attach "$claude_bg_id"`: the first and only thing that pane renders after the loops
  begin, and the worker's persistent foreground slot from that point on, structurally identical in
  role to today's bare OpenCode/Goose invocation.
- **Verify a pinned model in three layers, since no single check catches every failure shape.** A
  pinned `--model` can be wrong in ways that are syntactic (catchable before any API call) or
  semantic (only Claude's own backend knows), and the `--bg` call itself returns exit `0` in every
  case (see the related research's model-behavior table):
  1. **Pre-flight regex, before `claude --bg` is ever called.** Accept only the confirmed-valid
     shapes — a bare alias (`sonnet`, `opus`, `haiku`, `fable`) or a full model ID
     (`claude-<family>-<version>[-<version>]*`) — and fail closed on anything else. Given the provider
     segment is already stripped and validated before this point (below), a qualified `provider/model`
     string cannot structurally reach this regex via the normal tuple-parsing path; the regex's
     practical value is catching a malformed *model* segment (a typo, a nonexistent version) before
     spending a real API call — the qualified-form failure itself is measured to happen only if that
     stripping step is bypassed or never implemented, which is exactly why the strip-then-validate
     step below is load-bearing, not optional.
  2. **Bounded post-launch liveness check.** Poll `claude agents --json` for the new background ID;
     if `state` reaches `failed` within a short bounded window, treat the launch as failed. This is
     the backstop for any regex-shaped-but-still-invalid value the pre-flight check didn't anticipate.
  3. **Transcript substitution-warning scan.** Neither layer above catches a *valid* alias the
     account is not entitled to — measured to silently substitute another model and complete the
     mission successfully, with no failed state and no non-zero exit code anywhere. The only signal
     is the transcript line `Model "<X>" is restricted by your organization's settings. Using <Y>
     instead.` A pinned model is only "honored," not merely "attempted," once this scan confirms that
     line's absence.
- **Steady state.** The existing `tui` readiness probe, `_deliver_notifications`, `_watch_progress`,
  and pane-identity checks operate against the attached pane exactly as they already operate against
  a foreground OpenCode/Goose session. No Claude-specific delivery code is added here — of this PRD's
  five named exceptions (Outcome 2), launch assembly, model verification, and unexpected-death
  recovery are covered above in this section; the nine-path termination backstop and the relaunch
  prior-session check are teardown/recovery-path concerns covered separately in Recovery Semantics.
- **Termination.** A new watcher, added alongside the existing `_deliver_notifications &`/
  `_watch_progress &`/`_watch_drain &` background loops, polls Sergeant's own durable state. On
  reaching terminal durable state, it runs `claude stop <background-id>`. This causes the attached
  `claude attach` call to exit on its own, `sgt-interactive-worker`'s foreground call returns, and
  the already-installed `_finish` runs unmodified.
- **Unexpected death.** If the recorded Claude session is `stopped` while Sergeant's own durable
  state is **not** terminal, treat it as an unexpected runtime failure, not a normal exit: run
  `claude respawn <background-id>` (restores identical `sessionId`, conversation intact), then
  re-run `claude attach <background-id>` as the worker's new foreground slot. No prompt is
  re-delivered specially; the existing notification loop resumes against the re-attached pane.

## Recovery Semantics

- **Every path that can end a worker's pane or process must also stop its recorded Claude background
  session — an exhaustive sweep of this codebase's termination logic found nine, not the two most
  obvious ones.** A background Claude session is not a child of the worker's process group and is
  invisible to process-tree/process-group signaling and to `tmux kill-pane` alike (Claude Harness
  Lifecycle; the spike's own description of the supervisor). Confirmed present in:
  - `sgt-cleanup`'s `_stop_local_worker`/`_stop_validation_pane` — the primary reaper, gated on
    terminal status.
  - `sgt-watch`'s `_recycle_terminal_worker` — fires automatically on every sync once status is
    `done`/`failed:*`/`drained`, no operator action required; the highest-frequency path and the one
    most likely to leak silently, since nothing is watching for it to fail.
  - `sgt-drain-force`'s force-stop loop — already the weakest existing path (signals a single PID,
    no process-group sweep at all, unconditionally stamps `force-stopped` regardless of whether
    anything was actually reached).
  - `sgt-recover`'s stall-recovery kill — the highest-severity instance, because it kills a pane
    *confirmed still alive* purely because it looks stalled; the stall itself may be the background
    session legitimately still computing.
  - `sgt-respond`'s superseded-pane kill and its own relaunch-failure kills.
  - `sgt-dispatch`'s post-launch rollback kills (three sites, for different dispatch-time failures
    after the pane already exists).
  - `sgt-validate`'s validation-launch rollback — a secondary instance, if `no-mistakes`'s own review
    axes invoke Claude in background mode; lower confidence than the others but the same shape.
  - `sgt-interactive-worker`'s own `_drain_terminate` (cooperative drain).
  - `sgt-interactive-worker`'s own `_finish` — the ninth site, found during implementation rather
    than by the original sweep: it runs as the terminal step of every exit path, including a clean
    `done` exit none of the eight external scripts above ever observe, so it needs its own backstop
    independent of them, not merely a restatement of `_drain_terminate`'s cooperative-drain case.

  Each of these must call `claude stop <id>` for the recorded `claude_background_id`, idempotent (a
  repeat stop on an already-stopped id is a no-op success, not an error), before or alongside
  whatever pane/process action it already takes. This list closing at nine rather than two or three
  is itself the point: a fix scoped to only the instances a prior pass happened to name would leave
  the identical defect in whichever path was checked last.

- **Extend the existing identity-persistence convention; do not invent a new one.**
  `sgt-interactive-worker`'s `_record_worker_identity` already atomically persists `worker_pid`/
  `worker_process_group`/`worker_process_start` to fleet state via tmp+mv, specifically so
  `sgt-cleanup` can terminate escaped descendants after the pane exits. Add `claude_background_id`
  (and `claude_session_id`) to that same function, written immediately after the `--bg` launch
  captures them — the cheapest point to know this identity, since the worker script itself invoked
  `--bg` and holds the id as a live shell variable with zero file I/O needed in-process. Every one of
  the seven other consumers above already has the corresponding fleet-state directory in scope at the
  exact point it kills something today — extending each with one more best-effort read and a `claude
  stop` call is mechanical, not new architecture. `_drain_terminate` and `_finish` are both partial
  exceptions worth naming precisely: each runs in-process — `_drain_terminate` forked from the worker
  itself, `_finish` as the worker's own EXIT-trap handler — so each can use the inherited shell
  variable directly and does not strictly need the persisted file for its own purposes. Both should
  still be persisted regardless, both to keep every consumer's read pattern uniform and because the
  other seven paths are genuinely separate processes with no such inheritance available.

- **A relaunch must check for, and stop, a still-live background session from a *prior* attempt
  before starting a new one against the same worktree — a distinct leak, not the same bug as
  teardown.** `sgt-recover`'s stall-recovery relaunch and `sgt-respond`'s superseded-worker relaunch
  both launch a brand-new worker pane against the same worktree without checking whether the attempt
  being replaced left a background session running. Neither script reads process identity at all
  today (only pane identity), so this has no existing equivalent to extend — it is new logic, not a
  missed read. Left unaddressed, a stalled-but-still-computing background session and a freshly
  relaunched one could run concurrently against the same worktree: a correctness hazard (two agents
  editing the same files), not merely a resource leak. Required check, before either script's `tmux
  new-window` relaunch: read the prior `claude_background_id`; if `claude agents --json` reports it
  genuinely alive (`working`/`blocked`, not already `stopped`/`failed`), stop it explicitly and record
  that a live session was preempted, before dispatching the replacement.

- `claude stop`/`kill` are documented as literal aliases with no described graceful-shutdown window.
  Measured against a real mid-task bash loop, an external stop left no corrupted output and no
  orphaned child processes, but this is evidence the common case is safe, not a guaranteed atomicity
  property — the nine-path backstop above exists precisely because no single termination watcher
  firing first can be assumed.
- The termination watcher must not treat `.sergeant-status=done` alone as sufficient to stop the
  runtime; it must also require `.sergeant-result` to be non-empty, matching `_finish`'s own existing
  empty-result-means-orphaned handling and `templates/worker-brief.md:131-132`'s required write
  order (result before status). This ordering is what makes an external stop on "done" safe — it is
  not new to this PRD, but the termination watcher's correctness depends on it and must not silently
  assume the reverse order is also safe.
- A session that cannot be identity-verified (recorded `id`/`sessionId`/`cwd` mismatch) falls back to
  existing orphan/recovery semantics unchanged; this PRD does not introduce a parallel Claude
  recovery path in `sgt-respond` or `sgt-recover` beyond the prior-session check above.

## Privacy and Security Constraints

- Mission bodies and human responses remain in existing durable files; Claude receives only the
  fixed ID-bearing nudge through the existing notification channel, unchanged.
- Session names (`--name`) are display metadata only, never an ownership credential; every
  destructive operation (`stop`, worktree retirement) verifies the recorded `id` and `sessionId`
  against the live session before acting.
- No permission-bypass policy is introduced for Claude beyond what already exists for OpenCode; using
  `bypassPermissions` for Claude requires the same one-time interactive disclaimer acceptance Claude
  Code itself already requires, documented as a per-machine/account setup step, not something a
  dispatch can silently satisfy.
- `claude rm` is never called automatically; stopping ends resource consumption without discarding
  transcript history a later investigation might need.

## Compatibility and Rollout

- `_sgt_harness_launch_contract`'s `claude` row moves from `unmeasured unmeasured unmeasured -` to:
  model transport `argv-bare` (bare alias or full ID as direct argv, no provider qualification —
  measured to fail the session, not merely to be rejected, if a `provider/model` form is passed
  through). Sergeant's existing pinned-tuple grammar (`provider/model[:variant]`, `_SGT_AGENT_MODEL_RE`)
  is unchanged; for Claude specifically, the provider segment is validated against a fixed accepted
  value and stripped before the model segment reaches the pre-flight regex in Claude Harness
  Lifecycle — the qualified form must never reach `--model` itself, per the measured failure above.
  Variant transport remains `unmeasured` (no variant selector was found or tested for Claude, and the
  field fails closed per the existing contract's own doctrine, not because Claude is known to lack
  it) — a pinned tuple carrying a variant for the Claude harness fails closed before launch, the same
  as any other unmeasured-variant harness would. `base_argv` set at launch time per the Lifecycle
  section above rather than as a fixed table value, since the background ID is only known after
  `--bg` returns.
- `_sgt_resolve_agent_launch`'s `case "$model_transport"` block currently implements only
  `argv-qualified` and `env-goose`; it needs a new `argv-bare)` arm to actually consume the new
  contract value, even though the Claude-specific launch step (above) immediately clears
  `SGT_LAUNCH_MODEL_ARGV` afterward — the shared function still needs to populate it correctly in the
  first place, and a future harness reusing `argv-bare` should not depend on Claude's own downstream
  clearing step to paper over a missing shared-code case.
- OpenCode and Goose rows, `_sgt_harness_ready`, and `_deliver_notifications` are unchanged.
- New capability gate: verify `claude --bg`, `claude attach`, `claude stop`, `claude agents --json`,
  and `claude respawn` are all present (`claude --help`) before accepting a Claude dispatch. The
  bypass-permissions disclaimer precondition is documented (Privacy and Security Constraints above)
  but is not independently checkable by this gate today — the related research was unable to safely
  reproduce the refusal's exact text/exit code on any available machine (every one had already
  accepted the disclaimer through prior use) to pattern-match against; until a fresh machine/account
  confirms it, the gate documents the precondition as a required manual per-machine/account setup
  step rather than asserting an automated check it cannot back with a measured signature.
- No project YAML migration is required; no change to fleet state schema.

## Measurable Acceptance Criteria

1. **CH-1:** Given a Claude worker whose mission reaches `.sergeant-status=done` with a non-empty
   `.sergeant-result`, the worker process exits within the termination watcher's poll interval with
   no manual intervention, and `_finish` runs its genuine-completion branch.
2. **CH-2:** `_deliver_notifications`, `_watch_progress`, `_watch_drain`'s polling logic, and the
   pane-identity checks in `sgt-interactive-worker` contain no `if [[ "$harness" == "claude" ]]`-style
   branch. The launch-assembly step, the three-layer model-verification logic, the unexpected-death
   respawn+reattach loop, the nine-path termination backstop, and the relaunch prior-session check
   (all in Claude Harness Lifecycle / Recovery Semantics) are the five named exceptions, not
   violations of this criterion — see Outcome 2.
3. **CH-3:** A pinned `--model` is verified in the three layers Claude Harness Lifecycle specifies:
   an out-of-shape value (e.g., any `provider/model` qualified form) is rejected before `claude --bg`
   is called; a shaped-but-invalid value is caught by the post-launch liveness check within its
   bounded window; and a valid-but-unentitled value is caught by the transcript substitution-warning
   scan even though the mission itself completes successfully. `claude agents --json` state alone is
   verified to be insufficient for the third case and must not be treated as sufficient evidence on
   its own.
4. **CH-4:** A fake-CLI test suite covers: identity persistence across `respawn`; unknown
   `agents --json` fields ignored; unknown state values fail closed; `stop`-causes-`attach`-to-exit
   with exit code `0`; a message queued while the session is busy is delivered after the current
   turn; and, for each of the nine independent termination paths named in Recovery Semantics
   individually (`sgt-cleanup` ×2, `sgt-watch`, `sgt-drain-force`, `sgt-recover` stall-kill,
   `sgt-respond` supersede-kill, `sgt-dispatch` rollback, `sgt-validate` rollback,
   `sgt-interactive-worker`'s own `_drain_terminate`, and `sgt-interactive-worker`'s own `_finish`),
   that its `claude stop` backstop is idempotent whether or not the termination watcher already
   stopped the session first.
5. **CH-5:** A real-Claude contract test reproduces the original defect end to end — mission written,
   `.sergeant-status=done` set after `.sergeant-result`, worker exits unattended, no live `claude`
   process or monitoring loop remains — and is re-run against every future Claude Code version bump
   before that version is trusted in production, since the whole fix depends on the measured
   `stop`-causes-`attach`-to-exit behavior continuing to hold.
6. **CH-6:** An OpenCode and a Goose dispatch, run before and after this change, produce identical
   observable behavior (dispatch, delivery, recovery, cleanup).
7. **CH-7:** The capability gate's documentation (not an automated check — see Compatibility and
   Rollout) names the bypass-permissions disclaimer as a required one-time per-machine/account setup
   step, with a pointer to the exact command (`claude --dangerously-skip-permissions`, run
   interactively once) that satisfies it. This criterion is superseded by an automated check once a
   fresh machine/account measurement records the refusal's exact text and exit code.
8. **CH-8:** Given a repo whose recorded `claude_background_id` is still genuinely alive
   (`working`/`blocked`) at the moment `sgt-recover`'s stall-recovery relaunch or `sgt-respond`'s
   superseded-worker relaunch would otherwise fire, that session is explicitly stopped and the
   preemption is recorded before the replacement worker is dispatched — no two Claude processes ever
   run concurrently against the same worktree as a result of either relaunch path.

## Delivery Boundary

This PRD covers the Claude harness's launch, delivery reuse, termination, and recovery lifecycle
only. It does not cover a generic multi-harness background-session abstraction (Non-Goals), and it
does not cover the `SessionStart`-hook-based context-injection mechanism recorded as a rejected
approach in the related research — that remains available for a future PRD if a requirement emerges
that the existing attached-pane delivery path cannot satisfy.
