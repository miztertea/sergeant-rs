# Cross-Partition Synthesis (§8.3 step 5)

> **Superseding note (added post-round-1-adjudication, V10).** This
> document is a **phase artifact**, frozen at the moment it was written: its
> stage/workflow vocabulary (W-numbers, stage names, stage lists, and the
> 966-unit census throughout) reflects the corpus *as it stood at synthesis*
> and is now historical. `adjudication-round1.md`'s structural rulings
> (A3–A8) subsequently reordered, demoted, restored, merged, split, and
> removed stages and whole packages — and A9–A12 backfilled, normalized,
> and added units — none of which is reflected below. **Do not treat this
> document as a live map of current stage boundaries or unit counts.** For
> the authoritative, current state of any package, read that package's own
> `draft-workflows/<name>/provenance.md` and `workflow.toml`; for how the
> two diverge and why, `adjudication-round1.md` is the bridge document.
> This note is additive — the content below is preserved unedited as the
> historical record of the synthesis pass itself.

**Input:** 966 behavior units in `reference-corpus/behavior-units/P1.ndjson` … `P8.ndjson`
(P1 131, P2 127, P3 96, P4 100, P5 149, P6 142, P7 112, P8 109). No duplicate ids.
**Source snapshot:** `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`.
**Method:** §6 ICM decomposition ladder applied to the *union* of the eight partitions;
partition-local `representation` / `workflow` / `stage` assignments are treated as
proposals, not verdicts — this pass merges, re-splits, and (where the evidence
requires) overrules them, recording every overrule as a conflict entry in §6.

**Representation census (as extracted):** agents-invariant 105 · workflow 66 ·
stage 203 · stage-context 299 · shared-context 127 · shared-helper 49 · helper 73 ·
obsolete-mechanism 28 · engine-gap 16.

**Coverage accounting (every id has exactly one home):**

| Home | Units |
|---|---|
| §1 candidate workflow clusters | 568 |
| §2 permanent-instruction set | 103 |
| §3 shared-context / helper map | 248 |
| §4 obsolete-mechanism roll-up | 28 |
| §5 engine-pressure roll-up | 16 |
| §7 unassigned (explicit, with reasons) | 3 |
| **Total** | **966** |

Engine-gap units are additionally cross-referenced to the workflow cluster whose
procedure they arise inside; their *home* for coverage purposes is §5.

*Lint note.* A section will mention more ids than it homes, because clusters name the
shared contexts and helpers they consume and §5/§7 name their supporting evidence.
Verified mechanically: 966 distinct ids appear in the document; all 966 appear in
§1–§5 or §7 (i.e. none is mentioned only in the conflict table); §1 homes exactly the
568 workflow/stage/stage-context units and cross-references eight others
(BU-P1-071, BU-P1-078, BU-P2-101, BU-P2-102, BU-P3-002, BU-P3-004, BU-P6-049,
BU-P7-026), each marked as belonging to §3; and the three §7 units appear nowhere
else.

---

## 1. Candidate workflow list

Thirty-five candidates. Each stage below was put through §6.3's reimplementation
test — *if this were reimplemented tomorrow by different machinery, would the
checkpoint still exist?* Stages that failed the test were demoted to helpers (§3)
and are named as such. `A` = actor stage (§6.4, judgment required),
`D` = deterministic-machinery candidate (§6.5, would be an execute stage the moment
one exists; today an actor stage that invokes a helper).

### Group A — project and installation

#### W1 `load-project` — resolve a project's ownership, paths, and instruction layers
*Purpose:* establish, before any mutation, which repositories own the requested
outcome, where they are, what instructions govern them, and what state they are in.
*Trigger:* a project is named, registered, edited, synced, or listed; or repository
ownership is not already established.
*Units (26):* BU-P5-040 BU-P5-090 BU-P5-091 BU-P5-092 BU-P5-093 BU-P5-094 BU-P5-095
BU-P5-096 BU-P5-097 BU-P5-098 BU-P5-099 BU-P5-101 BU-P5-102 BU-P5-103 BU-P5-108
BU-P5-109 BU-P5-110 BU-P5-111 BU-P6-010 BU-P6-011 BU-P6-012 BU-P6-013 BU-P6-014
BU-P6-021 BU-P6-022 BU-P6-035

| Stage | Kind | Durable outcome | Survives reimplementation? |
|---|---|---|---|
| `00-resolve-project-name` | A | An exact registered project name is bound, or the run stops asking whether to register (BU-P5-092, BU-P5-108) | Yes — "which project are we in" is a checkpoint regardless of whether `sgt-list` or a database answers it |
| `10-resolve-context` | A | Owning repos, absolute paths, clone state, roles/groups, and the layered instruction set are recorded as the governing context (BU-P5-093/094/096, BU-P6-021/022) | Yes |
| `20-register-or-edit` | A | A project definition is written to the Sergeant-owned config path and validated, or the prior definition is restored (BU-P5-097/098/099/101/103) | Yes — "the definition is valid and visible" is the checkpoint, not the YAML format |
| `30-sync-repositories` | D | Every required repo is cloned/refreshed, or the run stops naming the exact failure (BU-P5-095, BU-P5-102, BU-P6-013, BU-P6-014) | Yes |
| `40-report-state` | D | A read-only per-repo report of clone/branch/cleanliness/ahead-behind and open tracked work (BU-P6-012, BU-P6-035) | **Borderline** — this is closer to a query than a checkpoint; kept as a stage only because operators do care whether it succeeded before planning |

*Demoted from the extraction:* `list-projects` (BU-P6-010/011), `project-status`
(BU-P6-012), `project-sync` (BU-P6-013/014), `project-task-list` (BU-P6-035) were
each extracted as standalone workflows in P6. They are command surfaces, not
procedures with bounded outcomes and completion conditions — folded in here. See
conflict **X11**.

#### W2 `project-graph` — build and publish one project knowledge graph
*Purpose:* produce exactly one merged, published graph per project, outside every
source repository, usable for architecture questions.
*Trigger:* architecture work needs whole-project structure, or the operator asks for
a graph/refresh.
*Units (13):* BU-P5-100 BU-P5-105 BU-P5-106 BU-P5-107 BU-P5-112 BU-P6-088 BU-P6-089
BU-P6-090 BU-P6-091 BU-P7-003 BU-P7-086 BU-P7-088 BU-P8-103

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-resolve-output-path` | A | One project-level output path is confirmed (or requested from the user) and is outside every source repo (BU-P5-100, BU-P5-107, BU-P8-103) | Yes |
| `10-extract-per-repo` | D | Per-repo extraction completed, with in-source output staged out of the way and code-only fallback when no LLM key exists (BU-P6-089, BU-P6-090, BU-P7-088) | Yes |
| `20-merge-or-fail` | D | All-or-nothing: any repo's extraction failure fails the run before merge (BU-P6-091) | Yes — "we never publish a partial graph" outlives any merger |
| `30-publish-atomically` | D | Readers see the complete old or complete new graph, never a torn state; a failed swap leaves the previous output valid (BU-P6-088, BU-P7-003, BU-P7-086) | Yes |
| `40-consume` | A | The graph is queried for focused questions or the report read for broad context (BU-P5-106, BU-P5-112, BU-P5-105) | **No** — demote to helper/context (§3); "I ran a query" is not a checkpoint operators track |

Extracted in P5/P7 as stages of `load-project`, in P6 as its own workflow `graphify`.
Promoted to its own workflow here: it has an independent trigger, a bounded outcome
(a published graph), and its own failure mode. See conflict **X9**.

#### W3 `sergeant-setup` — bootstrap or repair an installation, interactively and idempotently
*Purpose:* bring an installation from any partial state to a verified-complete state
without ever silently reconfiguring anything the operator did not consent to.
*Trigger:* first install, a new project/repository to register, a broken or
incomplete installation, or a verification request.
*Units (52):* BU-P5-001 BU-P5-002 BU-P5-003 BU-P5-004 BU-P5-005 BU-P5-006 BU-P5-007
BU-P5-008 BU-P5-009 BU-P5-010 BU-P5-011 BU-P5-012 BU-P5-013 BU-P5-014 BU-P5-015
BU-P5-016 BU-P5-017 BU-P5-018 BU-P5-019 BU-P5-020 BU-P5-021 BU-P5-022 BU-P5-023
BU-P5-024 BU-P5-026 BU-P5-027 BU-P5-028 BU-P5-029 BU-P5-030 BU-P5-031 BU-P5-032
BU-P5-033 BU-P5-034 BU-P5-035 BU-P5-036 BU-P5-037 BU-P6-003 BU-P6-018 BU-P7-025
BU-P7-036 BU-P7-037 BU-P7-038 BU-P7-039 BU-P7-040 BU-P8-041 BU-P8-042 BU-P8-044
BU-P8-045 BU-P8-047 BU-P8-048 BU-P8-049 BU-P8-051

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-detect-prerequisites` | D | Every checked tool is classified present / installable / unsupported; required gaps stop the run unless the user accepts the risk (BU-P5-009/010/011/013, BU-P6-003, BU-P7-025, BU-P7-026†, BU-P8-041/042) | Yes |
| `05-file-capability-gaps` | A | Each unsupported capability becomes an approved tracked issue, or is reported as an unfilled gap (BU-P5-012) | Yes |
| `10-install-commands` | A | Commands resolve on PATH, verified; failure stops with the expected source paths named (BU-P5-015/016/017/018/019) | Yes |
| `20-global-config` | A | One machine-wide `dev_root` exists and parses; an existing file is never overwritten without backup + diff + confirmation (BU-P5-020/021/022, BU-P8-044) | Yes |
| `30-project-interview` | A | A complete project definition is captured from the user, previewed in full, and written only after confirmation (BU-P5-023/024/026/027, BU-P8-045) | Yes — **this stage is the U3 case**; see engine-gap **G5** |
| `40-repair-existing` | A | An existing definition is validated, a minimal diff shown, and changes applied only after confirmation with a mandatory post-confirmation backup (BU-P5-028/029/030, BU-P7-038) | Yes |
| `50-sync-and-verify` | D | The four verification commands run in fixed order, stopping at the first failure (BU-P5-031) | Yes |
| `60-task-tracking-init` | A | Tracked-work storage initialized per registered repo, each behind explicit consent (BU-P5-032/033, BU-P7-037) | Yes |
| `70-optional-capabilities` | A | Worktree pools and graph output initialized only where explicitly desired; declining never marks setup incomplete (BU-P5-034/035, BU-P6-018, BU-P8-047/048) | Yes |
| `90-completion-summary` | D | Every checklist item resolved as `[ok]` / `[skipped]` / `[issue: id]` (BU-P5-007/036/037/008, BU-P7-039/040, BU-P8-049/051) | Yes |

Standing constraints spanning every stage (candidates for a workflow-local
`CONTEXT.md`, not stages): write only to Sergeant-owned paths (BU-P5-004),
never to other tools' config surfaces (BU-P5-005, BU-P7-036), never auto-initialize
external tools (BU-P5-006). † BU-P7-026 lives in the shared-context map (§3).

`sergeant-install` (P8, from `docs/getting-started.md`) is the same procedure told
as a checklist rather than as phases — merged. See conflict **X10**.

