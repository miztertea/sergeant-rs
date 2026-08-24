# ADR 0022: The Agy (Antigravity) adapter — the print/loop transport
pair, the soft-deny discrepancy inverted by transport, and a second
honest no-sandbox posture

**Status:** Accepted, 2026-08-23. Implemented across four waves (W1
`agy -p --output-format stream-json`, W2 registration + the `sgt agy`
passthrough, W3 `--input-format stream-json` input loop, W4 this
record), merged to `integration/agy` (#251, #252, #253), on
`docs/proposals/agy-adapter-2026-08-23.md`.

## Context

Owner commission, same day (2026-08-23): build the Antigravity
(`agy`) adapter, Claude/Codex/OpenCode feature parity at minimum,
exceed it where agy measurably does better. The plan is explicit that
it follows ADR 0020's own invitation — "the pattern any future
adapter should copy rather than re-derive" — and ADR 0021's worked
second application of it; this ADR takes that literally, the same way
ADR 0021 did for ADR 0020.

Four owner rulings frame this sprint and are cited by name rather
than restated:

- **K1** — dev-test pin is `gemini-3.7-flash-low` (free tier,
  owner-authorized); the adapter takes model as a parameter and
  hardcodes nothing.
- **K2** — scope is the adapter, no core changes, with one
  pre-ratified named exception: the `sgt agy` passthrough
  (`cli.rs`/`harness.rs`), the exact mechanical mirror of the `goose`
  block (ADR 0006 D2). See the exception ledger below for every other
  touch outside `src/backend/` this sprint actually made.
- **K3** — version target `0.2.3`.
- **K4** — the name is `agy` everywhere: registry name, `--backend
  agy`, `DaemonConfig.agy`, `sgt agy`, `src/backend/agy.rs` — the
  binary's own name, the verb-execs-binary precedent (`goose`).

Carried without re-litigation from ADR 0020/0021: **R1** (a measured
version floor is provenance, not a gate), **R2** (harness and backend
are separate, user-composable axes), **R3** (core stays; adapter-local
evidence types, not a contract-v2 seam), **R4** (parity is the floor,
not the ceiling — a measured better-than-sibling capability is used
and the record amended).

Every behavioral claim below is measured against the installed agy —
**1.1.17** at the probe packet's own measurement time, auto-updated to
**1.1.19** mid-sprint (see "Version churn" below) — in the probe
evidence packet (`sergeant-rs-workspace`:`knowledge/evidence/
agy-adapter-probes-2026-08-23.md` plus its W1 probe amendment) or the
fixtures/live tests committed alongside `src/backend/agy.rs`, and is
tagged exactly as that module's own doc comments and 38-row
`ADMISSION_ROWS` ledger tag it: **[packet N]**, **[W1/W2/W3 PN]**,
**[changelog]**, or **[doc-claimed]**.

## Decision

### Harness and backend axes: nothing new decided here (R2)

`sgt agy` sets `SGT_ORIGIN_CLIENT=agy` the same way `sgt opencode` has
since ADR 0006, and `router.rs`'s four-tier ladder
(`Explicit > OriginAffinity > WorkspaceDefault > GlobalDefault`)
already resolves any backend name the registry holds. W2 registered
`"agy"` in `BackendRegistry` (`daemon.rs`) the same mechanical way
W2 of the codex and opencode sprints did — the block is a direct,
named mirror of the opencode registration block, including that
`AgyBackend::capabilities().ask` is `false` unconditionally, so
(unlike opencode's own `approval_flow`/`ask` withdrawal seeding) there
is nothing to seed `seed_capability_provenance_from` with.
`tests/agy_routing.rs` mirrors `tests/codex_routing.rs`/
`tests/opencode_routing.rs` line for line: explicit `--backend agy`,
origin affinity from `sgt agy`, and estate `default_backend = "agy"`
all reach the real registered adapter. No new precedence code, no new
env var, nothing decided here that ADR 0006/0020/0021 did not already
decide.

### Version provenance is R1 from birth, and the floor and the
installed build genuinely differ this time

`agy.rs` never had a refusal branch to strike — written against R1's
posture from its first commit, exactly as ADR 0020/0021 record for
codex and opencode. What is new here: `MEASURED_FLOOR = (1, 1, 17)`
is the version the probe packet was actually measured against, and
the binary **auto-updated to 1.1.19 between the packet and W1** — the
first time in this repo's adapter history that the floor and the
installed build have diverged mid-sprint rather than merely being
different numbers in the abstract. R1 makes this a non-event by
construction: 1.1.19 reads `Measured` (at-or-above the floor), a build
below 1.1.17 would still read `available: true` with an honest
unmeasured-provenance detail, and every fixture in this module is
named for the version it was actually captured at
(`agy-1.1.19-*.jsonl`) rather than silently backdated to the packet's
number. What *is* refused — the A2 split, carried verbatim — is a
version string that cannot be parsed, or a `--help` that does not
offer this adapter's own launch grammar; neither is a version-policy
question. One correction of the sprint plan's own attribution, stated
because build-version arguments are exactly the reasoning R1's own
fail-closed rules exist to resist: the empty-SUCCESS stream-drop fix
is agy's **1.1.18** changelog entry, not 1.1.16 as the plan and packet
both guessed — a fact that changes nothing about the classifier
(`classify_terminal`'s empty-SUCCESS rule is fail-closed by
construction and version-independent) but is recorded rather than
silently corrected without a trace.

### The transport story: `agy -p --output-format stream-json` (W1)
and an adapter-driven `--input-format stream-json` input loop (W3),
both per execution

**`agy -p <prompt> --output-format stream-json`** (W1): one OS
process per turn over a harness-minted, **server-side durable**
conversation — `init` (line 1, before any model output) →
`step_update`* → `result{status}`. `conversation_id` is
harness-minted, so `PreparedExecution::native_id` is honestly `None`
at PREPARE, and LAUNCH waits, bounded (`INIT_LINE_BUDGET`, 30s), for
that `init` line before returning a handle at all — the identical
fail-closed discipline ADR 0020/0021 apply to their own ambiguous
terminals, applied here to session *birth*. This is the fallback
under every capability this ADR names.

**The `--input-format stream-json` input loop** (W3): a persistent
child (`RuntimeScope::PerExecution` unchanged — `mod.rs`'s
ENSURE-RUNTIME seam stays untouched, K2) driven over **plain
stdio** — zero new crates, no ports, no auth posture to carry, the
cheapest second transport any adapter in this registry has had. One
child carries the whole execution's conversation, minted once on the
child's own first `init` line and never re-minted; the driver
serializes turns (wait for `result` before writing the next NDJSON
line) as its own rule, even though W3 P2 measured the harness already
serializing internally — nothing measured a bound on that internal
queue, and sergeant's SEND contract is per-turn regardless of what the
harness would tolerate. Closing stdin is the graceful shutdown; a
group SIGKILL is the ungraceful one, carried from opencode probe 11's
grandchild lesson without re-deriving it (R2) — `kill_process_group`
is reused verbatim.

Both transports share one decoder (`TurnAccumulator`/
`ingest_line`/`classify_terminal`) — the narration rule holds
structurally, not by two decoders agreeing today, the same posture
ADR 0020/0021 state for their own shared-decoder shapes.

### The soft-deny discrepancy — resolved, and it resolves two
different ways on the two transports

The probe packet measured a **hard** deny at 1.1.17: typed
`tool_info.error {type: "TOOL_ERROR", message: "…user denied
permission…"}`, terminal `status: "ERROR"`, exit 1. **W1's own live
reproduction against the installed 1.1.19 measured something else
entirely on print mode**, and inverted the packet's own changelog
hypothesis (soft-deny 1.1.3 later tightened into a hard deny) in the
process — soft-deny is the *current* print-mode behavior, not
superseded history:

- the tool step resolves `ACTIVE → DONE`, with **no** `tool_info.error`
  and **no** `output` — structurally indistinguishable from an
  ordinary clean tool completion;
- the terminal is `CANCELED`, `response: ""`, exit **0**;
- the *only* machine-readable evidence anywhere is a plain-text stderr
  notice (`denial_evidence_in_stderr`) — two independent live
  reproductions, byte-identical stderr captures.

`classify_terminal` took stderr as a fourth argument to see this
signal at all (a declared departure from the W1 spec's three-argument
signature, forced by measurement), and an unrequested `CANCELED`
classifies fail-closed ambiguous (`native: Unknown`, `signal:
Running`) rather than trusted as a clean completion — never `Failed`
(no explicit statement exists to fail on) and never `Completed` (the
work plausibly did not happen). `FakeStep::print_soft_denied_tool`
(W4) pins this exact honesty hazard deterministically: structured
evidence alone reads as clean, and only the terminal classification
tells the truth.

**W3 measured the loop transport doing the *opposite*** — the wave's
own most operationally important finding, because it means the two
transports cannot share one denied-tool story: the tool step carries
the packet's own **typed** `TOOL_ERROR` verbatim, the terminal is a
typed `ERROR` with that same message, stderr is **empty** (no
auto-denial notice at all), and the **child process itself exits 1**
— so a message queued behind the denied one never runs. Both
`tool_denial_evidence` (the packet's typed detector, print's fallback)
and `denial_evidence_in_stderr` (print's primary detector) stay live
in the same module because a build could emit either shape on either
transport; nothing here assumes one is dead. `FakeStep::
loop_denied_tool_kills_child` (W4) pins the loop shape, including that
a subsequent SEND against the same execution is refused, not queued —
there is no live child left to carry it.

### R4 deltas: identity-before-output and a registry-first native
subagent record

1. **Identity, the resolved model, and the effective permission mode
   all arrive on `init`, line 1, before any model output** [packet 1,
   W1 P2] — the strongest launch-time pin verification measured in
   this registry to date: claude verifies post-hoc from `modelUsage`,
   opencode post-hoc from `export`, codex records substitution as
   undetectable. `verify_pin_from_init`'s `Substituted` verdict
   refuses the LAUNCH itself, for zero turns spent on output the human
   did not ask for — a stronger claim than opencode's own
   `InitEchoVerifiedPin`-adjacent tier (ADR 0021), because agy's
   comparison needs no provider-prefix splitting at all: ids are flat
   (`gemini-3.7-flash-low`).
2. **`native_subagents: true` on the loop transport — the wave's
   headline, and the first `true` for this flag anywhere in the
   registry.** Admitted on all three pieces of evidence the spec
   demanded and nothing less [W3 A1, `live_agy_loop_invokes_a_
   subagent_and_records_its_typed_conversation_id`]: a step with
   `step_type "subagent"` / `tool_name invoke_subagent`; a **typed**
   `subagent_info` payload carrying a child `conversation_id` distinct
   from the parent's, plus a `log_uri` to the child's own
   `transcript.jsonl`; and that step reaching `DONE`. Explicitly *not*
   accepted as evidence: assistant text claiming a delegation
   happened, a tool step distinguished only by its name, or a
   `subagent_info` with no child conversation id. The child id is
   carried verbatim into `tool.completed` so a human can resume that
   trajectory by hand; sergeant does not adopt it as a second
   execution — nothing prepared it.
3. **Per-step usage** [packet 1]: every `step_update` may carry its
   own `{input, output, thinking, cache_read, total}`, known during
   the turn rather than only at its end.
4. **Native `--json-schema`** [packet 6, re-captured live at 1.1.19 on
   both transports]: a CLI flag, not a protocol negotiation. The
   terminal `result` carries a validated `structured_output` object
   beside the prose `response`, measured on **every** turn of a
   multi-turn loop child, not only the child's own final result — W3's
   open question on that point closed the good way. Adapter-local, no
   v1 boolean invented (R3), the posture codex's and opencode's own
   `structured_output` rows already take.
5. **Zero-quota introspection** [changelog 1.1.12, W1 P0]: print mode
   answers read-only slash commands with `usage.total_tokens: 0`,
   `num_turns: 0`, and an empty `conversation_id` — no turn, no quota,
   no conversation left behind. No sibling adapter can read the
   harness's own effective configuration or remaining quota without
   spending a turn; this module uses it once, at probe time, for the
   permission posture and the trusted-workspace check
   (`read_config_probe`).

### Two refutations, recorded as results, not gaps

- **`ask` and `approval_flow` are measured false, not merely
  unmeasured, on both transports.** The roster names `ask_question`/
  `ask_permission`, and `ask_question` is a real step type — but W3 P1
  wrote sixteen candidate reply-event names into a live loop child and
  every one but `control_request` was skipped with `warning: ignoring
  unsupported stream input message event`; `control_request` itself is
  refused as "not supported yet" (rc=2, upstream's own word). There is
  no message the driver may send to approve, deny, or answer anything.
  `Capabilities::ask` forbids guessing a question from prose, and this
  is the stronger statement of the two refutations: even a **typed**
  question would be unanswerable on this transport. **OpenCode keeps
  the registry's only `true` on either flag** — this ADR does not
  contest that record, it confirms agy does not join it.
- **A SIGINT-based interrupt upgrade was tried and refuted.** W3 P4
  measured a SIGINT to a loop child producing `status: "ERROR"`,
  `error: "timeout waiting for response"` — **byte-identical** to a
  plain `--print-timeout` expiry (W1 P5) — with no
  cancel-the-turn-keep-the-session gesture and the child dead within
  ~100ms regardless. A SIGINT-first ladder would trade a measured
  process-group-kill guarantee for a terminal that cannot tell a
  deadline from an interrupt. `interrupt` stays
  `ProcessTreeTermination` on both transports (the downgrade is
  journaled, not silent — codex §7.3's own precedent), and
  `classify_terminal` gained arm 1a: that ambiguous `ERROR`/timeout
  terminal now reads `InterruptedRunning` when this adapter's own
  `interrupt_requested` bit is set and `Failed` when it is not,
  carrying `terminal_ambiguity: "timeout_or_interrupt"` in both
  readings so a reader can see the classifier leaned on the bit, not
  on the wire.

### No native OS sandbox to claim — NORTH-STAR amendment 4 has
nothing to bind, the identical posture ADR 0021 already recorded

ADR 0020's NORTH-STAR amendment 4 (2026-08-21) says an adapter *may*
use its harness's native enforcement where one exists — permissive,
not mandatory. Free reconnaissance first: `nsjail` and
`sandbox-exec` both appear **nowhere** in the installed binary's
strings, so the packet's OS-native-mechanism claim is website
documentation with no corroboration in the shipped artifact.
`--sandbox`/`--add-dir` are accepted on the loop grammar and change
**nothing observable** on the `init` line (`permission_mode` still
reads `request-review`, no sandbox field at all) — sandbox state is
not launch-observable, so this adapter must not pretend to report it,
and composes neither flag by default on either transport. One paid
probe (W3 S1) went further and found a genuine, if broken, second
fact: `toolPermission: proceed-in-sandbox` with **no**
`permissions.allow` rule at all, launched `--sandbox`, lifted the
permission gate cleanly for `run_command` — no auto-deny, no "user
denied permission" anywhere — and then failed at the *mechanism*
itself: `tool_info.error {TOOL_ERROR, "connecting to sandbox server:
read unix @->@: recvmsg: connection reset by peer"}`. So
`proceed-in-sandbox` is evidenced as a real second permission channel
on a host where the sandbox server does not actually run — which is
exactly why nothing is claimed, and why S2/S3 (a write-escape probe)
were cut deliberately rather than for budget: with no working sandbox
server on this host, a write-escape probe would only measure the same
connection failure again. **Sergeant's observation layer stays the
sole source of truth for this adapter, exactly as it already is for
core and for opencode** — no dated amendment is appended here,
identically to ADR 0021's own reasoning for opencode: amendment 4's
allowance is permissive, and this adapter has nothing that qualifies
as native enforcement to reach for.

### The measured permission-injection channel, and the launch-time
honesty check that ships regardless

The panel amendment's rung (a) asked for a measured clean injection
channel before shipping on agy's defaults was even considered — probe
2 had already measured the default `request-review` posture
auto-denying every tool call and erroring the whole print-mode turn,
so an adapter on defaults cannot execute a tool-bearing Work at all.
[W1 P2] found the channel: workspace-scope settings
(`<cwd>/.agents/`, `.gemini/`, `.antigravity/`, `.antigravitycli/`,
and the cwd root itself) changed `/config`'s answer in **none** of
five measured cases, and no config-home environment variable exists
in the binary's strings — but the CLI does read
`$HOME/.gemini/antigravity-cli/settings.json`, a Go
`os.UserHomeDir()` path, and `$HOME` is per-process. A per-run `HOME`
override (`AgyConfig::settings_home`) is therefore the lever: with
`permissions.allow: ["command(echo)", "command(echo *)"]` as the
*only* delta, the identically-shaped `run_command` that was
auto-denied in the control ran, output recovered, terminal `SUCCESS`,
and nothing landed in the Work's own diff surface. **W1/W3 wire the
mechanism and synthesize no policy**: mapping a Work's declared
mutation surface onto agy's `command(...)`/`read_file(...)`/
`write_file(...)`/`read_url(...)`/`mcp(...)` namespaces (W3 additionally
read the *authoritative* namespace regex out of the binary — two more
namespaces, `execute_url`/`unsandboxed`, than the docs list) remains
unbuilt, because a policy invented here would be a security decision
with no measurement behind it. The blanket
`--dangerously-skip-permissions` is never a default (claude #47) and
this module composes it nowhere.

Rung (b) ships **regardless of rung (a)'s outcome, as the panel
amendment required**: the effective `permission_mode` is read off the
`init` line and, on the loop transport, at **child start** — before
any message is written at all, for zero quota — so a tool-bearing
intent launching under a denying posture is reported honestly at
launch, never discovered mid-run as a turn cancellation. Deliberately
over-warns: W1 P2 measured `request-review` echoed identically on both
the denied and the permitted control turns, so the mode string alone
predicts nothing, which is why the notice names "any tool call not
covered by an allow-rule" rather than claiming every call is at risk.

### The two crash-window and hang fixes folded in along the way

Two fixes, neither a capability claim, both worth recording because
they shape how this adapter behaves under conditions the spec did not
originally anticipate:

- **The zero-quota `/config` probe is not always zero-interaction.**
  `run_probe` runs synchronously inside `daemon::start_with`, before
  the daemon's own descriptor is published — the identical blocking-
  registration-call class this project already tracks for a blocking
  HTTP client built during registration (the 0.2.2 opencode-registration
  panic, c46152a2). An **unauthenticated** `agy` (no cached credentials
  under the effective settings home) answers `-p "/config"` by printing
  an OAuth URL and blocking on an interactive login for up to 60s
  before giving up on its own — measured during the W2 registration
  wave, not anticipated by W1. `CONFIG_PROBE_BUDGET` (5s, generous
  headroom over the sub-second reply a real authenticated `agy` gave)
  now bounds the call; a probe that cannot answer inside it is killed
  and treated as any other probe failure, best-effort either way.
- **The turn-end posture race.** `PermissionPosture` was originally
  computed only from LAUNCH's own round trip through
  `FirstTurnSignal`; the loop's own reader thread is *also* the thread
  that composes `conversation.turn.ended`, so a sufficiently fast
  child could reach turn-end before LAUNCH had finished storing the
  posture it needed to report. The reader now computes the posture
  itself at init-parse time, making the turn-end read race-free by
  construction rather than by timing luck.

### A recorded operational lesson from spec-authoring, not a code
defect (X0)

One live probe during W3's spec-authoring pass was spent by accident,
not by design, and is recorded here because a live-turn budget that
only counts deliberate turns is not a budget: composing
`--disable-slash-commands` **together with** a slash-command prompt
(`agy -p "/usage" --output-format json --disable-slash-commands`)
turns what looks like the zero-quota introspection path into an
ordinary, paid model turn — and with no `--model` given, it resolved
to the account's *default* model, not the pinned `gemini-3.7-flash-low`
(a K1 violation as well as a budget one). It paid for one genuine
finding — a hardcoded permission boundary
(`Matches hardcoded system protection boundary rule`) that no
`permissions.allow` rule reaches, delivered in the packet's *original*
1.1.17 typed-error shape, which is why `tool_denial_evidence` was
right to be kept as a detector even though it does not fire on the
1.1.19 print-mode soft-deny. `read_config_probe`'s own composition
(`-p "/config" --output-format json`, no `--disable-slash-commands`)
does not repeat this combination.

### The admission-rows / L8-structural pattern — reused verbatim

Contract v1 (`src/backend/mod.rs`) is untouched (R3): thirteen
booleans, no typed capability enum. This adapter's `AdmissionRow`
ledger (`agy.rs`, 38 rows) and its own structural agreement test are
ADR 0020's own pattern, copied rather than re-derived, exactly as
ADR 0021 copied it before. `agy.rs` declares **no new `KIND_*`
constant** — it reuses `KIND_TURN_HARNESS_ERROR` (from `codex.rs`,
already in `api::SSE_EVENT_KINDS`'s vocabulary) rather than forcing a
core edit to add one, distinguished by a `phase` field in the payload.

**The divergence between the two transport columns is deliberately
small**, the identical honest result ADR 0021 reports for opencode's
own two columns: the loop's wins are almost entirely in *cost and
timing* (zero-quota identity, a pre-turn pin refusal, a zero-quota
resume-fork check, no argv cap) and live in tiers and adapter-local
rows, not in flipped v1 booleans. Exactly one boolean moves between
the two `Capabilities` values — `native_subagents`, on the typed
record W3 A1 demanded of it — and `ask`/`approval_flow` do **not**
move; they are measured false on both transports, a refutation stated
identically twice rather than assumed to carry over.

## K2 exception ledger — every touch outside `src/backend/agy.rs`
this sprint made, complete, for owner ratification at the head PR

| Item | Wave | File(s) | Reason |
|---|---|---|---|
| The `sgt agy` passthrough | W2 | `src/cli.rs`, `src/harness.rs` | K2's own pre-ratified exception, named by the ruling itself: the exact mechanical mirror of the `goose` block (ADR 0006 D2) — origin-affinity routing has no origin to affine from without it. A new `Command::Agy` variant, its `dispatch` arm, and doc-comment counts (`four` → `five` harnesses) updated alongside. |
| `"agy"` registered in `BackendRegistry` | W2 | `src/daemon.rs` | `DaemonConfig` gained an `agy: Option<AgyConfig>` field plus the construction/registration and event-sink-wiring block, mirroring the opencode block directly above it line for line. A real edit to a core file outside `src/backend/`, even though ("Harness and backend axes," above) it decides nothing new: the routing ladder and `sgt agy`'s origin affinity already existed: registration only makes the name they already resolve toward real. |
| Required PUT-site + recovery-arm row | W1 | `tests/a4_blob_ref_pinning.rs` | Gate-forced, not discretionary — A4's own blob-ref-pinning suite requires every new raw-stream blob-capture site to carry a recoverability row, or the suite fails closed on its own. The adapter cannot exist without this row. |
| Registered-backend count/list widened in three pre-existing fixtures | W2 | `tests/m2_daemon_api.rs`, `tests/m3_execution.rs`, `tests/m4_backends.rs` | Mechanical, not a design choice: registering a sixth backend moves every fixture that hardcoded how many backends exist. A direct, forced consequence of the `daemon.rs` registration item, in the same commit, never a separate decision. |
| `agy_backend`/`agy_routing` wired into `c2-suites.sh` | W1/W2 | `scripts/coverage/c2-suites.sh` | Gate-forced by `tests/coverage_stage_membership.rs` (built during the opencode sprint, ADR 0021): every new suite is wired into a coverage stage or named in its `ALLOWLIST` in the same commit that creates it, or the guard fails closed. Both new suites were wired in their own creating commits — neither was ever an orphan. |
| `tests/agy_backend.rs`, `tests/agy_routing.rs` | W1–W3 | (new files) | The adapter's own contract and routing suites — outside `src/backend/` only because integration tests live at the crate root by this repo's existing layout, not because they test anything but this adapter. Named here for completeness of the ledger, not because either file is a scope question. |

None of the six items touch `src/backend/mod.rs`'s contract, the
router, or the engine — K2's actual substance (R3) is intact. Named
individually, per ADR 0021's own precedent, so a reader auditing "no
core changes" can check each item against the reason it was actually
necessary rather than take "the adapter" on faith.

## Consequences

Agy is a fully registered, routed, capability-honest fourth native
backend alongside Claude, Codex, and OpenCode, on two transports
neither of which shares the other's denied-tool shape — both handled,
neither assumed. Every capability it claims carries a named admission
test or a named, specific negative; `native_subagents` on the loop
transport is a registry first, earned on typed evidence and nothing
weaker. Two refutations (`ask`/`approval_flow` measured false, the
SIGINT interrupt upgrade refuted) are recorded as results rather than
silently absorbed as gaps — opencode keeps the registry's only `true`
on either capability flag. The version-provenance posture ADR 0020
made repo-wide holds here through its first genuinely mid-sprint
version bump, without exception. No native sandbox exists for this
adapter to claim, stated as a fact about agy — corroborated by
`strings`, not merely by silent documentation — not a gap in this
adapter's own coverage. The K2 exception ledger above is this
sprint's complete, honest account of every place work outside
`src/backend/agy.rs` was actually required.

## Open questions / hand-offs

- **`profiles`** is a DECLARED DIVERGENCE, not a full claim: generic
  sergeant axes (executable + env) reach every turn on both
  transports, `config_home` is refused rather than ignored, but agy's
  own `--agent`/custom-agent mechanism is unwired — `agy agents`
  printed an empty list on this host, and the documented workspace
  mechanism for defining one did not work here either [W1 P6]. A
  future wave with a host where `--agent` actually resolves is the
  hand-off.
- **`proceed-in-sandbox`'s own permission-channel property** (W3 S1,
  "No native OS sandbox to claim," above) is evidenced but unusable on
  this host — a future wave on a host with a working sandbox server
  could re-measure it as a genuine second permission channel needing
  no per-Work allow-rule synthesis, distinct from the `HOME`-relocated
  settings channel W1 shipped.
- **Whether agy runs tool commands in their own process group** is
  unmeasured [W1 P4]: `ps -g <pgid>` listed only the `agy` leader
  while a tool-spawned `sleep 120` was in flight. A group kill is
  correct either way, which is why `kill_process_group` is reused
  without re-arguing the question, but a future probe with a
  longer-lived tool child could settle it.
- **Whether agy publishes any CLI stability policy at all** remains
  unmeasured beyond "none found" — the same open question ADR 0021
  records for opencode, carried here for the same reason: R1's
  provenance posture does not depend on the answer.
- **Mapping a Work's declared mutation surface onto agy's
  `permissions` namespaces** (the config-injection channel's own
  policy layer) is explicitly unbuilt across W1 and W3 both — a
  security decision this sprint declined to invent without measurement
  behind it, named as a hand-off rather than deferred silently.

## Alternatives considered

**Shipping on agy's defaults for tool-bearing Work stages** — refused
directly by the panel amendment this ADR's own permission-channel
section describes: probe 2 measured the default posture auto-denying
every tool call and erroring the whole print-mode turn, so an adapter
on defaults cannot execute a tool-bearing stage at all. The two-rung
ladder (a measured injection channel, plus the launch-time honesty
check regardless of whether one exists) is what was actually built.

**A SIGINT-based interrupt upgrade** — tried, measured, refuted (see
"Two refutations," above): the resulting terminal is byte-identical
to a plain deadline expiry, which would trade a measured guarantee for
a mislabelled one. `ProcessTreeTermination` stands, carried rather
than upgraded, unlike opencode's own `NativeSessionAbort` upgrade
(ADR 0021) — a genuinely different measured outcome for a
structurally similar upgrade attempt, recorded as such rather than
forced to match.

**Guessing an actor question from `ask_question`'s presence in the
tool roster, or from narration** — refused directly: `Capabilities::
ask`'s own contract forbids inferring authorship from prose, and W3's
own measurement (sixteen candidate reply events, all rejected or
ignored) shows the stronger fact that even a typed question on this
transport has no channel to answer it on regardless.

**A typed `Capabilities` v2 enum for this adapter alone** — refused by
R3, identically to ADR 0020/0021's own refusal of the same idea; the
adapter-local `AdmissionRow` ledger is what was actually built,
designed to lift whole into a real v2 when one lands.
