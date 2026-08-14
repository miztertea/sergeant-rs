# FOUNDATION-1 — critic: assumptions

Blind critic report per `docs/gauntlet/contracts/FOUNDATION-1.md`, axis 3
(**assumptions**). Artifact graded:
`reference/proposal-foundation-rationalization.md` §1–§8. Every claim below
was checked against the repository as it stands (working tree, git history,
and `gh` against `miztertea/sergeant-rs`) on 2026-08-14, not against a
summary of the proposal.

## Method note

The checklist in the contract (`resolve_data_dir`, `ensure_daemon` call
sites, `surfaces_dir`/`data_dir`, the §30 citation, `validate-and-ship`'s
40/50 stages, `src/web.rs` size, the dispositions-doc silence, the measured
figures, and all 22 cited issue numbers) was checked exhaustively, plus
every other checkable file path, line number, ruling id, and quotation in
§1–§8. The overwhelming majority resolved to exactly what the proposal
claims — that is reported here too, not just the misses, per the contract's
own warning against inventing findings to look useful. Three confirmed
discrepancies survived; none invalidates the section it sits in.

---

## Findings

### F1 — `severity: warning` — §2.2, "closed six issues"

**Claim.** "The 2026-08-14 cross-platform sprint, run through the engine. It
closed six issues (#16, #70, #83, #86, #87, #91)..."

**What I checked.** `gh issue view` for each of the six, plus each issue's
GitHub timeline (`gh api repos/miztertea/sergeant-rs/issues/<n>/timeline`)
for `closed`/`reopened` events. Also traced the PRs that fixed each one
(#92, #93, #98) to their base branch, and checked whether the branch tip
that carries all of them is an ancestor of `origin/main`.

**What I found.** All six issues are currently **OPEN** on GitHub
(`state: OPEN`, `closedAt: null`), and none has ever recorded a `closed`
event in its timeline — they were not closed and reopened, they were simply
never closed. The fixing PRs (#92 "Wave 1/G1", #93 "Wave 1/G2", #98 "Wave
2/G4") all target `base: integration/cross-platform-2026-08-14`, not `main`,
so GitHub's `Closes #N` auto-close never fired even though PR #93's own body
literally says "Closes #16." `git merge-base --is-ancestor` confirms the
current tip (`a740d30`, which contains all of these fixes via the
`fix/adrs-0005-0011` → `integration/cross-platform-2026-08-14` merge chain)
is **not** an ancestor of `origin/main`. This is consistent with the
proposal's own §7 admission that "the integration branch is unmerged" — but
§2.2 states "it closed six issues" as a plain, unqualified fact, and a
reader checking the tracker right now will find all six open. By contrast
#73, whose fixing PR (#79) targeted `main` directly, *is* closed
(`closedAt: 2026-08-14T09:33:23Z`) — confirming the mechanism (base branch)
is what determines whether "closed" is literally true, and that mechanism
cut against six of these seven.

**Does the section survive?** Yes. §2.2 is method/basis framing, not one of
the specific defect citations §3 is built on; the six defects those issues
describe are independently real (the fixing commits exist, reviewed and
gated, just not yet merged to `main`). But the sentence as written asserts
a settled fact that is false against the current repository state, in the
section whose job is to establish that this round's evidence is measured
rather than asserted. It should read "fixed" (or "landed on the integration
branch") rather than "closed," or be qualified the way §7 already qualifies
the branch state elsewhere.

---

### F2 — `severity: warning` — §3.6, "#11 was fixed on 2026-08-13"

**Claim.** "`NORTH-STAR.md` still lists the dashboard as a surface and its
Wave 3 plans `#11/#16`; and #11 was fixed on 2026-08-13, which the proposed
freeze would have foreclosed."

**What I checked.** `gh issue view 11` for `closedAt`; the merging PR (#76,
"TUI: pre-loop hangup guard (#26) and fleet column truncation (#11)") and
the PR (#79, "Bug sprint 2026-08-14: five issues... via sergeant Work
items") that actually closed it; commit timestamps in both UTC and the
repo's own local convention (`-04:00`, per every commit's `%ad`).

**What I found.** Issue #11's `closedAt` is `2026-08-14T09:33:22Z`. The
commit that merged the fix (`6e5fbbf`) is timestamped
`2026-08-14T00:16:05-04:00`. Both readings land on **2026-08-14**, not
2026-08-13 — there is no timezone under which the close event lands on the
13th. `north-star-arbitration-2026-08-11.md:196` (the "delete the dashboard"
argument this subsection is contrasting) is dated 2026-08-11, so the
underlying point — the fix landed after the argument for freezing/deleting
the dashboard was made, and the freeze was never actually enforced — holds
under the corrected date too; only the specific day cited is wrong.

**Does the section survive?** Yes. The date is off by one, but §3.6's claim
depends on "#11 landed after the 2026-08-11 argument," which is true on
either the 13th or the 14th.

---

### F3 — `severity: info` — §5.5, "m2 t7" cited as a test that changes

**Claim.** "Blast radius, known: `AGENTS.md`'s standard-loop step 2 relies
on auto-spawn on a fresh boot; pinned contract tests change (m2 t7,
`tests/m6_surfaces.rs:414`, `tests/m8_estate_cli.rs:1080`)..."

**What I checked.** Traced "m2 t7" to `docs/gauntlet/contracts/M2.md`
acceptance criterion 7 and its implementing test,
`t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`
(`tests/m2_daemon_api.rs:1535`). Read the test body and cross-checked which
verb triggers its auto-spawn assertion against §5.5's own no-spawn/keep-spawn
split (`run`, `respond`, `retry`, `extend`, `cancel` keep auto-spawn;
`status`, `work show|list|transcript`, `analytics`, the TUI lose it).

**What I found.** t7's auto-spawn assertion is driven by `sgt run` ("No
daemon running: `sgt run` auto-spawns one and submits"), which §5.5 itself
keeps in the auto-spawn set — that assertion is unaffected by this change.
The test's later `sgt work list --json` call runs *after* `run` has already
spawned a live daemon, so it never exercises `work list`'s own auto-spawn
behavior from a cold data dir; the test as written has no code path that
would break under §5.5. The other two citations in the same sentence do not
have this problem: `tests/m6_surfaces.rs:414` asserts "bare `sgt` must
auto-spawn a daemon like every other client" and
`tests/m8_estate_cli.rs:1080`'s block runs `status` specifically to
auto-spawn and asserts exactly one daemon exists afterward — both pin
behavior §5.5 removes.

**Does the section survive?** Yes. The general claim — that removing
auto-spawn from observation verbs has pinned-test blast radius — is still
correct and still evidenced by the other two citations; one of the three
supporting examples doesn't actually support it.

---

## What checked out (no finding)

Reported per the contract's instruction that a clean result on a checked
item is itself evidence, not just silence:

- `resolve_data_dir` (`src/cli.rs`, function at line 406–419, cited range
  400–421 covers its doc comment): confirmed it walks to a discovered
  estate root via `Workspace::estate_root` and then unconditionally joins
  `DEFAULT_ESTATE_DATA_DIR`, never reading anything from the manifest
  itself. `[estate] surfaces_dir` (`src/domain/workspace.rs:204`, doc at
  199–204) confirmed present and honored, overridable by `SGT_SURFACES_DIR`
  (`src/daemon.rs:775`). No `[estate] data_dir` field exists anywhere in
  `src/domain/manifest.rs` today — confirming §5.4's "add `data_dir`" is
  actually additive. (The 400–421 citation style, doc-comment-inclusive, is
  the repo's own convention — issue #73's body cites the identical range.)
- `ensure_daemon`: **exactly ten** call sites in `src/cli.rs` (426, 439,
  549, 576, 590, 604, 618, 708, 723, 766), covering bare `sgt`, `daemon
  start`, `status`, all three `work` subcommands, `analytics`, `web`, plus
  mutating verbs — matches "ten call sites" and the verb list in §3.4/§5.5
  exactly. `doctor` and `daemon stop` confirmed to never call it.
- §30 citation: `src/cli.rs:424–426` carries the literal "§30: bare `sgt` is
  the TUI" comment immediately above the `ensure_daemon` call it describes;
  "§30" itself is a live, repo-wide citation convention (ADR 0010,
  `tests/m6_surfaces.rs`, `src/tui.rs`, `src/api.rs`, `docs/gauntlet/contracts/M6.md`
  all cite it the same way).
- `validate-and-ship` has **exactly seven** stages
  (00-check-scope/10-do-the-work/20-select-intent-transport/30-start-run/
  40-drive-gates/50-reconcile-custody/60-close-out). `40-drive-gates`'s
  `CONTEXT.md` confirms the `auto-fix`/`no-op`/`ask-user` three-way
  classification verbatim. `50-reconcile-custody`'s `CONTEXT.md` confirms
  the `branch_sync` decision table (`sync`/`continue_active_run`/
  `recover_custody`) including the `--keep-local` dirty-worktree remedy,
  verbatim.
- `src/web.rs` is exactly 779 lines. `sgt web` verb confirmed
  (`Command::Web`, `src/cli.rs:762`).
- `north-star-dispositions-2026-08-11.md`: zero case-insensitive matches for
  "web", "dashboard", or "freeze" — confirms the proposal's claim it "never
  mentions" any of the three. `north-star-arbitration-2026-08-11.md:196` is
  exactly the "Delete `src/web.rs` (779) + `web/` (224) + the `sgt web`
  verb... #11 (width) dies with the freeze" sentence. `NORTH-STAR.md:44`
  lists dashboard as a surface; `NORTH-STAR.md:103-104` plans Wave 3 as
  "T-series slice (composer, legible thread, respond, #11/#16)" — both
  confirmed.
- Issues #15, #21 (dashboard), #60 (env/PATH misdiagnosis), #64
  (self-hosting contradiction), #73 (closed; body is literally a
  base-commit bisect: "Pre-existing on `main` at `1929a90` — verified by
  running the same test on the base commit"), #80 (opened 23 minutes after
  #73 closed, titled exactly the re-homed contract question), #90, #94,
  #95, #96, #100, #18, #25, #81, #82, #83, #85, #86, #87 all exist and
  their titles/state match what the proposal implies. #94's two occurrences
  and their transcript quotes ("I'll simply wait for the background task
  notification instead") are close to verbatim in the issue's own comments,
  including the "second occurrence... the dispatch brief demanded empirical
  proof" causal claim.
- R-WATCH-3's quoted text ("observation must not materialize the thing
  observed") is exact, from `docs/glossary.md:120`.
- `sgt doctor`'s message "no daemon running; the next client command starts
  one" is an exact string match, `src/cli.rs:2305`.
- `AGENTS.md`'s standard-loop step 2 is exactly `sgt status`
  (`AGENTS.md:76`), which is on the `ensure_daemon` list above — confirms
  the "relies on auto-spawn on a fresh boot" claim.
- `docs/DEVELOPMENT.md`'s gate-ownership rule text and its never naming
  `validate-and-ship` are both confirmed (lines 68-70, and the "Shipping
  gate" section at 92-94 describes the procedure in prose with no workflow
  citation). The sprint's own `lessons.md` (G3) independently recorded the
  identical diagnosis in near-identical words ("An owning document that
  summarizes a workflow['s procedure] without citing it guarantees readers
  stop there").
- The "roughly eight minutes / three gates ran serially / one lost to a
  harness kill" figures (§3.1) are not independently attested anywhere
  outside `docs/adr/0005-gating-becomes-a-dispatched-work.md`, which the
  proposal is built on and which states them in the same words. Not a
  finding — the contract lists ADRs 0005-0011 as read-as-context material
  the proposal is entitled to rely on — but noted since it is the one
  measured figure I could not corroborate from a source independent of the
  proposal's own decision record.
- All seven ADRs (0005-0011) exist with titles matching the §1 table
  exactly. R-NS-6 ("execution ≠ dialogue") is quoted exactly from
  `NORTH-STAR.md:52`. The "clone → `sgt` on PATH → `sgt init` → open your
  harness → say let's work on the api bug" loop and the "under five minutes
  of setup" acceptance line are both exact quotes from `NORTH-STAR.md:20-25`.
  `.sergeant/data` is confirmed gitignored (`.gitignore:13`). "The manifest
  ... the MVP plan calls the keystone" traces correctly through
  `NORTH-STAR.md:68` to `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`.

## Summary

Three confirmed factual discrepancies (F1-F3), all warning/info severity,
none of which invalidates the section it appears in. Given the volume of
specific, checkable claims in this proposal — file paths, line numbers,
exact quotations, issue numbers and their state, measured stage counts —
the hit rate is unusually high. The two real errors (F1, F2) share a
pattern worth naming even though neither is a finding on its own: both
overstate completion by one notch — "closed" where the tracker says open,
"the 13th" where the record says the 14th — in the direction that makes the
evidentiary base look slightly more finished than the repository shows.
Neither changes what §3 is actually arguing.
