# Enactability critic — macbook-arrival-2026-08-15

**Axis:** Can a Work execute each Wave-1/Wave-2 section as written, or does confident prose
hide an undecided question? Per contract (`docs/gauntlet/contracts/MACBOOK-ARRIVAL-1.md`
§ "Axes"): specifically, WC's open-ended acceptance criterion and whether WD's brief can
actually enforce R6 given the workflow's stage files were only partially read.

**Session:** 2026-08-15. Every primary source read in this session; all verified claims
are **verified in session**. Beliefs are labelled **believed**.

---

## Sources read

| Source | Why |
|---|---|
| `docs/gauntlet/contracts/MACBOOK-ARRIVAL-1.md` | Governing contract — axis spec |
| `docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md` (v3) | Artifact under review |
| `gh issue view 128/129/130 --repo miztertea/sergeant-rs` | Issue bodies cited by plan |
| `scripts/gate.sh` (full) | WA's subject; `daemon_env_ok()` implementation |
| `.sergeant/workflows/validate-and-ship/CONTEXT.md` | Workflow orientation |
| `.sergeant/workflows/validate-and-ship/00-check-scope/CONTEXT.md` | Entry-point mechanics |
| `.sergeant/workflows/validate-and-ship/20-select-intent-transport/CONTEXT.md` | Plan's v2 claim re: no gate.sh reference |
| `.sergeant/workflows/validate-and-ship/30-start-run/CONTEXT.md` | Plan's v2 claim; BU-P1-042 location |
| `.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md` | BU-P1-072 skip-constraint; R6 |
| `.sergeant/workflows/validate-and-ship/50-reconcile-custody/CONTEXT.md` | §8 risk-5 gap |
| `.sergeant/workflows/validate-and-ship/60-close-out/CONTEXT.md` | §8 risk-5 gap |
| `sgt run --help` | §4/§6 dispatch flags |
| `sgt repo add --help` | §4 estate-setup claims |
| `sgt watch --help` | §7 watcher claim (`--follow`) |
| `sgt work --help` (subcommand listing) | §7 verification claim (`sgt work show`) |

---

## Finding E1 — WARNING · CONFIRMED

### WC's "documented, no fix" outcome is structurally incompatible with `--workflow implement`'s TDD stage, and the plan's §7 verification rule contradicts the §6 exception without resolving it

**Exact plan text (§6, enactability claim):**
> "WC's brief states explicitly that 'profiled, no safe fix found, documented instead' is an
> acceptable stage-30 outcome, not a failed Work — its own issue text says so."

**Exact plan text (§6, workflow selection):**
> "`--workflow implement` for WA/WB/WC (bounded, single-outcome code fixes, each with a named
> acceptance test — `implement`'s `10-implement-with-tdd` stage delegates to `tdd`, `30-review`
> delegates to `code-review`)"

