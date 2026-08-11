# 01-route-finding

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

**Trigger:** the validation pipeline surfaces an actionable finding

**Outcome:** the finding becomes owning-repo task tracker work rather than being fixed inline by the run

**Statement (the operative rule):** The validation pipeline run is validation-only and must not fix findings; actionable findings are routed into separate, deduplicated owning-repo task tracker tasks with the no-mistakes-finding disposition step.

## What must become true here (durable outcome)

The finding becomes owning-repo task tracker work rather than being fixed inline by the run — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0704`: If more than one existing task tracker task matches a finding's deduplication key, the no-mistakes-finding disposition step fails rather than picking one to update.
- `BU-0709`: A finding disposed gate causes the no-mistakes-finding disposition step to exit with status 2 after recording it in the task tracker, signaling to the caller that this finding remains blocking regardless of the task tracker side effect having succeeded.
- `BU-0710`: A finding disposed ask-user causes the no-mistakes-finding disposition step to exit with status 2 after recording it in the task tracker, signaling that the finding requires human escalation.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0090`: The required `--disposition` per finding is explicit: `gate` creates/updates P1 work and retains the gate, `ask-user` creates/updates P1 work and preserves human escalation, the task tracker creates/updates nonblocking actionable debt, and `ignore` records that no card is needed.
- `BU-0091`: Warning debt becomes P2, informational debt becomes P3, and repeated finding IDs update the same card while retaining the latest run ID, head SHA, location, description, and originating intent.
- `BU-0092`: Reruns preserve any existing repo-specific or manually added task tracker labels while ensuring the required validation pipeline and `finding` labels remain present without duplication.
- `BU-0093`: On rerun, visible active cards stay in their current state, while explicitly hidden states are resurfaced: closed cards are reopened and deferred cards are undeferred before the finding body is refreshed.
- `BU-0094`: Correctness, security, data-integrity, and test findings cannot be deferred or ignored; cosmetic and evidence-only findings never create cards.
- `BU-0693`: The no-mistakes-finding disposition step requires every finding field (run id, head SHA, finding id, severity, kind, description, intent, disposition) to be present, and fails naming the specific missing field.
- `BU-0694`: A finding's disposition must be exactly one of gate, the task tracker, ignore, or ask-user; any other value is rejected.
- `BU-0695`: A finding whose kind is correctness, security, data-integrity, or test may only be disposed as gate or ask-user; disposing such a finding to the task tracker or ignore is rejected outright.
- `BU-0696`: A finding whose kind is cosmetic or evidence never creates the task tracker card, regardless of the requested disposition, and the command succeeds reporting that no card was created.
- `BU-0697`: Only cosmetic or evidence findings may ever be disposed as ignore; for any other kind, requesting ignore is rejected — actionable findings must gate, ask the user, or route to the task tracker.
- `BU-0698`: A finding's task tracker priority is derived from its disposition and severity: gate or ask-user is always priority P1; a td-routed finding is P2 for warning severity or P3 for info/informational severity; any other severity routed to the task tracker is rejected.
- `BU-0699`: The no-mistakes-finding disposition step requires the named project's config file to exist and the yq, git, and the task tracker tools to be available before doing anything else.
- `BU-0700`: The target repository's filesystem path is resolved by looking up its name in the named project's configuration; an unrecognized repo name is rejected.
- `BU-0701`: The no-mistakes-finding disposition step refuses to act on a repo path that is not actually a cloned git repository.
- `BU-0702`: Each finding is given a stable deduplication key derived from the repo and finding id, embedded as a 'Deduplication key:' line in the task tracker card body, so the same finding can be recognized again on a later run.
- `BU-0703`: A finding lookup treats the task tracker list JSON result of null as 'no existing card' rather than an error, so the router proceeds to create a new task instead of failing.
- `BU-0705`: If a matching finding's task tracker card was closed, the no-mistakes-finding disposition step reopens it before updating it.
- `BU-0706`: If a matching finding's task tracker card had a deferral set, the no-mistakes-finding disposition step clears the deferral when the finding recurs, rather than leaving it silently deferred.
- `BU-0707`: An existing matching task tracker card has its priority, description (the full current finding body), and labels rewritten to reflect the current run's finding data.
- `BU-0708`: If no existing matching task tracker card is found, the no-mistakes-finding disposition step creates a new task tracker task carrying the full finding body, computed priority, and standard labels.

