# Classification Ledger

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`classification-ledger.md`). This is the **adjudication surface**: where a
reviewer starts to challenge workflow/stage boundaries, cross-partition
disagreements, and low-confidence extractions before the corpus is frozen
(`docs/gauntlet/contracts/N1.md`'s Method: "Refute (≥2 independent reviewers
challenging workflow/stage boundaries, missing behaviors, and every
engine-gap claim) → Adjudicate (evidence-only; rulings recorded) → Freeze").

**Adjudication status of this document.** The eight partitions (P1–P8) were
extracted largely independently before `synthesis.md`'s cross-partition pass
merged, re-split, and overruled their proposals — the twenty conflicts in §3
below are exactly the disagreements that independent extraction surfaced, and
`synthesis.md` §8's refute-stage notes (reproduced in §6 below) are the
specific challenges the synthesis itself flags as its weakest points. A
second, independent round of reviewers (boundary-honesty findings N1-BH-*,
completeness/invention findings N1-R3-*) has now re-attacked those
boundaries, and the orchestrator's rulings are recorded in
`adjudication-round1.md`. Per that document's A14: two independent reviewers
challenged the §3 conflict set and objected only to X1 and X8 (plus X3's
citation) — every entry below now carries an **ADJUDICATED** status citing
its ruling, in place of the prior blanket **PROPOSED**. See the "Round-1
adjudication" section at the end of this document for the full summary; any
residual disagreement a future reviewer wants to reopen still follows the
same rule — cite specific units/sections and either accept, amend, or record
a new conflict, never silently resolve by a single pass.

---

## 1. Per-partition unit counts

