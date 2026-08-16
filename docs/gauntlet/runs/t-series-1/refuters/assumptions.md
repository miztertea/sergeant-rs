# T-SERIES-1 — refuter: assumptions

Adversarial refutation per `docs/gauntlet/contracts/T-SERIES-1.md`, axis 3
(**assumptions**), of `docs/gauntlet/runs/t-series-1/critics/assumptions.md`.
I did not write the proposal and did not write the critic report. Every
claim below was re-verified independently — fresh `gh` calls, fresh `git
show`/`git log`/`git merge-base`, and a fresh read of
`reference/proposal-tui-t-series.md` itself — rather than trusted from the
critic's evidence.

## Method note

For each of the critic's three findings I re-ran the checks from scratch and
additionally tried grounds the critic's own writeup doesn't fully close out:
an independent grep sweep for every "PR #111" / conditional occurrence to
check the critic's ~20-row table for completeness; a byte-level re-derivation
of the two disputed commit hashes (F2) rather than trusting the critic's
`git show` output; a check for any cross-reference elsewhere in the document
that would rescue F3's dead link; and a re-verification, from the live
`github/main` remote rather than this session's local branch tip, that the
retained/reap API and filesystem-reliability check the critic says "already
shipped" actually exist there — my first attempt at this check, against the
local checked-out `src/`, found nothing and would have produced a false
refutation of the critic's central claim had I not caught that the local
branch tip is 49 commits behind real `main` and re-run it against `github/main`,
exactly as the critic's own method note describes doing. All three findings
survive. None of my independent checks turned up evidence undermining any of
them.

---

## F1 — proposal-wide, "PR #111 is an open, unmerged, concurrent dependency" — **CONFIRMED**, severity unchanged (error)

**Ground 1: is it factually wrong?** Independently re-verified, not trusted.

```
gh pr view 111 --json state,mergedAt,mergeCommit,baseRefName,headRefOid,headRefName
```

returned `state: MERGED`, `mergedAt: 2026-08-15T15:28:02Z`, merge commit
`3a46b87c17d249655708ed5ac32f6704738776cf`, base `main`, head
`bceed965c24de7fa781001e3bd7835d8ef58b139` — matches the critic exactly.

