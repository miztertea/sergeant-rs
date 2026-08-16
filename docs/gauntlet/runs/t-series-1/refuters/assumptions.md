# T-SERIES-1 — refuter: assumptions

Adversarial refutation per `docs/gauntlet/contracts/T-SERIES-1.md`, axis 3
(**assumptions**), of `docs/gauntlet/runs/t-series-1/critics/assumptions.md`.
I did not write the proposal and did not write the critic report. Every
claim below was re-verified independently — fresh `gh` calls against
`miztertea/sergeant-rs`, fresh `git` commands, fresh file reads — rather
than trusted from the critic's citations.

## Method note

One pitfall discovered and worked around: this working copy's `origin`
remote is a **local file-path alias** (`/home/miztertea/sergeant-rs`), not
the live GitHub repository — checking "current `main`" against it silently
reproduces the proposal's own stale audit point (`242abe3`) instead of
verifying anything. The live repository is the `github` remote (matching
the critic's own method note: "a fetch of its `main` to a local `github`
remote"). A first pass of mine against `origin/main` produced false
negatives for the retained/reap API and the filesystem-reliability Doctor
check before I caught this and re-ran everything against `github/main`
(fetched fresh this session). Flagging this because it is exactly the kind
of trap this axis exists to catch, and it would have produced a false
refutation of the critic's "what checked out" section had I not caught it.

For each of the critic's three findings I re-ran the named checks from
scratch and additionally re-verified a sample of the "what checked out"
section, since a false negative there would undermine "does the section
survive" for all three findings.

---

## F1 — proposal-wide, "PR #111 is an open, unmerged, concurrent dependency" — **CONFIRMED**, severity unchanged (error)

**Ground 1: factually wrong?** Re-ran the contract's named check myself:
`gh pr view 111 --json state,mergedAt,mergeCommit,baseRefName,headRefOid` →
`MERGED`, `mergedAt: 2026-08-15T15:28:02Z`, merge commit `3a46b87c...`, base
`main`, head `bceed965c24de7fa781001e3bd7835d8ef58b139`. Matches the critic
exactly.

I re-ran the critic's own grep sweep (`PR #111`, `integration branch`,
`if... merge`, `conditional`) against the proposal myself rather than
trusting their table, and independently landed on the same ~20 locations,
same section numbers, same line content. I found **one additional
location** the critic's table omits: §24.2's reference list, line 2277,
`[Open integration PR #111](https://github.com/miztertea/sergeant-rs/pull/111)`
— the link label itself asserts "Open," which is now false the same way
every other instance is. This doesn't change the finding (it's the same
defect, one more site), but it does mean the critic's sweep, while
matching every location it found, was not fully exhaustive. Noting it as
corroboration, not a refutation.

**Ground 2: out of scope?** No. Squarely a current-repository-state claim
about a checkable GitHub object, the exact class of claim this axis exists
to check, and the contract names this exact check as the starting point.

**Ground 3: style preference dressed as defect?** No. Merged/unmerged is
binary and checkable, not phrasing.

**Ground 4: does the proposal already say this elsewhere, rescuing the
locations the critic flagged?** I checked whether any section hedges or
corrects the "unmerged" framing. None does — every location in the
critic's table and the one I found independently states the "unmerged/
concurrent" framing without qualification. No rescue.

**Ground 5: severity — is `error` correctly calibrated, or inflated?**
I pushed harder here than the other two findings, because the precedent
(`docs/gauntlet/runs/foundation-1/refuters/assumptions.md`, F1) calibrated
an analogous "false but categorically true/false" claim to `warning`, not
`error`, on the reasoning that no downstream section's *design* depended on
it — it was evidentiary/methodological framing text no Work would consume
as instruction. I checked whether F1 here is the same shape and concluded
it is not, for a reason the critic's own writeup doesn't fully spell out:
§3.3's "never treated as merged" and the T2-06/T2-46 decision-register
entries are not narrative background, they are **prescriptive instructions
a dispatched Work would read as binding** — "never treat PR #111 as
merged" is exactly the kind of clause that could cause a T0 Work to
actively avoid consuming the already-shipped, already-verified retained/
reap API (confirmed real and correct below) and instead re-implement or
gate around semantics that already exist on `main`, producing a materially
wrong artifact rather than just imprecise prose. That is a concrete failure
mode the FOUNDATION-1 F1 precedent didn't have (its false claim sat in pure
narrative text no Work consumed as a directive), and it's why I don't think
this is over-inflated the way it might first appear by analogy. `error` is
correctly calibrated, and if anything the critic's own writeup undersells
why it clears that bar — it stops at "every location survives mechanical
correction" without naming the enactment risk the stale imperative
framing creates.

