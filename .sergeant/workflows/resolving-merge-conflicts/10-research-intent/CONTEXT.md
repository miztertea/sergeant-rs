# 10-research-intent: research intent

## Inputs

| File | Layer | Why |
|---|---|---|
| ../CONTEXT.md | L1 | workflow orientation (first stage now that `00-assess-state` is demoted) |

## Purpose

The intent behind each conflicting side is researched.

Trigger (workflow-level): A git merge or rebase is in a conflicted state.

## What must become true here (durable outcome)

The intent behind each conflicting side is researched.

## Behavior contract

- **For each conflict, the actor traces the original intent behind each side's change via commit messages, PRs, and issues/tickets before attempting resolution.**
  (trigger: conflicting hunks have been identified; outcome: the intent behind each conflicting change is understood before it is resolved)

## Helper: assess state (folded from demoted `00-assess-state`, N1 adjudication A4)

`00-assess-state` was classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 it is demoted and its behavior folded here as a helper invoked before researching intent, subordinate to this stage's own judgment-bearing outcome:

- **The first checkpoint establishes the current merge/rebase state by inspecting git history and the conflicting files.**
  (trigger: the workflow begins; outcome: the actor has an accurate picture of what is conflicting and why)

## Bounded judgment

Apply `@@bounded-judgment`.

### J2 — delegated to this stage
- Which primary sources (commit messages, PRs, issues/tickets) to inspect when tracing each side's original intent.

### J1 — local choices allowed
- None beyond ordinary tool mechanics.

### J0 — must become `needs_input`
- No commit message, PR, or issue/ticket can be found for one side's change: state what evidence was checked and ask the user for the missing context, or explicit permission to proceed on the visible diff alone, rather than guessing at unstated intent.

### Completion boundary
This stage may complete only once the current merge/rebase state is established and the intent behind each conflicting side is traced — or the stage has stopped at the J0 case above.

### Decision evidence
Traced intent per conflicting side is this stage's own durable output, recorded per `output/README.md`.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
