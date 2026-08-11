# 08-quick-override

## Inputs

| File | Layer | Why |
|---|---|---|
| ../_config/workflow-level-helpers.md | L3 | deterministic machinery that applies throughout this workflow, not just this stage |

## Purpose

**Trigger:** the maintainer gives a direct state-change instruction

**Outcome:** the named state is applied directly, bypassing the ordinary multi-step triage procedure

**Statement (the operative rule):** When the maintainer directly names a target state for an item (e.g. asks to move it to `ready-for-agent`), the triage skill trusts that instruction and applies the state directly rather than re-running the full recommend/verify/grill procedure.

## What must become true here (durable outcome)

The named state is applied directly, bypassing the ordinary multi-step triage procedure — per the Statement above, which is the operative rule this stage exists to enforce.

## Guidance (synthesis-time stage-context attachments)

Judgment calls an actor executing this stage must apply (promoted from `stage-context` classification records attached to this checkpoint, per `.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson`):

- `BU-1169`: Even in the quick-override path, the triage skill confirms the specific action it is about to take (role change, comment, close) with the maintainer before acting.
- `BU-1170`: The quick state override path skips grilling entirely.
- `BU-1171`: If the quick override moves an item to `ready-for-agent` without a grilling session having been run, the triage skill asks the maintainer whether they want an agent brief written.

