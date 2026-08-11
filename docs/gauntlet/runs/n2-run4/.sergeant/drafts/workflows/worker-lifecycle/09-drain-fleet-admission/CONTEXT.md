# 09-drain-fleet-admission

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** a drain is activated, optionally with --wait

**Outcome:** new pane admission is refused immediately; existing workers are allowed to finish cooperatively rather than being force-terminated on timeout

**Statement (the operative rule):** A drain refuses new pane starts for the matching scope while still storing responses generation-safely for later delivery; `--wait` activates the drain, then waits for live workers in scope to finish their current turn and exit, and on timeout leaves the drain active, exits nonzero, and names the unresolved workers without terminating any of them.

## What must become true here (durable outcome)

New pane admission is refused immediately; existing workers are allowed to finish cooperatively rather than being force-terminated on timeout — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0154`: A worker is only treated as finished when its exit can be proven, so a worker whose identity was never recorded blocks the wait rather than being silently counted as drained.
- `BU-0348`: Cooperative drain first performs a durable task tracker handoff and finalizes the accepted action lease (from the agent's own proof, or explicitly recorded as still pending) before publishing the drained status and terminating; the drain path never invents a result, leaving status honestly as the nonterminal "drained" with any prior result file removed.
- `BU-0423`: Drain admission is checked only at the point a new worker pane/process would be launched; drain never blocks storing a response or delivering it to an already-live pane.
- `BU-0424`: If drain is active when a relaunch would otherwise occur, the worker response-delivery step records a generation-bound drain_held marker (so an explicit undrain can re-evaluate this exact waiting worker once) and exits successfully having stored the response without relaunching.
- `BU-0489`: Global or project drain blocks a stall relaunch; on drain, recovery escalates to needs_input without stamping the one-shot recovery marker, so recovery remains retryable once undrain occurs.
- `BU-0515`: While a drain is active, response-driven relaunches and stall recovery are refused for matching projects; responses due during the drain are stored generation-safely for later delivery rather than dropped.
- `BU-0516`: --wait activates the drain before it begins waiting, so no new work can be admitted during the wait for live workers in scope to finish their current turn and exit.
- `BU-0517`: --wait exits nonzero on timeout, leaves the drain active, and names the unresolved workers; it never terminates any worker itself.
- `BU-0519`: A project name and --global are mutually exclusive scopes for the drain step; combining them is a hard error, because silently letting --global win would escalate a one-project pause into blocking every project.
- `BU-0522`: --timeout and the drain-wait timeout/interval environment variables must be a non-negative whole number of seconds; an unvalidated value could otherwise silently become 0 inside arithmetic (turning a bounded wait into an immediate timeout) or abort `sleep` midway with the drain already active.
- `BU-0523`: A worker's exit is never inferred from missing identity: a pane that is running but whose identity was never recorded (the interactive worker harness skips recording it when `ps` cannot report a pgid or start time) blocks the drain wait rather than being silently counted as finished.
- `BU-0524`: A worker with a currently-live PID is only treated as 'gone' once its recorded process-start token no longer matches the process now holding that PID (proving PID reuse); when either side's start time cannot be obtained, the tokens are incomparable and the worker stays 'unverifiable' rather than being declared gone.
- `BU-0525`: force-stopped, orphaned, and failed* worker statuses are excluded entirely from --wait's cooperative-drain tracking, because a cooperative wait can never resolve them by waiting — resolving them is the forced-drain step's job, not the drain step --wait's.
- `BU-0526`: A worker reported 'done' or 'drained' is trusted at its word (treated as resolved) for --wait purposes only when its liveness is unverifiable and not contradicted by an actually-live process; a 'done'/'drained' status accompanied by a provably live process still blocks the wait, because the interactive worker harness writes 'drained' before killing its pane and before its exit-path handoff runs.
- `BU-0527`: A nonterminal worker with no recorded project cannot be attributed to any drain scope, so it is reported as unresolved by --wait rather than silently skipped.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0518`: --wait is refused unless the mode is 'drain' (not with --undrain or --status, since there is nothing to wait for), and --timeout is refused unless --wait was also given.
- `BU-0520`: A project name argument to the drain step must match ^[A-Za-z0-9][A-Za-z0-9._-]*$; anything else is refused.
- `BU-0529`: Locked drain mutations (write/clear) pass caller-supplied text such as --reason as argv elements to the locked helper, never interpolated into a shell string, so it is never re-parsed by the shell.
- `BU-0539`: Removing a drain explicitly restores admission for that scope; the undrain step is idempotent — undraining a scope that is not currently drained still exits 0.
- `BU-0540`: --global and an explicit project target are mutually exclusive for the undrain step.
- `BU-0541`: A project name argument to the undrain step must match ^[A-Za-z0-9][A-Za-z0-9._-]*$; anything else is refused before any drain state is touched.
- `BU-0542`: _sgt_is_drained treats an empty or syntactically-invalid project name as absent, checking only the global drain in that case, rather than erroring or attempting to match it against a project drain file.
- `BU-0562`: A drain file is written atomically via a temp-file-then-rename, and drain activation is determined solely by the file's existence — the reason/actor/deadline fields it may also carry are for human inspection only and are never consulted to decide whether a drain is active.
- `BU-0563`: Removing a global or per-project drain file runs under the admission lock and is idempotent — safe to call when no drain is currently active.

