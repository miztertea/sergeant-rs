# 10-evaluate: validate condition, evaluate, classify outcome, then resume

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage only) |

## Purpose

One of six typed condition kinds is evaluated; external checks bind to the worker's own recorded remote. This is the workflow's sole surviving stage (N1 adjudication A4): the other three extracted stages — validating the condition's field/value allowlist, classifying the evaluation outcome, and resuming the worker — carried no argument beyond §6.5's boilerplate and fold in here as ordered helper invocations. This stage itself is kept for a different reason than "judgment": it is the direct source of engine-gap **G1** (survives, per `reference-corpus/synthesis.md` §5 and `reference-corpus/engine-pressure.md`) — the *scheduling* of this stage's re-evaluation, periodic and without a live process burning a billed turn, is exactly what no lower rung can own. That is a categorical engine-gap argument, not a §6.3 implementation-swap argument, and it survives the case-by-case review untouched.

Trigger (workflow-level): A worker is in the `waiting` state with a recorded wake condition.

## What must become true here (durable outcome)

A strict field/value allowlist is enforced before evaluation; one of six typed condition kinds is then evaluated, with external checks bound to the worker's own recorded remote; the outcome is classified met / unmet / permanently-unsatisfiable→escalate / deadline→failed; and the worker is resumed on a met outcome.

## Behavior contract

- **Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored.**
  (trigger: a waiting worker's condition is due for evaluation; outcome: every wake condition either resolves to met, unmet, an adapter error, permanently unsatisfiable (escalate), or explicitly unsupported — never silently stuck)
  — `BU-P6-096`, `reference/sergeant-upstream/bin/sgt-wake` (L9-16)
- **Evaluating an external GitHub check status always binds the query to the worker's own recorded worktree's remote, never to whatever repository the scheduler process happens to be running from, because the scheduler is normally invoked from somewhere other than the worker's own repository.**
  (trigger: a github_check wake condition is evaluated; outcome: a resolution failure is never confused with a genuinely still-pending check)
  — `BU-P6-100`, `reference/sergeant-upstream/bin/sgt-wake` (L284-291)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

This is the direct source of engine-gap **G1** (survives): the *scheduling* of this stage — periodic re-evaluation without a live process burning a billed turn — is exactly what no lower rung can own. See `reference-corpus/synthesis.md` §5. N1 adjudication A4 reviewed this note against §6.3's reimplementation test: it is not an implementation-swap argument at all (swapping the evaluation logic would indeed leave the checkpoint unchanged) — it is an argument that the engine itself lacks a rung for periodic, processless re-evaluation, which is a different and stronger claim than the test asks about. The stage is kept on that basis.

## Helper invocations (folded stages, N1 adjudication A4)

Three stages extracted as their own candidates (ladder §6.5, "deterministic-machinery candidate") carried no "Additional note" and fold in here as ordered helper invocations: validating the condition runs first; classifying the outcome and resuming the worker run after evaluation.

**1. validate condition** (formerly `00-validate-condition`) — a strict field/value allowlist is enforced — no dash-leading values, secret-shaped names screened — before evaluation.

- **A wake condition's field names and value characters are both drawn from a strict allowlist — no field outside a fixed vocabulary is accepted, no value may begin with a dash (so it can never be misread as a flag by gh/td), and every field is additionally screened for secret-shaped names — before the condition is ever evaluated.**
  (trigger: a wake condition file is read for evaluation; outcome: a wake condition can never smuggle an unexpected flag or a secret into a downstream gh/td invocation)
  — `BU-P6-097`, `reference/sergeant-upstream/bin/sgt-wake` (L23-32, L69-74)
- **A wake-condition file may only contain the allowlisted field names and alphanumeric-safe values for its declared kind; it must never be used to persist arbitrary shell commands, prompt bodies, response text, tokens, or secrets.**
  (trigger: a worker writes a wake condition file; outcome: the wake-condition file cannot become an injection vector or an accidental secret-storage location, because only a narrow allowlisted schema is accepted)
  — `BU-P7-098`, `reference/sergeant-upstream/templates/worker-brief.md` (section '### 4. Escalate and resume', wake-condition paragraph)

**2. classify outcome** (formerly `20-classify-outcome`) — outcome is classified met / unmet / permanently-unsatisfiable→escalate / deadline→failed.

- **A wake condition distinguishes 'unmet' (may still become true on a later attempt) from 'escalate' (has become permanently unsatisfiable, so continuing to retry would be dishonest and wasteful) — for example a GitHub check that has already concluded with a non-success outcome can never become success, so it escalates rather than being retried until the attempt budget or deadline runs out.**
  (trigger: a wake condition is evaluated and found not yet satisfied; outcome: a permanently-unsatisfiable condition surfaces to the operator immediately rather than silently exhausting its retry budget first)
  — `BU-P6-098`, `reference/sergeant-upstream/bin/sgt-wake` (L268-274, L486-491)
- **A wake condition past its optional `deadline` must transition the worker to a failed status with a recorded reason (and never call sgt-respond to resume it), rather than continuing to wait past a caller-specified bound.**
  (trigger: sgt-wake evaluates a wake condition whose deadline has already passed; outcome: waiting is never truly unbounded — an expired deadline converts an indefinitely stuck wait into an explicit, terminal failure rather than an eternal wait or a spurious resume)
  — `BU-P7-097`, `reference/sergeant-upstream/tests/sgt-wake-test.sh` (lines 401-402)

**3. resume** (formerly `30-resume`) — the worker is resumed on a met outcome.

- **Evaluating whether a waiting worker's durable wake condition is satisfied recognizes a fixed set of condition kinds — a timestamp, a GitHub check, a sibling fleet task's completion, a tracked-work task's completion, a deployment (unsupported today), or an explicit human response (never auto-evaluable) — and every kind not recognized is treated as needing human input rather than silently ignored.**
  (trigger: a waiting worker's condition is due for evaluation; outcome: every wake condition either resolves to met, unmet, an adapter error, permanently unsatisfiable (escalate), or explicitly unsupported — never silently stuck)
  — `BU-P6-096`, `reference/sergeant-upstream/bin/sgt-wake` (L9-16)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
