# Provenance Map

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`provenance-map.md`). Traces every `decompose`-dispositioned path in
`source-inventory.md` forward to the behavior units extracted from it and the
destination artifact(s) those units landed in — `draft-workflows/<name>/`,
`permanent-instructions.md`, `helper-map.md`, `shared-context-map.md`,
`obsolete-mechanisms.md`, `engine-pressure.md`, or `classification-ledger.md`
§5's unassigned list. This is a mechanically generated join across
`source-inventory.md`, `behavior-units/*.ndjson`'s `source.path` field, and
`synthesis.md`'s five homing sections (§1–§5) plus §7's unassigned list —
built to satisfy exactly one property: every path a reviewer can look up in
`source-inventory.md` as `decompose` can be followed forward to where its
behavior actually lives in this corpus, and every corpus artifact can be
traced backward to the exact source line it was extracted from (via the unit
id → `behavior-units/*.ndjson` → `quote_hash`).

## Method

1. `source-inventory.md`'s 179 rows were parsed for `(path, disposition)`.
   139 are `decompose`.
2. Every behavior unit's `source.path` field (`behavior-units/*.ndjson`,
   itself carrying `source.locator` and `source.quote_hash` for the exact
   quoted text — the canonical citation, not repeated here) was grouped by
   file.
3. Each unit's **destination artifact** was derived from its representation
   assignment, cross-checked against `synthesis.md`'s own homing sections so
   the destination reflects the *synthesized* corpus home, not a
   partition-local proposal that synthesis overruled (`synthesis.md`'s own
   invariant: "every id has exactly one home" across §1 candidate workflow
   clusters, §2 permanent-instruction articles, §3 shared-context/helper
   candidates, §4 obsolete-mechanism clusters, §5 engine-pressure claims, or
   §7 unassigned — verified to hold exactly for all 979 units before this
   table was built. (Round-1 adjudication added/split units: the corpus grew
   from 966 to 979 after `adjudication-round1.md`'s A10/A12 rulings; this
   invariant was originally verified against 966 and has not been
   re-verified end-to-end against `synthesis.md` §1–§7 for the 13 new units,
   since `synthesis.md` itself is frozen pre-adjudication — see that
   document's superseding note.):
   - `workflow` / `stage` / `stage-context` → `draft-workflows/<slug>/`
     (the `synthesis.md` §1 candidate workflow the unit's own `workflow`
     field, or the workflow whose `Units (N):` list cites it, names).
   - `agents-invariant` → `permanent-instructions.md`, with the specific
     Article.
   - `shared-context` → `shared-context-map.md`, with the specific
     candidate.
   - `helper` / `shared-helper` → `helper-map.md`, with the specific
     candidate.
   - `obsolete-mechanism` → `obsolete-mechanisms.md`, with the specific `Mx`
     cluster.
   - `engine-gap` → `engine-pressure.md`, with the specific `Gx` claim
     (rejected claims still resolve to a specific `Gx` entry there, per
     `engine-pressure.md`'s "each rejected claim listed with the lower rung
     that absorbs it").
   - The 3 units `synthesis.md` §7 explicitly excludes from every category →
     `classification-ledger.md` §5.

## Coverage summary

- **179** total files inventoried; **139** dispositioned `decompose`.
- **Every one of the 139 `decompose` files yielded at least one behavior
  unit — zero gaps.** No `decompose`-dispositioned file has a zero-unit row
  in the table below.
- **One file outside the `decompose` set nonetheless yielded a unit:**
  `.agents/skills/grill-with-docs/agents/openai.yaml` is dispositioned
  `helper-evidence` in `source-inventory.md` (cross-harness display metadata,
  expected to inform `helper-map.md` without itself being extracted), yet
  BU-P3-004 was extracted from it (a real, high-confidence, cited unit — the
  cross-harness mirror of the no-auto-invoke flag, homed to `helper-map.md`'s
  `explicit-invocation-metadata` candidate). This is flagged as a documented
  inconsistency between `source-inventory.md`'s disposition and what
  extraction actually did, not corrected retroactively in either document —
  the unit is real and correctly cited, and `source-inventory.md`'s
  disposition legend already allows `helper-evidence` files to "inform
  `helper-map.md`," which is exactly what happened, just via a formal
  extracted unit rather than an informal read.
- No unit's `source.path` falls outside the 179 inventoried files (checked:
  zero orphans).
- Per-partition `decompose` file counts: P1 5 · P2 7 · P3 10 · P4 9 · P5 6 ·
  P6 36 · P7 60 · P8 6 (139 total). P6/P7's file counts are large relative to
  their unit counts (P6: 143 units / 36 files ≈ 4.0 units/file — updated
  from 142 by round-1 adjudication's A10 split, see the `bin/sgt-validate`
  row below; P7: 112 units / 60 files ≈ 1.9 units/file) because `bin/` and
  `tests/` are
  fine-grained one-script/one-test-per-behavior file layouts, unlike the
  prose skill files in P1–P5 — consistent with `synthesis.md` §8's own
  observation about extraction granularity varying by source-file shape.
- **Round-1 adjudication added/split units: the corpus grew from 966 to 979.**
  Per `adjudication-round1.md`, thirteen ids are new since this table was
  first built: `BU-P1-132`…`BU-P1-137` (six, the AGENTS.md Procedural skills
  routing table, A12/R3-08), `BU-P5-150`…`BU-P5-153` (four, the `dispatch`
  worker-contract's remaining items 6/16/17/18, A12/R3-08), `BU-P8-110` and
  `BU-P8-111` (two, split from the now-retired `BU-P8-077`, A12), and
  `BU-P6-143` (one, split off `BU-P6-129`'s ownership-claim sub-behavior,
  A10 — included here alongside the A12 units because it lands in one of
  the same four rows this fix touches). The table below (four rows:
  `AGENTS.md`, `skills/dispatch/SKILL.md`, `docs/using-sergeant.md`,
  `bin/sgt-validate`) and the file-count sums above reflect all thirteen;
  no other row changed. `BU-P8-077` is retained in its row's id list (not
  deleted) per its own record's id-stability note.
- The 17 symlinked `.claude/skills/*` entries `source-inventory.md` declines
  to separately enumerate (mirroring `.agents/skills/*` 1:1) are correctly
  absent from this table for the same reason — their disposition and
  partition are identical to their target, which the target's row already
  covers.

## Per-file table

Columns: source path (relative to `reference/sergeant-upstream`, per
`source-inventory.md`'s own convention) · partition · unit count · unit ids
(space-separated, sorted) · destination artifact(s), with a `×N` count where
a file's units split across more than one destination. A row flagged
*(inventory disposition: X)* is not itself `decompose` in
`source-inventory.md` — see the coverage-summary note above for the one such
row.

| Source path | Partition | Units | Unit ids | Destination artifact(s) |
|---|---|---|---|---|
| `.agents/skills/code-review/SKILL.md` | P2 | 18 | BU-P2-001 BU-P2-002 BU-P2-003 BU-P2-004 BU-P2-005 BU-P2-006 BU-P2-007 BU-P2-008 BU-P2-009 BU-P2-010 BU-P2-011 BU-P2-012 BU-P2-013 BU-P2-014 BU-P2-015 BU-P2-016 BU-P2-017 BU-P2-018 | draft-workflows/code-review/ ×14; helper-map.md (Workflow-local helpers (kept local)) ×3; shared-context-map.md (Workflow-local contexts (kept local per §6.6)) |
| `.agents/skills/codebase-design/DEEPENING.md` | P4 | 9 | BU-P4-013 BU-P4-014 BU-P4-015 BU-P4-016 BU-P4-017 BU-P4-018 BU-P4-019 BU-P4-020 BU-P4-021 | draft-workflows/deepen-module/ ×7; shared-context-map.md (Workflow-local contexts (kept local per §6.6)) ×2 |
| `.agents/skills/codebase-design/DESIGN-IT-TWICE.md` | P4 | 5 | BU-P4-022 BU-P4-023 BU-P4-024 BU-P4-025 BU-P4-026 | draft-workflows/deepen-module/ ×4; shared-context-map.md (Workflow-local contexts (kept local per §6.6)) |
| `.agents/skills/codebase-design/SKILL.md` | P4 | 12 | BU-P4-001 BU-P4-002 BU-P4-003 BU-P4-004 BU-P4-005 BU-P4-006 BU-P4-007 BU-P4-008 BU-P4-009 BU-P4-010 BU-P4-011 BU-P4-012 | shared-context-map.md (codebase-design-vocabulary) ×12 |
| `.agents/skills/diagnosing-bugs/SKILL.md` | P2 | 31 | BU-P2-019 BU-P2-020 BU-P2-021 BU-P2-022 BU-P2-023 BU-P2-024 BU-P2-025 BU-P2-026 BU-P2-027 BU-P2-028 BU-P2-029 BU-P2-030 BU-P2-031 BU-P2-032 BU-P2-033 BU-P2-034 BU-P2-035 BU-P2-036 BU-P2-037 BU-P2-038 BU-P2-039 BU-P2-040 BU-P2-041 BU-P2-042 BU-P2-043 BU-P2-044 BU-P2-045 BU-P2-046 BU-P2-047 BU-P2-048 BU-P2-049 | draft-workflows/diagnose-bug/ ×29; shared-context-map.md (Workflow-local contexts (kept local per §6.6)); helper-map.md (Workflow-local helpers (kept local)) |
| `.agents/skills/domain-modeling/ADR-FORMAT.md` | P4 | 6 | BU-P4-038 BU-P4-039 BU-P4-040 BU-P4-041 BU-P4-042 BU-P4-043 | helper-map.md (domain-artifacts) ×4; shared-context-map.md (domain-modeling) ×2 |
| `.agents/skills/domain-modeling/CONTEXT-FORMAT.md` | P4 | 6 | BU-P4-044 BU-P4-045 BU-P4-046 BU-P4-047 BU-P4-048 BU-P4-049 | helper-map.md (domain-artifacts) ×4; shared-context-map.md (domain-modeling) ×2 |
| `.agents/skills/domain-modeling/SKILL.md` | P4 | 11 | BU-P4-027 BU-P4-028 BU-P4-029 BU-P4-030 BU-P4-031 BU-P4-032 BU-P4-033 BU-P4-034 BU-P4-035 BU-P4-036 BU-P4-037 | shared-context-map.md (domain-modeling) ×8; helper-map.md (domain-artifacts) ×3 |
| `.agents/skills/grill-with-docs/SKILL.md` | P3 | 3 | BU-P3-001 BU-P3-002 BU-P3-003 | draft-workflows/grill-with-docs/ ×2; helper-map.md (explicit-invocation-metadata) |
| `.agents/skills/grill-with-docs/agents/openai.yaml` *(inventory disposition: helper-evidence)* | P3 | 1 | BU-P3-004 | helper-map.md (explicit-invocation-metadata) |
| `.agents/skills/grilling/SKILL.md` | P3 | 5 | BU-P3-005 BU-P3-006 BU-P3-007 BU-P3-008 BU-P3-009 | draft-workflows/grilling/ ×5 |
| `.agents/skills/implement/SKILL.md` | P2 | 6 | BU-P2-050 BU-P2-051 BU-P2-052 BU-P2-053 BU-P2-054 BU-P2-055 | draft-workflows/implement/ ×6 |
| `.agents/skills/no-mistakes/SKILL.md` | P2 | 48 | BU-P2-056 BU-P2-057 BU-P2-058 BU-P2-059 BU-P2-060 BU-P2-061 BU-P2-062 BU-P2-063 BU-P2-064 BU-P2-065 BU-P2-066 BU-P2-067 BU-P2-068 BU-P2-069 BU-P2-070 BU-P2-071 BU-P2-072 BU-P2-073 BU-P2-074 BU-P2-075 BU-P2-076 BU-P2-077 BU-P2-078 BU-P2-079 BU-P2-080 BU-P2-081 BU-P2-082 BU-P2-083 BU-P2-084 BU-P2-085 BU-P2-086 BU-P2-087 BU-P2-088 BU-P2-089 BU-P2-090 BU-P2-091 BU-P2-092 BU-P2-093 BU-P2-094 BU-P2-095 BU-P2-096 BU-P2-097 BU-P2-098 BU-P2-099 BU-P2-100 BU-P2-101 BU-P2-102 BU-P2-103 | draft-workflows/validate-and-ship/ ×45; engine-pressure.md (G8 runtime role enforcement); helper-map.md (branch-sync-decision-table` and pipeline inspection commands); shared-context-map.md (Workflow-local contexts (kept local per §6.6)) |
| `.agents/skills/prototype/LOGIC.md` | P3 | 9 | BU-P3-020 BU-P3-021 BU-P3-022 BU-P3-023 BU-P3-024 BU-P3-025 BU-P3-026 BU-P3-027 BU-P3-028 | draft-workflows/prototype/ ×9 |
| `.agents/skills/prototype/SKILL.md` | P3 | 10 | BU-P3-010 BU-P3-011 BU-P3-012 BU-P3-013 BU-P3-014 BU-P3-015 BU-P3-016 BU-P3-017 BU-P3-018 BU-P3-019 | draft-workflows/prototype/ ×6; shared-context-map.md (Workflow-local contexts (kept local per §6.6)) ×4 |
| `.agents/skills/prototype/UI.md` | P3 | 11 | BU-P3-029 BU-P3-030 BU-P3-031 BU-P3-032 BU-P3-033 BU-P3-034 BU-P3-035 BU-P3-036 BU-P3-037 BU-P3-038 BU-P3-039 | draft-workflows/prototype/ ×11 |
| `.agents/skills/research/SKILL.md` | P3 | 5 | BU-P3-040 BU-P3-041 BU-P3-042 BU-P3-043 BU-P3-044 | draft-workflows/research/ ×5 |
| `.agents/skills/resolving-merge-conflicts/SKILL.md` | P3 | 6 | BU-P3-045 BU-P3-046 BU-P3-047 BU-P3-048 BU-P3-049 BU-P3-050 | draft-workflows/resolving-merge-conflicts/ ×6 |
| `.agents/skills/sergeant-setup/SKILL.md` | P5 | 37 | BU-P5-001 BU-P5-002 BU-P5-003 BU-P5-004 BU-P5-005 BU-P5-006 BU-P5-007 BU-P5-008 BU-P5-009 BU-P5-010 BU-P5-011 BU-P5-012 BU-P5-013 BU-P5-014 BU-P5-015 BU-P5-016 BU-P5-017 BU-P5-018 BU-P5-019 BU-P5-020 BU-P5-021 BU-P5-022 BU-P5-023 BU-P5-024 BU-P5-025 BU-P5-026 BU-P5-027 BU-P5-028 BU-P5-029 BU-P5-030 BU-P5-031 BU-P5-032 BU-P5-033 BU-P5-034 BU-P5-035 BU-P5-036 BU-P5-037 | draft-workflows/sergeant-setup/ ×36; engine-pressure.md (G5 re-enterable needs-input stage) |
| `.agents/skills/tdd/SKILL.md` | P2 | 13 | BU-P2-104 BU-P2-105 BU-P2-106 BU-P2-107 BU-P2-108 BU-P2-109 BU-P2-110 BU-P2-111 BU-P2-112 BU-P2-113 BU-P2-114 BU-P2-115 BU-P2-116 | draft-workflows/tdd/ ×8; shared-context-map.md (test-quality` (the `tdd` reference half)) ×5 |
| `.agents/skills/tdd/mocking.md` | P2 | 5 | BU-P2-117 BU-P2-118 BU-P2-119 BU-P2-120 BU-P2-121 | shared-context-map.md (test-quality` (the `tdd` reference half)) ×5 |
| `.agents/skills/tdd/tests.md` | P2 | 6 | BU-P2-122 BU-P2-123 BU-P2-124 BU-P2-125 BU-P2-126 BU-P2-127 | shared-context-map.md (test-quality` (the `tdd` reference half)) ×6 |
| `.agents/skills/to-spec/SKILL.md` | P4 | 8 | BU-P4-050 BU-P4-051 BU-P4-052 BU-P4-053 BU-P4-054 BU-P4-055 BU-P4-056 BU-P4-057 | draft-workflows/to-spec/ ×5; helper-map.md (Workflow-local helpers (kept local)) ×3 |
| `.agents/skills/to-tickets/SKILL.md` | P4 | 17 | BU-P4-058 BU-P4-059 BU-P4-060 BU-P4-061 BU-P4-062 BU-P4-063 BU-P4-064 BU-P4-065 BU-P4-066 BU-P4-067 BU-P4-068 BU-P4-069 BU-P4-070 BU-P4-071 BU-P4-072 BU-P4-073 BU-P4-074 | draft-workflows/to-tickets/ ×8; shared-context-map.md (ticket-shaping) ×8; helper-map.md (expand-migrate-contract) |
| `.agents/skills/triage/AGENT-BRIEF.md` | P3 | 7 | BU-P3-076 BU-P3-077 BU-P3-078 BU-P3-079 BU-P3-080 BU-P3-081 BU-P3-082 | helper-map.md (Workflow-local helpers (kept local)) ×7 |
| `.agents/skills/triage/OUT-OF-SCOPE.md` | P3 | 14 | BU-P3-083 BU-P3-084 BU-P3-085 BU-P3-086 BU-P3-087 BU-P3-088 BU-P3-089 BU-P3-090 BU-P3-091 BU-P3-092 BU-P3-093 BU-P3-094 BU-P3-095 BU-P3-096 | shared-context-map.md (triage-state-machine) ×6; draft-workflows/triage/ ×6; helper-map.md (Workflow-local helpers (kept local)) ×2 |
| `.agents/skills/triage/SKILL.md` | P3 | 25 | BU-P3-051 BU-P3-052 BU-P3-053 BU-P3-054 BU-P3-055 BU-P3-056 BU-P3-057 BU-P3-058 BU-P3-059 BU-P3-060 BU-P3-061 BU-P3-062 BU-P3-063 BU-P3-064 BU-P3-065 BU-P3-066 BU-P3-067 BU-P3-068 BU-P3-069 BU-P3-070 BU-P3-071 BU-P3-072 BU-P3-073 BU-P3-074 BU-P3-075 | draft-workflows/triage/ ×23; shared-context-map.md (triage-state-machine) ×2 |
| `.agents/skills/wayfinder/SKILL.md` | P4 | 26 | BU-P4-075 BU-P4-076 BU-P4-077 BU-P4-078 BU-P4-079 BU-P4-080 BU-P4-081 BU-P4-082 BU-P4-083 BU-P4-084 BU-P4-085 BU-P4-086 BU-P4-087 BU-P4-088 BU-P4-089 BU-P4-090 BU-P4-091 BU-P4-092 BU-P4-093 BU-P4-094 BU-P4-095 BU-P4-096 BU-P4-097 BU-P4-098 BU-P4-099 BU-P4-100 | draft-workflows/wayfinder/ ×17; helper-map.md (Workflow-local helpers (kept local)) ×6; shared-context-map.md (Workflow-local contexts (kept local per §6.6)) ×2; engine-pressure.md (G7 dynamic ticket graph) |
| `AGENTS.md` | P1 | 67 | BU-P1-001 BU-P1-002 BU-P1-003 BU-P1-004 BU-P1-005 BU-P1-006 BU-P1-007 BU-P1-008 BU-P1-009 BU-P1-010 BU-P1-011 BU-P1-012 BU-P1-013 BU-P1-014 BU-P1-015 BU-P1-016 BU-P1-017 BU-P1-018 BU-P1-019 BU-P1-020 BU-P1-021 BU-P1-022 BU-P1-023 BU-P1-024 BU-P1-025 BU-P1-026 BU-P1-027 BU-P1-028 BU-P1-029 BU-P1-030 BU-P1-031 BU-P1-032 BU-P1-033 BU-P1-034 BU-P1-035 BU-P1-036 BU-P1-037 BU-P1-038 BU-P1-039 BU-P1-040 BU-P1-041 BU-P1-042 BU-P1-043 BU-P1-044 BU-P1-045 BU-P1-046 BU-P1-047 BU-P1-048 BU-P1-049 BU-P1-050 BU-P1-051 BU-P1-052 BU-P1-053 BU-P1-054 BU-P1-055 BU-P1-056 BU-P1-057 BU-P1-058 BU-P1-059 BU-P1-060 BU-P1-061 BU-P1-132 BU-P1-133 BU-P1-134 BU-P1-135 BU-P1-136 BU-P1-137 | draft-workflows/task-intake-and-route/ ×12; draft-workflows/direct-implementation/ ×9; permanent-instructions.md (Article VIII — Procedure discovery and loading) ×7; draft-workflows/dispatch/ ×6; permanent-instructions.md (Article II — Roles and execution mode) ×3; permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent) ×3; permanent-instructions.md (Article III — Authority boundaries and ownership) ×3; permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline) ×3; shared-context-map.md (project-configuration) ×3; permanent-instructions.md (Article VII — Instruction and documentation authority) ×2; shared-context-map.md (worker-state-vocabulary) ×2; shared-context-map.md (intent-provenance) ×2; draft-workflows/validate-and-ship/ ×2; permanent-instructions.md (Article I — Resolve before acting); permanent-instructions.md (Article VI — Secrets, privacy, and transport); permanent-instructions.md (Article V — Measured, not assumed); obsolete-mechanisms.md (M1 Coordinator-pane binding and identity proof); shared-context-map.md (`@@procedural-skills-routing` — pending: no named candidate yet exists for this table as of this fix; overlaps `@@skill-discovery`'s BU-P1-117, see that unit's own note; N1 adjudication A12, R3-08) ×6 |
| `README.md` | P1 | 36 | BU-P1-062 BU-P1-063 BU-P1-064 BU-P1-065 BU-P1-066 BU-P1-067 BU-P1-068 BU-P1-069 BU-P1-070 BU-P1-071 BU-P1-072 BU-P1-073 BU-P1-074 BU-P1-075 BU-P1-076 BU-P1-077 BU-P1-078 BU-P1-079 BU-P1-080 BU-P1-081 BU-P1-082 BU-P1-083 BU-P1-084 BU-P1-085 BU-P1-086 BU-P1-087 BU-P1-088 BU-P1-089 BU-P1-090 BU-P1-091 BU-P1-092 BU-P1-093 BU-P1-094 BU-P1-095 BU-P1-096 BU-P1-097 | draft-workflows/validate-and-ship/ ×9; helper-map.md (finding-router) ×9; classification-ledger.md §5 (unassigned) ×2; shared-context-map.md (project-configuration) ×2; permanent-instructions.md (Article III — Authority boundaries and ownership) ×2; helper-map.md (branch-sync-decision-table` and pipeline inspection commands) ×2; shared-context-map.md (@@launch-record) ×2; draft-workflows/dispatch/ ×2; permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline); permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity); shared-context-map.md (@@review-axes); obsolete-mechanisms.md (M1 Coordinator-pane binding and identity proof); obsolete-mechanisms.md (M3 Pane as the notification channel); shared-context-map.md (@@drain-admission-lock) |
| `bin/_sgt-drain.sh` | P6 | 8 | BU-P6-052 BU-P6-053 BU-P6-057 BU-P6-058 BU-P6-059 BU-P6-060 BU-P6-061 BU-P6-062 | helper-map.md (mutual-exclusion) ×3; draft-workflows/drain-fleet/ ×3; permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent) ×2 |
| `bin/_sgt-harness.sh` | P6 | 6 | BU-P6-065 BU-P6-066 BU-P6-067 BU-P6-068 BU-P6-069 BU-P6-070 | helper-map.md (harness-registry) ×2; obsolete-mechanisms.md (M4 Pane as the liveness signal) ×2; permanent-instructions.md (Article V — Measured, not assumed); permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity) |
| `bin/_sgt-intent.sh` | P6 | 5 | BU-P6-046 BU-P6-047 BU-P6-048 BU-P6-049 BU-P6-050 | permanent-instructions.md (Article V — Measured, not assumed); permanent-instructions.md (Article VI — Secrets, privacy, and transport); draft-workflows/dispatch/; shared-context-map.md (intent-provenance); helper-map.md (intent-intake) |
| `bin/_sgt-response-lock.sh` | P6 | 4 | BU-P6-051 BU-P6-054 BU-P6-055 BU-P6-056 | helper-map.md (action-lease) ×2; shared-context-map.md (response-evidence-schema); permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent) |
| `bin/sgt-ack-response` | P6 | 4 | BU-P6-031 BU-P6-032 BU-P6-033 BU-P6-034 | draft-workflows/respond-to-worker/ ×2; obsolete-mechanisms.md (M5 Loose worktree files as durable state); shared-context-map.md (response-evidence-schema) |
| `bin/sgt-callback` | P6 | 6 | BU-P6-117 BU-P6-118 BU-P6-119 BU-P6-120 BU-P6-121 BU-P6-122 | draft-workflows/deliver-external-callback/ ×4; helper-map.md (callback-plumbing); helper-map.md (payload-safety) |
| `bin/sgt-cleanup` | P6 | 8 | BU-P6-135 BU-P6-136 BU-P6-137 BU-P6-138 BU-P6-139 BU-P6-140 BU-P6-141 BU-P6-142 | draft-workflows/reconcile-and-cleanup-fleet/ ×6; helper-map.md (process-identity); shared-context-map.md (response-evidence-schema) |
| `bin/sgt-context` | P6 | 3 | BU-P6-020 BU-P6-021 BU-P6-022 | draft-workflows/load-project/ ×2; helper-map.md (project-configuration) |
| `bin/sgt-dag-dispatch-hook` | P6 | 1 | BU-P6-016 | engine-pressure.md (G2 fleet identity + dependency advance) |
| `bin/sgt-dag-run` | P6 | 1 | BU-P6-017 | engine-pressure.md (G2 fleet identity + dependency advance) |
| `bin/sgt-dispatch` | P6 | 6 | BU-P6-123 BU-P6-124 BU-P6-125 BU-P6-126 BU-P6-127 BU-P6-128 | draft-workflows/dispatch/ ×5; obsolete-mechanisms.md (M7 Pane-scoped rollback) |
| `bin/sgt-drain` | P6 | 2 | BU-P6-063 BU-P6-064 | engine-pressure.md (G4 admission block); draft-workflows/drain-fleet/ |
| `bin/sgt-drain-force` | P6 | 3 | BU-P6-039 BU-P6-040 BU-P6-041 | draft-workflows/drain-fleet/; helper-map.md (process-identity); obsolete-mechanisms.md (M6 Detached background harness sessions) |
| `bin/sgt-graphify` | P6 | 4 | BU-P6-088 BU-P6-089 BU-P6-090 BU-P6-091 | draft-workflows/project-graph/ ×4 |
| `bin/sgt-interactive-worker` | P6 | 10 | BU-P6-107 BU-P6-108 BU-P6-109 BU-P6-110 BU-P6-111 BU-P6-112 BU-P6-113 BU-P6-114 BU-P6-115 BU-P6-116 | draft-workflows/dispatch/ ×5; permanent-instructions.md (Article V — Measured, not assumed); shared-context-map.md (launch-evidence); draft-workflows/drain-fleet/; obsolete-mechanisms.md (M2 Pane as the worker's process); draft-workflows/respond-to-worker/ |
| `bin/sgt-list` | P6 | 2 | BU-P6-010 BU-P6-011 | draft-workflows/load-project/ ×2 |
| `bin/sgt-no-mistakes-finding` | P6 | 4 | BU-P6-023 BU-P6-024 BU-P6-025 BU-P6-026 | draft-workflows/validate-and-ship/ ×2; shared-context-map.md (review-severity-and-axes); helper-map.md (finding-router) |
| `bin/sgt-notify` | P6 | 4 | BU-P6-027 BU-P6-028 BU-P6-029 BU-P6-030 | draft-workflows/respond-to-worker/ ×2; permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity); obsolete-mechanisms.md (M3 Pane as the notification channel) |
| `bin/sgt-recover` | P6 | 5 | BU-P6-071 BU-P6-072 BU-P6-073 BU-P6-074 BU-P6-075 | draft-workflows/recover-stalled-worker/ ×4; obsolete-mechanisms.md (M6 Detached background harness sessions) |
| `bin/sgt-respond` | P6 | 6 | BU-P6-076 BU-P6-077 BU-P6-078 BU-P6-079 BU-P6-080 BU-P6-081 | draft-workflows/respond-to-worker/ ×3; obsolete-mechanisms.md (M8 Response delivery/acknowledgement split); obsolete-mechanisms.md (M5 Loose worktree files as durable state); obsolete-mechanisms.md (M6 Detached background harness sessions) |
| `bin/sgt-review-findings` | P6 | 6 | BU-P6-082 BU-P6-083 BU-P6-084 BU-P6-085 BU-P6-086 BU-P6-087 | draft-workflows/route-review-findings/ ×4; shared-context-map.md (review-severity-and-axes); helper-map.md (finding-router) |
| `bin/sgt-status` | P6 | 1 | BU-P6-012 | draft-workflows/load-project/ |
| `bin/sgt-sync` | P6 | 2 | BU-P6-013 BU-P6-014 | draft-workflows/load-project/ ×2 |
| `bin/sgt-td-create` | P6 | 1 | BU-P6-036 | draft-workflows/dispatch/ |
| `bin/sgt-td-list` | P6 | 1 | BU-P6-035 | draft-workflows/load-project/ |
| `bin/sgt-td-memory` | P6 | 2 | BU-P6-037 BU-P6-038 | helper-map.md (handoff-provenance); permanent-instructions.md (Article VI — Secrets, privacy, and transport) |
| `bin/sgt-treehouse-init` | P6 | 2 | BU-P6-018 BU-P6-019 | draft-workflows/sergeant-setup/; draft-workflows/dispatch/ |
| `bin/sgt-undrain` | P6 | 1 | BU-P6-015 | draft-workflows/drain-fleet/ |
| `bin/sgt-validate` | P6 | 7 | BU-P6-129 BU-P6-130 BU-P6-131 BU-P6-132 BU-P6-133 BU-P6-134 BU-P6-143 | draft-workflows/validate-and-ship/ ×4; obsolete-mechanisms.md (M1 Coordinator-pane binding and identity proof); helper-map.md (owned-write); draft-workflows/validate-and-ship/ (pending — `BU-P6-143`, the ownership-claim durable-rule half split off `BU-P6-129` at N1 adjudication A10, not yet cited in that package's `provenance.md` as of this fix) ×1 |
| `bin/sgt-validation-worker` | P6 | 4 | BU-P6-042 BU-P6-043 BU-P6-044 BU-P6-045 | draft-workflows/validate-and-ship/ ×2; obsolete-mechanisms.md (M6 Detached background harness sessions); permanent-instructions.md (Article VI — Secrets, privacy, and transport) |
| `bin/sgt-wake` | P6 | 5 | BU-P6-096 BU-P6-097 BU-P6-098 BU-P6-099 BU-P6-100 | draft-workflows/wake-and-resume/ ×4; engine-pressure.md (G1 wait/wake scheduling) |
| `bin/sgt-watch` | P6 | 6 | BU-P6-101 BU-P6-102 BU-P6-103 BU-P6-104 BU-P6-105 BU-P6-106 | draft-workflows/monitor-fleet/ ×4; permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent); obsolete-mechanisms.md (M4 Pane as the liveness signal) |
| `bin/wiki-daily-digest` | P6 | 4 | BU-P6-092 BU-P6-093 BU-P6-094 BU-P6-095 | draft-workflows/wiki-digest/ ×2; shared-context-map.md (wiki-conventions) ×2 |
| `docs/README.md` | P8 | 6 | BU-P8-001 BU-P8-002 BU-P8-003 BU-P8-004 BU-P8-005 BU-P8-006 | permanent-instructions.md (Article VII — Instruction and documentation authority) ×4; permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline); permanent-instructions.md (Article VI — Secrets, privacy, and transport) |
| `docs/callbacks.md` | P8 | 22 | BU-P8-007 BU-P8-008 BU-P8-009 BU-P8-010 BU-P8-011 BU-P8-012 BU-P8-013 BU-P8-014 BU-P8-015 BU-P8-016 BU-P8-017 BU-P8-018 BU-P8-019 BU-P8-020 BU-P8-021 BU-P8-022 BU-P8-023 BU-P8-024 BU-P8-025 BU-P8-026 BU-P8-027 BU-P8-028 | helper-map.md (callback-plumbing) ×11; shared-context-map.md (callback-protocol) ×4; permanent-instructions.md (Article VI — Secrets, privacy, and transport) ×3; draft-workflows/reconcile-and-cleanup-fleet/ ×2; engine-pressure.md (G3 acknowledgement gate on cleanup); helper-map.md (payload-safety) |
| `docs/getting-started.md` | P8 | 11 | BU-P8-041 BU-P8-042 BU-P8-043 BU-P8-044 BU-P8-045 BU-P8-046 BU-P8-047 BU-P8-048 BU-P8-049 BU-P8-050 BU-P8-051 | draft-workflows/sergeant-setup/ ×8; permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline); permanent-instructions.md (Article V — Measured, not assumed); obsolete-mechanisms.md (M1 Coordinator-pane binding and identity proof) |
| `docs/repo-scoped-skills.md` | P1 | 4 | BU-P1-128 BU-P1-129 BU-P1-130 BU-P1-131 | shared-context-map.md (@@worker-brief-skill-bundle) ×2; permanent-instructions.md (Article VIII — Procedure discovery and loading); permanent-instructions.md (Article III — Authority boundaries and ownership) |
| `docs/schema.md` | P8 | 12 | BU-P8-029 BU-P8-030 BU-P8-031 BU-P8-032 BU-P8-033 BU-P8-034 BU-P8-035 BU-P8-036 BU-P8-037 BU-P8-038 BU-P8-039 BU-P8-040 | shared-context-map.md (project-configuration) ×3; helper-map.md (project-configuration) ×3; helper-map.md (graphify-plumbing) ×3; permanent-instructions.md (Article VI — Secrets, privacy, and transport); shared-context-map.md (dispatch-routing-context); helper-map.md (identity-precedence) |
| `docs/skills.md` | P1 | 16 | BU-P1-112 BU-P1-113 BU-P1-114 BU-P1-115 BU-P1-116 BU-P1-117 BU-P1-118 BU-P1-119 BU-P1-120 BU-P1-121 BU-P1-122 BU-P1-123 BU-P1-124 BU-P1-125 BU-P1-126 BU-P1-127 | draft-workflows/vet-external-skill/ ×9; permanent-instructions.md (Article VIII — Procedure discovery and loading) ×5; shared-context-map.md (@@skill-locations); shared-context-map.md (skill-discovery) |
| `docs/troubleshooting.md` | P8 | 16 | BU-P8-094 BU-P8-095 BU-P8-096 BU-P8-097 BU-P8-098 BU-P8-099 BU-P8-100 BU-P8-101 BU-P8-102 BU-P8-103 BU-P8-104 BU-P8-105 BU-P8-106 BU-P8-107 BU-P8-108 BU-P8-109 | permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent) ×7; draft-workflows/recover-stalled-worker/ ×3; permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity) ×2; helper-map.md (worker-state-vocabulary); permanent-instructions.md (Article III — Authority boundaries and ownership); obsolete-mechanisms.md (M9 Shell-distribution targets); draft-workflows/project-graph/ |
| `docs/using-sergeant.md` | P8 | 44 | BU-P8-052 BU-P8-053 BU-P8-054 BU-P8-055 BU-P8-056 BU-P8-057 BU-P8-058 BU-P8-059 BU-P8-060 BU-P8-061 BU-P8-062 BU-P8-063 BU-P8-064 BU-P8-065 BU-P8-066 BU-P8-067 BU-P8-068 BU-P8-069 BU-P8-070 BU-P8-071 BU-P8-072 BU-P8-073 BU-P8-074 BU-P8-075 BU-P8-076 BU-P8-077 BU-P8-078 BU-P8-079 BU-P8-080 BU-P8-081 BU-P8-082 BU-P8-083 BU-P8-084 BU-P8-085 BU-P8-086 BU-P8-087 BU-P8-088 BU-P8-089 BU-P8-090 BU-P8-091 BU-P8-092 BU-P8-093 BU-P8-110 BU-P8-111 | permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity) ×6; draft-workflows/validate-and-ship/ ×5; draft-workflows/direct-implementation/ ×3; draft-workflows/dispatch/ ×3; permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent) ×3; draft-workflows/task-intake-and-route/ ×2; permanent-instructions.md (Article VI — Secrets, privacy, and transport) ×2; permanent-instructions.md (Article III — Authority boundaries and ownership) ×2; permanent-instructions.md (Article I — Resolve before acting); permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline); helper-map.md (capability-probe); permanent-instructions.md (Article V — Measured, not assumed); obsolete-mechanisms.md (M9 Shell-distribution targets); obsolete-mechanisms.md (M1 Coordinator-pane binding and identity proof); obsolete-mechanisms.md (M2 Pane as the worker's process); obsolete-mechanisms.md (M3 Pane as the notification channel); helper-map.md (notify-marker); draft-workflows/monitor-fleet/; shared-context-map.md (worker-state-vocabulary); shared-context-map.md (wake-conditions); draft-workflows/drain-fleet/ ×2 (`BU-P8-110`, `BU-P8-111` — successors of `BU-P8-077`, split at N1 adjudication A12 and re-cited in that package's `provenance.md`/`30-force-stop/CONTEXT.md` in place of the now-retired unit at N1 verifier round 2 finding V4; `BU-P8-077` itself is retained above for id stability only, per its own record's "do not cite in new work" note); draft-workflows/respond-to-worker/; draft-workflows/reconcile-and-cleanup-fleet/; shared-context-map.md (wiki-conventions) |
| `docs/what-is-sergeant.md` | P1 | 14 | BU-P1-098 BU-P1-099 BU-P1-100 BU-P1-101 BU-P1-102 BU-P1-103 BU-P1-104 BU-P1-105 BU-P1-106 BU-P1-107 BU-P1-108 BU-P1-109 BU-P1-110 BU-P1-111 | permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline) ×3; permanent-instructions.md (Article III — Authority boundaries and ownership) ×2; shared-context-map.md (@@installation-ownership-boundary); shared-context-map.md (@@project); shared-context-map.md (@@repository); shared-context-map.md (@@task); shared-context-map.md (@@fleet); shared-context-map.md (@@worker); shared-context-map.md (@@decision-request); draft-workflows/direct-implementation/; draft-workflows/task-intake-and-route/ |
| `mise.toml` | P6 | 5 | BU-P6-001 BU-P6-002 BU-P6-003 BU-P6-004 BU-P6-005 | helper-map.md (install-plumbing) ×2; draft-workflows/sergeant-setup/; helper-map.md (capability-probe); helper-map.md (harness-registry) |
| `opencode.json` | P6 | 1 | BU-P6-006 | shared-context-map.md (skill-discovery) |
| `schema/project.yaml.example` | P7 | 4 | BU-P7-001 BU-P7-002 BU-P7-003 BU-P7-004 | shared-context-map.md (project-configuration) ×2; draft-workflows/dispatch/; draft-workflows/project-graph/ |
| `scripts/hooks/pre-push` | P6 | 3 | BU-P6-007 BU-P6-008 BU-P6-009 | draft-workflows/validate-and-ship/ ×2 (re-homed from draft-workflows/repo-release-verification/, N1 adjudication A6 — see Tombstones); helper-map.md (install-plumbing) |
| `skills/cross-repo-work/SKILL.md` | P5 | 16 | BU-P5-038 BU-P5-039 BU-P5-040 BU-P5-041 BU-P5-042 BU-P5-043 BU-P5-044 BU-P5-045 BU-P5-046 BU-P5-047 BU-P5-048 BU-P5-049 BU-P5-050 BU-P5-051 BU-P5-052 BU-P5-053 | draft-workflows/cross-repo-work/ ×15; draft-workflows/load-project/ |
| `skills/dispatch/SKILL.md` | P5 | 40 | BU-P5-054 BU-P5-055 BU-P5-056 BU-P5-057 BU-P5-058 BU-P5-059 BU-P5-060 BU-P5-061 BU-P5-062 BU-P5-063 BU-P5-064 BU-P5-065 BU-P5-066 BU-P5-067 BU-P5-068 BU-P5-069 BU-P5-070 BU-P5-071 BU-P5-072 BU-P5-073 BU-P5-074 BU-P5-075 BU-P5-076 BU-P5-077 BU-P5-078 BU-P5-079 BU-P5-080 BU-P5-081 BU-P5-082 BU-P5-083 BU-P5-084 BU-P5-085 BU-P5-086 BU-P5-087 BU-P5-088 BU-P5-089 BU-P5-150 BU-P5-151 BU-P5-152 BU-P5-153 | draft-workflows/dispatch/ ×32 (includes `BU-P5-150`/`151`/`152`/`153`, dispatch worker-contract items 6/16/17/18, extracted at N1 adjudication A12/R3-08 and routed into `80-monitor`'s `60-render-brief` at N1 verifier round 2 finding V3 — see that package's `provenance.md` and `80-monitor/CONTEXT.md`); obsolete-mechanisms.md (M2 Pane as the worker's process) ×2; helper-map.md (worktree-pool) ×2; draft-workflows/cross-repo-work/; obsolete-mechanisms.md (M5 Loose worktree files as durable state); engine-pressure.md (G2 fleet identity + dependency advance); engine-pressure.md (G6 child-workflow invocation) |
| `skills/load-project/SKILL.md` | P5 | 23 | BU-P5-090 BU-P5-091 BU-P5-092 BU-P5-093 BU-P5-094 BU-P5-095 BU-P5-096 BU-P5-097 BU-P5-098 BU-P5-099 BU-P5-100 BU-P5-101 BU-P5-102 BU-P5-103 BU-P5-104 BU-P5-105 BU-P5-106 BU-P5-107 BU-P5-108 BU-P5-109 BU-P5-110 BU-P5-111 BU-P5-112 | draft-workflows/load-project/ ×17; draft-workflows/project-graph/ ×5; permanent-instructions.md (Article VII — Instruction and documentation authority) |
| `skills/sergeant-help/SKILL.md` | P5 | 17 | BU-P5-113 BU-P5-114 BU-P5-115 BU-P5-116 BU-P5-117 BU-P5-118 BU-P5-119 BU-P5-120 BU-P5-121 BU-P5-122 BU-P5-123 BU-P5-124 BU-P5-125 BU-P5-126 BU-P5-127 BU-P5-128 BU-P5-129 | draft-workflows/sergeant-help/ ×13; helper-map.md (Workflow-local helpers (kept local)) ×3; permanent-instructions.md (Article VII — Instruction and documentation authority) |
| `skills/wiki/SKILL.md` | P5 | 20 | BU-P5-130 BU-P5-131 BU-P5-132 BU-P5-133 BU-P5-134 BU-P5-135 BU-P5-136 BU-P5-137 BU-P5-138 BU-P5-139 BU-P5-140 BU-P5-141 BU-P5-142 BU-P5-143 BU-P5-144 BU-P5-145 BU-P5-146 BU-P5-147 BU-P5-148 BU-P5-149 | draft-workflows/wiki-digest/ ×14; shared-context-map.md (wiki-conventions) ×3; helper-map.md (Workflow-local helpers (kept local)) ×3 |
| `templates/worker-brief.md` | P7 | 10 | BU-P7-005 BU-P7-006 BU-P7-007 BU-P7-008 BU-P7-009 BU-P7-010 BU-P7-011 BU-P7-012 BU-P7-013 BU-P7-098 | draft-workflows/worker-mission/ ×5; permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity); obsolete-mechanisms.md (M2 Pane as the worker's process); engine-pressure.md (G1 wait/wake scheduling); permanent-instructions.md (Article IV — Evidence over optimism; fail closed, never fail silent); draft-workflows/wake-and-resume/ |
| `tests/global-state-isolation-test.sh` | P7 | 5 | BU-P7-020 BU-P7-021 BU-P7-022 BU-P7-023 BU-P7-024 | permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity) ×2; helper-map.md (test-infrastructure` (source-repo self-hosting)) ×2; classification-ledger.md §5 (unassigned) |
| `tests/instruction-policy-test.sh` | P7 | 5 | BU-P7-014 BU-P7-015 BU-P7-016 BU-P7-017 BU-P7-018 | permanent-instructions.md (Article VII — Instruction and documentation authority) ×2; shared-context-map.md (skill-discovery); draft-workflows/dispatch/; draft-workflows/cross-repo-work/ |
| `tests/mise-check-test.sh` | P7 | 3 | BU-P7-025 BU-P7-026 BU-P7-027 | draft-workflows/sergeant-setup/; shared-context-map.md (skill-discovery); helper-map.md (capability-probe) |
| `tests/mise-install-test.sh` | P7 | 2 | BU-P7-028 BU-P7-029 | helper-map.md (install-plumbing) ×2 |
| `tests/no-remote-test.sh` | P7 | 1 | BU-P7-019 | permanent-instructions.md (Article IX — Scope, deployment model, and delivery discipline) |
| `tests/repo-skills-test.sh` | P7 | 3 | BU-P7-030 BU-P7-031 BU-P7-032 | shared-context-map.md (skill-discovery) ×3 |
| `tests/runtime-bash-test.sh` | P7 | 3 | BU-P7-033 BU-P7-034 BU-P7-035 | permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity); helper-map.md (capability-probe); draft-workflows/respond-to-worker/ |
| `tests/sergeant-setup-test.sh` | P7 | 5 | BU-P7-036 BU-P7-037 BU-P7-038 BU-P7-039 BU-P7-040 | draft-workflows/sergeant-setup/ ×5 |
| `tests/sgt-ack-response-test.sh` | P7 | 4 | BU-P7-041 BU-P7-042 BU-P7-043 BU-P7-044 | draft-workflows/respond-to-worker/ ×3; engine-pressure.md (G9 crash-safe publication) |
| `tests/sgt-callback-test.sh` | P7 | 2 | BU-P7-067 BU-P7-068 | draft-workflows/deliver-external-callback/; shared-context-map.md (callback-protocol) |
| `tests/sgt-cleanup-cross-filesystem-test.sh` | P7 | 1 | BU-P7-079 | engine-pressure.md (G9 crash-safe publication) |
| `tests/sgt-cleanup-test.sh` | P7 | 3 | BU-P7-080 BU-P7-081 BU-P7-082 | permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity); draft-workflows/reconcile-and-cleanup-fleet/; helper-map.md (process-identity) |
| `tests/sgt-dispatch-adopt-branch-test.sh` | P7 | 2 | BU-P7-069 BU-P7-070 | draft-workflows/dispatch/ ×2 |
| `tests/sgt-dispatch-bash32-test.sh` | P7 | 1 | BU-P7-071 | permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity) |
| `tests/sgt-dispatch-brief-test.sh` | P7 | 1 | BU-P7-112 | draft-workflows/dispatch/ |
| `tests/sgt-dispatch-coordinator-pane-test.sh` | P7 | 1 | BU-P7-078 | draft-workflows/dispatch/ |
| `tests/sgt-dispatch-identity-test.sh` | P7 | 1 | BU-P7-072 | draft-workflows/dispatch/ |
| `tests/sgt-dispatch-model-tuple-test.sh` | P7 | 1 | BU-P7-073 | draft-workflows/dispatch/ |
| `tests/sgt-dispatch-oc-target-test.sh` | P7 | 1 | BU-P7-074 | draft-workflows/dispatch/ |
| `tests/sgt-dispatch-td-test.sh` | P7 | 1 | BU-P7-075 | draft-workflows/dispatch/ |
| `tests/sgt-dispatch-unpushed-guard-test.sh` | P7 | 1 | BU-P7-076 | helper-map.md (worktree-pool) |
| `tests/sgt-dispatch-worker-test.sh` | P7 | 1 | BU-P7-077 | permanent-instructions.md (Article X — Deferred work, recovery, and installation integrity) |
| `tests/sgt-drain-force-test.sh` | P7 | 1 | BU-P7-083 | draft-workflows/drain-fleet/ |
| `tests/sgt-drain-terminate-test.sh` | P7 | 1 | BU-P7-084 | draft-workflows/drain-fleet/ |
| `tests/sgt-drain-test.sh` | P7 | 1 | BU-P7-085 | helper-map.md (mutual-exclusion) |
| `tests/sgt-drain-worker-test.sh` | P7 | 1 | BU-P7-108 | draft-workflows/drain-fleet/ |
| `tests/sgt-graphify-test.sh` | P7 | 3 | BU-P7-086 BU-P7-087 BU-P7-088 | draft-workflows/project-graph/ ×2; helper-map.md (graphify-plumbing) |
| `tests/sgt-harness-test.sh` | P7 | 2 | BU-P7-089 BU-P7-090 | helper-map.md (harness-registry) ×2 |
| `tests/sgt-interrupted-fallback-test.sh` | P7 | 1 | BU-P7-091 | shared-context-map.md (recovery-visibility) |
| `tests/sgt-lease-convergence-test.sh` | P7 | 2 | BU-P7-049 BU-P7-050 | engine-pressure.md (G9 crash-safe publication); helper-map.md (action-lease) |
| `tests/sgt-lease-exit-branch-test.sh` | P7 | 1 | BU-P7-051 | engine-pressure.md (G9 crash-safe publication) |
| `tests/sgt-lease-finalizer-test.sh` | P7 | 2 | BU-P7-052 BU-P7-053 | draft-workflows/respond-to-worker/ ×2 |
| `tests/sgt-lib-notification-target-test.sh` | P7 | 2 | BU-P7-054 BU-P7-055 | helper-map.md (owned-write) ×2 |
| `tests/sgt-lib-owned-file-test.sh` | P7 | 2 | BU-P7-056 BU-P7-057 | helper-map.md (owned-write) ×2 |
| `tests/sgt-no-mistakes-finding-test.sh` | P7 | 1 | BU-P7-065 | draft-workflows/validate-and-ship/ |
| `tests/sgt-notify-test.sh` | P7 | 2 | BU-P7-047 BU-P7-048 | draft-workflows/respond-to-worker/ ×2 |
| `tests/sgt-recover-drain-test.sh` | P7 | 1 | BU-P7-092 | draft-workflows/recover-stalled-worker/ |
| `tests/sgt-recover-lease-owner-test.sh` | P7 | 1 | BU-P7-093 | draft-workflows/recover-stalled-worker/ |
| `tests/sgt-recover-replacement-test.sh` | P7 | 1 | BU-P7-094 | draft-workflows/recover-stalled-worker/ |
| `tests/sgt-recover-test.sh` | P7 | 1 | BU-P7-095 | draft-workflows/recover-stalled-worker/ |
| `tests/sgt-respond-drain-test.sh` | P7 | 1 | BU-P7-058 | draft-workflows/respond-to-worker/ |
| `tests/sgt-respond-recovery-test.sh` | P7 | 1 | BU-P7-059 | draft-workflows/respond-to-worker/ |
| `tests/sgt-respond-test.sh` | P7 | 1 | BU-P7-060 | draft-workflows/respond-to-worker/ |
| `tests/sgt-response-lock-release-test.sh` | P7 | 2 | BU-P7-045 BU-P7-046 | helper-map.md (action-lease) ×2 |
| `tests/sgt-review-findings-test.sh` | P7 | 4 | BU-P7-061 BU-P7-062 BU-P7-063 BU-P7-064 | shared-context-map.md (review-severity-and-axes) ×2; draft-workflows/route-review-findings/ ×2 |
| `tests/sgt-td-memory-worktree-test.sh` | P7 | 1 | BU-P7-066 | draft-workflows/worker-mission/ |
| `tests/sgt-validate-test.sh` | P7 | 2 | BU-P7-103 BU-P7-104 | helper-map.md (capability-probe); draft-workflows/validate-and-ship/ |
| `tests/sgt-validation-worker-test.sh` | P7 | 1 | BU-P7-105 | draft-workflows/validate-and-ship/ |
| `tests/sgt-wake-test.sh` | P7 | 2 | BU-P7-096 BU-P7-097 | engine-pressure.md (G1 wait/wake scheduling); draft-workflows/wake-and-resume/ |
| `tests/sgt-watch-background-test.sh` | P7 | 1 | BU-P7-099 | draft-workflows/monitor-fleet/ |
| `tests/sgt-watch-recycle-test.sh` | P7 | 1 | BU-P7-100 | draft-workflows/monitor-fleet/ |
| `tests/sgt-watch-snapshot-test.sh` | P7 | 1 | BU-P7-101 | draft-workflows/monitor-fleet/ |
| `tests/sgt-watch-test.sh` | P7 | 1 | BU-P7-102 | helper-map.md (state-resolution-consistency) |
| `tests/sgt-worker-drain-test.sh` | P7 | 1 | BU-P7-107 | draft-workflows/drain-fleet/ |
| `tests/sgt-worker-handshake-test.sh` | P7 | 1 | BU-P7-109 | draft-workflows/respond-to-worker/ |
| `tests/sgt-worker-model-tuple-test.sh` | P7 | 1 | BU-P7-111 | draft-workflows/dispatch/ |
| `tests/sgt-worker-readiness-test.sh` | P7 | 1 | BU-P7-110 | draft-workflows/worker-mission/ |
| `tests/sgt-worker-test.sh` | P7 | 1 | BU-P7-106 | obsolete-mechanisms.md (M2 Pane as the worker's process) |

## Tombstones

Destinations removed after this map was built, with where their units actually live now.

- **`draft-workflows/repo-release-verification/`** (candidate **W19**, `synthesis.md` §1) — removed per N1 adjudication A6 (finding N1-BH-06, `adjudication-round1.md`). The package was demoted from a standalone workflow: its split from `validate-and-ship` during synthesis was file-shape mirroring (matching the proposal's own worked example by name), and §6.2's workflow test was never actually argued for it. Its two units, `BU-P6-007` and `BU-P6-008` (this repository's own git pre-push hook), are re-homed as a helper inside `draft-workflows/validate-and-ship/20-select-intent-transport/`. The row above (`scripts/hooks/pre-push`) has been updated in place to point at the new destination; this entry is the tombstone for the removed package directory itself. See `draft-workflows/validate-and-ship/provenance.md`'s "Re-homed from repo-release-verification (A6)" section for the full record.