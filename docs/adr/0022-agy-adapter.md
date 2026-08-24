# ADR 0022: The Agy (Antigravity) adapter — the print/loop transport
pair, the soft-deny discrepancy inverted by transport, the registry's
first native subagents, and a second honest no-sandbox posture

**Status:** Accepted, 2026-08-23. Implemented across four waves (W1
`agy -p --output-format stream-json`, W2 registration + the `sgt agy`
passthrough, W3 `--input-format stream-json` input loop, W4 fake
fidelity + this record + the 0.2.3 finalize), merged to
`integration/agy` (#251, #252, #253) under head PR #250, on
`docs/proposals/agy-adapter-2026-08-23.md`.

## Context

Owner commission, same day (2026-08-23): build the Antigravity
(`agy`) adapter, Claude/Codex/OpenCode feature parity at minimum,
exceed it where agy measurably does better. The plan is explicit that
it follows ADR 0020's own invitation — "the pattern any future
adapter should copy rather than re-derive" — and ADR 0021's worked
second application of it; this ADR takes that literally, the same way
ADR 0021 did for ADR 0020. Where a section here matches those two,
that is the point, not an oversight; where agy's measurement diverges
from theirs, the divergence is named rather than smoothed over.

Four owner rulings frame this sprint and are cited by name rather
than restated:

- **K1** — dev-test pin is `gemini-3.7-flash-low` (free tier,
  owner-authorized); the adapter takes model as a parameter and
  hardcodes nothing. Every live test and every paid probe pins it —
  with exactly one recorded violation, X0 below, which is why X0 is
  in this record at all.
- **K2** — scope is the adapter, no core changes, with one
  pre-ratified named exception: the `sgt agy` passthrough
  (`cli.rs`/`harness.rs`), the exact mechanical mirror of the `goose`
  block (ADR 0006 D2). Every other touch outside `src/backend/agy.rs`
  is on the exception ledger below, complete, for ratification at the
  head PR.
- **K3** — version target `0.2.3` (`Cargo.toml`, the 0.2.1/0.2.2
  patch-bump precedent).
- **K4** — the name is `agy` everywhere: registry name, `--backend
  agy`, `DaemonConfig.agy`, `sgt agy`, `src/backend/agy.rs` — the
  binary's own name, the verb-execs-binary precedent (`goose`).

Carried without re-litigation from ADR 0020/0021: **R1** (a measured
version floor is provenance, not a gate), **R2** (harness and backend
are separate, user-composable axes; and, in this sprint's own rung
log, reuse a shipped shape rather than re-derive it), **R3** (core
stays; adapter-local evidence types, not a contract-v2 seam), **R4**
(parity is the floor, not the ceiling — a measured
better-than-sibling capability is used and the record amended,
`[[repo-is-a-snapshot]]`).

Every behavioral claim below is measured against the installed agy —
**1.1.17** at the probe packet's measurement time, auto-updated to
**1.1.19** mid-sprint (below) — in the probe evidence packet
(`sergeant-rs-workspace`:`knowledge/evidence/
agy-adapter-probes-2026-08-23.md` plus its W1 probe amendment), in
W3's own probe record and live transcripts, or in the
fixtures/live tests committed alongside `src/backend/agy.rs`. Each is
tagged exactly as that module's own doc comments and its 38-row
`ADMISSION_ROWS` ledger tag it: **[packet N]**, **[W1/W2/W3 PN]**,
**[changelog]**, or **[doc-claimed]**. A claim with no tag is a
defect, in that module and in this record alike.

## Decision

### Harness and backend axes: nothing new decided here (R2)

`sgt agy` sets `SGT_ORIGIN_CLIENT=agy` the way `sgt opencode` has
since ADR 0006, and `router.rs`'s four-tier ladder
(`Explicit > OriginAffinity > WorkspaceDefault > GlobalDefault`)
already resolves any backend name the registry holds. W2 registered
`"agy"` in `BackendRegistry` (`daemon.rs`) the same mechanical way W2
of the codex and opencode sprints did — a direct, named mirror of the
opencode block immediately above it, including one deliberate
omission: `AgyBackend::capabilities().ask` is `false` on both
transports unconditionally, so unlike opencode's registration there is
nothing to seed `seed_capability_provenance_from` with.
`tests/agy_routing.rs` mirrors `tests/codex_routing.rs`/
`tests/opencode_routing.rs`: explicit `--backend agy`, origin affinity
from `sgt agy`, and estate `default_backend = "agy"` each reach the
real registered adapter, host-independently. `goose` stays the
canonical *unregistered* fixture name, untouched. No new precedence
code, no new env var, nothing decided here that ADR 0006/0020/0021 did
not already decide — registration is verification of shipped
machinery, exactly as ADR 0020's A1 framed it.

### Version provenance is R1 from birth — and this is the first
sprint where the floor and the installed build actually diverged

`agy.rs` never had a refusal branch to strike: it was written against
R1's posture from its first commit, as ADR 0020/0021 record for codex
and opencode. What is new here is that the two numbers came apart
under the sprint's own feet. `MEASURED_FLOOR = (1, 1, 17)` is the
version the probe packet was measured at; the binary **auto-updated to
1.1.19 between the packet and W1**. R1 makes that a non-event by
construction — 1.1.19 reads `Measured`, a build below 1.1.17 still
reads `available: true` with an honest unmeasured-provenance detail
and is never blocked — and the discipline that keeps it honest is
naming: every fixture in this module is named for the version it was
actually captured at (`tests/fixtures/agy-1.1.19-*`), never backdated
to the packet's number, and the module doc states both numbers and why
they differ. What *is* refused — the A2 split, carried verbatim — is a
version string that cannot be parsed at all, or a `--help` that does
not offer this adapter's own launch grammar
(`required_flags_are_exactly_what_the_launch_grammar_composes` checks
both flag lists against both argv builders, in both directions);
neither is a version-policy question.

One attribution correction, made in the open because a build-version
argument is precisely the reasoning R1's fail-closed rules exist to
resist: this sprint's plan and the probe packet both attribute the
empty-SUCCESS stream-drop fix to agy **1.1.16**. The CLI's own bundled
changelog names it **1.1.18**. This changes nothing about the
classifier — `classify_terminal` arm 3's empty-SUCCESS rule is
fail-closed by construction and version-independent, pinned by
`agy-1.1.19-dropped-stream-empty-success.jsonl` — and it is recorded
rather than silently corrected, so that no later reader can reach for
"the installed build is ≥ 1.1.18" as grounds to weaken a fail-closed
rule.

### The transport story: `agy -p --output-format stream-json` (W1) and
an adapter-driven `--input-format stream-json` input loop (W3), both
per execution, one decoder

**`agy -p <prompt> --output-format stream-json`** (W1): one OS process
per turn over a **harness-minted, server-side durable** conversation —
`init` (line 1, before any model output) → `step_update`* →
`result{status}`. Because `conversation_id` is harness-minted,
`PreparedExecution::native_id` is honestly `None` at PREPARE, and
LAUNCH waits, bounded (`INIT_LINE_BUDGET`, 30s), for that `init` line
before returning a handle at all — the identical fail-closed
discipline ADR 0020/0021 apply to their own ambiguous terminals,
applied here to session *birth*. This transport is the fallback under
every capability this ADR names.

**The `--input-format stream-json` input loop** (W3): a persistent
child driven over **plain stdio** — zero new crates, no ports, no auth
posture to carry; the cheapest second transport any adapter in this
registry has had, and the reason the owner's opencode-era dependency
carve-out has no analog to spend here. `runtime_scope()` stays
`RuntimeScope::PerExecution`, so `mod.rs`'s ENSURE-RUNTIME seam stays
untouched (K2). One child carries the whole execution's conversation,
minted once on that child's own `init` line and never re-minted
(`a_loop_turn_boundary_resets_the_accumulator_but_not_the_
conversation`). The driver serializes turns — wait for `result` before
writing the next NDJSON line — as its own rule, even though W3 P2
measured the harness already serializing internally (two messages
written back-to-back at t=0.001s ran strictly sequentially, turn 2's
first step landing 205ms after turn 1's result). Two reasons, said out
loud in the `turn_serialization` row rather than assumed: nothing
measured a *bound* on that internal queue, and sergeant's SEND
contract is per-turn regardless of what the harness would tolerate.
Closing stdin is the graceful shutdown; a group SIGKILL is the
ungraceful one, carrying opencode probe 11's grandchild lesson without
re-deriving it (R2 — `kill_process_group` is reused verbatim,
`loop_interrupt_group_kills_the_child_and_its_grandchild`).

**Both transports decode through one path** — `TurnAccumulator` /
`ingest_line` / `classify_terminal` — so the narration rule holds
structurally, not by two decoders happening to agree today: ADR
0020/0021's own posture for their shared-decoder shapes.

**One declared spec deviation, recorded rather than resolved
silently.** W3's spec said `--input-format` joins `REQUIRED_FLAGS`
(§2.1) *and* that an `Auto` transport resolution whose gate fails
falls back to print with a probe detail (§2.8). Those cannot both
hold: a flag in `REQUIRED_FLAGS` makes the whole probe
`available: false`, so an older `agy` with no input loop would be
refused outright instead of being served on the print transport it
fully supports. §2.8 is the load-bearing sentence, so
`LOOP_GATE_FLAGS` is a separate list gating only the loop, and the
membership rule is still mechanically enforced across both lists and
both argv builders.

### The soft-deny discrepancy — resolved, and it resolves two
different ways on the two transports

The probe packet measured a **hard** deny at 1.1.17: typed
`tool_info.error {type: "TOOL_ERROR", message: "permission check failed
… user denied permission to run command"}`, terminal `status: "ERROR"`,
exit 1 [packet 2]. **W1's live reproduction against the installed
1.1.19 measured something else entirely on print mode**, inverting the
packet's own changelog hypothesis (that soft-deny was 1.1.3 behavior
later *tightened* into a hard deny) in the process — soft-deny is the
*current* print-mode behavior, not superseded history [W1 P2's control
turn and W1 P3's turn 1; two independent live reproductions, and only
two, with byte-identical stderr captures]:

- the tool step resolves `ACTIVE → DONE`, with **no** `tool_info.error`
  and **no** `output` — structurally indistinguishable from an
  ordinary clean tool completion;
- the terminal is `CANCELED`, `response: ""`, and the process exits
  **0**;
- the *only* machine-readable evidence anywhere is a plain-text stderr
  notice, committed as one `include_str!` fixture of the captured bytes
  (`agy-1.1.19-denial-notice.txt`) and detected by
  `denial_evidence_in_stderr`.

Three structural consequences, none of them prose: `classify_terminal`
takes the drained **stderr** as a fourth argument — a declared
departure from the W1 spec's three-argument signature, forced by
measurement, because the spec's signature could not see the only live
signal at all; an unrequested `CANCELED` classifies **fail-closed
ambiguous** (`native: Unknown`, `signal: Running`, arms 5/6) rather
than as a clean completion, never `Failed` (no explicit statement
exists to fail on) and never `Completed` (the work plausibly did not
happen); and exit 0 is not treated as a completion.
`a_denied_tool_call_is_a_cancelled_turn_not_a_hang` is the print
transport's admission test (`non_blocking_run`, tier
`DeniedToolCancelsTheTurn`).

