# 30-publish-atomically: publish atomically

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-merge-or-fail/output/README.md | L4 | upstream artifact produced by `20-merge-or-fail` |

## Purpose

Readers see the complete old or complete new graph, never a torn state; a failed swap leaves the previous output valid.

Trigger (workflow-level): Architecture work needs whole-project structure, or the operator asks for a graph/refresh.

## What must become true here (durable outcome)

Readers see the complete old or complete new graph, never a torn state; a failed swap leaves the previous output valid.

## Behavior contract

- **Publishing a freshly built project knowledge graph is atomic: concurrent readers see either the complete old graph or the complete new graph throughout the whole run, never a partial or missing state, regardless of whether the configured output is a plain directory or a symlink pointing inside or outside a source repo.**
  (trigger: a project's knowledge graph is rebuilt; outcome: the published graph is always internally consistent and complete at every point in time, never observed half-written)
  — `BU-P6-088`, `reference/sergeant-upstream/bin/sgt-graphify` (L7-10)
- **A published cross-repo knowledge graph is replaced only after a graphify run completes in full, and the output directory may sit inside a source repo without its own artifacts being re-ingested as source.**
  (trigger: sgt-graphify runs for a project with a graphify block; outcome: readers of the published graph output never observe a partial or torn merge)
  — `BU-P7-003`, `reference/sergeant-upstream/schema/project.yaml.example` (lines 90-92)
- **Publishing a merged project graph must be atomic via a symlink swap (`mv -T`): if that atomic rename fails, the old symlink must remain pointing at the previous, still-valid output rather than being left dangling or partially updated.**
  (trigger: sgt-graphify finishes merging a new project graph and publishes it; outcome: consumers of the published graph output link never observe a dangling or half-updated symlink, even when the final atomic swap itself fails)
  — `BU-P7-086`, `reference/sergeant-upstream/tests/sgt-graphify-test.sh` (lines 551-557)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
