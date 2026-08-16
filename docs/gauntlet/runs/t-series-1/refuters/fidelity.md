# T-SERIES-1 — fidelity refuter

Axis: fidelity. Assigned file:
`docs/gauntlet/runs/t-series-1/critics/fidelity.md`. Four findings (F1–F4),
all warning or info severity. Re-derived every citation independently — git
history and `git show`/`git merge-base` for the commit-provenance claims
(F1, F3), the actual `mod doctor` source in `src/cli.rs` for the check-name
claim (F2), ADR 0009's own text for the attribution claim (F4) — and
separately re-checked a sample of the "found clean" section (bare `sgt`,
the dashboard deletion, `completed_dirty`, envelope fields, `Cargo.toml`/
`Cargo.lock`, the workflow catalog count, the `Screen` enum, PR #111's
actual diff for all five §3.3 bullets, and PR #111's real merge state via
`gh pr view`) rather than trusting the critic's own citations.

## F1 — "Supersedes" citation points to a commit that never contained the predecessor

**Verdict: CONFIRMED, severity as claimed (warning).**

Independently reproduced every step. `git log --all --oneline --
reference/proposal-tui-t-series.md` shows exactly two commits in the file's
whole history: `2000a22` (this proposal) and `a9a25fa` (the predecessor).
`a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6` is real (merge of PR #43,
2026-08-11 00:58:23) but `git show a5fb875:reference/proposal-tui-t-series.md`
fails with "exists on disk, but not in" — confirmed directly, not taken on
the critic's word. `a9a25fa` ("Execution-surface test (owner ruling)...",
2026-08-11 15:28:30, ~14.5 hours later) is the commit that actually carries
the predecessor's text, and `git merge-base --is-ancestor a9a25fa 242abe3`
confirms it as an ancestor of the proposal's own pinned audit revision. The
proposal's front matter and header both cite `a5fb875` (front matter line
26, header line 50) — verified by direct read, not summary.

