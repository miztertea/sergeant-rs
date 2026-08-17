# Glossary

Terms fixed by the 2026-08-14 cross-platform decisions
(`docs/adr/0001-platform-targets-and-measurement-posture.md` through
`docs/adr/0004-cross-platform-development-constraints.md`), and by a
second, later interview the same day, the "grill-with-docs" round
(`docs/adr/0005-gating-becomes-a-dispatched-work.md` through
`docs/adr/0011-delete-the-dashboard.md`), and by a third interview on
2026-08-16, the ICM-R0 owner rulings
(`docs/adr/0013-icm-r0-owner-rulings.md`). Definitions here are the
vocabulary those ADRs use; read the ADR itself for the decision and its
rationale.

**Placement Ladder (PL).** `sergeant-rs-workspace/knowledge/evidence/reference/proposal-icm-r-procedure-authority.md`'s
eight-rung classification (PL-0 absorbed/obsolete through PL-7 engine gap)
answering "what is the lowest-authority, smallest-surface representation
that faithfully owns this behavior?" — applied to one source-cited
behavior unit at a time, stopping at the first rung that honestly holds.
Distinct from the pre-existing decomposition ladder in
`.sergeant/workflows/repo-to-icm/_config/icm-ladder.md`: PL extends it
with the driver/admission-boundary discriminator (Captain versus
stage-actor versus deterministic versus runtime; pre-Work versus in-Work
versus post-Work) the prior ladder lacked. Ratified as accepted vocabulary
by `docs/adr/0013-icm-r0-owner-rulings.md`'s decision 1.

**Bounded-Judgment Ladder (J).** The companion six-rung ladder (J5
governing constraint through J0 not-delegated/conflicting/risk-changing)
answering "what authority allows this actor to decide this material
question without returning to a human or higher authority?" A stage or
skill cites the first rung that actually resolves a material decision; a
J0 landing means stop and produce one precise question rather than guess.
`validate-and-ship/40-drive-gates`'s existing auto-fix/no-op/ask-user
finding split is the concrete precedent this ladder generalizes. Ratified
alongside PL by `docs/adr/0013-icm-r0-owner-rulings.md`'s decision 1.

**Measured target.** A platform (Linux, macOS, or Windows-via-WSL2 per
`docs/adr/0001-platform-targets-and-measurement-posture.md`'s D1) that has
earned the label "measured on `<host>`" by satisfying both of D8's
requirements: `scripts/probe-env.sh` has run there with its output
recorded in `docs/environments/<host>.md`, and the full test suite has run
there with a published skip count. While the project is alpha, no platform
is ever called "supported" — only "measured", and only once both
conditions hold.

**Platform fact.** A compile-time property of the host `sgt` is built for
— what `docs/adr/0002-platform-boundary-shape.md`'s D4 draws the boundary
around. Platform facts are exposed behind `#[cfg]`-selected modules with
injected-probe functions (D2, D3), never a runtime-dispatched trait,
because which platform a binary was built for does not vary at runtime the
way backend choice does. Docker's and the `claude` CLI's runtime behavior
are explicitly not platform facts — see backend capability, below.

**Backend capability.** A runtime-varying property of the adapter chosen
for a given Work via `--backend claude|fake|docker`
(`src/backend/mod.rs:669`'s `Backend` trait), subject to runtime
withdrawal when the installed harness doesn't support it. Docker Desktop's
macOS-specific bind-mount and uid semantics, and whatever `claude -p`
measurably does or doesn't do on a given host, are backend capabilities,
not platform facts — `docs/adr/0002-platform-boundary-shape.md`'s D4 keeps
them out of the platform boundary on purpose, because they're owned by the
adapter's own measure-first discipline (`LESSONS.md`'s L1), not by a
platform module speaking on the adapter's behalf.

**Durability promise.** What `docs/adr/0003-durability-promise-and-storage-preconditions.md`'s
D5 defines sergeant as actually guaranteeing: not that the daemon survives
the machine going away (sleep, shutdown, a WSL2 distro teardown, a kernel
panic are all the same undifferentiated class of event to this
architecture), but that the journal makes work **resumable** after it does.
The platform-relevant question this implies is never "did the daemon
survive" but "after an abrupt daemon death, does work actually resume, or
does it land in `blocked`" — which is exactly the question macOS currently
answers worse than Linux, per issue #18.

**Use tier / develop tier.** Vocabulary the 2026-08-14 interview named for
two audiences a measured target could in principle serve separately —
*develop tier* for a platform sergeant-rs itself is built and tested on,
*use tier* for one where a colleague only runs `sgt` — but the interview
record supplies only one fact about it, not a full definition: per D1, no
platform is currently use-only; every measured target is a develop-tier
target. The precise boundary between the two tiers (what would actually
qualify a platform as use-only) was not specified in the interview; treat
the gloss above as the terms' plain meaning, not a ratified definition.
Background, not part of the decision itself: `NORTH-STAR.md`'s Waves
section gates "stranger onboarding + prebuilt binary" behind "envelope +
dogfood round 2" — consistent with no use-only audience existing yet, but
not something D1 itself states.

**Advisory-locking-unreliable filesystem.** A filesystem where `flock`-style
advisory locking cannot be trusted to actually exclude a second holder —
the predicate `docs/adr/0003-durability-promise-and-storage-preconditions.md`'s
D6 generalizes to, rather than naming a specific path. The originating case
is WSL2's `drvfs` (a data dir cloned under `/mnt/c/...`, crossing into the
Windows filesystem where `flock` degrades along with git worktree
operations and file watching), but the predicate is written broadly enough
to also cover NFS and SMB shares, which have the same underlying problem.
`sgt init` and daemon start must refuse outright on such a filesystem,
because the architecture's one-owner `daemon.lock` invariant
(`docs/DEVELOPMENT.md`) depends on advisory locking actually holding.

