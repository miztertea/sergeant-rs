# T-SERIES-1 — refuter: assumptions

Adversarial refutation per `docs/gauntlet/contracts/T-SERIES-1.md`, axis 3
(**assumptions**), of
`docs/gauntlet/runs/t-series-1/critics/assumptions.md`. I did not write the
proposal and did not write the critic report. Every claim below was
re-verified independently — fresh `gh` calls against
`miztertea/sergeant-rs`, fresh `git log`/`git show`/`git merge-base`/`git
ls-tree` against local history, and fresh reads of
`reference/proposal-tui-t-series.md` — rather than trusted from the critic's
evidence trail.

## Method note

For each of the critic's three findings I re-ran the checks from scratch and
additionally tried grounds the critic's writeup doesn't fully close: an
alternate non-merge reading of "conditional"; a hedge elsewhere in the
document that would rescue the stale framing; whether the critic's own
"safety-relevant" characterization of a cited commit holds up against that
commit's actual diff rather than just its title; and whether severity, not
just correctness, is calibrated to actual consequence. Two findings (F1, F3)
survive exactly as reported. One (F2) survives on the facts but not at the
severity claimed — I found grounds the critic didn't check that argue for a
downgrade.

---

## F1 — proposal-wide, "PR #111 is an open, unmerged, concurrent dependency" — **CONFIRMED**, severity unchanged (error)

**Ground 1: factually wrong?** Independently re-verified.

```
gh pr view 111 --json state,mergedAt,mergeCommit,baseRefName,headRefOid,headRefName
```

returns `state: MERGED`, `mergedAt: 2026-08-15T15:28:02Z`, merge commit
`3a46b87c17d249655708ed5ac32f6704738776cf`, base `main` — matches the critic
exactly. `gh issue view 120 --json state,closedAt` independently confirms
`state: OPEN`, `closedAt: null`, so the proposal's line 1916 ("the gate
defect in PR #111 must be resolved before its result is trusted") is
correctly *not* flagged stale by the critic — I checked this myself rather
than assume the critic's silence on it was an oversight.

I re-grepped the whole document myself for `"PR #111"`, `"integration
branch"`, `"conditional"`, `"if.*merge"` independent of the critic's search
terms and got the same ~20-location set: frontmatter `integration_review`
(no `merged` field), header L49, §3.3's full body (including the
`not treated as merged` sentence and Decision T2-06), §6.1 item 8, §6.2's
non-goal, §7.2, §10.5, §12.3, §12.4 (heading + Decision T2-46), §13.3,
§13.7, §13.10, §15.3, §16.1, §19.6, §19.8 (heading + body), §20.4, register
lines for T2-06/T2-46, acceptance items 54–55, and falsifier 21. I spot-read
eight of these directly in the file (§3.3, §12.4, §19.8, items 54/55,
falsifier 21, and both register lines) rather than trusting the critic's
quoted excerpts, and every one matches the critic's transcription
verbatim.

**Ground 2: out of scope?** No — a merge-state fact, squarely inside axis 3,
not grading an unbuilt implementation and not re-litigating the North Star
gate ruling (the contract's own non-goals list excludes only that specific
ruling, not ordinary PR-state facts).

**Ground 3: style preference dressed as defect?** No — "merged" vs. "open" is
binary and checkable, not phrasing.

**Ground 4: said elsewhere?** I grepped the whole document for hedge language
that would rescue this (`"as of writing"`, `"may have merged"`, `"by the
time"`, `"re-check"`, `"re-verify"`, `"gh pr view"`) — zero hits. Every
occurrence of "unmerged"/"not treated as merged"/"concurrent" in the
document is itself part of the stale claim, not a qualifier of it. This
defense fails; the critic's implicit "no rescue found" holds under an
independent search with different terms.

