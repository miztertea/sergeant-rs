# Invariants axis — critic findings
**Plan under review:** `docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md`
**Axis:** NORTH-STAR.md ownership boundaries · docs/DEVELOPMENT.md architecture
invariants · AGENTS.md "When NOT to use `sgt`" boundary · Ponytail Minimality
Ladder for everything the plan proposes building
**Contract:** `docs/gauntlet/contracts/MACBOOK-ARRIVAL-1.md`, non-goals §: the five
owner rulings (R1–R6) are not re-litigated on their merits; only a ruling
*as written* that misrepresents what was said or contradicts a governing
document is in scope.

Sources read in this session (primary):
- `docs/gauntlet/contracts/MACBOOK-ARRIVAL-1.md`
- `docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md`
- `NORTH-STAR.md`
- `docs/DEVELOPMENT.md`
- `AGENTS.md`
- `reference/notes/ideaos-agent-contract.md`
- `skills/estate-navigation/SKILL.md`
- `sgt repo add --help` (executed in session)
- `sgt init --help` (executed in session)
- `.sergeant/workflows/validate-and-ship/{00-check-scope,10-do-the-work,20-select-intent-transport,30-start-run,40-drive-gates,50-reconcile-custody,60-close-out}/CONTEXT.md` (all six stage files)

---

## Summary

Three findings; two warnings, one info. No NORTH-STAR ownership boundary
violations, no "When NOT to use `sgt`" violations, no Ponytail rung
violations. Details below.

---

## F1 — Wave 0 git operations cited against estate-navigation skill, but the skill does not document them

**Severity:** INFO
**Status:** CONFIRMED (verified in session against skill and help text)

### Plan text at issue

> Wave 0 (orchestrator, not a dispatched Work — CLI scaffolding per
> `AGENTS.md`'s "When NOT to use `sgt`" and the `estate-navigation` skill).
> [v2] Already run this session:
>
> ```sh
> sgt repo add sergeant-rs --origin https://github.com/miztertea/sergeant-rs.git
> git -C repos/sergeant-rs fetch origin integration/macbook-arrival-2026-08-15
> git -C repos/sergeant-rs checkout integration/macbook-arrival-2026-08-15
> ```

### Governing text

`skills/estate-navigation/SKILL.md`, "Bringing the working set up to date"
section:

> - **A repo not yet cloned, or not yet declared:**
>   `sgt repo add <name> --origin <url>` — clones `--origin` into
>   `repos/<name>` if the directory doesn't exist yet, or verifies it's
>   already a git repository if it does, then declares `[[repo]]`.
> - **An already-declared repo whose local clone is behind its remote:**
>   **no `sgt` verb covers this today** … Fall back to a manual per-repo
>   pull and say so:
>   ```sh
>   git -C repos/<name> pull --ff-only
>   ```

### Argument

The skill documents exactly two paths for the "bringing the working set up
to date" operation: `sgt repo add` for repos not yet declared, and
`git pull --ff-only` for already-declared repos that need updating. The plan
invokes the skill as authorisation for Wave 0, then executes three
operations:

1. `sgt repo add` — covered by the first skill path; verified by
   `sgt repo add --help` ("Clone `--origin` into `repos/<name>` if the
   directory does not exist yet … then add `[[repo]]`"). **Fully
   authorized.**
2. `git -C repos/sergeant-rs fetch origin integration/macbook-arrival-2026-08-15`
   — not mentioned in the skill.
3. `git -C repos/sergeant-rs checkout integration/macbook-arrival-2026-08-15`
   — not mentioned in the skill.

The skill's second path (`git pull --ff-only`) addresses "already-declared,
behind remote on the *current* branch." That is a different use case from
"fresh clone on main, need to switch to a non-default branch." `git pull
--ff-only` on main would not produce the integration-branch HEAD the plan
needs; the plan's `fetch + checkout` sequence is functionally correct for
its goal.

The issue is that the plan cites the skill as the authorising basis for the
entire Wave 0 sequence, but steps 2 and 3 are not covered by anything the
skill documents. A reader following the skill would find no authorization
for branch-switching on a fresh estate clone.

This is a gap in the skill's documented coverage rather than a plan
violation — the plan's git operations are safe and correct. But the plan
treats the skill's authorization as broader than it is, and does not
acknowledge the deviation or name it as a skill gap.

### Correction

Either: note explicitly in §4 that steps 2–3 are a branch-switch operation
beyond the skill's documented "pull existing" path (naming this as an honest
gap, per the skill's own honesty convention for undocumented cases); or open
a skill update that documents the branch-switch pattern alongside `git pull
--ff-only`.

---

## F2 — R6 enforcement is textual-only; validate-and-ship stage 30 documents a skip-capable default that R6 must override

**Severity:** WARNING
**Status:** CONFIRMED (verified in session against all six stage files)

### Plan text at issue

> R6 | `validate-and-ship` is never deferred, skipped, or run at a reduced
> profile — every stage runs in full, every time | WD's brief and dispatch
> carry no `--skip`/reduced-profile flag of any kind (R6) — the full
> seven-stage pipeline runs every time.

And §2 R6 note:

> if `no-mistakes` itself offers a "medium profile skips review/document
> stages" option (seen referenced in `20-select-intent-transport`'s own
> citations), WD's brief explicitly refuses it

And §7:

> **R6 enforcement**: before dispatching WD, the orchestrator confirms its
> brief carries no skip/reduced-profile instruction, and after dispatch
> confirms (via `sgt work show`/`no-mistakes axi status`) that every stage
> actually ran — a stage silently skipped is a defect to report, not an
> efficiency to accept.

### Governing text

`.sergeant/workflows/validate-and-ship/30-start-run/CONTEXT.md`, BU-P1-042:

> **sgt-validate's default medium profile skips the redundant no-mistakes
> review and document stages.**
> (trigger: a validation boundary is launched with the default profile;
> outcome: review/document stages are not duplicated when the coordinator
> already covered them)
> — `BU-P1-042`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L154-155)

