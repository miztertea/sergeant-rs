# Fidelity critic — MacBook-arrival plan

**Axis:** fidelity — do `docs/DEVELOPMENT.md`, ADR 0005, the estate-navigation
skill, and issues #128/#129/#130 actually say what the plan says they say?

**Reviewer:** Sonnet 4.6, blind seat, fresh context.
**Date:** 2026-08-15.
**Artifact:** `docs/gauntlet/runs/macbook-arrival-2026-08-15/plan.md` (v3).

## Sources read in-session (verified)

| Source | How read |
|---|---|
| `docs/DEVELOPMENT.md` | Full file, in-session |
| `docs/adr/0005-gating-becomes-a-dispatched-work.md` | Full file, in-session |
| `skills/estate-navigation/SKILL.md` | Full file, in-session |
| `AGENTS.md` | Full file, in-session (not a symlink — regular file, 14 754 bytes) |
| GitHub issue #128 | `gh issue view 128 --repo miztertea/sergeant-rs`, in-session |
| GitHub issue #129 | `gh issue view 129 --repo miztertea/sergeant-rs`, in-session |
| GitHub issue #130 | `gh issue view 130 --repo miztertea/sergeant-rs`, in-session |
| `.sergeant/workflows/validate-and-ship/CONTEXT.md` | Full file, in-session |
| `.sergeant/workflows/validate-and-ship/20-select-intent-transport/CONTEXT.md` | Full file, in-session |
| `.sergeant/workflows/validate-and-ship/30-start-run/CONTEXT.md` | Full file, in-session |
| `.sergeant/workflows/validate-and-ship/40-drive-gates/CONTEXT.md` | Full file, in-session |
| `.sergeant/workflows/implement/CONTEXT.md` | Full file, in-session |
| `.sergeant/workflows/tdd/CONTEXT.md` | Full file, in-session |

Sources I could not read in-session and therefore cannot verify claims against:

