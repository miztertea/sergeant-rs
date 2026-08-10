# 00-validate-condition: validate condition

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

A strict field/value allowlist is enforced — no dash-leading values, secret-shaped names screened — before evaluation.

Trigger (workflow-level): A worker is in the `waiting` state with a recorded wake condition.

## What must become true here (durable outcome)

A strict field/value allowlist is enforced — no dash-leading values, secret-shaped names screened — before evaluation.

## Behavior contract

- **A wake condition's field names and value characters are both drawn from a strict allowlist — no field outside a fixed vocabulary is accepted, no value may begin with a dash (so it can never be misread as a flag by gh/td), and every field is additionally screened for secret-shaped names — before the condition is ever evaluated.**
  (trigger: a wake condition file is read for evaluation; outcome: a wake condition can never smuggle an unexpected flag or a secret into a downstream gh/td invocation)
  — `BU-P6-097`, `reference/sergeant-upstream/bin/sgt-wake` (L23-32, L69-74)
- **A wake-condition file may only contain the allowlisted field names and alphanumeric-safe values for its declared kind; it must never be used to persist arbitrary shell commands, prompt bodies, response text, tokens, or secrets.**
  (trigger: a worker writes a wake condition file; outcome: the wake-condition file cannot become an injection vector or an accidental secret-storage location, because only a narrow allowlisted schema is accepted)
  — `BU-P7-098`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume', wake-condition paragraph)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
