# T-SERIES-1 — fidelity critic

Axis: does the proposal say what the repository actually contains and
decided, and only that? Every consumed-surface claim (shipped MVP, WATCH,
Estate, `completed_dirty`, dashboard deletion, the integration branch behind
PR #111) was checked directly against the repository — `src/cli.rs`,
`src/api.rs`, `src/tui.rs`, `src/runtime/engine.rs`, ADRs 0005–0011,
`docs/gauntlet/contracts/WATCH.md`, `NORTH-STAR.md`, `.sergeant/index.md`, and
`gh issue`/`gh pr` state — never trusted from the proposal's own prose. Every
§23 (Dispositions) label was checked against the 2026-08-11 predecessor's
actual text, read at the commit that actually contains it
(`a9a25fa68938323d9585edc687fbf0e965084c2e`), not summarized from memory.

## The predecessor-location problem (found before the disposition sweep)

The proposal's front matter and body both cite the predecessor at
`a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`:

```yaml
supersedes:
  path: reference/proposal-tui-t-series.md
  revision: a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6
```

> **Supersedes:** [`reference/proposal-tui-t-series.md@a5fb875`](https://github.com/miztertea/sergeant-rs/blob/a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6/reference/proposal-tui-t-series.md)

`git show a5fb875:reference/proposal-tui-t-series.md` fails: `fatal: path
'reference/proposal-tui-t-series.md' exists on disk, but not in
'a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6'`. That commit (Tue Aug 11 00:58:23
2026, merge of PR #43, "ideaos-agent-prototype-gauntlet") predates the
predecessor file's existence in this repository entirely — it is the
predecessor's own self-declared `audit_revision` (the state of the repo *it*
was audited against), copied forward into `supersedes.revision` rather than
the commit that actually contains its text. The predecessor was vendored six
hours later at `a9a25fa` (Tue Aug 11 15:28:30 2026, "Execution-surface test
(owner ruling)... T-series TUI proposal vendored... via the inbox"). See F1.

This meant the disposition sweep below had to locate the real predecessor
independently before any `§23` label could be checked — the citation the
proposal itself provides does not resolve.

## Findings

### F1 — "Supersedes" citation (front matter and body) points to a commit that does not contain the predecessor

- **severity:** warning
- **section:** front matter `supersedes.revision`; body "Supersedes" line
- **claim/text at issue:** `reference/proposal-tui-t-series.md@a5fb875` — both
  as a YAML revision pin and as a clickable GitHub blob link.
- **what I checked:** `git cat-file -t a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`
  (a valid commit), then `git show a5fb875...:reference/proposal-tui-t-series.md`
  (fails — path does not exist at that commit), then `git log --follow` on the
  file to find where it actually enters history.
- **what I found:** `a5fb875` is real but is the predecessor's own
  `audit_revision` field (what state of the repo the 2026-08-11 proposal was
  written against), not a commit that contains the proposal file. The file
  was committed six-plus hours later at `a9a25fa`. The new proposal follows
  the same "cite my own audit_revision, not my commit" convention for
  itself (`audit_revision: 242abe3...`, no self-referential commit hash
  given at all) — internally consistent, but the "Supersedes" line is framed
  as a locator ("blob/.../reference/proposal-tui-t-series.md") that a reader
  or refuter clicking through gets a GitHub 404 from, and `git show` on the
  literal citation fails the same way. The contract itself hit this: its own
  Axes §1 text says "check each disposition against that predecessor's
  actual text rather than trusting the label," which requires locating that
  text — the citation given does not get a reader there.
- **does the section survive the correction:** yes. The predecessor is
  locatable (`a9a25fa`), its content is exactly what a proposal superseding
  the 2026-08-11 T-series document should be citing, and every disposition
  checked below against the real text holds up except F3. The fix is
  mechanical: point `supersedes.revision` and the body link at `a9a25fa`,
  or keep `a5fb875` as a separate "predecessor's own audit basis" note if
  that provenance detail is worth preserving, but not present it as the
  file's location.

### F2 — §12.3's Doctor check list does not match the actual check names `DoctorReport` produces

- **severity:** warning
- **section:** §12.3 (Health)
- **claim/text at issue:** "Current checks include installation, environment,
  data directory, Docker, journal, projection, daemon, estate, profiles, and
  disk pressure."
- **what I checked:** `Report::run` in `src/cli.rs@242abe3` (the function
  that assembles every `Check` Doctor emits) and each check function's own
  `Check::ok`/`Check::warn`/`Check::fail` name literal
  (`git_check`, `claude_check`, `environment_check`, `data_dir_check`,
  `docker_check`, `journal_check`, `projection_check`, `daemon_check`,
  `permission_mode_check`, `estate_check`, `disk_pressure_check`,
  `src/cli.rs:1750-1777`).
- **what I found:** the real, emitted check names are `git`, `claude`,
  `environment`, `data_dir`, `docker`, `journal`, `projection`, `daemon`,
  `permission_mode`, `estate`, `disk_pressure` — eleven checks. There is no
  check named `installation` (the two checks that plausibly correspond to
  that word, `git` and `claude`, are dropped from the proposal's list
  entirely and folded into an invented umbrella name). There is no check
  named `profiles`; the real check is `permission_mode`. Since T2-45 commits
  Health to rendering the shared `DoctorReport` unmodified ("Extract Doctor's
  structured `Check`/`Report` result… let both CLI and TUI consume it"), the
  check names §12.3's prose promises are not the names an operator will
  actually see.
- **does the section survive the correction:** yes. T2-45's mechanism
  (render whatever `DoctorReport.checks` contains, not a hand-picked list)
  means the implementation is unaffected regardless of what this prose
  calls the checks — this is a proposal-text accuracy defect, not a scope
  or behavior error. An operator reading §12.3 to know what Health will
  show would expect `profiles` and `installation` rows and get
  `permission_mode`, `git`, and `claude` instead.

### F3 — §23.2 presents the predecessor's silence on repo/group as a "previous decision"

- **severity:** warning
- **section:** §23.2 (Revised), row "Repo/group outside TUI"
- **claim/text at issue:** the Revised table lists, under "Previous
  decision": "Repo/group outside TUI," revised to "Full current lifecycle
  included through extract-on-contact."
- **what I checked:** the full predecessor text (`a9a25fa`, 1943 lines) for
  every mention of `repo`, `group`, `Estate`, `sgt repo`, `sgt group` —
  including its explicit non-goals list (§5.2, 22 bulleted exclusions) and
  its in-scope list (§5.1, 15 items).
- **what I found:** zero mentions. The predecessor's §5.2 non-goals list
  names journal search, new Work states, workflow authoring, file/diff
  views, host metrics, OpenTelemetry, mouse, web redesign, graph canvas, a
  plugin framework, archival semantics, and two specific issues (#26, #45)
  — repo and group management appear nowhere, neither in scope nor
  excluded. The predecessor simply never considered estate administration;
  "Estate" as a concept does not exist in it (confirmed separately — its
  top nav is exactly `Home Fleet Workflows`, §6.1). Labeling this absence a
  "previous decision" that the new proposal now "revises" overstates what
  was actually decided in 2026-08-11 — there was no ruling to revise, only
  a gap to fill. (Contrast with genuine revisions in the same table, e.g.
  "Doctor CLI-only," which the predecessor states explicitly as **Decision
  T-18 (R1)**: "Doctor remains a CLI-only diagnostic. The TUI has a small
  connection overlay, not a System dashboard" — that row's label is
  accurate because a real decision exists to check it against.)
- **does the section survive the correction:** yes. The substantive change
  — Estate becoming a first-class destination with full repo/group/Doctor
  lifecycle — is real, current, and correctly scoped elsewhere in the
  proposal (§12, §16.2). Only the disposition-table framing is wrong: this
  row belongs under "new scope the predecessor never addressed," not
  "revised decision." A one-word fix (e.g. "Repo/group unaddressed" instead
  of "outside TUI") would close this without touching any normative
  content.

### F4 — §5.5 credits ADR 0009 with moving Doctor and Watch into the no-auto-spawn set, when ADR 0009's own text says they were already there

- **severity:** info
- **section:** §5.5 (Observation never materializes the daemon)
- **claim/text at issue:** "ADR 0009 moved `status`, Work reads, analytics,
  Watch, Doctor, and TUI into the no-auto-spawn set."
- **what I checked:** ADR 0009's Decision section verbatim against this
  sentence.
- **what I found:** ADR 0009's own Decision text: "`status`, `work
  show`/`list`/`transcript`, `analytics`, and the TUI **join** `sgt doctor`,
  `sgt watch`, and `sgt daemon stop` **in** the no-spawn set." Its Context
  section is explicit that Doctor and Watch's membership predates this ADR:
  "`docs/gauntlet/contracts/WATCH.md`'s R-WATCH-3 already ruled this
  principle for one verb... owner-ruled 2026-08-13... This decision is that
  follow-on being resolved" (ADR 0009 itself is dated 2026-08-14). So ADR
  0009 is the ruling that added `status`/Work-reads/analytics/TUI to a set
  Doctor and Watch already belonged to a day earlier — not the ruling that
  moved all six. The proposal's sentence lists all six as things "ADR 0009
  moved," conflating pre-existing membership with the new ruling.
- **does the section survive the correction:** yes, cleanly. The
  operative claim — all six verbs are in the no-auto-spawn set today, which
  is what Decision T2-16 depends on — is correct and independently verified
  (`src/cli.rs:1341`: "`analytics`, and `tui` — every verb ADR 0009 moved
  into the no-spawn set" is the repository's own comment, so the proposal's
  phrasing actually echoes an imprecision already present in the codebase
  comment it was likely drawing from, not an invention). This is a
  precision nit about attribution, not a scope or behavior error.

## What I checked and found clean

**MVP/shipped-surface claims (§3.1, §3.2), verified at the pinned
`242abe3c4a889c2b666c7ce34b32812dd1ee8d61`:**

- `sgt tui` is an explicit, no-auto-spawn verb; bare `sgt` prints a static
  homepage (`src/cli.rs:64,198,821-826,1210`) — matches ADR 0010.
- The current TUI's `Screen` enum has exactly `Fleet` and `Detail`
  (`src/tui.rs:136-141`); its footer keymap is `j/k move · enter detail ·
  r refresh · i respond · c cancel · q quit` — no retry/extend bound,
  matching the Executive Summary's "small response/cancel keymap."
  `DETAIL_EVENT_TAIL: usize = 40` (`src/tui.rs:58`) matches "forty recent
  events" exactly.
- `GET /v1/work/{id}/transcript` exists and is wired
  (`src/api.rs:392,1777`); `POST /v1/work/{id}/extend` exists and is
  distinct from `retry` (`src/api.rs:396,2214-2276`); `completed_dirty` is
  a real reported state (`src/api.rs:1504`, `4365`); `sgt watch` is
  implemented and documented as adjudicated (`src/cli.rs:159-179`,
  `WATCH.md`'s "Ruled." language throughout).
- The root workflow catalog (`.sergeant/index.md@242abe3`) lists exactly
  **23** published workflows — matches "23 admitted workflows" precisely.
- `Cargo.toml` pins `ratatui = "0.30.2"`; its own comment confirms
  Crossterm is reached only through Ratatui's re-export, matching §3.2 and
  §8.3's claims about the dependency graph.
- `sgt repo`/`sgt group` subcommands exist (`src/cli.rs:216,222,889-890`);
  Doctor's `filesystem_check` did not yet exist at `242abe3` and is
  correctly described in §12.4/§12.3 as landing only if PR #111 merges.
- Issues #11, #16, #26 are `CLOSED` (`gh issue view`) — matches "shipped
  fixes that T-Series must preserve." Issues #15, #21 (dashboard
  reactivation) are `CLOSED` — matches "closed when the dashboard was
  deleted." No `src/web.rs`, no `web/` directory, no web-tagged test file,
  and no `web` verb exist at `242abe3` — the dashboard is in fact fully
  gone, not disabled-and-retained (the predecessor's actual 2026-08-11
  disposition, §15, Decision T-50).

**PR #111 / integration-branch technical content**, verified at the
proposal's own pinned `251a6f1c09caee95fcac30f724dab0ece166cae0` (not just
the branch's later head) to make sure nothing being described was added only
after the pin:

- `GET /v1/retained` and `POST /v1/work/{id}/reap`, plus `sgt work
  retained`/`sgt work reap`, both exist at the pinned revision
  (`src/api.rs:397-398`, `src/cli.rs:362-373`).
- The ceiling-interrupted-lands-in-`blocked` behavior exists at the pinned
  revision, named by its own regression test:
  `a_ceiling_interrupt_lands_the_work_in_blocked_not_wedged_active`
  (`src/runtime/engine.rs:4781`).
- Doctor's `filesystem_check` exists at the pinned revision
  (`src/cli.rs:1935,2497`).
- Human transcript rendering gains timestamps at the pinned revision, via
  `sgt work transcript`'s CLI rendering (not the API — the diff lands in
  `src/cli.rs`, not `src/api.rs`), matching #96's framing in the PR body.
- The branch name (`integration/path-to-mac-2026-08-15`) and PR number
  (111) are correct. `#120` is confirmed still `OPEN` via `gh issue view`,
  matching §3.3's "and #120 remains open."

**WATCH vocabulary (§14):** `WATCH.md`'s ruled set is exactly `needs_input,
blocked, waiting, failed, completed, canceled`, with `pending`/`active`
excluded — matches the proposal's six-state list and exclusion claim
verbatim (order differs, content does not).

**R-NS-6 (§5.6):** `NORTH-STAR.md`'s R-NS-6 text ("execution ≠ dialogue...
sgt owns message mechanics... the harness owns the conversation") matches
the proposal's restatement in substance.

**§23.1 (Adopted from the previous proposal):** every one of its twelve
items — Work-centered Home/Fleet/Workflows, canonical Work surface,
Attention drawer, deliberate multiline composer, local slash palette,
workflow chooser/reference, endpoint-backed workflow catalog, ordinal
workflow rail, Work-local Evidence and graph, separate Journal proposal, API
invalidation discipline, `TestBackend` semantic testing — is independently
present in the predecessor (`a9a25fa`) under a named decision (T-21, T-22,
Attention drawer §6.2, T-48, §7.3, §7.4, T-17, T-37/T-44, §4.6/T-15, T-19,
T-47, T-54). No invented provenance.

**§23.2 (Revised), remaining rows beyond F2:** "Bare `sgt` is the TUI"
(predecessor: "This proposal makes bare `sgt` the primary interactive
surface," line 73), "Home/Fleet/Workflows → Estate added" (predecessor's nav
is exactly `Home Fleet Workflows`, no Estate concept anywhere), "Doctor
CLI-only" (Decision T-18, quoted above), "Local custom editor" (Decision
T-48, R7, explicitly bounded and explicitly open to override by "a narrowly
vetted text-area dependency" — the new proposal's revision is the predecessor's
own named escape hatch, not a contradiction of it), "Submit/respond/retry/
cancel" (predecessor §4.4's exact mutation list, no `extend`), "Event-derived
conversation" (Decision T-14: "a pure presentation fold over known event
kinds"), "no output/envelope UI" (predecessor's §4.3 "N3 facts" list has
neither), "completed only" (zero `completed_dirty` mentions in predecessor)
— all check out.

No finding invalidates any section's premise. F1, F2, and F3 are warning
severity (a broken/misleading citation, a mismatched check-name list, and an
overstated disposition label — all mechanically fixable without touching any
normative decision); F4 is info severity and echoes an imprecision already
present in the repository's own code comments rather than inventing one.
Every other consumed-surface claim I independently verified against `src/`,
ADRs, `WATCH.md`, `.sergeant/`, and GitHub issue/PR state — MVP shipped
behavior, WATCH's vocabulary, Estate's CLI basis, `completed_dirty`, the
dashboard's actual deletion, and PR #111's technical content at its pinned
revision — matched the proposal's description exactly.

**Scope note:** whether PR #111's current `MERGED` state (confirmed via `gh
pr view 111`) makes the proposal's pervasive "conditional, not yet merged"
framing (§3.3, T2-06/T2-07, §12.4, §19.8, falsifiers, acceptance #54/#55)
stale prose is a real, checkable question — but the contract's own Axes §3
(`assumptions`) names it explicitly as that axis's starting point ("is
every factual claim true *as of this session*, not as of the proposal's own
audit timestamp"), which is a staleness question, not a fidelity one. I
independently confirmed the branch's *technical content* (retained/reap
routes, the ceiling→`blocked` fix, the filesystem Doctor check, transcript
timestamps) matches what the proposal describes — that is this axis's
remit, and it is clean. I did not grade whether the branch's merge status
should now change the proposal's conditional framing; that is left to
whichever critic owns `assumptions`.
