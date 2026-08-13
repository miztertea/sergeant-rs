# 20-select-intent-transport: select intent transport

## Inputs

| File | Layer | Why |
|---|---|---|
| ../00-check-scope/output/README.md | L4 | upstream artifact produced by `00-check-scope`, when entered via the directly-invoked path |
| ../10-do-the-work/output/README.md | L4 | upstream artifact produced by `10-do-the-work`, when entered via the directly-invoked path |

## Purpose

Readiness, launch ownership, and an isolated code snapshot are established (coordinator-launched entry only), then the intent transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run.

Trigger (workflow-level): Implementation, native tests, lint and independent review are complete and the coordinator has reached the approved shipping boundary.

## What must become true here (durable outcome)

A published readiness marker asserts the exact intent revision, the exact reviewed head, and an explicit pass on every review axis; an identity-checked launch reservation and an isolated, re-verified code snapshot are held; the transport is probed against the installed build's real capability, decided once with explicit consent for the exposing option, recorded twice for audit, and re-checked before the run.

## Behavior contract

- **Whether a validation run's canonical intent is passed to the shipping-gate tool via a private file path or via argv is decided once, probed against the actually-installed build's real capability (not assumed), and requires explicit operator consent to select the argv transport at all, because argv exposes intent content through process listings and /proc.**
  (trigger: a validation launch is about to decide how to pass canonical intent; outcome: intent content is never exposed via argv without the operator's explicit, per-run consent, and the transport decision is grounded in what the installed tool actually supports)
  — `BU-P6-134`, `reference/sergeant-upstream/bin/sgt-validate` (L274-281)
- **Canonical intent content must never appear in process arguments (readable by any local process via ps or /proc/<pid>/cmdline); sgt-validate therefore probes no-mistakes' own --help output before creating a run and requires an --intent-file capability, delivering intent through a path instead of argv.**
  (trigger: the coordinator is about to launch the final validation run; outcome: the intent's content is never exposed to any other local process through argv, by construction)
  — `BU-P8-085`, `reference/sergeant-upstream/docs/using-sergeant.md` (L333-339 (Intent transport and the argv consent gate))
- **The transport actually used for a validation launch is recorded twice for durable audit: a current-run marker cleared only on retry-reset, and an append-only log of timestamp/transport/HEAD/intent-revision for every committed launch, so the privacy decision stays auditable across retries; the validation worker also re-checks the recorded transport against the build that will actually run, so a no-mistakes binary swapped between launch and run can neither silently downgrade a private transport into argv nor invoke a flag that build no longer accepts.**
  (trigger: a validation run's launch-time transport decision must still hold true at actual run time; outcome: a TOCTOU-style swap of the no-mistakes binary between launch and run can never silently weaken the privacy guarantee already recorded)
  — `BU-P8-087`, `reference/sergeant-upstream/docs/using-sergeant.md` (L348-355)
- **sgt-validation-worker must pass the canonical intent to no-mistakes via an `--intent-file` path rather than inline content in argv (so intent content never appears in process arguments/listings), and capability-probing invocations of no-mistakes (--version, `axi run --help`) must not count as a validation run.**
  (trigger: sgt-validation-worker launches no-mistakes with the canonical intent; outcome: intent content (which may include sensitive implementation detail) never transits argv where it would be visible to other users on the same host via process listings)
  — `BU-P7-105`, `reference/sergeant-upstream/tests/sgt-validation-worker-test.sh` (lines 37-38)
- **Before invoking the shipping-gate tool, the validation worker re-reads and re-hashes the canonical intent file and refuses to proceed if its content changed since the coordinator's own initial verification, so a content edit made in the gap between launch and run is caught rather than silently validated.**
  (trigger: the validation worker is about to invoke no-mistakes; outcome: the intent that gates a validation run is guaranteed byte-identical to the intent verified at launch, with no window for a late substitution)
  — `BU-P6-042`, `reference/sergeant-upstream/bin/sgt-validation-worker` (L172-183)

## Helper: verify readiness, acquire launch reservation, reserve isolated snapshot (folded from demoted `00-verify-readiness`, `10-acquire-launch-reservation`, `20-reserve-isolated-snapshot`, N1 adjudication A4)

These three checkpoints were classified at extraction as deterministic machinery (ladder §6.5) with no checkpoint argument beyond the boilerplate; per adjudication A4 all three are demoted and folded here, in sequence, as helpers preceding the transport decision. They apply only to the **coordinator-launched entry** (a worker's already-reviewed commit is being launched into validation beside the worker); the directly-invoked entry (`00-check-scope` → `10-do-the-work`) has no separate worker to hand off from and skips straight to the transport decision below.

- **A validation run can only be launched once the worker has published a readiness marker asserting the exact intent revision, the exact reviewed head commit, and that all three independent review axes (standards, spec, readiness) explicitly passed — a stale head, a mismatched intent revision, or any axis not equal to 'passed' each refuse the launch with its own specific reason.**
  (trigger: the coordinator attempts to launch validation; outcome: validation can never be launched against work that has not been reviewed, or against a different commit than the one that was reviewed)
  — `BU-P6-130`, `reference/sergeant-upstream/bin/sgt-validate` (L236-269)
- **A worker may only request the final no-mistakes boundary by writing durable validation-ready evidence (intent_revision, head_sha, and pass/fail for standards/spec/readiness review) and notifying the coordinator; the worker itself is forbidden from running no-mistakes.**
  (trigger: a worker's native validation and independent reviews all report zero blockers; outcome: only the coordinator, never the worker itself, ever crosses the final shipping-gate checkpoint)
  — `BU-P8-082`, `reference/sergeant-upstream/docs/using-sergeant.md` (L312-317 (Final no-mistakes boundary))
- **Before cloning the validation checkout or publishing any launch state, the coordinator must acquire an identity-checked validation-launch reservation for that exact task/repository pair, and concurrent launch attempts fail closed until the recorded owner exits or stale-ownership recovery proves the reservation abandoned.**
  (trigger: two launches of sgt-validate for the same task/repository could race; outcome: only one validation run per task/repository can ever be in flight, and a concurrent second attempt fails closed rather than double-launching)
  — `BU-P8-084`, `reference/sergeant-upstream/docs/using-sergeant.md` (L328-331)
- **A validation run's ownership can only pass from the original dispatching coordinator to a different claimant through an explicit ownership-claim request; resuming from the original owning coordinator re-asserts ownership automatically, but any other caller is refused validation unless it explicitly claims ownership, and a caller that is neither the original owner nor an explicit claimant is refused outright even if the original owner still appears live.**
  (trigger: a validation run is about to launch and its coordinator ownership must be resolved; outcome: exactly one accountable coordinator ever owns a validation run, and only ever by original dispatch or explicit claim, never by default or inference)
  — `BU-P6-143`, `reference/sergeant-upstream/bin/sgt-validate` (L282-308) — split off `BU-P6-129` at N1 adjudication A10; routed here at N1 verifier round 2 (finding V3)
- **Validation runs against a code-cloned isolated snapshot, not the worker's live worktree — created via a shared, no-checkout local clone then checked out at the exact reviewed commit, with an owner marker recorded inside the clone's own git directory — so a shipping-gate run can never observe a worktree the worker continues to mutate concurrently.**
  (trigger: a validation run needs a code snapshot to validate against; outcome: a shipping-gate run's verdict is always about a genuinely frozen snapshot of exactly the reviewed commit, never a moving worktree)
  — `BU-P6-133`, `reference/sergeant-upstream/bin/sgt-validate` (L829-845, L855-858)
- **The isolated validation code snapshot's identity is re-verified against the reviewed commit immediately before invoking the shipping-gate tool — the snapshot must still be at the exact reviewed HEAD and have a clean tree — so validation can never silently run against code that changed after review.**
  (trigger: the validation worker is about to invoke no-mistakes against the isolated snapshot; outcome: the exact code that was reviewed is the exact code that gets validated — no substitution window)
  — `BU-P6-044`, `reference/sergeant-upstream/bin/sgt-validation-worker` (L123-129)

`BU-P6-129` (the coarse "launching validation is its own bounded, independently invocable procedure" claim) is cited at workflow level in `provenance.md`, not repeated here: per N1 adjudication A10 (finding N1-BH-08) it is `confidence: low` and narrowed to that coarse boundary claim in the corpus, since its four original sub-claims are now separately, more strongly cited above.

## Helper: repo-level pre-push gate (re-homed from the demoted `repo-release-verification` package, N1 adjudication A6)

`repo-release-verification` was demoted from a standalone workflow (finding N1-BH-06: it was file-shape mirroring — §6.2's workflow test was never actually argued for it). Its behavior is this repository's own git pre-push hook, which mechanically gates *every* push in this repository (not only pushes made during this workflow) — the closest binding checkpoint to it in `validate-and-ship` is the point where committed work is about to be handed to the pipeline for the first time, which is this stage's own precondition chain. See `provenance.md`'s "Re-homed from repo-release-verification (A6)" section for the full re-homing record.

- **Before every git push, the drain test suite must run and pass; the push is blocked on failure unless the operator explicitly opts out with git push --no-verify.**
  (trigger: operator runs git push; outcome: push proceeds only after the drain suite passes, or the operator explicitly consents to skipping validation)
  — `BU-P6-007`, `reference/sergeant-upstream/scripts/hooks/pre-push` (L2-11)
- **If the tooling required to run the pre-push validation (mise, docker) is unavailable, the hook fails closed with exit 1 and an actionable message, rather than silently skipping validation and letting the push through.**
  (trigger: required validation tooling is missing on push; outcome: a push with unrunnable validation is blocked with a diagnosis, never silently allowed through)
  — `BU-P6-008`, `reference/sergeant-upstream/scripts/hooks/pre-push` (L29-33, L35-39)

## Judgment required

This is an actor stage (ladder §6.4): the acting harness must inspect evidence, choose among alternatives, ask the user where the behavior contract above requires it, or explain a decision — it is not mechanically executable from the contract alone. Treat the statements above as binding constraints on that judgment, not as a script to execute verbatim.

## Output

Declared in `output/README.md` (Layer 4). See that file for the expected artifact and its merge disposition.