**Skip count.** The number of tests a suite run skipped via `SKIPPED-ENV`
probe-gating (`docs/DEVELOPMENT.md`'s two-environment rule), published
alongside a green run as part of what makes a platform "measured" under
D8. Without a published count, a fully green run at four measured
environments could have silently skipped most of what actually exercises
each one — the count is what keeps "measured" honest.

**Gate Work.** A dispatched Work whose durable outcome is a shipping-gate
run, per `docs/adr/0005-gating-becomes-a-dispatched-work.md`'s D1. Because
the gate is itself a Work with its own isolated surface, it is the sole
owner of the branch it runs against for the duration of the run — the
same single-owner posture `docs/DEVELOPMENT.md`'s data-dir invariant
already enforces elsewhere, which is what let the old "an actor inside a
worktree never invokes the gate" rule dissolve instead of needing an
exception carved into it. Captain's role narrows to reading a gate Work's
findings and deciding, not driving the pipeline by hand.

**Harness passthrough.** `sgt <harness> -- <args>` (`sgt claude`,
`sgt codex`, `sgt opencode`, `sgt goose`), per
`docs/adr/0006-harness-passthrough.md`'s D2. `sgt` composes the actor
environment and binds the estate, then **exec**s into the harness process
— it never forks and supervises it. The exec-not-supervise boundary is
load-bearing, not incidental: a passthrough that grew a process table, a
pid file, or a restart policy would be the "reconstructed tmux-era
supervision" `NORTH-STAR.md`'s "Never" list already rules out. A human
surface (below), not something a workflow drives.

**Human surface.** A command whose audience is a person sitting at a
terminal, not an actor executing a Work — `sgt init`, `sgt doctor`, the
harness passthrough (`sgt claude` and its siblings,
`docs/adr/0006-harness-passthrough.md`), `sgt tui`, and the bare-`sgt`
homepage (both `docs/adr/0010-bare-sgt-is-a-homepage.md`'s D6). The
dashboard (`docs/adr/0011-delete-the-dashboard.md`) was considered and is
not one of them — the owner's named human surfaces are exactly this list,
and the dashboard's absence from it is part of why it was deleted rather
than kept.

**No-spawn set.** The verbs that must never auto-spawn a daemon just to
observe it, per `docs/adr/0009-auto-spawn-never-on-observation.md`'s D5:
`sgt doctor`, `sgt watch`, `sgt daemon stop` (ruled first for `watch` by
`sergeant-rs-workspace/knowledge/evidence/gauntlet/contracts/WATCH.md`'s R-WATCH-3), and now also `status`,
`work show`/`list`/`transcript`/`retained`, `work reap`'s unconfirmed
preview (its `--yes` disposal path still mutates and stays outside this
set), `analytics`, and the TUI. Auto-spawn survives only on verbs that
mutate durable state: `run`, `respond`, `retry`, `extend`, `cancel`, and
`work reap --yes`. The principle behind the set, stated first in
R-WATCH-3: "observation must not materialize the thing observed."
