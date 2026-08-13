# MVP-2 D2 — `--setting-sources` de-leak + interrupt usage, measured

Governing: `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md` MVP-2 table,
lane D2; `docs/gauntlet/contracts/N4.md`'s adjudication (D2 is not that
contract, but the same session's lane brief cites it for context);
LESSONS L1 ("measure the Claude CLI, never trust its docs or its exit
codes"). Host: Cerberus, `docs/environments/cerberus.md`. Installed CLI at
measurement time: `claude --version` → `2.1.228 (Claude Code)` (note: this
is one patch ahead of `docs/environments/cerberus.md`'s 2026-08-11 row of
2.1.227 — drift is a fact to record, not a blocker, per that file's own
convention; `MIN_TRUSTED_VERSION` in `src/backend/claude.rs` stays pinned at
2.1.226, the version these contract tests measured, and 2.1.228 passes that
gate).

Total real-token spend across every probe in this note: **~$0.09** (eight
bounded `claude-haiku-4-5` turns), well inside the ≤$2 lane budget.

## Item 1: what `--setting-sources` actually controls

### Setup

Scratch git repo (outside this checkout, in the session scratchpad),
containing an `AGENTS.md` (Sergeant's `INSTRUCTION_FILE`) with a distinctive
marker:

```
# Project memory marker

If you are asked for "the secret codeword", the answer is: zebra-pineapple-77
```

Prompt (every run, `--model claude-haiku-4-5`, `--output-format
stream-json`): *"What is the secret codeword? Answer with ONLY the codeword
itself, or the single word NONE if you don't know one."*

### Runs and results

| `--setting-sources` value | file present | result |
|---|---|---|
| `user` (today's `Suppress` grammar) | `AGENTS.md` only | `NONE` |
| `user,project` | `AGENTS.md` only | `NONE` |
| *(no flag at all — CLI default)* | `AGENTS.md` only | `NONE` |
| *(no flag at all)* | `AGENTS.md` **and** `CLAUDE.md`, identical content | `zebra-pineapple-77` |
| `user` | `AGENTS.md` and `CLAUDE.md` | `zebra-pineapple-77` |

The fourth row is the control: it proves the measurement methodology can
detect the marker at all (native discovery of a file named `CLAUDE.md`
fires with zero flags). The fifth row is the load-bearing one: `--setting-
sources user` — the exact grammar `Suppress` sends today — does **not**
suppress it.

### What actually happened (traced via full stream-json capture)

For the `CLAUDE.md`-present runs, the model does not receive the file's
content injected into its system prompt at all. Instead it **actively calls
its own `Read` tool** on `CLAUDE.md`, in every case, motivated by its own
default system prompt (visible in the `thinking` block: "let me check my
memory / project files"). This is identical whether `--setting-sources` is
absent, `user`, or `user,project`. It never happened for `AGENTS.md` under
any of those three combinations — the model's default system prompt only
nudges it to check the literal filename `CLAUDE.md`; `--bare --help`'s own
text calls this mechanism "CLAUDE.md auto-discovery" and lists it as a
*separate* thing `--bare` disables, distinct from settings sources.

### Conclusion

`--setting-sources` governs which `.claude/settings*.json`-style
configuration files load (permissions, hooks, MCP server config) — **not**
whether the actor's own agentic loop reads a memory file already sitting in
its cwd. Two consequences, both now stated in
`src/backend/claude.rs::setting_sources_args`'s doc comment and the module
doc:

1. `Suppress`'s existing grammar (`--setting-sources user`) was never
   actually suppressing `AGENTS.md` consumption for this adapter — there was
   nothing native tied to that filename to suppress in the first place. It
   still earns its keep for what the flag *does* gate, so it is unchanged.
2. `InstructionPolicy::Local`'s original design intent ("the actor would
   consume the repository's own instruction file natively") has no
   `--setting-sources` value that implements it for `AGENTS.md`. What
   `Local` actually widens, honestly: whether the repository's own
   `.claude/settings.json` / `.claude/settings.local.json` — hooks, tool
   permissions, MCP servers, potentially command execution via a
   repo-authored hook — take effect. That is real, repo-controlled surface,
   and larger than "reads a text file", not smaller.

### Translation implemented

`Suppress` → `--setting-sources user` (unchanged). `Local` →
`--setting-sources user,project,local` (loads every settings source the CLI
defines — the literal, measured scope of what this flag controls). Pinned
by `d2_instruction_policy_translates_to_the_measured_setting_sources_value`
(`tests/m4_backends.rs`) against a stub, and by
`r_mvp1_4_local_instructions_policy_is_accepted_at_submit_and_reaches_the_backend`
(`tests/m3_execution.rs`) end to end over HTTP submit. The submit-time
refusal `InstructionPolicyUnmeasured` is removed
(`Engine::check_instruction_policy`); the mixed-policy conflict refusal is
unchanged.

`Engine::resolve_instruction_identities`'s content-hash pin of `AGENTS.md`
is unaffected and still runs unconditionally — it is honest bookkeeping
about the file a native-read mechanism *would* read, not a claim that one
exists for this adapter under either policy.

## Item 2 (context, not new measurement here)

Capability provenance durability (cv2 item 7) is implemented and tested in
`src/backend/claude.rs` (`seed_capability_provenance`,
`latest_ask_withdrawal_version`) and `src/daemon.rs` (wired at backend
registration, before the registration probe is journaled) — see those
modules' doc comments and `tests/m4_backends.rs`'s
`d2_a_journaled_ask_withdrawal_survives_a_daemon_restart_at_the_matching_version`.
No live-CLI measurement was needed for this item beyond the existing 2.1.227
ask-grammar withdrawal already on record
(`docs/gauntlet/notes/cerberus-ask-grammar-remeasurement-2026-08-11.md`);
this note only carries this item's cross-reference for completeness.

## Item 3: what the CLI emits on interrupt

Bounded `claude-haiku-4-5` turns asked to write a ~600-word essay (long
enough to still be generating a few seconds in), killed mid-stream two ways:

### SIGKILL (`Child::kill()` — what `ClaudeBackend::interrupt`/`stop` actually send)

- Killed **before any `assistant` content had streamed**: raw capture is one
  `system`/`rate_limit_event` line and nothing else. No envelope, no usage,
  no text.
- Killed **after the first `assistant`/`thinking` chunk had streamed**: raw
  capture includes that chunk, which carries its own `usage` object
  (`input_tokens`, `cache_creation_input_tokens`, `output_tokens: 4` at that
  point) — but still **no terminal `type:"result"` envelope**. This confirms
  the module docs' existing "measured: no result envelope" claim, and adds
  the finer fact: whatever partial `assistant` chunks did stream before the
  kill remain in the raw archive (already durable today via `BlobStore`) and
  may carry a `usage` snapshot of *that chunk's own accounting* — not a
  cumulative turn total, and not currently parsed out by this adapter.

### SIGTERM (not a mechanism this adapter sends — measured for the contrast)

The CLI traps it, aborts the stream gracefully, and **does** emit a terminal
envelope:

```json
{"type":"user","message":{"role":"user",
  "content":[{"type":"text","text":"[Request interrupted by user]"}]}, ...}
{"is_error":true,"terminal_reason":"aborted_streaming",
 "subtype":"error_during_execution","type":"result",
 "usage":{"input_tokens":0, ...}, "total_cost_usd":0.00062,
 "modelUsage":{"claude-haiku-4-5-20251001":{"inputTokens":555,
   "outputTokens":13,"costUSD":0.00062, ...}}, ...}
```

### Conclusion — fact-finding only, no promise attached

This is where E1's canceled-turn telemetry gap (`docs/gauntlet/notes/
mvp-bucketing-2026-08-11.md` MVP-1 row) actually lives: under the mechanism
sergeant uses today (SIGKILL), there is no cumulative usage to recover on
interrupt beyond whatever `assistant` chunks happened to stream first. A
SIGTERM-based interrupt would recover a real, structured usage/cost record
— but changing `interrupt`'s signal is a design decision with its own
tradeoffs (does the CLI's own cleanup work reliably under load? does
`"aborted_streaming"` need new outcome-mapping?) that this measurement does
not settle and this lane does not implement. Recorded in
`src/backend/claude.rs`'s module docs and `docs/environments/cerberus.md`.