- `LESSONS.md` (L6, L7, L12, L15, L20 references) — not read; plan calls these beliefs or reads them as cross-references from DEVELOPMENT.md; L6/L15 appear in DEVELOPMENT.md by name and are consistent there
- `docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` §6 — not read per task scope ("Do NOT read…unless the plan cites something specific there you need to check"; this is cited only as a precedent for a dispatch pattern, not a primary source)
- `docs/handoff/path-to-mac.md` step 8 — not read; cited as the source of WA's bash-3.2-clean constraint
- `src/runtime/surface.rs:332,431-441` — not read; cited as the source for the estate-clone branch-cutting fact (plan attributes it to the prior sprint's own plan, not directly read from source)

---

## Findings

### F1 — Medium-profile citation misattributed to `20-select-intent-transport` [WARNING — CONFIRMED]

**Plan text** (§2, R6):

> "if `no-mistakes` itself offers a 'medium profile skips review/document stages' option
> (seen referenced in `20-select-intent-transport`'s own citations), WD's brief explicitly
> refuses it"

**What the sources actually say:**

The medium-profile citation (`BU-P1-042`: *"sgt-validate's default medium profile skips the
redundant no-mistakes review and document stages"*) appears in
**`.sergeant/workflows/validate-and-ship/30-start-run/CONTEXT.md`** (lines 66–68), not in
`20-select-intent-transport/CONTEXT.md`.

I read both files in full. `20-select-intent-transport/CONTEXT.md` contains no mention of
"medium profile", "review and document stages", or `BU-P1-042`. The citation lives in
`30-start-run/CONTEXT.md`:

> ```
> - **sgt-validate's default medium profile skips the redundant no-mistakes review and document stages.**
>   (trigger: a validation boundary is launched with the default profile; outcome: review/document
>    stages are not duplicated when the coordinator already covered them)
>   — `BU-P1-042`, `reference/sergeant-upstream/AGENTS.md` (AGENTS.md L154-155)
> ```
> — `.sergeant/workflows/validate-and-ship/30-start-run/CONTEXT.md`, lines 66–68

**Severity:** WARNING. The substantive claim is correct — the workflow does cite a medium-profile
option that skips stages, and R6's refusal of it is consistent with the actual behavior contract.
The error is the stage-file attribution: `30-start-run`, not `20-select-intent-transport`. A
correction to plan §2 R6 would read: "seen referenced in `30-start-run`'s own citations."

**Correction:** Replace "`20-select-intent-transport`'s own citations" with
"`30-start-run`'s own citations".

---

### F2 — Three validate-and-ship stage files do not reference `scripts/gate.sh` [CONFIRMED — no error]

**Plan text** (§v2 header note):

> "Verified by reading `.sergeant/workflows/validate-and-ship`'s actual stage files
> (`20-select-intent-transport`, `30-start-run`, `40-drive-gates`) — none of them reference
> `scripts/gate.sh`; every one invokes `no-mistakes axi run`/`axi respond`/`axi status` directly."

**Verified in-session:** All three stage `CONTEXT.md` files have been read. None of them
contain the string `scripts/gate.sh` or any reference to the gate script. All three
reference `no-mistakes`/`axi` verbs:
- `20-select-intent-transport`: references `axi run --help` (as a probing invocation, BU-P7-105)
- `30-start-run`: references `no-mistakes axi run`, `axi status`, `axi respond`, `axi abort`
- `40-drive-gates`: references `axi run`, `axi respond`, `axi status`, `axi abort`

**Result:** Plan's claim is accurate. No finding.

---

### F3 — `implement` workflow delegation claims [CONFIRMED — no error]

**Plan text** (§6):

> "`--workflow implement` for WA/WB/WC (bounded, single-outcome code fixes,
> each with a named acceptance test — `implement`'s `10-implement-with-tdd`
> stage delegates to `tdd`, `30-review` delegates to `code-review`)"

**Verified in-session** from `.sergeant/workflows/implement/CONTEXT.md`, "Relationships to other workflows":

> ```
> - `10-implement-with-tdd` delegates to **tdd**.
> - `30-review` delegates to **code-review**.
> ```

**Result:** Exact match. No finding.

---

### F4 — ADR 0005 D1 authority-split citation [CONFIRMED — no error]

**Plan text** (§7):

> "WD's findings: `auto-fix` on the Work's own judgment for mechanical/
> low-risk items, `no-op` recorded, `ask-user` relayed to the owner verbatim
> and never resolved autonomously (ADR 0005, part of D1)."

**Verified in-session** from ADR 0005, "Decision" section:

> "**The auto-fix / no-op / ask-user authority split is unchanged (part of D1).**
> `validate-and-ship`'s `40-drive-gates` stage already classifies every
> finding into one of three actions: `auto-fix` (mechanical/low-risk, the
> actor may authorize on its own judgment), `no-op` (informational, nothing
> to do), or `ask-user` (challenges the user's deliberate intent or touches
> product behavior — a decision only the user can make, relayed verbatim and
> never resolved autonomously)."
> — `docs/adr/0005-gating-becomes-a-dispatched-work.md`

**Result:** Verbatim match. No finding.

---

### F5 — Issue #130 candidate-fix characterization [CONFIRMED — no error]

**Plan text** (§3, #130 row):

> "the issue's own body sketches a candidate fix (skip the check on the direct-fork/
> non-systemd path, keep it where a supervisor could silently drop env)"

**Verified in-session** from `gh issue view 130`:

> "A cheaper interim fix might be to skip `daemon_env_ok`'s check specifically on the
> direct-fork (non-systemd) path — where the inheritance argument already gives confidence —
> and keep it as a hard requirement only where a supervisor (systemd) could silently drop
> the env, which is the scenario that motivated this check in the first place."

**Result:** Accurate paraphrase. No finding.

---

### F6 — Issue #129 characterization [CONFIRMED — no error]

**Plan text** (§3, #129 row):

> "Deterministic extra `OBSERVE` in restart reconciliation (3 vs. expected 2),
> pure engine logic, no OS process involved — genuine engine-logic debugging"

**Verified in-session** from `gh issue view 129`:

> "Three OBSERVEs recorded against the survivor's `first_execution` id, where
> the test expects exactly two (one live, pre-restart; one from restart reconciliation)."
> "uses only `FakeBackend` (no real OS process for the backend under test, no
> `/proc`/`ps` involved) — this is pure engine/recovery logic"

**Result:** Accurate. No finding.

---

### F7 — Issue #128 characterization [CONFIRMED — no error]

**Plan text** (§3, #128 row):

> "Submission throughput floor measures ~5 works/s vs. a 12 works/s floor on this hardware —
> real profiling needed, not a guessed fix"

**Verified in-session** from `gh issue view 128`:

> "burst 25 (full submit path): 5.0 works/s in 4.994275917s, submission throughput fell to
> 5.0 works/s at burst 25, below the 12 works/s floor"
> "needs actual profiling, not a plausible guess. Filing rather than adjusting the floor or
> the code — lowering the floor without a measured justification would misrepresent a real
> finding as a decision."

**Result:** Accurate. No finding.

---

### F8 — DEVELOPMENT.md line 41 citation [CONFIRMED — no error]

**Plan text** (§6):

> "WB gets the adjacent-append crash-window check (**L6**, `docs/DEVELOPMENT.md:41`)"

**Verified in-session** from `docs/DEVELOPMENT.md` line 41:

> "**Adjacent-append crash windows** are this architecture's recurring hazard (LESSONS L6):
> any path appending two causally-linked events must tolerate the second one missing or write
> one compound event. Check for this class in review of any journal-touching change."

**Result:** Correct line and content. No finding.

---

### F9 — AGENTS.md "When NOT to use `sgt`" citation for Wave 0 [PLAUSIBLE]

**Plan text** (§4):

> "Wave 0 (orchestrator, not a dispatched Work — CLI scaffolding per
> `AGENTS.md`'s "When NOT to use `sgt`" and the `estate-navigation` skill)"

**What the sources say:**

The `AGENTS.md` routing table (lines 46–48) contains: "The estate isn't set up yet, or `sgt
doctor` reports a fixable install/config fault | `sgt init` / `sgt doctor` (not a skill —
CLI verbs; see "When NOT to use `sgt`")". This cross-references the section the plan cites,
though the routing table itself is a stronger and more direct support than the "When NOT to
use `sgt`" section text, which focuses on when not to dispatch `sgt run` Work items (not
specifically on CLI estate-setup verbs). The `estate-navigation` SKILL.md directly confirms
`sgt repo add` as the correct path for adding a missing repo.

**Verdict:** PLAUSIBLE. The citation is defensible through the routing table's own
cross-reference, but the named section's text is about `sgt run` dispatch judgment rather
than estate scaffolding directly. The estate-navigation skill citation is the stronger and
more direct source for the specific Wave 0 commands.

**Severity:** INFO. The plan's conclusion (Wave 0 is orchestrator-run CLI scaffolding, not
a dispatched Work) is well-supported by both sources; only the citation precision is
imperfect.

---

### F10 — Prior sprint dispatch pattern citation [PLAUSIBLE]

**Plan text** (§6):

> "This is the same pattern the prior sprint's plan used
> (`docs/gauntlet/runs/path-to-mac-2026-08-15/plan.md` §6:
> "Briefs are files, passed `"$(cat …)"`"), adapted to this session's scratchpad path"

**Verdict:** PLAUSIBLE. The cited file was not read in-session (per scope: that run's
directory is off-limits unless a specific claim needed checking; this is a pedigree claim,
not a governing-document claim). Cannot confirm or deny that §6 of that plan uses that
exact phrasing. The dispatch pattern itself (briefs in files, `"$(cat file)"` shell
substitution) is mechanically sound regardless of precedent; the precedent citation is
background, not load-bearing.

**Severity:** INFO.

---

### F11 — `src/runtime/surface.rs:332,431-441` line citation [PLAUSIBLE]

**Plan text** (§4):

> "Every Work below cuts its own branch from the **estate clone's** current
> HEAD (`src/runtime/surface.rs:332,431-441`, cited in the prior sprint's own plan)"

**Verdict:** PLAUSIBLE. The plan explicitly attributes this citation to the prior sprint's
plan rather than claiming it was read directly this session. The source file was not read
in-session. The underlying claim (Works cut branches from the estate clone) is consistent
with everything else observed (ADR 0005's "Consequences" section discusses `sergeant/<work-id>`
branch minting and surface mechanics).

**Severity:** INFO.

---

### F12 — `docs/handoff/path-to-mac.md` step 8 [PLAUSIBLE]

**Plan text** (§6):

> "WA gets the bash-3.2-clean constraint (`docs/handoff/path-to-mac.md` step 8)"

**Verdict:** PLAUSIBLE. `docs/handoff/path-to-mac.md` was not read in-session. Cannot
confirm the file exists, that it has a step 8, or that step 8 names a bash-3.2-clean
constraint.

**Severity:** INFO. If the file or step does not exist, a Work executing WA's brief would
receive a citation that cannot be resolved; that would be an enactability defect, not a
fidelity one under this axis.

---

## Summary table

| ID | Plan location | Severity | Verdict | Description |
|---|---|---|---|---|
| F1 | §2 R6 | WARNING | CONFIRMED | Medium-profile citation misattributed to `20-select-intent-transport`; it is in `30-start-run` |
| F2 | §v2 header | — | CONFIRMED OK | Stage files correctly claimed to not reference `scripts/gate.sh` |
| F3 | §6 | — | CONFIRMED OK | `implement` delegation to `tdd`/`code-review` matches CONTEXT.md exactly |
| F4 | §7 | — | CONFIRMED OK | ADR 0005 D1 authority-split citation accurate |
| F5 | §3 #130 row | — | CONFIRMED OK | Issue #130 fix sketch accurately paraphrased |
| F6 | §3 #129 row | — | CONFIRMED OK | Issue #129 characterization accurate |
| F7 | §3 #128 row | — | CONFIRMED OK | Issue #128 characterization accurate |
| F8 | §6 | — | CONFIRMED OK | `DEVELOPMENT.md:41` adjacent-append citation correct |
| F9 | §4 | INFO | PLAUSIBLE | AGENTS.md "When NOT to use `sgt`" works via routing-table cross-reference, not directly |
| F10 | §6 | INFO | PLAUSIBLE | Prior sprint dispatch pattern not verified (off-limits file) |
| F11 | §4 | INFO | PLAUSIBLE | `surface.rs:332,431-441` not verified (cited as from prior sprint, not read direct) |
| F12 | §6 | INFO | PLAUSIBLE | `path-to-mac.md` step 8 not verified (file not read) |

**One real finding (F1, WARNING).** The plan contains one confirmed misattribution: the
"medium profile" citation belongs to `30-start-run/CONTEXT.md`, not
`20-select-intent-transport/CONTEXT.md`. The decision it supports (R6: refuse any
reduced-profile dispatch) is correct and the actual source for that behavior exists in the
workflow; only the stage-file label is wrong.

**Four PLAUSIBLE findings (F9–F12)** are claims this critic could neither confirm nor deny
from the sources read in-session. None of them are load-bearing for the plan's three-Work
sprint or its gate Work: the plan's substantive decisions rest on sources that were verified.
