# 00-orient — #297: the close sweep deletes other Works' merged evidence

## 1. Pinned fixed point (`@@pin-fixed-point`)

- **Revision:** `968c9edf1ceb9c214eb9daff8460f132c4dfaef2`
  (`968c9edf #334: close the journal-integrity defect at the choke point`),
  the lane base of `/var/tmp/hats6/sweep`. Confirmed:
  `git rev-parse HEAD` → the SHA above; `git status --porcelain` empty at
  orientation start and again at orientation end.
- Every later stage of this run is judged against this revision. Nothing
  downstream re-pins (`@@pin-fixed-point`: "Nothing downstream may re-pin").
- No diff is in play at this stage (this is a defect-fix run, not a review
  of an existing diff), so the non-empty-diff clause does not apply here.

## 2. Spec / acceptance source (`@@identify-spec-source`)

Located by the priority order; the first two tiers both hit, so nothing was
invented.

| Tier | Source | What it governs |
|---|---|---|
| Explicit reference in the intent | GitHub issue **#297** (`gh issue view 297`), by the outside reporter | The defect, its repro, and its root cause |
| Path supplied by the intent | `knowledge/evidence/resources/host-atlas-s6-series/brief-297-sweep-scoping.md` | The wave's boundary, fail-closed posture, and what the test must prove |
| Governing contract (J5, wins over the brief) | `knowledge/evidence/resources/host-atlas-series/W1-HIERARCHICAL-EXECUTION.md` §3, §4 ("Landed stage-output contracts remain authoritative"), decision W1-13 | Composed stage identity, and that a **container** may declare its own output contract |
| Shipped in-repo contract | `.sergeant/workflows/*/**/output/README.md` (`**Disposition:**` lines), read by `declared_output_disposition` (`src/domain/workflow.rs:1709`) | A stage's output disposition |

**Acceptance criterion this change is judged against** — issue #297's own
"Fix" paragraph, quoted: *"Read the running workflow's own declared `stages`
list straight out of its `workflow.toml` … and scope the sweep to only those
stage ids. When the workflow source is missing or its `workflow.toml` is
unreadable, sweep nothing rather than falling back to the unscoped
behavior."*

## 3. The defect, reproduced on the real thing