*Round-1 adjudication added/split units: totals below are post-adjudication
(979), not the 966 `synthesis.md` was built against. P1 (+6, the AGENTS.md
routing table), P5 (+4, `dispatch`'s remaining worker-contract items), P6
(+1, split off `BU-P6-129`), and P8 (+2, split from retired `BU-P8-077`)
changed; see `adjudication-round1.md` A10/A12 and `provenance-map.md`'s
matching coverage-summary note.*

| Partition | Units | Source scope (`source-inventory.md`) |
|---|---|---|
| P1 | 137 | Root (`AGENTS.md`, `README.md`) — coordinator/executor invariants, direct-mode/dispatch/no-mistakes AGENTS.md-level rules |
| P2 | 127 | `.agents/skills/` dev-skills-a — `code-review`, `diagnosing-bugs`, `implement`, `no-mistakes` (reference), `tdd` |
| P3 | 96 | `.agents/skills/` dev-skills-b — `grilling`, `grill-with-docs`, `prototype`, `research`, `resolving-merge-conflicts`, `triage` |
| P4 | 100 | `.agents/skills/` design-skills — `deepen-module`, domain-modeling, `to-spec`, `to-tickets`, `wayfinder` |
| P5 | 153 | `.agents/skills/sergeant-setup/` + `skills/` ops-skills — `sergeant-setup`, `load-project`, `cross-repo-work`, `dispatch` (skill text), `sergeant-help`, `wiki` |
| P6 | 143 | `bin/` (shared libs + commands), `mise.toml`, `opencode.json` — drain, wake, dispatch/respond/recover machinery, DAG hooks, callback plumbing |
| P7 | 112 | `tests/` — response/lease/notification lifecycle, dispatch/drain/recovery/cleanup, worker/pane lifecycle, validate/no-mistakes launch |
| P8 | 111 | `docs/` root-level (`callbacks.md`, `troubleshooting.md`, `using-sergeant.md`, schema docs) — operator-facing behavior and durability contracts |
| **Total** | **979** | 139 `decompose`-dispositioned files of 179 total (see `provenance-map.md`) |

No duplicate ids across partitions (verified at `synthesis.md`'s input step,
when the corpus stood at 966; not re-verified against the 13 round-1 ids,
though none collides by construction — new ids continue each partition's
existing numeric range).

## 2. Confidence distribution

*Round-1 adjudication added/split units: figures below are re-derived by
script against the current `behavior-units/*.ndjson` (979 units), not the
966-unit distribution originally recorded here. Beyond the thirteen new
units' own confidence (all `high`), several existing units'
confidence changed independent of the count — most visibly `BU-P6-129` and
`BU-P8-077`, both demoted `high` → `low` per A10/A12's narrowing rulings —
so partitions whose *unit count* didn't move (P7) still show a changed
distribution.*

**Corpus-wide:** `high` 929 · `medium` 42 · `low` 8.

| Partition | high | medium | low |
|---|---|---|---|
| P1 | 134 | 3 | 0 |
| P2 | 119 | 8 | 0 |
| P3 | 95 | 1 | 0 |
| P4 | 99 | 1 | 0 |
| P5 | 150 | 3 | 0 |
| P6 | 138 | 4 | 1 |
| P7 | 103 | 8 | 1 |
| P8 | 91 | 14 | 6 |
| **Total** | **929** | **42** | **8** |

P8 (root-level `docs/`) carries both the largest `low`-confidence share (6
of 8) and the largest `medium` share (14 of 42) — consistent with `docs/`
prose requiring more inference from stated behavior to normalized statement
than a test file's literal assertions or a skill's imperative instructions.
This is a distribution finding, not a defect: `confidence` records how
directly the source supports the statement, and documentation is
structurally the most interpretive source class in the corpus.

**Representation × confidence is not cross-tabulated here** — every
`engine-gap` claim's confidence is recorded per-claim inside
`engine-pressure.md` (e.g. G3 and G5 are both `medium`, correctly, per
`synthesis.md` §5); this ledger tracks confidence as a corpus-wide property,
not duplicated per representation.

## 3. Cross-partition conflicts

Twenty entries, unchanged in substance from `synthesis.md` §6's own record
(**Type S** = semantic contradiction, **Type C** = classification
contradiction, **Type B** = boundary/naming contradiction). Each carries its
original citations plus a **PROPOSED** resolution for the next adjudication
round to accept or overturn.

### X1 — Cross-filesystem cleanup (Type S)
**Conflict:** Sergeant **refuses** a cross-filesystem layout between fleet
state and worktrees rather than falling back from atomic rename, vs cleanup
**must support** a copy-based cross-filesystem fallback with a CRITICAL
diagnostic on rollback failure.
**Citations:** BU-P8-108 (`docs/troubleshooting.md`) vs BU-P7-079
(`tests/sgt-cleanup-cross-filesystem-test.sh`).
**ADJUDICATED (adjudication-round1.md A13 — overturned):** adopt BU-P7-079's
copy-based cross-filesystem fallback with a CRITICAL rollback diagnostic as
the corpus's canonical extracted behavior — tests outrank documentation per
§5 of the extraction method (the binding rule). The original PROPOSED
resolution's reason for preferring BU-P8-108 was circular: it discounted
BU-P7-079's evidentiary weight by citing `engine-pressure.md` G9's rejection
of BU-P7-079, but that G9 rejection in turn cited this same X1 ruling (P8
wins) as its own reason to prefer BU-P8-108. A13 severs the circularity and
lets the test-backed unit win on its own §5 priority, unconditionally. This
is a ruling about which *behavioral statement the corpus extracted*, not
about sergeant-rs's engine — `engine-pressure.md` G9 separately re-derives,
on independent architectural evidence (not on this ledger's citation fight),
that sergeant-rs's actual data-dir/surface topology makes the cross-
filesystem split BU-P7-079 describes structurally impossible today; see
G9's revised entry. BU-P8-108 is retained here as provenance for the
invariant the shipped architecture happens to satisfy by construction, not
as the winning citation.

### X2 — Graph output location (Type S)
**Conflict:** output is **never** published inside an owning source
repository, vs the output directory **may** sit inside a source repo with
Sergeant staging extraction outside it before publish.
**Citations:** BU-P5-107, BU-P5-100 (`skills/load-project/SKILL.md`) vs
BU-P7-003 (`tests/`), BU-P8-035, BU-P8-036 (`docs/schema.md`).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the tests/schema position (P7/P8) as the operative rule —
the output directory may sit inside a source repo, with Sergeant handling
re-ingestion by staging extraction outside it before publish. Downgrade
BU-P5-107/BU-P5-100 from an absolute prohibition to a **documented
recommendation** ("prefer a location outside every source repo") inside W2
`project-graph`'s stage `00-resolve-output-path` context — the skill text
stated a recommendation as an absolute, which tests and schema both
contradict.

### X3 — Shipping-gate unattended consent (Type S)
**Conflict:** `--yes` must **never** be used, vs `--yes` is the user's
standing consent and the documented sole exception to the ask-user
escalation rule.
**Citations:** BU-P1-072 (`README.md`), BU-P8-083 (`docs/using-sergeant.md`)
vs BU-P2-100 (vendored `no-mistakes/SKILL.md`). **Citation correction
(finding N1-BH-12, adjudication-round1.md A14):** BU-P1-072 was originally
misattributed to `AGENTS.md` above; its actual `source.path` is
`README.md` (verified directly in `behavior-units/P1.ndjson`: `"path":
"reference/sergeant-upstream/README.md"`, locator "README.md L275",
quote "Do not use `--yes`. Use `--skip=<steps>` only for stages already
proven irrelevant..."). Fixed in place; no change to the resolution below.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** resolve by scope — Sergeant-coordinated runs (W18
`validate-and-ship` invoked by the coordinator, or through W6/W9's shipping
boundary) forbid `--yes`; the vendored skill's `--yes` exception describes
standalone `no-mistakes` usage outside Sergeant's coordination, which this
repository does not itself exercise. Neither source unit states this scope
today; the ledger records this as the reading a future generator/CONTEXT.md
should adopt, and flags both units for an explicit scope clause at the next
revision of either source document.

### X4 — Worker invocation of the shipping gate (Type C+S)
**Conflict:** an absolute prohibition stated as a hard error, vs an explicit
user override that must be rendered verbatim into a worker's brief, vs the
same rule classified as an engine gap.
**Citations:** BU-P1-040, BU-P1-131, BU-P8-082 (invariant) vs BU-P7-112
(`tests/` — override must survive into the brief) vs BU-P2-056 (engine-gap,
rejected as `engine-pressure.md` G8).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** three-part resolution. **(a)** The prohibition stands as
`agents-invariant` (Article III of `permanent-instructions.md`). **(b)** The
explicit user override (BU-P7-112) is the sole documented exception path and
must be threaded verbatim into the worker's brief, never silently honored or
silently dropped. **(c)** The engine-gap framing (G8) is rejected per its own
ruling — no runtime role enforcement is claimed necessary; enforcement stays
instruction-based unless a future authority-model capability is designed for
independent reasons. This resolves the three-way disagreement by giving each
citation its correct rung rather than picking one to the exclusion of the
others.

### X5 — Bash-3.2 portability (Type C)
**Conflict:** an operating invariant every runtime path must satisfy and
every shipped script must parse under, vs a distribution-specific target a
compiled binary makes obsolete.
**Citations:** BU-P7-033, BU-P7-071, BU-P1-065 (`agents-invariant`) vs
BU-P8-102 (`obsolete-mechanism`).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt BU-P8-102's obsolete-mechanism ruling — sergeant-rs ships
a compiled binary with no Bash runtime path in scope at all, so the Bash-3.2
target cannot bind anything sergeant-rs executes. BU-P7-033/BU-P7-071/
BU-P1-065 are retained as provenance for *what* the old invariant guaranteed
(portable, dependency-free scripts), not as a live constraint. **Caveat:**
BU-P8-102 is itself `low` confidence (§4 below) — this resolution should be
revisited first if any higher-confidence evidence surfaces on either side,
since the resolution currently rests on the weaker-confidence citation.

### X6 — Launch mode (Type C)
**Conflict:** "dispatch uses only persistent interactive sessions and
rejects every non-interactive launch mode" recorded as a live stage-context
rule, vs the identical statement recorded as obsolete and structurally
reversed by D2.
**Citations:** BU-P1-057 (stage-context) vs BU-P7-008, BU-P8-064
(obsolete-mechanism).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** D2 (GAUNTLET.md deviation register — headless `claude -p`
turns, no daemon TTY/pane) settles it. Re-home BU-P1-057 as
obsolete-mechanism provenance inside M2's cluster in `obsolete-mechanisms.md`
rather than a live stage-context rule; P1's classification is the outlier and
is overruled by the settled deviation-register ruling this milestone does
not re-litigate.

### X7 — Setup backup ordering (Type S)
**Conflict:** a timestamped backup is created **before** writing in the
new-project phase, vs a backup is created **only after** the user confirms,
with the skill instructed not to even describe a pre-confirmation backup.
**Citations:** BU-P5-027 (Phase 4) vs BU-P5-030 (Phase 5) and BU-P7-038
(`tests/`, stated as a general rule).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the stricter, test-corroborated rule — no backup write
occurs before the user has confirmed the destructive action (BU-P5-030 +
BU-P7-038). This is also the reading consistent with `permanent-instructions.md`
Article III's standing-authorization boundary ("standing authorization
removes repetitive confirmation only — never risk acceptance"). Flag
BU-P5-027 (Phase 4) as a same-source drafting inconsistency inside the
`sergeant-setup` draft-workflow package rather than a genuine second rule —
Phase 4 and Phase 5 describe the same procedure and cannot both be followed
as written.

### X8 — Sequential one-question-at-a-time human input (Type C)
**Conflict:** an engine gap requiring a new runtime capability, vs ordinary
actor procedure needing nothing, vs explicitly "representable today via the
existing needs-input state, not an engine gap."
**Citations:** BU-P5-025 (engine-gap) vs BU-P3-007 (`grilling`,
stage-context) vs BU-P4-053 (`to-spec`, stage-context with an explicit
non-gap note).
**ADJUDICATED (adjudication-round1.md A13 — overturned):** BU-P4-053's
"representable today via the existing needs-input state, not an engine gap"
note stands — and stands for BU-P5-025 too, not only for BU-P3-007/
BU-P4-053. The distinction the original PROPOSED resolution leaned on
(BU-P5-025 supposedly needs a not-yet-built re-enterable-stage capability
that BU-P3-007/BU-P4-053 merely stand to benefit from later) does not
exist: `engine-pressure.md` G5 is itself rejected on re-derivation, because
the "never attempted" lower rung its narrowing rested on — a needs_input
round-trip paired with re-entering the same stage — already exists in the
shipped engine. Verified directly: `src/domain/work.rs`'s
`WorkState::can_transition` allows `NeedsInput → Active`, and
`src/api.rs` exposes both `POST /v1/work/{id}/input` (`work_input`,
command `work.respond`, "answer a work that asked for input (§12's
needs-input verb)") and `POST /v1/work/{id}/retry` (`work_retry`,
"re-enter the current stage (§12's retry verb)") — composed, these are
exactly the per-question needs_input/retry loop G5's narrowing proposed as
the missing capability. BU-P5-025 is therefore misclassified as
`engine-gap`; it reclassifies to `stage-context`, alongside BU-P3-007 and
BU-P4-053, representable today with no engine change. See
`engine-pressure.md`'s revised G5 entry for the full re-derivation.

### X9 — Graph building (Type B)
**Conflict:** a standalone workflow with its own trigger and all-or-nothing
merge, vs a stage inside `load-project`.
**Citations:** BU-P6-088/089/090/091 (workflow `graphify`) vs
BU-P5-105/106/107/112, BU-P7-003/086/088 (stages of `load-project`).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt W2 `project-graph` as promoted in `synthesis.md` §1 —
independent trigger, bounded outcome (a published graph), and its own
failure mode justify a standalone workflow; `load-project`'s `40-consume`
stage is correctly demoted to helper/context per synthesis's own stage
table.

### X9b — Daily digest (Type B)
**Conflict:** `wiki` with a `00-digest` stage, vs `wiki-daily-digest` as its
own workflow.
**Citations:** BU-P5-130/131/137…149 vs BU-P6-092/093.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt W35 `wiki-digest` as the single merged workflow, per
`synthesis.md` §1's fold.

### X10 — Installation (Type B)
**Conflict:** a ten-phase interactive skill, vs a nine-item completion
checklist from the getting-started docs, with different phase sets and
different prerequisite lists.
**Citations:** BU-P5-001…BU-P5-037, BU-P7-036…BU-P7-040 (`sergeant-setup`)
vs BU-P8-041…BU-P8-051 (`sergeant-install`).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt W3 `sergeant-setup` as the merged workflow (already done
in `synthesis.md` §1). For the still-unresolved prerequisite-list
discrepancy (P5 omits Python 3 and Node; P8 adds them), PROPOSE taking the
**union** (P8's superset) as `00-detect-prerequisites`'s checked-tool list —
a superset check is strictly safer than an incomplete one, and no evidence
favors P5's omission being intentional. This specific sub-point should be
confirmed against the real `sgt doctor`/prerequisite-check source when N2
generates the workflow, not treated as settled by this ledger alone.

### X11 — Command surfaces promoted to workflows (Type B)
**Conflict:** `list-projects`, `project-status`, `project-sync`,
`project-task-list`, `treehouse-init`, `doctor-capability-check` each
extracted as a standalone workflow in P6, vs the same commands treated as
verification steps inside `load-project`/`sergeant-setup` in P5.
**Citations:** BU-P6-011, BU-P6-012, BU-P6-013, BU-P6-035, BU-P6-018,
BU-P6-003 vs BU-P5-092, BU-P5-093, BU-P5-101, BU-P5-102, BU-P5-031,
BU-P5-034.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt P5's framing, per `synthesis.md` §1 — these fail §6.2's
bounded-outcome/completion-condition test (a listing is not a procedure with
a bounded outcome); fold as stages/verification steps of W1 `load-project`
and W3 `sergeant-setup` as the synthesized stage tables already show. P6's
per-command workflow framing is retained only as an alternate proposal in
this ledger, not carried into `draft-workflows/`.

### X12 — The fleet runtime (Type B)
**Conflict:** P7 files six behaviors under one umbrella
`software-change (…)` name, while P6/P8 give the same behaviors six distinct
workflow names.
**Citations:** `software-change (drain/recovery)` BU-P7-083/084/092/093/
094/095/107/108 vs `drain-admission-control` BU-P6-057/058 and
`stall-recovery` BU-P6-071/072/073/075; `software-change (fleet monitoring)`
BU-P7-099/100/101 vs `fleet-observation` BU-P6-101 and
`fleet-reconciliation` BU-P6-103/104/105; `software-change (notification
handshake)` BU-P7-035…109 vs `respond-and-resume` BU-P6-032/034/078/079/080;
`software-change (independent review)` BU-P7-063/064 vs
`review-finding-routing` BU-P6-082/084/085/086.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the six distinct workflow names (W10–W16) — each is
independently bounded, independently triggerable, and independently
recoverable, which is more legible than one umbrella label spanning six
unrelated triggers. P7's `software-change` grouping is retained as a
cross-reference note in each of W10–W16's `index.md`, not as a competing
workflow.

### X13 — Waiting workers (Type S)
**Conflict:** "a waiting worker **may remain alive** or may exit after a
durable handoff", vs a worker must never poll or sleep and must exit
cleanly after publishing its condition.
**Citations:** BU-P1-036 (`AGENTS.md`) vs BU-P7-010
(`templates/worker-brief.md`), BU-P8-074 (`docs/using-sergeant.md`).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the stricter rule (must exit cleanly after publishing
its wait condition) as canonical — it is corroborated by two independent
sources against `AGENTS.md`'s one, and it is the rule `engine-pressure.md`
G1's scheduler design already depends on ("no actor process running while
waiting" is exactly the property G1's acceptance test checks). Flag
`AGENTS.md`'s "may remain alive" phrasing as a documentation inconsistency
to fix at generation time; do not carry it forward as a live option in W9
`worker-mission`.

### X14 — Review auto-fix (Type S)
**Conflict:** disabled by default but re-enabled by a repo- or global-level
override, vs an auto-fix must **never** be authorized in Sergeant's
validation-only workflow.
**Citations:** BU-P2-080 (vendored gate skill) vs BU-P8-100
(`docs/troubleshooting.md`).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the absolute (BU-P8-100, already `agents-invariant` at
Article III — "validation agents never modify source while reporting
findings") for sergeant-rs's own `no-mistakes` usage. The override capability
in the vendored skill (BU-P2-080) describes generic upstream behavior this
repository does not exercise; record as a scoping note in W18
`validate-and-ship`'s `CONTEXT.md` rather than as a contradiction requiring
a source-document fix.

### X15 — Dependency-edge enforcement (Type S)
**Conflict:** enforcement is "left entirely to the dispatched workers"
reading their own brief, vs the acceptance test that a dependent must be
**held**, not merely advised, vs an external scheduler that genuinely
advances stages on completion.
**Citations:** BU-P5-074 vs BU-P5-065's acceptance test vs BU-P6-016/017.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** already resolved inside `engine-pressure.md` G2's split
acceptance test — adopt the split: **(a)** grouping identity and recording
(evidenced by `sgt-dispatch`) and **(b)** advance-on-completion enforcement
(evidenced by the DAG hook). Neither artifact alone evidences both halves;
both are required for the full claim and neither is redundant with the
other.

### X16 — Direct execution (Type B)
**Conflict:** a six-stage `direct-mode` from `AGENTS.md`, vs an eight-step
`direct-implementation` from `docs/using-sergeant.md`, with different step
boundaries (the docs split reconciliation and the shipping gate into their
own steps).
**Citations:** BU-P1-007/107 + BU-P1-008…014 vs BU-P8-055 +
BU-P8-053/056/058.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt W6's eight-step boundary (already merged in
`synthesis.md` §1 "on the docs' finer boundary") as the canonical stage
list — a finer split that keeps reconciliation and the shipping gate as
independently observable checkpoints is strictly more informative than the
coarser six-stage version. `AGENTS.md`'s six-stage framing is retained as a
coarser restatement of the same procedure, not a competing boundary.

### X17 — The shipping gate under four names (Type B)
**Conflict:** the shipping gate filed under four different workflow names
across four partitions with different stage sets; separately,
`validate-and-ship` in P6 names something else entirely (the source repo's
own pre-push hook).
**Citations:** `no-mistakes` BU-P2-057…103 (pipeline-driving; launch
machinery) vs `no-mistakes-shipping-gate` BU-P1-069…080 vs
`no-mistakes (as consumed by software-change)` BU-P7-065/104/105, all vs
`validate-and-ship` BU-P6-007/008.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt W18 `validate-and-ship` as the single merged workflow
(ten stages, per `synthesis.md` §5's own U2 verdict that §6.3's test
discriminates cleanly once the source's flat command list is split by
outcome), with W19 `repo-release-verification` split out as the source
repo's own separate pre-push gate — this is already `synthesis.md`'s
ruling and this ledger adopts it without change.

### X18 — Intent transport (Type S)
**Conflict:** "no delivered content **ever** appears in process arguments"
and "canonical intent content must **never** appear in process arguments",
vs a documented per-invocation operator-consent path that selects exactly
that argv transport.
**Citations:** BU-P8-065, BU-P8-085 (absolute) vs BU-P6-047, BU-P8-086
(consent-gated exception).
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed, genuinely open — same shape as X3):** read the absolute as the
default and the consent-gated path as a narrowly-scoped, explicitly-logged,
single-invocation deviation — matching Article III's "standing authorization
removes repetitive confirmation only, never secret exposure." Neither source
unit states this scope today, so this is a proposed reading for a future
generator to adopt, not a resolution either source document already
supports; flag both pairs of units for an explicit scope clause at their
next revision.

### X19 — Unfinished response handshakes at cleanup (Type S)
**Conflict:** cleanup must never be forced and deliberately refuses while
the handshake could still be completed, vs cleanup **can** retire an
unfinished handshake when the owning task is closed and the worker is
provably dead by four independent proofs.
**Citations:** BU-P8-104 vs BU-P8-105/BU-P8-106.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the reconciling reading `synthesis.md` itself already
offers — refuse-by-default with a proven-dead exception. BU-P8-104's refusal
is the default; BU-P8-105/BU-P8-106's four-proof exception is the sole,
narrowly-scoped override. Both units should be read together as one rule,
not as two competing absolutes — this requires no further evidence to
resolve, only reading the two citations as complementary rather than as a
true contradiction.

### X20 — Procedure invocation (Type S)
**Conflict:** `implement` disables model invocation and must be explicitly
invoked, vs the stated architecture that model-invoked disciplines "load
automatically whenever the task matches" and the standing rule to load a
procedure whenever its trigger applies.
**Citations:** BU-P2-051 (+ BU-P3-002/004) vs BU-P1-118, BU-P1-021.
**ADJUDICATED (adjudication-round1.md A14 — no reviewer objection; adopted
as proposed):** adopt the orchestrator/discipline split — top-level workflow
selection (W5 `task-intake-and-route`) loads automatically per Article
VIII's "load a procedure only when its trigger applies," while `implement`
(W23) and its explicitly no-auto-invoke siblings are *disciplines* selected
by explicit operator request or by a dispatch-brief's routing stage (W9's
`10-triage-and-route`), never autonomously self-triggered. BU-P1-129's
bundling of `implement` as a discipline (not a top-level workflow) is the
correct classification, and the apparent tension with Article VIII
dissolves once the workflow/discipline distinction is drawn explicitly —
recorded here as the reading to encode in the `explicit-invocation-metadata`
entry of `helper-map.md`.

