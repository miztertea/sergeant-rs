# ADR 0004: Cross-platform development constraints

**Status:** Accepted, 2026-08-14.

## Context

Bringing macOS into the measured-target set (ADR 0001) means this repo's
own scripts and CI shape have to hold up there too, not just the Rust code
behind the platform boundary (ADR 0002). Two separate constraints came out
of the interview: what shell dialect the repo's scripts can rely on, and
how CI should be laid out now that a real macOS lane is coming.

## Decision

**Bash 3.2 is the script floor (D7).** macOS ships bash 3.2.57, frozen at
that version for licensing reasons (GPLv2 vs. GPLv3), and every script in
this repo's `scripts/` tree must stay compatible with it. Re-measured
2026-08-14, scoped to the repo's own maintained scripts under `scripts/`
(25 `.sh` files, `find scripts -name '*.sh' | wc -l`) rather than trusting
the interview's own figures: zero uses of associative arrays
(`declare -A`), zero `mapfile`/`readarray`, zero `${x,,}`/`${x^^}` case
conversion, zero `&>>`, and zero real bash negative array indices — the
handful of `[-1]`/`[-N]`-shaped matches that turn up (`scripts/demo.sh:309-311`,
`scripts/perf/s2-churn.sh:142`, `scripts/perf/common.sh:472-621`) are all
Python list indexing inside `python3 <<EOF` heredocs embedded in those
scripts, not bash syntax, and don't count against the bash floor. There is
exactly **one** bash-4.3+ dependency in the whole tree: `local -n` at
`scripts/perf/common.sh:56`, a nameref that bash 3.2 does not support —
this matches the interview's own figure exactly, down to the file and
line. The interview also decided to wire `shellcheck` into CI: it is
already installed on Cerberus (confirmed present at `/usr/bin/shellcheck`,
version 0.11.0, on this host) but is currently absent from
`.github/workflows/` (only `ci.yml` and `coverage.yml` exist there today,
neither runs shellcheck).

**CI shape (D9).** This repo is public, and standard GitHub-hosted
runners — including macOS and Windows — are free and unmetered for public
repos with no minute cap; the widely-known 10x (macOS) / 2x (Windows)
runner-minute multipliers apply only to private repos. Cost is therefore
not the constraint on this repo's CI; wall-clock is. Because matrix jobs
run concurrently, a matrix's wall-clock cost is `max(job)`, not
`sum(job)`, and the dominant term on every platform is the same one
already noted in `docs/DEVELOPMENT.md`: the bundled DuckDB C++ build
(~500 translation units, ~10 minutes cold), which makes build caching the
real lever for keeping any platform lane fast, not job count. The decided
shape: per push, run Linux `fmt`/`clippy`/`test` plus a cheap cross
`cargo check` against the macOS target — this is exactly what compensates
for ADR 0002's D2 negative consequence, that the macOS-only module isn't
compiled at all on a Linux dev host; on PR-to-main, nightly, and release,
run the full matrix across all measured targets.

## Alternatives considered

No alternative script-floor version (e.g., requiring a newer bash via
Homebrew on macOS contributors' machines, or rewriting the one `local -n`
site to avoid the nameref) is recorded as considered in the interview
beyond the floor itself — bash 3.2 compatibility is the decision, not one
candidate among several.

For CI shape, the interview's own record explicitly flags and corrects a
wrong claim rather than presenting a rejected alternative: an earlier
claim by the orchestrating session, that macOS runners bill at a 10x
multiplier for this repo, was **wrong** and was corrected by the owner —
that multiplier is real for GitHub-hosted runners in general but does not
apply here because this repo is public. The corrected fact (cost is not a
constraint) is what the decided per-push/PR-to-main split above is built
on, not a considered-and-rejected alternative in its own right.

## Consequences

D7's practical consequence is that `scripts/perf/common.sh:56`'s `local -n`
site is now a known, named incompatibility with the macOS-shipped bash —
this ADR does not fix it (scope limits below), only records it as the one
site that needs attention before a perf script can be measured to work
under macOS's bash. Wiring `shellcheck` into CI is hygiene on top of that,
not a version gate in itself: per the interview's own framing, a macOS CI
job actually running `/bin/bash` is the real enforcement of bash 3.2
compatibility; `shellcheck` catches shell bugs and portability smells
independent of that specific version floor.

D9's consequence is that the compensating cross `cargo check` lane named
in ADR 0002 has a concrete home — the per-push CI job — but is not yet
implemented; `.github/workflows/ci.yml` today runs only a single
`ubuntu-latest` job (`fmt`/`clippy`/`test`) with no macOS target check and
no separate PR-to-main/nightly/release matrix tier. This ADR records the
decided shape; building it — the macOS `cargo check` lane, the
PR-to-main/nightly/release matrix tier, and `shellcheck` wiring — is
separate, not-yet-filed implementation work, not something this ADR does.

## Open questions

None identified in the interview record beyond the implementation gap
already named above as a consequence, not an unresolved decision.
