# Provenance — Implement

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W23** `implement`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-050` | The implement skill implements a piece of work described by the user against a spec or set of tickets. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (front matter description, lines 3-3) |
| `BU-P2-051` | The implement skill disables automatic model-driven invocation (`disable-model-invocation: true`): it must be explicitly invoked by the user or coordinator, not triggered implicitly by the model recognizing the situation. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (front matter policy, lines 4-4) |
| `BU-P2-052` | The implement workflow should use the tdd workflow where possible, at seams pre-agreed for testing. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 9-9) |
| `BU-P2-054` | Once implementation is done, the code-review skill/workflow is used to review the work. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 13-13) |
| `BU-P3-004` | Cross-harness metadata mirrors the Claude-Code-specific disable-model-invocation flag so a non-Claude-Code harness (OpenCode) enforces the same explicit-invocation-only rule. | `reference/sergeant-upstream/.agents/skills/grill-with-docs/agents/openai.yaml` (policy.allow_implicit_invocation) |

## Stages

### `10-implement-with-tdd`

No directly-cited units against the stage's own judgment-bearing outcome (delegated or structural — see the stage's own CONTEXT.md). Folds the demoted `20-verify` checkpoint as a helper (see Adjudication A4 below):

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-053` | During implementation, typechecking and single test files should be run regularly, with the full test suite run once at the end. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 11-11) |

### `30-review`

No directly-cited units against the stage's own judgment-bearing outcome (delegated or structural — see the stage's own CONTEXT.md). Folds the demoted `40-commit` checkpoint as a helper (see Adjudication A4 below):

| Unit | Statement | Source |
|---|---|---|
| `BU-P2-055` | The final step of implement is to commit the work to the current branch. | `reference/sergeant-upstream/.agents/skills/implement/SKILL.md` (body, lines 15-15) |

## Adjudication A4

N1 adjudication A4 (finding N1-BH-02, `reference-corpus/adjudication-round1.md`): every stage whose `CONTEXT.md` justification was only the §6.5 deterministic-machinery boilerplate is demoted by default, folded into the adjacent judgment-bearing stage as a helper invocation.

- **`20-verify` — DEMOTED.** Classified at extraction as deterministic machinery (ladder §6.5) with no "Additional note" checkpoint argument; fails the reimplementation test as an independent checkpoint (running typecheck/tests is a repeatable operation subordinate to the implementation checkpoint, not itself a durable state anyone inspects). Folded into `10-implement-with-tdd` (its sole neighbor, and the stage whose ongoing judgment it is subordinate to) as a helper. `BU-P2-053` survives, re-homed.
- **`40-commit` — DEMOTED.** Same boilerplate-only classification, no surviving checkpoint argument. Folded into `30-review` (its only neighbor; committing is the mechanical conclusion of a reviewed, verified change, not an independently observable checkpoint). `BU-P2-055` survives, re-homed. `30-review`'s output now carries the `promote` disposition `40-commit`'s output previously carried, since `30-review` is now the workflow's last stage.

## Notes

**Synthesis notes:** Explicit-invocation-only (BU-P2-051) — this workflow must never be auto-loaded merely because the task looks like implementation; its cross-harness mirror is BU-P3-004.

## Curation note (promotion gate-record completion, 2026-08-11)

`implement`'s promotion commit (68f2765) recorded packaging and the
NEEDS-JUDGMENT delegation check (delegates to `tdd` at
`10-implement-with-tdd` and `code-review` at `30-review`, both already
promoted) but no engine-acceptance gate evidence. This note completes the
record: `docs/icm/promotion-spec-2026-08-11.md` §3's procedure, run
2026-08-11 against `/home/miztertea/sergeant-runb/target/debug/sgt` in a
package-private scratch subject repo and data dir, `SGT_FAKE_SCRIPT`
unset — `work.state == "completed"`; one `workflow.bound` whose
`stage_bindings` matched `workflow.toml`'s two stages
(`10-implement-with-tdd`, `30-review`) in order; matching
`stage.entered`/`stage.completed` pairs in that order; one terminal
`work.completed` with `stages == 2`; two distinct `execution_id`s
(`01KZREP5204GGQEKQ8PETEAA6E`, `01KZREP5207VQ48SJDV29V5SRP`). Daemon
stopped and pgrep-confirmed gone before teardown. Per spec §1's D9
observation, the closing stage (`30-review`) declares a `promote`-
dispositioned output with no finalize step named — not a promotion
blocker, recorded here rather than left implicit.