---

## 4. Low-confidence units

*Round-1 adjudication added/split units: eight units corpus-wide are now
`confidence: low` — not the two this section originally recorded. The two
original (`BU-P8-028`, `BU-P8-102`) are unchanged and still listed in full
below; six more were demoted since, independent of the A10/A12 unit-count
growth: `BU-P6-129` and `BU-P8-077` by A10/A12's own narrowing rulings
(§2's note above), and `BU-P7-087`/`BU-P8-009`/`BU-P8-056`/`BU-P8-057` at
N1 verifier round 2's finding V8 (citation-span disputes, honesty-over-
laundering per A2's ruling — an unreproducible citation is recorded, not
hidden). The six are listed compactly, not in full per-unit form, as a
residual for a future pass to expand; see each id's own record in
`behavior-units/*.ndjson` for the complete note.*

Two of the eight units — the original pair this section was built
around — belong to P8 and are listed here in full per the N1 gate's
requirement to surface, not silently absorb, weak extractions.

### BU-P8-028
- **Source:** `reference/sergeant-upstream/docs/callbacks.md` L186-196
  ("ws-lab Consumer Handoff")
- **Representation:** `helper` (unnamed workflow/stage — a general-purpose
  mechanic, not tied to one checkpoint)
- **Why low:** the "ws-lab Consumer Handoff" section documents one worked
  example of a callback consumer, from which the extractor generalized a
  reusable behavior — the statement is an inference from an example, not a
  direct instruction, which is exactly what separates `low` from `medium`
  confidence per `docs/icm/record-shapes.md` §3.