**Exact plan text (§6, acceptance-test discipline):**
> "Each brief states: the issue's full body (not just its title) as settled prior art; the exact
> acceptance test (the issue's own repro command where one exists)"

**Exact plan text (§7, orchestrator verification):**
> "No `completed` believed on its own: verify `git log base..sergeant/<id>` is non-empty and the
> named acceptance test actually passes, independently, the same discipline the prior sprint's #94
> finding demands."

**Governing primary source 1 — issue #128 repro command (verified in session):**
```
cargo test --test m2_daemon_api t12_submission_throughput_has_an_automated_floor -- --test-threads=1
```
This is the only repro command in the issue body. On a documentation-only outcome (no code fix, no
floor revision), this test **still fails**. The issue's own text confirms: "5.0 works/s ...
submission throughput fell to 5.0 works/s at burst 25, below the 12 works/s floor." The test is a
failing assertion about real performance; documenting the measurement leaves it failing.

**Governing primary source 2 — plan §6 naming `implement`'s TDD stage (believed to require
a green cycle; the `implement` workflow's own stage files were not read in this session):**
The plan says `implement`'s `10-implement-with-tdd` stage "delegates to `tdd`." The TDD discipline
is red→green: a Work that completes TDD without making the test green has not finished the stage.
Whether the `implement` workflow's own stage contract allows a documentation-only commit to satisfy
`10-implement-with-tdd` is **unknown in this session** — the implement workflow's stage files were
not read. The concern is therefore **PLAUSIBLE-to-CONFIRMED**: the plan asserts "documented outcome
is acceptable" without explaining which mechanism in `--workflow implement` produces `completed`
without a green acceptance test.

**The §7 contradiction:** The orchestrator's §7 rule is a blanket discipline: "verify ... the named
acceptance test actually passes." WC's named acceptance test is `t12_submission_throughput_has_an_automated_floor`. On a documentation-only path, the test does not pass. The §7 rule would catch
this as incomplete — contradicting the §6 claim that documentation-only is an acceptable outcome.
The plan names this as "risk 1" in §8 but does not resolve the contradiction:

> (§8 risk 1) "WC (#128) may not resolve to a fix at all. The issue's own text says profiling,
> not a guessed fix, is the actual scope — a Work here may legitimately end in 'documented,
> not fixed,' and that is an acceptable outcome per R-S0-1-style honesty, not a failed Work."

Naming the risk ≠ resolving it. The undecided question: when WC produces a documentation-only
commit and `t12_submission_throughput_has_an_automated_floor` still fails, does the orchestrator
treat the Work as completed or as failed? Neither the §7 rule nor the §6 brief resolves this.

**Severity: WARNING.** Not ERROR because:
1. WC also has a third valid outcome the plan allows: "propose a floor revision" with measured
   justification. A floor revision would make the test pass, eliminating the tension. The pure
   "documented, no fix, no floor revision" path is where the gap lives — and only if the Work
   cannot find a justifiable floor.
2. The plan names the risk rather than hiding it.

**What a correction looks like:**
Option A: Add a WC-specific carve-out to §7 — "for WC, 'the acceptance test passes' is satisfied
by either (a) `t12` passing, or (b) a measured disposition (floor revision or documented
non-fix) committed on the branch and the orchestrator confirms the disposition explicitly with the
owner." 

Option B: Dispatch WC as `--workflow research` (or a custom brief matching a profile that doesn't
require a green acceptance test), with a note that if a fix is found during research, WC is
re-dispatched as `--workflow implement`.

Either correction resolves the ambiguity; absent one, a dispatched WC Work faces a structurally
undefined completion criterion when the code fix path closes out.

---

## Finding E2 — INFO · CONFIRMED

### Plan's R6 medium-profile citation points to the wrong stage file; the actual mechanism is in `30-start-run`, not `20-select-intent-transport`; and it describes `sgt-validate`'s behavior, not `axi run`'s

**Exact plan text (§2 R6):**
> "validate-and-ship is never deferred, skipped, or run at a reduced profile — every stage runs in
> full, every time, no exceptions (§6, §7). WD's brief and dispatch carry no `--skip`/reduced-
> profile flag of any kind; if `no-mistakes` itself offers a 'medium profile skips review/document
> stages' option (seen referenced in `20-select-intent-transport`'s own citations), WD's brief
> explicitly refuses it"

**Governing primary source 1 — `30-start-run/CONTEXT.md` lines 66–68 (verified in session):**
```
- **sgt-validate's default medium profile skips the redundant no-mistakes review and document stages.**
  (trigger: a validation boundary is launched with the default profile; outcome: review/document
  stages are not duplicated when the coordinator already covered them)
  — `BU-P1-042`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L154-155)
```
BU-P1-042 is cited in `30-start-run/CONTEXT.md`, **not** `20-select-intent-transport/CONTEXT.md`.
`20-select-intent-transport/CONTEXT.md` was read in full in this session; it contains no reference
to a medium profile or BU-P1-042.

**Governing primary source 2 — BU-P1-042's trigger and scope (verified in session):**
The trigger is "a validation boundary is launched with the default profile" — this describes
`sgt-validate`'s behavior when a coordinator validates a worker's reviewed commit. The
validate-and-ship workflow's stages call `no-mistakes axi run` directly (confirmed by reading
stages 20, 30, 40 — none invoke `sgt-validate`). When WD is dispatched via `sgt run --workflow
validate-and-ship` and the stages call `axi run`, the `sgt-validate` medium profile is **not in
the dispatch path**. The misattribution is therefore doubly wrong: wrong stage file, and wrong
tool path.

**Governing primary source 3 — the actual skip constraint for `axi run`, `40-drive-gates/CONTEXT.md` lines 72–74 (verified in session):**
```
- **Do not use --yes; use --skip=<steps> only for stages already proven irrelevant — skipping is
  not a substitute for checks that have not been performed.**
  — `BU-P1-072`, `reference/sergeant-upstream/README.md` (README.md L275)
```
The real `--skip` risk for WD is `axi run --skip=<steps>`. This is already constrained by the
stage file itself: "only for stages already proven irrelevant." WD's brief reinforcing R6 is
consistent with — and complementary to — this existing stage-level constraint.