**Verdict: CONFIRMED**, severity `error` unchanged, with one additional
corroborating location (§24.2/line 2277) beyond the critic's own table.

---

## F2 — §3.3 / frontmatter, "PR #111 at `251a6f1`" cites the wrong commit — **CONFIRMED**, severity downgraded (error → warning)

**Ground 1: factually wrong?** Re-verified independently:

- `gh pr view 111 --json headRefOid` → `bceed965c24de7fa781001e3bd7835d8ef58b139`.
- `git show -s --format='%H %ad %s' 251a6f1c09caee95fcac30f724dab0ece166cae0`
  → `Merge pull request #122 from miztertea/lane/w6c-gate-fixes`,
  `2026-08-15 11:05:36 -0400`.
- `gh pr view 122 --json title,mergeCommit,baseRefName,state` → confirms
  `251a6f1` is PR #122's merge commit exactly, title "W6c: shipping-gate
  fixes — reap preview auto-spawn (ADR 0009) and worktree-remove-failure
  leak," base `integration/path-to-mac-2026-08-15` — a different PR merged
  onto the integration branch, not PR #111 or `main`.
- `git merge-base --is-ancestor 251a6f1... bceed96...` → exit 0 (is an
  ancestor); reverse direction → exit 1 (not an ancestor). Confirms `251a6f1`
  is a real, earlier point on the same lineage, not a fabrication, and not
  the tip.
- `git log 251a6f1..bceed96` → exactly two commits missed:
  `11b138c` and `bceed96` itself. Matches the critic's count and identity
  exactly.

**Ground 2/3: out of scope or style?** No — a specific-commit citation
check, checkable today, the contract's own named category.

**Ground 4: said elsewhere?** No other location cites a different, correct
SHA for PR #111's head; the same wrong SHA appears in all three places
(frontmatter, header, §3.3) with no correction elsewhere.

