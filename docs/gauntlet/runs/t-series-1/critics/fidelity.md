# T-SERIES-1 — fidelity critic

Axis: does the proposal say what the repository actually contains and
decided, and *only* that? Every claim about the shipped MVP, WATCH, Estate,
`completed_dirty`, the dashboard deletion, and the integration branch behind
PR #111 was checked directly against source (`src/`, `tests/`, `Cargo.toml`/
`Cargo.lock`, `GAUNTLET.md`, ADRs 0009–0011, `docs/gauntlet/contracts/
WATCH.md`) and against GitHub (`gh pr view/diff 111`, `gh issue view`), not
against the proposal's own prose. Every §23 (Dispositions) label was checked
against the 2026-08-11 predecessor's actual text, read in full at the commit
that actually contains it (`a9a25fa68938323d9585edc687fbf0e965084c2e`,
1943 lines), not summarized from this proposal.

## Findings

### F1 — The proposal's own "Supersedes" citation points to a commit that never contained the predecessor

- **severity:** warning
- **section:** front matter (`supersedes.revision`) and the header's
  **Supersedes** line
- **claim/text at issue:** Front matter: `supersedes: path:
  reference/proposal-tui-t-series.md, revision:
  a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`. Header: "**Supersedes:**
  [`reference/proposal-tui-t-series.md@a5fb875`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/reference/proposal-tui-t-series.md)".
- **what I checked:** `git show a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6:reference/proposal-tui-t-series.md`
  and the equivalent GitHub contents API call; `git log --all --oneline --
  reference/proposal-tui-t-series.md`, which shows the file has exactly two
  commits in its whole history.
- **what I found:** `a5fb875` is a real commit (merge of PR #43,
  "ideaos-agent-prototype-gauntlet," 2026-08-11 00:58) but does not contain
  `reference/proposal-tui-t-series.md` at all — `git show` fails with
  "exists on disk, but not in …" and the GitHub contents API returns a 404.
  It is the predecessor's own `audit_revision` field
  (`/tmp/predecessor.md` line 26: `audit_revision:
  a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6` — the state of the repo the old
  proposal was written *against*), copied forward into `supersedes.revision`
  instead of the commit that actually contains the old proposal's text. The
  predecessor document was itself committed roughly 14.5 hours later, at
  `a9a25fa68938323d9585edc687fbf0e965084c2e` ("Execution-surface test (owner
  ruling): workflow vs CLI surface vs operator skill," 2026-08-11 15:28).
  This is exactly the citation the contract directs a critic to follow ("check
  each disposition against that predecessor's actual text") — as written, it
  does not resolve, and a reader clicking the header's GitHub link gets a
  live 404.
- **does the section survive the correction:** yes. The predecessor is
  locatable at `a9a25fa` (or via its live path at the proposal's own pinned
  `242abe3`, since `a9a25fa` is an ancestor of `242abe3` — confirmed with
  `git merge-base --is-ancestor`), and every §23 disposition I checked
  against that real text holds up except F3 below. The fix is mechanical:
  point `supersedes.revision` and the header link at `a9a25fa`.

### F2 — §12.3's Doctor check list names checks that do not exist and omits two that do

- **severity:** warning
- **section:** §12.3 (Health)
- **claim/text at issue:** "Current checks include installation,
  environment, data directory, Docker, journal, projection, daemon, estate,
  profiles, and disk pressure."
- **what I checked:** every `Check::ok`/`Check::warn`/`Check::fail` call
  site in `src/cli.rs`'s `mod doctor` (the module `Report::run` assembles
  from), grepping each check function and its literal name string.