**Ground 5: severity inflated?** No. I considered downgrading to `warning`
on the critic's own argument that no section's *design* changes — only
prose. But the contract's own axis description names this exact failure
mode ("sections are built on it") as the most dangerous kind, and two
concrete consequences push this past a prose nit: (a) Decision T2-06 gates
T0 — the very first dispatched unit of work — on resolving "merged or
explicitly excluded," a question that is already answered; a Work executing
T0 in good faith could burn a cycle re-litigating settled state or, worse,
misread the unresolved-looking register entry as license to pin an
outdated base. (b) Acceptance item 55 ("If PR #111 does not land, no
retained/reap placeholder or claim remains") is not merely stale, it
describes a branch of the acceptance contract that can now never fire —
that's a defect in the contract itself, not just its prose, since a
never-fireable acceptance clause is dead weight in exactly the document
whose job is to be executable. `error` is correctly calibrated; I don't find
grounds to raise or lower it.

**Verdict: CONFIRMED.** Survives all five grounds, severity unchanged.

---

## F2 — proposal-wide, "PR #111 at `251a6f1`" cites the wrong commit — **CONFIRMED on the facts, severity should be downgraded from `error` to `warning`**

**Ground 1: factually wrong?** Independently re-verified, and I went one step
further than the critic's own evidence.

```
gh pr view 111 --json headRefOid   → bceed965c24de7fa781001e3bd7835d8ef58b139
gh pr view 122 --json mergeCommit,baseRefName,title
  → mergeCommit: 251a6f1c09caee95fcac30f724dab0ece166cae0
  → baseRefName: integration/path-to-mac-2026-08-15
  → title: "W6c: shipping-gate fixes — reap preview auto-spawn (ADR 0009)
     and worktree-remove-failure leak"
```

Confirmed exactly: `251a6f1` is PR #122's merge commit onto the integration
branch, not PR #111's head. `git merge-base --is-ancestor 251a6f1 bceed96`
succeeds (real ancestor); the reverse fails — same relationship the critic
found. `git log --oneline 251a6f1..bceed96` shows exactly the two commits
the critic names: `11b138c` and `bceed96` itself, both landing
`2026-08-15T15:17`–`15:18Z`, after the cited pin.

**Where I went further:** the critic infers `bceed96` is "exactly the kind
of late safety-relevant fix a reviewer would want to know they hadn't seen
yet" from its *title* alone ("Retrospective §1.3: the merge check caught a
sweep about to destroy evidence"). I ran `git show --stat` on both missed
commits myself. Both touch exactly one file each:
`.../runs/path-to-mac-2026-08-15/retrospective.md` — `11b138c` adds 321
lines of retrospective prose, `bceed96` edits 37 lines of the same file. **Neither
touches any source file, test, API route, or CLI verb.** They are
session-retrospective documentation commits about a branch-cleanup lesson,
not code fixes. The critic's own "safety-relevant fix" framing overstates
what these commits actually are — they are not a fix at all, safety-relevant
or otherwise, and nothing in either commit could contradict or extend the
five candidate surfaces §3.3 lists. I checked this directly rather than
inferring from commit-message titles, which the critic's report does not
do.

**Ground 2/3: out of scope or style?** No — a wrong-commit citation is a
plain factual claim.

**Ground 4: said elsewhere?** No rescue — the same three locations (front
matter, header, §3.3 body) all repeat the same wrong hash with no
qualifying language nearby; I re-read all three myself.

**Ground 5: severity — this is where I diverge from the critic.** The
critic assigns `error`, the same tier as F1, but the critic's own closing
sentence for F2 undercuts that: "the citation itself is wrong and should be
corrected... now that the whole 'which revision did we review' question is
superseded by F1 anyway... it only matters as a historical audit trail at
this point." That is a `warning`-grade consequence description, not an
`error`-grade one, and it's the same description the critic uses to justify
`warning` on F3 ("real hash, wrong field, dead link... affects traceability
but not the argument's content"). I checked for a reason F2 should be
treated differently from F3 despite the parallel structure and found none:

- **No tooling depends on the field.** `grep -rn "integration_review"` across
  `src/`, `scripts/`, and all of `docs/`/`reference/` turns up only the
  proposal itself and the critic report — nothing parses this frontmatter
  mechanically. A wrong hash here is read by a human clicking a link, exactly
  like F3's broken `supersedes` link.
- **The content it grounds is independently confirmed unaffected** — by
  both the critic and my own re-check of the two missed commits' diffs — so
  unlike a citation that props up a false substantive claim, this one
  props up a true claim with an imprecise pointer.
- **The one basis for treating it as more severe than F3** — that the
  missed commits might carry safety-relevant content a reviewer needs —
  does not survive checking the actual diffs, which I did and the critic's
  report did not.

On the "would a Work be blocked or produce a wrong artifact" bar from the
FOUNDATION-1 precedent: no downstream Work consumes this specific hash as an
instruction or acceptance criterion (checked: it appears only in frontmatter
metadata, the header, and §3.3's illustrative code block, never in a
Decision, register line, or acceptance item). That is a materially different
consequence profile than F1, whose stale conditionals reach into Decision
T2-06 and a live acceptance criterion. I recommend downgrading F2 to
`warning`, on par with F3, rather than `error`.

**Verdict: CONFIRMED (facts), severity revised: `error` → `warning`.**

---

## F3 — frontmatter, `supersedes.revision` does not point to the predecessor proposal — **CONFIRMED**, severity unchanged (warning)

**Ground 1: factually wrong?** Independently re-verified via a different
path than the critic used.

```
gh api repos/miztertea/sergeant-rs/contents/reference/proposal-tui-t-series.md?ref=a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6
  → 404 Not Found
```

I used the GitHub Contents API directly (rather than the critic's local
`git show`) to reproduce the exact reader-facing failure the header's blob
link would hit — confirmed 404, i.e., the linked page genuinely does not
resolve, not just a local-git technicality. Locally, `git show
a5fb875:reference/proposal-tui-t-series.md` also fails ("exists on disk, but
not in a5fb875"), and `git ls-tree -r a5fb875 --name-only | grep
proposal-tui` returns nothing — the path is fully absent from that tree,
confirmed two independent ways.

`git log --follow --diff-filter=A` for the file's true introducing commit
returns `a9a25fa68938323d9585edc687fbf0e965084c2e` ("Execution-surface test
(owner ruling)...", 2026-08-11T15:28:30Z) — matches the critic. I read that
commit's own frontmatter directly: `audit_revision:
a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6`. This confirms the critic's
specific mechanism claim, not just the surface symptom: the cited hash is
real, but it's the *predecessor proposal's own audit-basis pin* — the
commit the predecessor proposal was itself auditing against — copied into
the current proposal's `supersedes.revision` field, which should instead
name the commit where the predecessor *file* actually existed (`a9a25fa`).

**Ground 2/3: out of scope or style?** No — this is a specific, checkable
git-history claim, and it's a fully mechanical error (wrong field populated
with a real-but-misapplied hash), not a style judgment.

**Ground 4: said elsewhere?** The critic correctly scopes this to the
citation itself and explicitly declines to duplicate the fidelity axis's job
of checking whether §3.1/§23's prose description of the predecessor is
accurate — I agree with that scoping. I checked whether §3.1 or §23
independently supply a working pointer to the predecessor that would make
the broken header link redundant rather than load-bearing; they describe
changes in prose only, no alternate commit citation. The broken link is the
only mechanical path a reader has to the exact predecessor diff; no
rescue.

**Ground 5: severity?** `warning` is right. Applying the same consequence
bar used for F2 above: nothing downstream (no Decision, no register line, no
acceptance item) depends on this specific hash resolving; it affects only a
reader's ability to click through to the predecessor text, and the
proposal's own prose (§3.1, §23) carries the substantive disposition
argument independent of the link. I don't find grounds to raise it to
`error` (no section's claims depend on it) or lower it to `info` (unlike a
mere imprecision, the link is categorically broken — 404, not just
approximate).

**Verdict: CONFIRMED.**

---

## Summary

All three findings survive on the facts, independently re-derived via fresh
`gh` and `git` queries rather than trusted from the critic's citations. F1
and F3 also survive at the severities the critic assigned — I found no
grounds, factual or consequential, to move either. F2 survives factually
(the cited commit really is the wrong one, a different PR's merge commit
mislabeled as PR #111's head) but not at the severity claimed: checking the
two commits the critic said were missed — something the critic's own report
does not do — shows both are documentation-only retrospective edits to a
single markdown file, not the "safety-relevant fix" the critic's title-only
inference suggested, and no Decision, register line, or acceptance
criterion consumes the specific hash the way F1's stale conditionals reach
into T2-06 and acceptance item 55. F2 is recommended for downgrade from
`error` to `warning`, bringing it in line with F3's materially identical
"real hash, wrong role, doesn't change the argument" shape. Net effect on
the unit's bounded outcome: one `error`-severity finding (F1) and two
`warning`-severity findings (F2 revised, F3) survive on the assumptions
axis — all confirmed as mechanically correctable without any section's
underlying design changing, consistent with the critic's own "does the
section survive?" conclusions.