**Ground 5: severity — is `error` correctly calibrated, or inflated?**
Here I part ways with the critic. `error` for F2 looks inherited from F1's
severity by association (both concern PR #111's state) rather than earned
on F2's own downstream impact. Applying the same bar I used to *uphold*
F1's `error` rating — "would a Work reading this literally be blocked or
produce a wrong artifact" — F2 fails that bar in the other direction nobody
building T0 resolves "the actual implementation base" (T2-06) by checking
out the historical audit SHA `251a6f1`; that field is pure provenance, not
an instruction any Work executes against. The critic's own conclusion
confirms this: "nothing on the two missed commits contradicts anything the
proposal claims," and "the citation only matters as a historical audit
trail at this point." Compare this directly to **F3 in this same critic
report** — also a real hash, also misattributed to the wrong field/PR, also
zero content impact, also "the section survives without a design change" —
which the critic themselves scored `warning`. I can't find a principled
difference between F2 and F3's actual downstream risk: both are dead/
mislabeled provenance pointers with no propagation into normative content
(unlike F1's ~20 locations spanning decision-register entries and
acceptance criteria). F2 citing a *different, real PR's* merge commit
rather than a non-existent one is a more embarrassing mistake, but
embarrassment isn't the axis's severity currency — propagated consequence
is, per the contract's own framing ("dangerous... because sections are
built on it"). No section is built on the specific SHA. I'm downgrading
F2 to `warning`, matching F3's calibration for the same-shape defect.

**Verdict: CONFIRMED** as a factual finding; **severity downgraded from
`error` to `warning`** — inherited-severity-by-association from F1, not
independently earned by F2's own (zero) downstream impact, and
inconsistent with the critic's own, correct `warning` calibration of the
structurally identical F3.

---

## F3 — frontmatter, `supersedes.revision` does not point to the predecessor proposal — **CONFIRMED**, severity unchanged (warning)

**Ground 1: factually wrong?** Re-verified independently:

- `git show a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6:reference/proposal-tui-t-series.md`
  → `fatal: path ... does not exist in 'a5fb875...'`. Confirmed the blob
  link 404s.
- `git log --follow --diff-filter=A -- reference/proposal-tui-t-series.md`
  → single hit, `a9a25fa68938323d9585edc687fbf0e965084c2e`, "Execution-surface
  test (owner ruling): workflow vs CLI surface vs operator skill,"
  `2026-08-11T15:28:30Z`. This is the actual commit that introduced the
  predecessor file.
- `git show a9a25fa...:reference/proposal-tui-t-series.md | grep audit_revision`
  → `audit_revision: a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`. Confirms
  `a5fb875` is the predecessor's own audit-basis pin, not a commit at which
  the predecessor file exists — exactly the mislabeled-field the critic
  describes.

**Ground 2/3: out of scope or style?** No — a specific-commit citation
check.

**Ground 4: said elsewhere?** I checked §3.1's disposition table and §23's
Adopted/Revised/Rejected lists myself (the two places most likely to
duplicate or correct this pointer) — neither restates a commit hash for the
predecessor, both describe changes in prose only. No rescue, matching the
critic's own scope note that fidelity's job (checking the prose against
`a9a25fa`'s real text) is separate from this citation defect.

**Ground 5: severity?** `warning` is correctly calibrated and I would
resist pushing it either direction. Not `error`: no normative or
prescriptive content depends on this field resolving (unlike F1); a reader
loses only the one-click diff-opening convenience, not the argument itself,
matching the critic's own "does the section survive" analysis, which I
independently reached the same way. Not `info`: unlike a merely imprecise
citation, this one is *completely* broken (the link 404s outright, not "one
of three sources slightly off"), so it clears `warning`'s bar the same way
F2 now does under my revised reading — the two are peers.

**Verdict: CONFIRMED**, severity `warning` unchanged.

---

## Spot-check of "what checked out" (no finding, per critic)

Since a false negative here would silently prop up "does the section
survive" for all three findings, I independently re-ran a sample rather
than trusting it wholesale — this is also where I hit and corrected the
`origin`-vs-`github` remote trap described in the method note above:

- `git diff 242abe3 github/main -- src/tui.rs` → empty. Confirmed
  byte-identical across the 49-commit gap, matching §3.2's claim.
- `git rev-list --count 242abe3..github/main` → `49`. Confirms the critic's
  "behind real main by 49 commits" figure exactly.
- `git show github/main:src/web.rs` → path does not exist. Confirms
  dashboard deletion holds on live `main`.
- `git show github/main:.sergeant/index.md` → contains the literal text
  "23 packages," and `git diff 242abe3 github/main -- .sergeant/index.md`
  is empty. Confirms the workflow-count claim independently of the
  critic's own count method.
- `docs/adr/` at `242abe3`: exactly `0001`–`0011` plus `README.md` — ADRs
  `0005`–`0011` are seven files, confirmed present, matching every citation
  in §3.4/§24.2.
- `git show github/main:src/api.rs` and `src/cli.rs`: `POST /work/{id}/reap`
  → `reap_work`, `GET /retained` → `list_retained`, CLI verbs `sgt work
  retained`/`sgt work reap --yes` all present and wired exactly as §12.4
  and §13.7 describe. (My first attempt against the stale `origin` remote
  found none of this — see method note.)
- `git show github/main:src/cli.rs` for the Doctor check: `fs_locking`
  module, `Reliability` enum (`Reliable`/`Unreliable`/`Unknown` arms),
  `platform::fs_locking::detect_for_path` wired into a `Check` — present
  exactly as §12.3/§19.6 describe.
- `gh pr view 105` / `gh pr view 69` → both `MERGED`
  (`2026-08-14T23:46:04Z`, `2026-08-13T20:51:17Z`); `gh issue view 15` /
  `21` → both `CLOSED` (`2026-08-15T02:42:48Z`); `gh issue view 120` →
  still `OPEN`, `closedAt: null`. All match the critic's citations exactly.
- Cargo pins: `Cargo.toml` → `ratatui = "0.30.2"`; `Cargo.lock` resolves
  `ratatui 0.30.2` and a second `crossterm 0.29.0` entry alongside an
  unrelated transitive `crossterm 0.28.1`. Matches §8's version claims and
  the critic's disambiguation of the two `crossterm` entries.

No discrepancies found in the sample. The critic's "what checked out"
section holds.

---

## Summary

All three findings survive adversarial refutation on factual correctness,
scope, style, and cross-reference grounds — none is false, mischaracterized,
or rescued elsewhere in the proposal. Severity is where this pass diverges
from the critic: **F1's `error` is upheld and, if anything, underargued** —
the stale "never treated as merged" framing sits in prescriptive
decision-register and section text a dispatched Work would read as binding,
not narrative background, which is a materially different (and worse) risk
than the FOUNDATION-1 precedent's analogous finding that was calibrated to
`warning`. **F2 is downgraded from `error` to `warning`** — its severity
reads as inherited from F1 by topical association (both concern PR #111)
rather than earned by F2's own downstream impact, and the critic's own
analysis of F2 ("nothing... contradicts anything the proposal claims," "only
matters as a historical audit trail") is structurally identical to their
analysis of F3, which they correctly scored `warning` — treating two
same-shape defects at different severities within the same report is the
inconsistency this refutation pass corrects. **F3's `warning` is upheld**
as-is. One additional corroborating location for F1 (§24.2, line 2277)
surfaced during independent re-verification, strengthening rather than
weakening the finding. The `origin`-remote trap documented above is worth
carrying forward to future refuters on this repository: local `origin` is
a stale file-path alias, not the live repository — only `github/main`
(fetched fresh) verifies current `main` state.