- **Adjudication note:** homed to `helper-map.md`'s `callback-plumbing`
  candidate alongside seven `high`-confidence siblings from the same
  document family; the low-confidence status of this one unit does not
  weaken the candidate's overall evidence, since the other seven units
  independently corroborate the same mechanic from direct instruction, not
  from an example.

### BU-P8-102
- **Source:** `reference/sergeant-upstream/docs/troubleshooting.md` L129-146
  ("Bash 3.2 validation")
- **Representation:** `obsolete-mechanism` (M9 cluster)
- **Why low:** the troubleshooting doc describes a validation *procedure*
  for confirming Bash-3.2 compatibility, from which the extractor inferred
  the underlying durable claim ("this target is/was load-bearing") rather
  than reading it stated directly — the same example-vs-instruction gap as
  BU-P8-028.
- **Adjudication note:** this is the single citation the X5 resolution
  (§3 above) rests on for its obsolescence ruling. Because it is the
  weaker-confidence side of that conflict (against three `high`-confidence
  units on the other side), X5's now-**ADJUDICATED** (A14, no reviewer
  objection) resolution is still flagged there as the first candidate to
  revisit if stronger evidence emerges for either position — adjudicated
  status settles which reading the corpus currently adopts, not the
  underlying confidence gap; this ledger entry and X5's entry should be read
  together.

