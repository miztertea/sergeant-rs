# Provenance — Vet External Skill

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W34** `vet-external-skill`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-119` | Before adopting an external skill, it must be vetted through a fixed sequence: read its complete SKILL.md and referenced scripts, confirm its source and update mechanism, check its filesystem/shell/network/git/credential actions, verify it does not conflict with repository AGENTS.md or safety policy, pin or lock its source where supported, and test it in a disposable repository or worktree before broad installation. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L124-131, vet-external-skill workflow trigger) |

## Stages

### `00-read-source`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-120` | Read the external skill's complete SKILL.md and referenced scripts before adopting it. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L126, vet step 1) |

### `10-confirm-provenance`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-121` | Confirm the external skill's source and update mechanism. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L127, vet step 2) |

### `20-check-actions`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-122` | Check the external skill's filesystem, shell, network, Git, and credential actions. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L128, vet step 3) |

### `30-verify-no-conflict`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-123` | Verify the external skill does not conflict with repository AGENTS.md or safety policy. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L129, vet step 4) |

### `50-test-in-disposable-copy`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-125` | Test the external skill in a disposable repository or worktree before broad installation. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L131, vet step 6) |
| `BU-P1-124` (helper invocation, folded from demoted `40-pin-source`) | Pin or lock the external skill's source where the installer supports it. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L130, vet step 5) |

### `60-update-managed`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-126` | For skills.sh-managed skills, rerun the official installer and inspect the diff and updated lock file before accepting changes. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L138-139, skills.sh update path) |

### `60-update-owned`

| Unit | Statement | Source |
|---|---|---|
| `BU-P1-127` | For Sergeant-owned skills, update this repository through a reviewed PR and run tests/instruction-policy-test.sh plus the full Sergeant test suite. | `reference/sergeant-upstream/docs/skills.md` (docs/skills.md L142-144, Sergeant-owned update path) |

## Notes

**Synthesis notes:** Five ordered checkpoints (`00`-`30`, `50`) plus two mutually exclusive update variants (`60-update-managed`/`60-update-owned`) reached only when refreshing an already-adopted skill. Each step's outcome ("the source was read", "the actions were checked", "it was tested in a disposable copy") survives any reimplementation of *how* the checking is done — a strong candidate for the smallest complete reference workflow in the corpus.

## Adjudication A4

- **`40-pin-source` — DEMOTED.** Its CONTEXT.md carried only the §6.5 deterministic-machinery boilerplate ("candidate execute-stage workload") with no additional checkpoint argument (no "Additional note" section). Per A4's default rule, folded into `50-test-in-disposable-copy` as a helper invocation; `BU-P1-124` moves with it. The stage directory is removed; `50-test-in-disposable-copy`'s Inputs table now points to `30-verify-no-conflict/output/README.md`. No renumbering: `00`, `10`, `20`, `30`, `50`, `60`/`60` remain correctly ordered without `40`.
- **`60-update-managed` — KEPT (reclassified).** Carried the §6.5 boilerplate plus a real "Additional note": it is an alternate entry point (reached only when updating an already-adopted, skills.sh-managed skill, not during the `00`-`50` sequence) and mutually exclusive with `60-update-owned`. Judged against §6.3's reimplementation test: the checkpoint is not the installer mechanism but the decision to accept the update after inspecting its diff/lock-file change — that decision is unchanged by any reimplementation of the installer, so it is genuine judgment, not subordinate machinery. There is also no larger stage for it to be "subordinate" to (it is an independent alternate-entry checkpoint, not a step inside another stage's crossing). Argument survives — kept, and reclassified from `stage (§6.3, deterministic-machinery candidate)` to `actor-stage (§6.4, judgment)`; its CONTEXT.md's "Deterministic-machinery candidate" section is replaced with a "Judgment required" section, the "Additional note" is preserved unchanged.
- **`60-update-owned` — KEPT (reclassified).** Same reasoning as `60-update-managed`: the "Additional note" establishes it as the mutually-exclusive alternate-entry counterpart, and the actual checkpoint — gating changes on a reviewed PR plus a passing instruction-policy test and full suite — is a genuine judgment/review gate that survives reimplementation of the test-runner mechanism. Kept and reclassified identically.


**Curation note (`docs/icm/promotion-spec-2026-08-11.md` §1 finalize gap):** this is the workflow's true closing stage (per `workflow.toml`'s own `stages` order — `60-update-owned` is last) and its `output/README.md` declares a `promote` disposition, but names no deterministic finalize step (D9, convention §1a open questions) — one of the corpus's 30 packages in that shape. Not a promotion blocker on the convention's own text; recorded here per the spec's curation rule rather than silently laundered.
