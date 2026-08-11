# 08-evaluate-and-resume-wait

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the wake-condition step is invoked for a waiting worker

**Outcome:** resumption happens only for the exact worker whose condition evaluates as met, tagged with a required generation

**Statement (the operative rule):** The wake-condition step evaluates the wake condition and resumes the exact waiting worker through the worker response-delivery step when the condition is met; every condition requires `generation=<int>`.

## What must become true here (durable outcome)

Resumption happens only for the exact worker whose condition evaluates as met, tagged with a required generation — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-0150`: A `github_check` wake condition requires both `run_id` and `check_name`, and resumes only when that exact check concludes `success`; `failure`, `cancelled`, `skipped`, `timed_out`, and every other non-success conclusion never resume the worker.
- `BU-0151`: A condition that can no longer be met converts the worker to `needs_input` with the remedy in `.sergeant-message` instead of retrying until its deadline — covering a named check that concluded unsuccessfully, a check absent from an already-completed run, an ambiguous duplicate check name, or a condition missing a required field.
- `BU-0152`: `human_response` conditions never auto-resume and always convert the worker to `needs_input` for a human response delivered through the worker response-delivery step; `deployment` remains a declared condition kind but also escalates to `needs_input` until an installation-specific deployment adapter is wired.
- `BU-0453`: The wake-condition step only evaluates a worker whose current status is exactly 'waiting'; any other status refuses evaluation.
- `BU-0454`: The wake-condition step refuses to proceed unless a wake-condition file exists in the worktree.
- `BU-0455`: Every field name in a wake-condition file is checked against a strict allowlist; any unknown field name causes the whole evaluation to be rejected outright.
- `BU-0456`: Even an allowlisted field name is rejected if it matches a secret-like pattern (token, password, key, secret, auth, credential, and variants), as a belt-and-suspenders check.
- `BU-0458`: Wake-condition field values (other than check_name) are restricted to a pattern that never admits a leading '-', because every value becomes an argument to gh(1) or the task tracker(1) and a dash-leading value would be read as an option rather than as data.
- `BU-0459`: check_name uses a wider allowlist admitting the punctuation real human-authored GitHub check names contain, but still excludes shell metacharacters, quotes, redirection, and control characters, and is used only as quoted argv/printf data, never eval'd or word-split.
- `BU-0460`: Both the kind and generation fields are required in a wake condition; either being absent is a hard failure before any condition is evaluated.
- `BU-0461`: The wake-condition step serializes itself with a per-(task,repo) lock directory; if the lock directory exists but its recorded owner pid is no longer alive, the wake-condition step reclaims it by removing and recreating it, rather than blocking forever.
- `BU-0462`: If a wake condition's deadline has already passed, the worker is moved to a terminal 'failed: deadline exceeded' state instead of being retried.
- `BU-0463`: If a wake condition's max_attempts has already been reached, the worker is moved to needs_input for manual intervention instead of being retried further.
- `BU-0468`: Every GitHub query a wake condition makes is explicitly bound to the worker's own recorded worktree/remote, never the scheduler's own current directory, because gh(1) otherwise silently resolves against the wrong repository.
- `BU-0469`: Resolving a worktree's origin remote to a GitHub owner/repo slug anchors the hostname match so that a host merely ending in 'github.com' (e.g. mygithub.com) is never mistaken for github.com and queried as the wrong repository.
- `BU-0470`: A github_check condition with a missing or non-numeric run_id, or a missing check_name, escalates immediately rather than being retried, because no amount of waiting can make a malformed condition well-formed.
- `BU-0471`: For github_check, zero matching checks in a still-running workflow run is treated as unmet (the check may not have started yet); zero matching checks in an already-completed run is escalated as permanently unsatisfiable, since the named check will never appear.
- `BU-0472`: For github_check, exactly one matching but still-incomplete check is unmet; a concluded 'success' is met; any other concluded conclusion escalates as permanently unsatisfiable rather than retried.
- `BU-0473`: For github_check, more than one job sharing the same check_name is escalated as ambiguous rather than guessed at.
- `BU-0474`: The deployment and human_response condition kinds are always treated as unsupported, and so is any unrecognized kind — none of these are auto-evaluated.
- `BU-0475`: An 'unsupported' evaluation result sets the worker to needs_input for manual response, and the wake-condition step exits non-zero.
- `BU-0476`: An 'escalate' evaluation result records the escalation reason to the fleet diagnostic and sets the worker to needs_input describing the condition as no longer satisfiable, rather than retrying it.
- `BU-0477`: An 'error' evaluation result (an adapter/transient failure) records the reason to the diagnostic but is retried with a recorded attempt and backoff, rather than escalated to needs_input.
- `BU-0478`: An 'unmet' evaluation result is retried with a recorded attempt and backoff exactly like an error result, but without writing a diagnostic.
- `BU-0479`: Once a condition is met, the wake-condition step resumes the worker by invoking the worker response-delivery step with a message describing the wake evidence piped to it as the response text.

## Deterministic machinery this stage uses

Helper-rung behaviors subordinate to this checkpoint's own outcome:

- `BU-0450`: The wake-condition step requires exactly two positional arguments, a task ID and a repo; any other count is rejected.
- `BU-0451`: Each of the task ID and repo arguments to the wake-condition step must match a restricted identifier pattern; either failing is rejected.
- `BU-0452`: The wake-condition step refuses to proceed if the named repo has no fleet-state directory, or if its recorded worktree does not exist.
- `BU-0457`: A field present in the wake-condition file with an empty value is rejected with a distinct error from an invalid-character value, so the operator is not misled into hunting for a bad character that is not there.
- `BU-0464`: Each unmet or errored evaluation attempt is recorded with a pseudorandom jitter (0-9 seconds) added to the configured base backoff, publishing the timestamp before which no subsequent attempt should occur.
- `BU-0465`: A not_before condition is met once the current time is at or after the recorded timestamp field; before that it is unmet.
- `BU-0466`: A fleet_dependency condition is met only when the dependency task/repo's own status file reads exactly 'done'; any other recorded status, including a failed or missing one, is unmet, never met.
- `BU-0467`: A td_dependency condition is met only when the task tracker reports one of done/closed/complete/completed; an empty status is treated as a distinct adapter error (not simply unmet), and any other value is unmet.