### The other six (compact form)

| Id | Source | Representation | Why low |
|---|---|---|---|
| `BU-P6-129` | `bin/sgt-validate` L2 | `workflow` | Demoted at A10: its single 83-char header-comment quote supported only the coarse workflow-boundary claim, not the four sub-requirements the statement originally asserted; those now rest on stronger sibling citations (`BU-P6-130`, `BU-P6-131`, `BU-P6-133`, `BU-P6-143`). |
| `BU-P8-077` | `docs/using-sergeant.md` L231-243 | `stage` | Retired at A12: split into `BU-P8-110`/`BU-P8-111`; its own record now admits it is a superseded split, so it is no longer citable as an operative high-confidence unit — retained only for id stability. |
| `BU-P7-087` | `tests/sgt-graphify-test.sh` line 17 | `helper` | DISPUTED at N1 verifier round 2 finding V8: the cited line is a real, hash-verified quote but an unrelated test-fixture variable declaration; the actual supporting evidence (the F2 symlink-alias fixture comment and lines) sits at L26-28, not the cited locator. |
| `BU-P8-009` | `docs/callbacks.md` L15 | `agents-invariant` | DISPUTED at N1 verifier round 2 finding V8: the cited span is only the tail fragment of its sentence; the subject and negation ("Sergeant never accepts an") sit on the preceding line (L14), excluded from the quoted span. |
| `BU-P8-056` | `docs/using-sergeant.md` L23 | `stage-context` | DISPUTED at N1 verifier round 2 finding V8: the cited span is real but is step 3 of the Direct-mode list, not step 2 as the locator claims; the actual step-2 text is one line up, at L22. |
| `BU-P8-057` | `docs/using-sergeant.md` L24 | `agents-invariant` | DISPUTED at N1 verifier round 2 finding V8: the cited span is real but is step 4 of the Direct-mode list, not step 3 as the locator claims; the actual step-3 text is one line up, at L23. The underlying claim is still true and independently corroborated by `tests/instruction-policy-test.sh`, but this record's own citation does not carry it. |

