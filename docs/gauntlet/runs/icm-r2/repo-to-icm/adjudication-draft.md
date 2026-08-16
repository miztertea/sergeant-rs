# Package adjudication: repo-to-icm

Producer draft, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md`
decisions 8–9). Produced against
`reference/proposal-icm-r-procedure-authority.md` §8 (Library
Reconciliation Method) and `docs/icm/record-shapes.md` §6 (package-
adjudication record shape). This is a producer-only draft — self-checked,
not independently reviewed; an independent reviewer position performs that
review separately (`docs/icm/convention.md` §6.2–6.3, ADR decision 6).

Every file under `.sergeant/workflows/repo-to-icm/` was read in full for
this adjudication: `CONTEXT.md`, `index.md`, `workflow.toml`, all three
`_config/` files, all eleven stage `CONTEXT.md` files (`00-contract`
through `90-reconcile`, including the `kind = "execute"` `65-self-check`),
every stage `references/*.md` and `output/README.md`, and all three
`scripts/*.py` files. `git log` and `git log -p` were checked for
`_config/icm-ladder.md` and for the workflow as a whole, including the
ICM-R1 landing commit (`dd3c0ef`).

## Original intention

`repo-to-icm` (proposal §9.1) exists to convert a repository's distributed
procedural knowledge — skills, agent instructions, scripts, docs, tests —
into **draft** ICM workflow packages plus an evidence-backed report, so
that a repository's actual procedural surface can be discovered,
source-cited, and reconciled against the ICM decomposition ladder without
any generated content ever becoming runnable procedure by construction. It
was admitted under `docs/gauntlet/contracts/N2.md` and has since been
amended twice under measured evidence: v2 (`docs/gauntlet/runs/n2-run2/`)
added the `20-harvest` partition-checkpoint protocol, the mandatory
consequence-class sweep, the `40-classify` §6.3-before-`helper` rule and
over-promotion self-check, `80-adversarial-review`'s fourth axis, and
`finalize.py`'s evidence-preservation guard; v3 (`docs/gauntlet/contracts/
N4.md`) added `65-self-check` as the first real `kind = "execute"` stage.
It is explicitly reflexive for this pilot: it is the same reconciliation
method (§8) this ICM-R2 producer pass is itself applying, one layer up,
against `repo-to-icm`'s own package.

## Current trigger and outcome

**Trigger:** an already-admitted Work whose task names a subject repository
(or subtree) to decompose, either to seed a first ICM decomposition of a
new repository or to measure this workflow's own recall/precision against
an already-adjudicated reference corpus. `repo-to-icm` never decides
whether this generation should happen — per `_config/icm-ladder.md` §6.1a
(added ICM-R1, see Finding A below on its citation), that discriminator is
exactly what separates a workflow from a Captain skill, and this package
receives an admitted intent rather than producing one.

**Bounded outcome:** every workflow candidate `50-synthesize` names is
materialized as a complete draft package under
`.sergeant/drafts/workflows/<candidate>/`, mechanically self-checked
(`65-self-check`) and lint-repaired (`70-lint`), independently challenged
(`80-adversarial-review`), adjudicated and measured
(`90-reconcile`), with the run's own `output/` artifacts finalized per the
D9 disposition policy. The run never writes to `.sergeant/workflows/` and
never edits the engine.

## Driver and admission boundary

**Driver:** ten actor stages (fresh execution per stage, no shared
conversation state, Layer-4 named-artifact handoff only) plus one
deterministic `kind = "execute"` stage (`65-self-check`, a pinned container
sergeant invokes directly with no model in the loop, N4 §11.2). All eleven
stages are driven by `sergeant` itself once the Work is admitted — none of
them converse with a user or decide what Work should exist; that
discriminator is checked explicitly, in order, before any lower ladder
rung, by `_config/icm-ladder.md §6.1a`.

**Admission boundary:** in-work, always. `00-contract` is the only stage
that reads anything outside this workflow's own artifact chain (the Work's
initiating task and the worktree); every later stage's contract is fully
determined by named upstream Layer-4 artifacts. There is no pre-work or
post-work behavior — the workflow's authority begins at `00-contract` and
ends at `90-reconcile`'s `finalize.py` invocation.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-R2C-01 | `CONTEXT.md` — workflow converts repo procedural knowledge into draft ICM packages + report; never publishes; never edits engine | PL-4 | J4 (Work intent fixes subject/scope); J5 (never publish/edit engine — `docs/icm/convention.md` §2 rule 4) | STAND | `draft/CONTEXT.md` |
| BU-R2C-02 | `00-contract/CONTEXT.md` — resolves subject/revision/scope/exclusions/success-criteria; fails closed on ambiguity | PL-5 | J2 (revision/scope resolution); J0 (ambiguity → `# AMBIGUOUS — NOT RESOLVED`, this engine's substitute for a `needs_input` hold) | STAND | `draft/00-contract/CONTEXT.md` |
| BU-R2C-03 | `10-inventory/CONTEXT.md` — deterministic four-way disposition of every in-scope file, grouped into partitions | PL-5 | J2 (disposition assignment, partitioning); J1 (partition naming) | STAND | `draft/10-inventory/CONTEXT.md` |
| BU-R2C-04 | `20-harvest/CONTEXT.md` + `references/partition-checkpoint-protocol.md` — extracts source-cited units per partition, with checkpoint/retry and mandatory 5-class sweep | PL-5 | J2 (unit extraction, splitting, sweep); J0-equivalent (partition boundary stop, no mid-turn ask primitive) | STAND | `draft/20-harvest/CONTEXT.md` |
| BU-R2C-05 | `30-normalize/CONTEXT.md` — rewrites units implementation-independent, evidence fields untouched | PL-5 | J5 (`source.*` fields immutable — inherited, not this stage's to reopen); J2 (rewrite/split judgment) | STAND | `draft/30-normalize/CONTEXT.md` |
| BU-R2C-06 | `40-classify/CONTEXT.md` — applies `_config/icm-ladder.md` in strict order, §6.3-before-`helper` rule, over-promotion self-check | PL-5 | J5 (ladder order is fixed, not reopenable); J2 (rung selection, rationale) | STAND | `draft/40-classify/CONTEXT.md` |
| BU-R2C-07 | `50-synthesize/CONTEXT.md` + `references/synthesis-method.md` — clusters classification records into 7 buckets by behavioral contract, not source file | PL-5 | J5 (cluster-by-contract-not-file — `docs/icm/record-shapes.md` §6 rule 4); J2 (naming, ordering) | STAND | `draft/50-synthesize/CONTEXT.md` |
| BU-R2C-08 | `60-draft/CONTEXT.md` — materializes draft packages under `.sergeant/drafts/workflows/`, never `.sergeant/workflows/` | PL-5 | J5 (draft-only boundary, `docs/icm/convention.md` §2 rule 4); J2 (collision rename, provenance-inference marking) | STAND | `draft/60-draft/CONTEXT.md` |
| BU-R2C-09 | `65-self-check/CONTEXT.md` + `workflow.toml` `[stage."65-self-check"]` — pinned container runs the validator against this workflow's own tree; exit code alone decides the stage outcome (N4 §11.2) | PL-6 (execute-stage-implemented deterministic mechanism, proposal §5.8) | no J-ladder — no actor turn exists to exercise judgment; only two mechanical outcomes | STAND | `draft/65-self-check/CONTEXT.md` |
| BU-R2C-10 | `70-lint/CONTEXT.md` + `references/mechanical-vs-substantive.md` — validates candidates and this workflow's own tree, repairs mechanical defects, logs substantive ones | PL-5 | J2 (mechanical/substantive classification, repair); J5 (never force-fix a substantive defect) | STAND | `draft/70-lint/CONTEXT.md` |
| BU-R2C-11 | `80-adversarial-review/CONTEXT.md` + `references/challenge-checklist.md` — fresh-execution, four-axis independent challenge, no edit authority | PL-5 | J5 (no edit authority, `docs/icm/convention.md` §4.9/§6.3 review independence); J2 (severity judgment) | STAND | `draft/80-adversarial-review/CONTEXT.md` |
| BU-R2C-12 | `90-reconcile/CONTEXT.md` + `references/reconciliation-method.md` — adjudicates findings, assembles measurement package, consolidates grammar pressure, runs `finalize.py` | PL-5 | J5 (the one point content may be edited, scoped to accepted repairs only); J2 (accept/reject/park) | STAND | `draft/90-reconcile/CONTEXT.md` |
| BU-R2C-13 | `_config/run-discipline.md` §1 — blindness rule: every stage's actor is blind to `reference-corpus/` for a measurement run | PL-3 (workflow-local shared method) | J5 (governing constraint, cited by every stage's own `## Bounded judgment`) | STAND | `.sergeant/workflows/repo-to-icm/_config/run-discipline.md` (unchanged) |
| BU-R2C-14 | `_config/run-discipline.md` §2 — `# AMBIGUOUS — NOT RESOLVED` fail-closed propagation, the workaround for the missing actor-initiated mid-turn ask primitive | PL-3 | J0-equivalent by design (this *is* the workflow's own J0 mechanism given the platform's actual grammar) | STAND | `.sergeant/workflows/repo-to-icm/_config/run-discipline.md` (unchanged) |
| BU-R2C-15 | `_config/evidence-policy.md` — quote+hash citation discipline, one-behavior-per-unit rule, mechanism/intent separation | PL-3 | J5 (evidence fields immutable once minted) | STAND | `.sergeant/workflows/repo-to-icm/_config/evidence-policy.md` (unchanged) |
| BU-R2C-16 | `_config/icm-ladder.md` — the §6.1–6.7 decomposition ladder, distilled for `40-classify`, including §6.1a's driver/admission-boundary discriminator | PL-3 (one of the ladder's two named canonical homes, proposal §7.1) | J5 (rung order fixed) | **FOLD** (citation correction — see Finding A) | `draft/_config/icm-ladder.md` |
| BU-R2C-17 | `scripts/validate-structure.py` — deterministic structural validator, dual-mode (admitted tree / draft candidate path) | PL-6 (workflow-local helper) | none — mechanical, actor reviews its structured output | STAND | `.sergeant/workflows/repo-to-icm/scripts/validate-structure.py` (unchanged) |
| BU-R2C-18 | `scripts/finalize.py` — deterministic D9 disposition finalize wrapper, forwards to `.sergeant/lib/finalize.py`, evidence-preservation guard (GP-5b) | PL-6 | none — deterministic; judgment about what should exist stays with the writing stage | STAND, with a flagged observation (see "Alternatives considered") | `.sergeant/workflows/repo-to-icm/scripts/finalize.py` (unchanged) |
| BU-R2C-19 | `scripts/test-finalize-evidence-guard.py` — standalone sandbox proof of the evidence-preservation guard, run by human/CI or `[S15]`, not by any stage | PL-6 | none | STAND | `.sergeant/workflows/repo-to-icm/scripts/test-finalize-evidence-guard.py` (unchanged) |
| BU-R2C-20 | Missing `## Authority envelope` on `CONTEXT.md` (Layer 1) — required by `docs/icm/convention.md` §6.1 / ADR decision 4 / proposal §7.2; confirmed absent by direct grep of every `## ` heading in the file | n/a (a required-section defect, not a new behavior to place) | J5 (the requirement itself is a governing constraint on every workflow package, not this package's to waive) | **FOLD** | `draft/CONTEXT.md` (Authority envelope section added) |
| BU-R2C-21a | Missing `## Bounded judgment` on `00-contract/CONTEXT.md` | n/a | J5 (ADR decision 4: required always, omission never ambiguous) | **FOLD** | `draft/00-contract/CONTEXT.md` |
| BU-R2C-21b | Missing `## Bounded judgment` on `10-inventory/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/10-inventory/CONTEXT.md` |
| BU-R2C-21c | Missing `## Bounded judgment` on `20-harvest/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/20-harvest/CONTEXT.md` |
| BU-R2C-21d | Missing `## Bounded judgment` on `30-normalize/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/30-normalize/CONTEXT.md` |
| BU-R2C-21e | Missing `## Bounded judgment` on `40-classify/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/40-classify/CONTEXT.md` |
| BU-R2C-21f | Missing `## Bounded judgment` on `50-synthesize/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/50-synthesize/CONTEXT.md` |
| BU-R2C-21g | Missing `## Bounded judgment` on `60-draft/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/60-draft/CONTEXT.md` |
| BU-R2C-21h | Missing execute-stage-adapted `## Bounded judgment` on `65-self-check/CONTEXT.md` (proposal §7.3's carve-out: an execute stage states which outcomes are mechanical and which block, even with no actor judgment) | n/a | J5 | **FOLD** | `draft/65-self-check/CONTEXT.md` |
| BU-R2C-21i | Missing `## Bounded judgment` on `70-lint/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/70-lint/CONTEXT.md` |
| BU-R2C-21j | Missing `## Bounded judgment` on `80-adversarial-review/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/80-adversarial-review/CONTEXT.md` |
| BU-R2C-21k | Missing `## Bounded judgment` on `90-reconcile/CONTEXT.md` | n/a | J5 | **FOLD** | `draft/90-reconcile/CONTEXT.md` |
| BU-R2C-22 | `_config/icm-ladder.md` §6.1a mis-attributes its own grounding to ADR 0013 "decision 1" (Names — accepting the PL/J terms), when the actual grounding is ICM-R0 gauntlet Finding ICMR-F3 (proposal §3.3); none of the ADR's twelve owner-ruled decisions covers the driver/admission-boundary discriminator | n/a (citation-accuracy defect, same file as BU-R2C-16) | J5 (citation accuracy is a governing requirement for any doctrine-citing content, not this package's judgment call) | **FOLD** | `draft/_config/icm-ladder.md` |
| BU-R2C-23 | `scripts/finalize.py` forwards to `.sergeant/lib/finalize.py`, a shared-helper location `docs/icm/convention.md` §1 does not name (only `.sergeant/common/{contexts,scripts,templates}`) | n/a (flagged observation, not this package's own behavior to fix — `.sergeant/lib/` is not owned by `repo-to-icm`) | — | STAND (flagged, out of this package's authority — see "Alternatives considered") | not applicable; evidence for a future shared-infrastructure review |

## Surviving package design

`repo-to-icm` survives as a single PL-4 workflow with ten actor stages and
one `kind = "execute"` stage, unchanged in shape, stage order, engine
contract, and `_config`/`scripts` content. The three amendments below are
the entire proposed change surface — every other file in the package
stands exactly as authored.

**1. `CONTEXT.md` gains a real `## Authority envelope` section** (proposal
§7.2 shape), placed after "What this workflow does" and before "The
blindness rule." It states: the workflow may decide subject-revision
resolution within worktree evidence, every behavior unit's placement and
disposition (subject to independent review), mechanical draft repairs, and
`80-adversarial-review` finding adjudication; it may not decide promotion,
`AGENTS.md` edits, or engine-gap authorization; the human/Captain gates are
promotion, resolving an `AMBIGUOUS` run, resolving an incomplete partition
ledger, and acting on a surviving engine-gap claim; and the decision record
is distributed across each stage's own declared `output/` artifact, not a
single log file. Full text: `draft/CONTEXT.md`.

**2. Every one of the eleven stage `CONTEXT.md` files gains a real
`## Bounded judgment` section** (proposal §7.3 shape for the ten actor
stages; the §7.3 execute-stage carve-out for `65-self-check`), each
grounded in that stage's own already-existing contract rather than generic
language — the J2 delegations, J1 local choices, and J0/`needs_input`
(or this workflow's actual `# AMBIGUOUS — NOT RESOLVED` substitute for it)
cases were derived directly from each stage's "How to do it" and "What must
become true here" sections already read for this adjudication, not
invented. `65-self-check` gets the adapted form the proposal specifies for
an execute stage: no J-ladder, because there is no actor turn, but an
explicit statement of which two outcomes are mechanical (exit 0/1) and
that no ambiguous condition exists for it to block on. Full text: eleven
files under `draft/<stage>/CONTEXT.md`.

**3. `_config/icm-ladder.md` §6.1a's citation is corrected** from ADR 0013
"decision 1" (which is "Names," unrelated) to ICM-R0 gauntlet Finding
ICMR-F3 (`reference/proposal-icm-r-procedure-authority.md` §3.3) — the
section's own substantive content (the driver/admission-boundary
discriminator itself) is unchanged and correct; only the attribution is
wrong, and only in the ICM-R1 landing commit's added prose, not in
anything authored before it. Full text: `draft/_config/icm-ladder.md`.

No stage boundary, `_config` policy, script, or output declaration is
disturbed. `workflow.toml`, `index.md`, and every `references/*.md` and
`output/README.md` file were read in full and found to need no change.

## Inputs and outputs

Inputs: an admitted Work naming a subject repository (path or subtree) and,
optionally, an explicit revision/scope/exclusions/success-criteria. Outputs
(this workflow's own declared per-run artifacts, `promote`d unless noted):
`00-contract/output/contract.md`; `10-inventory/output/inventory.md`;
`20-harvest/output/{behavior-units.ndjson, partition-ledger.md,
consequence-class-sweep.md}`; `30-normalize/output/
behavior-units.normalized.ndjson`; `40-classify/output/
classifications.ndjson`; `50-synthesize/output/candidates.md`;
`60-draft/output/draft-report.md` (plus the materialized draft packages
themselves, outside this workflow's own `output/` tree, under
`.sergeant/drafts/workflows/`); `65-self-check/output/
self-check-result.txt` (`evidence`, not `promote`); `70-lint/output/
lint-report.md`; `80-adversarial-review/output/{findings.ndjson,
review-summary.md}`; `90-reconcile/output/{adjudication-log.md,
measurement-package.md, grammar-pressure.ndjson}`. None of this changes
under the amendments above — they add sections to `CONTEXT.md` files, not
new artifacts.

## Review and promotion policy

**Artifact class:** the amended `CONTEXT.md` files and `_config/
icm-ladder.md` are workflow-authoring content (Layer 1/2/3 package
material), not per-run generated output. **Draft location:** `docs/
gauntlet/runs/icm-r2/repo-to-icm/draft/`, mirroring the live package's own
relative paths, per this pilot's own draft-publication boundary — nothing
under `.sergeant/workflows/repo-to-icm/` is touched by this producer pass.
**Independent review:** required before promotion (ADR decision 6: this is
promotable content — it becomes the actual `.sergeant/workflows/
repo-to-icm/` package content once accepted) — a fresh reviewer position
with no edit authority, distinct from this producer execution (ADR decision
7 / `docs/icm/convention.md` §6.3). **Acceptance criteria:** every claimed
missing-section defect verified against the live package (not merely
trusted from this record); every added `## Bounded judgment`/`##
Authority envelope` section checked against the stage's own actual
contract for accuracy, not just presence; the icm-ladder.md citation
correction checked against the ADR and proposal text directly. **Promotion
action:** copy the accepted `draft/` files over their corresponding live
paths under `.sergeant/workflows/repo-to-icm/`, in a human-reviewed change,
same as any other ICM-R2 pilot package. **Failure/remediation path:** a
rejected section is reworked by a subsequent producer pass against the
reviewer's specific finding, not silently dropped — the package remains
compliant-but-imperfect (STAND, unamended) rather than reverting to
non-compliant (no section at all) in the interim.

## Alternatives considered

- **Leave the missing Authority-envelope/Bounded-judgment sections
  unaddressed and adjudicate `repo-to-icm` as a clean STAND.** Rejected:
  the task's own wrinkle explicitly asked this to be verified, not assumed,
  and direct inspection (grepping every stage `CONTEXT.md`'s `## ` headings)
  confirmed the sections are absent from all eleven stages and the
  workflow-level file — a real, checkable compliance gap against ADR
  decision 4 and `docs/icm/convention.md` §6.1, not a hypothetical one.
  Silently calling this a clean STAND would itself repeat the "smoothed
  over" failure mode this pilot exists to catch.
- **Classify the missing-sections defect as HARVEST or REHOME instead of
  FOLD.** Rejected: nothing about the package's identity, ownership
  boundary, or stage structure is wrong — the defect is an incomplete
  instance of required content inside an otherwise-correct surviving
  package, which is exactly what FOLD names ("unit becomes context or a
  helper inside an owning package," proposal §5.10) rather than a move
  (REHOME), a split (SPLIT), or an extraction into a different owner
  (HARVEST).
- **Also extend `scripts/validate-structure.py` to check for `##
  Authority envelope`/`## Bounded judgment` presence, since the S-checks
  currently do not (confirmed by grep: no match for either string in the
  validator).** Considered, and rejected for this pass specifically: the
  task's own scope is "give repo-to-icm itself real Authority-envelope/
  Bounded-judgment sections," not extending the structural validator's own
  check surface — that is a second, independently reviewable change with
  its own design questions (what counts as "real" versus boilerplate
  content, mechanically) and belongs to a later pass once the section
  shape itself has been through independent review here first. Recorded as
  a known gap for that later pass, not silently assumed handled.
- **Treat `.sergeant/lib/finalize.py`'s location (outside `.sergeant/
  common/`) as this package's own defect to fix (BU-R2C-23).** Rejected:
  `repo-to-icm`'s own `scripts/finalize.py` is a thin, correctly-placed
  workflow-local wrapper (`docs/icm/convention.md` §5 rule 3) around a
  helper it does not own; `.sergeant/lib/` as a location is evidence for a
  future universal-scope review of shared infrastructure (ADR decision 3),
  not something this package's own adjudication has the authority or the
  full context to relocate.
- **Fix the icm-ladder.md §6.1a mis-citation by simply deleting the
  attribution rather than correcting it.** Rejected: the section's
  grounding is real and traceable (ICM-R0 Finding ICMR-F3) — removing the
  citation would lose that traceability instead of repairing it, and this
  file is one of only two canonical homes the proposal names for the
  placement ladder (§7.1), so its citations carry more weight than an
  ordinary reference file's would.

## Final disposition
STAND

The package's identity, stage structure, driver, and every `_config`/
`scripts`/`references` content item are correct as authored and require no
change. Three FOLD-class amendments (an Authority-envelope section, eleven
Bounded-judgment sections, and one citation correction) complete this
package's compliance with `docs/icm/convention.md` §6.1 and ADR decision 4
— content additions and one correction folded into the existing package,
not a change to what the package is or where it lives. This is not a
"clean" STAND (some files do change), which is why a reviewable `draft/`
tree accompanies this record rather than being omitted.

## Validation evidence

- Every file under `.sergeant/workflows/repo-to-icm/` was read directly
  (not sampled) for this adjudication — confirmed by the file-by-file
  citations throughout this record.
- `git log --oneline -- .sergeant/workflows/repo-to-icm/_config/
  icm-ladder.md` and `git log -p -1` against the ICM-R1 landing commit
  (`dd3c0ef`) were run directly to isolate exactly what that commit added
  to `_config/icm-ladder.md` (the new §6.1a section) versus what predates
  it — the diff, not a summary of the commit message, is what BU-R2C-22's
  finding is based on.
- The "decision 1" mis-citation was checked against three independent
  sources that must agree and did not: `docs/adr/0013-icm-r0-owner-
  rulings.md` decision 1's own text ("Names..."),
  `reference/proposal-icm-r-procedure-authority.md` §19 item 1 (same text,
  the ADR's own source list), and §3.3/Finding ICMR-F3 (the actual
  discriminator content) — decision 1 and Finding ICMR-F3 are unrelated
  provisions in the same document, confirmed by direct reading, not
  inferred from proximity.
- The Authority-envelope/Bounded-judgment absence was checked
  mechanically, not from a first impression: `grep -n "^## "` against every
  one of the eleven stage `CONTEXT.md` files and the workflow-level
  `CONTEXT.md` was run directly; the full heading list for every file is
  reproduced in this producer's own working notes and confirms zero
  matches for either required section string anywhere in the package.
- `scripts/validate-structure.py` was checked directly (`grep -n
  "Authority envelope\|Bounded judgment"`) and confirmed to contain no
  check for either section today — cited above as a known, out-of-scope
  gap (Alternatives considered), not silently assumed already covered.
- This is a producer self-check only (proposal §8.10): every behavior unit
  above is dispositioned, every citation resolves to a real file/line this
  producer actually opened, every FOLD rationale states why (not a
  restatement of the behavior), and no package assumes hidden conversation
  continuity between this record and the independent review that follows
  it. It is not promotion authority (`docs/icm/convention.md` §6.2) — an
  independent reviewer position, not this producer, adjudicates whether
  these three amendments are accepted.
