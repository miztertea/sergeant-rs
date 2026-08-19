# ADR 0018: "Measured" obliges validation; "unmeasured" obliges a clean build, not a block

**Status:** Accepted, 2026-08-18. Clarifies ADR 0001; does not supersede it.

## Context

ADR 0001 (D8) introduces "measured on `<host>`" as this repo's vocabulary
for platform claims, in place of "supported", and gives it a two-part test:
`scripts/probe-env.sh` has run on the host, and the full test suite has run
there with a published skip count. What ADR 0001 does not say is what
"unmeasured" *obliges* — whether code for an unmeasured platform may ship at
all, and on what basis.

That gap became load-bearing during dist adoption (`docs/measure-dist-
2026-08-18.md`, `docs/measure-dist-conditions-2026-08-18.md`,
`.github/workflows/release.yml`'s `package` job). `aarch64-apple-darwin` is
one of ADR 0001's two measured targets, and `dist build` was confirmed to
succeed there on a real `macos-latest` runner — but the binary-equivalence
and generated-installer checks (proposal §10 conditions 2-3) were run only
on `ubuntu-latest`; nothing exercised the produced macOS archive's installer
or verified its binary matches an ordinary `cargo build --release` output on
that platform. Shipping a release with that gap needed an explicit answer
to whether "unmeasured" is a release blocker, and ADR 0001 alone does not
give one.

The owner ruled on this directly, 2026-08-18, in his own words: *"Measured
means validated. Unmeasured means the code is written and should be right
but isn't fully tested in a Mac environment. So a clean build, check, etc
is all we can and should do until we decide to measure, but that could be
whenever I decide to do it. It shouldn't block us."*

## Decision

**"Measured" means validated.** ADR 0001's existing test (probe-env run
plus a published-skip-count full suite run) is unchanged; this ADR adds no
new criteria to it.

**"Unmeasured" does not mean untrustworthy or blocking.** It means the code
was written to be correct on that platform and has not yet had ADR 0001's
validation applied there. The obligation an unmeasured platform carries is
narrower and mechanical: produce a clean build and a clean `cargo check` (or
the platform-appropriate equivalent, e.g. `dist build` actually succeeding)
for that target. That is what this repo can and should do without a
measurement session; it is not a substitute for one, and it must not be
described as one.

**Unmeasured status does not block shipping.** A release, a packaging step,
or a feature may proceed for an unmeasured platform once its build/check is
clean, carrying an explicit "unmeasured" label rather than a false
"measured" one and rather than being withheld. Deciding to actually measure
a given platform — running probe-env and the full suite there — remains the
owner's call, on his own schedule; its absence today is not itself a defect
to fix under deadline pressure.

**Applied to `aarch64-apple-darwin` in `release.yml`'s `package` job
specifically:** the job builds and packages the macOS target because a
clean `dist build` there is exactly the obligation this ADR describes. The
workflow's comments say precisely what is proven (the build itself, on a
real `macos-latest` runner) and what is not (binary equivalence, installer
correctness) — see `docs/measure-dist-2026-08-18.md` and `docs/measure-
dist-conditions-2026-08-18.md`'s own "what this does NOT show" sections.

## Alternatives considered

- **Treat "unmeasured" as a release blocker** — require every platform in a
  release's packaging matrix to have passed ADR 0001's full test first.
  Rejected: the owner's ruling is explicit that this would gate on a
  measurement session's timing (his own, discretionary schedule) rather than
  on code correctness, for no evidenced defect. It would also contradict
  ADR 0001's own D1/D8, which already treat "measured" as an earned label
  applied over time, not a precondition for a platform's code existing.
- **Leave ADR 0001 unclarified and let each workflow/PR improvise wording**
  for what "unmeasured" permits. Rejected: this is exactly the ambiguity
  that produced the dist-adoption question in the first place, and would
  recur at the next unmeasured-platform decision point.
- **Supersede ADR 0001 outright** rather than clarify it. Rejected: D1
  (targets), D8's two-part measurement test, and D10 (leave unsupported
  builds alone) are all unchanged by this ruling; only the previously
  unstated obligation attached to "unmeasured" is new. A clarifying ADR is
  the correct instrument per this directory's own README, which reserves
  a new superseding ADR for decisions that actually change.

## Consequences

- Workflow and documentation comments that describe an unmeasured platform
  must distinguish "build succeeded" from "validated" explicitly, rather
  than letting a passing CI job imply the stronger claim. `release.yml`'s
  `package` job comment is the first place this is applied.
- A future decision to measure `aarch64-apple-darwin` (or any other
  unmeasured platform) is a scheduling choice for the owner, not a
  correctness gate this repo is currently failing.
- This ADR does not itself measure anything; it only states what the
  vocabulary obliges. The macOS binary-equivalence and installer gaps named
  in `docs/measure-dist-conditions-2026-08-18.md` remain open until someone
  actually runs that measurement.

## Open questions

- No specific date or trigger is attached to when `aarch64-apple-darwin`
  moves from "clean build" to "measured" under ADR 0001's own test — the
  owner's ruling explicitly leaves this to his discretion ("whenever I
  decide to do it").
- Whether this same posture should be written into ADR 0001 itself as an
  amendment, rather than living as a separate clarifying ADR, was not
  decided here; this ADR takes the narrower, lower-risk path of clarifying
  without editing the original record's D1/D8/D10 text.
