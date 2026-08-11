# AMBIGUOUS — NOT RESOLVED

What is ambiguous: the pinned revision of this run's subject cannot be
established, and the subject identification itself rests on inference
rather than an explicit statement in the Work's initiating task.

The Work's initiating task reads only: "Decompose this repository's
procedural knowledge into draft ICM workflows per
`.sergeant/workflows/repo-to-icm`." It names no repository path and no
revision. This worktree contains exactly two things below `.sergeant/`
(workflow machinery, not decomposition content) and a one-paragraph
`AGENTS.md` pointer: a single vendored subtree, `reference/sergeant-upstream`
(no `.git` of its own — confirmed absent), which holds all of the
worktree's substantive procedural content (skills/, docs/, scripts/,
tests/, its own AGENTS.md and README.md). `_config/run-discipline.md`'s own
worked example ("the N2 measurement run against `reference/sergeant-upstream`,
graded against `reference-corpus/`") and this stage's own `CONTEXT.md`
(which uses `reference/sergeant-upstream` as its paradigm vendored-subtree
example) both corroborate that `reference/sergeant-upstream` is the intended
subject — but neither is the Work's task itself naming it, so treat that
identification as a reasonable inference, not a settled fact this document
is entitled to assert as resolved.

Even granting that inference, the subject is a vendored subtree case
(`CONTEXT.md` §1, second bullet): its pinned revision is "not something to
(re)derive from `git rev-parse` inside it — there is no such object to
resolve against," and must instead come from "that subject's own provenance
document (a file like `UPSTREAM.md` alongside it, or named in the Work's
task)." No such document exists. There is no `UPSTREAM.md` anywhere in this
worktree, and the Work's task names none. One provenance-shaped file does
exist, `reference/sergeant-upstream/.agents/skills/PROVENANCE.md`, but it
records a *different, narrower* fact — the per-skill import of individual
folders under `.agents/skills/` from `github.com/mattpocock/skills`, each
with its own "locked folder hash" — not the pin of `reference/sergeant-upstream`
as a whole into this outer repository. Treating it as the answer would be
inventing a resolution method the vendored subject doesn't have, which
`CONTEXT.md` explicitly forbids: "If the Work's task names a subject with
no such record and no `.git`, that is the ambiguity ... do not invent a
resolution method it doesn't have."

What was checked:

- The Work's initiating task text, via
  `sgt --data-dir <run-data> --json work show 01KZNT2Y5BX7S26PJB3B1QVADW`
  (`work.intent` field) — no subject path, no revision named.
- Full worktree tree (`find . -maxdepth 3 -not -path '*/.git*'`): only
  `.sergeant/`, `AGENTS.md`, and `reference/sergeant-upstream/` exist below
  the worktree root.
- `git -C reference/sergeant-upstream rev-parse --is-inside-work-tree` —
  fails; `ls reference/sergeant-upstream/.git` — no such file or directory.
  Confirms the vendored-subtree case, not the live-checkout case.
- `find . -iname "UPSTREAM.md"` across the whole worktree — no results.
- `reference/sergeant-upstream/.gitignore`, `README.md`, and `AGENTS.md`
  (headers) — no vendoring/pin/revision statement for the subtree as a
  whole.
- `reference/sergeant-upstream/docs/` directory listing, plus a grep across
  `reference/sergeant-upstream/**/*.md` for "vendor", "provenance",
  "pinned", "revision", "commit", and bare 40-hex-character strings — the
  only provenance document found is the narrower
  `.agents/skills/PROVENANCE.md` described above; other hits were PR/commit
  references inside unrelated PRD/research prose, not a repo-level pin.
- `find . -maxdepth 2 -iname "*reference-corpus*"` — no `reference-corpus/`
  exists in this worktree, so per `_config/run-discipline.md`'s blindness
  rule there is nothing to be blind to and nothing to record as an
  explicit exclusion; none was named by the Work's task either, so none is
  recorded as excluded.

No subject, revision, or scope is recorded in place of the above — per
`_config/run-discipline.md` §2, this is the fail-closed marker itself, not
a partial contract for downstream stages to build on.

## Meta-level grammar-pressure moment

The engine gives no actor stage a way to pause its own turn and wait for a
human's disambiguating answer mid-run (`docs/gauntlet/notes/
n2-fake-backend-semantics.md`, as referenced by this stage's own
`CONTEXT.md`); a `needs_input`/`waiting` transition is runtime-driven from
outside the actor's turn, never actor-requested. The only fail-closed
action available to this turn was to write this marker and stop. This is
real signal about the current workflow grammar, not something to paper
over: `00-contract` is structurally unable to ask "which path under this
worktree is the subject, and where is its pin recorded?" even though that
is exactly the kind of one-line clarifying fact a human maintaining this
Work could supply in seconds. Recorded here per
`../_config/run-discipline.md` and `90-reconcile/references/
reconciliation-method.md` §3 for `90-reconcile` to pick up.
