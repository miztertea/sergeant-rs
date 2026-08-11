# Partition ledger — run 01KZQ32J2BAD4P8WJA9SWXRMZ9

Per `references/partition-checkpoint-protocol.md`. Seeded from
`../10-inventory/output/inventory.md`'s 21 named partitions (decompose
files only), in that file's recorded order. `Status` is exactly `done` or
`pending`, never a third state.

| Partition | Status | Unit id range | Notes |
|---|---|---|---|
| P1: Root agent policy | done | BU-0001–BU-0060 | AGENTS.md only; dense directive-per-bullet file, high unit count. |
| P2: Product overview, documentation index & help | done | BU-0061–BU-0128 | 6 files: README.md, docs/README.md, docs/what-is-sergeant.md, docs/skills.md, docs/repo-scoped-skills.md, skills/sergeant-help/SKILL.md. |
| P3: Installation, usage, troubleshooting & config schema | done | BU-0129–BU-0213 | 6 files: docs/getting-started.md, docs/using-sergeant.md, docs/troubleshooting.md, docs/schema.md, schema/project.yaml.example, mise.toml. |
| P4: Durable callback protocol | done | BU-0214–BU-0234 | docs/callbacks.md only. |
| P5: Project resolution, status, sync, td-query & graphify | done | BU-0235–BU-0266 | 7 files: bin/sgt-list, bin/sgt-context, bin/sgt-status, bin/sgt-sync, bin/sgt-td-list, bin/sgt-graphify, skills/load-project/SKILL.md. |
| P6: Cross-repo planning & dispatch | done | BU-0267–BU-0312 | 7 files: skills/cross-repo-work/SKILL.md, skills/dispatch/SKILL.md, bin/sgt-dispatch, bin/sgt-td-create, bin/sgt-treehouse-init, bin/_sgt-review-axes.sh, templates/worker-brief.md. |
| P7: Worker lifecycle: interactive session & validation | done | BU-0313–BU-0396 | 6 files: bin/_sgt-harness.sh, bin/_sgt-intent.sh, bin/sgt-td-memory, bin/sgt-interactive-worker, bin/sgt-validate, bin/sgt-validation-worker (per run3-inventory.md). Two files (sgt-interactive-worker 1137 lines, sgt-validate 1004 lines) are exceptionally dense; extraction captured every distinct, independently-triggerable behavior at a depth consistent with this stage's other partitions rather than a mechanical one-unit-per-line pass over every atomic-write helper invocation (evidence-policy's one-behavior-per-unit rule is about independently-triggerable behaviors, not every repetition of the same general atomic-publish/ownership-tracking pattern applied to a different state file). |
| P8: Response, wake & recovery | done | BU-0397–BU-0514 | 5 files: bin/sgt-respond, bin/sgt-ack-response, bin/sgt-wake, bin/sgt-recover, bin/_sgt-response-lock.sh (per run3-inventory.md). Dense state-machine bash touching every consequence class at once (response identity, delivery, recovery, drain/lease safety, escalation); extraction ran deeper than the P1–P7 average (118 units / 2048 lines) because the checklist's five hunt questions are largely what this partition's files *are*, not incidental guardrails inside otherwise-mechanical files. |
| P9: Drain control | done | BU-0515–BU-0567 | 4 files: bin/sgt-drain, bin/sgt-drain-force, bin/sgt-undrain, bin/_sgt-drain.sh (per run3-inventory.md). The admission-lock library (_sgt-drain.sh) carries most of this partition's consequence-class weight — hard-link-based mutual exclusion with nonce-bound reclaim/release, PID-liveness/PID-reuse detection duplicated in three call sites (sgt-drain --wait, sgt-drain-force, the lock's own owner-staleness check), and a deliberately best-effort (not fail-closed) Claude background-session id cross-check. |
| P10: Fleet monitoring & cleanup | done | BU-0568–BU-0679 | 2 files: bin/sgt-watch (738 lines, 40 units), bin/sgt-cleanup (2774 lines, 72 units — the densest single file harvested so far in this run; heavily transactional stage/verify/publish/rollback pattern repeated across worker-evidence removal, retry-owner validation, response-handshake retirement, and cleanup-phase checkpointing). Extraction ran at a depth consistent with this stage's other partitions (per P7's precedent) rather than a mechanical one-unit-per-branch pass over sgt-cleanup's many near-identical field-by-field retry-owner comparisons — related checks that verify the same identity claim from different angles were grouped into one unit per distinct verification target (see BU-0652–BU-0659) rather than split per `_die` call. |
| P11: Escalation & finding routing | done | BU-0680–BU-0815 | 4 files: bin/sgt-callback, bin/sgt-notify, bin/sgt-review-findings, bin/sgt-no-mistakes-finding (per run3-inventory.md). Extremely consequence-dense partition (durable-delivery/security-review-routing domain the checklist's own motivating example was drawn from); extraction ran deeper than the P1–P7 average (136 units / 2044 lines) to match. sgt-callback (Python, 960 lines) alone accounts for 55 units given its defense-in-depth re-validation-on-every-read pattern across origin/event/state schemas. |
| P12: Wiki capture & digest | done | BU-0816–BU-0858 | 2 files: skills/wiki/SKILL.md (14 units), bin/wiki-daily-digest (29 units). All quote/quote_hash pairs re-verified against contiguous source bytes (this attempt's own check) before append. |
| P13: DAG-driven dispatch (dagr integration) | done | BU-0859–BU-0876 | 2 files: bin/sgt-dag-run (12 units), bin/sgt-dag-dispatch-hook (6 units). Quote/quote_hash verified against contiguous source bytes before append. |
| P14: Shared bash foundation | done | BU-0877–BU-0926 | 2 files: bin/_sgt-bash-version.sh (1 unit — a simple version-compat gate), bin/_sgt-lib.sh (49 units — extremely consequence-dense shared library: harness launch-contract identity pinning, TOCTOU-safe owned-file read/write primitives, managed-coordinator-pane adoption/injection-safety, notification publish/delivery race-safety incl. the GH #168 delivery-vs-completion distinction, systemd background-monitor ownership/TOCTOU handling, and branch-reachability unpushed-commit detection). Extraction ran deeper than the P1–P7 average given this file's density; near-identical TOCTOU verify-open-verify-read-verify patterns applied to different state-file shapes were grouped into one unit per distinct verification target (per P7/P10 precedent) rather than split per individual re-check branch. Quote/quote_hash pairs re-verified independently against contiguous source bytes (sed+sha256sum recipe, evidence-policy) for a sample spanning both short and >500-char (prefix+span_bytes) records before append. |
| P15: Vendored single-doc engineering skills (mattpocock/skills) | done | BU-0927–BU-1032 | 9 files (per run3-inventory.md): code-review/SKILL.md, diagnosing-bugs/SKILL.md, grill-with-docs/SKILL.md, grilling/SKILL.md, implement/SKILL.md, research/SKILL.md, resolving-merge-conflicts/SKILL.md, to-spec/SKILL.md, wayfinder/SKILL.md. `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` is not part of this partition's harvest scope per the orchestrator ruling above (helper-evidence under run 3's scheme); diagnosing-bugs/SKILL.md's own reference to invoking that script is recorded (BU-0945) with a note, not a separate extraction of the script's contents. wayfinder/SKILL.md is the densest file in this partition (34 units / 128 lines) given its ticket-type, claim, and fog/scope vocabulary; the other 8 files are comparatively thin single-purpose skill wrappers. All quote/quote_hash pairs computed programmatically from the exact cited byte spans and independently re-verified (sed+sha256sum recipe) against a sample spanning short one-line and multi-line quotes before append. |
| P16: Vendored multi-doc skill: codebase-design | done | BU-1033–BU-1056 | 3 files: SKILL.md (7 units), DEEPENING.md (9 units), DESIGN-IT-TWICE.md (8 units). Design-vocabulary/method skill; glossary/definitional prose (Module/Interface/Depth/etc. term definitions, the deep-vs-shallow diagrams, the Relationships section) was not extracted as behavior units since it has no independently-triggerable trigger/outcome shape — only the file's actionable directives and rules were. Quote/quote_hash pairs computed programmatically (sed span + sha256sum) and independently re-verified against contiguous source bytes for all 24 records before append; BU-1049 quotes a >500-byte span (521 bytes) as a 500-byte prefix with `span_bytes` recorded, hash covers the full span per evidence-policy. |
| P17: Vendored multi-doc skill: domain-modeling | done | BU-1057–BU-1077 | 3 files: SKILL.md (9 units), ADR-FORMAT.md (6 units), CONTEXT-FORMAT.md (6 units). BU-1071 (ADR-FORMAT.md L39-47) quotes a >500-byte span (1235 bytes) as a 500-byte prefix with `span_bytes` recorded, hash covers the full span; all other quotes fit whole. The ADR-FORMAT.md three-part offer-test (L29-38) restates SKILL.md's identical rule (BU-1065) and was not re-extracted as a second unit — noted in BU-1065's `notes` instead. Quote/quote_hash pairs computed programmatically (sed span + sha256sum) and independently re-verified against contiguous source bytes for all 21 records before append. |
| P18: Vendored multi-doc skill: prototype | done | BU-1078–BU-1125 | 3 files: SKILL.md (8 units), LOGIC.md (17 units), UI.md (23 units). Dense, highly deterministic multi-step procedure files (branch selection, TUI build steps, sub-shape selection, switcher build steps, anti-pattern lists); extraction ran near one-behavior-per-distinct-rule depth given how few of this file's rules are genuinely near-duplicates of each other (contrast P7/P10's grouping precedent, which applied to literally repeated per-field checks — not applicable here). BU-1078 (SKILL.md L12-15, 513 bytes) and BU-1090 (LOGIC.md L30-35, 532 bytes) quote >500-byte spans as 500-byte prefixes with `span_bytes` recorded, hash covers the full span in both cases; all other quotes fit whole. Quote/quote_hash pairs computed programmatically (sed span + sha256sum) and independently re-verified against contiguous source bytes for all 48 records before append. |
| P19: Vendored multi-doc skill: tdd | done | BU-1126–BU-1142 | 3 files: SKILL.md (11 units), mocking.md (4 units), tests.md (2 units). tests.md's two worked-example code blocks (DB-bypass, tautological-total) and mocking.md's SDK-approach benefit list are illustrations/elaborations of rules already captured elsewhere (SKILL.md's implementation-coupled/tautological anti-patterns, mocking.md's own SDK-interfaces unit) and were not re-extracted as separate units — noted in the covering unit's `notes` instead, per this run's no-manufactured-duplicates practice (P17's ADR-FORMAT.md precedent). All 17 quotes fit within 500 bytes whole (no truncation needed). Quote/quote_hash pairs computed programmatically (sed span + sha256sum) and independently re-verified against contiguous source bytes for all 17 records before append. |
| P20: Vendored multi-doc skill: triage | done | BU-1143–BU-1195 | 3 files: SKILL.md (32 units), AGENT-BRIEF.md (7 units), OUT-OF-SCOPE.md (14 units). `.agents/skills/triage/agents/openai.yaml` is helper-evidence per run3-inventory.md, not harvested (consistent with the file census note above). SKILL.md's state-machine/dispatch table (roles, transitions, gather/recommend/verify/grill/apply-outcome steps, quick override) is this partition's densest, consequence-heavy content — safety/human-decision boundaries (e.g. never writing an already-implemented wontfix to `.out-of-scope/`, the discovery filter never blocking an explicitly-named PR) and a recovery-class ambiguous-verification-outcome reclassification were all present and swept. A handful of restated-in-brief duplicates across the partition's own files (SKILL.md's step 1 prior-notes mention vs. its own "Resuming a previous session" section; SKILL.md's wontfix/already-implemented outcome-table line vs. OUT-OF-SCOPE.md's fuller "When to write" rationale for the same boundary) were captured once at their fuller/primary statement and cross-referenced in notes rather than re-extracted, per this run's no-manufactured-duplicates practice (P17/P19 precedent). Quote/quote_hash pairs computed programmatically (Python span extraction + sha256) and independently re-verified via the sed+command-substitution+sha256sum recipe (evidence-policy) for a sample spanning short single-line and multi-line quotes, plus the one >500-byte span (AGENT-BRIEF.md L11-17, 518 bytes, quoted as a 500-char prefix with `span_bytes` recorded, hash over the full span), before append. |
| P21: Sergeant-authored operational skills | done | BU-1196–BU-1333 | 3 files (per run3-inventory.md): `.agents/skills/no-mistakes/SKILL.md` (64 units — dense state-machine/consent-gate reference: run lifecycle, gate/finding-action semantics, branch_sync recovery paths, ask-user escalation, `--yes` scope), `.agents/skills/sergeant-setup/SKILL.md` (37 units — a 10-phase interactive bootstrap wizard with a consent gate at nearly every write and an explicit write-path allowlist/denylist), `.agents/skills/to-tickets/SKILL.md` (37 units). Extraction ran deeper than the P1–P7 average given how consequence-dense the no-mistakes and sergeant-setup files are (per-sentence consent gates, fail-closed recovery branches, explicit never-rules), consistent with P8/P11/P14's precedent of matching depth to genuine density rather than a flat per-line rate; to-tickets is comparatively principle/style guidance and was extracted at moderate depth. All quote/quote_hash pairs computed programmatically (Python span extraction + sha256) over exact line-range spans and independently re-verified for all 138 records via the sed+command-substitution+sha256sum recipe (evidence-policy) before append — zero mismatches. 5 records quote a >500-byte span as a 500-character prefix with `span_bytes` recorded (BU-1218, BU-1241, BU-1252, BU-1271, BU-1276), hash covering the full span in each case. |

## Scheme provenance (run 4)

This resumed run (run 4, attempt 1) harvests by **run 3's partition
scheme**, per `output/run3-inventory.md` (a copy of run 3's committed
`10-inventory/output/inventory.md`, taken for scheme provenance only — see
below). Run 3 and this run's own `10-inventory` stage partitioned the same
file census differently (21 P-partitions vs. this run's 19 A–S partitions —
partitioning is actor judgment, not deterministic); the resume reconciles
at the **file level**, not the partition-label level. This run's own
`../10-inventory/output/inventory.md` (A–S) stands as this run's `10-inventory`
stage's record but is **not** the harvest bookkeeping — the rows above use
run 3's P-numbers and file lists.

Seeded units BU-0001–BU-0312 (partitions P1–P6, marked `done` above)
originate from run 3, not from this attempt's own reading — source:
committed evidence at
`/home/miztertea/sergeant-rs/docs/gauntlet/runs/n2-run3/.sergeant/workflows/repo-to-icm/20-harvest/output`
(`behavior-units.ndjson`, `consequence-class-sweep.md`,
`partition-ledger.md`, all copied verbatim into this run's `output/` as the
seed for this resume).

### FILE CENSUS CHECK — MISMATCH FOUND, HARVEST STOPPED (this attempt)

Per the seeding instruction's fail-closed condition: before harvesting any
`pending` partition (P7 onward), the union of decompose files across
`output/run3-inventory.md`'s 21 partitions must equal this run's own
`../10-inventory/output/inventory.md` decompose-file census exactly.

Computed both sets (82 files in run 3's union across P1–P21; 83 files in
this run's own A–S decompose census) and diffed them. They are **not**
equal — exactly one file differs:

- `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` — present
  in this run's own `../10-inventory/output/inventory.md` decompose census
  (partition E row: `` `diagnosing-bugs/` | D | H | `scripts/hitl-loop.template.sh`
  (D — HITL repro script the skill's Phase 1 explicitly names and hands to
  the user) ``), **absent** from run 3's decompose union — run 3's
  `output/run3-inventory.md` dispositions this same file
  `helper-evidence`, not `decompose` (row: `` `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`
  | helper-evidence | — | Copy-and-edit human-in-the-loop bash template
  invoked as this skill's Phase-1 last-resort loop mechanism. ``, and it is
  correspondingly absent from run 3's P15 (`.agents/skills/diagnosing-bugs/SKILL.md`
  is in P15; the `scripts/hitl-loop.template.sh` support file is not).

No other file differs between the two sets (verified: all 82 of run 3's
decompose-union files are present in this run's 83; the only file in this
run's 83 not in run 3's 82 is the one named above).

This is a genuine disposition disagreement between two independent
`10-inventory` runs over the same source file, not a missing/duplicated
file — it is the exact "in one set and not the other" condition the
seeding instruction names as the remaining fail-closed condition. Per that
instruction: **stop before harvesting.** This attempt did not read or
extract any P7–P21 file, did not append to `behavior-units.ndjson` or
`consequence-class-sweep.md` beyond the run-3 seed copy, and left every
P7–P21 row `pending` above. Resolving the disagreement (does
`.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh` belong in
partition P15's harvest scope or not) is a decision this stage's actor is
not positioned to make unilaterally — it requires either a human operator's
ruling or a reconciling pass over the two `10-inventory` dispositions
themselves, neither of which this stage's contract authorizes it to
perform. A future attempt resuming this ledger should not proceed past P6
until that ruling is recorded here.

### Orchestrator ruling on the census mismatch (recorded run 4, attempt 2, 2026-08-11)

Per L9, binding on every attempt of this stage:

> the census delta found at seeding — exactly one file,
> `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`, decompose
> in run 4's inventory vs helper-evidence in run 3's — is ruled in run 3's
> favor **[reference-corpus citation redacted at 90-reconcile, AF-0001 —
> the ruling originally quoted a specific `reference-corpus/` line number,
> disposition, and adjudication id here; that content is removed from this
> run's own committed evidence per the blindness rule, without changing the
> ruling's operative disposition below]**. The file is NOT harvested under
> this run's scheme. If the ledger's census-mismatch subsection does not yet
> record this ruling, append it there and then proceed — a census delta
> consisting ONLY of that one ruled file is resolved, not blocking. Any
> OTHER file-level delta remains fail-closed.

Disposition: `.agents/skills/diagnosing-bugs/scripts/hitl-loop.template.sh`
is **not** harvested by this stage — it is dispositioned `helper-evidence`
per run 3's `output/run3-inventory.md` (the scheme this resumed run
follows), and this stage's own `../10-inventory/output/inventory.md`'s
`decompose` disposition for the same file (partition E) stands recorded
here only as a generator-vs-prior-attempt disposition disagreement — it
does not change what this stage harvests. This stage's actor did not
consult `reference-corpus/` to reach or verify this ruling
(`../_config/run-discipline.md` §1 blindness rule); the ruling was supplied
pre-adjudicated by the orchestrator, not derived by this stage. **Note
(90-reconcile, AF-0002):** the ruling's original wording framed this as
"the comparison record" against a graded reference corpus; `../00-contract/
output/contract.md` §3 explicitly rules this run carries no measurement
framing and no answer-key directory is present in this worktree. That
framing has been removed from this disposition line (above) as part of the
AF-0002 repair — the disagreement is recorded here as an ordinary
generator-vs-prior-attempt disposition disagreement, not a graded-comparison
record, consistent with the contract's own ruling.

With this the only file-level delta between the two censuses, and it now
resolved, the census check passes: harvesting resumes at P7 using
`output/run3-inventory.md`'s partition scheme and per-partition file lists,
per the "Scheme provenance (run 4)" section above.
