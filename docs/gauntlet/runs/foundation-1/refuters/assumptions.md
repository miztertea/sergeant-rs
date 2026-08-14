# FOUNDATION-1 — refuter: assumptions

Adversarial refutation per `docs/gauntlet/contracts/FOUNDATION-1.md`, axis 3
(**assumptions**), of `docs/gauntlet/runs/foundation-1/critics/assumptions.md`.
I did not write the proposal and did not write the critic report. Every
claim below was re-verified independently against the repository — `gh`
against `miztertea/sergeant-rs` and local git — rather than trusted from the
critic's evidence.

## Method note

For each of the critic's three findings I re-ran the checks from scratch
(fresh `gh issue view`/`gh api` calls, fresh `git log`/`git merge-base`,
fresh file reads) and additionally tried to find grounds the critic didn't
consider: an alternate, non-GitHub reading of "closed"; a cross-reference
elsewhere in the proposal that already qualifies the claim; a timezone or
count error in the critic's own math; and whether the citation set the
critic used to trace "the fixing PRs" was actually complete. All three
findings survive. One of them (F1) is more strongly supported than the
critic's own writeup shows, not less.

---

## F1 — §2.2, "closed six issues" — **CONFIRMED**, severity unchanged (warning)

**Ground 1: is it factually wrong?** Independently re-verified, not trusted.

```
gh issue view {16,70,83,86,87,91} --json number,title,state,closedAt
```

returned `"state":"OPEN","closedAt":null` for all six, matching the critic
exactly. I additionally pulled the timeline for #16 and #91 myself (6 and 7
events respectively, zero `closed`/`reopened` events in either) — same
conclusion as the critic's exhaustive pass, on an independent sample.

I went further than the critic's evidence trail. The critic traces "the PRs
that fixed each one" to #92, #93, #98 — but those three PRs only account for
five of the six issues (#92 closes #83/#70, #93 closes #16, #98 closes
#86/#87). #91 is missing from that set. I found the actual fixing PR for
#91: **#101** ("Fix #91: per-run unique Docker container names, panic-safe
cleanup"), `baseRefName: integration/cross-platform-2026-08-14` — same
branch, same mechanism, still unmerged. I also pulled up **#89**, the
tracking PR for the whole sprint into `main` (`state: OPEN`, `mergedAt:
null`), whose own body states in a table: "**Closed on merge to main:**
#16, #70, #83, #86, #87" — future tense, five of the six (it doesn't even
list #91 in that line), written by whoever ran the sprint, confirming in
the sprint's own paper trail that these were understood as *not yet*
closed at the time of writing. This is stronger corroboration than the
critic assembled, not weaker — the finding survives re-verification with a
wider evidence base than the original.

`git merge-base --is-ancestor integration/cross-platform-2026-08-14
origin/main` — I ran this myself: exit non-zero, "NOT ANCESTOR". Confirmed.

**Ground 2: out of scope?** No. This is a file-and-issue-state factual claim,
squarely inside axis 3. It is not grading an unbuilt implementation (it's a
claim about GitHub issue state, checkable today), not re-litigating one of
the seven decisions, and not implementation design.

**Ground 3: style preference dressed as defect?** No. "Closed" vs. "open" is
a binary, checkable fact, not a phrasing preference.

**Ground 4: does the proposal already say this elsewhere?** I checked all
eight sections for language that would rescue "closed six issues" in §2.2,
not just §7. §7's "the integration branch is unmerged" is real
(`reference/proposal-foundation-rationalization.md:391-392`, confirmed by my
own `grep`) but it appears under "Explicit non-goals," framed around *why
this isn't a migration story* — it never references §2.2, never references
issue numbers, and a reader of §2.2 alone has no cue to go looking for it.
I also checked §3.1 and §3.6 (the only other places these or adjacent issue
numbers appear) and found no qualifying language there either — §3.1 never
mentions issue closure at all, and §3.6 is about #11/#16 dashboard freeze
timing, a different claim (see F2). So §7 doesn't functionally cover this
gap; it's adjacent, not a cross-reference. I tried this defense and it
doesn't hold.

I also tried a defense the critic didn't raise: maybe "closed" is sprint-internal
shorthand (a local journal/Work-tracking sense) rather than GitHub issue
state. I checked — `grep` across
`docs/gauntlet/runs/cross-platform-2026-08-14/{plan.md,lessons.md}` for
"closed" turns up nothing resembling that usage, and the parenthetical
`(#16, #70, #83, #86, #87, #91)` directly beside "closed six issues" is
unambiguously GitHub issue-number notation used identically everywhere else
in the document. This alternate reading doesn't survive either.

**Ground 5: is the severity inflated?** No — if anything it's already
conservatively pitched. Applying the "would a Work be blocked or produce a
wrong artifact" bar: §2.2 is evidentiary/methodological framing text
("This matters methodologically..."), not an instruction or acceptance
criterion any downstream Work consumes. No other section (checked via
`grep` for all six issue numbers across the whole document) depends on
these six being *closed* rather than *fixed-on-integration-branch* — §3's
defects are argued from the fixing commits' existence, which is true
regardless. `warning`, not `error`, is correctly calibrated. I don't find
grounds to downgrade to `info` either: unlike F2 (a one-day date slip) and
F3 (one of three supporting citations imprecise), F1 is a categorical
true/false claim ("closed") that is flatly false against the tracker right
now, in a document whose §2.2 exists specifically to establish that this
round's evidence is measured. That's a meaningfully different kind of miss
than F2/F3's imprecision, and `warning` (one notch above `info`) reflects
that difference correctly.

**Verdict: CONFIRMED.** Survives all five refutation grounds. Recommend
`§2.2` be corrected to "fixed" or "landed on the integration branch" per the
critic's suggested fix — this refutation pass turned up additional
corroborating evidence (PR #101, PR #89's own language) rather than any
weakening of the claim.

---

## F2 — §3.6, "#11 was fixed on 2026-08-13" — **CONFIRMED**, severity unchanged (warning)

**Ground 1: factually wrong?** Re-verified independently:

- `gh issue view 11 --json closedAt` → `2026-08-14T09:33:22Z`. Matches the
  critic's figure exactly.
- I located the merging commit myself via `git log --all --grep` rather than
  trusting the critic's hash, and found the same one: `6e5fbbf`, "Merge pull
  request #76 from miztertea/fix/26-11-tui", timestamped
  `2026-08-14 04:16:05 +0000` in my own `--date=iso-local` read (equivalent
  to the critic's `2026-08-14T00:16:05-04:00` — same instant, different
  display convention; I cross-checked the arithmetic: 04:16:05 UTC − 4h =
  00:16:05, consistent).
- I tried the one defense the critic's own report doesn't fully rule out:
  is there *any* timezone offset under which either timestamp reads as the
  13th? `closedAt` is 09:33:22Z; the merge commit is 04:16:05 UTC. The
  earlier of the two would need to be UTC-4:16 or a more extreme negative
  offset (i.e., west of roughly UTC-4) to land on the 13th — no real-world
  timezone in use by this repo's commits (`-04:00` is the convention per the
  critic's own scan of `%ad`) reaches that far. This defense fails.
- I confirmed `north-star-arbitration-2026-08-11.md:196` independently via
  direct read — dated 2026-08-11 in the filename and consistent with commit
  history around that date. The critic's underlying point (fix landed after
  the freeze argument was made) holds under my re-check.

**Ground 2/3: out of scope or style?** No — a specific calendar date claim,
checkable, not a design or scope question.

**Ground 4: said elsewhere?** No other section restates or corrects this
date; I grep'd for "2026-08-13" and "08-13" across the proposal — no other
occurrence.

**Ground 5: severity?** `warning` is right, not inflated — it's a plain
factual date error, but as the critic notes and I independently confirm,
the sentence's actual argument ("fixed after the freeze argument, freeze
never enforced") is true on either the 13th or the 14th, so it doesn't rise
to blocking severity. I don't find grounds to push it to `error` (nothing
downstream depends on the specific day) or down to `info` (unlike F3, this
isn't "one of three citations imprecise" — it's the citation itself being
wrong, just non-consequentially).