#### W4 `sergeant-help` — answer a Sergeant question from repository-owned documentation
*Purpose:* answer usage/setup/troubleshooting questions from documentation with an
explicit precedence order, read-only, never inventing behavior.
*Trigger:* the user asks what Sergeant is, how to install/configure/use it, where
skills come from, or how to diagnose a Sergeant error.
*Units (13):* BU-P5-113 BU-P5-114 BU-P5-115 BU-P5-117 BU-P5-119 BU-P5-120 BU-P5-121
BU-P5-122 BU-P5-125 BU-P5-126 BU-P5-127 BU-P5-128 BU-P5-129

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-classify-and-locate` | A | The question is bound to one primary document, which is read before any broad search; a missing primary document stops the run with its expected path (BU-P5-117, BU-P5-126) | Yes |
| `10-resolve-source-conflicts` | A | Where sources disagree, the answer follows the fixed precedence and the mismatch is reported as tracked work (BU-P5-122, BU-P5-127, BU-P5-120, BU-P5-119) | Yes |
| `20-answer-or-hand-off` | A | Either a fixed-format answer with command, preconditions, evidence and doc links, or an explicit hand-off to the owning procedure (BU-P5-121, BU-P5-125, BU-P5-128, BU-P5-129, BU-P5-113/114/115) | Yes |

### Group B — task intake and execution mode

#### W5 `task-intake-and-route` — turn a user request into a chosen, scoped execution mode
*Purpose:* the standing nine-step entry procedure every task passes through before
any implementation workflow starts.
*Trigger:* any task the user brings.
*Units (15):* BU-P1-003 BU-P1-025 BU-P1-026 BU-P1-027 BU-P1-028 BU-P1-029 BU-P1-030
BU-P1-031 BU-P1-032 BU-P1-033 BU-P1-034 BU-P1-038 BU-P1-108 BU-P8-053 BU-P8-054

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `01-load-context` | A | Owning repositories, inherited instructions and cross-repo dependencies are known (BU-P1-026) | Yes (delegates to W1) |
| `02-check-queue` | D | A matching tracked task is reused, or one is created because none is canonical (BU-P1-027) | Yes |
| `03-choose-mode` | A | Direct or dispatch is selected on the four stated criteria (BU-P1-028, BU-P1-003, BU-P1-108, BU-P8-053, BU-P8-054) | Yes |
| `04-reconcile-state` | D | Active workers, branches, worktrees, retained gates and handoffs are inspected; preserved work is resumed rather than duplicated (BU-P1-029) | Yes |
| `05-confirm-decisions` | A | Only genuinely unresolved scope/risk decisions are put to the user (BU-P1-030) | Yes |
| `06-execute` | A | Control passes to W6 or W8 (BU-P1-031) | Yes — the hand-off point is observable |
| `07-monitor` | D | Progress is evidenced by recent meaningful events plus exact process identity (BU-P1-032) | Yes |
| `08-handle-decisions` | A | Each gate resolved with a recorded human decision where required (BU-P1-033, BU-P1-038) | Yes |
| `09-reconcile-deliver` | A | PRs, merge order, merges/deployments and cleanup eligibility are settled (BU-P1-034) | Yes |

#### W6 `direct-implementation` — implement in the current session, one owning repository
*Purpose:* the same delivery contract as a dispatched worker, executed in-session.
*Trigger:* the user explicitly asks to work in this session **and** one repository
owns the complete outcome.
*Units (13):* BU-P1-007 BU-P1-008 BU-P1-009 BU-P1-010 BU-P1-011 BU-P1-012 BU-P1-013
BU-P1-014 BU-P1-016 BU-P1-107 BU-P8-055 BU-P8-056 BU-P8-058

Stages: `01-load-task-context` (A, BU-P1-008) · `02-reconcile-existing-state`
(D, BU-P1-009, BU-P8-056) · `03-claim-and-implement` (A, BU-P1-010, BU-P1-011) ·
`04-validate` (A, BU-P1-012) · `05-shipping-gate` (A, BU-P8-058 — invoked only at
the approved boundary; delegates to W18) · `06-pr-and-merge` (A, BU-P1-013) ·
`07-record-outcomes` (D, BU-P1-014). Standing constraints: BU-P1-016, BU-P1-007,
BU-P1-107, BU-P8-055. All seven survive reimplementation — each is a boundary an
operator would want measured independently.

#### W7 `cross-repo-work` — decompose an outcome across repositories and define delivery order
*Purpose:* produce a plan in which every required behavior has exactly one owning
repository, an acyclic dependency position, a brief, and acceptance evidence —
before any dispatch happens.
*Trigger:* resolved project context shows more than one repository owns the
requested outcome (not merely that the project has several repos).
*Units (17):* BU-P5-038 BU-P5-039 BU-P5-041 BU-P5-042 BU-P5-043 BU-P5-044 BU-P5-045
BU-P5-046 BU-P5-047 BU-P5-048 BU-P5-049 BU-P5-050 BU-P5-051 BU-P5-052 BU-P5-053
BU-P5-056 BU-P7-017

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `10-assign-ownership` | A | Exactly one owning repo per behavior, with role / deliverable / acceptance recorded (BU-P5-041/042/043) | Yes |
| `20-define-dependency-order` | A | An acyclic edge set in prerequisite>dependent form; cycles broken by a named contract artifact (BU-P5-044/045/046) | Yes |
| `30-inspect-repository-state` | D | Non-main branches, uncommitted changes, ahead/behind, worktrees, preserved workers recorded without mutating anything (BU-P5-047/048) | Yes |
| `40-define-delivery-gates` | A | Per-repo gate: owning task, fixed point, native commands, review sources, PR/deploy order, outstanding decisions (BU-P5-049/050) | Yes |
| `50-handoff-or-stop` | A | Either the plan is returned (planning-only) or control passes to W8; the coordinator never edits several repos itself (BU-P5-051, BU-P7-017) | Yes |
| `60-reconcile` | A | PR URLs, heads, CI, review threads, merge and deployment order, terminal task/fleet state (BU-P5-052/053) | Yes |

Prerequisite relation to W1/W8 recorded by BU-P5-038, BU-P5-039, BU-P5-056.

### Group C — dispatch and the worker fleet

This is the densest region of the corpus and the region P5/P6/P7/P8 partitioned most
differently. The synthesis splits it by *bounded outcome*, not by source file: one
coordinator-side workflow that produces running workers (W8), one worker-side
workflow that produces a delivered change (W9), and separate lifecycle workflows
for the things that happen to a running fleet (W10–W17). See conflict **X12**.

#### W8 `dispatch` — launch one isolated worker per owning repository under one task identity
*Purpose:* given a project, a brief or tracked task, and a repository set, produce
one durable task with an isolated work surface, a rendered mission brief, and a
running agent per repository — with every side effect validated and gated before the
next repository's dispatch begins.
*Trigger:* work spans repositories, contains ≥2 independent repository-owned tasks,
needs an isolated review worker, or the user asks for workers.
*Units (63):* BU-P1-004 BU-P1-005 BU-P1-006 BU-P1-057 BU-P1-058 BU-P1-060 BU-P1-093
BU-P1-094 BU-P5-054 BU-P5-055 BU-P5-057 BU-P5-058 BU-P5-059 BU-P5-060 BU-P5-061
BU-P5-062 BU-P5-066 BU-P5-067 BU-P5-068 BU-P5-070 BU-P5-071 BU-P5-074 BU-P5-075
BU-P5-076 BU-P5-078 BU-P5-079 BU-P5-080 BU-P5-081 BU-P5-082 BU-P5-083 BU-P5-084
BU-P5-085 BU-P5-086 BU-P5-087 BU-P5-088 BU-P5-089 BU-P6-019 BU-P6-036 BU-P6-048
BU-P6-107 BU-P6-110 BU-P6-113 BU-P6-115 BU-P6-116 BU-P6-123 BU-P6-124 BU-P6-125
BU-P6-126 BU-P6-128 BU-P7-002 BU-P7-016 BU-P7-069 BU-P7-070 BU-P7-072 BU-P7-073
BU-P7-074 BU-P7-075 BU-P7-078 BU-P7-111 BU-P7-112 BU-P8-059 BU-P8-069 BU-P8-070

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-check-queue-and-plan` | A | Either an existing tracked task supplies brief/branch/context, or a free-form brief plus explicit repo list is confirmed as accurate before anything is created (BU-P5-057/058/059/060, BU-P6-123) | Yes |
| `05-classify-risk` | A | The objective is routed to the standard-isolated path or forced onto an explicit intent-file path by a fixed safety-sensitive keyword set (BU-P6-048, BU-P7-016, BU-P8-069) | Yes |
| `10-preflight-capabilities` | D | Harness, model tuple, identity and pane/session bindings are all validated and rejected **before any durable state exists** (BU-P1-057/058/060/093/094, BU-P6-107, BU-P6-124, BU-P7-002, BU-P7-072, BU-P7-073, BU-P7-078) | Yes — "nothing was created if this failed" is the checkpoint |
| `15-check-admission` | D | The fleet-wide admission lock is held only across the first side effect, then released (BU-P6-128; blocks on W12's drain state) | Yes |
| `20-prepare-intent` | A | One canonical intent revision exists and is written identically to fleet state and every selected work surface (BU-P8-059, BU-P6-049†) | Yes |
| `30-create-tracked-work` | D | All-or-nothing task creation across every target repo, rolled back on any failure (BU-P5-088, BU-P6-036) | Yes |
| `40-reconcile-before-launch` | D | Bulk fleet reconciliation runs before new work is created (BU-P8-070) | Yes |
| `50-acquire-surface` | D | An isolated work surface per repo at a deterministic location; a branch already carrying unpushed committed work is refused unless explicitly adopted (BU-P5-061, BU-P6-019, BU-P6-125, BU-P7-069, BU-P7-070) | Yes |
| `60-render-brief` | D | Mission, merged instructions, dependency notes, delivery requirements and any verbatim user override are durably carried to the worker before it starts (BU-P5-062, BU-P5-087, BU-P7-112, BU-P7-074, BU-P7-075) | Yes |
| `70-launch-and-record` | D | Launch evidence is written `intended` then promoted to `confirmed` only on observed readiness; every per-repo failure records an orphaned status with a diagnostic before the loop aborts (BU-P6-110, BU-P6-126, BU-P7-111) | Yes |
| `80-monitor` | A | Escalations are read in full, human decisions obtained without inference, delivered to the exact task/repo pair (BU-P5-066/067/068, BU-P6-113, BU-P6-115, BU-P6-116) | Yes (delegates to W10) |
| `90-reconcile-fleet` | A | Per-repo verification of pinned scope, validation, review artifacts, zero blocking findings, CI, threads, and dependency merge order — never complete merely because PRs exist (BU-P5-070/071, BU-P1-006) | Yes |

Worker-contract content the dispatch stage *authors* but does not itself execute
(BU-P5-075/076/078/079/080/081/082/083/084/085/086/089, BU-P1-004/005) is the input
to W9 and W16; kept in this cluster because dispatch owns writing it into the brief.
† BU-P6-049 is shared context (§3).

**Obsolete-mechanism stress test (§8.2).** The `dispatch` skill's tmux/sentinel/
worker-Bash machinery is *not* in the stage list above. Every stage names a durable
outcome that is independent of tmux panes, `.sergeant-*` sentinel files, process
groups, or terminal nudges. Those mechanisms are rolled up in §4 (M1–M9) with the
durable policy each one was carrying. What survived the separation: preflight-before-
side-effect, all-or-nothing tracked-work creation, one canonical intent revision,
durable brief delivery, intended→confirmed launch evidence, per-repo failure
recorded rather than silent. What did not: the pane as the worker's identity, the
pane as the notification channel, the pane as the liveness signal, the brief as a
loose file, the nudge loop as delivery.

#### W9 `worker-mission` (software-change) — a dispatched worker delivers one repository's change
*Purpose:* from a brief, produce a merged-ready change with evidence.
*Trigger:* a worker starts against a rendered brief.
*Units (7):* BU-P7-005 BU-P7-007 BU-P7-009 BU-P7-012 BU-P7-013 BU-P7-066 BU-P7-110

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-pin-scope` | D | Refs fetched, a fixed base commit pinned, base SHA/commit list/diff scope recorded before implementation (BU-P7-005) | Yes |
| `10-triage-and-route` | A | Full originating context read, redundant work checked, and the work classified into one of five categories, each loading a different canonical procedure (BU-P7-007) | Yes — **the branching point that raises engine-gap G6** |
| `20-implement` | A | The chosen discipline (W19/W20/W21/W22/W25) runs to its own completion | Yes |
| `30-independent-review` | A | Every axis named in the brief's authoritative list run as separate non-contaminating parallel reviews, outputs unblended (BU-P7-013) | Yes |
| `40-escalate-or-continue` | A | A new gate is published only when a monotonic generation actually advanced; the handshake is acknowledged, accepted, acted on once, and marked complete (BU-P7-009, BU-P7-012) | Yes — the *handshake* is durable; the file-per-step mechanism is not (§4 M5) |
| `50-publish-result` | D | Handoff evidence recorded from the verified work surface; readiness bounded and reported rather than hanging (BU-P7-066, BU-P7-110) | Yes |

#### W10 `respond-to-worker` — deliver a human decision to a blocked worker and resume it
*Purpose:* a blocked/needs-input/waiting/orphaned worker is durably given exactly one
decision, applies it exactly once, and returns to forward progress.
*Trigger:* a worker has published an escalation and a human decision exists.
*Units (21):* BU-P6-027 BU-P6-030 BU-P6-032 BU-P6-034 BU-P6-078 BU-P6-079 BU-P6-080
BU-P6-114 BU-P7-035 BU-P7-041 BU-P7-042 BU-P7-044 BU-P7-047 BU-P7-048 BU-P7-052
BU-P7-053 BU-P7-058 BU-P7-059 BU-P7-060 BU-P7-109 BU-P8-079

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-precondition-check` | A | Exact question read, only genuinely missing decisions asked, decision recorded in tracked work, no unconsumed generation already pending (BU-P8-079) | Yes |
| `10-validate-target` | D | The target's status is one of the four respondable states and its recorded identity/ownership evidence verifies; anything else refuses (BU-P6-078, BU-P7-060) | Yes |
| `20-publish-response` | D | The response is durably stored (even under an active drain) before any delivery is attempted (BU-P7-058, BU-P7-035) | Yes |
| `30-deliver-and-accept` | D | Bounded readiness gate; on timeout, a nonce-scoped unreachable record plus a recoverable gate — never a fabricated acknowledgement (BU-P6-114, BU-P7-109, BU-P7-059) | Yes |
| `40-apply-and-acknowledge` | A | Decision applied once, truthful status restored, applied id/generation/status recorded, then acknowledged from the owning context (BU-P6-032, BU-P6-034, BU-P7-041, BU-P7-044) | Yes |
| `50-archive-evidence` | D | Body, generation, applied status and proof archived atomically; the recorded generation is fixed at acknowledgement time (BU-P7-042, BU-P7-052, BU-P7-053) | Yes |
| `60-notify-coordinator` | D | The update is classified into exactly one durable event kind and recorded; live transports are optional on top (BU-P6-027, BU-P6-030, BU-P7-047, BU-P7-048) | Yes |
| `70-relaunch-if-needed` | D | Convergence attempted through the single finalizer before any refusal; superseded identities preserved as evidence (BU-P6-079, BU-P6-080) | Yes |

#### W11 `recover-stalled-worker` — one bounded recovery attempt for a stalled worker
*Units (11):* BU-P6-071 BU-P6-072 BU-P6-073 BU-P6-075 BU-P7-092 BU-P7-093 BU-P7-094
BU-P7-095 BU-P8-095 BU-P8-099 BU-P8-109
*Trigger:* a worker is `in_progress` with a stall classification recorded by the watcher.
Stages: `00-collect-signals` (A — four signals together before any kill/relaunch
decision, BU-P8-095, BU-P8-099) · `10-preflight` (D — stall proof, lease convergence,
drain check, relaunch-metadata completeness, old identity, all run to completion
before the attempt is stamped, BU-P6-071/073/075, BU-P7-092/093) · `20-launch-replacement`
(D — replacement validated live *before* the original is retired, BU-P6-072, BU-P7-094)
· `30-retire-original` (D, BU-P7-095) · `40-escalate-on-second-attempt`
(A — exactly one bounded attempt; a second escalates to needs-input, BU-P6-071) ·
`50-escalate-undocumented` (A, BU-P8-109). All survive: "was recovery attempted, and
did it converge or escalate" is the checkpoint regardless of mechanism.

#### W12 `drain-fleet` — cooperative, bounded, non-destructive admission block
*Units (12):* BU-P6-015 BU-P6-039 BU-P6-057 BU-P6-058 BU-P6-062 BU-P6-064 BU-P6-111
BU-P7-083 BU-P7-084 BU-P7-107 BU-P7-108 BU-P8-077
Stages: `00-set-drain` (D — admission refused the instant it is set, scope global or
per-project, race closed by an explicit lock, BU-P6-057/058/062, BU-P8-077) ·
`10-await-convergence` (D — bounded wait; a worker counts as drained only when its
exit is provable; timeout leaves the drain active, exits non-zero, and names the
unresolved, BU-P6-064, BU-P8-077) · `20-worker-side-checkpoint` (D — idempotent
drain detection; publish handoff and settle the lease *before* terminating anything,
BU-P6-111, BU-P7-084, BU-P7-107, BU-P7-108) · `30-force-stop` (A — refused unless a
drain is already active, requires explicit confirmation or dry-run, displays exact
identity, BU-P6-039, BU-P7-083) · `40-undrain` (D — idempotent, mutually exclusive
scopes, BU-P6-015). All survive. Raises engine-gap **G4**.

#### W13 `monitor-fleet` — observe fleet state without mutating it
*Units (8):* BU-P6-101 BU-P6-103 BU-P6-104 BU-P6-105 BU-P7-099 BU-P7-100 BU-P7-101
BU-P8-072
Stages: `00-snapshot` (D — bounded, constant-size, versioned, strictly read-only;
`busy:true` only with a verified witness, otherwise `busy:null`, BU-P6-101, BU-P7-101)
· `10-evaluate-liveness` (D — identity plus recent meaningful progress with a defined
fallback chain; a stalled live worker records a non-terminal diagnostic, never an
automatic kill, BU-P8-072) · `20-reconcile-terminal` (D — a `done` status with an
empty result is refused as completion and marked orphaned; terminal recycling is
identity-bound and settles the lease first, BU-P6-103/104/105, BU-P7-100) ·
`30-background-watch` (D — idempotent start, failed-start detection, stale-unit
cleanup, graceful on unsupported platforms, BU-P7-099). All survive except
`30-background-watch`, which is **borderline** — it is closer to a deterministic
helper for keeping the observation running than a procedural checkpoint.

#### W14 `wake-and-resume` — resume a waiting worker when its durable condition is met
*Units (6):* BU-P6-096 BU-P6-097 BU-P6-098 BU-P6-100 BU-P7-097 BU-P7-098
Stages: `00-validate-condition` (D — strict field/value allowlist, no dash-leading
values, secret-shaped names screened, before evaluation, BU-P6-097, BU-P7-098) ·
`10-evaluate` (D — six typed kinds; external checks bound to the worker's own recorded
remote, BU-P6-096, BU-P6-100) · `20-classify-outcome` (D — met / unmet / permanently
unsatisfiable→escalate / deadline→failed, BU-P6-098, BU-P7-097) · `30-resume`
(D, BU-P6-096). All survive. This workflow is the direct source of engine-gap **G1** —
the *scheduling* of stage `10` is exactly what no lower rung can own.

#### W15 `reconcile-and-cleanup-fleet` — retire a completed task's surfaces and state
*Units (10):* BU-P6-135 BU-P6-136 BU-P6-137 BU-P6-140 BU-P6-141 BU-P6-142 BU-P7-081
BU-P8-026 BU-P8-027 BU-P8-092
Stages: `00-require-terminal` (A — every targeted repo safely terminal, owning task
verifiably closed, "not closed" distinguished from "could not be looked up",
BU-P6-135/136, BU-P8-092) · `10-verify-ownership` (D — repo identity, not path;
retry-owner spoofing vectors rejected, BU-P6-137, BU-P7-081) · `20-verify-handshakes`
(D — acknowledgement verified, re-verified under lock immediately before deletion,
terminal seal written, BU-P8-026, BU-P8-027) · `30-remove-surface` (D — resumable
cleanup-phase record published before and after; no process cwd inside the surface,
BU-P6-140, BU-P6-142) · `40-retire-state` (D — whole-task state retired only when
every repo is cleaned together, BU-P6-141). All survive.

#### W16 `route-review-findings` — turn independent review output into tracked work and a gate
*Units (6):* BU-P6-082 BU-P6-084 BU-P6-085 BU-P6-086 BU-P7-063 BU-P7-064
Stages: `00-parse-and-sanitize` (D, BU-P6-082) · `10-retain-artifact` (D — a
sanitized copy written to durable storage *before* any external side effect; the
failure diagnostic names the retryable next action, BU-P6-084) · `20-route-each`
(D — dedup marker scoped to axis+source+id+parent+branch; a divergent stored body is
refused untouched, BU-P6-085/086, BU-P7-063/064) · `30-publish-or-clear-gate`
(A — only after every finding reached tracked work, BU-P6-082). All survive.

#### W17 `deliver-external-callback` — durable at-least-once notification to a registered consumer
*Units (5):* BU-P6-117 BU-P6-120 BU-P6-121 BU-P6-122 BU-P7-067
Stages: `00-enqueue` (D — identity hashed from (type, source); repeat enqueue returns
the existing event, BU-P6-120) · `10-drain-and-retry` (D — one event claimed at a
time under a lock, stale claims reclaimable, bounded exponential backoff,
BU-P6-121, BU-P6-117) · `20-validate-acknowledgement` (D — strict versioned schema;
malformed/oversized never counts as success, BU-P6-117, BU-P7-067) · `30-seal`
(D — no cleanup while any event is unacknowledged; sealed history retired, not
deleted, BU-P6-122). All survive. Raises engine-gap **G3**.

### Group D — validation and shipping

#### W18 `validate-and-ship` (no-mistakes) — the single final shipping boundary
*Purpose:* validate a committed change through the pipeline to a terminal outcome,
routing every finding, without the validating actor ever editing the code.
*Trigger:* implementation, native tests, lint and independent review are complete
and the coordinator has reached the approved shipping boundary.
*Units (72):* BU-P1-042 BU-P1-043 BU-P1-069 BU-P1-070 BU-P1-072 BU-P1-074 BU-P1-075
BU-P1-076 BU-P1-077 BU-P1-079 BU-P1-080 BU-P2-057 BU-P2-058 BU-P2-059 BU-P2-060
BU-P2-061 BU-P2-062 BU-P2-063 BU-P2-064 BU-P2-065 BU-P2-066 BU-P2-067 BU-P2-068
BU-P2-069 BU-P2-070 BU-P2-071 BU-P2-072 BU-P2-073 BU-P2-074 BU-P2-075 BU-P2-076
BU-P2-077 BU-P2-078 BU-P2-079 BU-P2-080 BU-P2-081 BU-P2-082 BU-P2-083 BU-P2-084
BU-P2-085 BU-P2-086 BU-P2-087 BU-P2-088 BU-P2-089 BU-P2-090 BU-P2-091 BU-P2-092
BU-P2-093 BU-P2-094 BU-P2-095 BU-P2-096 BU-P2-097 BU-P2-098 BU-P2-099 BU-P2-100
BU-P2-103 BU-P6-023 BU-P6-026 BU-P6-042 BU-P6-044 BU-P6-129 BU-P6-130 BU-P6-133
BU-P6-134 BU-P7-065 BU-P7-104 BU-P7-105 BU-P8-082 BU-P8-084 BU-P8-085 BU-P8-087
BU-P8-089

| Stage | Kind | Durable outcome | Survives? |
|---|---|---|---|
| `00-verify-readiness` | D | A published readiness marker asserts the exact intent revision, the exact reviewed head, and an explicit pass on every review axis; any mismatch refuses with its own reason (BU-P6-130, BU-P8-082) | Yes |
| `10-acquire-launch-reservation` | D | An identity-checked reservation for the exact task/repo pair; concurrent attempts fail closed until the owner exits or stale ownership is proven (BU-P8-084, BU-P6-129) | Yes |
| `20-reserve-isolated-snapshot` | D | Validation runs against an isolated snapshot pinned at the reviewed commit with a clean tree, re-verified immediately before invocation (BU-P6-133, BU-P6-044) | Yes |
| `30-select-intent-transport` | A | The transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run (BU-P6-134, BU-P8-085, BU-P8-087, BU-P7-105, BU-P6-042) | Yes |
| `40-start-run` | D | A run exists on a feature branch with committed history, a verbatim intent, an initialized repo and a runnable pipeline agent; an in-flight matching run is reattached, never duplicated (BU-P2-059/060/061/062/063/064/065/066/067/068/069/070/071/072/073/074, BU-P1-069/070, BU-P1-042) | Yes |
| `50-drive-gates` | A | Every gate resolved by exactly one response; `ask-user` findings relayed verbatim and never resolved autonomously; the actor never edits the pipeline-owned worktree, aborts, or reruns to escape a gate (BU-P2-075/076/077/078/079/080/081/082/083/084/085/098/099/100/103, BU-P1-074/075/076) | Yes — **this is the judgment stage of the whole corpus** |
| `60-route-findings` | D | Every actionable finding becomes one deduplicated owning-repo task with a deterministic severity→priority mapping; correctness/security/data-integrity/test findings can never be deferred or ignored; no finding is fixed inside the run (BU-P6-023, BU-P6-026, BU-P1-080, BU-P7-065) | Yes |
| `70-reconcile-custody` | A | The structured branch-sync state is processed rather than improvised: sync / continue / recover-custody, never reset, stash, force or branch replacement (BU-P2-089/090/091/092/093/094, BU-P1-079) | Yes |
| `80-close-out` | A | Stop driving at `checks-passed`; on `failed`/`cancelled`, fix on the same branch and re-drive; summarize what the pipeline found and fixed (BU-P2-086/087/088/095/096/097, BU-P1-077, BU-P1-043) | Yes |
| `90-handover-log` | D | Every ownership transfer appended to an owner-only log; release tokens single-use (BU-P8-089, BU-P7-104) | Yes |

**U2 verdict.** §6.3's test *does* discriminate cleanly on `no-mistakes`, but only
after the source's flat command list is split by outcome. Ten stages survive; the
things that failed the test and became helpers are the *commands* — `axi`,
`axi status`, `axi logs`, `axi abort`, `axi sync --check` (BU-P2-101), the output
grammar (BU-P2-102), the `--intent-file`/`--intent` choice as a flag (BU-P1-071), and
the branch-sync decision table (BU-P1-078). A command is not a stage; "custody of the
branch is reconciled" is. Two entry variants share the same stage list: coordinator-
launched (starts at `00`) and directly invoked (`/no-mistakes`, starts at `40`, with
`10-check-scope`/`20-do-the-work` from BU-P2-058/059/060/061 preceding it in
task-first mode). Both stress the same boundaries, which is itself evidence the
boundaries are real.

#### W19 `repo-release-verification` — the source repository's own pre-push gate
*Units (2):* BU-P6-007 BU-P6-008. Single stage `release-verification` (D): the drain
suite must pass before every push; missing tooling fails closed rather than silently
skipping. Survives §6.3 by name — it is the proposal's own worked example. Scoped as
self-hosting behavior of the source repo, not a Sergeant-offered procedure.

### Group E — engineering-discipline workflows (the `.agents/skills` corpus)

#### W20 `diagnose-bug` (29): BU-P2-019 BU-P2-021 BU-P2-022 BU-P2-023 BU-P2-025 BU-P2-026 BU-P2-027 BU-P2-028 BU-P2-029 BU-P2-030 BU-P2-031 BU-P2-032 BU-P2-033 BU-P2-034 BU-P2-035 BU-P2-036 BU-P2-037 BU-P2-038 BU-P2-039 BU-P2-040 BU-P2-041 BU-P2-042 BU-P2-043 BU-P2-044 BU-P2-045 BU-P2-046 BU-P2-047 BU-P2-048 BU-P2-049
*Trigger:* "diagnose"/"debug this", or something reported broken, throwing, failing, slow.
Stages: `10-build-feedback-loop` (A — a named, already-run, red-capable, deterministic,
fast, agent-runnable command exists, or the run stops and asks for access/artifacts;
BU-P2-021/022/023/025/026/027/028/029/030) · `20-reproduce-and-minimize` (A —
the loop goes red on the user's exact symptom and every remaining element is
load-bearing; BU-P2-031/032/033/034/035/036) · `30-hypothesize` (A — 3–5 ranked
falsifiable hypotheses shown to the user; BU-P2-037/038/039) · `40-instrument`
(A — one probe per prediction, one variable at a time, tagged logs; BU-P2-040/041/042/043)
· `50-fix-with-regression-test` (A — test at a correct seam before the fix, or the
seam's absence recorded as the finding; BU-P2-044/045/046/047) · `60-cleanup-and-postmortem`
(A — repro gone, test passing, instrumentation removed, hypothesis recorded, architectural
hand-off if warranted; BU-P2-048/049). All six survive: each is a boundary whose failure
rate and cost an operator would want measured separately. §8.2's "strong low-ambiguity
reference workflow" assessment holds.

#### W21 `prototype` (26): BU-P3-010 BU-P3-011 BU-P3-012 BU-P3-013 BU-P3-014 BU-P3-019 BU-P3-020 BU-P3-021 BU-P3-022 BU-P3-023 BU-P3-024 BU-P3-025 BU-P3-026 BU-P3-027 BU-P3-028 BU-P3-029 BU-P3-030 BU-P3-031 BU-P3-032 BU-P3-033 BU-P3-034 BU-P3-035 BU-P3-036 BU-P3-037 BU-P3-038 BU-P3-039
Stages: `00-select-branch` (A — which question type; heuristic fallback recorded when
the user is unreachable; BU-P3-012/013/014) · `10-record-question` (A, BU-P3-021) ·
`20L-build-logic` / `20U-build-variants` (A — the conditional pair; BU-P3-020/022/023/024/025/026
and BU-P3-029/030/031/032/033/034) · `30-hand-off` (A, BU-P3-035) · `40-capture`
(A — validated decision folded into real code and rewritten to production standards;
throwaway preserved on a throwaway branch; BU-P3-019/027/028/036/037/039). All survive.
The A/U branch is the corpus's cleanest evidence for *conditional* procedure —
representable today as one selection stage plus mutually-exclusive downstream stages,
so it is **grammar pressure, not an engine gap** (see G6's ranking discussion).

#### W22 `tdd` (8): BU-P2-104 BU-P2-105 BU-P2-109 BU-P2-110 BU-P2-113 BU-P2-114 BU-P2-115 BU-P2-116
Stages: `00-agree-seams` (A — seams written down and confirmed with the user; no test
at an unconfirmed seam; BU-P2-109/110) · `10-red-green-cycle` (A — one seam, one test,
one minimal implementation, vertical slices only; BU-P2-113/114/115). Refactoring is
explicitly *not* a stage of this workflow (BU-P2-116 hands it to W24). The bulk of the
`tdd` source is reference guidance, not procedure — 16 units land in shared context (§3).

#### W23 `implement` (6): BU-P2-050 BU-P2-051 BU-P2-052 BU-P2-053 BU-P2-054 BU-P2-055
Stages: `10-implement-with-tdd` (A, delegates to W22) · `20-verify` (D — typecheck and
focused tests during, full suite once at the end; BU-P2-053) · `30-review`
(A, delegates to W24) · `40-commit` (D, BU-P2-055). Explicit-invocation-only
(BU-P2-051, and its cross-harness mirror BU-P3-004 in §3).

#### W24 `code-review` (14): BU-P2-001 BU-P2-002 BU-P2-003 BU-P2-004 BU-P2-006 BU-P2-007 BU-P2-009 BU-P2-010 BU-P2-013 BU-P2-014 BU-P2-015 BU-P2-016 BU-P2-017 BU-P2-018
Stages: `00-pin-fixed-point` (A/D — the point resolves and the diff is non-empty, or
this fails here rather than inside a sub-review; BU-P2-004/006) · `10-identify-spec-source`
(A — fixed priority order ending in asking the user; BU-P2-007) · `20/30-parallel-review`
(A ×2, isolated contexts; BU-P2-003/001/002/009/010/013/014/015) · `40-aggregate`
(A — two axes reported separately, never merged or reranked; BU-P2-016/017/018).
All survive. The two-axis separation is the durable design point (BU-P2-018), not the
sub-agent mechanism.

#### W25 `deepen-module` (11): BU-P4-013 BU-P4-014 BU-P4-015 BU-P4-016 BU-P4-017 BU-P4-020 BU-P4-021 BU-P4-022 BU-P4-023 BU-P4-024 BU-P4-026
Stages: `00-classify-dependencies` (A — four-way classification determining whether a
port is needed at all; BU-P4-014/015/016/017) · `10-design-it-twice` (A — ≥3
independently generated, structurally different designs, each under a distinct
constraint, compared on depth/locality/seam placement, ending in an opinionated
recommendation; BU-P4-022/023/024/026) · `20-test-at-new-interface` (A — old
shallow-module tests deleted, new tests assert through the interface only;
BU-P4-020/021). All survive.

#### W26 `resolving-merge-conflicts` (6): BU-P3-045 BU-P3-046 BU-P3-047 BU-P3-048 BU-P3-049 BU-P3-050
Stages: `00-assess-state` (D, BU-P3-046) · `10-research-intent` (A, BU-P3-047) ·
`20-resolve-hunks` (A — preserve both intents or pick with the trade-off recorded;
never invent behavior; never abort; BU-P3-048) · `30-validate` (D — typecheck, tests,
format in that order; BU-P3-049) · `40-finish` (D, BU-P3-050). All survive.

#### W27 `research` (5): BU-P3-040 BU-P3-041 BU-P3-042 BU-P3-043 BU-P3-044
Stages: `00-investigate` (A — primary sources only, every claim traced; BU-P3-042) ·
`10-write-findings` (D — one Markdown file, every claim cited, placed per the repo's
convention or an explicitly stated choice; BU-P3-043/044). Delegated to a background
execution context (BU-P3-041) — that delegation is a *scheduling* property, not a
stage.

#### W28 `grilling` (5): BU-P3-005 BU-P3-006 BU-P3-007 BU-P3-008 BU-P3-009
Stages: `00-interview-loop` (A — one question at a time, waiting for each answer;
discoverable facts looked up rather than asked; each question carries a recommended
answer; BU-P3-006/007/008) · `10-confirm-understanding` (A — an explicit user
confirmation gate before any action; BU-P3-009). Both survive. **Note the direct
tension with engine-gap G5** — the same "one question at a time, wait for the answer"
shape is classified here as ordinary actor procedure and in P5 as an engine gap
(conflict **X8**).

#### W29 `grill-with-docs` (2): BU-P3-001 BU-P3-003
A composition workflow: runs W28's interview while using the domain-modeling
discipline to capture ADRs/glossary entries as decisions land. Stages are W28's plus
a `capture-decisions` (D) checkpoint. Explicit-invocation-only (BU-P3-002, §3).
This is the corpus's cleanest example of **workflow composition without nesting** —
representable today by inlining, which is why it does *not* raise an engine gap.

#### W30 `triage` (29): BU-P3-051 BU-P3-052 BU-P3-054 BU-P3-055 BU-P3-056 BU-P3-057 BU-P3-058 BU-P3-060 BU-P3-061 BU-P3-062 BU-P3-063 BU-P3-064 BU-P3-065 BU-P3-066 BU-P3-067 BU-P3-068 BU-P3-069 BU-P3-070 BU-P3-071 BU-P3-072 BU-P3-073 BU-P3-074 BU-P3-075 BU-P3-089 BU-P3-090 BU-P3-091 BU-P3-092 BU-P3-093 BU-P3-096
Stages: `00-show-attention` (D — three fixed buckets, oldest first; BU-P3-062/063/064)
· `10-gather-context` (A — item and prior notes read; already-implemented check and
out-of-scope-KB concept match run; BU-P3-065/089) · `20-verify` (A — the claim
reproduced or the PR diff tested, reported as confirmed/failed/insufficient;
BU-P3-067) · `30-recommend` (A — category/state proposal, then wait for direction;
BU-P3-066) · `40-grill-if-underspecified` (A — delegates to W28/domain-modeling;
BU-P3-068) · `50-apply-outcome` (A — the terminal disposition with its required
artifact; BU-P3-069/070/071/072/074/090/091/092/093/096) · `resume` and
`quick-override` as re-entry variants (BU-P3-075, BU-P3-073). The state vocabulary
units (BU-P3-054/055/056/057/058) are the *outcome* names of `50`, and the transition
graph (BU-P3-060) is the workflow's own shape. Trigger BU-P3-061; PR variant BU-P3-052;
definition BU-P3-051. **BU-P3-060 is non-linear** (loops, maintainer override at any
point) — the extractor explicitly considered and rejected an engine-gap claim; that
rejection is upheld here, because each transition is a fresh invocation of a stage,
not a control-flow construct the runtime must own.

#### W31 `to-spec` (5): BU-P4-050 BU-P4-051 BU-P4-052 BU-P4-053 BU-P4-054
Stages: `00-gather-context` (A — synthesis only, never an interview; BU-P4-050/051) ·
`10-sketch-seams` (A — fewest new seams, highest possible seam, confirmed with the
user; BU-P4-052/053) · `20-write-and-publish` (D — fixed template, published to the
tracker with the ready label; BU-P4-054).

#### W32 `to-tickets` (8): BU-P4-058 BU-P4-064 BU-P4-065 BU-P4-068 BU-P4-070 BU-P4-071 BU-P4-072 BU-P4-073
Stages: `00-load-project-context` (A, BU-P4-064) · `10-extract-decisions-and-unknowns`
(A — an investigation ticket only for a genuinely blocking unknown, naming the exact
artifact it must produce; BU-P4-065) · `20-confirm-breakdown` (A — granularity,
ownership and blocking edges confirmed unless immediate publication was requested;
BU-P4-068) · `30-publish` (D — new tickets stay open; cross-repo blockers recorded as
counterpart ids plus merge order; BU-P4-070/071) · `40-report-frontier` (A — one
worker per owning repo as the default; reporting is not authorization to dispatch;
BU-P4-072/073). All survive.

#### W33 `wayfinder` (17): BU-P4-075 BU-P4-076 BU-P4-085 BU-P4-086 BU-P4-087 BU-P4-088 BU-P4-089 BU-P4-091 BU-P4-092 BU-P4-093 BU-P4-094 BU-P4-095 BU-P4-096 BU-P4-097 BU-P4-098 BU-P4-099 BU-P4-100
Stages: `00-name-destination` (A — via a grilling/domain-modeling session; scope
settled first; BU-P4-094) · `10-map-frontier` (A — breadth-first; stop and do not
create a map if no fog exists; BU-P4-094/095/088/089/091) · `20-create-tickets`
(D — specifiable decisions as child issues first, blocking edges wired in a second
pass; BU-P4-096) · `30-resolve-one` (A — claim, resolve by type, record the answer as
a resolution and a one-line pointer; at most one non-research ticket per session;
BU-P4-098/085/086/087/093/099/092) · `40-regraduate-fog` (A — the loop back to `10`;
BU-P4-076/097/100). Stages survive; the *loop* between `30` and `40` is what raises
engine-gap **G7** — which is **rejected** in §5.

#### W34 `vet-external-skill` (9): BU-P1-119 BU-P1-120 BU-P1-121 BU-P1-122 BU-P1-123 BU-P1-124 BU-P1-125 BU-P1-126 BU-P1-127
Six ordered actor/deterministic checkpoints, one per fixed vetting step
(BU-P1-120…BU-P1-125), plus update variants for managed and owned skills
(BU-P1-126, BU-P1-127) and the workflow definition (BU-P1-119). Each step's outcome
("the source was read", "the actions were checked", "it was tested in a disposable
copy") survives any reimplementation of *how* the checking is done. Strong candidate
for the smallest complete reference workflow in the corpus.

#### W35 `wiki-digest` (16): BU-P5-130 BU-P5-131 BU-P5-137 BU-P5-138 BU-P5-139 BU-P5-140 BU-P5-141 BU-P5-142 BU-P5-143 BU-P5-145 BU-P5-146 BU-P5-147 BU-P5-148 BU-P5-149 BU-P6-092 BU-P6-093
Stages: `00-read-schema` (D — the schema is read before any behavior change; a missing
schema stops the run before any page is written; BU-P5-137/145) · `10-dry-run`
(D — always first when regenerating or changing logic; BU-P5-138, BU-P6-093) ·
`20-inspect-preview` (A — secrets, duplicate entities, wrong outcomes, unresolved
errors; a secret stops the run and only the source *class* is recorded;
BU-P5-139/146/147) · `30-generate` (D — synthesis, never a transcript; collected from
every configured source with unavailable ones silently skipped; BU-P5-140/143, BU-P6-092)
· `40-publish-and-index` (D — the page exists and is linked, or the page is kept, its
path reported, and the digest marked incomplete; an existing page is never overwritten
with less information; BU-P5-141/148/149) · `50-log-ingest` (D, BU-P5-142). Workflow
definition and trigger: BU-P5-130/131. All survive. P5's `wiki` and P6's
`wiki-daily-digest` are the same procedure (conflict **X9b**, folded).

---

## 2. Permanent-instruction set (constitution outline)

103 units classified `agents-invariant`, deduplicated into **nine articles**. Each
article states the rule once; the unit ids are the evidence behind it. Two units
originally in this set are moved to §7 (unassigned).

### Article I — Resolve before acting
Establish ownership from resolved project context, never from the working directory
or from inference; the layered instruction set governs before the first mutation.
*BU-P1-001, BU-P8-052.*

### Article II — Roles and execution mode
The primary session coordinates by default; direct execution requires an explicit
request and a single owning repository. Never use the coordinator role as a reason to
stop at a plan when an implemented outcome was requested; a plan, task, finding, or
worker launch is not the outcome unless that is what was asked for.
*BU-P1-002, BU-P1-015, BU-P1-044.*

### Article III — Authority boundaries and ownership
The shipping gate is coordinator-owned; workers and remediation loops never run it,
and the gate is a final boundary, not an implementation loop. Validation agents never
modify source while reporting findings. Never modify configuration repositories.
Standing authorization removes repetitive confirmation only — never risk acceptance,
gate skipping, force operations, secret exposure, or destruction of preserved state.
*BU-P1-040, BU-P1-050, BU-P1-054, BU-P1-068, BU-P1-073, BU-P1-110, BU-P1-111,
BU-P1-131, BU-P8-083, BU-P8-091, BU-P8-100.*
**Contested:** the enforceability of the worker restriction — see conflicts **X4** and
engine-gap **G8** (rejected).

### Article IV — Evidence over optimism; fail closed, never fail silent
Do not infer progress from liveness. Do not rewrite an expected blocked exit as
orphaned. Do not clean a waiting surface. A live process is not proof of work; a
successful turn is not completion — a terminal state needs its substantiating
artifact. Tool absence produces an actionable fallback or an explicit blocker, never a
silent skip, false success, or indefinite wait. Ambiguity about identity or ownership
refuses rather than guesses.
*BU-P1-037, BU-P1-047, BU-P1-049, BU-P6-053, BU-P6-055, BU-P6-061, BU-P6-102,
BU-P7-011, BU-P8-072†, BU-P8-078, BU-P8-088, BU-P8-090, BU-P8-094, BU-P8-096,
BU-P8-097, BU-P8-104, BU-P8-105, BU-P8-106, BU-P8-107.* († also cited in W13.)

### Article V — Measured, not assumed
Capability is discovered by probing the installed thing's own surface, never inferred
from a version number or a name. A harness that is not installed is recorded as
*unmeasured*, which is not a claim it cannot do the thing. A pinned model tuple the
harness cannot honor is a terminal failure, never a silent fallback to an ambient
default. One declaration drives gate, probe, and invocation — never three.
*BU-P1-059, BU-P6-046, BU-P6-066, BU-P6-108, BU-P8-046, BU-P8-061.*

### Article VI — Secrets, privacy, and transport
Never commit secrets. No delivered content — briefs, responses, intent bodies,
prompts — appears in process arguments. Recording provenance never records the content
itself. Documentation examples carry no real credentials, private names, or bodies.
Callback executables come only from a fixed pre-installed location, never from a
request or from configuration, and callback records carry no platform ids, tokens, or
message bodies.
*BU-P1-055, BU-P6-038, BU-P6-045, BU-P6-047, BU-P8-006, BU-P8-009, BU-P8-011,
BU-P8-018, BU-P8-031, BU-P8-065, BU-P8-086.*
**Contested:** BU-P8-065's absolute against BU-P6-047/BU-P8-086's consent-gated
exception — conflict **X18**.

### Article VII — Instruction and documentation authority
Every directive names a trigger, a required or prohibited action, or the evidence that
proves compliance; vague quality directives are prohibited and must be replaced with
named commands, failure behavior, acceptance criteria, ownership, or review evidence.
Authority is single-owner: always-on execution and safety policy has exactly one
owner; trigger-specific procedure has exactly one owner; configuration fields have
exactly one owner; documentation never forks any of them. A command's own `--help`,
emitted contract, and tests outrank prose, and a disagreement is filed, not silently
resolved.
*BU-P1-017, BU-P1-018, BU-P5-104, BU-P5-123, BU-P7-014, BU-P7-018, BU-P8-002,
BU-P8-003, BU-P8-004, BU-P8-005.*

### Article VIII — Procedure discovery and loading
Load a procedure only when its trigger applies. The repository-local procedure file is
canonical and outranks any same-named registry entry; a registry's omission never makes
a procedure unavailable, and only the exact missing local path is reported — never a
protocol reconstructed from memory. Procedures are executable instructions: review the
source before installing or updating, and never infer provenance from a folder name.
No install step writes to a user's global agent configuration.
*BU-P1-019, BU-P1-020, BU-P1-021, BU-P1-022, BU-P1-023, BU-P1-024, BU-P1-056,
BU-P1-112, BU-P1-114, BU-P1-115, BU-P1-116, BU-P1-118, BU-P1-128.*

### Article IX — Scope, deployment model, and delivery discipline
One installation per developer; no central tenancy, org RBAC, shared credentials,
cross-machine leases, or team-wide fleet database; not a replacement for Git, the
forge, CI, or the tracker; never permission to push to a default branch. Direct-mode
work always uses a feature branch and always opens a PR. No remote-execution contract
exists anywhere in the distribution. Do not duplicate tasks, findings, PRs, workers, or
review passes when a canonical owner exists; do not repeatedly report a blocker whose
remediation is approved; do not leave finished work recorded as in-progress.
*BU-P1-045, BU-P1-046, BU-P1-048, BU-P1-064, BU-P1-098, BU-P1-100, BU-P1-109,
BU-P7-019, BU-P8-001, BU-P8-043, BU-P8-057.*

### Article X — Deferred work, recovery, and installation integrity
Deferred work is a durable waiting state with a recorded condition, never an
in-process sleep; a condition that can no longer be met converts to needs-input with
the remedy stated. Recovery is one-shot and escalates rather than retrying. The whole
worker-execution posture — permission bypass, no interactive prompts — is justified only
because the trust boundary is the reviewed intent, the injected brief, and the surface's
filesystem permissions; it is not itself a capability grant. Ownership of the launch
transport decision is fixed once and honored, never re-optimized at run time. Shared
credentials are never switched globally while unrelated runs are active. Portability and
test-isolation guarantees are proven by running, not by parsing.
*BU-P1-065, BU-P6-028, BU-P6-070, BU-P7-006, BU-P7-020, BU-P7-024, BU-P7-033,
BU-P7-071, BU-P7-077, BU-P7-080, BU-P8-067, BU-P8-068, BU-P8-074, BU-P8-076,
BU-P8-080, BU-P8-081, BU-P8-101, BU-P8-108.*
**Contested:** BU-P7-033/BU-P7-071/BU-P1-065's Bash-3.2 target against BU-P8-102's
obsolescence ruling — conflict **X5**. BU-P8-108's cross-filesystem refusal against
BU-P7-079 — conflict **X1**.

**Sizing note.** 103 units → 10 articles → roughly 45 distinct sentences. That is a
plausible `AGENTS.md`. The compression came almost entirely from Articles IV, VI and
VIII, where the same rule was restated in `AGENTS.md`, `README.md`, `docs/`, and a
test, each of which the extractors correctly recorded separately.

---

## 3. Shared context and helper candidates

248 units. Split into (a) reused *guidance* an actor reads and (b) reused *mechanics*
an actor invokes, per §6.6's test — belongs to one workflow → local; several with the
same contract → `.sergeant/common/`.

### 3a. Named shared contexts already conventionalized in the source (`@@name`)
The P1 extractor found twelve already behaving as `@@`-style shared context. These are
the ready-made seed for §7.5's convention.

| `@@name` | Units | Consumed by |
|---|---|---|
| `@@project` | BU-P1-101 | W1, W3, W5, W7, W8 |
| `@@repository` | BU-P1-102 | W1, W7, W8, W15 |
| `@@task` | BU-P1-103 | W5, W8, W9, W16, W32 |
| `@@fleet` | BU-P1-104 | W8, W12, W13, W15 |
| `@@worker` | BU-P1-105 | W8, W9, W10, W11, W12, W13 |
| `@@decision-request` | BU-P1-106 | W5, W8, W10, W18 |
| `@@review-axes` | BU-P1-085 | W8, W9, W16, W18, W24 |
| `@@launch-record` | BU-P1-091, BU-P1-092 | W8, W13 |
| `@@drain-admission-lock` | BU-P1-097 | W8, W10, W11, W12 |
| `@@installation-ownership-boundary` | BU-P1-099 | W3, W1 |
| `@@skill-locations` | BU-P1-113 | W3, W8, W34 |
| `@@worker-brief-skill-bundle` | BU-P1-129, BU-P1-130 | W8, W9 |

### 3b. Shared guidance (`.sergeant/common/context/`) — reused across workflows

| Candidate | Units | Shared by |
|---|---|---|
| `project-configuration` — project identity is the filename; `dev_root`; three-layer instruction order (defaults→group→repo, later wins, never structurally merged); groups; path forms | BU-P1-051 BU-P1-052 BU-P1-053 BU-P1-066 BU-P1-067 BU-P7-001 BU-P7-004 BU-P8-029 BU-P8-032 BU-P8-033 BU-P8-037 BU-P8-030 BU-P8-040 BU-P6-020 | W1, W2, W3, W5, W7, W8 |
| `worker-state-vocabulary` — the seven durable worker states, their meanings and required operator actions; nonterminal vs terminal; waiting semantics | BU-P1-035 BU-P1-036 BU-P8-073 BU-P8-098 | W8, W9, W10, W11, W12, W13, W14, W15 |
| `wake-conditions` — the six typed condition kinds with their required fields and resume rules | BU-P8-075 | W9, W14 |
| `intent-provenance` — one canonical intent revision governs implementation, review, PR text, successors, recovery, and validation; the eight required sections | BU-P1-039 BU-P1-041 BU-P6-049 | W8, W9, W16, W18 |
| `review-severity-and-axes` — the canonical severity vocabulary with its reviewer-spelling aliases; the canonical axis vocabulary from one source both ends consult; only error-family blocks | BU-P6-024 BU-P6-083 BU-P7-061 BU-P7-062 | W8, W9, W16, W18, W24 |
| `response-evidence-schema` — the four archived fields; atomic staged publication; one parser | BU-P6-033 BU-P6-051 BU-P6-139 | W10, W15 |
| `callback-protocol` — the four event classes, the invocation contract, the three acknowledgement shapes and their effects, the coverage requirement | BU-P7-068 BU-P8-014 BU-P8-019 BU-P8-021 BU-P8-022 | W15, W17 |
| `launch-evidence` — durable, unfalsifiable, credential-free launch evidence derived from the validated tuple, never the ambient environment | BU-P6-109 | W8, W13 |
| `skill-discovery` — the canonical tree, the mirrored discovery paths, the two configured roots, repository-local precedence, the coordinator-only vendoring of the gate skill | BU-P1-117 BU-P6-006 BU-P7-015 BU-P7-026 BU-P7-030 BU-P7-031 BU-P7-032 | W3, W8, W34 |
| `codebase-design-vocabulary` — module/interface/implementation/depth/seam/adapter/leverage/locality, the deletion test, depth-as-leverage, the two-implementation rule for seams, the terms to avoid | BU-P4-001 BU-P4-002 BU-P4-003 BU-P4-004 BU-P4-005 BU-P4-006 BU-P4-007 BU-P4-008 BU-P4-009 BU-P4-010 BU-P4-011 BU-P4-012 | W20, W22, W24, W25, W31 |
| `domain-modeling` — glossary discipline, terminology conflict surfacing, edge-case invention, cross-checking claims against code, ADR criteria, what not to record | BU-P4-027 BU-P4-028 BU-P4-031 BU-P4-032 BU-P4-033 BU-P4-034 BU-P4-035 BU-P4-037 BU-P4-042 BU-P4-043 BU-P4-046 BU-P4-049 | W25, W28, W29, W30, W31, W33 |
| `test-quality` (the `tdd` reference half) — behavior through public interfaces, seams, implementation-coupled and tautological anti-patterns, mocking only at system boundaries, SDK-shaped injectable dependencies, good/bad test red flags | BU-P2-106 BU-P2-107 BU-P2-108 BU-P2-111 BU-P2-112 BU-P2-117 BU-P2-118 BU-P2-119 BU-P2-120 BU-P2-121 BU-P2-122 BU-P2-123 BU-P2-124 BU-P2-125 BU-P2-126 BU-P2-127 | W20, W21, W22, W23, W25 |
| `ticket-shaping` — vertical slices, one-session sizing, one owning repo, no duplicates, observable acceptance criteria, no horizontal splits, update-don't-duplicate, superseding notes | BU-P4-059 BU-P4-060 BU-P4-061 BU-P4-062 BU-P4-063 BU-P4-066 BU-P4-069 BU-P4-074 | W7, W30, W32, W33 |
| `wiki-conventions` — capture vs curated separation; never raw prompts/bodies/secrets; a missing capture is fixed at its source, never hand-synthesized; the schema-driven synthesis prompt and the candidate-page block; captures are side effects of specific commands | BU-P5-132 BU-P5-133 BU-P5-135 BU-P6-094 BU-P6-095 BU-P8-093 | W8, W10, W15, W35 |
| `triage-state-machine` — one category role and one state role at a time; the AI-authorship disclaimer; the out-of-scope KB's purpose, shape, naming, and durability rule | BU-P3-053 BU-P3-059 BU-P3-083 BU-P3-084 BU-P3-085 BU-P3-086 BU-P3-087 BU-P3-088 | W30, W33 |
| `dispatch-routing-context` — the normalized review-routing context built from mission, role, group and merged instructions | BU-P8-038 | W8, W16 |
| `recovery-visibility` — committed work above the recorded base must be surfaced by every path, never reported as "no worktree, no pane" | BU-P7-091 | W8, W11, W13, W15 |
| Workflow-local contexts (kept local per §6.6) | `code-review`: BU-P2-008 · `no-mistakes`: BU-P2-102 · `diagnose-bug`: BU-P2-020 · `prototype`: BU-P3-015 BU-P3-016 BU-P3-017 BU-P3-018 · `deepen-module`: BU-P4-018 BU-P4-019 BU-P4-025 · `wayfinder`: BU-P4-077 BU-P4-080 | one workflow each |

### 3c. Shared mechanics (`.sergeant/common/bin/`) — reused deterministic helpers

| Candidate | Units | Shared by |
|---|---|---|
| `finding-router` — disposition→priority mapping, dedup markers, rerun/resurface semantics, label preservation, revision digests and superseded-revision preservation, sanitized artifact retention and retry | BU-P1-081 BU-P1-082 BU-P1-083 BU-P1-084 BU-P1-086 BU-P1-087 BU-P1-088 BU-P1-089 BU-P1-090 BU-P6-025 BU-P6-087 | W8, W9, W16, W18 |
| `mutual-exclusion` — hard-link locking chosen for portability, owner record published atomically with the lock, crash-safe reclamation bound to a proved-dead instance by nonce, diagnosable timeout reports | BU-P6-052 BU-P6-059 BU-P6-060 BU-P7-085 | W8, W10, W11, W12, W17, W18 |
| `owned-write` — stage to a private candidate, verify filesystem identity, publish by atomic rename, record the owned path; identity-sensitive reads accept only a regular mode-600 owner-owned file | BU-P6-132 BU-P7-056 BU-P7-057 BU-P7-054 BU-P7-055 | W8, W10, W15, W18 |
| `process-identity` — PID-reuse detection by recorded start time, process-group leadership checks, fail-closed refusal rather than guessing which processes are safe | BU-P6-040 BU-P6-138 BU-P7-082 | W11, W12, W13, W15 |
| `action-lease` — record an outcome exactly once and never overwrite it; reentrant settlement with futile-wait detection deferred to the exit boundary; release semantics and ownership safety | BU-P6-054 BU-P6-056 BU-P7-045 BU-P7-046 BU-P7-050 | W9, W10, W11, W12, W15 |
| `harness-registry` — one declaration driving capability gate, readiness probe and launch arguments; every row validated before the gate is trusted; probe independent of UI strings | BU-P6-005 BU-P6-065 BU-P6-069 BU-P7-089 BU-P7-090 | W3, W8, W10, W11 |
| `capability-probe` — verify support by probing the tool's own help surface for the exact flags needed, never a version string | BU-P6-004 BU-P7-027 BU-P7-034 BU-P7-103 BU-P8-060 | W3, W8, W18 |
| `intent-intake` — path safety (no newlines, traversal, symlink components, size, control characters) checked before content is read | BU-P6-050 | W8, W18 |
| `payload-safety` — size/line/control-character/metacharacter/secret-shape/platform-id rejection before anything is queued or written | BU-P6-119 BU-P8-017 | W16, W17 |
| `callback-plumbing` — profile path and permission verification at invocation time, origin binding, correlation-id and source-id patterns, event identity tuples, retry/claim/backoff state, scoped and fleet-wide drains, requeue after repair, consumer-side dedup requirement | BU-P6-118 BU-P8-008 BU-P8-010 BU-P8-012 BU-P8-013 BU-P8-015 BU-P8-016 BU-P8-020 BU-P8-023 BU-P8-024 BU-P8-025 BU-P8-028 | W15, W17 |
| `state-resolution-consistency` — every fleet command resolves drain state and identity through the same paths | BU-P7-102 | W8, W10, W11, W12, W13 |
| `identity-precedence` — the fixed four-level forge-identity precedence | BU-P8-039 | W8 |
| `handoff-provenance` — recording refuses unless the caller's path is the exact recorded owned surface | BU-P6-037 | W9, W10, W11 |
| `notify-marker` — one shared per-task marker polled by the watcher, so simultaneous updates collapse into one delayed wakeup rather than duplicate delivery | BU-P8-071 | W10, W13 |
| `install-plumbing` — glob-discovered symlinking of commands and sourced helpers, stale-link removal, safe uninstall, mise discovery order | BU-P6-001 BU-P6-002 BU-P6-009 BU-P7-028 BU-P7-029 | W3, W19 |
| `graphify-plumbing` — repo-name constraints, symlink-preserving publication with wiki/memory retention, Sergeant-side exclusion application and staged extraction, symlink-alias resolution before exclusion | BU-P8-034 BU-P8-035 BU-P8-036 BU-P7-087 | W2 |
| `worktree-pool` — leased pre-warmed surfaces preferred over fresh ones, with fallback and pooled return; the unpushed-work guard's false-positive rule | BU-P5-072 BU-P5-073 BU-P7-076 | W8, W15 |
| `test-infrastructure` (source-repo self-hosting) — the isolation audit's self-test, transitive coverage requirement | BU-P7-022 BU-P7-023 | W19 |
| `explicit-invocation-metadata` — the cross-harness mirror of the no-auto-invoke flag | BU-P3-002 BU-P3-004 | W23, W29 |
| `domain-artifacts` — CONTEXT.md / CONTEXT-MAP.md shape and lazy creation, ADR numbering and minimum viable form, glossary entry rules | BU-P4-029 BU-P4-030 BU-P4-036 BU-P4-038 BU-P4-039 BU-P4-040 BU-P4-041 BU-P4-044 BU-P4-045 BU-P4-047 BU-P4-048 | W25, W29, W30, W31, W33 |
| `expand-migrate-contract` | BU-P4-067 | W32 |
| `branch-sync-decision-table` and pipeline inspection commands | BU-P1-071 BU-P1-078 BU-P2-101 | W18 |
| Workflow-local helpers (kept local) | `code-review`: BU-P2-005 BU-P2-011 BU-P2-012 · `diagnose-bug`: BU-P2-024 · `sergeant-help`: BU-P5-116 BU-P5-118 BU-P5-124 · `to-spec`: BU-P4-055 BU-P4-056 BU-P4-057 · `triage`: BU-P3-076 BU-P3-077 BU-P3-078 BU-P3-079 BU-P3-080 BU-P3-081 BU-P3-082 BU-P3-094 BU-P3-095 · `wayfinder`: BU-P4-078 BU-P4-079 BU-P4-081 BU-P4-082 BU-P4-083 BU-P4-084 · `wiki-digest`: BU-P5-134 BU-P5-136 BU-P5-144 · `troubleshooting`: BU-P8-098 | one workflow each |

---

## 4. Obsolete-mechanism roll-up

28 units, nine clusters. Each names the mechanism the Rust runtime replaced
structurally and the durable policy (if any) that must survive it. Governing
rulings: **D2** (no TTY/pane; headless `claude -p`/`--resume` turns), CLAUDE.md
*One owner*, *the journal is the only truth*, *clients are equal*.

| # | Mechanism cluster | Units | Replaced by | Durable policy that must survive |
|---|---|---|---|---|
| **M1** | **Coordinator-pane binding and identity proof** — managed/named pane modes, verifying the pane against the live server, walking the process ancestry to prove residence, launching the coordinator from inside a session so instructions load | BU-P1-061 BU-P1-095 BU-P6-131 BU-P8-050 BU-P8-063 | Loopback API + bearer auth; the daemon owns all client identity | Never trust a self-asserted execution identity; verify it against the authority that would know, and refuse before creating any durable state. An execution context must have its governing instructions loaded and must be nameable by later operations. |
| **M2** | **Pane as the worker's process** — one tmux window per repo, attach to observe, persistent-interactive-only launch modes, process-group signalling, terminating the whole tree | BU-P5-064 BU-P5-069 BU-P6-112 BU-P7-008 BU-P8-064 BU-P7-106 | Headless per-turn processes owned by the daemon (D2); TUI/dashboard for observation (M6) | Durable session identity is separate from process lifetime. Live observation of in-progress work is a first-class requirement. Terminate the whole tree, never one shell. The harness-uniform lifecycle contract (launch, terminal states, recovery, races) is durable even though its tmux carrier is not. |
| **M3** | **Pane as the notification channel** — tmux injection as the delivery transport, degrade-to-callback when the pane is gone, the ID-bearing nudge retry loop, a reader that displays but never executes | BU-P1-096 BU-P6-029 BU-P8-066 | Journal + SSE/API delivery; clients hold no state | The durable record is the source of truth and a transport failure is not a recording failure. Delivered content is never executable. Initial-work delivery must be idempotent and crash-safe — already met by journal durability plus resumable session identity, so **not** an engine gap. |
| **M4** | **Pane as the liveness signal** — readiness by two consecutive stable renders, readiness defined without UI strings, pane output as the primary progress evidence | BU-P6-067 BU-P6-068 BU-P6-106 | Turn completion is the signal; there is no keystroke to land | A supervisor's liveness is not the wrapped work's progress. Readiness must never depend on a presentation string or an executable name. The two-observations-over-fixed-delay technique is recorded so a future attach-style feature does not re-learn it. |
| **M5** | **Loose worktree files as durable state** — the brief as a plain Markdown drop, pane-identity matching for acknowledgement, legacy on-disk marker migration | BU-P5-063 BU-P6-031 BU-P6-077 | Journaled workflow binding and stage context; journal-as-only-truth | Starting context must be durably and replayably carried before the actor begins. A state transition may only be acknowledged by its verified owner. When migrating durable state, write only absent fields and cross-check every derivation independently. |
| **M6** | **Detached background harness sessions** — stopping a live `--bg` session before relaunch, before force-stop, before response-driven relaunch; coordinator-liveness polling by PID and start time | BU-P6-041 BU-P6-043 BU-P6-074 BU-P6-081 | Headless turns exit with the turn; the daemon is the only long-lived owner | Never run two concurrent processes against one work surface across a relaunch. Every termination path must account for out-of-process-group resources it created. An orphaned run must not block forever on a dead supervisor. |
| **M7** | **Pane-scoped rollback** — killing only a pane this invocation created, disarming the trap once dispatch succeeded | BU-P6-127 | Journaled per-Work ownership | Roll back exactly what this invocation created and can still prove it owns; never touch pre-existing or concurrently replaced state. |
| **M8** | **Response delivery/acknowledgement split** — bounded acknowledgement timeout then exactly one relaunch as the documented recovery | BU-P6-076 | A turn either completes or its process exits; delivery and consumption collapse | Never leave the operator with no supported next action. |
| **M9** | **Shell-distribution targets** — the tmux window title as the stage label; Bash 3.2 runtime proof in a pinned container | BU-P8-062 BU-P8-102 | A compiled binary; the engine's own stage concept | Every dispatched execution carries a named stage — already provided structurally. Parsing proof is not runtime proof. **Contested:** whether the Bash-3.2 target is obsolete (X5). |

**Separation review (§8.2's requirement that the separation itself be reviewed).**
The test applied to each unit was: *remove the mechanism entirely — does a rule remain
that could be violated?* M3, M4, M5, M6, M7 and M8 all pass (a violable rule remains).
M2 passes only partially: "persistent interactive session only" is *not* a durable
policy — it is the mechanism, and D2 reverses it. M1's durable policy is real but is
already stated in Article IV and does not need re-encoding. M9's stage-label policy is
already satisfied by the engine. Net: of 28 mechanism units, **nine** carry a policy
that is not already stated elsewhere in the constitution — the rest are provenance.

---

## 5. Engine-pressure roll-up

16 engine-gap claims, merged into **nine** distinct claims. Six survive §6.7; three
are rejected. Rejections are recorded, not deleted (§8.1).

### Surviving, ranked by evidence strength

#### G1 — Runtime-owned durable wait/wake scheduling · **survives** · rank 1
*Merges:* BU-P6-099, BU-P7-010, BU-P7-096 (mutual duplicates of one seam).
*Behavior:* a Work suspends on a typed external condition, its process exits, and the
runtime resumes it automatically when the condition becomes true — with bounded
jittered retry, a hard deadline that fails the Work, and an attempt ceiling that
escalates to needs-input instead — with nothing polling in between.
*Lower rungs and why they fail:* a polling helper burns a live process and a billed
turn for spans that can be multi-day and does not survive a daemon or host restart; an
actor cannot resume itself after its own process exits; an external scheduler
re-invoking a stateless script relocates the durable fact (attempt count,
next-not-before, deadline) into loose files outside the journal, which is exactly the
anti-pattern §6.7 tests for. All three are **real** failures, not preferences.
*Evidence:* `bin/sgt-wake` + `.sergeant-wake-condition`; `tests/sgt-wake-test.sh`'s
four-part seam; `templates/worker-brief.md`'s six condition kinds and explicit sleep
prohibition; corroborated by BU-P8-074/075/076 and BU-P6-096/097/098/100.
*Adversarial narrowing:* the claim as written asks for "a first-class waiting state" —
but the engine **already** separates `waiting`, `needs_input` and `blocked`
(BU-P5-066 records this explicitly). The gap is *only* the scheduler: a journaled
timer plus a small set of runtime-evaluated predicates plus automatic stage re-entry.
Claim amended down to that.
*Why rank 1:* three independent partitions found it from four different artifact
classes (a command, a template, a test, a doc), the lower-rung failures are mechanical
rather than aesthetic, and the acceptance test is directly executable.

#### G2 — Fleet identity: a durable parent grouping N member Works with dependency edges · **survives** · rank 2
*Merges:* BU-P5-065, BU-P6-016, BU-P6-017.
*Behavior:* one logical cross-repository task durably groups N independent per-repo
Works under one identity, records declared dependency edges between them, exposes
fleet-scoped read views and operations, and admits a dependent only once its
prerequisites reach a terminal successful state — while each member remains
individually retryable and recoverable.
*Lower rungs and why they fail:* one Work with per-repo stages fails on §3.6 (one Git
surface per Work) and §3.5 (one backend per run) — a stage cannot fan out into N
independently-progressing sessions, and its completion is one signal, not N. N
independent Works correlated by a shared string leaves the engine unable to observe or
enforce the edges, and a crash mid-fan-out leaves no journaled record of membership or
intended graph — the exact ambiguity that must fail closed. Actor-issued fan-out is
the hidden-sub-workflow pattern §7.7/§10.3 reject.
*Evidence:* the fleet-state directory keyed by task id; watch aggregating per-repo
state into one live table; `--deps a>b,a>c`; task-scoped cleanup; the same notation
produced by planning (BU-P5-045/044) and consumed by dispatch; and — decisively — the
existence of `sgt-dag-run` / `sgt-dag-dispatch-hook`, which exist *only* to bolt an
external scheduler onto dispatch, with the `dag:` block being pure declarative data
the tool hands to `dagr`. **A lower rung was attempted in production and abandoned in
favour of an external tool.** That is the strongest kind of evidence §6.7 can receive.
*Adversarial narrowing:* BU-P5-074 states plainly that `--deps` enforcement is "left
entirely to the dispatched workers" — so the *enforcement* half of the claim is not
evidenced by `sgt-dispatch`; only *recording* is. The enforcement evidence comes from
the DAG units instead, where an external scheduler genuinely advances stages on
completion. The claim is therefore split in the acceptance test: (a) grouping identity
and fleet-scoped views — evidenced by dispatch; (b) advance-on-completion — evidenced
by the DAG hook. Both are required; they are evidenced by different artifacts.
Conflict **X15** records the internal disagreement.

#### G3 — Durable outbound notification queue independent of any Work's lifetime · **survives with a required amendment** · rank 3
*Claim:* BU-P8-007.
*Behavior:* deliver a deduplicated, at-least-once notification of a Work's
needs-input/blocked/failed/done transition to a registered consumer, retried on a
durable backoff schedule, from a process that is not the Work's, across daemon
restarts — and gate that Work's cleanup on delivery being acknowledged or explicitly
retired.
*Lower rungs and why they fail:* a stage inside the originating Work ends when the
Work's execution ends, but acknowledgement, retry, and the cleanup gate must keep
functioning afterwards; a helper at one transition cannot express a periodic
fleet-wide drain nor safe concurrent claim/backoff state; shared context can hold
configuration but not a per-event mutable state machine written from a wholly separate
process. All three are real.
*Evidence:* the whole documented protocol — per-event pending/delivering/acknowledged/
rejected with attempt count and backoff; a session-independent `drain --all`; cleanup
refusing until an acknowledgement check succeeds; a terminal seal closing the race.
*Required amendment (adversarial):* the claim never attempts the most obvious lower
rung available in *this* engine — **an external subscriber tailing the journal/SSE
stream and owning its own dedup and retry**. That rung genuinely covers delivery and
dedup. What it does *not* cover is the **cleanup gate**: a Work's own retirement being
blocked until an external party acknowledges. So the surviving core is narrower than
claimed: the runtime must own an *acknowledgement gate on terminal cleanup*, not
necessarily the whole delivery queue. The claim must be re-filed at that scope before
it is counted as accepted, and its recorded confidence of `medium` is correct.

#### G4 — Operator-declared, durable, scope-qualified admission block · **survives** · rank 4
*Claim:* BU-P6-063 (with BU-P8-077, BU-P8-078, BU-P6-057/058/062/064 as corroboration).
*Behavior:* refuse new stage/turn admission the instant a drain is set (globally or
per project), never terminate in-flight work, support a bounded wait that converges
when every in-scope turn finishes naturally, and on timeout name precisely what is
still unresolved — with an unverifiable worker blocking the wait rather than being
counted as drained.
*Lower rungs and why they fail:* a stage-local "should I start" helper cannot close an
atomic race against a concurrent dispatch and has no cross-Work visibility;
per-execution cancellation and timeout (§15.5/§15.6) govern one execution's own
lifecycle and have no concept of blocking *new* executions while leaving running ones
alone. Both are real.
*Evidence:* ~1,100 lines of file-based admission control and liveness verification
across `sgt-drain`, `_sgt-drain.sh` and `sgt-drain-force`, plus the interaction rules
every other command must honour (respond stores but holds relaunch, recovery is
refused under drain).
*Adversarial note:* the minimum capability is *small* — a durable flag on daemon state
consulted atomically by every stage-launch path, plus an enumeration of in-flight
executions. This is closer to R2/R5 than R7. It survives because no authored content
can hold it, but it should be ranked as **high evidence, low cost**, which makes it the
best first candidate to implement.

#### G5 — Data-dependent, variable-length human-input rounds inside one procedure · **survives, narrowed** · rank 5
*Claim:* BU-P5-025. This is contract unknown **U3**, and the answer is: *yes, it
produces the first honest `needs_input` finding, but a smaller one than claimed.*
*Behavior:* a procedure must ask N sequential dependent questions where N is
determined by earlier answers, stopping for each answer before formulating the next,
so an early answer can end the interview before later questions are asked.
*Lower rungs and why they fail:* one actor stage per phase fails because a measured
headless turn cannot suspend mid-turn and resume the same reasoning context (D2, M4);
one coarse per-phase round trip cannot express the early-stop the source explicitly
specifies; one stage per question cannot express a round count decided at runtime,
because the stage list is resolved and journaled before execution.
*Adversarial narrowing:* the claim's own minimum capability offers option (a) —
suspend and resume *the same actor context* mid-turn. That is over-asking and is not
required by the evidence. A lower rung the claim never attempts: **a re-enterable
stage** — the stage ends with `needs_input`, the answer is journaled, and the *same
stage* is re-entered as a fresh execution that reads the accumulated answers from
durable state. That is sufficient for every quoted behavior including early stop, and
it needs only one new engine fact: a stage may be re-entered an unbounded number of
times with its prior answers journaled against it. Claim amended to that; option (a)
struck.
*Cross-check:* the same shape appears in W28 (`grilling`, BU-P3-007) and W31
(BU-P4-053) and was classified there as ordinary actor procedure. That inconsistency
is conflict **X8**; the narrowed claim resolves it, because a re-enterable stage also
covers those two cases without either being a gap.

#### G6 — Conditional invocation of a named child procedure with its own checkpoints · **survives, partially** · rank 6
*Claim:* BU-P5-077 (with BU-P7-007's five-way routing as corroborating evidence).
*Behavior:* mid-procedure, a worker selects one of several named durable procedures
by judgment about the nature of the work, runs it with that procedure's own
checkpoint/retry/evidence semantics, and returns to the parent.
*Lower rungs and why they fail:* shared context can only supply guidance text — it
cannot grant the chosen procedure its own checkpoint boundaries, retry, blocked state,
or evidence. Treating the five procedures as helpers fails §6.5 directly: a helper is
deterministic machinery subordinate to a stage's outcome, and `diagnosing-bugs`, `tdd`
and `prototype` are multi-step judgment procedures — the proposal itself calls the
first "a strong low-ambiguity reference workflow". Both failures are real and
representational, not aesthetic.
*Adversarial reduction:* the third rung — the actor submits a second Work and the
parent correlates it — is dismissed by citing §7.7/§10.3. But those sections express a
*design preference* about visibility, not a measured inability. Under a strict reading
of §6.7 ("cannot be represented faithfully"), that rung **works**; it loses parent/child
visibility, deterministic cancellation reaching into the child, and parent-aware
recovery. So this claim is downgraded from "cannot be represented" to "can be
represented only by losing four named properties". It is counted as surviving because
the §6.5 helper-test failure is genuine and unavoidable, but it is ranked last and
should be re-filed as **grammar pressure for real parent/child Work identity** rather
than as a blocking gap. W21's A/U branch and W30's non-linear state machine are the
same pressure at lower intensity and correctly raised no claim.

### Rejected

#### G7 — Dynamically-discovered checkpoint graph with a claim primitive · **rejected**
*Claim:* BU-P4-090 (supported by BU-P4-082, BU-P4-083, BU-P4-084, BU-P4-098, BU-P4-100).
*Why rejected:* the claim's own "why each fails" for the third rung is *"this works
but places every durable fact outside Sergeant's journal, so Sergeant has no evidence
of it"*. That is an **ownership preference**, not a representational failure. Wayfinder
is faithfully represented today at the shared-context/helper rung with the issue
tracker as the durable store, including the claim primitive (assignment) and the
blocking graph (native dependencies) — the source says so explicitly. §6.7 asks
whether the behavior *cannot be represented*, not whether Sergeant would rather own
the data. The concurrency argument is the strongest part and is preserved: **if** a
future design brings the ticket graph inside Sergeant for other reasons, an
anti-double-claim admission primitive becomes necessary. Recorded as an
engine-pressure *observation* on the ownership boundary. Note the substantial overlap
with G2 and G6 — if either is implemented, this claim should be re-examined rather
than re-argued from scratch.

#### G8 — Runtime-enforced actor-role authority for a restricted command · **rejected**
*Claim:* BU-P2-056.
*Why rejected:* the claim's first lower rung — a repository-wide instruction — is the
source's *own* mechanism, and the stated reason it fails is "only binds an actor that
reads and honors it". If that counted as an engine-gap justification, **every**
invariant in §2 would be an engine gap, which makes the ladder's rung 6.1 meaningless.
The third rung (a role-checking wrapper) is dismissed for want of an unforgeable role
identity, but the runtime does distinguish a Work's own execution context from a
client call, and a helper can observe whether it is running inside a dispatched work
surface — neither was attempted. Recorded as an **authority-model observation**: if a
tool-gating or permission capability is designed for other reasons, this behavior is
its first consumer. Note this claim directly contradicts three P1 units that classify
the identical behavior as a plain invariant — conflict **X4**.

#### G9 — Crash-safe durable publication (four claims) · **rejected as gaps; re-homed** · 
*Claims:* BU-P7-043 (fault-injected multi-file publication), BU-P7-079
(cross-filesystem atomic publish), BU-P7-049 (converge on the agent's own proof when
the recorder dies), BU-P7-051 (every exit branch must settle the lease).
*Why rejected as gaps:*
- BU-P7-043's own minimum capability says *"or the append-only journal itself, per this
  repo's own architecture"* — the capability it asks for **already exists**. It is a
  requirement satisfied by the target architecture, not a gap in it.
- BU-P7-079 is a property of a file-based store split across filesystems. The daemon
  owns one data dir and the journal is the durable state; the topology the claim
  describes does not arise. It is also flatly contradicted by BU-P8-108, which says
  Sergeant *refuses* that layout rather than falling back (conflict **X1**).
- BU-P7-049 is precisely CLAUDE.md's adjacent-append crash-window hazard plus
  "recovery prefers evidence over optimism" — an invariant the architecture already
  holds and `recovery.rs` already implements.
- BU-P7-051 asks for a runtime-owned exit hook so no branch can skip settlement. In the
  daemon there is exactly one terminal-transition funnel; the seven-branch omission
  class is an artifact of hand-written Bash exit paths.
*Re-homed as:* three architecture invariants that must be *stated* rather than built —
publication is crash-safe and convergent on retry; a two-party handshake converges on
independently verifiable proof rather than refusing forever; terminal transition is a
single funnel. Added as evidence behind Article IV. These are the corpus's best
external corroboration that the existing invariants are the right ones, which is a
genuinely valuable finding even though the gap claims fail.

### Roll-up

| Claim | Units | Verdict | Rank |
|---|---|---|---|
| G1 wait/wake scheduling | BU-P6-099 BU-P7-010 BU-P7-096 | survives (narrowed to the scheduler) | 1 |
| G2 fleet identity + dependency advance | BU-P5-065 BU-P6-016 BU-P6-017 | survives (split acceptance test) | 2 |
| G3 acknowledgement gate on cleanup | BU-P8-007 | survives (amended, re-file at reduced scope) | 3 |
| G4 admission block | BU-P6-063 | survives (high evidence, low cost) | 4 |
| G5 re-enterable needs-input stage | BU-P5-025 | survives (narrowed; answers U3) | 5 |
| G6 child-workflow invocation | BU-P5-077 | survives partially (grammar pressure) | 6 |
| G7 dynamic ticket graph | BU-P4-090 | **rejected** — ownership preference | — |
| G8 runtime role enforcement | BU-P2-056 | **rejected** — would make every invariant a gap | — |
| G9 crash-safe publication | BU-P7-043 BU-P7-049 BU-P7-051 BU-P7-079 | **rejected** — already satisfied; re-homed to Article IV | — |

**Method observation for N2.** Ten of the sixteen first-pass claims survived in some
form and six did not, but *five of the six survivors needed narrowing* — most often
because the claim's stated minimum capability was larger than its own evidence
required. The generator measurement in §9.9's "engine-gap quality" dimension should
score *scope discipline* (is the minimum capability the smallest thing the evidence
forces?) separately from *rung discipline* (were the lower rungs named and genuinely
tried?). The corpus shows they fail independently.

---

## 6. Cross-partition conflicts

Twenty entries. **Type S** = semantic contradiction (the two units cannot both be
followed). **Type C** = classification contradiction (same behavior, incompatible ICM
form). **Type B** = boundary/naming contradiction (same behavior, different workflow
decomposition). All are preserved as adjudicated evidence, not erased.

| # | Type | Conflict | Citations |
|---|---|---|---|
| **X1** | S | Cross-filesystem cleanup: Sergeant **refuses** a cross-filesystem layout rather than falling back from atomic rename, vs cleanup **must support** a copy-based cross-filesystem fallback with a CRITICAL diagnostic on rollback failure. | BU-P8-108 (`docs/troubleshooting.md`) vs BU-P7-079 (`tests/sgt-cleanup-cross-filesystem-test.sh`). Tests outrank docs (§5), which would favour P7 — but P7's unit is a rejected engine gap (G9), so the durable rule is P8's refusal. |
| **X2** | S | Graph output location: output is **never** published inside an owning source repository, vs the output directory **may** sit inside a source repo and Sergeant handles re-ingestion by staging extraction outside it. | BU-P5-107, BU-P5-100 (`skills/load-project/SKILL.md`) vs BU-P7-003 (`tests/`), BU-P8-035, BU-P8-036 (`docs/schema.md`). Tests and schema outrank skill prose; the skill states a *recommendation* as an absolute. |
| **X3** | S | Shipping-gate unattended consent: `--yes` must **never** be used, vs `--yes` is the user's standing consent and the documented sole exception to the ask-user escalation rule. | BU-P1-072, BU-P8-083 (Sergeant `AGENTS.md` / `docs/using-sergeant.md`) vs BU-P2-100 (vendored `no-mistakes/SKILL.md`). Resolvable by scope (Sergeant-coordinated runs forbid it; standalone runs allow it) but neither unit states that scope. |
| **X4** | C+S | Worker invocation of the shipping gate: an absolute prohibition stated as a hard error, vs an explicit user override that must be rendered verbatim into a worker's brief, vs the same rule classified as an engine gap. | BU-P1-040, BU-P1-131, BU-P8-082 (invariant) vs BU-P7-112 (`tests/` — override must survive into the brief) vs BU-P2-056 (engine-gap, rejected as G8). Three partitions, three forms, one behavior. |
| **X5** | C | Bash-3.2 portability: an operating invariant every runtime path must satisfy and every shipped script must parse under, vs a distribution-specific target that a compiled binary makes obsolete. | BU-P7-033, BU-P7-071, BU-P1-065 (agents-invariant) vs BU-P8-102 (obsolete-mechanism). |
| **X6** | C | Launch mode: "dispatch uses only persistent interactive sessions and rejects every non-interactive launch mode" recorded as a live stage-context rule, vs the identical statement recorded as obsolete and structurally reversed by D2. | BU-P1-057 (stage-context) vs BU-P7-008, BU-P8-064 (obsolete-mechanism). D2 settles it; P1's classification is the outlier. |
| **X7** | S | Setup backup ordering: a timestamped backup is created **before** writing in the new-project phase, vs a backup is created **only after** the user confirms and the skill must not even instruct pre-confirmation backup. | BU-P5-027 (Phase 4) vs BU-P5-030 (Phase 5) and BU-P7-038 (`tests/`, stated as a general rule). Same skill, two phases, opposite orders. |
| **X8** | C | Sequential one-question-at-a-time human input: an engine gap requiring a new runtime capability, vs ordinary actor procedure needing nothing, vs explicitly "representable today via the existing needs-input state, not an engine gap". | BU-P5-025 (engine-gap) vs BU-P3-007 (`grilling`, stage-context) vs BU-P4-053 (`to-spec`, stage-context with an explicit non-gap note). Resolved by G5's narrowing. |
| **X9** | B | Graph building: a standalone workflow with its own trigger and all-or-nothing merge, vs a stage inside `load-project`. | BU-P6-088/089/090/091 (workflow `graphify`) vs BU-P5-105/106/107/112, BU-P7-003/086/088 (stages of `load-project`). Synthesis promotes it (W2). |
| **X9b** | B | Daily digest: `wiki` with a `00-digest` stage, vs `wiki-daily-digest` as its own workflow. | BU-P5-130/131/137…149 vs BU-P6-092/093. Merged as W35. |
| **X10** | B | Installation: a ten-phase interactive skill, vs a nine-item completion checklist from the getting-started docs, with different phase sets and different prerequisite lists. | BU-P5-001…BU-P5-037, BU-P7-036…BU-P7-040 (`sergeant-setup`) vs BU-P8-041…BU-P8-051 (`sergeant-install`). Merged as W3; the two prerequisite lists differ (P5 omits Python 3 and Node; P8 adds them) and that difference is unresolved. |
| **X11** | B | Command surfaces promoted to workflows: `list-projects`, `project-status`, `project-sync`, `project-task-list`, `treehouse-init`, `doctor-capability-check` each extracted as a workflow in P6, vs the same commands treated as verification steps inside `load-project` / `sergeant-setup` in P5. | BU-P6-011, BU-P6-012, BU-P6-013, BU-P6-035, BU-P6-018, BU-P6-003 vs BU-P5-092, BU-P5-093, BU-P5-101, BU-P5-102, BU-P5-031, BU-P5-034. Synthesis follows P5 — §6.2 requires a bounded outcome and completion condition, which a listing does not have. |
| **X12** | B | The fleet runtime: P7 files six behaviors under one umbrella `software-change (…)` name, while P6/P8 give the same behaviors six distinct workflow names. | `software-change (drain/recovery)` BU-P7-083/084/092/093/094/095/107/108 vs `drain-admission-control` BU-P6-057/058 and `stall-recovery` BU-P6-071/072/073/075; `software-change (fleet monitoring)` BU-P7-099/100/101 vs `fleet-observation` BU-P6-101 and `fleet-reconciliation` BU-P6-103/104/105; `software-change (notification handshake)` BU-P7-035…109 vs `respond-and-resume` BU-P6-032/034/078/079/080; `software-change (independent review)` BU-P7-063/064 vs `review-finding-routing` BU-P6-082/084/085/086. Synthesis follows P6/P8 (W10–W16). |
| **X13** | S | Waiting workers: "a waiting worker **may remain alive** or may exit after a durable handoff", vs a worker must never poll or sleep and must exit cleanly after publishing its condition. | BU-P1-036 (`AGENTS.md`) vs BU-P7-010 (`templates/worker-brief.md`), BU-P8-074 (`docs/using-sergeant.md`). |
| **X14** | S | Review auto-fix: disabled by default but re-enabled by a repo- or global-level override, vs an auto-fix must **never** be authorized in Sergeant's validation-only workflow. | BU-P2-080 (vendored gate skill) vs BU-P8-100 (`docs/troubleshooting.md`). |
| **X15** | S | Dependency-edge enforcement: enforcement is "left entirely to the dispatched workers" reading their own brief, vs the acceptance test that a dependent must be **held**, not merely advised, vs an external scheduler that genuinely advances stages on completion. | BU-P5-074 vs BU-P5-065's acceptance test vs BU-P6-016/017. Recorded inside G2's split acceptance test. |
| **X16** | B | Direct execution: a six-stage `direct-mode` from `AGENTS.md`, vs an eight-step `direct-implementation` from `docs/using-sergeant.md`, with different step boundaries (the docs split reconciliation and the shipping gate into their own steps). | BU-P1-007/107 + BU-P1-008…014 vs BU-P8-055 + BU-P8-053/056/058. Merged as W6 on the docs' finer boundary. |
| **X17** | B | The shipping gate is filed under four different workflow names across four partitions, with different stage sets: `no-mistakes` (P2 pipeline-driving; P6/P8 launch machinery), `no-mistakes-shipping-gate` (P1), `no-mistakes (as consumed by software-change)` (P7). Separately, `validate-and-ship` in P6 names something else entirely — the source repo's own pre-push hook. | BU-P2-057…103, BU-P6-129…134, BU-P8-084…089 vs BU-P1-069…080 vs BU-P7-065/104/105 vs BU-P6-007/008. Merged as W18; the pre-push hook split out as W19. |
| **X18** | S | Intent transport: "no delivered content **ever** appears in process arguments" and "canonical intent content must **never** appear in process arguments", vs a documented per-invocation operator-consent path that selects exactly that argv transport. | BU-P8-065, BU-P8-085 (absolute) vs BU-P6-047, BU-P8-086 (consent-gated exception). The absolutes are stated as invariants and the exception is stated as a supported flag; both are in the same document family. |
| **X19** | S | Unfinished response handshakes at cleanup: cleanup must never be forced and deliberately refuses while the handshake could still be completed, vs cleanup **can** retire an unfinished handshake when the owning task is closed and the worker is provably dead by four independent proofs. | BU-P8-104 vs BU-P8-105/BU-P8-106. Reconcilable (refuse-by-default with a proven-dead exception) but each is written as an absolute. |
| **X20** | S | Procedure invocation: `implement` disables model invocation and must be explicitly invoked, vs the stated architecture that model-invoked disciplines "load automatically whenever the task matches" and the standing rule to load a procedure whenever its trigger applies. | BU-P2-051 (+ BU-P3-002/004) vs BU-P1-118, BU-P1-021. Resolvable by the orchestrator/discipline split, but `implement` is classed as a discipline in BU-P1-129's bundle. |

---

## 7. Unassigned units

Three units have no home in any of the five categories above. Each is recorded with
its reason rather than being silently absorbed.

| Unit | Source | Reason |
|---|---|---|
| **BU-P1-062** | `README.md` — "the right unit of distribution for turning a general-purpose agent into a specialist is not a CLI tool or an MCP server, but a cloned directory of instructions, skills, and conventions" | Product positioning, not a directive. It names no trigger, no required or prohibited action, and no evidence of compliance — it fails the corpus's own observability test at BU-P1-017 and would be removed by BU-P1-018's rule. It cannot change a decision. Retained as provenance for the product's design rationale. |
| **BU-P1-063** | `README.md` — "Sergeant narrows firstmate's crew-orchestration idea to start from project topology" | Attribution and design lineage. Same failure against BU-P1-017. The operative half ("a project is a named collection of repositories, and everything flows from that definition") is already carried by the `project-configuration` shared context (§3b) and `@@project` (§3a). |
| **BU-P7-021** | `tests/global-state-isolation-test.sh` — the pre-created drain-directory "trap" that works because `mv <tmp> global` moves a file *inside* a pre-existing sentinel directory and the paired `rm -f` cannot remove a directory | Pure mechanism of the source repo's own Bash test harness, with no durable policy of its own. The policy it serves — a leak guard must itself be provably able to detect that class of leak — is stated independently by BU-P7-022, which is retained in the `test-infrastructure` helper map. Classified `helper` by the extractor; that classification cannot survive the mechanism it depends on, but it is not an obsolete *Sergeant* mechanism either, so it belongs in neither §3 nor §4. |

---

## 8. Notes for the refute stage (§8.3 step 6)

Reviewers should attack these first — they are where this synthesis is most exposed:

1. **W8 `dispatch` is 63 units and 12 stages.** That is by far the largest cluster.
   Either it is genuinely one procedure with twelve checkpoints, or it should split at
   `70-launch-and-record` into `plan-and-validate` and `launch-fleet`. Argue it.
2. **W18's ten stages** are the U2 answer. If §6.3 discriminates as cleanly as claimed,
   an independent reviewer should reproduce roughly the same ten boundaries from the
   same units. If they produce six or fifteen, the test does not discriminate.
3. **G6 and G7 are the soft ones.** G6 survives on a §6.5 helper-test failure alone;
   G7 was rejected on a reading of §6.7 that a reviewer may think too strict. Both
   rulings should be attacked directly.
4. **The G9 rejection re-homes four claims into an existing invariant.** If a reviewer
   thinks that is the synthesis grading its own architecture favourably, that is a fair
   challenge and should be made explicitly.
5. **X4, X5, X6 and X8 are classification conflicts the synthesis resolved by ruling.**
   Each ruling is a candidate for reversal on evidence.
6. **U1's answer: 966 units from 179 files**, well above the 150–400 scoping estimate.
   The overshoot is concentrated in P6/P7/P8 (`bin/`, `tests/`, `docs/`), where a
   single script or test file routinely yielded 8–15 units. Whether that granularity is
   correct — or whether those partitions over-extracted at the sentence level while
   P2/P3/P4 extracted at the paragraph level — is itself a finding N2's precision
   measurement will have to control for.
