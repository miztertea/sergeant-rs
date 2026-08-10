# Provenance — Cross-Repo Work

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W7** `cross-repo-work`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-038` | cross-repo-work decomposes a requested outcome across the repositories that own it and defines dependency and merge order before any dispatch happens. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 8-9) |
| `BU-P5-039` | cross-repo-work is loaded when the context-resolution output shows more than one repository owns the requested outcome, not merely because the project contains multiple repositories. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 13-14) |
| `BU-P5-056` | dispatch requires load-project (repos/paths/instructions known) and cross-repo-work (repos, dependency order, and per-repo work known, or manually confirmed equivalent) as completed prerequisites. | `reference/sergeant-upstream/skills/dispatch/SKILL.md` (lines 19-21) |

## Stages

### `10-assign-ownership`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-041` | For each required behavior, cross-repo-work names exactly one owning repository, including a repository only when it must change or produce delivery evidence, and records repo/role/delivers/acceptance for it. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 21-26) |
| `BU-P5-042` | The per-repository ownership record has a fixed shape: repo name, resolved role, the observable behavior or artifact it delivers, and the repo-native command or evidence that proves completion. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 27-34) |
| `BU-P5-043` | Ambiguous repository ownership is resolved using the project graph and existing contracts first; the user is asked only when two repositories could legitimately own a user-visible or durable contract. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 36-38) |

### `20-define-dependency-order`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-044` | Dependency edges are created only when one repository's merged or deployed result is required by another, expressed in the prerequisite>dependent notation accepted by the dispatch command. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 40-43) |
| `BU-P5-045` | Recognized dependency-edge evidence includes: a contract/schema producer preceding its consumers, infrastructure/config preceding runtime that requires it, independent implementations running in parallel once an approved contract exists, and deployment dependency recorded separately from code-merge dependency. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 45-51) |
| `BU-P5-046` | Cycles are rejected before dispatch; if a cycle reflects a genuinely coupled contract, cross-repo-work instead defines the contract artifact or compatibility phase that breaks the cycle. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 53-54) |

### `30-inspect-repository-state` (folded into `40-define-delivery-gates`, N1 adjudication A4)

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-047` | cross-repo-work runs the status command and records non-main branches, uncommitted changes, ahead/behind state, active worktrees, and preserved workers for every owning repository before planning proceeds. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 56-59) |
| `BU-P5-048` | cross-repo-work never stashes, resets, switches, or cleans repository state during planning; it either routes an existing canonical branch/worktree into the worker brief or stops for a decision when state conflicts with the requested outcome. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 61-63) |

### `40-define-delivery-gates`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-049` | Every per-repository delivery gate must include: the owning td task (or its creation requirement), the fixed point and preserved source state, repository-specific test/lint/typecheck/build commands, Standards and Spec review sources, PR dependency and deployment order, and any already-approved or still-missing data/security/destructive decisions. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 65-74) |
| `BU-P5-050` | The cross-repo plan is complete only when every owning repository has one implementation brief, acceptance evidence, and an acyclic dependency position. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 76-77) |

### `50-handoff-or-stop`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-051` | If the user requested planning only, cross-repo-work stops after returning the briefs, acceptance evidence, and dependency graph, without dispatching or editing any repository; if implementation was requested, it hands off to the dispatch workflow via its launch command, and the primary session itself never edits several repositories directly. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 79-85) |
| `BU-P7-017` | sgt-dispatch must never itself carry out `git checkout -b`, `git push -u origin`, or `gh pr create` as its own inline behavior in the cross-repo-work skill's prose; these operations belong to the dispatched worker, not to the coordinating skill. | `reference/sergeant-upstream/tests/instruction-policy-test.sh` (lines 69-71) |

### `60-reconcile`

| Unit | Statement | Source |
|---|---|---|
| `BU-P5-052` | After dispatched workers finish, cross-repo-work reconciles PR URLs and final heads, required CI and unresolved review threads, merge order from dependency edges, deployment order and cross-repo release notes, and terminal td/fleet state and cleanup eligibility. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 87-93) |
| `BU-P5-053` | cross-repo-work never reports the cross-repo outcome complete until every owning repository has a terminal result or an explicit preserved blocker. | `reference/sergeant-upstream/skills/cross-repo-work/SKILL.md` (lines 95-96) |

**Scope, N1 adjudication A8 (BH-10):** both units above are read as
scoped to the repos this Work's plan named — `BU-P5-052`'s "terminal
td/fleet state and cleanup eligibility" reports completion facts for those
repos, it does not assert this stage owns fleet-wide reconciliation
(`dispatch`) or cleanup (`reconcile-and-cleanup-fleet`). See
`60-reconcile/CONTEXT.md`'s "Scope note" and this package's `CONTEXT.md`
"Adjudication notes" section.

## Adjudication A4

Applying the reference-corpus's N1 round-1 adjudication (`reference-corpus/adjudication-round1.md` A4, finding N1-BH-02):

| Stage | Additional note present? | §6.3 reimplementation test | Decision |
|---|---|---|---|
| `30-inspect-repository-state` | none | Swapping the status-inspection implementation (which command reads branch/worktree/ahead-behind state) leaves the checkpoint — a read-only record of repo state before planning proceeds — unchanged. | **Demoted** — folded into `40-define-delivery-gates` as a preceding helper invocation. |
| `10-assign-ownership`, `20-define-dependency-order`, `40-define-delivery-gates`, `50-handoff-or-stop`, `60-reconcile` | n/a (extracted as actor-stage, §6.4, already judgment-bearing) | n/a | **Kept** as extracted. |

Stage count: 6 extracted → 5 surviving. No behavior unit was deleted; `BU-P5-047` and `BU-P5-048` remain cited, now under `40-define-delivery-gates`'s "Helper invocations" section (see that stage's `CONTEXT.md`).

