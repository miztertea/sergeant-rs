# 60-render-brief: render brief

## Inputs

| File | Layer | Why |
|---|---|---|
| ../50-acquire-surface/output/README.md | L4 | upstream artifact produced by `50-acquire-surface` |

## Purpose

Mission, merged instructions, dependency notes, delivery requirements and any verbatim user override are durably carried to the worker before it starts.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Mission, merged instructions, dependency notes, delivery requirements and any verbatim user override are durably carried to the worker before it starts.

## Behavior contract

- **Each worker's starting context must durably carry its mission, the merged/resolved agent instructions, dependency notes, and delivery requirements, before the worker begins.**
  (trigger: a worktree has been created for a dispatched repository; outcome: the worker never begins without a complete, self-contained starting brief)
  — `BU-P5-062`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 82)
- **If a canonical skill referenced by the routing table cannot be loaded, the generated brief's own embedded rules for that phase remain mandatory regardless.**
  (trigger: a canonical skill named by the routing table is unavailable; outcome: the phase's requirements still apply even without the specialized skill loaded)
  — `BU-P5-087`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 190)
- **A worker brief's instruction merge order is defaults, then group, then repo, and an explicit user override embedded in the dispatched brief (e.g. 'run no-mistakes for this worker before completion') must appear verbatim in the rendered brief rather than being silently dropped by the default no-mistakes-ownership instruction it overrides.**
  (trigger: sgt-dispatch renders a worker brief from layered defaults/group/repo instructions plus any explicit per-dispatch override text; outcome: instruction layering (defaults -> group -> repo) and an explicit dispatch-time override both render deterministically and verifiably into the final brief text a worker receives)
  — `BU-P7-112`, `reference/sergeant-upstream/tests/sgt-dispatch-brief-test.sh` (lines 382, 501-511)
- **sgt-dispatch must resolve an OpenCode (`oc`) target session for routing coordinator notifications by consulting `td` for an existing routing task before creating one, so coordinator notification routing reuses existing tracked infrastructure rather than duplicating it per dispatch.**
  (trigger: sgt-dispatch dispatches a worker under an OpenCode coordinator session; outcome: coordinator-notification routing infrastructure (its own td task) is discovered and reused, not silently recreated on every dispatch)
  — `BU-P7-074`, `reference/sergeant-upstream/tests/sgt-dispatch-oc-target-test.sh` (lines 34-40)
- **sgt-dispatch's `td` integration must distinguish an existing tracked task from one that needs to be newly created for a cross-repo brief, and this contract must be exercised against a full copy of every sourced helper (not a hand-picked subset), because a missing helper makes the copy fail at its own source line instead of exercising the behavior under test.**
  (trigger: a cross-repo brief may or may not already have a tracked td task; outcome: dispatch correctly attaches to existing tracked work or creates new tracked work, never silently duplicating or losing the tracking relationship)
  — `BU-P7-075`, `reference/sergeant-upstream/tests/sgt-dispatch-td-test.sh` (lines 19-24)
- **The --deps ordering string only expresses that one repository must finish before dependents can merge; enforcing it is left entirely to the dispatched workers reading it out of their own brief.**
  (trigger: a dependency string is declared for a dispatch; outcome: dependency intent is documented, but nothing outside the worker's own judgment enforces it)
  — `BU-P5-074`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 155-157)
- **Every dispatched worker pins a fixed point -- normally the merge-base with current origin/main -- and records the base SHA, commit list, and diff scope before implementing anything.**
  (trigger: a worker's session starts; outcome: the worker's later diff and evidence are always measured against a recorded, immutable starting point)
  — `BU-P5-075`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 166)
- **Every dispatched worker triages the full originating td issue/spec/comments and linked material, prior or redundant work, category, and readiness, and explicitly records when no originating spec exists.**
  (trigger: a worker's session starts; outcome: the worker never begins implementation without having read and recorded its own originating context)
  — `BU-P5-076`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 167)
- **A worker establishes public behavioral seams from td/spec evidence before writing tests; if a consequential seam is undecided, it escalates needs_input rather than guessing.**
  (trigger: a worker is about to define testable seams; outcome: undecided consequential design points are escalated, never silently assumed)
  — `BU-P5-078`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 174)
