# Library re-homing record — 2026-08-12

MVP-5 CONTENT, Lane F2. Executes the execution-surface re-triage's verdicts
(`docs/icm/retriage-2026-08-11.md`, both passes) against the promoted
`.sergeant/workflows/` library, per the North Star's owner ruling that a
package failing the §2a workflow test is re-homed to the surface it
actually belongs to, never merely deleted
(`docs/icm/convention.md` §2a). Provenance for every git-removed package is
preserved in this repository's git history — `git log --diff-filter=D --
.sergeant/workflows/<name>` finds the last commit that carried it.

Before: 35 published workflows. After: 23 (`.sergeant/index.md`) + 4
operator skills (`skills/`). The 21 packages the first re-triage pass rated
plain WORKFLOW all stand untouched; `load-project` and `sergeant-setup`
stand with their CLI-shaped slices removed (SPLIT verdicts); the other 12
retired in full.

## Full retirements (`git rm`)

| Package | Verdict | Where the content went |
|---|---|---|
| `respond-to-worker` | CLI-SURFACE, ABSORBED (retriage pass 1 + absorbed-sweep) — "collides with shipped `sgt respond` (`src/cli.rs:89`)"; `00-precondition-check`'s only judgment is idempotency filtering, `40-apply-and-acknowledge` applies an already-made human decision | Nowhere new — the shipped `sgt respond` / `POST /v1/work/{id}/input` already is this |
| `monitor-fleet` | CLI-SURFACE, PARTIAL→mostly ABSORBED — "the judgment this stage performs is interpreting... and reporting it, never acting on it"; read/report half absorbed into `sgt status` + `sgt work list` | Documented as absorbed here; the residual "verified busy witness"/stalled-worker diagnostic gap is `recover-stalled-worker`'s job, which already STANDS as a published workflow (`docs/icm/retriage-2026-08-11.md`'s absorbed-sweep: "`recovery.rs` only covers restart-time reconciliation, not live-turn staleness detection while the daemon stays up") |
| `project-graph` | CLI-SURFACE, NET-NEW-SURFACE — no whole-project multi-repo graph exists; distinct object from `sgt work show --graph`/`/v1/graph/work/{id}` (naming reconciled, not collided) | `sgt project graph` verb candidate, roadmap item 8 (`docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`) — unblocks on the estate landing |
| `reconcile-and-cleanup-fleet` | CLI-SURFACE, PARTIAL — per-repo surface teardown ABSORBED (`recovery.rs`'s automatic reconciliation); multi-repo "fleet task" grouping/handshake-ack cleanup is NET-NEW, no such domain object exists | `sgt fleet cleanup` verb candidate if the fleet-grouping object is ever ruled in (currently NOT-EVER per North Star's "fleet as a domain object" line) |
| `wake-and-resume` | CLI-SURFACE, NET-NEW (engine-primitive) — no periodic/processless re-evaluation scheduler exists; engine-gap **G1** | Post-MVP roadmap item 4 ("G1 scheduler (on a policy)"), `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md` |
| `deliver-external-callback` | CLI-SURFACE, NET-NEW-SURFACE — no callback/notification mechanism on the shelf; engine-gap **G3**, narrowed to an ack-gate | Post-MVP roadmap item 4 ("G3 callbacks (on a consumer)") |
| `drain-fleet` | CLI-SURFACE, NET-NEW-SURFACE — no admission-block primitive exists; engine-gap **G4** | `sgt daemon stop` (MVP-3, already shipped, "cheap-now" scoped narrower than this package per CUT 10's one-owner objection to multi-actor drain) covers the one-owner case; the broader `sgt fleet drain`/`force-stop` stays an open engine-gap |
| `route-review-findings` | CLI-SURFACE, NET-NEW-SURFACE — no "gate" or finding-routing concept anywhere in the engine | `sgt review route-findings`/`sgt gate clear` verb candidates, unbuilt (DELTA #5, `docs/gauntlet/notes/upstream-core-function-map-2026-08-11.md`) |
| `wiki-digest` | CLI-SURFACE, NET-NEW, questionable fit — "object is external wiki state, not sergeant's own — recommend parking" | Parked, not re-homed anywhere — explicitly out of scope for `sgt` |
| `sergeant-help` | OPERATOR-SKILL — "exactly 'instructions that teach the interactive harness how to operate sergeant well' (§2a bucket 3) — no worktree, no durable Work state needed for a doc lookup" | `skills/sergeant-help/SKILL.md`, content re-verified against the real built CLI (`sgt --help`, 2026-08-12), not copied blind |
| `grilling` | WORKFLOW-IF-E3 at retriage, **dissolved by North Star ruling R-NS-6** ("execution ≠ dialogue" — conversation is the harness's job, never engine work); dogfood measured 2/2 runs completing autonomously with zero `needs_input`, "negative value vs plain terminal Claude" | `skills/grilling/SKILL.md` |
| `grill-with-docs` | Same R-NS-6 dissolution — both stages delegated to `grilling`, identical E3 exposure | `skills/grill-with-docs/SKILL.md` |

## SPLIT verdicts executed (workflow core stays, CLI slice retires)

| Package | What retired | Where it went | What stays |
|---|---|---|---|
| `load-project` | The `20-register-or-edit` stage's folded "Sync repositories"/"Report state" helpers — upstream's `list-projects`, `project-status`, `project-sync`, `project-task-list` (retriage: "command surfaces, not procedures with a bounded outcome (§6.2)") | `sgt repo list` + `sgt doctor` (status/listing); `sgt repo add <name> --origin <url>` (clone-if-missing half of sync); the "pull existing repos" gap is named honestly in `skills/estate-navigation/SKILL.md`, not silently invented | `00-resolve-project-name`, `10-resolve-context`, and `20-register-or-edit`'s own register/edit judgment — version bumped 2→3 |
| `sergeant-setup` | `00-detect-prerequisites`, `10-install-commands`, `20-global-config`, `40-repair-existing`, `60-task-tracking-init`, `70-optional-capabilities` — "mechanical bootstrap/repair machinery... collide with existing `sgt doctor`"; the bootstrap flow itself now literally exists as shipped `sgt init` (MVP-3), which retriage's own writing (2026-08-11) predates | `sgt init` (scaffold) + `sgt doctor` (detection/health, every failing check names a remedy) | `05-file-capability-gaps` (drafts a tracked issue per unsupported capability, real judgment, not named CLI-SURFACE by retriage) and `30-project-interview` (project-definition interview — retriage flags it as duplicating `load-project`'s registration job, "a duplication defect, not a clean stage-boundary split"; **not fixed by this pass**, only noted, since fixing it means redesigning `30-project-interview` to delegate rather than reimplement — out of this re-homing pass's scope) — version bumped 2→3 |

## Consequential edits (not retirements, but caused by them)

- `.sergeant/workflows/wayfinder/00-name-destination/CONTEXT.md` and
  `.sergeant/workflows/triage/40-grill-if-underspecified/CONTEXT.md` both
  delegated to the now-retired `grilling` workflow package. Both Delegation
  sections now point at `skills/grilling/SKILL.md` (loaded and run live in
  the current session, never dispatched), and both note R-NS-6 resolves the
  E3 dependency they'd inherited from `grilling`'s prior WORKFLOW-IF-E3
  classification.
- `.sergeant/index.md`, `AGENTS.md` (and its `CLAUDE.md` symlink), and
  `README.md` all cited "35 published workflows" — corrected to 23, with
  the operator-skills layer named alongside. `AGENTS.md`'s routing table
  gained rows for `sergeant-help`, `estate-navigation`, `grilling`, and
  `grill-with-docs` at their new `skills/` paths, replacing the stale
  `.sergeant/workflows/sergeant-help/` path and the now-inaccurate
  "published workflows" framing of the grilling caveat.

## Not part of this record

Two unrelated defects fixed in passing per this lane's charter (separate
commits, not library re-homing): #53 (`partition-checkpoint-protocol.md`'s
wrong retry verb) and #57 (`validate-and-ship`'s S4 Inputs-table layer
defect, caught by re-running the §9.7 validator `--admitted` over the final
library per this record's own closing step).
