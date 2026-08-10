# N3 / GP-2 — what Claude Code 2.1.226 offers for an actor-authored ask

Governing: `docs/gauntlet/contracts/N3.md` Outcome 4; N2's grammar-pressure
report (GP-2 is the sole confirmed engine gap); LESSONS L1 ("measure the
Claude CLI, never trust its docs or its exit codes") and L8 ("a capability
flag is a claim: every advertised verb needs a contract test").

**Verdict: a clean mapping exists.** `Capabilities::ask` is `true` for the
Claude adapter, on the strength of the records below and nothing else.

## What was measured, and how

Installed CLI: `claude --version` → `2.1.226 (Claude Code)`, the version
`MIN_TRUSTED_VERSION` already pins. `claude auth status --json` →
`{"loggedIn": true, "authMethod": "oauth_token", "apiProvider": "firstParty"}`.

Two token-spending turns, model `claude-haiku-4-5-20251001`, in a scratch
directory outside the repo, in the adapter's own launch grammar minus the
permission flag (this container runs as root, and `--dangerously-skip-
permissions` refuses root without `IS_SANDBOX=1`; `--permission-mode plan`
was substituted, which changes nothing about the stream shape under test):

```sh
printf '<prompt>' | claude -p --verbose --output-format stream-json \
  --setting-sources user --model claude-haiku-4-5-20251001 \
  --permission-mode plan --session-id "$(uuidgen)"
```

Prompt A ("ask me which database and stop"), prompt B ("reply with exactly
the word DONE, do not ask anything"). Total cost of the measurement: two
haiku turns.

### Token-free surface scan, first

`claude --help` on 2.1.226 has **no** `--permission-prompt-tool`, and no flag
that turns an end-of-turn question into a blocking control request. The
affordances that looked adjacent and were rejected:

| affordance | why it is not the ask |
|---|---|
| `--input-format stream-json` | realtime *input* transport. Would let a turn be fed more user messages, but says nothing about the actor wanting one. Also incompatible with this adapter's one-process-per-turn model (D2). |
| `--brief` (SendUserMessage) | agent→user *notification*, one-way. Not a park. |
| `--permission-mode manual` | a permission gate — the adapter asking, not the actor. Would have produced `AskAuthor::Adapter` at best. |
| `AskUserQuestion`-style tooling | not present in the `init` tool list for a print-mode session on this build. |

### The record that does discriminate

Both turns emitted exactly one `system` line of subtype
`post_turn_summary`, immediately before the `result` envelope.

Prompt A (the actor asked):

```json
{"type":"system","subtype":"post_turn_summary",
 "summarizes_uuid":"bcafcd5d-a605-4731-ad9b-92047b2555c5",
 "status_category":"blocked",
 "status_detail":"Which database should I target: **postgres** or **sqlite**?",
 "needs_action":"Which database should I target: **postgres** or **sqlite**?",
 "uuid":"4d76df71-43ce-464d-a760-11134c7c2b3e",
 "session_id":"23e1c408-1d76-43bf-b7dd-61d28c089756"}
```

Prompt B (the actor did not ask):

```json
{"type":"system","subtype":"post_turn_summary",
 "summarizes_uuid":"53cc350f-fd48-4b19-84cb-2a66aaef022d",
 "status_category":"review_ready",
 "status_detail":"user requested confirmation",
 "needs_action":"",
 "uuid":"60c3de75-2f9f-4b68-b79c-4c8ad2244060",
 "session_id":"6002ad7e-3b30-4639-b871-0fb881f64961"}
```

Both turns' `result` envelopes were `is_error:false`, `stop_reason:"end_turn"`,
`terminal_reason:"completed"` — i.e. **the envelope alone cannot tell the two
apart**, and the adapter's pre-N3 behaviour was therefore to complete the
stage in both cases, carrying the unanswered question as the completion
summary. That is GP-2, reproduced.

## The rule the adapter implements

A non-empty `needs_action` string is an actor-authored question; nothing else
is (`ClaudeBackend`'s `actor_question`). Deliberately narrower than the
evidence permits:

- `status_category` is **not** part of the decision. Two values were observed
  (`blocked`, `review_ready`) and the vocabulary is otherwise unmeasured; L1
  says an unmeasured field is not evidence. It travels into the observation's
  `evidence` string, where a human reads it.
- The asymmetry decides the direction of the narrow rule. Parking a stage that
  did not need parking costs one `respond`. Completing a stage whose actor was
  waiting is the silent invention GP-2 was filed about. So the branch that
  fails closed is the one that parks.
- Model-pin substitution still wins over an ask: a question produced by a
  model the work never authorized is not a checkpoint to put a human in front
  of.

## Where the claim is pinned

- `src/backend/claude.rs` unit tests: both records above, verbatim, plus the
  empty/whitespace/missing cases and the pin-substitution precedence.
- `tests/m4_backends.rs` `a5_real_claude_reports_an_actor_authored_question_as_needs_input`
  — opt-in (`SERGEANT_CLAUDE_TESTS=1 … -- --ignored`), drives one real haiku
  turn through the adapter and fails loudly, naming this file, if 2.1.226 (or
  a later build) stops mapping an end-of-turn question this way. Its panic
  message says what to do: re-measure, and lower the flag rather than guess.

## Re-measure when

Any CLI version bump (the standing rule in CLAUDE.md), and specifically if
`post_turn_summary` changes subtype, drops `needs_action`, or starts emitting
`needs_action` for turns that asked nothing — the last of which would make
the adapter park stages that should have completed.
