# Extraction pass, 2026-08-17 — ADR 0014 decision 17

Audit trail for the Phase 4 step 2 extraction: identify every ruling in the
proposal corpus that binds present-day behavior of the code or the
operating doctrine and is not already recorded in an existing ADR, before
any of this corpus migrates to `sergeant-rs-workspace`. Authority:
`docs/adr/0014-product-workspace-split-owner-rulings.md` decision 17. This
pass does not move or delete any file — extraction only.

Method: each of the nine named proposal files, plus
`reference/review-northstar-outside-codex.md`, was read in full by an
independent reader (nine files) or directly by the orchestrating session
(the review document and `reference/notes/*.md`), then cross-checked
against every existing ADR (0001–0015) so nothing already recorded gets
re-minted. Candidates were then verified directly against the proposal
text and, where the candidate claimed already-implemented code, against
the actual source tree — not taken on the reader's word alone.

One new ADR came out of this pass (minted as 0016, renumbered to 0017 on integration — the Phase 2b Work independently minted 0016 for the template edition marker from the same base):

- **`docs/adr/0017-gate-work-branch-takeover.md`** — from
  `reference/proposal-foundation-rationalization.md` §8.6, "How a gate Work
  binds to the branch it reviews." Verified as already implemented:
  `crate::runtime::engine::branch_takeover_precondition` and
  `crate::runtime::surface::attach` exist in `src/runtime/engine.rs` and
  `src/runtime/surface.rs`, exercised by tests, and the code comments cite
  "§8.6" as their sole provenance record — a citation that would otherwise
  go stranded once the proposal document moves to the workspace repo. This
  extends ADR 0005 (gating becomes a dispatched Work) with the concrete
  binding mechanism ADR 0005 left as a mechanical gap.

Everything else below was read, considered, and deliberately **not**
extracted. Reasoning is per-file.

## reference/proposal-depot-rust-execution-surface.md

Entirely `status: proposed` design for "Depot," a hypothetical clean-room
successor system that does not exist in this codebase. No sentence reads
as a present-tense binding decision about Sergeant's current code or
doctrine — every normative statement ("Depot should…", "Depot never…") is
a constraint on the *proposed* system, contingent on Depot being built.
Nothing extracted. Pure argument/design record; safe to move as-is.

## reference/proposal-foundation-rationalization.md

