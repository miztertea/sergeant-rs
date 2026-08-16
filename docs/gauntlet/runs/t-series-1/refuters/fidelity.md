# T-SERIES-1 — fidelity refuter

Axis: fidelity. Assigned file:
`docs/gauntlet/runs/t-series-1/critics/fidelity.md`. Four findings (F1–F4),
all warning/info severity, none claimed to invalidate a section's premise.
Re-verified every factual predicate independently — did not take the
critic's excerpts, line numbers, or `gh`/`git` output on trust — against
`reference/proposal-tui-t-series.md`, the real repository tree, `git log`/
`git show`/`git merge-base` on the actual objects cited, ADR 0009's full
text, and live GitHub state via `gh`. Also spot-checked a sample of the
critic's "found clean" section (crossterm/ratatui versions, the workflow
catalog count, the `Screen` enum, bare-`sgt` behavior, issue #26 and PR #111
GitHub state) as a fabrication check in the spirit of the FOUNDATION-1
refuter's caught fabricated-quote precedent — no fabrication found; every
one of those spot-checked claims holds.

## F1 — "Supersedes" citation points to a commit that never contained the predecessor

**Verdict: CONFIRMED, severity as-is (warning).**

Independently reran the critic's checks rather than trusting them:

- `git show a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6:reference/proposal-tui-t-series.md`
  fails with "exists on disk, but not in a5fb875…" — confirmed verbatim.
- `git log --all --oneline -- reference/proposal-tui-t-series.md` returns
  exactly two commits ever: `a9a25fa` (the predecessor) and `2000a22`
  ("T-SERIES-1: revised operator-cockpit proposal, gate lifted, gauntlet
  contract" — this proposal itself). `a5fb875` genuinely has no relationship
  to the file at all.
- `git merge-base --is-ancestor a9a25fa 242abe3` exits 0 — confirmed `a9a25fa`
  really is an ancestor of the proposal's own pinned audit revision, so the
  critic's proposed fix location is real and reachable.
- `a9a25fa`'s actual commit timestamp is 2026-08-11 15:28:30 — matches the
  critic's "roughly 14.5 hours later" claim relative to `a5fb875`'s
  2026-08-11 00:58:23 (14.5 hours, exact).

No factual gap in the finding. On severity: the contract's fidelity axis
description explicitly names "check each disposition against that
predecessor's actual text rather than trusting the label" as this axis's
own method — a broken link at the exact citation meant to enable that check
is squarely on-axis, arguing against downgrading to info. Against
upgrading: the critic themself performed the correction (locating `a9a25fa`
via a two-commit file history) in minutes, and every §23 disposition in the
report was in fact checked against the real predecessor text despite the
broken pointer — so the defect never actually blocked verification in
practice, only misdirects a reader who clicks the header's GitHub link.
Warning is the right severity: real, reader-facing, mechanically fixable,
zero normative impact.

## F2 — §12.3's Doctor check list names checks that don't exist and omits two that do

**Verdict: CONFIRMED, severity as-is (warning).**

Independently grepped every `Check::ok`/`Check::warn`/`Check::fail` call
site in `src/cli.rs`'s doctor module rather than trusting the critic's list.
The real check names, straight from source: `git` (1978,1982,1987),
`claude` (2008,2010), `environment` (2051,2058), `data_dir`
(2297,2265,2284,2302), `docker` (2139,2144), `journal` (2358,2369,2384,2395),
`projection` (2437,2441), `daemon` (2464,2473,2491,2500,2514),
`permission_mode` (1810,1811,1829,1831,1832,1835,1836), `estate`
(1871,1872,1881,1882,1889,1895,1896,1956,1957,1965), `disk_pressure`
(2191,2192,2202,2203,2211,2212,2213) — eleven checks total, exactly matching
the critic's list. Grepped separately for `"installation"` and `"profiles"`
as check-name string literals: zero hits anywhere in `src/cli.rs`. Read the
proposal's actual §12.3 text at line 1153: "Current checks include
installation, environment, data directory, Docker, journal, projection,
daemon, estate, profiles, and disk pressure." — quote is accurate, not
mischaracterized. `git` and `claude` are absent from that sentence;
`permission_mode` doesn't appear under any name.

Finding fully holds. On severity: the critic's own note that Decision
T2-45 pins Health to render `DoctorReport.checks` unmodified — verified by
reading that decision in the proposal, present as cited — means this cannot
regress into a build defect; it's purely a reader-expectation defect in
prose. That argues against upgrading past warning. It does not argue for
downgrading to info either: unlike F4, this naming isn't inherited from an
existing codebase imprecision (no `installation`/`profiles` string appears
anywhere in `src/cli.rs`) — it looks like invented-for-the-proposal
labeling rather than an echoed error, which is a materially different (more
attributable to the proposal itself) class of mistake than F4's. Warning
stands.

## F3 — §23.2 labels the predecessor's silence on repo/group as a "previous decision" being revised

**Verdict: CONFIRMED, severity as-is (warning).**

Extracted the predecessor at `a9a25fa` fresh via `git show
a9a25fa:reference/proposal-tui-t-series.md` (1943 lines, matching the
critic's count) and grepped it independently for `repo|group|estate`
case-insensitively. Every "repo" hit is the generic noun "repository" (the
daemon, repository-owned workflows, repository files) — none refers to a
distinct repo/group management screen or decision. Confirmed the specific
citations: line 73 area has "This proposal makes bare `sgt` the primary
interactive surface" and the exact nav block; Decision T-21 at that location
pins top nav to exactly `Home Fleet Workflows` with an explicit list of
what's *not* there (no Work tab, no Journal tab, no System tab) — Estate is
not even named as a considered-and-excluded option, consistent with the
critic's "did not address estate administration at all" reading rather than
"considered it and put it outside." Confirmed the contrast case: Decision
T-18 at predecessor line 417 reads verbatim "Doctor remains a CLI-only
diagnostic. The TUI has a small connection overlay, not a System dashboard"
— a real, explicit, named ruling, unlike the repo/group row. Confirmed the
proposal's actual §23.2 row text at line 2151: "Repo/group outside TUI |
Full current lifecycle included through extract-on-contact" — quote
accurate.

Finding fully holds; the critic's own contrast (T-18 as a genuine revision
example in the same table) is real and sharpens the point rather than being
a rhetorical flourish. On severity: this sits exactly on the axis's
explicitly named method (grade §23 disposition labels against actual
predecessor text, not the label), which argues it deserves full attention,
but the critic's own assessment that the underlying substantive decision
(Estate as first-class, full lifecycle) is correctly and independently
scoped elsewhere (§12, §16.2) means no decision is wrong — only a table's
provenance framing. Same category and consequence as F1: a mislabeled
citation/provenance claim, mechanically fixable, zero normative drift.
Warning is consistent with F1's calibration and correct.

## F4 — §5.5 credits ADR 0009 with moving two surfaces that were already in the no-spawn set before it

**Verdict: CONFIRMED, severity as-is (info).**

Read ADR 0009 in full independently. Its Decision section verbatim: "`status`,
`work show`/`list`/`transcript`, `analytics`, and the TUI join `sgt doctor`,
`sgt watch`, and `sgt daemon stop` in the no-spawn set" — confirms Doctor and
Watch are the pre-existing set being *joined*, not moved by this ADR. Its
Context section states `docs/gauntlet/contracts/WATCH.md`'s R-WATCH-3 (Watch's
no-auto-spawn ruling) was "owner-ruled 2026-08-13," one day before ADR 0009's
own "Accepted, 2026-08-14" status line — independently confirmed by reading
`docs/gauntlet/contracts/WATCH.md:71`, which dates R-WATCH-3 to exactly
2026-08-13. So the chronology the critic relies on is real, not asserted.
Read the proposal's actual §5.5 text at line 416: "ADR 0009 moved `status`,
Work reads, analytics, Watch, Doctor, and TUI into the no-auto-spawn set." —
quote accurate; Doctor and Watch are indeed listed as if moved by ADR 0009
alongside the four surfaces it actually did move. Grepped `src/cli.rs` for
"ADR 0009" and found line 1341's doc comment: "`analytics`, and `tui` —
every verb ADR 0009 moved into the no-spawn set" — confirms the critic's
claim that this exact imprecision already exists in the repository's own
code, independent of the proposal.