`.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md`, BU-P1-072:

> **Do not use --yes; use --skip=<steps> only for stages already proven
> irrelevant — skipping is not a substitute for checks that have not been
> performed.**
> — `BU-P1-072`, `reference/sergeant-upstream/README.md` (README.md L275)

`.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md`, BU-P2-100:

> **`--yes` is the user's standing consent to drive every gate unattended …
> it should only be used when the user has asked to drive the whole run
> without checking back**
> — `BU-P2-100`, (lines 240-249)

### Argument

The `validate-and-ship` workflow's own stage files — all six read in session
— contain no mechanical blocker against skip/reduced-profile flags. The
stage files describe *procedure* and *judgment constraints*, not structural
enforcement. Specifically:

- Stage 30 (`30-start-run`) includes BU-P1-042 in its behavior contract,
  documenting that a medium profile which skips review and document stages
  is a known, tested behavior in the coordinator-launched path. This is a
  default behavior (triggered "when the coordinator already covered them"),
  not an opt-in. If WD's Work executor follows stage 30's default without
  an explicit full-profile override, stages will be skipped.
- Stage 40 (`40-drive-gates`) documents `--skip=<steps>` and `--yes` as
  real, available flags, with behavioral constraints (only for "stages
  already proven irrelevant" / only with explicit standing consent).
  Nothing in stage 40 mechanically prevents their use.
- Stages 50 and 60 contain no skip-related mechanisms.

R6's enforcement mechanism is entirely textual:
1. WD's brief says not to use skip/reduced-profile flags.
2. The orchestrator checks afterward that every stage ran.

Step 1 is an instruction to the Work executor — it can only be effective if
the executor follows it AND knows to actively override the default medium
profile. Step 2 is detection after the fact, not prevention.

R6 as stated ("every stage runs in full, every time") is not mechanically
enforceable by the stage files themselves; it relies on the brief correctly
mandating a full-profile invocation of `no-mistakes axi run` and the Work
executor following that instruction.

A secondary precision issue: the plan's parenthetical ("seen referenced in
`20-select-intent-transport`'s own citations") misattributes the medium-profile
reference. **Verified in session:** BU-P1-042 appears in `30-start-run`'s
CONTEXT.md, not in `20-select-intent-transport`'s. This misattribution does
not change R6's substance but is an inaccuracy in the plan's own rationale
for the ruling.

### Correction

WD's brief should not merely "refuse" the medium profile option — it should
*affirmatively mandate* a full-profile invocation (e.g., explicitly naming
the expected stages and requiring the executor to confirm all ran, or
specifying whatever flag or absence-of-flag produces full-profile behavior).
"Refuses X" in a brief is weaker than "requires Y" when X is the default
behavior. The plan's §7 post-dispatch verification ("confirms every stage
actually ran") is the right detection step but should be elevated to the
primary enforcement description, not framed as a secondary check.

Also correct §2 R6 note's stage attribution: "seen referenced in
`30-start-run`'s own citations," not `20-select-intent-transport`.

---

## F3 — Adjacent-append crash-window check specified for WB but not WC, which shares engine.rs in its file scope

**Severity:** WARNING
**Status:** CONFIRMED (verified in session against DEVELOPMENT.md and plan §3/§6)

### Plan text at issue

Plan §6, dispatch constraints for WA/WB:

> WA gets the bash-3.2-clean constraint (`docs/handoff/path-to-mac.md`
> step 8) and WB gets the adjacent-append crash-window check (**L6**,
> `docs/DEVELOPMENT.md:41`).

WC's brief is described only as:

> WC's brief states explicitly that "profiled, no safe fix found,
> documented instead" is an acceptable stage-30 outcome, not a failed Work

WC's file scope from plan §3:

> `tests/m2_daemon_api.rs`, submit-path code
> (`src/runtime/engine.rs`/`surface.rs`), `docs/perf/`

### Governing text

`docs/DEVELOPMENT.md`, "Architecture — the invariants that shape
everything," line 41:

> **Adjacent-append crash windows** are this architecture's recurring
> hazard (LESSONS L6): any path appending two causally-linked events must
> tolerate the second one missing or write one compound event. **Check for
> this class in review of any journal-touching change.**

### Argument

The architecture invariant is unconditional: "any journal-touching change."
It does not say "any journal-touching change in WB" or "any change to
recovery.rs." The check applies to all works in this sprint that may produce
code changes to `src/runtime/`.

Both WB and WC have `src/runtime/engine.rs` in their stated file scopes (§3
lists `src/runtime/engine.rs`/`surface.rs` for WC). WB's brief gets the
explicit adjacent-append crash-window instruction. WC's does not.

WC is framed as a profiling Work that may end without a code change, so the
risk is conditional. But the plan also says (§3) the Work "may end in a
code fix" — and `src/runtime/engine.rs` is explicitly in WC's file scope.
If WC's Work executor makes a journal-touching change to engine.rs and the
brief carries no adjacent-append constraint, that invariant check will not
be signalled. The `implement` workflow's `30-review` stage runs
`code-review`, but a reviewer has no reason to apply this specific check
unless instructed — it's an architecture-specific hazard the brief is
supposed to surface.

The asymmetry between WB and WC is especially notable because both Works
may touch the same file (`src/runtime/engine.rs`). The plan acknowledges the
overlap risk (§8 risk 4: "WC/WB file overlap risk") but only addresses it
as a merge-conflict concern — not as a missing invariant check in WC's
brief.

### Correction

Add the adjacent-append crash-window check to WC's brief, conditional on
whether WC produces a code change to `src/runtime/` — phrased as: "if any
fix is committed to `src/runtime/engine.rs` or `src/runtime/surface.rs`,
apply the adjacent-append crash-window check per `docs/DEVELOPMENT.md:41`
(LESSONS L6) before marking the Work complete."

---

## Items checked and not finding-worthy

The following checks produced no findings. Recorded so the adversarial
verifier knows what was examined.

### NORTH-STAR ownership boundaries

**Who owns dispatch?** Core owns "admission" (NORTH-STAR.md "Ownership"
section). The plan dispatches via `sgt run`, which routes through Core's
admission mechanism. The orchestrating session (Surface) "steers through
the API" — consistent with "Surfaces own presentation and steering through
the API only." No violation.

**Who owns merge to main?** Plan R1: "Only the owner merges to `main`; the
orchestrator merges freely against its own integration branch."
`docs/DEVELOPMENT.md` "Session conduct": "Never push directly to a default
branch." NORTH-STAR does not explicitly address orchestrator authority on
non-main integration branches. R1's restriction on main-merges is consistent
with DEVELOPMENT.md. The orchestrator's integration-branch merge authority
is neither explicitly granted nor forbidden by any named governing document.
**PLAUSIBLE** — owner-ruled, in-scope only if the ruling misrepresents what
was said; no governing text contradicts it.

**"A surface adds usability, never functionality"** (NORTH-STAR.md). The
plan proposes no new surface functionality: every orchestrator operation
uses existing `sgt` CLI verbs and existing workflows. No violation.

**R-NS-6 (execution ≠ dialogue):** Wave 0 is CLI setup, not a dispatched
Work; the orchestrator's monitoring and gate-adjudication steps are harness
operations, not engine work. No violation.

### AGENTS.md "When NOT to use sgt" boundary

Wave 0 is estate scaffolding, not a Work dispatch. AGENTS.md routing table
itself cross-references "When NOT to use `sgt`" for the estate-not-set-up
trigger. The plan's citation of this boundary for Wave 0 is the routing
table's own instruction. No violation.

WA/WB/WC are each: "contains two or more independent repository-owned tasks"
(three disjoint bug fixes) dispatched with `sgt run`, which is exactly the
dispatch trigger ("work that spans repositories, contains two or more
independent repository-owned tasks, needs an isolated independent-review
worker"). No violation.

### Ponytail Minimality Ladder

Every addition the plan proposes building:

| Addition | Rung | Argument |
|---|---|---|
| Estate registration (`sgt repo add`) | R2 | Existing `sgt` CLI verb |
| Branch-switch on estate clone (`git fetch + checkout`) | R4 | Native platform git operation |
| WA/WB/WC dispatch (`--workflow implement`) | R2 | Published workflow, already in estate |
| WD dispatch (`--workflow validate-and-ship`) | R2 | Published workflow, already in estate |
| Brief-as-file dispatch mechanic (`"$(cat file)"`) | R4 (shell feature) / R2 (prior sprint pattern) | Standard shell command substitution; plan cites prior sprint precedent |
| Watcher arming (`sgt --json watch --follow`) | R2 | Existing `sgt watch` verb |
| Post-dispatch stage verification (`sgt work show`/`no-mistakes axi status`) | R2 | Existing `sgt` surfaces |

No new abstractions, workflows, tools, or machinery proposed. No Ponytail
rung violations.

### sgt repo add --help vs. plan description

Plan §4: "`sgt repo add` clones a **fresh** copy of `--origin` into
`repos/<name>`; it does not adopt this primary checkout in place."

`sgt repo add --help` (run in session): "Clone `--origin` into `repos/<name>`
if the directory does not exist yet, or verify it is already a git repository
if it does, then add `[[repo]]`."

For the precondition stated in §4 (no prior `repos/sergeant-rs`), the
behavior is clone-fresh. The plan's characterization is accurate for its
stated precondition. The parenthetical "it does not adopt this primary
checkout in place" is correct: `--origin` is the remote URL
`https://github.com/miztertea/sergeant-rs.git`; `sgt repo add` clones from
that URL into `repos/`, independently of the current checkout. **Confirmed.**

### sgt init in Wave 0

The plan runs `sgt repo add` but not `sgt init`. `sgt init --help` (run in
session): "Scaffold an estate at the current directory (MVP-3): `[estate]`
in `sergeant.toml`, `repos/`, `.gitignore` entries … Idempotent." The plan
states `sgt doctor` reports "0 repositories declared" — meaning the estate
IS already initialized (init has run; estate section exists in
`sergeant.toml`) but no repos are declared. Running `sgt repo add` without
`sgt init` is correct when the estate is already scaffolded. **No missing
init step.**

### §8 risk 5 vs. stages 50/60

The plan names a residual risk (§8 risk 5): stages `50-reconcile-custody`
and `60-close-out` were not read before plan dispatch, leaving open the
possibility those stages reference `scripts/gate.sh`. Both stages read in
session. **Verified in session:** neither stage mentions `scripts/gate.sh`
or any skip mechanism. The risk named in §8 risk 5 is not substantiated by
the unread stages. (This is an enactability finding; recorded here only
because it was verified during this axis's research.)

---

*Written by the invariants critic seat, session 2026-08-15. Primary sources
only; every claim above is traced to a specific file and, where possible, a
line number.*