**Consequence for enactability:** The misattribution doesn't create a new enactability gap. The
R6 concern is real (a `--skip=<steps>` flag exists for `axi run`), the plan's brief-level
instruction is the right control, and the stage file itself already constrains the mechanism. The
plan's reasoning path is: "the medium profile from stage 20" → "brief refuses it." The correct
path is: "the `--skip` flag on `axi run` (BU-P1-072, in stage 40)" → "brief refuses it; stage
file itself constrains it to proven-irrelevant only."

**Severity: INFO.** The R6 control works; the plan's citation is wrong in two ways (wrong stage,
wrong tool), but the underlying concern is real and the dual-control mechanism (brief + post-hoc
orchestrator verification via `sgt work show`/`no-mistakes axi status`) is sound.

**What a correction looks like:**
Replace `"(seen referenced in `20-select-intent-transport`'s own citations)"` with:
`"(BU-P1-042, cited in `30-start-run`'s behavior contract, which describes sgt-validate's
medium profile — not axi run's; the axi-run skip risk is BU-P1-072 in `40-drive-gates`, already
limited by that stage's own contract to 'stages already proven irrelevant')"`

---

## Finding E3 — INFO · CONFIRMED (gap resolved in session)

### §8 risk 5's unread stages (50-reconcile-custody, 60-close-out) contain no gate.sh reference and no skip mechanism; the residual risk from risk 5 is no-mistakes' own internals only

**Exact plan text (§8 risk 5):**
> "#130 may still surface inside WD anyway, if `validate-and-ship`'s pipeline turns out to shell
> out to `scripts/gate.sh` somewhere this plan's read of `20-select-intent-transport`/`30-start-run`/
> `40-drive-gates` didn't catch — those three stages are what's been read; `50-reconcile-custody`/
> `60-close-out` and `no-mistakes`' own internals were not."

**Governing primary source — `50-reconcile-custody/CONTEXT.md` (verified in session, full read):**
The stage's entire behavior contract concerns `branch_sync` object processing: `axi sync`,
`continue_active_run`, `recover_custody`, `--keep-local`. No reference to `scripts/gate.sh`.
No stage-skip mechanism. No call to any external script.

**Governing primary source — `60-close-out/CONTEXT.md` (verified in session, full read):**
The stage's behavior contract concerns pipeline outcome routing: `checks-passed`, `passed`,
`failed`, `cancelled`. On `failed`/`cancelled`, the Work "fixes on the same branch and re-drives"
— no reference to `scripts/gate.sh`, no skip mechanism, no way to introduce #130's `/proc` error.

**Consequence for enactability:** §8 risk 5 correctly identifies the boundary of what was read.
Reading both stages now confirms: the residual risk from not reading them is zero. The only
remaining unread code path where #130 could appear is `no-mistakes`' own internals — which the
plan correctly names and correctly characterizes as a separate, smaller risk: "If WD hits #130's
exact failure mode, that is new evidence this plan's v2 correction was itself incomplete, not a
WD-owned bug — file it, don't force past it." That disposition is sound and is NOT affected by
anything in 50 or 60.

**Severity: INFO.** No correction needed. The risk-5 gap does not affect enactability.

---

## Checked and cleared (no finding)

### WA — "fix or narrow daemon_env_ok() for macOS" is actionable; no acceptance test is available, but the plan's "where one exists" carve-out covers it

**Plan text (§5):** "fix or narrow `daemon_env_ok()` for macOS"
**Plan text (§6):** "the exact acceptance test (the issue's own repro command **where one exists**)"

Issue #130 (verified in session) has no `cargo test` repro command. The failure is a bash error
from `scripts/gate.sh:89–96`. The issue body provides a concrete candidate fix: "skip
`daemon_env_ok`'s check specifically on the direct-fork (non-systemd) path." The plan says the
brief will include "the issue's full body" — so the Work has: the failure mode, the candidate fix
shape, the constraint (bash-3.2-clean per `docs/handoff/path-to-mac.md` step 8), and a testable
criterion (gate.sh must not exit with the /proc error when run on this MacBook). The estate IS
on macOS, so the Work can run gate.sh to verify. The "where one exists" carve-out in the plan is
deliberate and correct. **No enactability gap.**

