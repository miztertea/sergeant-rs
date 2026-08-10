# 10-verify-ownership: verify ownership

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-require-terminal/output/README.md | L4 | upstream artifact produced by `00-require-terminal` |

## Purpose

Repo identity, not path, is verified; retry-owner spoofing vectors are rejected.

Trigger (workflow-level): A task's repos are believed terminal and the operator (or an automated sweep) requests cleanup.

## What must become true here (durable outcome)

Repo identity, not path, is verified; retry-owner spoofing vectors are rejected.

## Behavior contract

- **Cleanup never trusts a fleet-recorded worktree path as sufficient proof of ownership on its own; the resolved owning repository must be the exact same repository a previous pass recorded (verified by a repo identity, not just a path), and any recorded worktree that is present must independently be verified to belong to that owning repository, because a worktree replaced by an unrelated repository would otherwise answer lookups out of a foreign tracked-work database.**
  (trigger: cleanup needs to resolve which repository owns a fleet repo's tracked work; outcome: tracked-work status can never be looked up against the wrong repository just because a path happens to coincide)
  — `BU-P6-137`, `reference/sergeant-upstream/bin/sgt-cleanup` (L925-929, L956-959)
- **Determining who legitimately owns a retry (whether the same repository is still the one recorded for a fleet task) must reject a wide range of repository-identity spoofing: symlink-aliased repos, same-origin clone replacement, independently-reset HEAD/refs, in-place repository or hook metadata changes, configured-worktree edits, repository replacement or move, and cross-project prefix-colliding or same-path repositories.**
  (trigger: cleanup or a related command re-verifies that a recorded repository is still the same one it was dispatched against; outcome: repository identity for retry/cleanup purposes cannot be spoofed by any of a wide, deliberately adversarial set of filesystem or git-state manipulations)
  — `BU-P7-081`, `reference/sergeant-upstream/tests/sgt-cleanup-test.sh` (line 731 (one of ~13 assert_retry_owner_rejected cases spanning lines 723-825))

## Deterministic-machinery candidate

Classified at extraction as deterministic machinery crossing this checkpoint (ladder §6.5) — a repeatable operation subordinate to the stage's outcome. No `kind = "execute"` stage exists in the current engine (N1 is content-only; the governing proposal's Phase B is not adopted at this milestone), so this remains an ordinary actor stage: the acting harness performs or invokes the equivalent deterministic step(s) itself (a helper script, or the operations named in the behavior contract above) and reports the structured result. It is a candidate `execute`-stage workload the moment that stage kind exists (proposal §12.3, §9.7).

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