- **what I found:** the eleven real check names, straight from the source,
  are `git`, `claude`, `environment`, `data_dir`, `docker`, `journal`,
  `projection`, `daemon`, `permission_mode`, `estate`, `disk_pressure`.
  There is no check named `installation` — the two checks that plausibly
  correspond to that umbrella word, `git` and `claude`, are dropped from
  the proposal's list entirely. There is no check named `profiles`; the
  real check that covers profile-adjacent state is `permission_mode`. Since
  Decision T2-45 commits Health to rendering the shared `DoctorReport`
  unmodified ("Extract Doctor's structured `Check`/`Report` result… let
  both CLI and TUI consume it"), the check names this prose promises are
  not the names an operator will actually see in either surface.
- **does the section survive the correction:** yes. T2-45's mechanism
  (render whatever `DoctorReport.checks` contains, not a hand-picked list)
  means the implementation itself is unaffected by what this prose calls
  the checks — this is a proposal-text accuracy defect, not a scope or
  behavior error. An operator reading §12.3 to predict what Health shows
  would expect `installation` and `profiles` rows and get `git`, `claude`,
  and `permission_mode` instead.

### F3 — §23.2 labels the predecessor's silence on repo/group as a "previous decision" being revised

- **severity:** warning
- **section:** §23.2 (Revised), row "Repo/group outside TUI"
- **claim/text at issue:** the Revised table's "Previous decision" column
  states "Repo/group outside TUI," which the new proposal now revises to
  "Full current lifecycle included through extract-on-contact."
- **what I checked:** the predecessor's full text (`a9a25fa`, all 1943
  lines) for every mention of `repo`, `group`, or `Estate` — including its
  explicit non-goals list and its top-level navigation.
