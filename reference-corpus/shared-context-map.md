# Shared Context Map

> **Superseding note (added post-round-1-adjudication, V10).** The `W`-numbered
> workflow references below are `synthesis.md`'s vocabulary as of that phase
> artifact and are now historical — `adjudication-round1.md`'s structural
> rulings (A3–A8) reordered, demoted, restored, merged, split, and removed
> stages and whole packages afterward. Each `draft-workflows/<name>/provenance.md`
> and `workflow.toml` is the authoritative, current source for which package
> actually shares a given context; `adjudication-round1.md` is the bridge
> document explaining how the two diverge. This note is additive — the
> entries below are otherwise unedited.

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`shared-context-map.md`). Sourced from `synthesis.md` §3a ("Named shared
contexts already conventionalized") and §3b ("Shared guidance"). Per
`docs/icm/convention.md` §4, a shared context is reusable *actor guidance* —
it answers "how should the actor reason while performing this stage," and it
resolves through exactly one path: `@@name` → `.sergeant/common/contexts/name.md`.
No per-workflow override, no search path (§4 rule 1).

Each entry: the guidance (what the actor is meant to internalize, not the raw
source text — normalized per §6 of the ICM ladder), the workflows that share
it, and the `@@name` it carries or would carry. §3a's twelve are already
conventionalized in the source (the extractor found old Sergeant already using
an equivalent reference-by-name pattern); §3b's are candidates this milestone
proposes naming for the first time, since old Sergeant expressed them as
duplicated prose rather than a named reference.

---

## Part 1 — Already-conventionalized (`@@name` given by the source)

### `@@project`
**Guidance:** what a project is — a named collection of repositories with
roles/groups and a layered instruction set — and how to resolve one before
acting.
**Source evidence:** BU-P1-101. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L29-31, "Project".
**Shared by:** W1 `load-project`, W3 `sergeant-setup`, W5
`task-intake-and-route`, W7 `cross-repo-work`, W8 `dispatch`.
**`@@name`:** `@@project`

### `@@repository`
**Guidance:** what a repository is within a project — its identity, path, and
role — distinct from a bare filesystem clone.
**Source evidence:** BU-P1-102. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L34-36, "Repository".
**Shared by:** W1 `load-project`, W7 `cross-repo-work`, W8 `dispatch`, W15
`reconcile-and-cleanup-fleet`.
**`@@name`:** `@@repository`

### `@@task`
**Guidance:** the durable tracked-work identity a workflow attaches
implementation, dispatch, and review evidence to; how to find or create the
canonical one before starting new work.
**Source evidence:** BU-P1-103. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L39-42, "Task".
**Shared by:** W5 `task-intake-and-route`, W8 `dispatch`, W9
`worker-mission`, W16 `route-review-findings`, W32 `to-tickets`.
**`@@name`:** `@@task`

### `@@fleet`
**Guidance:** the running-worker-set concept a task groups; what "fleet-scoped"
means for a read or an admission decision.
**Source evidence:** BU-P1-104. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L44-47, "Fleet".
**Shared by:** W8 `dispatch`, W12 `drain-fleet`, W13 `monitor-fleet`, W15
`reconcile-and-cleanup-fleet`.
**`@@name`:** `@@fleet`

### `@@worker`
**Guidance:** the durable per-repository execution identity, its states, and
that a worker's process is not the worker's identity.
**Source evidence:** BU-P1-105. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L49-52, "Worker".
**Shared by:** W8 `dispatch`, W9 `worker-mission`, W10 `respond-to-worker`,
W11 `recover-stalled-worker`, W12 `drain-fleet`, W13 `monitor-fleet`.
**`@@name`:** `@@worker`

### `@@decision-request`
**Guidance:** the shape of a request for a human decision — the question, the
options, and how it differs from a plain status update.
**Source evidence:** BU-P1-106. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L54-58, "Decision request".
**Shared by:** W5 `task-intake-and-route`, W8 `dispatch`, W10
`respond-to-worker`, W18 `validate-and-ship`.
**`@@name`:** `@@decision-request`

### `@@review-axes`
**Guidance:** the single authoritative source for which review axes exist and
what each one covers, so every consumer names the same axis the same way.
**Source evidence:** BU-P1-085. `reference/sergeant-upstream/README.md` L314,
"review axes single source".
**Shared by:** W8 `dispatch`, W9 `worker-mission`, W16
`route-review-findings`, W18 `validate-and-ship`, W24 `code-review`.
**`@@name`:** `@@review-axes`

### `@@launch-record`
**Guidance:** how to read a launch record honestly — `launch_state` semantics,
and provider/variant verification against what was actually pinned, not
assumed.
**Source evidence:** BU-P1-091, BU-P1-092. `reference/sergeant-upstream/README.md`
L217-221 (launch_state honesty), L222-225 (provider/variant verification).
**Shared by:** W8 `dispatch`, W13 `monitor-fleet`.
**`@@name`:** `@@launch-record`

### `@@drain-admission-lock`
**Guidance:** how admission blocking works — what a drain means for new
work, and how existing in-flight work is left alone.
**Source evidence:** BU-P1-097. `reference/sergeant-upstream/README.md`
L353-358, "drain admission locking".
**Shared by:** W8 `dispatch`, W10 `respond-to-worker`, W11
`recover-stalled-worker`, W12 `drain-fleet`.
**`@@name`:** `@@drain-admission-lock`

### `@@installation-ownership-boundary`
**Guidance:** which paths Sergeant owns and may write to versus paths it must
never touch (another tool's configuration, a project's own source).
**Source evidence:** BU-P1-099. `reference/sergeant-upstream/docs/what-is-sergeant.md`
L14-21, "installation ownership".
**Shared by:** W1 `load-project`, W3 `sergeant-setup`.
**`@@name`:** `@@installation-ownership-boundary`

### `@@skill-locations`
**Guidance:** the canonical filesystem locations a skill can live in and how
their precedence resolves when more than one applies.
**Source evidence:** BU-P1-113. `reference/sergeant-upstream/docs/skills.md`
L6-18, "skill locations table".
**Shared by:** W3 `sergeant-setup`, W8 `dispatch`, W34 `vet-external-skill`.
**`@@name`:** `@@skill-locations`

### `@@worker-brief-skill-bundle`
**Guidance:** which skills a rendered worker brief must vendor so a worker
can understand a referenced procedure (e.g. the shipping-gate contract)
without a global install.
**Source evidence:** BU-P1-129, BU-P1-130. `reference/sergeant-upstream/docs/repo-scoped-skills.md`
L12-30 (worker-brief inventory), L31-33.
**Shared by:** W8 `dispatch`, W9 `worker-mission`.
**`@@name`:** `@@worker-brief-skill-bundle`

---

## Part 2 — Candidates for first-time naming (proposed `@@name`)

These are reused *guidance*, but old Sergeant expressed each one as duplicated
prose across several documents rather than a single named reference (the
recurrence itself is the evidence of sharing — the same rule independently
extracted from more than one source file). §6.6 of the ladder puts each under
`.sergeant/common/contexts/` on first authoring, with the `@@name` below.

### `project-configuration`
**Guidance:** project identity is the filename; `dev_root`; the three-layer
instruction order (defaults → group → repo, later wins, never structurally
merged); groups; accepted path forms.
**Source evidence:** BU-P1-051, BU-P1-052, BU-P1-053, BU-P1-066, BU-P1-067,
BU-P7-001, BU-P7-004, BU-P8-029, BU-P8-030, BU-P8-032, BU-P8-033, BU-P8-037,
BU-P8-040, BU-P6-020. Anchor: `reference/sergeant-upstream/AGENTS.md` L179.
**Shared by:** W1 `load-project`, W2 `project-graph`, W3 `sergeant-setup`,
W5 `task-intake-and-route`, W7 `cross-repo-work`, W8 `dispatch`.
**`@@name`:** `@@project-configuration`

### `worker-state-vocabulary`
**Guidance:** the seven durable worker states, what each means, the required
operator action for each, and the terminal/nonterminal split — including that
`waiting` is not a failure state.
**Source evidence:** BU-P1-035, BU-P1-036, BU-P8-073, BU-P8-098. Anchor:
`reference/sergeant-upstream/AGENTS.md` L148, nonterminal states.
**Shared by:** W8 `dispatch`, W9 `worker-mission`, W10 `respond-to-worker`,
W11 `recover-stalled-worker`, W12 `drain-fleet`, W13 `monitor-fleet`, W14
`wake-and-resume`, W15 `reconcile-and-cleanup-fleet`.
**`@@name`:** `@@worker-state-vocabulary`
**Note:** X13 records an unresolved contradiction inside this candidate's own
source material — whether a waiting worker may remain alive or must exit after
a durable handoff. The context file itself must state both citations and the
open question, not silently pick one (see `synthesis.md` §6 X13).

### `wake-conditions`
**Guidance:** the six typed wake-condition kinds, their required fields, and
the resume rule for each.
**Source evidence:** BU-P8-075. Anchor: `reference/sergeant-upstream/docs/using-sergeant.md`
L196-208, L224-227 (wake condition table).
**Shared by:** W9 `worker-mission`, W14 `wake-and-resume`.
**`@@name`:** `@@wake-conditions`

### `intent-provenance`
**Guidance:** one canonical intent revision governs implementation, review, PR
text, successor work, recovery, and validation; the eight required sections of
that revision.
**Source evidence:** BU-P1-039, BU-P1-041, BU-P6-049. Anchor:
`reference/sergeant-upstream/AGENTS.md` L150-151.
**Shared by:** W8 `dispatch`, W9 `worker-mission`, W16
`route-review-findings`, W18 `validate-and-ship`.
**`@@name`:** `@@intent-provenance`

### `review-severity-and-axes`
**Guidance:** the canonical severity vocabulary and its accepted
reviewer-spelling aliases; the canonical axis vocabulary, from a single source
both dispatch and validation consult; only the error family of severities
blocks.
**Source evidence:** BU-P6-024, BU-P6-083, BU-P7-061, BU-P7-062. Anchor:
`reference/sergeant-upstream/bin/sgt-no-mistakes-finding` L79-88.
**Shared by:** W8 `dispatch`, W9 `worker-mission`, W16
`route-review-findings`, W18 `validate-and-ship`, W24 `code-review`.
**`@@name`:** `@@review-severity-and-axes`

### `response-evidence-schema`
**Guidance:** the four archived-response fields, atomic staged publication,
and that exactly one parser reads the archive — never two independently
maintained readers.
**Source evidence:** BU-P6-033, BU-P6-051, BU-P6-139. Anchor:
`reference/sergeant-upstream/bin/sgt-ack-response` L80-113.
**Shared by:** W10 `respond-to-worker`, W15 `reconcile-and-cleanup-fleet`.
**`@@name`:** `@@response-evidence-schema`

### `callback-protocol`
**Guidance:** the four callback event classes, the invocation contract, the
three acknowledgement shapes and their effects, and the coverage requirement
(every terminal transition must be representable).
**Source evidence:** BU-P7-068, BU-P8-014, BU-P8-019, BU-P8-021, BU-P8-022.
Anchor: `reference/sergeant-upstream/tests/sgt-callback-test.sh` lines 58-73.
**Shared by:** W15 `reconcile-and-cleanup-fleet`, W17
`deliver-external-callback`.
**`@@name`:** `@@callback-protocol`
**Note:** paired with the `callback-plumbing` shared helper
(`helper-map.md`) — this entry is the guidance an actor reads; the helper is
the mechanics an actor invokes.

### `launch-evidence`
**Guidance:** what makes launch evidence durable, unfalsifiable, and
credential-free — derived from the validated harness/model/variant tuple,
never read back from the ambient environment.
**Source evidence:** BU-P6-109. Anchor: `reference/sergeant-upstream/bin/sgt-interactive-worker`
L82-97.
**Shared by:** W8 `dispatch`, W13 `monitor-fleet`.
**`@@name`:** `@@launch-evidence`

### `skill-discovery`
**Guidance:** the canonical skill tree, the mirrored discovery paths across
harnesses, the two configured roots, repository-local precedence over a
registry, and that only the coordinator vendors the gate skill.
**Source evidence:** BU-P1-117, BU-P6-006, BU-P7-015, BU-P7-026, BU-P7-030,
BU-P7-031, BU-P7-032. Anchor: `reference/sergeant-upstream/docs/skills.md`
L66-78, "Sergeant-owned skills table".
**Shared by:** W3 `sergeant-setup`, W8 `dispatch`, W34 `vet-external-skill`.
**`@@name`:** `@@skill-discovery`

### `codebase-design-vocabulary`
**Guidance:** module/interface/implementation/depth/seam/adapter/leverage/locality
terminology; the deletion test; depth-as-leverage; the two-implementation rule
for seam placement; terms to avoid.
**Source evidence:** BU-P4-001 through BU-P4-012 (12 units). Anchor:
`reference/sergeant-upstream/.agents/skills/codebase-design/SKILL.md`
frontmatter description, L3.
**Shared by:** W20 `diagnose-bug`, W22 `tdd`, W24 `code-review`, W25
`deepen-module`, W31 `to-spec`.
**`@@name`:** `@@codebase-design-vocabulary`

### `domain-modeling`
**Guidance:** glossary discipline, terminology-conflict surfacing, edge-case
invention, cross-checking claims against code, ADR-worthiness criteria, and
what must not be recorded (implementation detail, not decisions).
**Source evidence:** BU-P4-027, BU-P4-028, BU-P4-031, BU-P4-032, BU-P4-033,
BU-P4-034, BU-P4-035, BU-P4-037, BU-P4-042, BU-P4-043, BU-P4-046, BU-P4-049.
Anchor: `reference/sergeant-upstream/.agents/skills/domain-modeling/SKILL.md`
frontmatter description, L3.
**Shared by:** W25 `deepen-module`, W28 `grilling`, W29 `grill-with-docs`,
W30 `triage`, W31 `to-spec`, W33 `wayfinder`.
**`@@name`:** `@@domain-modeling`

### `test-quality` (the `tdd` reference half)
**Guidance:** behavior tested through public interfaces and seams;
implementation-coupled and tautological test anti-patterns; mocking only at
system boundaries; SDK-shaped injectable dependencies; good/bad test red
flags.
**Source evidence:** BU-P2-106, BU-P2-107, BU-P2-108, BU-P2-111, BU-P2-112,
BU-P2-117 through BU-P2-127 (16 units total). Anchor:
`reference/sergeant-upstream/.agents/skills/tdd/SKILL.md` intro, line 10.
**Shared by:** W20 `diagnose-bug`, W21 `prototype`, W22 `tdd`, W23
`implement`, W25 `deepen-module`.
**`@@name`:** `@@test-quality`

### `ticket-shaping`
**Guidance:** vertical slices; one-session sizing; one owning repository per
ticket; no duplicates; observable acceptance criteria; no horizontal splits;
update rather than duplicate; how to note superseding.
**Source evidence:** BU-P4-059, BU-P4-060, BU-P4-061, BU-P4-062, BU-P4-063,
BU-P4-066, BU-P4-069, BU-P4-074. Anchor: `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md`
Principles, L15.
**Shared by:** W7 `cross-repo-work`, W30 `triage`, W32 `to-tickets`, W33
`wayfinder`.
**`@@name`:** `@@ticket-shaping`

### `wiki-conventions`
**Guidance:** captured content vs. curated content stays separate; never raw
prompts, bodies, or secrets in a capture; a missing capture is fixed at its
source, never hand-synthesized; the schema-driven synthesis prompt and
candidate-page block; captures are side effects of specific commands, never
freestanding.
**Source evidence:** BU-P5-132, BU-P5-133, BU-P5-135, BU-P6-094, BU-P6-095,
BU-P8-093. Anchor: `reference/sergeant-upstream/skills/wiki/SKILL.md` lines
13-15.
**Shared by:** W8 `dispatch`, W10 `respond-to-worker`, W15
`reconcile-and-cleanup-fleet`, W35 `wiki-digest`.
**`@@name`:** `@@wiki-conventions`

### `triage-state-machine`
**Guidance:** one category role and one state role are held at a time (never
both loosely); the AI-authorship disclaimer requirement; the out-of-scope
knowledge base's purpose, shape, naming, and durability rule.
**Source evidence:** BU-P3-053, BU-P3-059, BU-P3-083 through BU-P3-088.
Anchor: `reference/sergeant-upstream/.agents/skills/triage/SKILL.md` line 13.
**Shared by:** W30 `triage`, W33 `wayfinder`.
**`@@name`:** `@@triage-state-machine`

### `dispatch-routing-context`
**Guidance:** the normalized review-routing context built from mission, role,
group, and merged instructions, so a review's routing decision is derived from
the same shape dispatch produced.
**Source evidence:** BU-P8-038. Anchor: `reference/sergeant-upstream/docs/schema.md`
L104-107.
**Shared by:** W8 `dispatch`, W16 `route-review-findings`.
**`@@name`:** `@@dispatch-routing-context`

### `recovery-visibility`
**Guidance:** committed work sitting above the recorded base commit must be
surfaced by every recovery/monitoring/cleanup path — never silently reported
as "no worktree, no pane" when a real committed diff exists.
**Source evidence:** BU-P7-091. Anchor: `reference/sergeant-upstream/tests/sgt-interrupted-fallback-test.sh`
lines 1-13.
**Shared by:** W8 `dispatch`, W11 `recover-stalled-worker`, W13
`monitor-fleet`, W15 `reconcile-and-cleanup-fleet`.
**`@@name`:** `@@recovery-visibility`

---

## Part 3 — Workflow-local contexts (kept local per §6.6)

Each belongs to exactly one workflow in current evidence — no `@@name`,
authored directly in that workflow's own `CONTEXT.md`/`references/`.

| Workflow | Units | Anchor |
|---|---|---|
| W24 `code-review` | BU-P2-008 | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md`, Step 3, line 38 |
| W18 `validate-and-ship` | BU-P2-102 | `reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md`, Output format and exit codes, lines 264-271 |
| W20 `diagnose-bug` | BU-P2-020 | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md`, intro, line 10 |
| W21 `prototype` | BU-P3-015, BU-P3-016, BU-P3-017, BU-P3-018 | `reference/sergeant-upstream/.agents/skills/prototype/SKILL.md`, item 1, line 21 |
| W25 `deepen-module` | BU-P4-018, BU-P4-019, BU-P4-025 | `reference/sergeant-upstream/.agents/skills/codebase-design/DEEPENING.md`, Seam discipline, L29 |
| W33 `wayfinder` | BU-P4-077, BU-P4-080 | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md`, Refer by name, L17 |

---

## Summary

12 already-conventionalized `@@name` contexts (§3a) + 17 first-time-named
candidates (§3b, Part 2) + 6 workflow-local groups = every id in
`synthesis.md` §3's "reused guidance" half accounted for. Part 2's proposed
names follow the same flat-namespace, kebab-case convention the source's own
twelve already establish (`docs/icm/convention.md` §4 rule 3: one directory,
one file per name) — none collides with a Part 1 name.
