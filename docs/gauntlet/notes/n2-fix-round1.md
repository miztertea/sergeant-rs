# N2 fix round 1 — dispositions

Fixes applied against `docs/gauntlet/notes/n2-build-critique.json`'s 31
findings, targeting `.sergeant/workflows/repo-to-icm/`. Method: read each
finding's full evidence text, fixed at the source (stage `CONTEXT.md`,
`references/`, `_config/`, `scripts/validate-structure.py`), then
re-validated. No file under `reference/`, `reference-corpus/`, `src/`,
`CLAUDE.md`, `GAUNTLET.md`, or `LESSONS.md` was touched. Nothing committed.

All 7 errors and all 12 majors are fixed. Of 12 minors, 11 are fixed and 1
(F16) is declined with reason below — it names a real structural tension,
but resolving it requires a `docs/icm/convention.md`-level policy decision
(loosening the Inputs-table traversal boundary, or adding a new
"governing document" citation class) that is out of scope for a mechanical
N2 fix round.

## Errors (all fixed)

| ID | Fix |
|---|---|
| C1 | Every L4 Inputs row across all 10 stages now names the real per-run artifact (`contract.md`, `inventory.md`, `behavior-units.ndjson`, `behavior-units.normalized.ndjson`, `classifications.ndjson`, `candidates.md`, `draft-report.md`, `lint-report.md`, `findings.ndjson`/`review-summary.md`) instead of the upstream `output/README.md` declaration. |
| C2 | Added the missing contract-bearing Inputs rows: `80-adversarial-review` gained `../_config/evidence-policy.md`, `../_config/icm-ladder.md`, `../30-normalize/output/behavior-units.normalized.ndjson`, `../50-synthesize/output/candidates.md`; `70-lint` gained `../40-classify/output/classifications.ndjson`; `50-synthesize` gained `../_config/icm-ladder.md`; `90-reconcile` gained `../_config/icm-ladder.md`. |
| C3 | Created `.sergeant/index.md` (root catalog, listing `repo-to-icm`). Also added validator check `[S14]` (admitted mode only) so a published workflow missing from the catalog now fails lint instead of passing silently. |
| C4 | Added validator checks `[S11]` (a stage `output/README.md` that declares an artifact must also declare a disposition) and `[S12]` (if any output is declared, the closing stage's `CONTEXT.md` must name a finalize step) — the two D9-lintable checks convention.md names but the validator previously implemented neither. |

## Majors (all fixed)

| ID | Fix |
|---|---|
| C5 / F1 | Created `_config/run-discipline.md` (Layer 3) carrying the operative blindness-rule text, and added it to every stage's own Inputs table (`00` through `90`) — it is no longer parked only in Layer 1, which per convention.md §1a rule 5 the engine never delivers past the first stage. `../CONTEXT.md` (Layer 1) was trimmed to an orientation-level pointer. |
| C6 | `70-lint`'s Purpose no longer claims its own tree "doesn't need to re-verify at run time" (false for a run worktree with fresh NDJSON). Added step 6: run the validator with no argument against this workflow's own tree after all candidates are processed, and record the result — this is what actually exercises `[S9]` against `40-classify/output/classifications.ndjson`. |
| C7 / F13 | Added a "Working directory" section to `70-lint` and `90-reconcile` stating the actual cwd (repository root — confirmed against `WorkSurface::execution_cwd()` in `src/runtime/surface.rs` and `docs/gauntlet/notes/n2-fake-backend-semantics.md`), rewrote both helper invocations to repository-root-relative paths, and added both helpers (`../scripts/validate-structure.py`, `../scripts/finalize.py`) as L3 Inputs rows in the stages that invoke them. |
| C8 | `60-draft`'s Output section now records, explicitly, that the materialized draft packages sit outside the D9 disposition/finalize mechanism entirely — framed as a genuine meta-level grammar-pressure moment for `90-reconcile` to carry forward, not silently accepted as fine. `90-reconcile/references/reconciliation-method.md` §3's meta-level example list now names this moment explicitly. |
| F4 | Same defect as C1 (all ten stages' L4 rows) — resolved by C1's fix. |
| F5 | Same defect as C2 — resolved by C2's fix. |
| F6 | Rewrote `_config/evidence-policy.md`'s hash section: states explicitly the hash covers raw file bytes, never the JSON-escaped `quote` string; gives a capture-once-reuse-twice shell recipe (`QUOTE="$(sed -n 'm,np' file)"`) that eliminates the sed-vs-`printf '%s'` trailing-newline drift. Replaced `extraction-example.md`'s placeholder hash with a real, verified `sha256sum` digest. |
| F7 | Rewrote `reconciliation-method.md` §2: names all ten proposal §9.9 dimensions in-file (so recovering them never requires opening the proposal, which no Inputs table can cite per S6); separates the five internally-computable dimensions (adding the two the old list omitted — behavioral precision, provenance completeness, with concrete computation instructions) from the five requiring comparison, and from supporting run-statistics that are not themselves §9.9 dimensions. |
| F8 | Same defect as C14 — resolved together (see C14 below). |
| F9 | `synthesis-method.md`: heading corrected to "The seven buckets" (matching the seven numbered items); clarified that a `helper`/`stage` record missing its required `workflow` field is a `40-classify`-stage defect to record under the (renamed, generalized) `## Unattached records` heading, not silently dropped; extended the exactly-one-appearance rule from "buckets 1–3" to all seven buckets explicitly. Propagated "six buckets" → "seven buckets" everywhere else it appeared (`50-synthesize/CONTEXT.md`, `output/README.md`). |
| F10 | `reconciliation-method.md` §3 now gives the exact `grammar-pressure.ndjson` wrapper shape: nested `engine_gap` (never flattened, never carrying `representation` — keeping it correctly out of `[S9]`'s scope), worked JSON examples for both `source: behavior` and `source: meta`, and an explicit definition of what `source_evidence` means for a meta-level record (a pointer to where the gap was recorded, not a fabricated `behavior_id`). |
| F11 | `00-contract`'s "Fail closed" section no longer prescribes an unexecutable "stop and ask" (the real Claude backend never emits `NeedsInput`/`Waiting` — confirmed against `src/backend/claude.rs`, which only ever signals `Running`/`StageCompleted`/`Blocked`/`Failed`). Redefined the fail-closed action as writing `contract.md` headed `# AMBIGUOUS — NOT RESOLVED`, with no invented facts, plus a propagation rule in `_config/run-discipline.md` §2 that every downstream stage (all nine, via a "How to do it" step 0) now honors by stopping rather than proceeding on hollow upstream artifacts. |

## Minors

| ID | Disposition | Note |
|---|---|---|
| C9 | Fixed | Same line as F12 — `challenge-checklist.md`'s `../60-draft/output/manifest.md` (or equivalent)` corrected to `../60-draft/output/draft-report.md`, hedge removed. |
| C10 | Fixed | Added validator function `check_at_references` — walks every `.md` file under the package tree (not just each stage's own `CONTEXT.md`), skipping `output/`. Excludes the literal token `@@name` (case-insensitive) as convention.md's own documentation placeholder, resolving the "mention vs. use" ambiguity cheaply and correctly for this corpus. `60-draft/CONTEXT.md` step 5 now states the token literally (`@@name`-style) instead of periphrasis, since the check no longer false-positives on it. |
| C11 | Fixed (byproduct of C1) | L4 rows now name genuine per-run artifacts rather than the authored `output/README.md`; the described mislabeling no longer exists because the described row shape no longer exists. |
| C12 | Fixed | `70-lint/CONTEXT.md` step 5 now instructs explicitly: an `[S7]` hit naming a different package is a repository-wide result, not attributable to the candidate just validated — record it once under a `## Repository-wide` heading, not repeated per candidate. |
| C13 | Fixed (corrected mid-round after measuring the engine — L1 doctrine) | First pass converted `workflow.toml`'s `version = "1"` to a bare TOML integer per record-shapes.md's literal text, then broke engine binding: `src/domain/workflow.rs`'s `WorkflowSection.version` is a Rust `String` with no custom deserializer, so a bare TOML integer fails to parse (measured: `invalid type: integer `1`, expected a string`). Corrected: `workflow.toml`'s `version` stays a quoted, **digits-only** string (never `"v1"`/`"1.0"`) — record-shapes.md's integer *value* requirement, expressed in the type the engine's own loader hard-requires; `index.md`'s `version` (never read by the engine) stays a bare integer, unquoted, per record-shapes.md at face value. `draft-package-template.md` updated to prescribe the corrected (string, digits-only) form for future candidates. Validator check `[S13]` checks both forms' *numeric value* agree, without forcing workflow.toml's TOML type to something the engine rejects. |
| C14 | Fixed | Same defect as F8 — `90-reconcile/CONTEXT.md` step 4 rewritten into an explicit `git add` → `finalize.py --dry-run` → append to `measurement-package.md` → `git add` again → `finalize.py` (real) sequence, so the finalize record lands inside the same closing commit finalize.py makes, not a dirty file after it. |
| C15 | Fixed | Created `/AGENTS.md` at repo root, containing only the small-constitution shape from `docs/icm/convention.md` §3 verbatim (no `.sergeant-conventions` pointer added to `CLAUDE.md`, per instruction — that surface is orchestrator-owned). |
| F12 | Fixed | Same fix as C9. |
| F13 | Fixed | Same fix as C7. |
| F14 | Fixed | `60-draft/CONTEXT.md` step 1 now states explicitly that `.sergeant/drafts/workflows/` (and `.sergeant/drafts/`) may not exist yet in a fresh worktree — that is not an error, and creating it is an ordinary side effect of writing the first candidate package. |
| F15 | Fixed | Added a turn-budget fallback paragraph to `10-inventory/CONTEXT.md` mirroring `20-harvest`'s (partial coverage recorded plainly, not silently rounded up to "done"). Added `10-inventory` to `reconciliation-method.md` §3's meta-level grammar-pressure source list (previously named only `20-harvest` and `70-lint`). |
| F16 | **Declined, with reason** | Every governing document this workflow's stages cite normatively (`docs/icm/convention.md`, `docs/icm/record-shapes.md`, the proposal by §-number) sits outside `.sergeant/workflows/repo-to-icm/`, and `scripts/validate-structure.py`'s `[S6]` check (no path traversal outside the workflow directory or `.sergeant/common/`) makes it structurally impossible to list any of them as a real Inputs-table row — by design, not oversight (the traversal boundary is what makes `[S6]` a meaningful check at all). Resolving this properly means either amending `docs/icm/convention.md` itself to add a new "governing document" citation class exempt from the traversal boundary, or loosening `[S6]`'s boundary to allow a short allowlist of paths (`docs/icm/`, the named proposal file) — both are convention-level policy decisions with ripple effects on every future ICM workflow, not a mechanical per-file fix this round is scoped to make. F7's fix (embedding the ten §9.9 dimension names directly in `reconciliation-method.md` so no stage actually needs to open the proposal) removes the sharpest instance of this pressure without deciding the general policy question. Recording this disposition is itself consistent with the finding's own point: the workflow's citations to ungoverned-by-Inputs-table documents are real, structural, and worth a future milestone's attention, not something to paper over. |

## Verification

- `python3 .sergeant/workflows/repo-to-icm/scripts/validate-structure.py` — exit 0, `PASS: structure is clean`, both before content fixes broke nothing new and after all fixes above (re-run after every edit batch during this round).
- Engine bind re-verified against `target/debug/sgt` per
  `docs/gauntlet/notes/n2-fake-backend-semantics.md`'s method (scratch data
  dir + scratch git repo copy under `scratchpad/`, daemons killed after,
  `pgrep -af "debug/sgt --data-dir" | grep -v "bash -c"` empty at the end)
  — see the parent turn's final report for the run's own transcript
  summary.
