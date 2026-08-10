# 60-notify-coordinator: notify coordinator

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-archive-evidence/output/README.md | L4 | upstream artifact produced by `50-archive-evidence` |

## Purpose

The update is classified into exactly one durable event kind and recorded; live transports are optional on top.

Trigger (workflow-level): A worker has published an escalation and a human decision exists.

## What must become true here (durable outcome)

The update is classified into exactly one durable event kind and recorded; live transports are optional on top.

## Behavior contract

- **A worker's free-text update message is classified into exactly one durable event kind — completion (done*/failed*), escalation (needs_input*/blocked*), or a generic update — purely by matching the message's leading token, and that classification, not the raw text, is what becomes the durable record.**
  (trigger: a worker reports its status via sgt-notify; outcome: every notification is durably typed as completion, escalation, or update, independent of the message wording beyond its prefix)
  — `BU-P6-027`, `reference/sergeant-upstream/bin/sgt-notify` (L31-36)
- **A worker completion or escalation notification is also written as a durable wiki activity entry distinguishing the completion/escalation heading and, when present, extracting and linking any GitHub PR URL mentioned in the message.**
  (trigger: a worker update is being recorded; outcome: a durable, cross-referenced activity trail exists for every worker update independent of live delivery)
  — `BU-P6-030`, `reference/sergeant-upstream/bin/sgt-notify` (L111-124)
- **A worker's escalation notification is delivered as a durable, mode-600 marker file tagged `event=escalation`, and never exposes the message body in that marker; it is separately mirrored into the wiki activity log under a distinct 'Agent Escalation' label so a nonterminal escalation is never mislabeled as a completion.**
  (trigger: a worker publishes a needs_input escalation via sgt-notify; outcome: notification delivery is durable and private (secrets/message text never sit in a world-readable marker) while still being observable via a separate labeled activity trail)
  — `BU-P7-047`, `reference/sergeant-upstream/tests/sgt-notify-test.sh` (lines 30-44)
- **A `done:`-prefixed notification is classified and logged as an 'Agent Completion' event distinct from an escalation, and direct terminal-injection delivery is available only as an explicit backward-compatibility transport, never the default.**
  (trigger: sgt-notify is called with a done:-prefixed or explicit message; outcome: terminal and nonterminal notifications are classified differently by construction (message prefix), and the legacy tmux-injection transport is opt-in only)
  — `BU-P7-048`, `reference/sergeant-upstream/tests/sgt-notify-test.sh` (line 55)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
