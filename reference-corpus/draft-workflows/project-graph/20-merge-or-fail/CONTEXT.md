# 20-merge-or-fail: merge or fail

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-resolve-output-path/output/README.md | L4 | upstream artifact produced by `00-resolve-output-path` |

## Purpose

All-or-nothing: any repo's extraction failure fails the run before merge.

Trigger (workflow-level): Architecture work needs whole-project structure, or the operator asks for a graph/refresh.

## What must become true here (durable outcome)

All-or-nothing: any repo's extraction failure fails the run before merge.

## Behavior contract

- **Building a project graph across many repos is all-or-nothing at the merge step: if extraction fails for any included repo, the whole run fails before attempting to merge or publish anything, rather than publishing a graph silently missing some repos.**
  (trigger: one or more repos fail extraction during a multi-repo graph build; outcome: a published project graph is always complete over every repo it claims to cover, never silently partial)
  — `BU-P6-091`, `reference/sergeant-upstream/bin/sgt-graphify` (L430-433)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

"We never publish a partial graph" outlives any particular merger implementation. Judged against §6.3's reimplementation test (N1 adjudication A4): swapping the merger's implementation tomorrow would leave this checkpoint's guarantee — no repo's extraction failure is ever silently absorbed into a partial publish — completely unchanged, because the guarantee *is* the checkpoint, not an artifact of how the merge happens to be coded. **Kept** as a stage.

## Helpers (folded per N1 adjudication A4)

`10-extract-per-repo` and `30-publish-atomically` are demoted (neither carried an Additional note argument — both were justified only by the §6.5 boilerplate) and fold into this stage, now the workflow's second and final stage, as helper invocations bracketing the all-or-nothing gate:

- **Extract per repo** (performed before the gate). Extracting a repo's graph data stages the repo into a filtered, non-source-repo scratch location before scanning whenever any output-exclusion pattern applies. When no supported LLM API key is configured, graph extraction runs in code-only mode rather than aborting the whole multi-repo run.
  — `BU-P6-089`, `BU-P6-090`, `BU-P7-088`, `reference/sergeant-upstream/bin/sgt-graphify` (L349-355, L356-359), `reference/sergeant-upstream/tests/sgt-graphify-test.sh` (line 660)
- **Publish atomically** (performed after the gate passes). Publishing a freshly built project knowledge graph is atomic via a symlink swap (`mv -T`): concurrent readers see either the complete old graph or the complete new graph, never a partial or torn state; if the atomic rename fails, the old symlink remains pointing at the previous, still-valid output.
  — `BU-P6-088`, `BU-P7-003`, `BU-P7-086`, `reference/sergeant-upstream/bin/sgt-graphify` (L7-10), `reference/sergeant-upstream/schema/project.yaml.example` (lines 90-92), `reference/sergeant-upstream/tests/sgt-graphify-test.sh` (lines 551-557)

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