All four `DISPUTED` entries above are recorded per A2's ruling (see the
Lesson candidate in `adjudication-round1.md`): an unreproducible or
mis-anchored citation is a fact the corpus records, not one it hides or
silently re-anchors. See each id's full record in `behavior-units/*.ndjson`
for the complete note.

## 5. Synthesis-unassigned units

Three units have no home in any of `synthesis.md`'s five representation
categories (§1–§5) or the conflict table (§6); each is recorded with its
reason rather than silently absorbed, per the N1 contract's binding rule
that every unit must be traceable.

| Unit | Source | Reason |
|---|---|---|
| **BU-P1-062** | `README.md` — "the right unit of distribution for turning a general-purpose agent into a specialist is not a CLI tool or an MCP server, but a cloned directory of instructions, skills, and conventions" | Product positioning, not a directive. Names no trigger, no required/prohibited action, no evidence of compliance — fails the corpus's own observability test at BU-P1-017 and would be removed by BU-P1-018's rule. Cannot change a decision. Retained as provenance for the product's design rationale. |
| **BU-P1-063** | `README.md` — "Sergeant narrows firstmate's crew-orchestration idea to start from project topology" | Attribution and design lineage. Same failure against BU-P1-017. The operative half ("a project is a named collection of repositories, and everything flows from that definition") is already carried by the `project-configuration` shared context and `@@project`. |
| **BU-P7-021** | `tests/global-state-isolation-test.sh` — the pre-created drain-directory "trap" that works because `mv <tmp> global` moves a file *inside* a pre-existing sentinel directory and the paired `rm -f` cannot remove a directory | Pure mechanism of the source repo's own Bash test harness, with no durable policy of its own. The policy it serves (a leak guard must itself be provably able to detect that class of leak) is stated independently by BU-P7-022, retained in the `test-infrastructure` helper-map candidate. Classified `helper` by the extractor, but that classification cannot survive the Bash-specific mechanism it depends on, and it is not an obsolete *Sergeant* mechanism either — it belongs in neither `helper-map.md` nor `obsolete-mechanisms.md`. |