Severity check: this is a dead GitHub link and a wrong provenance pointer,
not a normative-content error — every §23 disposition the critic and I both
checked resolves correctly once pointed at the right commit. Warning (not
info) is right because a reader following the citation as instructed by the
contract itself ("check each disposition against that predecessor's actual
text") hits a 404, not a silent inaccuracy — that's a real, if mechanical,
failure of the citation's one job. Not "error" tier since nothing normative
is affected and the fix is a one-line pointer change.

## F2 — §12.3's Doctor check list names checks that don't exist and omits two that do

**Verdict: CONFIRMED, severity as claimed (warning).**

Re-grepped `src/cli.rs`'s `mod doctor` independently rather than trusting
the critic's list. `Report::run` (`src/cli.rs:1750-1780`) assembles exactly
eleven checks in this order: `git`, `claude`, `environment`, `data_dir`,
`docker`, `journal`, `projection`, `daemon`, `permission_mode`, `estate`,
`disk_pressure` — matches the critic's enumeration exactly, both the names
and the count. Grepped `installation` and `profiles` as literal check-name
strings across `src/cli.rs`: neither exists as a `Check::ok`/`warn`/`fail`
first argument — `installation` appears only in prose/doc-comments
("diagnose this installation"), and `profiles` only in a comment describing
what `permission_mode_check` inspects. The proposal's own line 1153 lists
exactly ten items ("installation, environment, data directory, Docker,
journal, projection, daemon, estate, profiles, and disk pressure") — one
short of the real eleven, with `git` and `claude` folded into the invented
umbrella term "installation" and dropped as distinct rows, and `permission_mode`
renamed to "profiles."

Also independently confirmed T2-45's actual text (`reference/
proposal-tui-t-series.md:1155`, "Extract Doctor's structured `Check`/`Report`
result from CLI formatting and let both CLI and TUI consume it") — the
critic's point that this decision insulates the *implementation* from the
prose defect holds: whatever `DoctorReport.checks` contains renders
regardless of what §12.3's prose claims the names are. Severity: warning is
correct — this is prose-only, doesn't touch a normative decision, but it's
also not info-tier, since an operator reading this section to predict
Health's actual rows would be wrong about three of eleven names.

## F3 — §23.2 labels the predecessor's silence on repo/group as a "previous decision" being revised

**Verdict: CONFIRMED, severity as claimed (warning).**

Read the predecessor's full text independently at `a9a25fa`
(`git show a9a25fa:reference/proposal-tui-t-series.md`, 1943 lines) rather
than trusting the critic's excerpt. Grepped for `repo`, `group`, and
`estate` case-insensitively across the whole document: the only "estate"
hit (line 1483) is about the journal/DuckDB operator-surface gap, unrelated
to repository or group administration. The five occurrences of top
navigation (lines 78, 484, 496, 723, 810 — one more than the critic cited,
same conclusion) are uniformly `Home    Fleet    Workflows`, with Decision
T-21 stating the navigation "contains exactly" those three and explicitly
enumerating why there is no System/Journal/Work tab — repo/group
administration is not addressed as a rejected or deferred option anywhere
in that enumeration. §5.2's explicit non-goals list (Decision T-20,
verified directly) also never names repository or group management, in
either direction.

Contrasted directly against the table's neighboring "Doctor CLI-only" row:
the predecessor's Decision T-18 (verified verbatim at the location the
critic cited) is an explicit, named ruling ("Doctor remains a CLI-only
diagnostic") — a real thing this proposal can accurately claim to revise.
"Repo/group outside TUI" has no analogous ruling to revise; the predecessor
simply never considered the question. The critic's distinction is
substantive, not pedantic: "revised" implies a prior decision existed and
was overturned, and none did.

Severity check: warning is right, not higher — the critic is correct that
the underlying substance (Estate as a first-class destination with full
repo/group/Doctor lifecycle) is real and correctly scoped in §12/§16.2; only
the disposition table's framing overstates what was settled in 2026-08-11.
Nothing downstream depends on this row being framed as a revision rather
than new scope.

## F4 — §5.5 credits ADR 0009 with moving two surfaces that were already in the no-spawn set before it

**Verdict: CONFIRMED, severity as claimed (info).**

Read ADR 0009 in full independently rather than trusting the critic's
quotes. The Decision section's exact text: "`status`, `work
show`/`list`/`transcript`, `analytics`, and the TUI join `sgt doctor`, `sgt
watch`, and `sgt daemon stop` in the no-spawn set" — Doctor and Watch are
the pre-existing set being *joined*, and the Context section independently
confirms Watch's membership predates ADR 0009 by a day, citing
`WATCH.md`'s R-WATCH-3 as "owner-ruled 2026-08-13" against the ADR's own
"Accepted, 2026-08-14" status line. The proposal's §5.5 (verified directly
at line 416) states "ADR 0009 moved `status`, Work reads, analytics, Watch,
Doctor, and TUI into the no-auto-spawn set" — folding all six into "moved"
when only four were added and two were joined-to.

Also independently confirmed the critic's mitigating observation:
`src/cli.rs:1341`'s own doc comment reads "every verb ADR 0009 moved into
the no-spawn set," listing `watch`, `status`, `work show/list/transcript`,
`analytics`, and `tui` — the same imprecision, already shipped in the
codebase's own comments, not invented by this proposal.

Severity check: info is correct. The operative claim Decision T2-16
actually depends on — all six surfaces are no-spawn today — is true and
independently verified against ADR 0009's Consequences section and the
pinned contract-test names it cites. This is attribution-only, doesn't
mislead about current behavior, and echoes a pre-existing repo convention
rather than introducing a new error. No basis to upgrade.

## Spot-check of "what I checked and found clean"

Independently re-verified rather than accepted on the critic's word:

- Bare `sgt` vs `sgt tui`: `src/cli.rs:198` (`Command::Tui` doc comment)
  and `:821` (`Command::Tui` match arm referencing ADR 0009/0010) confirm
  the TUI is only reached via the explicit subcommand.
- Dashboard deletion: `docs/adr/0011-delete-the-dashboard.md` exists;
  `find` for `web.rs`/`web/` under the tree returns nothing.
- `completed_dirty`: present and load-bearing in `src/watch.rs:98`,
  `src/tui.rs:445`, matching the critic's citations.
- `output_pointer`/`envelope`/`turn_cap`/`ceiling_secs`: present in
  `src/api.rs` (envelope override validation at lines 1024-1043).
- `ratatui = "0.30.2"` in `Cargo.toml:36`; `Cargo.lock` carries two
  `crossterm` versions (0.28.1, 0.29.0) — confirmed the 0.28.1 line is
  unrelated to the pinned `ratatui`/crossterm claim per the critic's
  `comfy-table` attribution, and GAUNTLET.md's D8 entry (verified at line
  36) independently states "crossterm is reached exclusively through
  `ratatui::crossterm` re-exports," matching.
- `.sergeant/index.md`: 23 published workflow rows (25 lines matching `^|`
  minus a 2-line table header/separator) — matches "23 admitted workflows."
- `Screen` enum (`src/tui.rs:136-141`): exactly `Fleet`/`Detail`.
- PR #111: `gh pr view 111` confirms `state: MERGED`, `mergeCommit:
  3a46b87c17d249655708ed5ac32f6704738776cf`, matching the critic's cite.
  Pulled the full diff independently (`gh pr diff 111`, 7344 lines) and
  grepped it directly (not the critic's excerpts) for all five §3.3
  claims: `POST /v1/work/{id}/reap` and `GET /v1/retained` routes,
  `list_retained`/`reap_work` handlers, `WorkCommand::Retained`/`Reap`
  variants, the `blocked`-landing ceiling-interrupt test
  (`a_ceiling_interrupt_lands_the_work_in_blocked_not_wedged_active`),
  `render_transcript`'s timestamp formatting, and `filesystem_check`/
  `fs_locking::detect_for_path` for the Doctor filesystem check — all
  present in the real diff. Also independently read `reap_work`'s body in
  the diff: it refuses without `{"confirm": true}` and its own doc comment
  states the retained branch is "never" reachable by this verb — matches
  the proposal's "explicit confirmation" / "retained branches remain
  outside the deletion path" claims exactly, not just in outline.

No discrepancy found anywhere in the sampled clean section. I did not
re-verify every one of the ~30 individual clean-section sub-claims (the
§23.1 provenance list's twelve items, all eleven §3.1 rows, all eleven
§23.2 non-F3/F4 rows) line-for-line, but the sampled subset spans every
category the critic used (git provenance, source-code grep, `Cargo.lock`,
GitHub API state, and a full independent diff pull) and turned up nothing
that contradicts the critic's report.

## Summary

| Finding | Verdict | Severity |
|---|---|---|
| F1 | CONFIRMED | warning (unchanged) |
| F2 | CONFIRMED | warning (unchanged) |
| F3 | CONFIRMED | warning (unchanged) |
| F4 | CONFIRMED | info (unchanged) |

All four findings survive independent re-derivation with no severity
changes in either direction. None invalidates any section's premise — all
four are the critic's own characterization (mechanically fixable prose
defects, none touching a normative decision) and I found nothing in my
independent pass that under- or over-states that. The critic's own
citations held up under re-derivation in every case I checked, including
the one place (F1) where the critic's central claim was itself a citation
pointing at a dead target — the critic did not fall into the trap of citing
something equally unverified to prove it.