One extraction (ADR 0016, above). Everything else in the document is
either explicitly marked "not decided here" / "unruled" / "not decided"
in its own text (§8.3 homepage estate-awareness, §8.4 `validate-and-ship`
re-homing, §8.7 manifest `data_dir` vs. `SGT_DATA_DIR` precedence, §8.5 the
dashboard's phone/second-machine use case), or is argument/evidence
supporting a topic ADR 0005 already covers (§3.1's cost evidence for
gating-as-dispatched-Work). Left as argument record.

## reference/proposal-harness-adapter-research-v2.md

Considered and **not extracted**, at medium confidence: two passages under
"Cerberus already proved two missing invariants" —
(1) protocol-derived semantic features (e.g. `post_turn_summary`) must be
runtime-admitted per version/fixture, never assumed from a prior version or
a stale fixture; (2) every launched turn must cause a later settlement
attempt, even when no client command follows it. Both are framed as
findings that justify the document's own *proposed* v2 adapter contract
(Part VI, explicitly "Proposed," not merged) rather than as freestanding
rulings in force today. They read closely as elaborations of already-
established measure-first doctrine (`LESSONS.md` L1, "measure the Claude
CLI, never trust its docs or its exit codes," cited in ADR 0002's D4) and
of the existing fail-closed-ambiguity posture (ADR 0007, ADR 0009) rather
than new binding facts. Judgment call, recorded for the human reviewer:
if a future pass finds the current adapter code does NOT already enforce
per-version capability admission and settlement-after-every-turn as
concrete behavior, this pair should be revisited as an ADR candidate.
Everything else in the document (Fit scores, the Agent View/SDK sidecar/
Agent teams assessments, the full "Proposed Sergeant adapter contract,
version 2") is explicitly scored recommendation or proposed-not-adopted
design. Not extracted.

## reference/proposal-icm-r-procedure-authority.md

Already fully extracted by `docs/adr/0013-icm-r0-owner-rulings.md`, which
records all twelve owner rulings answering the proposal's own §19 list.
Re-read against ADR 0013 directly (not merely summarized): found nothing
ADR 0013 fails to capture. The ladders themselves (Placement, Bounded-
Judgment) are proposed procedure awaiting rollout through ICM-R1/R2, not
yet binding beyond the names ADR 0013 decision 1 already ratifies. Nothing
further extracted.

## reference/proposal-journal-query-p2.md

Entirely `status: proposed` design for an unbuilt feature (`P2-JOURNAL`).
The document's own text states plainly that it has "zero production route
or CLI behavior if the contract is not accepted." All 55 numbered
"Decision P2-NN" entries are the proposal's own design-and-argument format
for a feature awaiting authorization, not owner rulings already in force.
Nothing extracted.

## reference/proposal-next-iteration-icm-workflows.md

Entirely forward-looking, gated behind not-yet-reached milestones N0–N7;
the document says so explicitly ("later milestones do not retroactively
become the justification for earlier machinery"). Its "Invariants the
Next Iteration Must Preserve" (§4) restate already-existing architecture
(journal as truth, ambiguity fails closed) rather than mint anything new.
Nothing extracted.

## reference/proposal-sgt-watch-v1.md

Describes an unbuilt feature (`sgt watch` subscription/notification layer).
The settled, present-tense doctrine this proposal's argument leans on —
auto-spawn never happens on pure-observation verbs, no exceptions — is
already ADR 0009 (and that ADR's own text traces directly to this
proposal's R-WATCH-3 principle). Two passages were considered above ADR9's
territory (§4/WATCH-11's permanent non-goals for wake/callback/durable-
subscription machinery, and §12.2's "event vocabulary stays domain-only,
never client-observation-derived") and judged not extraction-worthy: both
are scoped to a feature that does not exist yet, and both substantially
overlap already-settled boundaries (ADR 0002's platform-facts scope,
ADR 0006's exec-not-fork-and-supervise). Note for the record: §11.1's
claim that "auto-spawn still applies when `sgt watch` starts" appears to
predate and conflict with ADR 0009's stricter no-exceptions rule — this is
superseded thinking, not a binding ruling, and should not be read as
current when this document lands in the workspace.

## reference/proposal-s-series-stabilization.md

One item was investigated closely and deliberately **not** extracted: §10
"CI policy (owner direction, 2026-08-10)" — the standing rule that the
per-PR gate battery runs once per push (not twice) and that coverage is a
separate, non-blocking `workflow_dispatch`/weekly lane with its
`--fail-under-lines` gate pinned below the measured baseline. This is a
real, dated, owner-directed ruling, and it is already implemented
(`.github/workflows/ci.yml` and `coverage.yml`, verified directly). It is
not extracted to an ADR because it is already fully self-documented at its
point of enforcement — both workflow files carry inline comments stating
the policy and its rationale in full, including a citation back to this
proposal's §10 and to rulings R-S0-3/R-S0-9. Because those YAML files stay
in the product repo (they are not proposal documents and are not part of
this migration), the ruling is not stranded by the proposal's move the way
`branch_takeover_precondition`'s comment would have been — the citation
becomes a pointer to argument record, not to the ruling itself. Judgment
call, flagged for the human reviewer in case a stricter reading is wanted.

Several narrower items were considered and rejected as too narrow/already-
settled-elsewhere to warrant an ADR: `rust-toolchain.toml` staying on
`channel = "stable"` (verified current — matches the file as it stands,
reads as description of existing fact, not a new ruling); "no HTTP
shutdown endpoint, SIGTERM remains the sole graceful-stop path" (the
proposal's own text marks this "R1-rejected," i.e. already settled by a
prior ruling this file only restates); and the two named dead-code
exemptions (`SnapshotBeyondJournal` load path, `Analytics::table_rows`) —
narrow, symbol-level, code-review-convention detail, better suited to a
code comment or `LESSONS.md` entry than an ADR.

## reference/proposal-tui-t-series.md

The one clear owner ruling in this document (T2-14, dated 2026-08-16: TUI
reaches repo/group/Doctor only through new authenticated daemon API
routes, CLI keeps calling manifest/doctor functions directly) is already
`docs/adr/0012-estate-and-doctor-are-daemon-api-surface.md`. Re-read
directly: nothing else in the document rises even to low confidence as a
present-binding ruling — the T0–T4 cockpit it describes is explicitly
merged only to a feature-integration branch, not `main` (line 63), so
almost everything is forward-looking design for unmerged work. The
`ratatui-textarea` dependency-admission spike (§8.7) was considered: a
real, already-executed compatibility spike with recorded evidence, but the
spike was explicitly read-only (no edit to `Cargo.toml`/`Cargo.lock`) and
the feature that would consume it hasn't merged — it binds nothing about
present code yet. Not extracted.

## reference/review-northstar-outside-codex.md

An independent-reviewer critique document, not a proposal. Read in full:
every substantive claim is either an endorsement of a ruling already made
elsewhere (not originating in this document) or an explicit call for a
not-yet-made ruling ("this needs a design ruling before…", "has to be
settled", "is not fully settled"). Nothing in it is phrased as a settled,
binding decision in its own right. Nothing extracted.

## reference/notes/*.md

Read directly (not delegated): `gauntlet-pattern.md`, `ideaos-agent-
contract.md`, `fable5-techniques.md`. These describe the currently-
operating gauntlet development method (contract → build → gates → critic
panel → adjudicate; model assignment by earned need; Ponytail Minimality
Ladder; probe hygiene) — process doctrine for how *this repository's own
development work* is run, not a claim about the shipped product's
architecture or runtime behavior. ADR scope per `docs/adr/README.md` is
decisions that "fix an architectural shape once and are expensive to
re-litigate casually" for the product; the gauntlet method is explicitly
revised in place repeatedly (each file records its own dated revisions:
"economy revision 2026-08-08," "revision 2026-08-11," etc.), which is the
opposite of the append-only, rarely-revisited shape ADRs are for. These
notes are themselves already the durable record of that process doctrine
and are not proposal documents scheduled to move — confirmed they are not
among the nine files this pass was scoped to extract from. Nothing
extracted; flagged here only because the task asked these be read for
context.

## Summary

- New ADRs minted: **1** (`0017-gate-work-branch-takeover.md`).
- Borderline items deliberately left as summary-only, not ADRs (see above):
  the two Cerberus-derived adapter invariants in
  `proposal-harness-adapter-research-v2.md`, and the CI trigger-dedup /
  non-blocking-coverage-lane policy in `proposal-s-series-stabilization.md`
  §10. Both are real rulings; both were judged not to need a dedicated ADR
  because their record survives the move independent of the proposal
  document (existing ADR territory in the first case, self-documenting
  tracked config in the second) — flagged explicitly in case a future
  reviewer disagrees.
- No files were moved or deleted. This is extraction only; migration is a
  later step.