- **A worker in needs_input or blocked writes an escalation message and notifies the coordinator; on response, it consumes/removes the response, clears the message, logs the decision to td, restores in_progress, and continues -- the durable requirement is that answering a blocked worker always durably restores forward progress, regardless of the underlying file-based transport.**
  (trigger: a worker needs input or is blocked; outcome: the escalation-to-resume cycle always ends with the worker durably back in progress)
  — `BU-P5-079`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 176)
- **No-mistakes is run only at an explicit final shipping boundary, never for routine worker completion, prototypes, investigations, documentation drafts, intermediate commits, or remediation loops, unless the user explicitly overrides that default; safety-sensitive work follows a stricter path.**
  (trigger: a worker reaches a candidate shipping boundary; outcome: the expensive shipping gate runs only where it is actually warranted)
  — `BU-P5-080`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 178)
- **Every no-mistakes finding is routed through sgt-no-mistakes-finding into a separate, deduplicated, owning-repo td task; correctness/security/data-integrity/test/ask-user findings are P1 and gated, warning debt is P2, informational debt is P3, and cosmetic/evidence noise is ignored -- and findings are never remediated inside the validation run itself.**
  (trigger: no-mistakes has produced findings; outcome: every actionable finding becomes tracked work at a severity-appropriate priority, and the validation run itself stays read-only with respect to remediation)
  — `BU-P5-081`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 179)
- **Independent review runs as separate parallel subagents for the axes named by a shared axis-vocabulary source (standards, spec, readiness, plus a conditional accessibility axis for UI-facing work identified by role/group/description language), each described by guidance reproduced verbatim in the generated brief; the spec axis is explicitly skipped when no spec exists.**
  (trigger: a worker reaches independent review; outcome: review axes, their guidance, and their applicability conditions come from one canonical source rather than being redefined per invocation)
  — `BU-P5-082`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 180-181)
- **When a finding fails to route after parsing, the router retains a sanitized findings artifact under a fixed path and names the exact retry command; the artifact is retried from, never re-generated by re-running the reviewer, and a retained artifact that has not been retried is never deleted.**
  (trigger: a review finding fails to route; outcome: evidence is preserved and addressable rather than silently dropped or expensively re-derived)
  — `BU-P5-083`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 182)
- **A stored finding-card revision the router cannot prove it authored is preserved below a superseded-revision separator and the card gains a needs-reconciliation label; only the worker owning the finding may merge the two accounts and remove that label -- the router itself never clears it.**
  (trigger: the router would otherwise overwrite a card revision it did not create; outcome: concurrent writers to the same card never silently clobber each other; reconciliation is an explicit, owned obligation)
  — `BU-P5-084`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 183)
- **Findings sharing the same originating run, head, owning module, and root cause share one serialized remediation worker/branch; before merging that group, native tests and independent re-reviews verifying mutation-before-validation, partial-publication/rollback, and identity/provenance are rerun; after two remediation cycles, fix dispatch stops and an architectural/root-cause review plus a human decision is required.**
  (trigger: multiple findings share a root cause; outcome: remediation is deduplicated, re-verified before merge, and escalates to a human after bounded retries rather than looping indefinitely)
  — `BU-P5-085`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 184)
- **A worker writes .sergeant-result and sets its status to done only after every completion gate has passed; a failed status with an exact reason is reserved specifically for an unrecoverable terminal failure.**
  (trigger: a worker reaches a terminal outcome; outcome: the terminal status distinguishes verified success from unrecoverable failure, and neither is asserted prematurely)
  — `BU-P5-086`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (line 188)
- **Recovering a stuck, stale, or orphaned dispatched worker is done only through sgt-respond or an equivalent explicit action; a worker is never marked done manually, and a retry writes both the result and the done status only after every completion gate passes.**
  (trigger: a worker appears stuck, stale, or orphaned; outcome: recovery never fabricates a successful terminal state; it either delivers an explicit response or nothing changes)
  — `BU-P5-089`, `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 221-229)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

The BU-P5-075..089 range is worker-contract content this stage *authors into the brief* but does not itself execute (synthesis.md §1: "kept in this cluster because dispatch owns writing it into the brief") — it is the input to `worker-mission` and `route-review-findings`, not a claim that this stage performs that content's behavior. BU-P5-074 records that `--deps` *ordering* is expressed here but its *enforcement* is left entirely to the dispatched workers reading their own brief (conflict X15, folded into engine-gap G2's split acceptance test in `reference-corpus/synthesis.md` §5) — recording is this stage's job; enforcement is not.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
