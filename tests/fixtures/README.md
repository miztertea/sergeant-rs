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
