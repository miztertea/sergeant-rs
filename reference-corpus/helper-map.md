# Helper Map

> **Superseding note (added post-round-1-adjudication, V10).** The `W`-numbered
> workflow references and stage names below are `synthesis.md`'s vocabulary
> as of that phase artifact and are now historical — `adjudication-round1.md`'s
> structural rulings (A3–A8) reordered, demoted, restored, merged, split, and
> removed stages and whole packages afterward (e.g. `W19 repo-release-verification`,
> cited in two entries below, no longer exists as a standalone package — see
> `provenance-map.md`'s Tombstones section). Each `draft-workflows/<name>/provenance.md`
> and `workflow.toml` is the authoritative, current source for which package
> and stage actually consumes a given helper; `adjudication-round1.md` is the
> bridge document explaining how the two diverge. This note is additive — the
> entries below are otherwise unedited.

Part of the N1 reference corpus (`docs/gauntlet/contracts/N1.md`, §8.1's
`helper-map.md`). Sourced from `synthesis.md` §3c ("Shared mechanics") and the
workflow-local helper list embedded in the same section, applying §6.5/§6.6 of
the ICM decomposition ladder: a helper is deterministic machinery invoked
*while crossing* a checkpoint, subordinate to the stage's judgment-bearing
outcome (never a stage in its own right — §6.3), and it is **shared**
(`.sergeant/common/scripts/` per `docs/icm/convention.md` §5) only if several
workflows consume it under the *same contract*; otherwise it stays
workflow-local (§6.6).

Each entry: contract (what it guarantees, not how old Sergeant implemented
it — mechanism is noted separately where it matters), source evidence
(behavior-unit ids, with the anchor unit's path/locator inlined for
orientation; every id's full record and `quote_hash` is the canonical copy in
`reference-corpus/behavior-units/`), consuming workflows (by `synthesis.md`
§1 id), and the shared-vs-local verdict.

---

## Shared helpers (`.sergeant/common/scripts/` candidates)

### `finding-router`
**Contract:** map a review/validation finding's disposition to a priority
deterministically; deduplicate by a stable marker; on rerun, resurface or
preserve a superseded finding rather than silently dropping it; preserve
labels and revision digests across supersession; retain a sanitized copy of
the finding artifact durably, with retry on failure.
**Source evidence:** BU-P1-081, BU-P1-082, BU-P1-083, BU-P1-084, BU-P1-086,
BU-P1-087, BU-P1-088, BU-P1-089, BU-P1-090, BU-P6-025, BU-P6-087. Anchors:
`reference/sergeant-upstream/README.md` L306 (disposition/priority mapping);
`reference/sergeant-upstream/bin/sgt-no-mistakes-finding` L107, L160-162;
`reference/sergeant-upstream/bin/sgt-review-findings` L244-265.
**Used by:** W8 `dispatch`, W9 `worker-mission`, W16 `route-review-findings`,
W18 `validate-and-ship`.
**Verdict:** shared — four independent workflows apply the identical
disposition→priority→dedup contract to structurally different finding sources
(dispatch escalations, worker review output, `no-mistakes` output).

### `mutual-exclusion`
**Contract:** acquire an exclusive lock with hard-link-based locking (chosen
for Bash-3.2/POSIX portability, not a kernel-held handle); publish the owner
record atomically together with the lock; reclaim a lock whose recorded owner
is proved dead by nonce, never by suspicion; report a diagnosable timeout
naming the owner and the exact recovery action.
**Source evidence:** BU-P6-052, BU-P6-059, BU-P6-060, BU-P7-085. Anchors:
`reference/sergeant-upstream/bin/_sgt-drain.sh` L126-132 (rationale shared
with `bin/_sgt-response-lock.sh`'s identical lock design);
`reference/sergeant-upstream/tests/sgt-drain-test.sh` lines 15-27.
**Used by:** W8 `dispatch`, W10 `respond-to-worker`, W11
`recover-stalled-worker`, W12 `drain-fleet`, W17
`deliver-external-callback`, W18 `validate-and-ship`.
**Verdict:** shared — the exact same lock design (hard-link + atomic owner
record + nonce-proven reclaim) recurs verbatim in drain, response, and
validation-launch locking; a second, independently-drifting implementation
would be the anti-pattern Article VII exists to prevent.

### `owned-write`
**Contract:** stage new content to a private candidate location, verify its
filesystem identity (not merely its path), publish by atomic rename, and
record the owned path; identity-sensitive reads accept only a regular,
mode-600, owner-owned file — never a symlink or a file with looser
permissions.
**Source evidence:** BU-P6-132, BU-P7-054, BU-P7-055, BU-P7-056, BU-P7-057.
Anchors: `reference/sergeant-upstream/bin/sgt-validate` L470-491;
`reference/sergeant-upstream/tests/sgt-lib-notification-target-test.sh` lines
1-22.
**Used by:** W8 `dispatch`, W10 `respond-to-worker`, W15
`reconcile-and-cleanup-fleet`, W18 `validate-and-ship`.
**Verdict:** shared — the stage/verify-identity/atomic-rename contract is
consumed identically wherever a durable artifact must never be observed
half-written.

### `process-identity`
**Contract:** classify whether a recorded PID still names the process that
recorded it (PID-reuse detection by start time), verify process-group
leadership before treating a group as owned, and fail closed — refuse rather
than guess — whenever that proof is unavailable.
**Source evidence:** BU-P6-040, BU-P6-138, BU-P7-082. Anchors:
`reference/sergeant-upstream/bin/sgt-drain-force` L139-157;
`reference/sergeant-upstream/tests/sgt-cleanup-test.sh` line 3817 (Guard 1;
Guard 2 at line 3863).
**Used by:** W11 `recover-stalled-worker`, W12 `drain-fleet`, W13
`monitor-fleet`, W15 `reconcile-and-cleanup-fleet`.
**Verdict:** shared — every fleet-lifecycle workflow that must decide "is this
still the same process" needs the identical PID-reuse-safe test.

### `action-lease`
**Contract:** record an outcome exactly once and never overwrite it once
settled; support reentrant settlement attempts with futile-wait detection
deferred to the caller's exit boundary; release semantics that never release a
lease this caller does not own.
**Source evidence:** BU-P6-054, BU-P6-056, BU-P7-045, BU-P7-046, BU-P7-050.
Anchors: `reference/sergeant-upstream/bin/_sgt-response-lock.sh` L258-268;
`reference/sergeant-upstream/tests/sgt-response-lock-release-test.sh` lines
1-4.
**Used by:** W9 `worker-mission`, W10 `respond-to-worker`, W11
`recover-stalled-worker`, W12 `drain-fleet`, W15
`reconcile-and-cleanup-fleet`.
**Verdict:** shared — the exactly-once-settlement contract is identical across
every workflow that must not double-apply a human decision or double-count a
completion.

### `harness-registry`
**Contract:** one declaration per harness drives its capability gate,
readiness probe, and launch arguments together; every row is validated before
the gate is trusted; the readiness probe never depends on a UI string.
**Source evidence:** BU-P6-005, BU-P6-065, BU-P6-069, BU-P7-089, BU-P7-090.
Anchors: `reference/sergeant-upstream/mise.toml` tasks.check,
`_check_agent_harness`, L183-190; `reference/sergeant-upstream/tests/sgt-harness-test.sh`
lines 1-13.
**Used by:** W3 `sergeant-setup`, W8 `dispatch`, W10 `respond-to-worker`, W11
`recover-stalled-worker`.
**Verdict:** shared — directly evidences Article V's "one declaration drives
gate, probe, and invocation" rule; every consumer must derive its answer from
the same declaration or the corpus's own measured-capability discipline
breaks.

### `capability-probe`
**Contract:** verify a tool supports a needed capability by probing the
tool's own help/usage surface for the exact flags required — never by
inferring support from a version string.
**Source evidence:** BU-P6-004, BU-P7-027, BU-P7-034, BU-P7-103, BU-P8-060.
Anchors: `reference/sergeant-upstream/mise.toml` tasks.check, `_check_td`,
L169-172; `reference/sergeant-upstream/docs/using-sergeant.md` L60-64.
**Used by:** W3 `sergeant-setup`, W8 `dispatch`, W18 `validate-and-ship`.
**Verdict:** shared — the same probe-the-real-surface contract governs `td`,
harness, and `no-mistakes` capability checks alike (Article V).

### `intent-intake`
**Contract:** validate a path is safe to read as canonical intent content
before reading it — reject newlines, path traversal, symlink components,
oversized content, and control characters up front.
**Source evidence:** BU-P6-050.
Anchor: `reference/sergeant-upstream/bin/_sgt-intent.sh` L194-209.
**Used by:** W8 `dispatch`, W18 `validate-and-ship`.
**Verdict:** shared — single unit but two independent consumers with the
identical path-safety contract; not workflow-local by §6.6's test.

### `payload-safety`
**Contract:** reject an oversized, control-character-bearing, metacharacter-bearing,
secret-shaped, or platform-id-bearing payload before it is queued or written
— never after.
**Source evidence:** BU-P6-119, BU-P8-017. Anchors:
`reference/sergeant-upstream/bin/sgt-callback` L278-300;
`reference/sergeant-upstream/docs/callbacks.md` L84-87.
**Used by:** W16 `route-review-findings`, W17 `deliver-external-callback`.
**Verdict:** shared.

### `callback-plumbing`
**Contract:** verify the profile path and its permissions at invocation time;
bind origin by correlation-id and source-id patterns; classify every event
into one of four typed classes; track retry/claim/backoff state; support
scoped and fleet-wide drains; require requeue-after-repair and consumer-side
deduplication.
**Source evidence:** BU-P6-118, BU-P8-008, BU-P8-010, BU-P8-012, BU-P8-013,
BU-P8-015, BU-P8-016, BU-P8-020, BU-P8-023, BU-P8-024, BU-P8-025, BU-P8-028.
Anchors: `reference/sergeant-upstream/bin/sgt-callback` L127-144;
`reference/sergeant-upstream/docs/callbacks.md` L10-14 (Configure a profile).
**Used by:** W15 `reconcile-and-cleanup-fleet`, W17
`deliver-external-callback`.
**Verdict:** shared — this is `callback-protocol` (§3b, shared *context*)
paired with its executing mechanics; kept as a distinct helper entry because
the protocol document and the invocation/retry state machine are different
artifacts consumed differently (an actor reads the protocol; the mechanics run
it).

### `state-resolution-consistency`
**Contract:** every fleet command resolves drain state and worker identity
through the same shared code path — never a locally reimplemented check that
could drift from the canonical one.
**Source evidence:** BU-P7-102.
Anchor: `reference/sergeant-upstream/tests/sgt-watch-test.sh` lines 7-13.
**Used by:** W8 `dispatch`, W10 `respond-to-worker`, W11
`recover-stalled-worker`, W12 `drain-fleet`, W13 `monitor-fleet`.
**Verdict:** shared — five consumers is the corpus's clearest §6.6 case for
"same contract, several workflows."

### `identity-precedence`
**Contract:** resolve forge identity through a fixed four-level precedence
order, applied identically everywhere identity must be resolved.
**Source evidence:** BU-P8-039.
Anchor: `reference/sergeant-upstream/docs/schema.md` L60.
**Used by:** W8 `dispatch`.
**Verdict:** shared by contract shape even though currently single-workflow —
recorded as shared because its stated purpose is "the fixed... precedence,"
i.e. a definition meant to be reused wherever forge identity is resolved, not
a dispatch-only detail. Flag for re-verification once N2 measures other
consumers.

### `handoff-provenance`
**Contract:** refuse to record a handoff unless the caller's path is exactly
the recorded owned surface — never a path that merely looks equivalent.
**Source evidence:** BU-P6-037.
Anchor: `reference/sergeant-upstream/bin/sgt-td-memory` L26-36, L46-56.
**Used by:** W9 `worker-mission`, W10 `respond-to-worker`, W11
`recover-stalled-worker`.
**Verdict:** shared.

### `notify-marker`
**Contract:** one shared per-task marker is polled by the watcher, so
simultaneous updates collapse into a single delayed wakeup instead of
duplicate delivery.
**Source evidence:** BU-P8-071.
Anchor: `reference/sergeant-upstream/docs/using-sergeant.md` L157-159.
**Used by:** W10 `respond-to-worker`, W13 `monitor-fleet`.
**Verdict:** shared.

### `install-plumbing`
**Contract:** discover commands and sourced helpers by glob, symlink them
into place, remove stale links safely, and follow a fixed `mise` discovery
order.
**Source evidence:** BU-P6-001, BU-P6-002, BU-P6-009, BU-P7-028, BU-P7-029.
Anchors: `reference/sergeant-upstream/mise.toml` tasks.install, L20-23;
`reference/sergeant-upstream/tests/mise-install-test.sh` lines 42-51.
**Used by:** W3 `sergeant-setup`, W19 `repo-release-verification`.
**Verdict:** shared.

### `graphify-plumbing`
**Contract:** enforce repo-name constraints; publish graph output
symlink-preservingly with wiki/memory retention; apply Sergeant-side
exclusions with staged extraction; resolve symlink aliases before exclusion.
**Source evidence:** BU-P8-034, BU-P8-035, BU-P8-036, BU-P7-087. Anchors:
`reference/sergeant-upstream/docs/schema.md` L54;
`reference/sergeant-upstream/tests/sgt-graphify-test.sh` line 17.
**Used by:** W2 `project-graph`.
**Verdict:** workflow-local by current evidence (single consuming workflow),
but kept named and separable rather than inlined, since W2 was itself
promoted from a stage to a standalone workflow (conflict X9) and a second
graph-consuming workflow is plausible.

### `worktree-pool`
**Contract:** prefer a leased, pre-warmed surface over creating a fresh one,
with fallback to fresh creation and pooled return of the surface afterward;
the "does this branch carry unpushed work" guard has an explicit false-positive
rule so it never blocks a surface that is actually safe to reuse.
**Source evidence:** BU-P5-072, BU-P5-073, BU-P7-076. Anchors:
`reference/sergeant-upstream/skills/dispatch/SKILL.md` lines 124-126;
`reference/sergeant-upstream/tests/sgt-dispatch-unpushed-guard-test.sh` lines
1-9, 37-45.
**Used by:** W8 `dispatch`, W15 `reconcile-and-cleanup-fleet`.
**Verdict:** shared.

### `test-infrastructure` (source-repo self-hosting)
**Contract:** the isolation audit is itself self-tested, and its coverage
requirement is transitive — a suite that merely sources a helper inherits that
helper's isolation obligation rather than being exempt from it.
**Source evidence:** BU-P7-022, BU-P7-023.
Anchor: `reference/sergeant-upstream/tests/global-state-isolation-test.sh`
lines 263-267.
**Used by:** W19 `repo-release-verification`.
**Verdict:** workflow-local — this is the source repository's own test
infrastructure, scoped to W19 exactly as W19 itself is scoped (§8.2's "worked
example" ruling in `synthesis.md` §1).

### `explicit-invocation-metadata`
**Contract:** the cross-harness (Claude-frontmatter vs. other-harness-config)
mirror of the same "this procedure never auto-invokes" declaration, so the
no-auto-invoke rule is expressed once per harness rather than drifting between
them.
**Source evidence:** BU-P3-002, BU-P3-004.
Anchor: `reference/sergeant-upstream/.agents/skills/grill-with-docs/SKILL.md`
frontmatter, `disable-model-invocation`.
**Used by:** W23 `implement`, W29 `grill-with-docs`.
**Verdict:** shared.

### `domain-artifacts`
**Contract:** `CONTEXT.md`/`CONTEXT-MAP.md` shape and lazy creation rules; ADR
numbering and minimum-viable ADR form; glossary-entry rules (what must and
must not be recorded).
**Source evidence:** BU-P4-029, BU-P4-030, BU-P4-036, BU-P4-038, BU-P4-039,
BU-P4-040, BU-P4-041, BU-P4-044, BU-P4-045, BU-P4-047, BU-P4-048.
Anchor: `reference/sergeant-upstream/.agents/skills/domain-modeling/SKILL.md`,
File structure, L24.
**Used by:** W25 `deepen-module`, W29 `grill-with-docs`, W30 `triage`, W31
`to-spec`, W33 `wayfinder`.
**Verdict:** shared — five consuming workflows, all writing the same artifact
shapes under the same rules.

### `expand-migrate-contract`
**Contract:** a wide-refactor ticket splits into an `expand` phase (add the
new shape alongside the old) and a `migrate` phase (retire the old shape),
never a single combined change.
**Source evidence:** BU-P4-067.
Anchor: `reference/sergeant-upstream/.agents/skills/to-tickets/SKILL.md`, Wide
Refactors, L91-96.
**Used by:** W32 `to-tickets`.
**Verdict:** workflow-local (single consumer in current evidence).

### `branch-sync-decision-table` (and pipeline-inspection commands)
**Contract:** the fixed decision table governing sync/continue/recover-custody
of a pipeline-owned branch, plus the read-only inspection commands (`axi
status`, `axi logs`, `axi abort`, `axi sync --check`) that feed it. These are
the exact mechanisms that §6.3's reimplementation test *demoted* out of W18's
stage list (`synthesis.md` §1, U2 verdict) — a command is not a checkpoint,
"custody of the branch is reconciled" is.
**Source evidence:** BU-P1-071, BU-P1-078, BU-P2-101. Anchors:
`reference/sergeant-upstream/README.md` L271-273, invocation forms;
`reference/sergeant-upstream/.agents/skills/no-mistakes/SKILL.md`,
inspecting-state command reference, lines 253-262.
**Used by:** W18 `validate-and-ship` (stage `70-reconcile-custody`).
**Verdict:** workflow-local — single consumer, and by construction (it exists
*because* it failed the stage test), it should never be promoted to shared:
the durable outcome ("custody reconciled") already lives in W18's stage; the
table is its subordinate mechanism.

---

## Workflow-local helpers (kept local per §6.6)

Each of these has exactly one consuming workflow in current evidence, so it
stays under that workflow's own `scripts/` rather than
`.sergeant/common/scripts/`. Grouped by workflow; not expanded to full
per-unit contracts individually, since none has yet been asked to serve a
second workflow (§6.6's test is not met for any of them).

| Workflow | Helper units | Anchor |
|---|---|---|
| W24 `code-review` | BU-P2-005, BU-P2-011, BU-P2-012 | `reference/sergeant-upstream/.agents/skills/code-review/SKILL.md`, Step 1, line 21 |
| W20 `diagnose-bug` | BU-P2-024 | `reference/sergeant-upstream/.agents/skills/diagnosing-bugs/SKILL.md`, Phase 1 item 10, line 29 |
| W4 `sergeant-help` | BU-P5-116, BU-P5-118, BU-P5-124 | `reference/sergeant-upstream/skills/sergeant-help/SKILL.md`, lines 17-26 |
| W31 `to-spec` | BU-P4-055, BU-P4-056, BU-P4-057 | `reference/sergeant-upstream/.agents/skills/to-spec/SKILL.md`, spec-template, L55 |
| W30 `triage` | BU-P3-076, BU-P3-077, BU-P3-078, BU-P3-079, BU-P3-080, BU-P3-081, BU-P3-082, BU-P3-094, BU-P3-095 | `reference/sergeant-upstream/.agents/skills/triage/AGENT-BRIEF.md`, line 3 |
| W33 `wayfinder` | BU-P4-078, BU-P4-079, BU-P4-081, BU-P4-082, BU-P4-083, BU-P4-084 | `reference/sergeant-upstream/.agents/skills/wayfinder/SKILL.md`, The Map, L21 |
| W35 `wiki-digest` | BU-P5-134, BU-P5-136, BU-P5-144 | `reference/sergeant-upstream/skills/wiki/SKILL.md`, lines 25-29 |
| (troubleshooting reference, not a workflow) | BU-P8-098 | `reference/sergeant-upstream/docs/troubleshooting.md`, L88-93 (Pane is missing) — a documentation cross-reference, not an invoked script; retained here because the extractor originally classed it `helper` and no later pass reclassified it |

---

## Summary

22 shared-helper candidates, 8 workflow-local helper groups (34 units).
Together with `.sergeant/common/context/` (`shared-context-map.md`), these are
the corpus's answer to proposal §10: helpers answer "how is this operation
performed," and every entry above is deterministic machinery invoked while
crossing a checkpoint (§6.5) — none of them independently carries judgment,
which is what would have made it a stage instead (§6.3/§6.4).
