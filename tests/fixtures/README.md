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
