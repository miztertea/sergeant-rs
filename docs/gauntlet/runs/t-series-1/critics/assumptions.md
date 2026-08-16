# T-SERIES-1 — critic: assumptions

Blind critic report per `docs/gauntlet/contracts/T-SERIES-1.md`, axis 3
(**assumptions**). Artifact graded: `reference/proposal-tui-t-series.md`
§1–§25. Every claim below was checked against the repository as it stands
right now — the live `miztertea/sergeant-rs` GitHub repository (`gh`, and a
fetch of its `main` to a local `github` remote), not the proposal's own
`audit_revision` pin (`242abe3`) and not this session's local branch tip,
both of which are themselves now behind real `main` by 49 commits.

## Method note

The contract's named starting check — `gh pr view 111` for merge state —
was run first, then every other place "PR #111", "integration branch", or
"if... merges" appears was located by grep and individually graded. Beyond
that sweep, every other checkable commit hash, PR number, issue number, and
counted figure in the proposal (the `supersedes` pin, PR #105/#69, issues
#15/#21/#120, the 23-workflow catalog count, ADR existence, `src/web.rs`
absence, `src/tui.rs`'s claimed Fleet/Detail-only state, the Ratatui/
Crossterm version pins, and the shape of the retained/reap API the
conditional sections describe) was independently verified against the live
repository. The majority hold exactly as stated — reported below, not just
the misses, per the contract's own convention. Three confirmed
discrepancies survived, two of them severe because of how many sections
build on the fact they get wrong.

---

## Findings

### F1 — `severity: error` — proposal-wide, "PR #111 is an open, unmerged, concurrent dependency"

