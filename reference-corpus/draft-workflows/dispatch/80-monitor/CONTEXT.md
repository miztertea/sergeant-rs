# 80-monitor: monitor

## Inputs

| File | Layer | Why |
|---|---|---|
| ../70-launch-and-record/output/README.md | L4 | upstream artifact produced by `70-launch-and-record` |

## Purpose

Escalations are read in full, human decisions obtained without inference, delivered to the exact task/repo pair.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Escalations are read in full, human decisions obtained without inference, delivered to the exact task/repo pair.

## Behavior contract

- **needs_input and blocked are distinct nonterminal states for a dispatched worker; a worker waiting on CI, review threads, or dependencies stays in_progress unless it actually needs to escalate.**
  (trigger: sgt-watch observes a dispatched worker; outcome: the operator sees a precise nonterminal state rather than an undifferentiated 'still running')
  — `BU-P5-066`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 92)
- **When a worker escalates, the coordinator must read its full context, evidence, exact question/blocker, recommendation, and options; obtain an explicit human decision without inferring consequential intent; and deliver that decision to the specific task/repo pair via sgt-respond, which durably writes the response to fleet state before notifying the worker.**
  (trigger: a dispatched worker escalates; outcome: the human decision is fully informed, explicit, durably recorded, and delivered to the correct worker)
  — `BU-P5-067`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 94-98)
- **After a response is delivered, the worker must consume/remove the response, clear its escalation message, log the decision to td, and return to in_progress before continuing.**
  (trigger: a response has been delivered to an escalated worker; outcome: the worker's escalation state is durably cleared and the decision is logged before work resumes)
  — `BU-P5-068`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 99)
- **A worker's exit boundary settles the accepted notification action lease for every possible terminal status alike — done, failed, drained, needs_input, blocked, waiting, and orphaned — because every one of those exit paths is an equally valid place for a lease to be silently left outstanding forever if it is not settled at that single, unified boundary.**
  (trigger: a worker process is exiting, regardless of outcome; outcome: a notification's action-lease fate is always known — finalized or explicitly pending with a reason — no matter which of the seven exit paths the worker took)
  — `BU-P6-113`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L483-489)
- **A worker's exit is orphaned unless it produced a genuinely terminal status with substantiating evidence: a done status requires a non-empty result or the worker is reclassified orphaned; any other unrecognized status falls back to orphaned by default, so an unclassified exit is never mistaken for success.**
  (trigger: a worker process is exiting; outcome: only a status with real substantiating evidence is ever accepted as a genuine terminal outcome; everything else defaults to the honest, investigable orphaned state)
  — `BU-P6-115`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L490-495, L509-510)
- **For the Claude harness specifically, whether a pinned model was actually honored — rather than silently substituted by the provider — is confirmed only after the run completes, by scanning the session transcript for a known substitution-warning phrase; this check never blocks or changes the mission's outcome, it only records a diagnostic that survives the otherwise-unconditional cleanup of that diagnostic on success.**
  (trigger: a Claude worker with a pinned model has finished its run; outcome: a silent model substitution that a mission still completed 'successfully' is caught and durably recorded, even though nothing about the run's own exit signals failure)
  — `BU-P6-116`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L1109-1117)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Delegation

This stage's outcome is produced by running **respond-to-worker** to its own completion (context composition today — see `docs/icm/convention.md` §4 on `@@name` versus true nested-workflow invocation, which does not exist yet).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
