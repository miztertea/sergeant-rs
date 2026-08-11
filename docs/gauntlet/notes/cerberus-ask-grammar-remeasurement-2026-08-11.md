# Cerberus / CLI 2.1.227 — the ask grammar is absent; a5 measured red

Governing: `docs/gauntlet/notes/n3-claude-ask-measurement.md` (the 2.1.226
measurement and its "re-measure when" clause), LESSONS L1/L8,
`docs/environments/cerberus.md`.

**Verdict: on this host, Claude Code 2.1.227 emits no `post_turn_summary`
line at all — the ask affordance GP-2 rides on is not present. The adapter's
runtime withdrawal rule (INV-N3-06's fix) is the operative behavior;
`Capabilities::ask` lowers on the first completed turn, as designed.**

## What was measured, and how

First act per CLAUDE.md's version-bump rule: the opt-in suite
(`SERGEANT_CLAUDE_TESTS=1 cargo test --test m4_backends -- --ignored`),
2026-08-11, this host. Result: a1 ok, a3 ok, **a5 FAILED** — the
question-ending turn derived `StageCompleted { summary: Some("Which
database …") }` instead of `NeedsInput`, exactly the pre-N3 shape the
withdrawal rule anticipated.

Then the n3 note's own two-prompt protocol, extended to isolate variables —
five token-spending turns (~$0.15 total), all in a scratch dir outside the
repo, all completing with `is_error:false`, `terminal_reason:"completed"`:

| Probe | Model | Permission mode | Tools used | `post_turn_summary` lines |
|---|---|---|---|---|
| A (ask-and-stop) | haiku-4.5 | plan | no | **0** |
| B (say DONE, no ask) | haiku-4.5 | plan | no | **0** |
| A | haiku-4.5 | (CLI default) | no | **0** |
| A + file write first | haiku-4.5 | acceptEdits | yes (Write) | **0** |
| A | sonnet-5 | plan | no | **0** |

`task_summary` is also absent. New stream lines observed on 2.1.227 that
2.1.226's measurement did not record: `rate_limit_event` (top-level type)
and `system/thinking_tokens`. `claude --help` says nothing about any of
these (token-free surface scan, same as 2.1.226's).

## The confounder, stated honestly

Run B attempt 2 (`docs/gauntlet/runs/runB/attempt-2/`, the cloud container,
**the same CLI version 2.1.227**, ~03:44Z the same day) DID receive one
`post_turn_summary` (`status_category:"review_ready"`, `needs_action:""`).
Differences between that environment and this one: auth method
(`oauth_token`/firstParty there, `claude.ai` here — measured via
`claude auth status --json` both times), root vs uid 1001, and hours of
wall clock. The emission is therefore **conditional on something outside
the version string** — most plausibly server-side/account-side gating.
Which variable controls it was not isolated (would require the other
account/host); what is certain is that the capability cannot be assumed
from the version, which is L1's whole point.

## Dispositions

1. **The adapter needs no behavioral fix for this** — the runtime
   withdrawal path (one completed turn without the line lowers
   `ask_grammar_intact`, emits `conversation.turn.grammar_unmeasured`) is
   pinned token-free by m4's n27 and behaves correctly. The designed
   degradation stands: workflows declaring ask-needing stages fail
   preflight honestly.
2. **a5 is asserting an environment fact as a universal** — the affordance
   is present-or-absent per host/account, not per version. Disposition per
   the testing rules' probe-gate doctrine: a5 should discriminate — if the
   driven turn's transcript carries the line, assert the NeedsInput
   mapping (the 2.1.226 contract); if it carries none, assert the
   withdrawal fired (capability lowered + `grammar_unmeasured` journaled)
   and skip the mapping assertion loudly (`SKIPPED-ENV`). That is a code
   change → folded into the #46/#47 adapter-seam loop (R-S0-12).
3. **Run B re-run (#19) watch item changes:** GP-2's ask pathway cannot
   fire live on this host today; the re-run instead watches that the
   withdrawal path fires live (`grammar_unmeasured` in the journal, ask
   capability reported false thereafter).
4. `docs/environments/cerberus.md` gains the fact row; the n3 note's
   "re-measure when" clause fired and this file is its record.