**Ground 2 (completeness of the critic's own evidence):** I independently
grepped the whole proposal for `PR #111`, `integration branch`, `if.*merge`,
`conditional`, and `never treated as merged` rather than trusting the
critic's location list. My sweep surfaced every row in the critic's ~20-row
table, plus a small number of additional occurrences the critic's table
doesn't list as separate rows: line 10 ("accounts for the current open
integration branch") and line 41 ("current integration branch's candidate
retained-state surfaces") in the frontmatter description, and the §22
register line for **T2-07** (line 2064, "Consume retained/reap only if its
real surface lands") sitting immediately beside the T2-06 register line the
critic's table does cite. These are minor omissions from an otherwise
thorough table, not counterexamples — each resolves exactly like its
neighbors (mechanical correction, same underlying content), so they don't
change the verdict. If anything they strengthen F1's "volume and spread"
characterization: the defect is even more pervasive than the critic's own
count shows. Line 296 ("PR #111's source patches and contract record," a
citation-list entry) and line 1916 (the still-open #120 forward requirement,
which the critic correctly excludes as *not* stale) are the only other hits
from my sweep, and both are correctly out of scope for the same reasons the
critic gives for adjacent material.

**Ground 3: does the underlying design survive, independent of the critic's
say-so?** I re-verified this against the live repository myself rather than
accepting the critic's "what checked out" claims. `git show
github/main:src/api.rs` (fetched fresh via the `github` remote, since this
session's local branch tip is 49 commits behind real `main` — confirmed with
`git rev-list --count HEAD..github/main` = 49) contains `.route("/work/{id}/reap",
post(reap_work))` and `.route("/retained", get(list_retained))` at lines
397–398, and `async fn reap_work` / `async fn list_retained` are real
handlers (lines 2309, 2400). `git show github/main:src/cli.rs` contains
`WorkCommand::Retained` and `WorkCommand::Reap { id, yes }` (lines 364, 373).
`git show github/main:src/cli.rs` also contains the filesystem-reliability
Doctor check: `crate::platform::fs_locking::detect_for_path(data_dir)` and a
`Reliability` enum with `Reliable`/`Unreliable`/`Unknown` arms (lines
2500–2527). All of this matches §3.3, §12.3–§12.4, and §13.7's descriptions
field-for-field. (Note for anyone re-running this check: grepping the local
checked-out `src/` at this session's HEAD finds none of this, because HEAD
predates PR #111's merge by 49 commits — the check must run against
`github/main`, not local HEAD, exactly as the critic's method note says.)

**Ground 4: severity.** `error` is correctly calibrated. Applying the "would
a Work be blocked or produce a wrong artifact" bar: T0 (§20.1) is explicitly
gated on "PR #111 is either merged or explicitly excluded" (Decision T2-06);
a Work dispatched against the proposal's literal text would either stall on
a disposition that is already settled, or worse, carry forward "if merged"
hedges into shipped UI copy and acceptance criteria that a downstream Work
would then have to un-hedge itself. That is a wrong artifact, not a cosmetic
miss, which clears the bar for `error`. I don't find grounds to downgrade to
`warning`: unlike F3 (a dead link with no downstream consumer), F1's claim is
load-bearing across a decision the T0 phase gate depends on.

**Verdict: CONFIRMED.** Severity unchanged (`error`).

---

## F2 — §3.3 / frontmatter, "PR #111 at `251a6f1`" cites the wrong commit — **CONFIRMED**, severity unchanged (error)

**Ground 1: factually wrong?** Re-derived independently rather than trusting
the critic's `git show` output.

```
git show -s --format='%H %ad %s' 251a6f1c09caee95fcac30f724dab0ece166cae0
```
→ `Merge pull request #122 from miztertea/lane/w6c-gate-fixes`, 2026-08-15
11:05:36 -0400.

```
gh pr view 122 --json number,title,mergeCommit,baseRefName,state
```
→ `mergeCommit.oid: 251a6f1c...`, `baseRefName:
integration/path-to-mac-2026-08-15`, `state: MERGED`. Confirms the cited SHA
is PR #122's merge commit onto the integration branch, not PR #111 itself,
exactly as the critic reports.

```
git merge-base --is-ancestor 251a6f1c... bceed965c24de7fa781001e3bd7835d8ef58b139
```
→ exit 0 (ancestor), confirming `251a6f1` is a real, earlier point on the
same branch rather than a fabricated or unrelated hash — matches the
critic's characterization precisely.

I additionally re-derived the "two missed commits" claim independently:
```
git log --format='%H %ad %s' 251a6f1c...^..bceed965c...
```
returns exactly `11b138c` and `bceed965` (the critic's own list, reproduced
without relying on their enumeration).

**Ground 2 (an angle the critic didn't need to try, but I did): do the two
missed commits actually touch nothing the proposal claims?** I read both
commits' full diffs myself rather than trusting the critic's "neither
touches those surfaces" assertion. `git show --stat 11b138c` and `git show
--stat bceed965c...` both touch exactly one file each:
`docs/gauntlet/runs/path-to-mac-2026-08-15/retrospective.md` (321 insertions
for the first; 29 insertions/8 deletions for the second). Neither commit
touches `src/api.rs`, `src/cli.rs`, or any file under `src/`. This
independently confirms the critic's claim rather than merely repeating it.

**Ground 3: severity.** `error` fits for the same reason as F1: this isn't
just a stale-but-harmless pointer, it's a citation that resolves to the
*wrong artifact* — a different, sub-lane PR (#122) misattributed as PR
#111's reviewed head. A reader or downstream Work following the citation to
audit "what was reviewed" would open the wrong diff. I considered whether
this should collapse into F1 (same root cause, redundant) rather than stand
as its own `error`-severity finding, and conclude it shouldn't collapse: F1
is about staleness (a true-then, false-now conditional), F2 is about a
citation being *wrong even at the time it was written* (mislabeling PR
#122's merge commit as PR #111's head) — a different defect class that
would still be a finding even if PR #111 had never merged at all.

**Verdict: CONFIRMED.** Severity unchanged (`error`).

---

## F3 — frontmatter, `supersedes.revision` does not point to the predecessor proposal — **CONFIRMED**, severity unchanged (warning)

**Ground 1: factually wrong?** Re-verified independently.

```
git cat-file -t a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6
```
→ `commit` (real object, matching the critic).

```
git show a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6:reference/proposal-tui-t-series.md
```
→ `fatal: path ... exists on disk, but not in a5fb875...` — the blob link
404s, confirmed independently.

```
git log --follow --diff-filter=A --format='%H %ad %s' -- reference/proposal-tui-t-series.md
```
→ single result, `a9a25fa68938323d9585edc687fbf0e965084c2e`, matching the
critic's identified introducing commit.

I additionally read `a9a25fa`'s own version of the file's frontmatter myself
(`git show a9a25fa:reference/proposal-tui-t-series.md | head -20`) rather
than trusting the critic's paraphrase — it is a genuine, differently-scoped
predecessor proposal ("Sergeant-rs T-Series: Work-Centered Terminal
Interface," `status: proposed`), and its own `audit_revision` pin is not
shown in the first 20 lines I read, but the critic's claim is specifically
that a5fb875 is *some* meaningful hash from the predecessor's own frontmatter
(its audit basis), not that it's fabricated — and a5fb875 being a real,
dated commit (`Merge pull request #43`, 2026-08-11) that is simply the wrong
field's value is consistent with what I see. This is a plausible,
well-evidenced mix-up (predecessor's audit-basis hash copied into the
revised proposal's supersedes-pointer field) rather than a wild claim.

**Ground 2 (cross-reference rescue attempt):** I grepped the entire document
for `a9a25fa`, `a5fb875`, and `predecessor` myself. The only two hits for
`a5fb875` are the frontmatter field and the header line — the same pair the
critic already names. No other section restates, corrects, or qualifies this
hash. This defense fails to rescue the citation, confirming the critic's
"does the section survive" reasoning (the prose argument survives; the
specific link does not).

**Ground 3: severity.** `warning` is correctly calibrated, not inflated and
not underweighted. Applying the "would a Work be blocked or produce a wrong
artifact" bar: unlike F1/F2, no downstream section's *behavior* depends on
this hash resolving — it is a dead link in a traceability pointer, not a
premise a T0–T4 phase or acceptance criterion is built on. I considered
pushing it down to `info` (on the theory that F1/F2 already establish the
document's citation hygiene is loose, making this a third instance of the
same class of miss) but reject that: F1/F2 both involve the *live PR under
concurrent review*, the single most consequential citation in the document
by the contract's own framing ("§12.4 treats PR #111's retained/reap
surfaces as conditional on that PR merging" is the contract's named starting
point). F3 is a citation to a different, closed, historical artifact with no
bearing on any decision gate. That's a real severity distinction, and
`warning` — one notch below F1/F2's `error`, one notch above cosmetic —
reflects it correctly.

**Verdict: CONFIRMED.** Severity unchanged (`warning`).

---

## Summary

All three findings in `critics/assumptions.md` survive adversarial
refutation, re-verified independently rather than trusted from the critic's
evidence — including one check (F1's Ground 3) where my first attempt,
grepping this session's local checked-out branch, found none of the
retained/reap or filesystem-reliability surfaces the critic says already
ship, and would have produced a false refutation had I not caught that the
local branch tip is 49 commits behind real `main` and re-run the check
against the fetched `github/main` remote — exactly the distinction the
critic's own method note warns about. F1's completeness sweep turned up a
small number of additional stale-conditional locations (frontmatter lines
10/41, the T2-07 register line) the critic's table doesn't separately
enumerate; these strengthen rather than weaken F1's "volume and spread"
characterization and don't change its verdict. No grounds were found —
factual, completeness, cross-reference, or severity — on which any of the
three findings should be knocked down, upgraded, or downgraded from what the
critic assigned: F1 `error`, F2 `error`, F3 `warning`.