Finding holds with no gap. On severity: agree with info, and would resist
any upgrade — the critic's own "does the section survive" check (Decision
T2-16's operative dependency is "all six surfaces are no-spawn today," which
is true regardless of attribution) is correct on inspection: nothing built
on this sentence cares which ADR gets credit for which subset. The fact
that the exact same imprecision is already sitting in merged, shipped code
comments is a real mitigating fact, not critic editorializing — it means a
reader who cross-checks the proposal against the codebase (as this axis
exists to make them do) would find the *code* agrees with the proposal's
phrasing, so the defect is unusually low-stakes even among this report's
already-low-severity findings. Info, not warning, is correct.

## Summary

| Finding | Verdict | Severity change |
|---|---|---|
| F1 | CONFIRMED | none (warning, unchanged) |
| F2 | CONFIRMED | none (warning, unchanged) |
| F3 | CONFIRMED | none (warning, unchanged) |
| F4 | CONFIRMED | none (info, unchanged) |

All four findings survive independent, adversarial re-verification against
primary sources — none were mischaracterized, none rest on a citation that
doesn't check out, and none had their severity miscalibrated in either
direction. No fabricated citation or invented contract quote was found in
this report (the failure mode the FOUNDATION-1 fidelity refuter caught in
its own F2). Spot-checks of the critic's "found clean" section (crossterm/
ratatui dependency graph, the 23-entry workflow catalog, the `Screen` enum,
bare-`sgt` routing, issue #26 and PR #111's live GitHub state) all
independently confirmed accurate. Consistent with the critic's own
conclusion: no finding here invalidates any section's premise; all four are
mechanically fixable prose/citation accuracy defects, not scope or
consequence errors.