Both README-sourced units (BU-P1-062, BU-P1-063) were originally counted
inside the 105-unit `agents-invariant` representation census before being
moved here — `synthesis.md` §2's sizing note states this explicitly ("Two
units originally in this set are moved to §7"), which is why the
`agents-invariant` representation count (105, per §2 above) is two higher
than the article-citation count (103) inside `permanent-instructions.md`.

## 6. Refute-stage challenge docket (carried forward from `synthesis.md` §8)

These are the specific points `synthesis.md`'s own authors flagged as where
the synthesis is most exposed — the standing docket for the next independent
reviewer round this milestone's gate requires. Each is listed with its
current status; **none of these has yet received a second-reviewer verdict**,
so all remain **OPEN** for this milestone's gate.

| # | Challenge | Where it lives | Status |
|---|---|---|---|
| 1 | W8 `dispatch` is 63 units and 12 stages — by far the largest cluster. Either it is genuinely one procedure with twelve checkpoints, or it should split at `70-launch-and-record` into `plan-and-validate` and `launch-fleet`. | `draft-workflows/dispatch/`, `synthesis.md` §1 W8 | OPEN |
| 2 | W18's ten stages are the U2 answer (does §6.3's reimplementation test discriminate cleanly?). An independent reviewer should reproduce roughly the same ten boundaries from the same units; six or fifteen would mean the test does not discriminate. | `draft-workflows/validate-and-ship/`, conflict X17 | OPEN |
| 3 | G6 and G7 (`engine-pressure.md`) are the soft rulings. G6 survives on a single §6.5 helper-test failure; G7 was rejected on a reading of §6.7 a reviewer may consider too strict. | `engine-pressure.md` G6, G7 | OPEN |
| 4 | The G9 rejection re-homes four claims into an existing invariant — a reviewer may reasonably read this as the synthesis grading its own architecture favorably. | `engine-pressure.md` G9 | **PARTIALLY ADDRESSED** (adjudication-round1.md A13): the BU-P7-079 quarter of G9's rejection was found circular with X1 and has been re-derived on independent architectural evidence (`src/runtime/surface.rs`, `src/runtime/fsutil.rs`), not on the synthesis's own citation preference — see `engine-pressure.md` G9. The BU-P7-043/049/051 quarters were not entangled with X1 and are unchanged; whether *their* invariant-absorption readings are self-grading remains OPEN for Verify. |
| 5 | X4, X5, X6, and X8 are classification conflicts this ledger (via `synthesis.md`) resolved by ruling; each is a candidate for reversal on evidence. | §3 above | **PARTIALLY ADDRESSED**: X8 was reversed on evidence at adjudication-round1.md A13 (see §3) — exactly the kind of reversal this row anticipated. X4, X5, X6 were reviewed and adopted as proposed at A14 (no reviewer objection), now ADJUDICATED rather than PROPOSED, but not re-argued from new evidence; a future reviewer may still challenge them per the closing rule below. |
| 6 | U1's answer (966 units from 179 files, well above the 150–400 scoping estimate; round-1 adjudication added/split units since — now 979, see §1) is concentrated in P6/P7/P8 (`bin/`, `tests/`, `docs/`), where a single script or test file routinely yielded 8–15 units — whether that granularity is correct, or whether P6/P7/P8 over-extracted at the sentence level while P2/P3/P4 extracted at the paragraph level, is itself an open finding N2's precision measurement will need to control for. | Corpus-wide (§1 above) | OPEN |

**How to close an OPEN item:** per the N1 contract's binding rules, a
reviewer challenging any row above must cite the specific units/sections in
dispute and either accept the PROPOSED resolution, propose a different one
with its own citations, or record the disagreement as a new conflict entry
in §3 — silence is not a valid disposition.

## Round-1 adjudication

The refute phase produced 21 findings — 12 boundary-honesty findings
(N1-BH-01…12) and 9 completeness/invention findings (N1-R3-01…09) from two
independent reviewers — plus two substantive defects from the structural
lint. `adjudication-round1.md` (2026-08-10) is the orchestrator's ruling on
all 21 plus the two lint defects, recorded as fifteen numbered rulings
A1–A15. Per L9, these rulings are themselves reviewable findings; adjudicating
does not erase disagreement, it records a position with its evidence.

**Disposition by ruling, as it lands on this repository's artifacts:**

