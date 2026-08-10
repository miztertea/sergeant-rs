# 10-preflight-capabilities: preflight capabilities

## Inputs

| File | Layer | Why |
|---|---|---|
| ../05-classify-risk/output/README.md | L4 | upstream artifact produced by `05-classify-risk` |

## Purpose

Harness, model tuple, identity and pane/session bindings are all validated and rejected before any durable state exists.

Trigger (workflow-level): Work spans repositories, contains two or more independent repository-owned tasks, needs an isolated review worker, or the user asks for workers.

## What must become true here (durable outcome)

Harness, model tuple, identity and pane/session bindings are all validated and rejected before any durable state exists.

## Behavior contract

- **SERGEANT_AGENT or sgt-dispatch --agent may select opencode, oc, goose, claude, or an equivalent path whose basename is one of those names; dispatch uses only persistent interactive sessions and rejects every other agent and all non-interactive launch modes before creating worker state.**
  (trigger: a dispatch is requested; outcome: only a known interactive harness can ever get as far as creating worker state)
  — `BU-P1-057`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L186)
- **sgt-dispatch --model or SERGEANT_MODEL pins the harness model as provider/model[:variant]; --agent/SERGEANT_AGENT select the executable and are orthogonal to it; precedence is --model > SERGEANT_MODEL > the harness's ambient default, and an unpinned dispatch is recorded as unpinned rather than left blank.**
  (trigger: a dispatch is being launched; outcome: the resolved model tuple and its provenance (or explicit absence) are always recorded)
  — `BU-P1-058`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L187, model precedence)
- **A model/variant tuple the selected harness cannot honor fails before any intent file, td task, worktree, or fleet state is created, and a worker handed one fails terminally instead of inheriting the ambient default.**
  (trigger: a pinned tuple cannot be honored by the selected harness; outcome: no partial or silently-degraded worker state is ever created)
  — `BU-P1-060`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L187, fail-before-side-effect)
- **A resumed or recovered worker reads the same fleet record and inherits the same pin; a worker handed a tuple its harness cannot honor fails terminally rather than falling back to the ambient default.**
  (trigger: a worker is resumed or recovered; outcome: the model pin is preserved exactly across resumption, and an unhonorable pin never silently degrades)
  — `BU-P1-093`, `reference/sergeant-upstream/README.md` (README.md L227-229)
- **A model/variant pin fails closed in two distinct situations with a diagnostic that says which: no known transport (the harness is measured and exposes no way to pin that axis) versus unmeasured (the harness is not installed here, so its launch surface has not been observed) — the latter is not a claim the harness cannot do it.**
  (trigger: a model/variant pin cannot be applied; outcome: the failure diagnostic distinguishes a measured incapability from simple absence of measurement)
  — `BU-P1-094`, `reference/sergeant-upstream/README.md` (README.md L205-210, fail-closed distinction)
- **A worker never launches with a harness it cannot honor: the capability gate (is the harness accepted), a readiness probe, and the launch invocation are all validated up front, before any fleet state directory is even created, so an invalid harness is rejected before durable state exists to clean up.**
  (trigger: a worker supervisor is about to own an interactive harness pane; outcome: an unusable harness is rejected immediately, never discovered later as an unsatisfiable hang)
  — `BU-P6-107`, `reference/sergeant-upstream/bin/sgt-interactive-worker` (L33-40)
- **Which pinned model tuple a dispatched worker will run is resolved with a fixed, explicit-only precedence — a per-invocation flag beats an environment variable, which beats no pin at all — with no project-level or per-repo default in the precedence chain by deliberate decision, not by omission, and the resolution and its shape are both validated before any intent file, task, or worktree exists.**
  (trigger: a dispatch is choosing which model tuple to pin for its workers; outcome: the source of a pinned model is always explicit and recorded, never an implicit config default a caller might not realize is in effect)
  — `BU-P6-124`, `reference/sergeant-upstream/bin/sgt-dispatch` (L180-190)
- **GitHub CLI identity for dispatching a worker resolves in a fixed priority order: repo-level identity overrides project-level identity, which overrides the global default identity, which falls back to no identity switch.**
  (trigger: a worker is about to be dispatched against a repo that may need a different GitHub identity; outcome: exactly one identity source wins, in a documented and testable precedence order)
  — `BU-P7-002`, `reference/sergeant-upstream/schema/project.yaml.example` (lines 13-15)
- **A failed `gh auth switch` during dispatch identity resolution must set the fleet task's status to failed with a recorded diagnostic and abort the dispatch, rather than silently proceeding to dispatch under the wrong (or no) identity.**
  (trigger: the resolved GitHub identity cannot actually be switched to at dispatch time; outcome: a worker is never silently dispatched under an unintended identity; failure is loud, diagnosed, and terminal)
  — `BU-P7-072`, `reference/sergeant-upstream/tests/sgt-dispatch-identity-test.sh` (lines 1-9)
- **sgt-dispatch must pin an explicit provider/model/variant tuple for a dispatched worker and record it as durable, non-secret launch evidence in fleet state; every validation of that tuple must run and reject BEFORE any mutation, so a rejected dispatch leaves no new fleet task directory and no tmux log at all.**
  (trigger: a worker is dispatched with a specified (or defaulted) provider/model/variant; outcome: which exact model/provider/variant a worker ran under is a durable, auditable fact, and any invalid tuple is rejected with zero side effects rather than a partial dispatch)
  — `BU-P7-073`, `reference/sergeant-upstream/tests/sgt-dispatch-model-tuple-test.sh` (lines 1-11)
- **sgt-dispatch must be able to bind a coordinator pane without itself already running inside one, while refusing every forged, stale, or unreachable pane identity, and every such rejection must happen before any fleet task directory is created.**
  (trigger: sgt-dispatch is invoked from a shell with no ambient TMUX/TMUX_PANE; outcome: dispatch can be launched from outside any tmux pane while still establishing a verifiable, non-forgeable coordinator identity for notification routing)
  — `BU-P7-078`, `reference/sergeant-upstream/tests/sgt-dispatch-coordinator-pane-test.sh` (lines 1-11)

> **Read `pane`/`tmux` above as this project's durable execution/session identity, not literally.** Old Sergeant's tmux pane is obsolete here (deviation register D2; `reference-corpus/synthesis.md` §4 clusters M1-M4) — `BU-P1-057`, `BU-P7-073`, `BU-P7-078` carry a durable identity/liveness/ownership policy that survives the pane; the pane itself does not.

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Additional note

"Nothing was created if this failed" is the checkpoint — not how the probing is implemented. **Caveat on BU-P1-057/BU-P1-093/BU-P1-094:** these units' literal text requires "persistent interactive sessions" and rejects "non-interactive launch modes" — that half of the statement is old Sergeant's tmux-pane launch mechanism, ruled obsolete and structurally reversed by this project's own deviation register (D2: headless `claude -p --resume` turns, no persistent pane). Preserve the *durable* part of these citations only: a harness/model/identity tuple is validated and rejected before any durable state exists, and a resumed worker inherits the same pin. Do not implement a literal interactive-session requirement from this stage.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
