# Validate and Ship (no-mistakes)
Draft workflow package — candidate **W18** `validate-and-ship` from the N1
manual reference-corpus decomposition (the dev-corpus provenance record (not shipped)).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`.sergeant/common/contexts/icm-policy.md` §1a rule 5).

## Purpose

The single final shipping boundary: validate a committed change through the pipeline to a terminal outcome, routing every finding, without the validating actor ever editing the code.

## Trigger

Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-check-scope` | actor-stage (§6.4, judgment) | The invocation mode (validate-only / task-first) is determined and any specific user request is translated into concrete pipeline flags. Directly-invoked entry only — restored per N1 adjudication A5. |
| `10-do-the-work` | actor-stage (§6.4, judgment) | In task-first mode, the described task is carried out and committed on a feature branch before validation begins. Directly-invoked entry only — restored per N1 adjudication A5. |
| `20-select-intent-transport` | actor-stage (§6.4, judgment) | (Coordinator-launched entry only) readiness verified, launch reservation acquired, isolated snapshot reserved and re-verified; then, for either entry, the transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run. Folds three demoted checkpoints (N1 adjudication A4) and this package's repo-release-verification re-homing (N1 adjudication A6). |
| `30-start-run` | actor-stage (§6.4, judgment) | A run exists on a feature branch with committed history, a verbatim intent, an initialized repo and a runnable pipeline agent; an in-flight matching run is reattached, never duplicated. Re-rung from a §6.5 boilerplate classification to actor-stage per N1 adjudication A5 — see the stage's own CONTEXT.md. |
| `40-drive-gates` | actor-stage (§6.4, judgment) | Every gate resolved by exactly one response; ask-user findings relayed verbatim and never resolved autonomously; the actor never edits the pipeline-owned worktree, aborts, or reruns to escape a gate; every actionable finding routed to a deduplicated owning-repo task. Folds a demoted checkpoint (N1 adjudication A4). |
| `50-reconcile-custody` | actor-stage (§6.4, judgment) | The structured branch-sync state is processed rather than improvised: sync / continue / recover-custody, never reset, stash, force or branch replacement. |
| `60-close-out` | actor-stage (§6.4, judgment) | Stop driving at `checks-passed`; on `failed`/`cancelled`, fix on the same branch and re-drive; summarize what the pipeline found and fixed; any coordinator ownership transfer during the run is durably logged. Folds a demoted checkpoint (N1 adjudication A4). |

`00-verify-readiness`, `10-acquire-launch-reservation`, and `20-reserve-isolated-snapshot` (original numbering) were demoted per N1 adjudication A4 and folded into `20-select-intent-transport`; `60-route-findings` was demoted and folded into `40-drive-gates`; `90-handover-log` was demoted and folded into `60-close-out`. See each surviving stage's own `CONTEXT.md` and the dev-corpus provenance record (not shipped)'s "Adjudication A4" section for the per-stage keep/demote reasoning. `00-check-scope` and `10-do-the-work` were restored (never dissolved) and `40-start-run` (original numbering, now `30-start-run`) was re-rung to an actor stage, per N1 adjudication A5 (finding N1-BH-04) — see that same file's "Adjudication A5" section. `repo-release-verification` was demoted from a standalone workflow and re-homed into `20-select-intent-transport` per N1 adjudication A6 — see that same file's "Re-homed from repo-release-verification (A6)" section.

## Relationships to other workflows

- `40-drive-gates` folds the finding-routing behavior formerly attributed to a package named `route-review-findings`, which was never built (retriaged to CLI-SURFACE/NET-NEW-SURFACE unbuilt verb candidates — dev-corpus retriage record, kept in this project's private development record). The actual live mechanism is `sgt-no-mistakes-finding`'s deterministic routing, already folded into `40-drive-gates` as its own helper. **Corrected 2026-08-16, ICM-R2 pilot review:** the prior text here named `route-review-findings` as a currently-invoked delegate; no such package exists.

## Authority envelope

This workflow receives an already-admitted Work intent (either the coordinator-launched entry's readiness-verified handoff, or the directly-invoked entry's live `/no-mistakes` request).

### Workflow may decide
- The invocation mode (validate-only vs. task-first) and how a specific user request translates into concrete pipeline flags (`00-check-scope`).
- How to authorize `auto-fix`/`no-op` gate findings on its own judgment (`40-drive-gates`).
- How to process a structured branch-sync state deterministically (`50-reconcile-custody`).

### Workflow may not decide
- Whether to resolve an `ask-user` gate finding — it is relayed verbatim to the user, never resolved autonomously (`40-drive-gates`).
- To edit the pipeline-owned worktree, abort, or rerun mid-gate to escape a finding.

### Resolved (issue #123): a dispatched run of this workflow pushing a branch, opening a PR, or triggering CI is not a gap
Push, PR-open, and CI-run are this workflow's ordinary, correct, ungated
behavior — they create the review artifact, they don't bypass review. The
sensitive action is merge, and merge is held structurally by this
repository's own GitHub configuration (`allow_auto_merge: false`, no
branch protection rule on `main`), not by anything in this package's
content. That is a repository setting outside this package's own
visibility, not something a stage here can verify at runtime — worth
re-checking directly if either setting ever changes, since this
workflow's own content has no way to detect that.

### Human or Captain gates
- Every `ask-user` gate finding.

### Decision record
Material decisions are recorded per-stage in each stage's own `## Bounded judgment` section below and in the gate's own findings table (`40-drive-gates`).

## Notes for reviewers

**U2 verdict** (the dev-corpus provenance record (not shipped) §1): the §6.3 reimplementation test *does* discriminate cleanly here, but only after the source's flat command list is split by outcome — the things that failed the test and became helpers, not stages, are the individual commands (`axi`, `axi status`, `axi logs`, `axi abort`, `axi sync --check`), the output grammar, the `--intent-file`/`--intent` flag choice, and the branch-sync decision table.

**Two entry variants, redesigned per N1 adjudication A5.** Coordinator-launched entry begins at `20-select-intent-transport` (its folded helpers cover the readiness-marker, launch-reservation, and isolated-snapshot preconditions that only apply to a coordinator handing off an already-reviewed worker commit). Directly-invoked entry (`/no-mistakes`, run by the actor in the current session) begins at `00-check-scope`, proceeds through `10-do-the-work` in task-first mode, and rejoins the shared pipeline at `20-select-intent-transport`. `00-check-scope`/`10-do-the-work` were previously dissolved into workflow-level citations and prose rather than materialized as stages, to dodge an id collision with what were then `10-acquire-launch-reservation`/`20-reserve-isolated-snapshot` — adjudication A5 (finding N1-BH-04) confirmed that was a violation (id collisions are renamed, never dissolved) and directed their restoration; renumbering the whole package (see the stage table above) both resolved the collision and gave the two entry variants a coherent single ordered list, per convention.md's single-linear-stage-list model (no engine-level branching exists at this milestone).

Per A11, the reader-note previously repeating "read `pane`/`tmux` as this project's durable session/execution identity" has been removed as redundant: the affected statements (both folded into `60-close-out`) are cited here already normalized to that reading in the corpus; see deviation register D2 and obsolete-mechanism clusters M1-M4 (the dev-corpus provenance record (not shipped) §4) for the underlying obsolescence.

## Provenance

See the dev-corpus provenance record (not shipped) for the complete stage-to-behavior-unit mapping and workflow-level citations. **Corrected 2026-08-16, ICM-R2 pilot review:** this and every other in-package citation previously pointed at a co-located `provenance.md` that does not exist anywhere in this package — apparently never carried over when the package was promoted out of its draft-workflow location.
