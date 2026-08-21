# Recorded fixtures

`claude-2.1.226-turn.jsonl` — four verbatim stream-json lines from one real
print-mode turn, recorded 2026-08-09 in this container against Claude Code
2.1.226:

```
IS_SANDBOX=1 claude -p --verbose --output-format stream-json \
  --setting-sources user --session-id 3f2a9c14-... --model haiku \
  --dangerously-skip-permissions
prompt: "Run the bash command: echo hello-fixture. Then reply with exactly: OK"
```

The turn's other 22 lines (`system:thinking_tokens`, `system:init`,
`rate_limit_event`, thinking blocks, `post_turn_summary`) are omitted; the
four kept lines — assistant `tool_use`, user `tool_result`, assistant `text`,
and the `result` envelope — are byte-for-byte as the CLI emitted them.
Nothing here is authored: this is a recording, and it is what the
deterministic §20/§27 tests replay.

## `claude-2.1.226-post-turn-summary-no-ask.jsonl` — recorded, from another turn

One verbatim `system`/`post_turn_summary` line, recorded 2026-08-10 against the
same 2.1.226 build during the N3 ask measurement
(`docs/gauntlet/notes/n3-claude-ask-measurement.md`, "prompt B"): a turn that
was told to answer and ask nothing, so `needs_action` is the empty string.

It is kept separately, and named separately, because it is **not** from the
turn above — that turn's own `post_turn_summary` was among the 22 lines the
recording dropped, and inventing it back would be exactly the fabrication this
directory refuses. Tests that need a *complete* turn splice this line in before
the recorded `result` envelope (`recorded_turn()` in `tests/m4_backends.rs`),
which is where 2.1.226 emits it, and the splice is a documented composition of
two recordings rather than a claim to be one.

Why any test needs it: the adapter withdraws `Capabilities::ask` when a turn
completes with no `post_turn_summary` at all (INV-N3-06 — the absence used to
fail open). Replaying a stream that omits the line is therefore a stream that
says "this CLI has lost the ask grammar", which is true of the fixture and
false of the CLI.

## `claude-2.1.226-substitution-envelope.derived.json` — derived, not recorded

**This file is not a recording, and the name says so.** The M4 contract's
acceptance 2 asks for the substitution path to be "unit-tested against a
recorded fixture of the substitution envelope", and no such recording can
exist here: print-mode substitution cannot be provoked on this account (it is
entitled to every model the pin would ask for), and the spike's actual
substitution evidence is a *TUI transcript warning line*, not a print-mode
result envelope (`reference/sergeant-upstream/docs/research/
claude-background-harness-spike.md`, "the warning"). Recording it would
require an unentitled account, which is an environment, not a test.

So this fixture is *derived* from the recorded envelope above by exactly
three edits, and nothing else:

1. the `modelUsage` key `claude-haiku-4-5-20251001` → `claude-sonnet-5-20260101`
2. its `canonicalModel` `claude-haiku-4-5` → `claude-sonnet-5`
3. `result` → `"mission accomplished"` (the spike's point: the mission
   succeeded while the pin did not hold)

Every other byte — `is_error: false`, `subtype: "success"`,
`api_error_status: null`, the usage block, the cost — is the measured 2.1.226
shape. What the substitution test therefore pins is the adapter's fail-closed
*rule* (an envelope whose model fields do not match the pin is substitution,
whatever the mission said), against a measured envelope shape, with a
scenario taken from the spike. It does not pin a measurement of print mode's
substitution surface, and the test says so in its own doc comment. Keeping
the derivation on disk, rather than as a `json!` literal inside the test,
is what makes the difference between "recorded" and "derived" reviewable.

## `docker-stage-completed.payload.json` — recorded, from a real docker run

