# 20-30-parallel-review: parallel review (Standards + Spec)

## Inputs

| File | Layer | Why |
|---|---|---|
| ../10-identify-spec-source/output/README.md | L4 | upstream artifact produced by `10-identify-spec-source` |
| ../references/smell-baseline.md | L3 | the fixed Fowler-smell baseline pasted into the Standards sub-agent's prompt |

## Purpose

Both axes run as isolated sub-agents, spawned in a single message, and each produces its own report.

Trigger (workflow-level): A diff needs review before merge (invoked directly or delegated from `worker-mission`/`implement`).

## What must become true here (durable outcome)

Both axes run as isolated sub-agents, spawned in a single message, and each produces its own report — or the Spec sub-agent is explicitly skipped when stage 10 recorded no spec source.

## Behavior contract

- **The Standards and Spec reviews run as parallel, isolated sub-agents, spawned together in a single message using two parallel general-purpose Agent tool calls, so neither review's context pollutes the other; this stage aggregates neither — that is `40-aggregate`'s job.**
  (trigger: both review sources are identified; outcome: two independent, concurrently-dispatched review passes are produced)
- **A documented repository standard always overrides the smell baseline: where the repo's own standard endorses something the baseline would flag, the smell is suppressed.**
  (trigger: a baseline smell conflicts with a repo-documented standard; outcome: the repo's documented standard wins and the smell is suppressed)
- **Every baseline smell is a labelled judgment-call heuristic, never a hard violation, and the reviewer must skip anything tooling already enforces.**
  (trigger: applying the smell baseline; outcome: smells are reported as judgment calls, not hard failures; tooling-enforced items are not repeated)
- **The Standards sub-agent's prompt must include the diff command and commit list, the located standards-source files plus the full smell baseline (`../references/smell-baseline.md`) pasted in — the sub-agent has no other access to it — and a brief asking it to report per-file/hunk hard standard violations (cited to the standard) and baseline smells (named and quoted), distinguishing hard violations from judgment calls, skipping tooling-enforced items, under 400 words.**
  (trigger: spawning the Standards sub-agent; outcome: the sub-agent has everything needed to produce a bounded, well-formed Standards report)
- **The Spec sub-agent's prompt must include the diff command and commit list, the path or fetched contents of the spec, and a brief asking it to report missing/partial requirements, scope creep (unasked-for behavior), and requirements that look implemented but wrong, quoting the spec line for each finding, under 400 words.**
  (trigger: spawning the Spec sub-agent; outcome: the sub-agent has everything needed to produce a bounded, well-formed Spec report)
- **If no spec is found, the Spec sub-agent is skipped entirely and the final report notes this explicitly.**
  (trigger: no spec source was located in Step 2; outcome: the Spec axis is honestly reported as unavailable rather than fabricated)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Assembling the Standards sub-agent prompt: diff command, commit list, located standards-source files, and the full smell baseline from `../references/smell-baseline.md`.
- Assembling the Spec sub-agent prompt: diff command, commit list, and the located spec source contents.
- Judging which baseline smells are worth reporting as judgment calls versus which are already tooling-enforced.

### J1 — local choices allowed
- Exact sub-agent invocation wording, so long as both required prompts (Standards, and Spec when not skipped) are dispatched in the same message.

### J0 — must become `needs_input`
- The standards-source discovery step itself fails to locate the repo's documentation directory structure (not merely "no standards documented," which the smell baseline already covers, but genuine inability to search the repo): stop and ask rather than inventing a standards source.

### Completion boundary
This stage may complete only when the Standards sub-agent has reported, and the Spec sub-agent has either reported or been explicitly skipped per stage 10's recorded "no spec" answer (J4).

### Decision evidence
Write both sub-agent reports verbatim to `output/README.md`, tagged by axis; note explicitly if the Spec axis was skipped and why.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