**W3 measured the loop transport doing the opposite** — the wave's own
most operationally important measurement, because it means the two
transports cannot share one denied-tool story [W3 A2, one live turn,
no allow-rule]: the tool step resolves `ACTIVE → ERROR` carrying the
packet's own **typed** `TOOL_ERROR` verbatim; the terminal is a typed
`ERROR` with that same message; stderr is **empty** — no auto-denial
notice at all; and the **child process itself exits 1**, so a message
queued behind the denied one never runs.
`a_denied_tool_on_the_loop_kills_the_child_and_the_next_send_is_
refused` is that transport's admission test (tier
`DeniedToolKillsTheChild`), and it pins the refusal as well as the
death: a subsequent SEND is refused, not queued behind a child that no
longer exists. Both detectors therefore stay live in the same module —
`tool_denial_evidence` (the packet's typed shape, print's fallback,
kept "in case a build emits it" and vindicated here) and
`denial_evidence_in_stderr` (print's primary) — with a committed
fixture each, because a build could emit either shape on either
transport and nothing here assumes one is dead. The conversation
survives the child's death: a fresh child re-adopted it at zero quota
immediately afterwards. The honest operational consequence is recorded
rather than acted on: the loop's dead-transport path is *routine*, not
exceptional, which is a genuine argument for print as an operator's
default until a per-Work allow-rule policy exists.

### R4 deltas: five, each with its own named admission test

ADR 0020 names R4 as "parity is the floor, not the ceiling." This
sprint cashes it in five times. Each is a row in `ADMISSION_ROWS` with
a named test, not a paragraph:

1. **Identity, the resolved model, and the effective permission mode
   all arrive on `init`, line 1, before any model output** [packet 1,
   W1 P2] — the strongest launch-time pin verification measured in this
   registry. The corrected comparison, stated exactly as the plan's own
   panel amendment corrected it: **claude** verifies its pin *post-hoc*
   from `modelUsage`; **opencode** verifies it *post-hoc* from
   `export`/the response (substitution detectable, but only after the
   turn — its own rows' words); **codex** records substitution as
   genuinely *undetectable*. Agy verifies it **at launch, before
   output**, which is why `verify_pin_from_init`'s `Substituted`
   verdict *refuses the LAUNCH* rather than reporting after the fact —
   and the comparison needs no provider-prefix splitting, because agy's
   ids are flat (`gemini-3.7-flash-low`) where opencode's are
   `provider/model`. Admission tests:
   `live_agy_init_line_echoes_the_pinned_model_and_mints_the_
   conversation` (print, LiveMeasured) and, on the loop,
   `live_agy_loop_resume_echoes_the_conversation_before_any_turn`
   (LiveMeasured), which is also the admission test for the
   adapter-local `identity_before_first_turn` row, tier
   `InitAtChildStart`. That loop row is the sharper form of the same
   fact and a registry first in its own right: `init` is emitted at
   **child start**, before any message is consumed — proven by an
   empty-stdin child that emitted `init` and exited 0 having consumed
   nothing [W3 P1 row I] — so pin refusal, the permission-posture
   notice, the trusted-workspace notice and the silent-resume-fork
   check all run for **zero quota**, where print mode must burn turn 1
   to learn the same things.
2. **`native_subagents: true` on the loop transport — the sprint's
   headline, and the first `true` for this flag anywhere in the
   registry.** Admitted on all three pieces of evidence the spec
   demanded and nothing less [W3 A1,
   `live_agy_loop_invokes_a_subagent_and_records_its_typed_
   conversation_id`, tier `TypedSubagentInfoRecord`, LiveMeasured]: a
   `step_update` with `step_type: "subagent"` and `tool_name:
   invoke_subagent`; a **typed** `subagent_info` payload on it carrying
   a child `conversation_id` **distinct from the parent's** plus a
   `log_uri` to the child's own `transcript.jsonl`; and that step
   reaching `DONE`. Two shape corrections the measurement forced on the
   changelog's prose, recorded in the row: the payload is a **list**
   (`subagent_info.subagents[{type_name, role, initial_prompt,
   conversation_id, log_uri}]`), not the flat object the prose implied,
   and the child's identity appears **only on the resolved step** — the
   `ACTIVE` one carries the first three fields and no id. Explicitly
   *not* accepted as evidence: assistant text claiming a delegation
   happened, a tool step distinguished only by its name, or a
   `subagent_info` with no child conversation id. The child id is
   carried verbatim into `tool.completed` so a human can resume that
   trajectory by hand; sergeant does **not** adopt it as a second
   execution, which would be an execution nothing prepared.
3. **Per-step usage** [packet 1, W1 P2]: every `step_update` may carry
   its own `{input, output, thinking, cache_read, total}`, so usage is
   known *during* a turn, not only at its end — one `usage.updated` per
   step plus one final `scope: "turn"` event from `result.usage`, both
   carried verbatim, never a synthetic sum
   (`per_step_and_terminal_usage_become_usage_events`). The loop
   transport's own row adds the hazard it owns rather than inheriting
   the print claim: `result.num_turns`, `step_index` and
   `duration_seconds` are **conversation-scoped**, not child-scoped
   (W3 P2 saw 0,1,2 then 3,4; W3 P3's resumed child opened at
   `step_index` 5 with `num_turns` 3 and `duration_seconds` 133.16), so
   nothing keys on any of them starting at zero and `duration_seconds`
   is never read as a turn duration
   (`a_conversation_scoped_counter_is_never_assumed_to_start_at_zero`).
4. **Native `--json-schema`** [packet 6, re-captured live at 1.1.19 on
   both transports]: a CLI flag, not a protocol negotiation. The
   terminal `result` carries a validated `structured_output` object
   **beside** the prose `response`, plus a `json_schema` echo
   (`the_json_schema_fixture_carries_structured_output_beside_the_
   response`, tier `NativeSchemaFlag`). W3 closed the open question the
   good way: `--help` says the flag is "only applicable to the final
   result", ambiguous on a multi-turn child between each *turn's* result
   and the *child's*; two live turns through one child with a two-field
   schema both carried a validated `structured_output`
   (`{word:alpha,n:1}` then `{word:bravo,n:2}`), so a `None` on an
   intermediate turn would be an anomaly, not an expectation
   (`the_loop_schema_fixture_carries_structured_output_on_every_turn`).
   Adapter-local, no v1 boolean invented (R3) — the posture codex's and
   opencode's own `structured_output` rows already take. The adapter
   wires the channel (`AgyConfig::json_schema`) and synthesizes no
   schema: sergeant has no per-stage output-schema surface and
   inventing one is a core change (K2).
5. **Zero-quota introspection** [changelog 1.1.12, W1 P0]: print mode
   answers read-only slash commands (`/config`, `/usage`,
   `/permissions`, `/agents`) with `usage.total_tokens: 0`,
   `num_turns: 0` and an **empty** `conversation_id` — no turn, no
   quota, no conversation left behind. No sibling adapter can read its
   harness's effective configuration or remaining quota without
   spending a turn. This module uses it once, at probe time, for the
   permission posture and the trusted-workspace check
   (`read_config_probe`; `decode_config_probe_reads_the_measured_
   answer`, `decode_config_probe_counts_allow_rules`). Its own honesty
   hazard is pinned too: that zero-quota answer is byte-identical in
   shape to the empty-SUCCESS terminal, so
   `a_slash_command_result_never_reads_as_a_completed_turn` guards the
   confusion, and `--disable-slash-commands` is composed on every real
   turn (`every_turn_composes_disable_slash_commands`) so a stage's own
   prose can never accidentally take this path.

### Two refutations, recorded as results, not gaps

- **`ask` and `approval_flow` are measured *false*, not merely
  unmeasured, on both transports.** The tool roster names
  `ask_question`, `ask_permission` and `ask_custom_permission`, and
  `CORTEX_STEP_TYPE_ASK_QUESTION` exists, so a question may well
  *surface* — the refutation is about the **reply channel**. W3 P1
  wrote sixteen candidate reply-event names into a live loop child;
  fifteen were skipped with `warning: ignoring unsupported stream input
  message event`, and the sixteenth, `control_request`, was refused
  *"not supported yet"* (rc=2, upstream's own words). Print mode's
  complete 123-name symbol table contains no answer, reply or
  permission handler at all. There is no message the driver may send to
  approve, deny, or answer anything, on either transport. This is the
  stronger of the two refutations: `Capabilities::ask` forbids guessing
  a question from prose, and these rows say that even a **typed**
  question would be unanswerable here — a stage sergeant would park
  forever. **OpenCode keeps the registry's only `true` on either
  flag** (ADR 0021); this ADR does not contest that record, it confirms
  agy does not join it. Both rows carry `Evidence::LocallyMeasured`
  with the probe named, not `Unmeasured` — a claimed negative, with
  transcripts.
- **A SIGINT-based interrupt upgrade was tried and refuted.** W3 P4
  measured a SIGINT to a loop child producing `status: "ERROR"` with
  `error: "timeout waiting for response"` — **byte-identical** to a
  plain `--print-timeout` expiry (W1 P5) — with no
  cancel-the-turn-keep-the-session gesture anywhere, and the child dead
  within ~100ms regardless. A SIGINT-first ladder would trade a
  measured process-group-kill guarantee for a terminal that cannot tell
  a deadline from an interrupt. `interrupt` therefore stays
  `ProcessTreeTermination` on both transports
  (`agy_interrupt_kills_the_process_group`,
  `loop_interrupt_group_kills_the_child_and_its_grandchild`), and the
  refutation is banked as a *classifier* amendment instead:
  `classify_terminal` gained arm 1a, so that ambiguous `ERROR`/timeout
  terminal reads `InterruptedRunning` when this adapter's own
  `interrupt_requested` bit is set and `Failed` when it is not,
  carrying `terminal_ambiguity: "timeout_or_interrupt"` in **both**
  readings so a reader can see the classifier leaned on the bit rather
  than on the wire (the fixture is read both ways). Where an interrupt
  downgrade happens it is journaled, never silent — codex §7.3's own
  precedent, carried.

### The permission posture: a measured injection channel, a launch-time
honesty report that ships regardless, and one silent-write hazard

The plan's panel amendment replaced "ship on agy's defaults" with a
two-rung ladder, because [packet 2] had already measured that the
default `request-review` posture auto-denies every tool call — so an
adapter on defaults cannot execute a tool-bearing Work stage at all.

**Rung (a), the injection channel — measured, and the negative space
measured first** [W1 P2]. Workspace-scope settings do **not** exist: a
`settings.json` under `<cwd>/.agents/`, `<cwd>/.gemini/`,
`<cwd>/.antigravity/`, `<cwd>/.antigravitycli/` or `<cwd>/` itself
changed `/config`'s answer in **none** of five measured cases, and a
`strings` scan of the binary names **no** config-home environment
variable. What the CLI does read is
`$HOME/.gemini/antigravity-cli/settings.json`, a Go
`os.UserHomeDir()` path — and `$HOME` is per-process. So a per-run
`HOME` is the lever: with `permissions.allow: ["command(echo)",
"command(echo *)"]` as the *only* delta, the identically-shaped
`run_command` that was auto-denied in the control **ran**, output
recovered, terminal `SUCCESS`, and nothing landed in the Work's own
diff surface. `AgyConfig::settings_home` carries that channel and the
launch composes `HOME=<dir>` (tier `SettingsHomeViaHome`;
`a_permission_config_reaches_every_turn_without_dirtying_the_work_
diff`, and on the loop `a_settings_home_reaches_a_loop_child` — the
channel was re-exercised under its own `HOME` for every paid W3 probe,
three of which got three different measured behaviors out of it, which
is the channel working). Two operator facts the channel owes its user,
stated rather than assumed: a `HOME` override also relocates the
credential store and the conversation store, so a settings home that
does not carry or symlink the CLI's own `antigravity-cli` state will
fail authentication; and `toolPermission` accepts exactly
`request-review`, `strict` and `proceed-in-sandbox` — any other value,
`accept-edits` included, **silently falls back to `request-review`**.

**W1 and W3 both wire the mechanism and synthesize no policy.**
Mapping a Work's declared mutation surface onto agy's permission
namespaces remains unbuilt. W3 did improve the *inputs* to that future
policy by reading the **authoritative** namespace regex out of the
binary — `^(command|read_file|write_file|read_url|mcp|execute_url|
unsandboxed)\s*\(.*\)$`, two namespaces more than the docs list — and
still synthesized nothing, because a policy invented here would be a
security decision with no measurement behind it. The blanket
`--dangerously-skip-permissions` is **never** a default (claude #47)
and this module composes it nowhere, on either transport.

**Rung (b) ships regardless of rung (a)'s outcome, as the panel
amendment required.** The effective `permission_mode` is read off the
`init` line and reported at launch, so a tool-bearing intent launching
under a denying posture is named honestly *at launch* rather than
discovered mid-run as a turn cancellation
(`a_denying_permission_mode_is_reported_at_launch_not_mid_turn`; on the
loop, `a_loop_launch_learns_identity_and_posture_before_any_message_is_
written`, tier `InitEchoPermissionModeBeforeAnyTurn` — read at child
start, before turn 1, for zero quota). It deliberately **over-warns**:
W1 P2 measured `request-review` echoed identically on the denied
control turn and the permitted one, so the mode string alone predicts
nothing, which is exactly why the notice names "any tool call not
covered by an allow-rule" rather than claiming every call is at risk.

**One hazard the W1 spec did not anticipate, declared rather than
absorbed** [W1 P3]: with a `cwd` outside `trustedWorkspaces` and
`allowNonWorkspaceAccess: false`, a `write_to_file` call **wrote to
the CLI's own scratch directory**, the Work's cwd stayed empty, the
turn terminated `SUCCESS`, and nothing on stderr or in the NDJSON said
so. LAUNCH emits `phase: "cwd_outside_trusted_workspaces"` for it,
read from the same zero-quota `/config` probe
(`a_cwd_outside_the_trusted_workspaces_is_reported_at_launch`,
`a_cwd_inside_a_trusted_workspace_raises_no_notice`). Also measured
and left unexplained rather than rationalized: the file-edit surface is
**not** gated the same way as `command` — `write_to_file` ran under
default `request-review` with no allow-rule and no `--mode`. Nothing
here claims to know why.

### No native OS sandbox to claim — NORTH-STAR amendment 4 has nothing
to bind, the identical posture ADR 0021 already recorded

ADR 0020's NORTH-STAR amendment 4 (2026-08-21) says an adapter **may**
use its harness's native enforcement where one exists — permissive,
not mandatory — and records codex's `workspace-write` sandbox as the
first adapter to use that allowance. Agy's own docs claim an OS-native
mechanism (nsjail on Linux), which would put this squarely in
amendment 4's territory. Three measurements, cheapest first, say
otherwise:

- **Free reconnaissance:** `nsjail` appears **nowhere** in the
  installed binary's strings, and neither does `sandbox-exec`. The
  packet's OS-native-mechanism claim is website documentation with no
  corroboration in the shipped artifact.
- **Grammar:** `--sandbox` and `--add-dir` are accepted on the loop
  grammar and change **nothing observable** on the `init` line —
  `permission_mode` still reads `request-review`, and there is no
  sandbox field at all. Sandbox state is **not launch-observable**, so
  this adapter must not pretend to report it, and composes neither flag
  by default on either transport.
- **One paid probe** [W3 S1], which found a genuine, if broken, second
  fact: `toolPermission: proceed-in-sandbox` with **no**
  `permissions.allow` rule at all, launched `--sandbox`, **lifted the
  permission gate cleanly** for `run_command` — no auto-deny, no "user
  denied permission" anywhere — and then failed at the *mechanism*:
  `tool_info.error {TOOL_ERROR, "connecting to sandbox server: read
  unix @->@: recvmsg: connection reset by peer"}`, a retry resolving
  `DONE` with no output, and a terminal `ERROR` carrying the same
  string.

So `proceed-in-sandbox` is evidenced as a real **second permission
channel** — one needing no per-Work allow-rule synthesis — on a host
where the sandbox server does not actually run. That is exactly why
nothing is claimed, and why an uninvited `--sandbox` here would be not
merely an invented launch decision but a broken one. S2 and S3 (a
write-escape probe) were **cut deliberately, not for budget**: with no
working sandbox server on this host, a write-escape probe would only
have measured the same connection failure again.

**Sergeant's observation layer therefore stays the sole source of truth
for this adapter, exactly as it already is for core and for opencode.**
No dated NORTH-STAR amendment is appended here, on ADR 0021's own
reasoning verbatim: amendment 4's allowance is permissive, this adapter
has nothing that qualifies as native enforcement to reach for, and
amendment 4's own third numbered consequence already says that an
adapter's enforcement, where none exists, changes nothing about where
sergeant charges dirty evidence from. The decision is an *argued*
silence with a corroborating negative, not an absence — which is what
the `sandbox` admission row's own note records, and what the plan's W4
section meant by "W4 owes ADR 0022 either way."

### Two fixes folded in along the way, neither a capability claim

- **The zero-quota `/config` probe is not always zero-*interaction*.**
  `run_probe` runs synchronously inside `daemon::start_with`, before
  the daemon's own descriptor is published — the same
  blocking-call-during-registration class this project already tracks
  from the 0.2.2 opencode-registration panic (`c46152a2`). An
  **unauthenticated** `agy` (no cached credentials under the effective
  settings home) answers `-p "/config"` by printing an OAuth URL and
  blocking on an interactive login for up to 60s before giving up on
  its own — measured during W2's registration wave, where it reproduced
  as a deterministic ~60s daemon-boot hang, not anticipated by W1.
  `CONFIG_PROBE_BUDGET` (5s, generous headroom over the sub-second
  reply a real authenticated `agy` gave) now bounds the call; a probe
  that cannot answer inside it is killed and treated as any other probe
  failure, `available` stays true, best-effort either way. The fix is
  revert-verified: unbounding the call makes
  `a_hung_config_probe_is_killed_within_budget_and_falls_back` fail at
  60.007s, naming the constant.
- **The turn-end posture race.** `PermissionPosture` was originally
  computed only from LAUNCH's round trip through `FirstTurnSignal`,
  while the loop's own reader thread is *also* the thread that composes
  `conversation.turn.ended` — so a sufficiently fast child could reach
  turn-end before LAUNCH had stored the posture it needed to report,
  surfacing as a ~1-in-3 deterministic-tier flake with
  `permission_posture: null`. The reader now computes the posture
  itself at init-parse time, on the same thread and lock, making the
  turn-end read race-free by construction rather than by timing luck
  (pre-fix tree reproduced twice; 6/6 consecutive suite runs green
  after).

### X0 — an operational lesson from spec-authoring, recorded because a
budget that counts only deliberate turns is not a budget

One live probe during W3's spec-authoring pass was spent **by
accident**: composing `--disable-slash-commands` **together with** a
slash-command prompt (`agy -p "/usage" --output-format json
--disable-slash-commands`) turns what looks like the zero-quota
introspection path into an ordinary, paid model turn — 79k tokens —
and, with no `--model` given, it resolved to the account's *default*
model rather than the pinned `gemini-3.7-flash-low`, making it a K1
violation as well as a budget one. It is recorded here, in the packet,
and in the W3 PR body rather than netted out of a total.

It paid for one genuine finding, which is why it is not merely an
embarrassment: a **hardcoded permission boundary** (`Matches hardcoded
system protection boundary rule`) that no `permissions.allow` rule
reaches — delivered in the packet's *original* 1.1.17 typed-error
shape, which is the evidence that `tool_denial_evidence` was right to
be kept as a detector even though it does not fire on 1.1.19's
print-mode soft-deny. `read_config_probe`'s own composition
(`-p "/config" --output-format json`, **no**
`--disable-slash-commands`) does not repeat the combination.

### Fake-backend fidelity: the deterministic backend learned three agy
shapes and reused two (W4)

`src/backend/fake.rs` gained three `FakeStep` constructors, mirroring
how it already learned codex's and opencode's shapes (R2), so that a
suite needing agy's measured hazards does not need agy installed:
`print_soft_denied_tool` (the clean-looking `tool.requested`/
`tool.completed` pair with no error field, paired with the
fail-closed-ambiguous terminal — the honesty hazard of the print
transport, deterministically);
`invalid_model_refusal` (mints no identity at all — uniquely,
`FakeBackend::launch` consumes it and returns an error directly,
leaving no execution behind, matching `FirstTurnSignal::
RefusedBeforeIdentity`); and `loop_denied_tool_kills_child` (typed
`TOOL_ERROR`, failed terminal, transport marked dead so the next
`send()` is refused rather than queued). Two further agy shapes needed
**no new fidelity at all** and are cited by name in both doc comments
rather than re-derived: `death_without_terminal` is already agy's
group-SIGKILL shape (W1 P4), and `with_server_minted_native_ids`
already generalizes agy's own `native_id: None` at PREPARE. Nine new
tests, two of which exist specifically to prove the two *reused*
shapes match agy's own admission rows by name.

### The admission-rows / L8-structural pattern — reused verbatim

Contract v1 (`src/backend/mod.rs`) is untouched as a **contract** (R3):
thirteen booleans plus `AskAuthor`/`RuntimeScope`, no typed capability
enum, no new trait method, no new event kind. The file itself gains
exactly one thing — a `pub mod agy;` declaration and its doc comment —
which is on the K2 ledger below rather than hidden inside "adapter
work".

This adapter's `AdmissionRow` ledger is ADR 0020's own pattern, copied
rather than re-derived exactly as ADR 0021 copied it before: **38 rows**
(13 v1 booleans × 2 transports, plus 12 adapter-local rows), each with
`capability | transport | claimed | tier | evidence | admission_test |
note`, and three compile-adjacent structural tests rather than a review
discipline — `admission_rows_agree_with_capabilities` (a claimed row
must name an admission test, an unclaimed row must not, and `claimed`
must agree with `Backend::capabilities()`),
`every_admission_test_name_resolves_to_a_real_test` (the name is read
back against the text of this module and of `tests/agy_backend.rs`, so
a typo'd or later-renamed test cannot sit in the ledger citing
something that does not exist — a latent gap inherited from the
opencode template, closed here), and
`a_claimed_row_naming_a_live_test_is_labelled_live_measured` (the
`Evidence` tier must agree with whether the named test is a `live_agy_*`
one). Like opencode's ledger and unlike codex's there is **no
`Stability` column**: every row would carry the identical value, so the
fact is stated once in `render_admission_rows`'s own header — here with
this sprint's extra reason, that the installed build moved
1.1.17 → 1.1.19 during the sprint itself.

`agy.rs` declares **no new `KIND_*` constant**. One would force an
`api::SSE_EVENT_KINDS` edit to satisfy `tests/m6_surfaces.rs`'s `t6`,
which is core (K2); instead it reuses `KIND_TURN_HARNESS_ERROR` (from
`codex.rs`, already in that vocabulary and already meaning "the harness
said something went wrong"), distinguished by a `phase` field in the
payload.

**The divergence between the two transport columns is deliberately
small**, the same honest result ADR 0021 reports for opencode's own two
columns — but for a different reason, worth naming because the two
adapters look superficially alike here. OpenCode's two transports
differ in *flags* (serve adds `approval_flow` and `ask`). Agy's differ
almost entirely in **cost and timing**: zero-quota identity, a pre-turn
pin refusal, a zero-quota resume-fork check, no argv cap
(`prompt_channel`, tier `StdinNdjsonNoArgvCap` — the prompt travels as
one NDJSON line's content, so the measured 131072-byte `E2BIG` argv
wall does not bind, though `LOOP_PROMPT_CAP` still refuses at PREPARE
at 16 MiB, because "no measured limit" is not "no limit"). Those wins
live in tiers and adapter-local rows, not in flipped v1 booleans.
**Exactly one boolean moves between the two `Capabilities` values** —
`native_subagents`, on the typed record W3 A1 demanded — and
`ask`/`approval_flow` do **not** move: they are measured false on both,
a refutation stated independently twice rather than inherited across a
transport boundary, because a claim carried across transports is a
claim nobody made.

## K2 exception ledger — every touch outside `src/backend/agy.rs` this
sprint made, all four waves, complete, for owner ratification at the
head PR

K2's own text is "scope is the adapter; no core changes," with the
`sgt agy` passthrough pre-ratified by name. Everything else that
landed outside `src/backend/agy.rs` is here — eleven items, each with
its wave, its files, and the reason it was necessary rather than
chosen:

| # | Item | Wave | File(s) | Reason |
|---|---|---|---|---|
| 1 | The module declaration | W1 | `src/backend/mod.rs` (+7) | A `pub mod agy;` line and its doc comment — the entire edit. The **contract** in that file (the `Backend` trait, `Capabilities`, `AskAuthor`, `RuntimeScope`) is byte-unchanged; a module that is never declared is a module that never compiles. Listed first, and separately from "adapter work", precisely because this is the one core *file* K2's promise is most naturally read against. |
| 2 | Required PUT-site + recovery-arm row | W1 | `tests/a4_blob_ref_pinning.rs` (+127) | Gate-forced, not discretionary: A4's blob-ref-pinning suite requires every new blob-capture site (agy's raw-stream archive) to carry a recoverability row, or the suite fails closed on its own. The adapter cannot exist without this row; it is that suite's admission gate operating exactly as designed. |
| 3 | The adapter's own suites | W1–W3 | `tests/agy_backend.rs` (+3602), `tests/agy_routing.rs` (+254) | Outside `src/backend/` only because integration tests live at the crate root by this repo's existing layout — template-pre-ratified by the codex/opencode precedent. Named for the ledger's completeness, not because either file is a scope question. |
| 4 | Fixtures captured at 1.1.19 | W1, W3 | `tests/fixtures/agy-1.1.19-*` (26 files) | The evidence itself: 13 print-transport captures (W1) and 13 loop captures (W3), each cut from a paid or free measurement and named for the version it was captured at, never backdated to `MEASURED_FLOOR`. |
| 5 | Coverage stages wired at birth | W1, W2 | `scripts/coverage/c2-suites.sh` (+19) | Gate-forced by `tests/coverage_stage_membership.rs` (built during the opencode sprint, ADR 0021, #231(b)): a new suite is wired into a coverage stage or named in the `ALLOWLIST` in the same commit that creates it, or the guard fails closed. `c2-agy_backend` and `c2-agy_routing` were each wired in their own creating commit — neither was ever an orphan, and this sprint added nothing to the 18-suite allowlist. |
| 6 | `"agy"` registered in `BackendRegistry` | W2 | `src/daemon.rs` (+31) | `DaemonConfig` gained an `agy: Option<AgyConfig>` field plus the construction/registration and event-sink-wiring block, a line-for-line mirror of the opencode block directly above it. A real edit to a core file, named as such — even though ("Harness and backend axes," above) it decides nothing new: the routing ladder and `sgt agy`'s origin affinity already existed, and registration only makes the name they already resolve toward real. |
| 7 | The `sgt agy` passthrough | W2 | `src/cli.rs` (+17/-9), `src/harness.rs` (1 line) | **K2's own pre-ratified exception**, named by the ruling: the exact mechanical mirror of the `goose` block (ADR 0006 D2) — origin-affinity routing has no origin to affine from without it. A `Command::Agy` variant and its `dispatch` arm in `cli.rs`; `harness.rs`'s change is **doc-only**, a stale verb-list line (`claude, codex, opencode, goose` → `…, agy`). `toolchain_path_dirs`, `prepare` and `exec` needed **zero** code change — unlike the opencode sprint, agy's `~/.local/bin` was already on the toolchain path, so this sprint added no PATH line at all. |
| 8 | Registered-backend count/list widened | W2 | `tests/m2_daemon_api.rs`, `tests/m3_execution.rs`, `tests/m4_backends.rs` | Mechanical fallout of item 6, in the same commit, never a separate decision: a sixth registered backend moves every fixture that hardcoded how many backends exist (m2 12→13 events, m3 backend lists, m4 4→5 probed). Each edited by name. |
| 9 | Fake-backend fidelity fallout | W4 | `tests/m4_backends.rs` (+3) | Mechanical fallout of the three new `FakeStep` constructors in `src/backend/fake.rs`, in the same commit: one pre-existing struct literal gains the three new fields. (`fake.rs` itself is inside `src/backend/` and is not a ledger item; the fidelity work it carries is described above.) |
| 10 | The 0.2.3 version bump | W4 | `Cargo.toml`, `Cargo.lock` | K3, named by the ruling. The `sergeant-rs` package's own `version` line is the only change; `Cargo.lock` regenerated `--offline`, no dependency touched, **no new crate and no new feature flag anywhere in this sprint** — the loop transport is plain stdio, so the dependency carve-out the opencode sprint used in its narrower form has no analog to spend here. |
| 11 | Release and reference documentation | W4 | `CHANGELOG.md`, `README.md`, `docs/DEVELOPMENT.md`, `docs/adr/0022-agy-adapter.md`, `docs/proposals/agy-adapter-2026-08-23.md` | The finalize files the plan's own W4 section names: the 0.2.3 changelog entry, one `README.md` quickstart line (`sgt codex / sgt opencode / sgt agy / sgt goose`), the `DEVELOPMENT.md` backend-list sentence, this ADR, and the plan's own as-landed postscript. Documentation of the sprint, not sprint scope creep — but on the ledger, because "outside `src/backend/agy.rs`" is a mechanical test and this record should pass it as written. |

None of the eleven touches the `Backend` contract, the router, the
engine, or the event vocabulary — K2's actual substance (R3) is
intact. They are named individually, per ADR 0021's precedent, so a
reader auditing "no core changes" can check each item against the
reason it was actually necessary rather than take "the adapter" on
faith.

## Consequences

Agy is a fully registered, routed, capability-honest **fourth native
backend** alongside Claude, Codex and OpenCode, on two transports
neither of which shares the other's denied-tool shape — both handled,
neither assumed, and the divergence stated in the ledger rather than
averaged into one story. Every capability it claims carries a named
admission test or a named, specific negative, enforced by three
structural tests rather than by review discipline;
`native_subagents: true` on the loop transport is a registry first,
earned on a typed record and nothing weaker.

Two refutations are recorded as **results**: `ask` and `approval_flow`
are measured false on both transports with the probe transcripts to
back the negative — opencode keeps the registry's only `true` on either
flag — and the SIGINT interrupt upgrade was tried, measured, and
refuted, banked as a classifier amendment instead of a capability tier.
Two adjacent adapters, structurally similar upgrade attempts, opposite
measured answers, neither forced to match the other: that is R4 working
in the direction it is least comfortable in.

The R1 provenance posture held through this repo's first genuinely
mid-sprint version bump (1.1.17 → 1.1.19) without exception or special
pleading, and through one attribution correction made in the open. No
native sandbox exists for this adapter to claim — stated as a fact
about agy, corroborated by the binary's own strings rather than by
documentation's silence, and not a gap in this adapter's coverage.
The permission story ships as the panel's two-rung ladder: a measured
injection channel and a launch-time honesty report that ships whether
or not the channel is used, with the policy layer left explicitly
unbuilt rather than invented. The K2 exception ledger above is this
sprint's complete, honest account of every place work outside
`src/backend/agy.rs` was actually required.

**Live-turn spend, whole sprint, K1-pinned except where named:** W1
spent 9 of 15 budgeted; W2 spent **zero** (stub-driven end to end); W3
spent 13 of 20 (5 spec probes + 7 admissions + 1 accidental, X0, the
sprint's only unpinned turn); W4's own budget is 5, reserved for the
end-to-end proof — the fake-fidelity, doctrine and finalize work
recorded here spent none.

## Open questions / hand-offs

- **`profiles` is a DECLARED DIVERGENCE, not a full claim.** Generic
  sergeant axes (executable + env) reach every turn on both transports
  (`a_profile_executable_and_env_reach_every_turn`,
  `a_profile_executable_and_env_reach_a_loop_child`), and a profile's
  `config_home` is **refused, not ignored** — honoring it by guessing
  an agy config-home variable that measurement says does not exist
  would be the adapter inventing a launch decision. But agy's own
  `--agent`/custom-agent mechanism is unwired: `agy agents` printed an
  empty list on this host, and defining one by the documented workspace
  mechanism did not work either [W1 P6, a free step; **zero** live
  turns spent on it]. A future wave on a host where `--agent` actually
  resolves is the hand-off — and opencode's precedent applies verbatim
  when it does: an agent applied to turn 1 must be re-applied on every
  resume, and that re-application is unmeasured.
- **How deep to surface a subagent's own trajectory.** The typed
  `subagent_info` carries a `log_uri` to the child's
  `transcript.jsonl`; this wave carries the child `conversation_id` and
  that URI verbatim into `tool.completed` so a human can follow them by
  hand, and stops there. Whether sergeant should read that transcript,
  project child events into the parent's graph, or expose the child as
  an addressable sub-execution is a **core** surface question (it is
  not obviously representable in today's execution model), deliberately
  not answered by one adapter's wave.
- **`proceed-in-sandbox` as a second permission channel** (W3 S1) is
  evidenced but unusable on this host — the gate lifts, the sandbox
  server is absent. A future wave on a host with a working sandbox
  server could re-measure it as a genuine channel needing no per-Work
  allow-rule synthesis, distinct from the `HOME`-relocated settings
  channel this sprint shipped. That would also be the wave with
  something for NORTH-STAR amendment 4 to bind.
- **Mapping a Work's declared mutation surface onto agy's
  `permissions` namespaces** (`command`, `read_file`, `write_file`,
  `read_url`, `mcp`, `execute_url`, `unsandboxed` — the binary's own
  authoritative regex) is explicitly unbuilt across W1 and W3 both: a
  security decision this sprint declined to invent without measurement
  behind it, named as a hand-off rather than deferred silently.
- **The X0 hazard is standing, not closed.** `--disable-slash-commands`
  composed with a slash-command prompt silently converts the zero-quota
  introspection path into a paid, unpinned turn. The adapter's own
  probe composition avoids it; nothing structurally prevents a future
  probe author from repeating it. A guard — refusing that flag
  combination at composition time — is a candidate, unbuilt this
  sprint.
- **Free-tier quota shape is unmeasured** (packet open question 5): a
  429, a hang, and a typed error are all plausible and none was
  observed. Live tests are opt-in-gated, bounded and small, and never
  required CI checks, which is the mitigation rather than an answer.
- **Upstream churn is the live risk, and the sprint already
  demonstrated it.** The installed build moved 1.1.17 → 1.1.19 *during*
  this sprint and the print-mode denial shape inverted with it.
  Antigravity publishes no CLI breaking-change policy that this sprint
  could find — the same open question ADR 0021 records for opencode,
  carried here for the same reason (R1's posture does not depend on the
  answer) but with a sharper standing hand-off: a build past 1.1.19
  should be re-measured against the two denial fixtures before it is
  assumed to behave like either of them.
- **Conversation scoping beyond resume-by-id is unmeasured**: the
  `resume` rows state exactly what probe 5 proved (resume by id, same
  host, same user) and no more. `--project`'s identity effects and
  `--continue`'s cwd→id cache semantics are untouched, and the adapter
  composes neither flag.
- **Whether agy runs tool commands in their own process group** is
  unmeasured [W1 P4]: `ps -g <pgid>` listed only the `agy` leader while
  a tool-spawned `sleep 120` was in flight. A group kill is correct
  either way, which is exactly why `kill_process_group` is reused
  without re-arguing the question — but a probe with a longer-lived
  tool child could settle it.
- **`history` is `false` on both transports** and is a real absence,
  not an oversight: agy documents no export verb, and §15 forbids
  emulating one. OpenCode's `history: true` (ADR 0021) stands alone in
  the registry.

## Alternatives considered

**Shipping on agy's defaults for tool-bearing Work stages** — refused
by the plan's own panel amendment before any wave ran, and the
measurement backs it: [packet 2] and W1 P2's control both show the
default posture auto-denying every tool call, so an adapter on defaults
cannot execute a tool-bearing stage at all. The two-rung ladder (a
measured injection channel, plus a launch-time honesty report
regardless) is what was actually built.

**Defaulting `--dangerously-skip-permissions`, or composing `--sandbox`
by default** — refused. The first is claude #47's settled answer and
this module composes it nowhere; the second would be an invented launch
decision *and*, on this host, a broken one (W3 S1's connection reset).

**A SIGINT-based interrupt upgrade** — tried, measured, refuted (above):
the resulting terminal is byte-identical to a plain deadline expiry,
so the upgrade would trade a measured guarantee for a mislabelled one.
`ProcessTreeTermination` stands, carried rather than upgraded — unlike
opencode's own `NativeSessionAbort` upgrade (ADR 0021). Structurally
similar attempts, genuinely different measured outcomes, recorded as
such rather than forced to match.

**Guessing an actor question from `ask_question`'s presence in the tool
roster, or from narration** — refused directly: `Capabilities::ask`'s
contract forbids inferring authorship from prose, and W3 P1's
measurement shows the stronger fact that even a typed question on this
transport has no channel to answer it on. [changelog 1.1.12]'s own
"headless `-p` runs … settle a choice themselves where they would
otherwise ask" is evidence against a question surfacing at all.

**Claiming `profiles` on agy's own `--agent` mechanism** — refused for
lack of a working mechanism to measure, not for lack of interest: the
documented custom-agent definition path did not work on this host and
zero live turns were spent chasing it. The row says `true` on generic
axes with the divergence declared, rather than `true` on a mechanism
nobody exercised.

**Declaring a new `KIND_*` event constant for agy's harness errors** —
refused: it would force an `api::SSE_EVENT_KINDS` edit, which is core
(K2). `KIND_TURN_HARNESS_ERROR` already carries the meaning, and a
`phase` field in the payload carries the distinction.

**A typed `Capabilities` v2 enum for this adapter alone** — refused by
R3, identically to ADR 0020/0021's refusal of the same idea; the
adapter-local `AdmissionRow` ledger is what was actually built,
designed to lift whole into a real v2 when one lands, with nothing
thrown away in the meantime.
