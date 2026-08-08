# Claude Background Harness — Research Spike

**Date:** 2026-08-07
**Decision:** Launch Claude workers with `claude --bg`, then hold the worker's persistent slot on
`claude attach <id>` in place of a bare harness invocation. End a mission with an external `claude
stop <id>`; this is measured to make the attached call exit on its own, which is what makes
`sgt-interactive-worker`'s existing `_finish`/`EXIT` trap actually fire. Deliver every notification
through the existing `tmux send-keys` loop, unmodified — it already works once the pane is attached.
Do not use `claude --resume <id> --bg`, `claude -p --resume`, or a `SessionStart`-hook-based
delivery design; each is measured or documented to fail or conflict with an existing constraint (see
Rejected approaches).

## Scope and evidence

This spike measures Claude Code 2.1.220's native background-session CLI
([code.claude.com/docs](https://code.claude.com/docs)) against this repository's own
`bin/_sgt-lib.sh`, `bin/_sgt-harness.sh`, `bin/sgt-interactive-worker`, `docs/using-sergeant.md`, and
`templates/worker-brief.md`, pinned at `efcc96639ab83caf908e651bbedc0790487620a0`. Every claim about
`claude` CLI behavior below was reproduced live against the installed binary, not read from
documentation alone; every claim about this codebase was read from the cited file.

## The defect this spike is scoped to fix

`bin/sgt-interactive-worker` installs `trap '_finish $?' EXIT`, then ends with a plain foreground
call, `"$AGENT" "$@"` (not `exec`). For Claude specifically, `_sgt_harness_launch_contract`'s row is
`unmeasured unmeasured unmeasured -` — `base_argv` is `-`, meaning Claude launches with no arguments
at all, as a bare interactive TUI. That TUI can write `.sergeant-status=done` and sit at its own
prompt indefinitely without the process itself returning, so the installed trap never fires.

Measured directly, independently of this spike: a live, idle `claude` process was found 4 hours 25
minutes after its task had genuinely completed, with over 5 minutes of accumulated CPU time and two
monitoring loops (`_deliver_notifications`, `_watch_progress`) still running alongside it, for a
single task. Fixed at the time by killing the process by hand, which is not a supportable steady
state: every Claude-harness dispatch requires the same manual intervention before `_finish`, drain,
watch-recycle, or cleanup can proceed.

## Measured findings

### `claude --bg` launches a real, detached background session

`claude --bg --name <name> "<prompt>"` returns immediately, printing a short background ID. The
session is owned by a per-user supervisor process, independent of the launching terminal, and
appears in `claude agents --json` with fields including `id`, `pid`, `cwd`, `kind`, `sessionId`,
`name`, `status`, `state`, and (observed in every record but undocumented) `startedAt`. A session
that is `working`, `blocked`, or has a terminal attached stays alive; an unattended **finished**
session is stopped by the supervisor after about an hour unless pinned — not a concern for the
design below, since the worker stays attached (and therefore active) for the mission's whole
duration.

### `claude attach <id>` is the load-bearing primitive

`claude attach <id>` blocks the calling process in a full interactive TUI, rendering the same way a
freshly-launched foreground session would. Measured directly:

- **An external `claude stop <id>`, run from a separate process while another process holds
  `attach` open, causes the attached process to exit on its own**, printing `Session <id> has
  exited.`, with exit code `0`. This is the mechanism the whole fix depends on.
- A full end-to-end pipeline reproduces the intended outcome: an agent writes `.sergeant-result`
  then `.sergeant-status=done` (the order `templates/worker-brief.md:131-132` already mandates), a
  separate watcher process polls and sees `done`, runs `claude stop <id>`, the attached call exits
  with code `0`, and a `_finish`-equivalent function reading that exit code and the status file
  takes the genuine-completion branch. A first attempt with the write order reversed (status
  written before result) was correctly caught by `_finish`'s own existing empty-result-means-
  orphaned handling — confirming the mechanism's safety depends on that existing, already-mandated
  ordering, not on anything new.
- A message sent via `tmux send-keys` to an attached session that is **actively generating** is
  queued ("Press up to edit queued messages"), not dropped or corrupted, and delivered as the next
  turn once the current one completes. A session that stalls mid-turn (observed: frozen at a fixed
  token count for close to two minutes) can be recovered by sending `Escape` first, then the
  message — the queued message is delivered as the answer to Claude's own "What should Claude do
  instead?" prompt.
- Detaching from `attach` never stops the session (`←`, `Ctrl+Z`, `/exit`, double `Ctrl+C`/`Ctrl+D`
  all leave it running) — not required by the design below, since the worker never detaches except
  via the external `stop` above, but relevant to any future troubleshooting flow that wants to peek
  at a live session without ending it. `←` always returns to Agent View; `Ctrl+Z` returns to wherever
  the attach was run from (a plain shell, if launched the way this design launches it) and has no
  documented on/off toggle, unlike `←`'s `leftArrowOpensAgents` setting — prefer `Ctrl+Z` for any
  scripted use.

### `claude respawn <id>` is identity-preserving, for recovery only

`claude respawn <id>` restarts a session, running or stopped, on a new OS process but the identical
`sessionId` — confirmed by direct before/after comparison. It takes no new prompt; it resumes
exactly where the conversation left off. This makes it correct for recovering an **unexpectedly**
stopped session (the underlying process crashed) but not a routine wake mechanism — see Rejected
approaches for why an earlier design considered it one.

### `--model` accepts a bare alias or full ID directly as argv, and only that

`claude --bg --model <alias-or-id> "<prompt>"` pins the model with no qualification needed. This
maps onto the existing `_sgt_harness_launch_contract` shape (`model_transport` field) as a new
`argv-bare` value, alongside the existing `argv-qualified` (OpenCode) and `env-goose` (Goose)
values.

Three distinct model-launch behaviors were measured, and all three return CLI exit code `0` — the
launch call's own exit code is never sufficient evidence a pinned model actually took effect:

- **A provider-qualified value** (`anthropic/claude-sonnet-5`, matching Sergeant's own pinned-tuple
  grammar) launches, then the session fails **before attempting the task at all**: it moves to
  `"state": "failed"` in `claude agents --json` within a few seconds of launch — the exact elapsed-time
  label shown in the TUI is randomized flavor text (observed as both "Baked for 0s" and "Worked for
  0s" across separate reproductions) and is not itself stable evidence, only the `state` value and the
  message text are — and the transcript's first and only line is `There's an issue with the selected
  model (anthropic/claude-sonnet-5). It may not exist or you may not have access to it.` No process is
  left running afterward. This means `provider_scope` for Claude is not `any` the way it is for
  OpenCode — Claude's `--model` grammar has no provider-qualification syntax at all.
- **A valid, entitled bare alias or full ID** (`sonnet`) launches and completes the task normally.
- **A valid alias the account is not entitled to** (`opus`, on an account without Opus access)
  launches, prints a visible but non-fatal warning — `⚠ Model "opus" is restricted by your
  organization's settings. Using claude-sonnet-5 instead.` — and then **completes the task
  successfully on the substituted model**, ending in a normal, non-`failed` state. A successful
  mission is therefore not proof the pinned model was honored; only scanning the transcript for this
  exact substitution warning reveals it. This is the same shape of gap the existing launch contract
  already names for OpenCode's variant pin (`variant_verified=false` — "TRANSPORT ONLY, NOT
  VERIFICATION") and applies equally to Claude's model pin.

`variant_transport` was not measured — no `--agent`-style variant selector or equivalent was tested
for Claude, and none is documented. It remains `unmeasured` and fails closed for a pinned variant,
per the existing contract's own doctrine, not because Claude is known to lack the capability.

### A real provisioning precondition: the bypass-permissions disclaimer

Per Anthropic's own documentation (`code.claude.com/docs/en/agent-view.md`): `claude --bg
--permission-mode bypassPermissions` (or `--dangerously-skip-permissions`) is refused until a human
has run `claude --dangerously-skip-permissions` interactively at least once on that machine/account,
to accept a one-time bypass disclaimer. This is a one-time, per-machine/account setup step, not
something a dispatch can satisfy itself.

**The exact refusal text and exit code for the not-yet-accepted case were not independently
reproduced** — every machine used for this spike had already accepted the disclaimer through prior
normal use, and deliberately un-accepting it to test the refusal was judged too disruptive to that
environment to attempt. Treat the refusal condition as documented, not measured, until a genuinely
fresh machine/account is available to confirm the exact message and exit code — see the PRD's CH-7
for how this limits what the capability gate can check today.

## Rejected approaches

### `claude --resume <id> --bg "<prompt>"` for delivery

Measured under both identifier forms, and the two failure modes are materially different:

- **Full session UUID:** silently ignores the resume target and launches an unrelated new session
  instead — no error, no refusal. `claude agents --json` shows two independent sessions.
- **Short background ID:** does not create a working session at all. It opens Claude's interactive
  session picker, searches for the literal ID string, and gets stuck at `No sessions match
  "<id>"` — a `--bg` process can never supply the keystroke needed to leave a picker, so this does
  not resolve on its own. `claude agents --json` reports this as `"state": "blocked"` (settled within
  roughly 15 seconds and unchanged through a 45-second observation window in one reproduction — not
  `"working"`, and not confirmed to hold at exactly this value across every version/timing). `blocked`
  is a real Sergeant signal already, which weakens but does not remove the concern: a stuck picker
  reporting a generic `blocked` state with no `waitingFor` reason is still not distinguishable from
  genuine `needs_input` without inspecting the transcript, and this design's own monitoring (§11-style
  mapping) treats bare `blocked` as "not itself terminal, re-read Sergeant durable state" rather than
  as an error — so a wrong invocation of this rejected mechanism would still sit unresolved rather
  than surface as a clear failure. Re-verify the exact reported state on any future Claude Code
  version before relying on this distinction either way.

Neither form is documented, and neither should be reachable in the implementation this PRD
authorizes.

### `claude -p --resume <id> "<prompt>"` for delivery

Real and scriptable (`docs/en/sessions.md`'s own "ask an existing session a question" pattern), but
rejected for two independent reasons, not one:

- **Capability:** measured against a session still registered as a background agent (the state this
  design's own worker stays in for the whole mission), and the two identifier forms fail two
  *different* ways — the same distinction that matters for `--resume --bg` above applies here too:
  - **Full session UUID:** `claude -p --resume <uuid> "<prompt>"` is refused with `Error: Session
    <uuid> is currently running as a background agent (bg). Use \`claude agents\` to find and attach
    to it, or add --fork-session to branch off a copy.`, exit code 1. It only succeeds once the
    session has already been stopped.
  - **Short background ID:** fails earlier, at identifier parsing, before the "is it running"
    check is ever reached — `claude -p --resume <short-id> "<prompt>"` gives `Error: --resume
    requires a valid session ID or session title when used with --print. Usage: claude -p --resume
    <session-id|title>. Provided value "<short-id>" is not a UUID and does not match any session
    title.`, exit code 1. Since the short background ID is what this design's own Terminology and
    Lifecycle sections use throughout, this form is the one an implementer would actually reach for
    first, and it never reaches the "background agent" refusal at all — it is not usable in this
    form regardless of the session's running state.
- **Principle:** `docs/using-sergeant.md:75-77` states, verbatim: *"Workers always run as persistent
  interactive TTY sessions. Sergeant never starts one-shot run, prompt, print, or automatic modes."*
  `-p`/`--print` is exactly the one-shot mode this already forbids, independent of whether it would
  otherwise work.

### `SessionStart` hook + `--settings`-injected fleet config for delivery

Built and measured working: a hook supplied via `claude --bg --settings <path>` at launch fires on
every subsequent `respawn` with `source:"resume"`, without the flag needing to be reapplied, and can
inject `additionalContext` (e.g., pending notification content) automatically. Rejected once the
simpler `attach`-based design was found to need no new delivery code at all — this mechanism remains
available if a future requirement needs to inject context Sergeant cannot deliver through the
existing pane (e.g., context needed before the worker is first attached), but nothing in this PRD's
scope needs it.

### Ending the mission by making Claude's own TUI exit, or by removing tmux for Claude entirely

Not Sergeant's decision to make about a third party's TUI, and would force unrelated rewrites of
pane identity, recovery, watch, recycle, and cleanup for no measured gain — every one of those
subsystems already works unchanged once the worker's foreground slot is `attach` instead of a bare
invocation.