- **what I found:** zero mentions. The predecessor's top nav is exactly
  `Home    Fleet    Workflows` (confirmed at four separate places in the
  document, e.g. lines 78, 484, 496, 723) — no Estate tab, no repo/group
  screen, and its non-goals enumeration (journal search, host metrics,
  OpenTelemetry, mouse, web redesign, graph canvas, plugin framework,
  archival semantics, issues #26/#45) never names repository or group
  management either as excluded or in scope. The predecessor did not
  address estate administration at all — it is not that the predecessor
  decided repo/group belonged "outside the TUI" and this proposal now
  overturns that call; there was no ruling on the question in 2026-08-11.
  Contrast this row with a genuine revision in the same table — "Doctor
  CLI-only" — which the predecessor states as an explicit, named decision
  (T-18, R1: "Doctor remains a CLI-only diagnostic. The TUI has a small
  connection overlay, not a System dashboard," line 417) that this proposal
  can accurately say it revises.
- **does the section survive the correction:** yes. The substantive
  change — Estate becoming a first-class destination with full repo/group/
  Doctor lifecycle — is real, current, and correctly scoped elsewhere in
  the proposal (§12, §16.2). Only the disposition-table framing overstates
  what 2026-08-11 actually settled: this row belongs under "new scope the
  predecessor never addressed," not "revised decision." Fixable without
  touching any normative content.

### F4 — §5.5 credits ADR 0009 with moving two surfaces that were already in the no-spawn set before it

- **severity:** info
- **section:** §5.5 (Observation never materializes the daemon)
- **claim/text at issue:** "ADR 0009 moved `status`, Work reads, analytics,
  Watch, Doctor, and TUI into the no-auto-spawn set."
- **what I checked:** ADR 0009's Decision and Context sections verbatim
  against this sentence.
- **what I found:** ADR 0009's own Decision text is "`status`, `work
  show`/`list`/`transcript`, `analytics`, and the TUI **join** `sgt
  doctor`, `sgt watch`, and `sgt daemon stop` **in** the no-spawn set" —
  Doctor and Watch are the pre-existing set being joined, not surfaces this
  ADR moved. Its Context section is explicit that Watch's membership
  predates it by a day: "`docs/gauntlet/contracts/WATCH.md`'s R-WATCH-3
  already ruled this principle for one verb… owner-ruled 2026-08-13…" while
  ADR 0009 itself is "Accepted, 2026-08-14." So ADR 0009 is the ruling that
  added four surfaces to a set Doctor and Watch already belonged to, not
  the ruling that moved all six. Notably, this exact imprecision already
  exists in the repository's own code: `src/cli.rs:1341`'s doc comment
  reads "`analytics`, and `tui` — every verb ADR 0009 moved into the
  no-spawn set" — the proposal's phrasing echoes an existing codebase
  comment rather than inventing a new error.
- **does the section survive the correction:** yes, cleanly. The operative
  claim — all six surfaces are no-spawn today, which is what Decision
  T2-16 depends on — is correct and independently verified. This is an
  attribution imprecision, not a wrong end state, and it isn't even novel
  to this proposal.

## What I checked and found clean

**§3.1's stale-assumption table**, row by row, against source at the
proposal's own pinned audit revision (`242abe3`) and `gh issue view` state:

- Bare `sgt` opens the TUI → confirmed false today: `src/cli.rs`'s
  `Command::Tui` is explicit (`src/cli.rs:198,821`), bare `sgt` calls
  `print_homepage` — matches ADR 0010 and `GAUNTLET.md`'s D10 entry.
- Web disposition → confirmed: `docs/adr/0011-delete-the-dashboard.md`
  records outright deletion (D7); no `src/web.rs`, `web/`, or `sgt web`
  verb exist in the tree — the dashboard is fully gone, not
  disabled-and-retained (which was the predecessor's own actual
  2026-08-11 position, its Decision T-50).
- #11/#16/#26 → the predecessor's own §16.1–16.3 (`/tmp/predecessor.md:
  1449-1463`) shows it owned and closed #11 and #16 but explicitly did
  **not** claim #26 ("does not claim to close #26 without its specific …
  fix"); all three issues are `CLOSED` per `gh issue view` (26 closed
  2026-08-14, before this proposal's 2026-08-15 timestamp) — the table's
  "current fact" column adds #26 as newly fixed without contradicting the
  predecessor's own exclusion of it.
- Thread from event history vs. dedicated transcript → confirmed;
  `GET /v1/work/{id}/transcript` and `work_transcript` exist in
  `src/api.rs` (lines 392, 1777); predecessor has no transcript endpoint.
- No output/envelope UI → confirmed; predecessor never mentions an output
  pointer or turn-envelope UI field; `src/api.rs` has both
  (`output_pointer`, `envelope`/`turn_cap`/`ceiling_secs`).
- submit/respond/retry/cancel vs. extend → confirmed; predecessor has zero
  matches for "extend" as a mutation verb; `src/cli.rs` has
  `Command::Extend`, distinct from retry.
- completed only vs. `completed_dirty` → confirmed; predecessor has zero
  mentions of `completed_dirty`; it is real and current (`src/api.rs:1504`,
  `src/tui.rs:445,1630+`, `src/watch.rs:98`).
- No headless return path vs. `sgt watch` shipped → confirmed;
  `docs/gauntlet/contracts/WATCH.md` is adjudicated, `src/watch.rs` exists.
- Estate outside TUI discussion vs. mature repo/group/Doctor CLI →
  confirmed; `src/cli.rs` has full `RepoCommand`/`GroupCommand` lifecycles
  (this is the same underlying gap F3 flags as mislabeled elsewhere).

**§23.1 (Adopted from the previous proposal):** all twelve items —
Work-centered Home/Fleet/Workflows, canonical Work surface, Attention
drawer, deliberate multiline composer, local slash palette, workflow
chooser/reference, endpoint-backed workflow catalog, ordinal workflow rail,
Work-local Evidence and graph, separate Journal proposal, API invalidation
discipline, `TestBackend` semantic testing — independently traced to a
named predecessor decision (canonical Work surface at line 97; Attention
drawer at line 87; slash palette at §7.3; ordinal rail at Decision T-44,
"never a percentage gauge"; separate Journal at Decision T-19; invalidation
discipline at line 300; `TestBackend` at Decision T-54). No invented
provenance.

**§23.2 (Revised), remaining rows beyond F3/F4:** "Bare `sgt` is the TUI"
(predecessor line 73: "This proposal makes bare `sgt` the primary
interactive surface"), "Doctor CLI-only" (predecessor Decision T-18,
verbatim above), "Local custom editor" (predecessor line 1377's
local-editor-vs-vetted-dependency framing, which already names the
override condition this proposal exercises), "Submit/respond/retry/cancel"
(predecessor's exact mutation list, no extend), "Event-derived
conversation" (predecessor Decision T-14: "a pure presentation fold over
known event kinds"), "no output/envelope UI" and "completed only" (both
absent from predecessor, confirmed above) — all check out as genuine
revisions of real predecessor positions.

**PR #111's actually-diffed content**, checked with `gh pr diff 111`
against §3.3's five bullets — all five confirmed present in the real diff:

- ceiling interruption lands in `blocked` (closes #90): the PR's own body
  and `src/cli.rs`'s `Extend` doc-comment diff both describe this.
- transcript timestamps (closes #96): `render_transcript` in `src/cli.rs`
  adds `[{ts}] {label}` formatting, reading the journal's existing `"ts"`
  field, not deriving one.
- Doctor filesystem-reliability check (closes #85):
  `src/platform/fs_locking.rs` diff adds `sgt doctor`'s `filesystem` row.
- `GET /v1/retained` and `sgt work retained`: present verbatim
  (`list_retained` route/handler, `WorkCommand::Retained`).
- `POST /v1/work/{id}/reap` and `sgt work reap --yes`: present verbatim
  (`reap_work`, `WorkCommand::Reap { id, yes }`), refuses without
  confirmation, and never touches the retained branch — matching the
  proposal's "explicit confirmation" / "retained branches remain outside
  the deletion path" claims exactly.

None of these four surfaces (`GET /v1/retained`, `sgt work retained`,
`POST /v1/work/{id}/reap`, `sgt work reap`) exist in `src/api.rs`/
`src/cli.rs` at the proposal's own pinned main revision (`242abe3`) —
grepping `retained`/`reap` there turns up only unrelated prose ("retained
branch," "test rig … reap_daemons"), confirming these genuinely are
PR #111-only candidate surfaces, not already-shipped main behavior the
proposal is hiding behind a false conditional.

**Other verified-accurate facts:** `ratatui = "0.30.2"` (`Cargo.toml`);
Crossterm is reached only through `ratatui-crossterm`'s own dependency on
`crossterm 0.29.0` (`Cargo.lock`), matching both the §3.2 claim and the
0.29.0 Crossterm docs cited in §8.8/§24.3 — a second, unrelated `crossterm
0.28.1` in the lockfile is pulled in by `comfy-table`, not the TUI stack,
and `GAUNTLET.md`'s D8 deviation-register entry independently confirms
"crossterm is reached exclusively through `ratatui::crossterm`
re-exports." The workflow root catalog lists exactly 23 published entries
(`.sergeant/index.md`), matching §11.1/§3.2's "23 admitted workflows"
precisely. The current TUI's `Screen` enum (`src/tui.rs:136`) is exactly
`Fleet`/`Detail`, matching §3.2's "the TUI still has only Fleet and Detail
screens." WATCH's real state set is the six states §14 lists
(`needs_input, waiting, blocked, failed, completed, canceled`, excluding
`pending`/`active`) per `WATCH.md`'s R-WATCH-1 ruling, and §14 correctly
labels its own attention-drawer bucketing of `completed_dirty` as the
cockpit's own choice rather than something WATCH itself does ("Watch does
not treat `pending` or `active` as notice-producing states, but the
cockpit may show them under Running because the drawer is a fleet view,
not the Watch protocol itself" — correctly hedged, not a fidelity issue).
The integration branch's cited head commit `251a6f1` matches both the
local `integration/path-to-mac-2026-08-15` branch tip and PR #111's real
merge-commit second parent on GitHub. The executive summary's "a
successful walk-away ship gate" traces to `GAUNTLET.md`'s real 2026-08-13
MVP close-out entry ("ship gate 01KZY9ZFTE … passed," "ship gate PASS, #19
closed") — a different, already-shipped milestone from PR #111's own
sprint's later-discovered `#120` gate defect, and the proposal correctly
keeps the two separate rather than conflating them (§19.12 names the PR
#111 gate defect specifically and by number without walking back the
MVP's own earlier, independently passed gate).

No finding invalidates any section's premise. F1–F3 are warning severity
(a broken/misleading citation, a mismatched Doctor check-name list, and an
overstated disposition label) — all mechanically fixable without touching
any normative decision. F4 is info severity and echoes an imprecision
already present in the repository's own code comments rather than
inventing one. Every other consumed-surface claim I independently verified
against `src/`, ADRs, `WATCH.md`, `.sergeant/`, `Cargo.lock`, and GitHub
issue/PR state matched the proposal's description exactly.

**Scope note:** whether PR #111's current `MERGED` state (confirmed via
`gh pr view 111`, merged into the real GitHub `main` at `3a46b87`, itself
an ancestor of the actual current `main` tip) makes the proposal's
pervasive "conditional, not yet merged" framing (§3.3, T2-06/T2-07, §12.4,
§19.8, the falsifiers, acceptance items #54/#55) stale prose is a real,
checkable question — but the contract's own Axes §3 (`assumptions`) names
it explicitly as that axis's starting point ("is every factual claim true
*as of this session*, not as of the proposal's own audit timestamp"),
which is a currency/staleness question, not a fidelity one. I independently
confirmed the branch's *technical content* — the retained/reap routes, the
ceiling→`blocked` fix, the filesystem Doctor check, transcript timestamps —
matches what the proposal describes; that is this axis's remit, and it is
clean. Whether the branch's now-merged status should change the proposal's
conditional framing is left to whichever critic owns `assumptions`.