| Ruling | Findings covered | Disposition | Where it lands |
|---|---|---|---|
| A1 | lint defect 1 / BH-03 context | `record-shapes.md` §4's representation vocabulary amended to the corpus's actual enum (fixed directly by the orchestrator, not this fixer). | `docs/icm/record-shapes.md` §4 |
| A2 | R3-01, R3-02, BH-11 | `record-shapes.md` §3 gains the quote-hash preimage convention and a required `quote` field; corpus-wide `quote`/`quote_hash` re-derivation across all 966 units (979 as of round-1 adjudication's A10/A12 additions/splits) is a separate fixer task, not this one. | `docs/icm/record-shapes.md` §3; `behavior-units/*.ndjson` (not touched by this pass) |
| A3–A8 | BH-01, BH-02, BH-04, BH-06, BH-07, BH-10 | Draft-workflow structural fixes (dispatch ordering, stage demotion, validate-and-ship restoration, package merges). Out of scope for the two files this pass owns. | `draft-workflows/` |
| A9–A12 | BH-03/R3-05, BH-08, BH-09, R3-08/R3-09 | Unit-record backfills (rationale, alternatives, normalization, missing coverage). Out of scope for this pass. | `behavior-units/*.ndjson` |
| **A13** | **BH-05, R3-04, R3-03** | **Implemented this pass.** X1 overturned (test-backed BU-P7-079 wins over doc-backed BU-P8-108, per §5's tests-outrank-docs rule; the circular G9 citation severed). X8 overturned (BU-P4-053's "representable today" reading stands for BU-P5-025 too). G5 rejected on re-derivation against the shipped engine (`src/domain/work.rs`, `src/api.rs`), moved from Surviving to Rejected. G9's BU-P7-079 quarter re-derived on independent architectural evidence (`src/runtime/surface.rs`, `src/runtime/fsutil.rs`), landing again at rejected but on non-circular grounds. | `classification-ledger.md` §3 X1, X8; `engine-pressure.md` G5, G9 |
| **A14** | (no reviewer objection to X2–X7, X9–X20; BH-12 for X3's citation) | **Implemented this pass.** X2, X4–X7, X9, X9b, X10–X20 each moved PROPOSED → ADJUDICATED, citing this ruling, resolution text otherwise unchanged. X3's citation corrected (BU-P1-072's `source.path` is `README.md`, not `AGENTS.md`, verified against `behavior-units/P1.ndjson`) and also moved to ADJUDICATED. | `classification-ledger.md` §3 |
| A15 | R3-06, R3-07 | `reference-corpus/lint.py` joins the corpus; `FROZEN.md` written at freeze. Out of scope for this pass — a corpus-wide freeze action, not a ledger edit. | `reference-corpus/lint.py`, `FROZEN.md` (not yet written) |

**What this pass changed, concretely.** `classification-ledger.md` §3: all
twenty conflict entries (X1–X20, including X9b) carry an **ADJUDICATED**
status line citing A13 or A14; X1 and X8's resolutions are rewritten to the
overturned reading; X3's citation is corrected in place. `engine-pressure.md`:
G5 moved from "Surviving claims" (five slots remain: G1, G2, G3, G4, G6) to
"Rejected claims" (now four: G5, G7, G8, G9), with its full original claim,
narrowing, and the re-derivation that overturned it kept on the record; G9's
BU-P7-079 sub-claim carries both its superseded (circular) rejection ground
and its re-derived (architectural) one; the roll-up table, survivor/
rejection counts, and the "Method observation for N2" paragraph are updated
to match.

**Residuals owned by the Verify pass.** This adjudication round settled the
21 refute-phase findings' disposition, not every open question the corpus
still carries:

- §6's Refute-stage challenge docket, items 1, 2, 3, and 6 (W8 `dispatch`'s
  stage count, W18's ten-stage boundary reproducibility, G6/G7's soft
  rulings, and U1's extraction-granularity question) remain fully **OPEN** —
  untouched by A13/A14, they were never in this round's scope.
- Docket items 4 and 5 are **PARTIALLY ADDRESSED** by this round (see the
  updated rows in §6 above) but not closed: G9's BU-P7-043/049/051 quarters'
  self-grading concern is unresolved, and X4/X5/X6 were adopted without new
  evidence, not re-argued.
- A2's corpus-wide `quote`/`quote_hash` re-derivation across all 966 units
  (979 as of round-1 adjudication's A10/A12 additions/splits; see §1)
  (A9–A12's rationale/alternatives/normalization/coverage backfills, and
  A3–A8's draft-workflow structural fixes) are separate, larger fixer tasks
  this pass did not touch — this pass's ownership is scoped to
  `classification-ledger.md` and `engine-pressure.md` only. (A9–A12 have
  since been implemented by a later pass, per the notes on `BU-P1-132`…
  `BU-P1-137`, `BU-P5-150`…`BU-P5-153`, `BU-P6-143`, and `BU-P8-110`/
  `BU-P8-111` in `behavior-units/*.ndjson`; this residual bullet is left
  otherwise unchanged as the historical record of what this specific pass's
  scope was.)
- A15's `lint.py` addition and `FROZEN.md` freeze action have not run; this
  document's own consistency (e.g. the corrected X1/X8/X3 entries, the
  revised G5/G9 entries) has not yet been machine-checked against a lint
  pass, only hand-verified against `behavior-units/*.ndjson` and the actual
  `src/` sources cited above.

Any reviewer reopening a residual above follows this document's own closing
rule: cite the specific units/sections in dispute and either accept, amend
with new citations, or record a new conflict entry — silence is not a valid
disposition.