W2's A4 pinning suite (`tests/a4_blob_ref_pinning.rs`) needs a `stage.completed`
event payload shaped exactly as the real Docker executor produces it —
`detail` a *stringified* JSON object, not a nested object — to prove
`blob::refs_in_payload`'s recursive walk recovers the two blob refs inside it
while a flat top-level scan recovers none (A4's whole claim).

Captured 2026-08-21 on this container, real Docker Engine, via
`DockerBackend::observe` over a container running
`sh -c "echo hello-stdout-a4-fixture; echo hello-stderr-a4-fixture 1>&2; exit 0"`
— the exact `BackendSignal::StageCompleted { summary }` string the adapter
produced, wrapped in the `{"stage_id", "index", "detail"}` shape
`Engine::settle_launch` (`engine.rs`) journals it under. Every field —
`image_id`, the two timestamps, the two BLAKE3 refs, the two tails — is the
real captured value; nothing here is authored. The one-off capture test used
to produce it is not kept in the tree (it duplicates `m7_docker_executor.rs`'s
own `exit_zero_completes_and_nonzero_fails_with_captured_evidence` harness
almost exactly); reproduce it by running that shape against any command and
copying the resulting `summary` string.

## Codex fixtures (W1, `src/backend/codex.rs`)

All six recordings below are copied byte-for-byte from the H0 seats' own
captured stdout/stderr under `/var/tmp/codex-probe/` (volatile scratch, never
committed itself), against **codex-cli 0.149.0**, standalone musl build,
Cerberus, 2026-08-21. Nothing here is authored — see `w1-spec.md`'s Appendix A
for the same recordings inline, with the run that produced each one named.

- `codex-0.149.0-agent-message-turn.jsonl` — source
  `sgt-adapter-research/runA2.jsonl`. The plain 4-line turn: `thread.started`
  → `turn.started` → `item.completed{agent_message}` → `turn.completed{usage}`.
  Identical shape recorded independently in `run1.jsonl`/`runF/G/H/L.jsonl`.
- `codex-0.149.0-command-execution-turn.jsonl` — source
  `auth-sandbox-approvals/run4_bypass.jsonl`. The only recording containing
  real tool evidence: `item.started` + `item.completed` for a
  `command_execution` item, with `exit_code`/`status`/`aggregated_output`.
  Required `--dangerously-bypass-approvals-and-sandbox` to produce (this
  host's bwrap cannot initialize a sandbox at all).
- `codex-0.149.0-turn-failed.jsonl` — source `sgt-adapter-research/runB.jsonl`,
  `-m gpt-5.6-nonexistent-model`. An `item.completed{error}` model-metadata
  *warning* (does not end the turn) → a stream `error` line → `turn.failed`,
  carrying the API's 400 message. Pins that a stream `error` is journaled and
  never a terminal, and that `turn.failed` is (§4.4).
- `codex-0.149.0-uncorroborated-narration-turn.jsonl` — source
  `auth-sandbox-approvals/run2_onrequest.jsonl`. §4.3's hazard: two
  `agent_message` items narrating a specific low-level command failure
  (`bwrap: loopback: Failed RTM_NEWADDR`) with **no** `command_execution` item
  anywhere in the stream and exit 0. Same shape independently recorded in
  `run3_wwrite.jsonl`, `runC.jsonl`, `runJ.jsonl`. This is the fixture
  `narration_produces_no_tool_events` replays — the single most important test
  in the wave (§4.3, §7.2 item 4).
- `codex-0.149.0-untrusted-dir-refusal.stderr.txt` — source
  `notgit/run5_notrust.stderr.log` (its paired `.jsonl` is 0 bytes, not kept
  separately since an empty file carries nothing a test reads). The pre-turn
  refusal shape: plain-text stderr (`Reading additional input from
  stdin...\nNot inside a trusted directory and --skip-git-repo-check was not
  specified.`), zero stdout JSON, nonzero exit — caught at LAUNCH and turned
  into a `BackendError::Failed` before any turn is registered (§3.1, §5.2).
- `codex-0.149.0-parse-error.stderr.txt` — source
  `sgt-adapter-research/runA.stderr.log` (paired `runA.exitcode` = 2, recorded
  here as the fact rather than a seventh file: `codex exec -a` is a clap parse
  error, exit 2 — `-a`/any approval flag cannot be passed to this transport at
  all, measured-negative).

No derived fixture was needed for W1: unlike Claude's substitution envelope,
every shape this adapter's decoder decides on was actually recorded on this
build. `w1-spec.md` Appendix A also records two facts these fixtures do not
carry a file for — the `--output-schema`/`-o` agreement
(`sgt-adapter-research/runE.jsonl` + `last_message.txt`, both byte-identical
`{"word":"ok"}`, which is why the adapter never passes `-o`) and the rollout
directory layout (`~/.codex/sessions/<y>/<m>/<d>/rollout-…-<thread_id>.jsonl`)
— neither is fixture-replayed by any test, so neither is copied here.

## App-server fixtures (W3, `src/backend/codex_appserver.rs`)

Captured token-free where noted, live otherwise, against **codex-cli
0.149.0**, `~/.local/bin/codex`, Cerberus, 2026-08-21, while writing the W3
spec's own re-measurement (§0.3) and again while implementing it — the
scratch driver lived under `/var/tmp/codex-w3-probe/` (volatile, never
committed itself). Nothing here is authored except the two files marked
`.derived.`, and each of those states exactly what was changed and why.

- `codex-appserver-0.149.0-handshake-and-thread-start.jsonl` — recorded,
  token-free (no `turn/start` ever sent): three requests
  (`initialize`/`initialized`/`thread/start`) driven over `--listen
  stdio://`, the seven lines the child wrote back. Pins the wire rules §1.5.1
  claims: no `jsonrpc` field on anything received, `configWarning` and
  `remoteControl/status/changed` arriving unsolicited and *before* the
  `thread/start` result they have nothing to do with, and the sandbox/policy
  echo on `thread/start`'s result.
- `codex-appserver-0.149.0-experimental-api-required.json` — recorded,
  token-free: the same session without `capabilities.experimentalApi`,
  `thread/start`'s `-32600` refusal verbatim.
- `codex-appserver-0.149.0-server-request-methods.txt` — the 11
  `ServerRequest` method names, `[schema-claimed]`, dumped from
  `generate-json-schema`'s `ServerRequest.json`.
- `codex-appserver-0.149.0-thread-item-types.txt` — the 18 `ThreadItem`
  variant names, `[schema-claimed]`, from the same schema dump.
- `codex-appserver-0.149.0-schema-fingerprint.txt` — the pinned 14-file list,
  the digest rule, and both the SHA-256 the spec quotes (re-measured
  identical: `91d5ac1…`) and the BLAKE3 this codebase actually pins as
  `MEASURED_PROTOCOL_FINGERPRINT` (`d0fbf8d…`) — blake3 is already an in-tree
  dependency, sha256 is not (§2.5's own instruction: "the implementer
  re-computes the constant with blake3 and pins that").
- `codex-appserver-0.149.0-completed-turn.jsonl` — recorded, one real live
  turn (`gpt-5.6-luna`, "Reply with exactly the word ok and nothing else."):
  the full item stream including `reasoning` items, `item/agentMessage/delta`
  (counted, never decoded), two `thread/tokenUsage/updated` pushes *before*
  `turn/completed`, and the terminal itself (`status: "completed"`, no
  `usage` on the turn object — confirming §2.3's "usage arrives on its own
  notification, not on the terminal" claim measured, not assumed).
- `codex-appserver-0.149.0-turn-failed.jsonl` — recorded, one real live turn,
  `-m gpt-5.6-nonexistent-model`: a `warning` notification (model metadata not
  found, turn continues), then a bare `error` notification and `turn/completed
  {status:"failed"}` carrying the same message, `codexErrorInfo: "other"`
  (measured — not one of the arms the spec's prose lists by name, which is
  itself a finding: the taxonomy's schema `oneOf` does include `"other"`
  alongside `"unauthorized"`/`"badRequest"`/etc., confirmed by reading
  `CodexErrorInfo` out of the schema dump directly).
- `codex-appserver-0.149.0-interrupted-turn.jsonl` — recorded, one real live
  turn interrupted mid-flight (`turn/interrupt` sent after the first
  `item/started`), **the single most load-bearing capture in this wave**: it
  shows `turn/interrupt`'s `{"result":{}}`, the `turn/completed
  {status:"interrupted"}` that follows, and then a **second** `turn/start` on
  the *same* thread completing normally — live proof of §2.2's whole claim
  (a distinct terminal, and the conversation stays resumable) driven through
  this adapter's own per-execution stdio child, not just the daemon control
  socket the spec's own M10 row cites.
- `codex-appserver-0.149.0-command-execution-item.derived.jsonl` — **derived,
  not recorded.** A live `commandExecution` item could not be captured on
  this host: every attempt (`auth-sandbox-approvals` capture for W1, and a
  fresh attempt for W3) hits the same measured wall — bubblewrap cannot
  initialize a network namespace here (`bwrap: loopback: Failed RTM_NEWADDR:
  Operation not permitted`), and unlike the exec transport there is no
  `--dangerously-bypass-approvals-and-sandbox` escape hatch on `thread/start`
  that was tried; the model narrated the sandbox failure in prose instead of
  ever producing a `commandExecution` item at all. So this file is the
  W1 exec-transport recording (`codex-0.149.0-command-execution-turn.jsonl`)
  translated key-for-key into app-server's measured shapes: the envelope
  (`{"method":"item/completed","params":{"item":…,"threadId":…,"turnId":…}}`),
  the item type (`command_execution` → `commandExecution`), and every field
  M9 measured (`exit_code`→`exitCode`, `aggregated_output`→`aggregatedOutput`,
  plus the app-server-only `cwd`/`durationMs`/`processId`/`source`/
  `scriptPath`/`commandActions`/`pluginId` fields, populated with plausible
  values since W1's recording never had them to translate). Every value that
  *was* measured (the command string, the output text, the exit code) is
  unchanged from the recording; every field that only app-server has is new
  and is exactly what this note says it is.
- `codex-appserver-0.149.0-unauthorized-turn-failed.derived.json` —
  **derived, not recorded.** §2.8's test needs `codexErrorInfo: "unauthorized"`
  and this account is authenticated, so it cannot be provoked. Derived from
  the real `turn-failed.jsonl` recording above by exactly two edits: the
  `error.message`'s embedded JSON string and `codexErrorInfo` changed from the
  measured `400`/`"other"` (bad model) shape to a `401`/`"unauthorized"` shape
  — both real arms of the same `CodexErrorInfo` schema enum, confirmed present
  in the schema dump; nothing else in the envelope moved.
- `codex-appserver-0.149.0-request-user-input.derived.json` — **derived, not
  recorded.** §2.4's five-step admission test never got past step 3 on this
  build (below): this is `item/tool/requestUserInput`'s params built strictly
  from `ToolRequestUserInputParams`'s own schema-required fields
  (`isBlocking`, `itemId`, `questions`, `threadId`, `turnId`, and each
  question's required `id`/`header`/`question`), with the thread/turn ids
  reused from the real `interrupted-turn.jsonl` capture above so the fixture
  at least names a thread that really existed.

**§2.4's `ask` measurement, done live and recorded here rather than as a
sixth fixture** (its outcome is a negative, which is why nothing above
carries it): two prompt formulations were tried against `gpt-5.6-luna`,
`approvalPolicy: "never"`, on a fresh thread each. Formulation 1 ("ask me
what topic before doing anything else") produced the question as
plain-text `agentMessage` prose, never a tool call. Formulation 2 ("you
must call the request_user_input tool") produced an `agentMessage` reading
*"I can't call `request_user_input` because this session isn't in Plan
mode."* — a new, measured, and more specific reason than "the model didn't
feel like it": the tool itself may be gated behind a thread mode this
adapter's `thread/start` params never request. Per §2.4's own outcome
table this is exactly the first bullet ("step 3 never fires after both
prompts → `evidence: Unmeasured`") with an even more specific note than the
spec anticipated, and `ask` stays `false` for the app-server transport too.
