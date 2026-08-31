# Hierarchical execution: nested workflow packages and child Work

W1 answers two different needs that must not be collapsed into one mechanism: **private procedural decomposition inside one Work**, and **a separately durable task an actor discovers mid-run**. The first is a **nested workflow package** — engine-level recursion, still the same Work. The second is **child Work** — an ordinary, separately admitted Work, related to its parent only by a validated causal link.

## Nested workflow packages

A stage directory may itself contain a valid `workflow.toml`. That marker makes the stage a *container* whose implementation is another workflow package — the same grammar [the workflow package reference](../reference/workflow-package.md) already defines, reused recursively rather than through a second `subworkflow.toml` syntax, a DAG format, or a shell command that starts a private runtime of its own (W1-01, W1-05).

```text
workflow/
  workflow.toml
  00-orient/
    CONTEXT.md
  10-investigate/
    workflow.toml
    00-lead/
      CONTEXT.md
    10-code/
      CONTEXT.md
    20-synthesize/
      CONTEXT.md
  20-implement/
    CONTEXT.md
```

A container stage has **no implicit actor** — if a squad-lead execution is wanted at the top of a nested package, it is an explicit nested stage (`00-lead` above), never a hidden dual behavior a container directory triggers on its own (W1-02). Nesting can go two or more levels deep; W1 adds no product-level depth limit (only ordinary resource/turn/time envelopes bound it).

**Flatten-at-load.** The loader does not give the engine a tree to schedule. It flattens a nested package at load time: a container contributes no stage of its own, its leaves splice into the parent's one ordered stage list with `parent/child`-composed string ids (`10-investigate/00-lead`, `10-investigate/10-code`, …), and everything downstream — binding, advancement, retry, replay — runs against that flat list exactly as it always has (W1-03, W1-04). The hierarchy is real (the TUI renders it, `sgt work show` reports it), but the scheduler never sees anything but a flat sequence of leaves. A container's own closing point is tracked in a journaled side-table keyed by which flat leaf index is the last one under it, not as a stage of its own — a container is not a stage, and has no stage-shaped state to fail or retry.

A container closes only once its nested package completes and any output contract declared on the container itself is satisfied against the shared Work evidence/artifact surface — the landed expected-output/required-column/disposition/branch-status contracts apply identically to a nested leaf and to the container that wraps it, never bypassed by recursion (W1-13).

**Cycle guard.** A nesting path that would recurse into a package that contains itself — by name, the same failure mode a symlink cycle would produce — fails closed at load time rather than recursing without end.

Nested stages never leave the parent's envelope: same estate, same admitted repository scope, same pinned bases, same Work branches/worktrees, same Work lifecycle. This is procedural decomposition, not a second mutation authority — every fresh execution a nested leaf launches is still, ordinarily, a fresh execution over the *same* parent Work surfaces.

## Child Work

When an actor discovers an independently meaningful task — not a step of the procedure it is already running, but a separate thing that deserves its own durable record — it submits **ordinary Work**, against the same host daemon, by explicitly addressing the estate:

```bash
sgt -C "$SERGEANT_ESTATE_ROOT" run "..."
```

`-C <estate-root>` is the same explicit-addressing mechanism every other estate-scoped invocation already uses (W1-06). The child submission is a normal Work: it undergoes ordinary estate/repository/workflow preflight, gains no `sergeant.toml` of its own, and does not reopen ancestor discovery from inside a Work surface — that remains a non-goal (§12 of the ratifying proposal; "implicit estate discovery from Work surfaces" is explicitly listed as one).

