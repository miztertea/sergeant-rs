# 30-sync-repositories: sync repositories

## Inputs

| File | Layer | Why |
|---|---|---|
| ../20-register-or-edit/output/README.md | L4 | upstream artifact produced by `20-register-or-edit` |

## Purpose

Every required repo is cloned/refreshed, or the run stops naming the exact failure.

Trigger (workflow-level): A project is named, registered, edited, synced, or listed; or repository ownership is not already established.

## What must become true here (durable outcome)

Every required repo is cloned/refreshed, or the run stops naming the exact failure.

## Behavior contract

- **A missing required repository is synced only once the requested work actually requires it, via sgt-sync <project>; the workflow stops if cloning or pulling fails.**
  (trigger: a required repository is not present locally; outcome: repositories are fetched lazily and on failure the workflow does not proceed on an incomplete checkout)
  — `BU-P5-095`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (lines 28-29)
- **sgt-sync <project> runs only when repositories actually need cloning or refreshing, not unconditionally after every edit.**
  (trigger: project YAML has been verified; outcome: sync is triggered by actual need, not by habit)
  — `BU-P5-102`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 47)
- **Syncing a project's repos treats three distinct repo states differently: an already-cloned repo on a named branch is pulled fast-forward-only (never merged), an existing non-git directory is left untouched with a warning, and a missing repo with a configured URL is cloned; every other combination is reported and skipped rather than guessed at.**
  (trigger: operator runs sgt-sync <project>; outcome: every repo ends the run either cloned, pulled, or explicitly reported as needing manual attention — never silently skipped without a reason)
  — `BU-P6-013`, `reference/sergeant-upstream/bin/sgt-sync` (L30-45)
- **A repo pull only proceeds fast-forward and is skipped with a warning (never force-merged or rebased) when the branch has diverged or has no upstream, and a detached HEAD is skipped outright rather than guessed at.**
  (trigger: an existing cloned repo is being synced; outcome: a repo is never mutated in a way the operator did not ask for; ambiguous local state is reported, not resolved automatically)
  — `BU-P6-014`, `reference/sergeant-upstream/bin/sgt-sync` (L33-39)
- **If a required repository entry has no clone URL, load-project stops with the repository name and the missing field named explicitly.**
  (trigger: a required repository lacks a URL; outcome: the operator gets a precise, actionable error rather than a downstream sync failure)
  — `BU-P5-109`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 73)
- **If a required executable is missing, load-project reports the executable and a platform-neutral installation requirement, and never invents a fallback parser.**
  (trigger: a required executable is absent; outcome: missing tooling is reported honestly instead of being silently worked around with invented parsing logic)
  — `BU-P5-110`, `reference/sergeant-upstream/skills/load-project/SKILL.md` (line 74)

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
