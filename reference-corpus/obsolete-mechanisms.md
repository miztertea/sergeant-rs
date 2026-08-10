# Obsolete-Mechanism Roll-Up

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`obsolete-mechanisms.md`). Sourced from `synthesis.md` §4, which is itself the
proposal's §8.2-designated "obsolete-mechanism stress test": the `dispatch`
skill's tmux/sentinel/worker-Bash machinery must have its durable procedure
separated from the Bash architecture that carried it, and that separation must
itself be reviewed (`synthesis.md` §4's own "Separation review" section, kept
below).

28 behavior units, nine mechanism clusters (M1–M9). Governing rulings this
milestone does not re-litigate: **D2** (no daemon TTY/pane — headless
`claude -p [--resume]` turns), and CLAUDE.md's *One owner*, *the journal is the
only truth*, and *clients are equal* invariants. Every cluster below states
what the mechanism *was* (Bash/tmux implementation detail — not durable
behavior per the contract's binding rule that mechanism is not behavior),
*why* sergeant-rs's architecture replaces it structurally (a Rust-level
invariant that makes the old mechanism's job unnecessary, not merely a
reimplementation of it), the *surviving policy* (the rule that remains
violable and must still be honored), and *where that policy now lives* in this
corpus or the sergeant-rs codebase.

---