This is the one sanctioned exception to the worker-surface prohibition [AGENTS.md's ESTATE section](../../AGENTS.md) states: a worker does not otherwise invoke an estate-scoped `sgt` command from its own surface, *except* this one addressed, validated path. See that section for the doctrine text itself; this page describes the mechanism the doctrine narrows around.

### Causation: claimed, then validated, never trusted

A managed execution — the launch an actor stage or an execute stage runs under — receives exactly three environment values, injected at the adapter launch seam:

```text
SERGEANT_ESTATE_ROOT
SERGEANT_WORK_ID
SERGEANT_EXECUTION_ID
```

Three values, not more: W1-07 names this the minimum transport context a child CLI call needs, arrived at by checking the R1–R6 minimality rungs first — no existing managed-execution identity environment was found to reuse, and nothing narrower than the estate root plus the parent Work/execution pair can carry product lineage.

When the CLI issues `sgt run` from inside such an execution, it reads `SERGEANT_WORK_ID`/`SERGEANT_EXECUTION_ID` out of that environment and sends them as **claimed** parent Work/execution coordinates — named as claims deliberately, because an environment variable is a transport hint, not trusted lineage, and no adapter clears its environment before handing it to a child process. **`SERGEANT_ESTATE_ROOT` is deliberately never read this way.** The estate for the child submission is resolved the ordinary way — `-C` or cwd, admitted like any other addressed root — specifically so that reading a *claimed* root out of the environment could never become the implicit estate discovery from a Work surface that is a standing non-goal. The actor names `-C` itself; the triple only ever supplies the Work/execution half of the claim.

The daemon validates a claim against its own journal before recording anything: does the claimed parent Work exist, does it belong to the addressed estate, does the claimed execution belong to that Work (W1-08). A claim that checks out becomes a recorded relation on the child's own `work.submitted` event, extending the existing causation/correlation fields the event envelope already carried rather than standing up a second agent-tree store (W1-09).

**A failed claim never refuses the child Work.** This is the ratification's own clause, not an implementation shortcut: the child submission proceeds as an ordinary, causation-less Work, and the daemon instead journals a `causation_unverified` marker — the claimed coordinates verbatim, plus the reason the check failed — in the same `work.submitted` payload, so no crash window can separate a submission from the marker explaining it. Refusal would be actively wrong here: nothing clears an adapter's environment, and a long-lived actor session can easily outlive the very execution whose coordinates it inherited, making a stale claim the ordinary case rather than the suspicious one. An operator reading `sgt work show` sees exactly what was claimed and exactly why it didn't validate, which is what lets a stale inherited environment be told apart from a forged or misconfigured one without correlating against anything else.

### Independence

A child Work has its own work id, workflow pin, repository scope, base SHAs, Work surfaces and branches, stages and executions, and terminal result. Causation is a recorded relationship, never a lifecycle: nothing here is inherited or cascaded automatically.

- parent completion does not auto-cancel the child (W1-12);
- parent cancellation does not auto-cancel the child (W1-12);
- child repository scope is never silently inherited from the parent (W1-11);
- child output is never auto-merged or cherry-picked into a parent branch;
- child failure does not silently rewrite parent stage state;
- a child targeting a repository the parent also targets still gets its own Work branch/worktree — never a second writer sharing the parent's.

If a workflow actually wants to wait on a child's outcome, `sgt run --wait` submits, then observes the terminal state over the ordinary API/event path (W1-10) — the client waits, the engine holds nothing new.

### Recovery

A nested package recovers from the journal's hierarchical stage ids and ordinary stage events, which identify the deepest incomplete path — the recursive scheduler is reconstructed from package identity plus events, never from a process tree. Child Work recovers exactly as any other Work does, independently of its parent: causation is evidence and relationship, never lifecycle ownership, so a parent's own recovery has nothing to do with whether its children recover.

## What the old worker prohibition protected, and how this preserves it

Before W1, AGENTS.md's ESTATE section forbade a worker from invoking any estate-scoped `sgt` command from its own surface, without exception. The reason was exact-root admission: a worker's surface carries no `sergeant.toml`, so nothing there could address an estate at all, and the prohibition made that structural fact an explicit rule rather than an implicit one. The substance being protected was never "a worker must never submit Work" — it was **a worker must not silently become a nested Captain**, deciding and dispatching without anything recording that it did.

W1 does not weaken that. It answers the legitimate child-Work case by explicit addressing plus validated causation instead of by refusal: ordinary admission still requires an exact, explicitly named estate root; explicit addressing means the worker names `-C "$SERGEANT_ESTATE_ROOT"` itself rather than having ambient discovery reach for one; and journal-validated causation means the daemon — not the claim — decides whether a relation gets recorded at all. A worker that tries to address an estate it was never given still refuses, the same as it always did; the one path that now succeeds is exactly the one the ratification named and no other (host-atlas r3 ratification, ruling 2).

## Decisions

| ID | Decision |
|---|---|
| W1-01 | Reuse `workflow.toml` recursively as the nested-package marker — no `subworkflow.toml`, no DAG syntax. |
| W1-02 | A container stage has no implicit actor; a squad-lead execution is an explicit nested stage. |
| W1-03 | A nested package stays inside the same Work: same surfaces, same scope, same lifecycle. |
| W1-04 | Hierarchy is encoded in the existing string stage id (`parent/child`), not a new recursive id type. |
| W1-05 | Recursion lives in the existing engine — retry, replay, and recovery own it, not shell/process recursion. |
| W1-06 | Child Work uses ordinary `sgt run -C` — the existing explicit-addressing mechanism, nothing new. |
| W1-07 | Managed execution carries exactly three causation values — the measured minimum transport context. |
| W1-08 | A claimed parent Work/execution is validated against the daemon's own journal before any relation is recorded. |
| W1-09 | Causation extends the existing event envelope's own causation/correlation fields — no second agent-tree store. |
| W1-10 | `--wait` is submit-then-observe client behavior over the existing API/event path — no new engine hold state. |
| W1-11 | Child repository scope is never auto-inherited from the parent. |
| W1-12 | Parent/child completion and cancellation never auto-cascade in either direction. |
| W1-13 | Nested leaves and container closure reuse the landed stage-output/finalize contracts — never a second completion or artifact model. |

See [the workflow package reference](../reference/workflow-package.md) for the `workflow.toml`/stage-directory grammar this feature reuses recursively, and [host runtime and estates](host-runtime.md) for the estate-admission machinery every child-Work submission goes through like any other.
