# Glossary

Terms fixed by the 2026-08-14 cross-platform decisions
(`docs/adr/0001-platform-targets-and-measurement-posture.md` through
`docs/adr/0004-cross-platform-development-constraints.md`). Definitions
here are the vocabulary those ADRs use; read the ADR itself for the
decision and its rationale.

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