### WB — acceptance test is concrete and fully specified

**Plan text (§5):** "root-cause and fix the extra OBSERVE"
**Issue #129 repro (verified in session):**
```sh
cargo test --test m3_execution t8_restart_resumes_unambiguous_work_and_blocks_ambiguous_work -- --test-threads=1
```
This is a deterministic-fail cargo test: the issue says "reproduces every time, including alone
with `--test-threads=1`." The Work has: full issue body, repro command, candidate areas
(tokio scheduling vs. latent reconciliation race), the adjacent-append crash-window check
constraint from the brief. **Enactable as written.**

### WD dispatch flags — all verified valid

`sgt run --help` (verified in session) confirms: `--backend`, `--workflow`, `--turns`,
`--ceiling-secs`, `--repo`, `--json` are all valid flags. `--profile` is also valid (the plan
correctly notes it was tried and refused at Wave 0 because the estate has no profile sections —
consistent with `sgt run --help` showing the flag exists but profiles being estate-defined).

### sgt watch --follow — verified valid

`sgt watch --help` (verified in session): `--follow` exists and behaves exactly as the plan
needs — "Remain attached after a nonterminal match... exits once the Work reaches
`completed`/`canceled`." Plan §7's "Watchers armed... backgrounded (`sgt --json watch <id>` /
`--follow`)" is mechanically sound.

### sgt work show — verified valid

`sgt work --help` (verified in session) lists `show` as a subcommand: "Show one work item."
Plan §7's "confirm (via `sgt work show`/`no-mistakes axi status`) that every stage actually ran"
is a valid post-hoc verification mechanism. **No gap.**

### sgt repo add --origin — matches plan §4

`sgt repo add --help` (verified in session): "clone `--origin` into `repos/<name>` if the
directory does not exist yet, or verify it is already a git repository if it does." Plan §4 says
"`sgt repo add` clones a fresh copy of `--origin` into `repos/<name>`; it does not adopt this
primary checkout in place." This is correct — the `--origin` flag is required unless `repos/<name>`
already exists. The three Wave-0 commands in §4 are syntactically valid and semantically correct.

### validate-and-ship stages 20/30/40 do not reference scripts/gate.sh — confirmed

Plan v2 note: "reading `.sergeant/workflows/validate-and-ship`'s actual stage files
(`20-select-intent-transport`, `30-start-run`, `40-drive-gates`) — none of them reference
`scripts/gate.sh`." Read all three in session: confirmed. Each stage invokes `axi run`/`axi
respond`/`axi status` from no-mistakes directly. `scripts/gate.sh` does not appear.

---

## Summary table

| ID | Scope | Severity | Verdict | Short statement |
|---|---|---|---|---|
| E1 | WC | WARNING | CONFIRMED | "documented, no fix" has no clear completion path through `--workflow implement`'s TDD stage; §7 verification rule contradicts §6's exception without resolving it |
| E2 | WD / R6 | INFO | CONFIRMED | Medium-profile citation (BU-P1-042) is in `30-start-run`, not `20-select-intent-transport`; describes sgt-validate, not axi run; actual skip constraint (BU-P1-072) is already in stage 40's own contract |
| E3 | §8 risk 5 | INFO | CONFIRMED (gap resolved) | 50-reconcile-custody and 60-close-out verified gate.sh-free and skip-mechanism-free; residual risk 5 is no-mistakes internals only, correctly named by plan |
| — | WA | — | NO FINDING | "where one exists" carve-out covers missing cargo-test; Work can run gate.sh on-host |
| — | WB | — | NO FINDING | Acceptance test is concrete (`t8_...` command matches issue #129 repro) |
| — | WD flags | — | NO FINDING | All `sgt run` flags verified valid |
| — | WD watchers | — | NO FINDING | `sgt watch --follow` and `sgt work show` both verified valid |

**Governing axiom applied:** A finding that can be neither confirmed nor refuted is recorded
PLAUSIBLE, never dropped (`docs/gauntlet/contracts/MACBOOK-ARRIVAL-1.md` §RA "Rules that outrank
the loop"). No findings reached PLAUSIBLE-only status; E1 is CONFIRMED by the contradicting plan
texts themselves; E2/E3 are CONFIRMED by direct source reads.
