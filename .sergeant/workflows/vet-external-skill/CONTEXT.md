# Vet External Skill
Draft workflow package — candidate **W34** `vet-external-skill` from the N1
manual reference-corpus decomposition (`docs/gauntlet/contracts/N1.md`).
This is Layer 1 orientation only — it is never delivered as a stage's
instructions; each stage's own `CONTEXT.md` (Layer 2) is the actor's
contract (`docs/icm/convention.md` §1a rule 5).

## Purpose

Vet an external skill through a fixed sequence before adopting it, and keep already-adopted skills updated through the same discipline.

## Trigger

Before adopting an external skill, or when an adopted skill needs updating.

## Stages

| Stage | Ladder rung (as extracted) | Durable outcome |
|---|---|---|
| `00-read-source` | actor-stage (§6.4, judgment) | The external skill's complete SKILL.md and referenced scripts are read before adopting it. |
| `10-confirm-provenance` | actor-stage (§6.4, judgment) | The external skill's source and update mechanism are confirmed. |
| `20-check-actions` | actor-stage (§6.4, judgment) | The external skill's filesystem, shell, network, Git, and credential actions are checked. |
| `30-verify-no-conflict` | actor-stage (§6.4, judgment) | The external skill does not conflict with repository AGENTS.md or safety policy. |
| `50-test-in-disposable-copy` | actor-stage (§6.4, judgment) | The external skill's source is pinned or locked where the installer supports it; the skill is tested in a disposable repository or worktree before broad installation. |
| `60-update-managed` | actor-stage (§6.4, judgment — reclassified, N1 adjudication A4) | For skills.sh-managed skills: rerun the official installer and inspect the diff and updated lock file before accepting changes. |
| `60-update-owned` | actor-stage (§6.4, judgment — reclassified, N1 adjudication A4) | For Sergeant-owned skills: update this repository through a reviewed PR and run the instruction-policy test plus the full test suite. |

## Notes for reviewers

Five ordered checkpoints (`00`-`30`, `50`) plus two mutually exclusive update variants (`60-update-managed`/`60-update-owned`) reached only when refreshing an already-adopted skill. Each step's outcome ("the source was read", "the actions were checked", "it was tested in a disposable copy") survives any reimplementation of *how* the checking is done — a strong candidate for the smallest complete reference workflow in the corpus.

**N1 adjudication A4:** the former `40-pin-source` stage carried only the §6.5 deterministic-machinery boilerplate as its stage-level justification, with no additional checkpoint argument; it is demoted and folded into `50-test-in-disposable-copy` as a helper invocation. `60-update-managed` and `60-update-owned` also carried the §6.5 boilerplate, but each also carried a real "Additional note" checkpoint argument; judged against §6.3's reimplementation test, both are KEPT (their outcome — a human/agent decision to accept an update after inspection — survives any reimplementation of the underlying installer/test mechanism) and reclassified from `stage (§6.3, deterministic-machinery candidate)` to `actor-stage (§6.4, judgment)`. No renumbering: `00`, `10`, `20`, `30`, `50`, `60`/`60` remain correctly ordered without `40`. See `docs/gauntlet/promoted-provenance/vet-external-skill.md`'s "Adjudication A4" section.

**The two-entry structural tension (ICM-R3):** `workflow.toml` declares one
single linear `stages` list containing all seven stages in order; the
"mutually exclusive update variants" description above assumes the engine
supports starting a Work's walk at a `60-*` stage and stopping there rather
than walking the full list from `00-read-source`. This reading is
consistent with `validate-and-ship/CONTEXT.md`'s own already-published
two-entry-point precedent, but is not independently engine-verified from
content alone (no `src/` evidence of stage-subset entry or partial-stage-
walk support was found). Recorded here as the working assumption, not a
settled fact; an execution-valid run would confirm it.

## Authority envelope

This workflow receives an already-admitted intent: vet a named external
skill before adopting it, or update an already-adopted skill.

### Workflow may decide
- What counts as a "referenced script" when reading the skill's full
  instructions (`00-read-source`).
- Whether a claimed source and update mechanism are confirmable from
  available evidence (`10-confirm-provenance`).
- How to assess the skill's actual side-effect surface from source
  inspection (`20-check-actions`).
- Whether a given instruction or action actually conflicts with repository
  `AGENTS.md` or safety policy (`30-verify-no-conflict`).
- Whether a disposable-copy test run is representative enough to trust
  (`50-test-in-disposable-copy`).
- Accept or reject an update after inspecting its diff/lock-file change or
  its PR and test results (`60-update-managed`, `60-update-owned`).

### Workflow may not decide
- Adopt an external skill without completing the fixed sequence.
- Proceed past unconfirmable provenance, a concerning action finding, a
  confirmed `AGENTS.md`/safety-policy conflict, a failing disposable-copy
  test, or a suspicious update diff without asking.
- Ship a Sergeant-owned skill update without a reviewed PR and a passing
  instruction-policy test plus full suite.

### Human or Captain gates
- Provenance cannot be confirmed (`10-confirm-provenance`).
- A checked action is severe enough that continuing would be irresponsible
  without a stop (`20-check-actions`).
- A conflict with `AGENTS.md` or safety policy is found
  (`30-verify-no-conflict`).
- The disposable-copy test fails (`50-test-in-disposable-copy`).
- Inspection of a managed-skill update finds something that should not be
  silently accepted (`60-update-managed`).

### Decision record
Material decisions (confirmed provenance, checked actions, conflict
verdicts, test results, accept/reject decisions, any `J0` stop) are
recorded in each stage's own turn and surfaced through `needs_input` where
applicable; this workflow declares no separate decision-log file.

## Provenance

See `docs/gauntlet/promoted-provenance/vet-external-skill.md` for the complete stage-to-behavior-unit mapping and workflow-level citations.