**Claim.** The proposal's frontmatter, header, and body assert throughout
that PR #111 is open and must not be treated as landed. Representative
instances: frontmatter `integration_review` block (no `merged` field, framed
as a concurrent review); header line 49, "**Concurrent integration
review:** PR #111 at `251a6f1`"; §3.3, "The branch is not binding main
truth... [PR #111] is reviewed as a concurrent dependency, never treated as
merged"; **Decision T2-06**, "T0 pins the actual implementation base after
PR #111 is either merged or explicitly excluded"; **Decision T2-46**,
"Retained-state UI is conditional consumption of a real merged API"; and
falsifier 21, "uses PR #111 facts as shipped before merge."

**What I checked.** `gh pr view 111 --json state,mergedAt,mergeCommit,
baseRefName` — the exact check the contract names.

**What I found.** PR #111 is `MERGED`, `mergedAt: 2026-08-15T15:28:02Z`,
merge commit `3a46b87c17d249655708ed5ac32f6704738776cf`, base `main`. It is
already fact, not a concurrent review. Every place this conditional
recurs, located by grepping "PR #111", "integration branch", "if... merge",
and "conditional":

| Location | Current text's framing | Stale now? | Section survives? |
|---|---|---|---|
| Frontmatter `integration_review` | reviewed as unmerged | yes | yes — add `merged: true` |
| Header L49 "Concurrent integration review" | concurrent/pending | yes | yes — reads "Merged integration review" |
| §3.3 (whole section) | branch "not binding," "never treated as merged" | yes | yes — becomes a plain source-of-truth note |
| T2-06 register line (§1, §22) | "either merged or explicitly excluded" | yes, now trivially resolved | yes — T0's job here is already done |
| §6.1 item 8 "conditional retained-state disposal" | conditional | yes | yes — unconditional now |
| §6.2 non-goal "silent consumption of unmerged PR #111 features" | assumes unmerged | yes, now moot | yes — the non-goal is satisfied by construction, reword to avoid implying it's still open |
| §7.2 "Retained-state result if PR #111 lands" | conditional entry point | yes | yes — unconditional entry point |
| §10.5 "If PR #111 lands, retained-state markers may appear" | conditional | yes | yes — unconditional |
| §12.3 "If PR #111 lands, filesystem reliability joins the report" | conditional | yes — **already shipped**, see What Checked Out | yes — state as fact |
| §12.4 heading + body, "conditional on PR #111" / "If the integration branch's retained/reap surfaces merge" | conditional | yes | yes — becomes an ordinary section, no heading caveat |
| T2-46 register line | "conditional... merged API" | yes | yes |
| §13.3 "once PR #111 or equivalent lands, visible journal timestamps" | conditional | yes — **already shipped** | yes |
| §13.7 "conditional retained state" | conditional | yes | yes |
| §13.10 action matrix "conditional reap" | conditional | yes | yes |
| §15.3 "Conditional if merged: /retained /reap" | conditional | yes | yes — moves into core palette vocabulary |
| §16.1 "conditional retained/reap" client methods | conditional | yes | yes — ordinary typed methods |
| §19.6 "If PR #111 lands, filesystem reliability is included" | conditional | yes | yes |
| §19.8 heading + body, "Integration branch conditional tests" / "If retained/reap lands" | conditional | yes | yes — becomes an ordinary test section, no longer gated |
| §20.4 T3 "conditional retained/reap consumption" | conditional | yes | yes |
| Acceptance §21 item 54 | "If PR #111 lands, retained/reap is consumed..." | yes | yes — drop the hedge, keep the requirement |
| Acceptance §21 item 55 | "If PR #111 does not land, no retained/reap placeholder..." | yes, now vacuous | yes — the branch of the acceptance contract that can never fire; harmless but should be struck |
| Falsifier 21, §24.1 | "uses PR #111 facts as shipped before merge" | yes, now a falsifier for a condition that cannot recur | yes — harmless, but should be reworded or removed since the underlying event already happened by rule (PR #111 is now the actual base) |

**Does the section survive?** Yes, every one of the ~20 locations survives
a purely mechanical correction (delete the hedge, state the now-settled
fact) — none requires a design change, because the content each location
describes (retained/reap semantics, the filesystem-reliability Doctor
check) matches what actually shipped, confirmed independently below. But
the volume and spread of the defect — a single stale premise propagated
through the frontmatter, the header, one full section (§3.3), one full
subsection (§12.4), one full test section (§19.8), two acceptance criteria,
and a falsifier — is exactly the "sections are built on it" danger the
contract's own axis description names, even though no single section's
underlying design is wrong.

---

### F2 — `severity: error` — §3.3 / frontmatter, "PR #111 at `251a6f1`" cites the wrong commit

**Claim.** Frontmatter: `integration_review: revision:
251a6f1c09caee95fcac30f724dab0ece166cae0`. Header line 49: "**Concurrent
integration review:** [PR #111 at `251a6f1`]." §3.3: "head =
251a6f1c09caee95fcac30f724dab0ece166cae0."

**What I checked.** `gh pr view 111 --json headRefOid` for the PR's actual
head commit at merge time, then `git show -s --format='%H %ad %s'` on the
cited SHA `251a6f1c09caee95fcac30f724dab0ece166cae0` to see what it actually
is, then `git merge-base --is-ancestor` in both directions to place it
relative to the PR's real head.

**What I found.** PR #111's real head commit, per GitHub, is
`bceed965c24de7fa781001e3bd7835d8ef58b139` ("Retrospective §1.3: the merge
check caught a sweep about to destroy evidence," `2026-08-15T ~15:xx`,
immediately before the merge commit `3a46b87`). The SHA the proposal cites,
`251a6f1c09caee95fcac30f724dab0ece166cae0`, is a real commit, but it is
**`gh pr view 122`'s merge commit** — "Merge pull request #122 from
miztertea/lane/w6c-gate-fixes," a different, earlier-merged sub-lane PR
("W6c: shipping-gate fixes — reap preview auto-spawn (ADR 0009) and
worktree-remove-failure leak") that landed on the *integration* branch
(`base: integration/path-to-mac-2026-08-15`) before PR #111 was cut, not on
`main`, and not PR #111 itself. `251a6f1` is a genuine ancestor of the real
head `bceed96` (confirmed both directions with `git merge-base
--is-ancestor`), so it is not a fabricated hash — it is a real point
earlier on the same branch, mislabeled as the tip. At minimum two more
commits landed on the reviewed branch after the point the proposal actually
audited: `11b138c` ("Session retrospective: residue sweep, instruction
defects, tool-call patterns") and `bceed96` itself (the merge-check-caught-
a-sweep-about-to-destroy-evidence commit) — the second of which, by its own
title, is exactly the kind of late safety-relevant fix a reviewer would
want to know they hadn't seen yet.

**Does the section survive?** Yes. Nothing in §3.3's bullet list of
candidate surfaces (ceiling-interrupt → `blocked`, transcript timestamps,
filesystem-reliability Doctor check, `GET /v1/retained`, `POST
/v1/work/{id}/reap`) is contradicted by the two missed commits — I checked
their content and neither touches those surfaces. But the citation itself
is wrong and should be corrected to `bceed965c24de7fa781001e3bd7835d8ef58b139`
now that the whole "which revision did we review" question is superseded by
F1 anyway (the branch merged; the citation only matters as a historical
audit trail at this point).

---

### F3 — `severity: warning` — frontmatter, `supersedes.revision` does not point to the predecessor proposal

**Claim.** Frontmatter: `supersedes: path: reference/proposal-tui-t-series.md,
revision: a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`. Header line 50:
"**Supersedes:** [`reference/proposal-tui-t-series.md@a5fb875`]" — a live
GitHub blob link to that path at that revision.

**What I checked.** `git show a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6:
reference/proposal-tui-t-series.md` (does the path exist at that commit),
then `git log --follow --diff-filter=A -- reference/proposal-tui-t-series.md`
to find the commit that actually introduced the file.

**What I found.** The commit `a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`
exists (`Merge pull request #43`, 2026-08-11), but
`reference/proposal-tui-t-series.md` does not exist in that commit's tree —
`git show` returns "path ... exists on disk, but not in
a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6." The blob link in the header
therefore 404s on GitHub. The file's actual (and only) prior state was
introduced by commit `a9a25fa68938323d9585edc687fbf0e965084c2e`
("Execution-surface test (owner ruling): workflow vs CLI surface vs
operator skill," 2026-08-11T15:28:30Z) — and that commit's own version of
the file carries `audit_revision: a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`
in *its* frontmatter. The cited hash is real, but it is the **predecessor
proposal's own audit-basis pin**, copied into the wrong field of the
revised proposal — `supersedes.revision` should identify the commit at
which the predecessor *file* existed (`a9a25fa`), not the commit the
predecessor was itself auditing against.

**Does the section survive?** Yes. §3.1's disposition table and §23's
Adopted/Revised/Rejected lists describe what actually changed in prose, and
a reader can still follow the argument without the link resolving —
checking whether that prose accurately reflects `a9a25fa`'s real text is
the fidelity axis's job, not this one, and I did not duplicate that check.
This finding is scoped to the citation itself: the specific commit named as
"the predecessor" is not a commit where the predecessor exists, which
breaks the one mechanical way a reader could open the exact diff the
proposal claims to be revising.

---

## What checked out (no finding)

- `gh pr view 111`: base `main`, head branch `integration/path-to-mac-2026-08-15`
  — both match §3.3 exactly.
- PR #111's own body (fetched live) independently corroborates §3.3's
  characterization of what was true at review time: "four false `passed`
  verdicts," "#120 stays open," and the same five bullet surfaces §3.3
  lists (ceiling-interrupt → `blocked` is #90, transcript timestamps is
  #96, filesystem reliability is #85, retained/reap is #109) — all closed
  or referenced by that exact PR.
- Issue #120 (`no-mistakes` diff-base short-circuit): confirmed `state:
  OPEN`, `closedAt: null`, right now — §1916's "the gate defect in PR #111
  must be resolved before its result is trusted" is a forward requirement
  and, since #120 is still open, is not itself stale.
- Issues #15 and #21: both `CLOSED`, `closedAt: 2026-08-15T02:42:48Z`, via
  PR #105 ("Fixes #68, #21, #15"), which is itself confirmed `MERGED`
  (`2026-08-14T23:46:04Z`) — matches §3.1's disposition table exactly.
- PR #69 ("sgt watch — the harness subscription CLI"): confirmed `MERGED`
  (`2026-08-13T20:51:17Z`) — matches §3.1's "headless return path" claim.
- `src/web.rs`: confirmed absent both at `242abe3` (the proposal's pinned
  audit point) and at current `main` — the dashboard-deletion claim holds
  at both timestamps.
- `src/tui.rs`: byte-for-byte identical between `242abe3` and current
  `main` (`diff` empty, both 2614 lines) — §3.2's "the TUI still has only
  Fleet and Detail screens" has not gone stale despite 49 commits of
  unrelated work landing on `main` since the proposal's audit point.
- ADRs 0005–0011: all seven exist at `242abe3`, titles match every citation
  in §3.4 and §24.2, including 0011 ("Delete the dashboard," Accepted
  2026-08-14) whose own text independently confirms `src/web.rs` was 779
  lines and `sgt web` a live verb before deletion — consistent with, though
  not itself cited by, the proposal.
- The workflow catalog: `.sergeant/index.md` lists exactly 23 published
  workflows at `242abe3`, byte-identical to the same file at current
  `main` (`diff` empty) — the "23 admitted workflows" figure in §3.2 and
  §11.1 is accurate both then and now.
- Ratatui/Crossterm: `Cargo.toml` pins `ratatui = "0.30.2"`; `Cargo.lock`
  resolves `ratatui 0.30.2` and `crossterm 0.29.0` as the ratatui-facing
  edge (a second, unrelated `crossterm 0.28.1` transitive entry also
  exists but is not what §8.3/§8.8 cite) — matches every version-specific
  claim and doc link in §8. `ratatui-textarea` is absent from both
  `Cargo.toml` and `Cargo.lock`, consistent with T2-31's own framing that
  it is not yet adopted, only researched and T0-gated.
- The retained/reap API surface §3.3, §12.4, and §13.7 describe as
  conditional is, on inspection of current `main`'s `src/api.rs` and
  `src/cli.rs`, **exactly** what's shipped: `GET /v1/retained` →
  `list_retained`, `POST /v1/work/{id}/reap` → `reap_work`, CLI verbs
  `sgt work retained` / `sgt work reap --yes`, and the response/print paths
  carry `path`, `bytes` (byte count), and `disposition` (reason) per
  binding — matching §12.4's "retained binding, path, reason, and byte
  count" claim field-for-field.
- The filesystem-reliability Doctor check §12.3/§19.6 describe as
  conditional is present in current `main`'s `src/cli.rs` (`fs_locking`
  module, `Reliability` enum, a `Check` built from
  `platform::fs_locking::detect_for_path`) — matches the claim exactly,
  again modulo the "if it lands" framing F1 covers.

## Summary

Three confirmed findings. Two (F1, F2) are `error` severity and both trace
back to the same root: the proposal's integration-branch citations were
accurate at its own audit moment but the moment has passed — PR #111
merged hours before this grading, and the specific commit the proposal
cites as that branch's reviewed state is not even the branch's actual head,
it is a different, earlier, misattributed PR's merge commit. Neither
defect invalidates any section's design: every conditional location I
found resolves to the same content the section already describes, just
without the "if" — the retained/reap API and the filesystem-reliability
Doctor check both exist on `main` right now in the exact shape §12.3–§12.4
and §13.7 predict, and nothing on the two missed commits contradicts
anything the proposal claims. The third (F3, `warning`) is a broken
citation in the frontmatter's own supersession pointer — real hash, wrong
field, dead link — that affects traceability but not the argument's
content. Everything else independently checkable in this ~2,300-line
proposal — PR states, issue states, file existence at two different
timestamps 49 commits apart, dependency versions, API route/CLI-verb
citations, and the one counted figure (23 workflows) — held exactly as
claimed both at the proposal's own audit point and right now.
