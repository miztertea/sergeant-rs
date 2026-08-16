# T-SERIES-1 — fidelity refuter

Axis: fidelity. Assigned file:
`docs/gauntlet/runs/t-series-1/critics/fidelity.md`. Four findings (F1-F4),
all on the proposal itself (front matter/header, §12.3, §23.2, §5.5). Did
not read any other axis's critic or refuter output. Re-derived every
citation independently: re-read the full proposal at
`reference/proposal-tui-t-series.md` (2305 lines), the predecessor's actual
text at both commits the critic names, the relevant `src/cli.rs` doctor
module in full, ADR 0009 in full, and `gh pr view 111` directly rather than
trusting the critic's transcript of any of these.

## F1 — "Supersedes" citation points to a commit that never contained the predecessor

**Verdict: CONFIRMED, severity as-is (warning).**

Independently reran the critic's own checks: `git show
a5fb875e51f9fa9e2c34d508d5a3b1c6ee5aa8b6:reference/proposal-tui-t-series.md`
fails with "exists on disk, but not in ...", confirming the commit is real
(merge of PR #43, 2026-08-11 00:58) but does not contain the file. `git log
--all --oneline -- reference/proposal-tui-t-series.md` shows exactly two
commits for the file's whole history: `2000a22` (this proposal) and `a9a25fa`
("Execution-surface test (owner ruling)...", 2026-08-11 15:28, whose own
commit message confirms "T-series TUI proposal vendored"). `git merge-base
--is-ancestor a9a25fa... 242abe3...` returns true, confirming the critic's
claim that the predecessor is reachable from the proposal's own pinned audit
revision.

Nothing in this refutes cleanly. The citation is mechanically wrong exactly
as described: `supersedes.revision` and the header link both point to the
commit the predecessor's own `audit_revision` field recorded, not the commit
that actually contains the predecessor's text — a distinguishable, checkable
error, not a stylistic complaint. A reader clicking the header's GitHub link
gets a live 404, independently verified via the same reasoning (no GitHub API
call needed once `git show` fails locally — the object simply is not in that
tree).

Severity: warning is correct, not higher. The proposal's own §23 content —
what the finding's remediation matters for — was independently checked by
this refuter against the *correct* predecessor commit (`a9a25fa`) in F3
below, and every §23.1/most-of-§23.2 disposition traced cleanly. So the
broken citation does not appear to have caused a downstream fidelity failure
in the proposal's actual disposition claims; it is a dangling pointer, fixed
by repointing two fields to `a9a25fa`. Not info, because the axis's own
named check method (contract §1: "check each disposition against that
predecessor's actual text") is completely blocked for anyone who trusts the
citation as given rather than independently locating the file — that is a
real, if mechanically fixable, defect in the document's verifiability, which
is more than a wording nit.

## F2 — §12.3's Doctor check list names checks that do not exist and omits two that do

**Verdict: CONFIRMED, severity as-is (warning).**

Independently grepped `src/cli.rs`'s `mod doctor` for every `Check::ok`/
`Check::warn`/`Check::fail` name-string literal, and separately confirmed the
assembly order via `doctor::run`'s own body (`src/cli.rs:1751-1777`):
`git_check()`, `claude_check()`, `environment_check()`, `data_dir_check`,
`docker_check`, `journal_check`, `projection_check`, `daemon_check`,
`permission_mode_check`, `estate_check`, `disk_pressure_check` — eleven
checks, names `git`, `claude`, `environment`, `data_dir`, `docker`,
`journal`, `projection`, `daemon`, `permission_mode`, `estate`,
`disk_pressure`. No check is named `installation` or `profiles` anywhere in
the module. Went one step further than the critic and checked whether
`Report`'s own doc comments or `print()` output might group checks under an
"installation" category header that could make the proposal's word choice a
defensible paraphrase rather than an error — they do not: `Report::print()`
(`src/cli.rs:1721-1745`) emits one flat line per check by its literal `name`
field, no section headers, no synonyms. The word "installation" in the
source appears only in doc-comment prose describing what `healthy()` means
conceptually (`src/cli.rs:1684-1686`), never as a rendered check name. Also
confirmed the critic's `permission_mode` ↔ "profiles" mapping is not a
stretch: `permission_mode_check`'s own body (`src/cli.rs:1794-1814`) reports
declared profiles by name and is the only check that mentions "profiles" at
all.

Also checked whether this list is repeated or depended on anywhere else in
the proposal that would raise its severity — grepped the full document for
"installation" and "profiles" outside §12.3: both appear only in loose
prose ("Is the installation healthy?" in §2.1's rhetorical question list,
"installation checks" as a category label in §5.3) never as a second,
consistent binding claim, and §19.6 (Doctor parity)'s acceptance criterion
("no check disappears or changes status/remedy between surfaces") does not
hardcode any of the wrong names. So the defect is contained to one
prose sentence and does not propagate into a testable acceptance criterion.
Warning, not higher, is the correct severity — confirmed, not upgraded.

## F3 — §23.2 labels the predecessor's silence on repo/group as a "previous decision" being revised

**Verdict: CONFIRMED, severity as-is (warning), with one refinement.**

Read the predecessor's full text (`a9a25fa`, 1943 lines, `/tmp/predecessor.md`
in this session) independently. Grepped `\brepo\b`, `\bgroup\b`, `\bestate\b`
as whole words rather than substrings (the critic's own write-up mentions
"repo" and "group" without specifying word-boundary grep, so this refuter
re-ran it stricter to check for a false negative): `estate` — zero matches
anywhere in the predecessor. `group` — exactly two matches, both non-Estate
uses ("group, sort, filter, truncate... Fleet rows client-side" at line 439,
a UI-list verb; "Within each group, Work remains ordered..." at line 525,
referring to Attention-drawer buckets, not an Estate group). `repo` as a
whole word does not appear at all; every hit for the substring "repo" is
"repository" used generically (the codebase, or "repository-owned
procedure"), never a distinct estate/repo-management concept. The four nav
citations the critic gives (lines 78, 484, 496, 723) all independently
confirmed as exactly `Home    Fleet    Workflows`, no Estate tab. The
predecessor's explicit non-goals list (§5.2, Decision T-20, read in full)
enumerates journal search, new Work states, workflow authoring, file/diff
views, host metrics, OpenTelemetry, mouse, web redesign, graph canvas,
plugin framework, archival semantics, and issues #26/#45 — repo/group
management is absent from this list too, in either direction. The critic's
conclusion holds: 2026-08-11 did not rule on estate administration at all,
so "Revised" mischaracterizes new scope as a reversed prior decision.

Refinement on severity, considered and rejected as a downgrade: the
"Doctor CLI-only" row in the same table is a genuine, citable revision
(predecessor's Decision T-18, "Doctor remains a CLI-only diagnostic," read
verbatim at line 417) — so the table format itself is capable of accurate
rows, meaning this row's inaccuracy is not a systemic table-format
limitation but a specific false claim about what 2026-08-11 decided. That
argues for keeping warning rather than downgrading to info (unlike F4, which
echoes a pre-existing repository-wide imprecision, this row invents a
disposition — a subtly different failure). Considered arguing for an
*upgrade*: overstating new scope as "already revised from a prior decision"
could cause a reader to under-scrutinize genuinely novel territory (Estate
repo/group) as merely a revisited call. But the critic's own text confirms
the substantive Estate content is independently well-specified elsewhere
(§12, §16.2) and does not itself rest on the false "revision" framing for
justification — the mislabeling doesn't hide or license unreviewed content,
it just misdescribes provenance. Warning, unchanged, is correct.

## F4 — §5.5 credits ADR 0009 with moving two surfaces that were already in the no-spawn set before it

**Verdict: CONFIRMED, severity as-is (info).**

Read ADR 0009 in full independently. Its Decision section verbatim: "`status`,
`work show`/`list`/`transcript`, `analytics`, and the TUI join `sgt doctor`,
`sgt watch`, and `sgt daemon stop` in the no-spawn set" — word-for-word match
to the critic's quote. Its Context section states R-WATCH-3 already ruled
`watch`'s no-auto-spawn membership "owner-ruled 2026-08-13," one day before
ADR 0009's own "Accepted, 2026-08-14" status line. So Doctor and Watch are
pre-existing members being joined, not moved by this ADR — the critic's read
is correct. Independently found the same `src/cli.rs:1341` doc comment the
critic cites ("Used by `watch`, `status`, `work show`/`list`/`transcript`,
`analytics`, and `tui` — every verb ADR 0009 moved into the no-spawn set"),
confirming this exact imprecision predates the proposal in the repository's
own code.

Checked one thing the critic did not explicitly verify: whether the
imprecision changes what Decision T2-16 (the proposal's own decision that
depends on this sentence) actually commits to. T2-16 reads "`sgt tui`
continues to refuse without a running daemon..." — "continues to," present
tense, does not depend on which ADR is credited with which surface's
membership, only on the end state (all six surfaces refuse today), which is
independently true. So the attribution error is fully inert with respect to
the proposal's own binding decision, same conclusion the critic reached.
Info, not warning, is correct — this is strictly narrower in consequence
than F1-F3, all three of which affect either a citation's resolvability
(F1), a reader's mental model of literal rendered output (F2), or how much
scrutiny a section invites (F3); F4 affects none of these.

## Summary

| Finding | Verdict | Severity change |
|---|---|---|
| F1 | CONFIRMED | none (warning) |
| F2 | CONFIRMED | none (warning) |
| F3 | CONFIRMED | none (warning) |
| F4 | CONFIRMED | none (info) |

All four findings survive independent re-derivation from source, not the
critic's transcripts of it. None invalidate any section's premise — every
finding's own "does the section survive the correction" analysis was
independently re-checked and holds. No fabricated citations found in this
axis's report (contrast FOUNDATION-1's fidelity F2, where the refuter caught
an invented contract quote — no equivalent problem exists here; every quote
and line-number citation in this critic's report resolved to real text at
the cited location). No downgrade or upgrade of any severity is warranted;
the critic's own severity assignments already track the actual gradient of
consequence correctly (F1-F3 warning for reader-facing or verifiability
defects, F4 info for an inert attribution nit).

Also independently re-verified the critic's scope note: `gh pr view 111
--json state,mergedAt,mergeCommit` confirms `MERGED`, `mergedAt:
2026-08-15T15:28:02Z`, merge commit `3a46b87c...` — matching the critic's
citation exactly. Agree with the critic's own scoping: this is a currency/
staleness question the contract assigns to `assumptions`, not `fidelity`
("is every factual claim true *as of this session*" vs. fidelity's "does the
proposal say what the repository actually contains and decided" — PR #111's
technical content, which is fidelity's remit, was independently confirmed
accurate regardless of the branch's merge status). Not a fidelity finding
either the critic or this refuter should be raising.
