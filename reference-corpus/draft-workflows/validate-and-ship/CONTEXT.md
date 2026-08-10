# Validate and Ship (no-mistakes)
Draft workflow package — candidate **W18** `validate-and-ship` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`),
decomposed from `reference/sergeant-upstream` per
`reference-corpus/synthesis.md` §1. This is Layer 1 orientation only —
it is never delivered as a stage's instructions; each stage's own
`CONTEXT.md` (Layer 2) is the actor's contract (`docs/icm/convention.md`
§1a rule 5).

## Purpose

The single final shipping boundary: validate a committed change through the pipeline to a terminal outcome, routing every finding, without the validating actor ever editing the code.

## Trigger

Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-verify-readiness` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A published readiness marker asserts the exact intent revision, the exact reviewed head, and an explicit pass on every review axis; any mismatch refuses with its own reason. |
| `10-acquire-launch-reservation` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | An identity-checked reservation for the exact task/repo pair; concurrent attempts fail closed until the owner exits or stale ownership is proven. |
| `20-reserve-isolated-snapshot` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Validation runs against an isolated snapshot pinned at the reviewed commit with a clean tree, re-verified immediately before invocation. |
| `30-select-intent-transport` | actor-stage (§6.4, judgment) | The transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run. |
| `40-start-run` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | A run exists on a feature branch with committed history, a verbatim intent, an initialized repo and a runnable pipeline agent; an in-flight matching run is reattached, never duplicated. |
| `50-drive-gates` | actor-stage (§6.4, judgment) | Every gate resolved by exactly one response; ask-user findings relayed verbatim and never resolved autonomously; the actor never edits the pipeline-owned worktree, aborts, or reruns to escape a gate. |
| `60-route-findings` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Every actionable finding becomes one deduplicated owning-repo task with a deterministic severity→priority mapping; correctness/security/data-integrity/test findings can never be deferred or ignored; no finding is fixed inside the run. |
| `70-reconcile-custody` | actor-stage (§6.4, judgment) | The structured branch-sync state is processed rather than improvised: sync / continue / recover-custody, never reset, stash, force or branch replacement. |
| `80-close-out` | actor-stage (§6.4, judgment) | Stop driving at `checks-passed`; on `failed`/`cancelled`, fix on the same branch and re-drive; summarize what the pipeline found and fixed. |
| `90-handover-log` | stage (§6.3, deterministic-machinery candidate — see stage CONTEXT.md) | Every ownership transfer is appended to an owner-only log; release tokens are single-use. |

## Relationships to other workflows

- `60-route-findings` delegates to **route-review-findings**.

## Notes for reviewers

**U2 verdict** (`reference-corpus/synthesis.md` §1): the §6.3 reimplementation test *does* discriminate cleanly here, but only after the source's flat command list is split by outcome — the things that failed the test and became helpers, not stages, are the individual commands (`axi`, `axi status`, `axi logs`, `axi abort`, `axi sync --check`; BU-P2-101), the output grammar (BU-P2-102), the `--intent-file`/`--intent` flag choice (BU-P1-071), and the branch-sync decision table (BU-P1-078). Two entry variants share this stage list: coordinator-launched (starts at `00`) and directly-invoked (`/no-mistakes`, starts at `40`, with `10-check-scope`/`20-do-the-work` from BU-P2-058/059/060/061 preceding it in task-first mode — not materialized as separate stage directories here to avoid an id collision with `10-acquire-launch-reservation`/`20-reserve-isolated-snapshot`; recorded as a documented alternate entry point, not a distinct workflow).

**Reading `pane`/`tmux` in cited statements.** The following citations in this package's behavior contracts describe identity, liveness, or ownership checks in terms of old Sergeant's tmux pane: `BU-P7-104`, `BU-P8-089`. Per obsolete-mechanism clusters M1-M4 (`reference-corpus/synthesis.md` §4) and deviation register D2, this project structurally replaced the pane with headless per-turn processes owned by the daemon and a durable session/execution identity in the journal — there is no tmux pane in this architecture. Read every 'pane identity' / 'pane liveness' / 'pane recycling' phrase in those citations as **the durable execution or session identity this project already journals**, not as an instruction to introduce tmux. The policy (verify identity before acting, never infer liveness from a UI artifact, settle a lease before terminating) is durable; the pane is not.

## Provenance

See `provenance.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