**Verdict: CONFIRMED.**

---

## F3 — §5.5, "m2 t7" cited as an affected pinned test — **CONFIRMED**, severity unchanged (info)

**Ground 1: factually wrong?** I read `tests/m2_daemon_api.rs:1535-1571`
myself rather than trusting the critic's paraphrase. Confirmed line by
line: the function is `t7_cli_end_to_end_auto_spawn_and_second_daemon_fails_closed`;
its auto-spawn assertion fires on the `sgt run` call ("No daemon running:
`sgt run` auto-spawns one and submits" — matches the comment in the actual
source); the subsequent `sgt work list --json` call runs against a daemon
`run` already spawned, so it does not itself exercise `work list`'s
cold-start auto-spawn path. I also independently confirmed via `grep` that
§5.5 places `work show|list|transcript` in the **no-spawn** set (line 145
and line 308 of the proposal both list it), so the critic's framing of
"which side of the split this verb falls on" is accurate. This test, as
written, would not break under §5.5 on the strength of its `work list`
call — the critic is right.

I tried to refute this by checking whether some other assertion in the test
implicitly depends on `work list` triggering auto-spawn — e.g., if the
daemon were somehow stopped between `run` and `work list`. Re-reading the
function: no `stop_daemon` call appears until after all assertions, so the
daemon is live throughout. This defense fails; the critic's read is correct.

**Ground 2/3: out of scope or style?** No — this is "does test X actually
exercise behavior Y," a factual/enactability-adjacent claim properly graded
under assumptions since it's about what the cited test currently does, not
about how the future implementation should be built.

**Ground 4: said elsewhere?** No — §5.5 doesn't hedge this citation
anywhere else in the document; I checked both occurrences of "m2 t7"
(lines 145 and 321) and neither qualifies it.

**Ground 5: severity?** `info` is correctly calibrated, and I'd resist any
push to raise it to `warning`. The sentence cites three tests as blast
radius; two of the three (`tests/m6_surfaces.rs:414`,
`tests/m8_estate_cli.rs:1080`) I independently re-read and confirm do pin
exactly what §5.5 claims — `m6_surfaces.rs:414` asserts bare `sgt`
auto-spawns, and `m8_estate_cli.rs:1080` runs `status` specifically to
auto-spawn and asserts exactly one daemon afterward, both of which are
verbs §5.5 explicitly moves to the no-spawn set. The general claim (removing
auto-spawn from observation verbs has pinned-test blast radius) is correct
and two-thirds evidenced; only one supporting example among three is
mischosen. That's a precision defect in a citation list, not a wrong claim
about what the change does — `info` fits.

**Verdict: CONFIRMED.**

---

## Summary

All three findings in `critics/assumptions.md` survive adversarial
refutation on all five grounds (factual correctness, scope, style,
cross-reference, severity calibration), re-verified independently rather
than trusted from the critic's evidence. None of the three severities move.
F1 comes out of this pass *more* solidly evidenced than the critic
presented it (PR #101 and PR #89's own "Closed on merge to main" language,
neither cited in the original report). No grounds were found — factual,
scope, style, cross-reference, or severity — on which any of the three
should be knocked down or downgraded.