Not reasoned about — run end to end through the daemon, submit, and
teardown, using the existing `tests/m3_execution.rs` harness (R2: the
`the_finalize_sweep_removes_evidence_class_output_and_keeps_promote_class`
test at `tests/m3_execution.rs:1608` is already exactly this shape). A
temporary test was appended, run, and then removed
(`git checkout tests/m3_execution.rs`; the tree is clean at this commit —
writing the decisive test is `10-implement`'s work under `@@test-first`,
not this stage's).

Setup: workflow `sweep` declares exactly one stage, `00-evidence`. The mount
is seeded and **committed** with two directories before submit — the Work's
own `00-evidence/output/notes.md`, and `99-foreign/output/leftover.md`,
standing in for a prior, unrelated Work's already-merged evidence sitting in
the base branch. `99-foreign` is not a declared stage of `sweep`.

Command:

```
TMPDIR=/var/tmp/sgt-test-tmp CARGO_BUILD_JOBS=6 cargo nextest run --no-fail-fast \
  --test m3_execution -E 'test(repro_297)' --nocapture
```

Captured output (`/var/tmp/sgt-test-tmp/repro297.log`, exit 100):

```
REPRO sweep commit diff:
commit 412530eb7164d35a928cb958c25a45e98e463c61
Author: sergeant finalize <sergeant-finalize@localhost>
Date:   Mon Aug 31 01:58:51 2026 +0000

    sergeant: finalize sweep — remove evidence-class stage outputs

D	00-evidence/output/notes.md
D	99-foreign/output/leftover.md

thread 'repro_297_the_sweep_deletes_a_foreign_stage_output_dir' panicked at tests/m3_execution.rs:6486:5:
REPRO #297: another Work's merged evidence 99-foreign/output/leftover.md was deleted by this Work's finalize sweep
```

**Observable state, not inference:** the engine's own
`sergeant: finalize sweep` commit contains `D 99-foreign/output/leftover.md`.
Merging that branch deletes the other Work's evidence upstream. The defect is
reproduced.

## 4. Cause, located at file:line (the brief's account — verified, not assumed)

Every claim below was read in the lane at the pinned revision.

1. **The walk is unscoped.** `stage_output_dirs`
   (`src/runtime/surface.rs:1878`) → `collect_stage_output_dirs`
   (`:1885`) pushes *any* directory that has an `output/` child
   (`:1892-1897`), to `MAX_STAGE_SCAN_DEPTH` = 8 (`:1844`). Its own doc
   comment states the posture outright (`:1862-1865`):
   *"nothing here consults the workflow catalog: the worktree's own
   filesystem shape stays ground truth."* **Confirmed.**
2. **`finalize_sweep` (`src/runtime/surface.rs:2133`)** iterates that walk at
   `:2146` and applies no stage-id membership check before classifying,
   copying, `remove_dir_all`'ing (`:2170`) and committing (`:2185-2199`).
   **Confirmed.**
3. **`retain_stage_outputs` (`src/runtime/surface.rs:1983`)** iterates the
   same walk at `:1992`. **Confirmed.**
4. **Fail-open #1 (classification).** `declared_output_disposition`
   (`src/domain/workflow.rs:1709-1715`): `let Ok(text) =
   std::fs::read_to_string(readme) else { return OutputDisposition::Evidence
   };` — an unreadable README returns `Evidence`. A foreign directory has no
   README in this package at all, so it is classified `Evidence` and swept.
   **Confirmed.** (Note: `Evidence`-on-silence is *correct* for a declared
   stage — §1a's "silence promotes nothing" — the bug is that a foreign
   directory reaches this function at all.)
5. **Fail-open #2 (`retain_stage_outputs`).** `src/runtime/surface.rs:1998-2002`:
   `workflow_source.map(...).unwrap_or(OutputDisposition::Evidence)`.
   **Confirmed.**
6. **`finalize_sweep` already fails closed on absent source**:
   `src/runtime/surface.rs:2138-2140`, `let Some(workflow_source) =
   workflow_source else { return report; };`. **Confirmed** — the brief's
   claim that `retain_stage_outputs` lacks the equivalent is accurate.
7. **Composed-id shape.** `composed_stage_id`
   (`src/runtime/surface.rs:1947-1959`) returns the `/`-joined path from the
   worktree root down to the stage directory (`10-investigate/00-lead`).
   `StageDefinition::id` is documented as exactly that composition
   (`src/domain/workflow.rs:396-406`), and `WorkflowDefinition::load_dir`
   composes it at `src/domain/workflow.rs:1344` (`format!("{id}/{}",
   leaf.id)`). The two shapes match; a membership test against
   `WorkflowDefinition` ids is comparing like with like. **Confirmed, not
   assumed.**

The brief's stated cause is therefore **confirmed in full**. No refutation.

## 5. One thing the brief under-specifies — containers (J5, contract wins)

The brief and issue #297 both say "scope to the workflow's declared
**stages**". Taken literally against `WorkflowDefinition.stages`, that is
**not sufficient**, and implementing it literally would create a second,
opposite data-loss bug.

- `WorkflowDefinition::stages` is documented as **leaves only**:
  `src/domain/workflow.rs:525-530` — *"Stages in execution order — **leaves
  only**, one flat list … the container itself is never a stage (W1-02)."*
- A **container** may nevertheless declare its own output contract, and the
  engine checks it: `ContainerBoundary::container_id`
  (`src/domain/workflow.rs:498-503`) is *"the id the gate joins onto a
  package directory **and** onto a worktree"*; the gate runs at
  `src/runtime/engine.rs:3200-3208` via `OutputContract::Container`.
- W1 §4 ("Landed stage-output contracts remain authoritative") states it as
  contract: *"a container stage completes only after its nested package
  completes and any output contract declared on the container itself is
  satisfied against the shared Work evidence/artifact surface"*, with
  decision **W1-13** binding finalize to container closure.
- `stage_output_dirs`' own doc comment already says it finds container output
  dirs deliberately (`src/runtime/surface.rs:1863-1866`).

**Therefore the declared-id set is `stages[].id ∪ containers[].container_id`,
not `stages[].id` alone.** Cited J5 (governing constraint: W1 §4 / W1-13 and
the shipped container output gate) — the contract wins over the brief's
phrasing, per this run's own standing instruction. This is not a widening:
container ids are declared ids of the running workflow, so the sweep still
only ever narrows relative to today's unscoped walk.

## 6. Boundary of this change (J2 — this stage's delegated call)

**In scope**

1. A way to obtain the running workflow's declared composed ids — leaves and
   containers — from the package at `workflow_source`. R2 first:
   `WorkflowDefinition::load_dir` (`src/domain/workflow.rs:1030`) already
   parses exactly this and already composes the ids; a helper should reuse it
   rather than re-parse `workflow.toml` by hand.
2. Scoping the loop in `finalize_sweep` (`src/runtime/surface.rs:2146`) to
   that set.
3. Scoping the loop in `retain_stage_outputs` (`src/runtime/surface.rs:1992`)
   to that set, **plus** adding the fail-closed early return
   `finalize_sweep:2138` already has, so an absent `workflow_source` retains
   nothing rather than sweeping everything.
4. Fail closed in both places when `workflow_source` is absent **or** the
   package at it will not load — never a fallback to the unscoped walk.
5. The decisive end-to-end test of §3's shape (foreign dir survives; no
   deletion in the sweep commit's diff), wired into
   `scripts/coverage/c2-suites.sh` or `c3-spawning-suites.sh` at birth so
   `cargo nextest run --test c2_light -E 'test(coverage_stage_membership)'`
   is green (#231). Nested/composed and container ids covered, since §5 is
   where a naive fix breaks.

**Explicitly out of scope**

- Changing `stage_output_dirs`' walk shape, `MAX_STAGE_SCAN_DEPTH`, the
  `.git`/gitignore pruning, or the depth warning. The membership filter goes
  at the call sites (or as a filter over the walk's result), not by rewriting
  the walk's traversal.
- Changing `declared_output_disposition`'s `Evidence`-on-silence default
  (`src/domain/workflow.rs:1713`). That default is §1a's ruling
  ("silence promotes nothing") and is correct for a *declared* stage; the fix
  is to stop feeding it foreign ids, not to change what it returns.
- Any change to `copy_declared_output_artifacts`, `capture_dirty_patch`, the
  scoped `git add` (F-IN-01, `src/runtime/surface.rs:2176-2186`), the
  finalize commit identity/message, or teardown's dirty check.
- Any widening of what the sweep touches. This change only narrows.
- `sgt search`/`related` stay pure readers; the SQL boundary
  (`store::SqlText` / `Sql::of` / `ReadOnly` + `read_sql!`) is untouched.
- Recovering evidence already deleted in downstream estates by past runs, and
  any migration/repair verb for it. Not asked for; would be new public
  behaviour (J0 if it comes up).
- Rewriting or repointing the existing sweep tests beyond what the new
  membership scoping requires.

**Open question carried forward (not blocking — no rung needed yet):** whether
the reporter's proposed `declared_stage_ids` helper lands verbatim in
`src/domain/workflow.rs` is `10-implement`'s R-rung call; the brief already
delegates it ("you own whether it is what lands").

## 7. Completion boundary check

Revision pinned and confirmed (§1); spec/acceptance source located, not
invented (§2); defect reproduced with real captured output (§3); cause
located at file:line and the brief's account verified rather than assumed
(§4); boundary stated in words `20-panel` and `40-close` can hold this run to
(§6). No J0 was reached. Nothing under `src/` was changed by this stage;
`git status --porcelain` is empty apart from this artifact.