### M1 — Coordinator-pane binding and identity proof
**What it was:** managed/named tmux-pane modes; verifying a pane against the
live server; walking process ancestry to prove the coordinator's session
really runs where it claims; launching the coordinator from inside a session
so its governing instructions load.
**Units:** BU-P1-061, BU-P1-095, BU-P6-131, BU-P8-050, BU-P8-063.
**Replaced by:** the loopback HTTP/SSE API with bearer auth (CLAUDE.md
*Clients are equal*) — the daemon, not a pane, owns all client identity; there
is no pane for a client to bind to or spoof.
**Surviving policy:** never trust a self-asserted execution identity — verify
it against the authority that would actually know, and refuse before creating
any durable state; an execution context must have its governing instructions
loaded and be nameable by later operations.
**Where the policy now lives:** already stated at Article IV of
`permanent-instructions.md` ("A live, unreleased owner is never displaced by a
claim; a claimant must prove it truly runs where it claims to... never by
asserting an environment variable" `[BU-P8-088]`) — this cluster's own
identity-proof mechanism is the direct ancestor of that line, generalized
past tmux. **No re-encoding needed**; recorded here so the lineage is not
lost.

### M2 — Pane as the worker's process
**What it was:** one tmux window per repository; attach-to-observe as the
only liveness/output channel; persistent-interactive-only launch modes;
process-group signalling; killing the whole pane tree on termination.
**Units:** BU-P5-064, BU-P5-069, BU-P6-112, BU-P7-008, BU-P8-064, BU-P7-106.
**Replaced by:** headless per-turn processes owned by the daemon (D2); the
TUI/dashboard (M6 milestone) for observation instead of pane attachment.
**Surviving policy:** durable session identity is separate from process
lifetime; live observation of in-progress work remains a first-class
requirement; terminate the whole tree, never one shell; the harness-uniform
lifecycle contract (launch, terminal states, recovery, races) is durable even
though its tmux carrier is not.
**Where the policy now lives:** this is, almost verbatim, CLAUDE.md's own
top-level architecture invariant — *"Work state ≠ process state. A Claude
'session' is a durable conversation identity; the OS process exists per
turn."* — plus the M6-delivered TUI/dashboard for live observation, and W8
`dispatch`/W9 `worker-mission`/W10 `respond-to-worker`/W11
`recover-stalled-worker` in `synthesis.md` §1, whose stages are written to the
same durable-outcome standard regardless of what launches the worker.
**Caveat (§4's own "passes only partially"):** "persistent interactive
session only" is not itself a durable policy — it *is* the mechanism, and D2
structurally reverses it (conflict **X6** in `synthesis.md` §6 records the
one partition, BU-P1-057, that classified the reversed rule as still live).

### M3 — Pane as the notification channel
**What it was:** tmux text injection as the delivery transport; degrading to
a callback when the pane is gone; an ID-bearing nudge retry loop; a reader
that displays but never executes injected content.
**Units:** BU-P1-096, BU-P6-029, BU-P8-066.
**Replaced by:** journal + SSE/API delivery (CLAUDE.md *Clients are equal* —
clients hold no state and read only through the API).
**Surviving policy:** the durable record is the source of truth, and a
transport failure is never a recording failure; delivered content is never
executable; initial-work delivery must be idempotent and crash-safe.
**Where the policy now lives:** CLAUDE.md's *the journal is the only truth*
invariant; Article X of `permanent-instructions.md` ("The durably recorded
event is the source of truth for a worker update; any live delivery transport
is an optional layer on top" `[BU-P6-028]`). The idempotent/crash-safe
delivery half is **not** an engine gap — `synthesis.md` §4 notes this
explicitly, because journal durability plus resumable session identity
already meets it; the rejected/re-homed engine-gap cluster **G9**
(`synthesis.md` §5) makes the same point from the opposite direction (four
claims asking the runtime to *own* crash-safe publication were rejected
because it already does, structurally).

### M4 — Pane as the liveness signal
**What it was:** readiness declared by two consecutive stable pane renders;
readiness defined without referencing UI strings; pane output treated as the
primary progress evidence.
**Units:** BU-P6-067, BU-P6-068, BU-P6-106.
**Replaced by:** turn completion is the signal — there is no keystroke to
land, because a headless turn's process exits when the turn ends (D2).
**Surviving policy:** a supervisor's liveness is not the wrapped work's
progress; readiness must never depend on a presentation string or an
executable name; the two-observations-over-fixed-delay technique is worth
recording so a future attach-style feature does not have to re-learn it from
scratch.
**Where the policy now lives:** Article IV of `permanent-instructions.md`
("Do not infer progress from liveness... require recent, meaningful progress
evidence with a defined fallback chain" `[BU-P1-037, BU-P1-047, BU-P8-072]`);
the `capability-probe` and `harness-registry` shared helpers in
`helper-map.md` ("the readiness probe never depends on a UI string"). The
two-observations technique itself is not adopted by name anywhere yet — it is
recorded here, not lost, as prior art for a future attach-style feature (no
current workflow needs it, so no shared-context or helper claims it).

### M5 — Loose worktree files as durable state
**What it was:** the worker brief delivered as a plain Markdown file drop;
pane-identity matching used to decide whether an acknowledgement was genuine;
legacy on-disk marker migration logic.
**Units:** BU-P5-063, BU-P6-031, BU-P6-077.
**Replaced by:** journaled workflow binding and stage context — the resolved
`WorkflowDefinition`, including stage context text, is journaled in
`workflow.bound` before execution (`reference/proposal-next-iteration-icm-workflows.md`
§3.3); CLAUDE.md's *the journal is the only truth*.
**Surviving policy:** starting context must be durably and replayably carried
before the actor begins; a state transition may only be acknowledged by its
verified owner; when migrating durable state, write only the absent fields
and cross-check every derivation independently rather than trusting the
migration wholesale.
**Where the policy now lives:** the `workflow.bound` journal event itself
(the brief-as-loose-file problem cannot recur because the engine journals the
resolved definition, not a file reference); the `owned-write` shared helper
in `helper-map.md` (stage-to-candidate, verify identity, atomic-rename,
record-owned-path is exactly "acknowledged only by its verified owner"
generalized past pane-identity matching).

### M6 — Detached background harness sessions
**What it was:** stopping a live `--bg` session before relaunch, before
force-stop, before a response-driven relaunch; coordinator-liveness polling by
PID and recorded start time.
**Units:** BU-P6-041, BU-P6-043, BU-P6-074, BU-P6-081.
**Replaced by:** headless turns that exit with the turn itself; the daemon is
the only long-lived process owner (D2 — no daemon TTY/pane, so there is no
detached background session for the daemon to manage the lifecycle of).
**Surviving policy:** never run two concurrent processes against one work
surface across a relaunch; every termination path must account for
out-of-process-group resources it created; an orphaned run must not block
forever on a dead supervisor.
**Where the policy now lives:** Article X of `permanent-instructions.md`
("Recovery is one-shot per attempt: it escalates to needs-input rather than
retrying indefinitely" `[BU-P8-081]`); the `process-identity` and
`action-lease` shared helpers in `helper-map.md` (PID-reuse-safe identity
checks and exactly-once settlement are the structural replacement for
PID/start-time liveness polling).

### M7 — Pane-scoped rollback
**What it was:** killing only the one tmux pane this specific invocation
created; disarming a trap once dispatch had succeeded.
**Units:** BU-P6-127.
**Replaced by:** journaled per-Work ownership — the engine can name exactly
what a given Work created because it is all in the journal, not inferred from
"which pane did I open."
**Surviving policy:** roll back exactly what this invocation created and can
still prove it owns; never touch pre-existing or concurrently replaced state.
**Where the policy now lives:** Article IV of `permanent-instructions.md`,
almost verbatim ("Rollback undoes only what this invocation created and can
still prove it owns; preexisting, reused, or concurrently replaced state is
preserved untouched" `[BU-P8-090]`).

### M8 — Response delivery/acknowledgement split
**What it was:** a bounded acknowledgement-timeout window, after which
exactly one relaunch was the documented recovery action.
**Units:** BU-P6-076.
**Replaced by:** a turn either completes or its process exits — delivery and
consumption collapse into one event rather than needing a separate
timeout-then-relaunch protocol layered on top of a pane.
**Surviving policy:** never leave the operator with no supported next action.
**Where the policy now lives:** Article IV ("A pending response is never
overwritten... if a response was already applied, converge by rerunning the
same acknowledgement command" `[BU-P8-097]`) and Article X ("Recovery is
one-shot per attempt: it escalates to needs-input rather than retrying
indefinitely" `[BU-P8-081]`) of `permanent-instructions.md` together cover
this — the operator always has a named next action (rerun to converge, or
escalation).

### M9 — Shell-distribution targets
**What it was:** the tmux window title used as the stage label; a Bash-3.2
runtime-proof suite that re-execs itself inside a pinned `bash:3.2` Docker
image to prove portability by parsing.
**Units:** BU-P8-062, BU-P8-102.
**Replaced by:** a compiled binary (`sgt`), and the engine's own `WorkRun`
stage field (`reference/proposal-next-iteration-icm-workflows.md` §3.2 — "The
active workflow stage is not encoded as another Work state. It is stored
separately in `WorkRun`") — a compiled binary has no window title to derive a
label from, and needs none, because the stage is a first-class durable field.
**Surviving policy:** every dispatched execution carries a named stage — this
is already structurally provided; parsing proof is not runtime proof (a
Bash-3.2 *parse* check does not establish the same thing a compiled binary's
own test suite does).
**Where the policy now lives:** the `WorkRun.stage` field itself (already-met
structurally, nothing further to encode); the stage-label half needs no
constitution line because the engine cannot omit a stage the way a
hand-written script could forget to set a pane title.
**Contested (X5, unresolved):** whether the Bash-3.2 portability target this
cluster made obsolete (`BU-P8-102`, this cluster) is actually obsolete, or
whether it remains a live invariant every runtime path must satisfy
(`BU-P7-033`, `BU-P7-071`, `BU-P1-065` — classified `agents-invariant`, not
obsolete, by a different partition). sergeant-rs itself, as a compiled Rust
binary, is the strongest evidence for this cluster's side of the argument, but
the corpus records both citations rather than silently picking one — see
`permanent-instructions.md` Article X's own X5 note and `synthesis.md` §6 X5.

---

## Separation review (§8.2's requirement that the separation itself be reviewed)

The test applied to every one of the 28 units: *remove the mechanism
entirely — does a rule remain that could be violated?*

- **M3, M4, M5, M6, M7, M8** all pass outright — a violable rule remains in
  every case, restated above and pointed at its new home.
- **M2** passes only partially: "persistent interactive session only" is not
  itself a durable policy, it *is* the mechanism, and D2 reverses it outright.
  The rest of M2's cluster (durable identity separate from process lifetime,
  live observation as a first-class requirement, kill the whole tree, the
  uniform lifecycle contract) does pass.
- **M1**'s durable policy is real but is already stated at Article IV of
  `permanent-instructions.md` and does not need re-encoding — recorded here
  only for lineage, not as a second source of the same rule (Article VII's own
  single-owner rule would otherwise be violated by this document).
- **M9**'s stage-label policy is already satisfied structurally by the
  engine's `WorkRun.stage` field; there is nothing left to encode as policy.

**Net finding:** of the 28 mechanism units, nine carry a policy that is not
already stated elsewhere in the corpus's constitution (`permanent-instructions.md`)
or the sergeant-rs codebase's own invariants — the remaining nineteen are
retained here as provenance (what the mechanism was, and why its job no
longer exists) rather than as a second, independently-drifting statement of a
rule the constitution already carries. This mirrors Article VII's own
single-owner discipline: a rule is stated once, and this document points to
that one place rather than restating it.

## What did *not* survive (mechanism only, no durable policy)

Named explicitly per `synthesis.md` §1's W8 note, since the `dispatch` skill
is this milestone's designated stress test: the pane as the worker's
identity, the pane as the notification channel, the pane as the liveness
signal, the brief as a loose file, and the nudge loop as delivery. None of
these five has a line anywhere in `permanent-instructions.md`,
`helper-map.md`, or `shared-context-map.md` — correctly, because each is pure
Bash/tmux implementation mechanism with no durable behavior riding inside it
that the structural replacement does not already supply for free.
