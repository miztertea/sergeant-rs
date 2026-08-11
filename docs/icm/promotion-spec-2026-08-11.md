# Curation spec: `reference-corpus/draft-workflows/` → `.sergeant/workflows/`

Governing documents: `docs/icm/convention.md` (the four-layer model, the
draft/admitted publication boundary — §2), `docs/icm/record-shapes.md` (the
`index.md` front-matter shape). Source of record for content: the 34
N1-adjudicated packages under `reference-corpus/draft-workflows/`
(`docs/gauntlet/contracts/N1.md`). Exemplar of the target shape:
`.sergeant/workflows/repo-to-icm/`, the one package currently admitted.

Provenance note on this spec's own charter: `docs/gauntlet/notes/
v2-measurement-and-migration-plan.md` does not carry the promotion addendum
section on this branch — it is Run A/B/Cerberus-handoff content only, dated
before this step. This spec instead follows the charter as given directly:
*curate `reference-corpus/draft-workflows/` — the 34 adjudicated packages,
source of record — into a runnable library; structure-validate each; run
each through the engine on the fake backend as its acceptance gate.*

This document was written after promoting and engine-testing three packages
end to end — `research` (1 stage), `worker-mission` (4 stages),
`sergeant-setup` (8 stages), chosen as small/medium/large by stage count —
against `/home/miztertea/sergeant-runb/target/debug/sgt`. §3 below is that
procedure, transcribed from what actually ran, not a plan.

---

## 1. The structural delta

Every one of the 34 draft packages already carries the full ICM shape as
authored: `index.md`, `workflow.toml`, a Layer-1 `CONTEXT.md`, a
`provenance.md`, and per stage a Layer-2 `CONTEXT.md` with a resolving
`## Inputs` table, plus a Layer-4 `output/README.md` naming an artifact and
a `promote`/`evidence` disposition — verified by direct scan of all 34 (no
stage anywhere in the corpus is missing an Inputs table or a Disposition
line). `_config/` (Layer 3) appears in 2 of 34 (`direct-implementation`,
`sergeant-setup`); no draft carries `references/` or `scripts/`. This is
**not** a from-scratch authoring job — it is packaging. Per package, exactly
these changes:

1. **Move, not copy.** `git mv reference-corpus/draft-workflows/<name>/
   .sergeant/workflows/<name>/` — convention §2 rule 2 forbids the package
   existing in both trees at once.
2. **`index.md` front matter:** `status: draft` → `status: published`
   (convention §2 rule 3 — status must agree with location).
3. **`index.md` body text:** the boilerplate sentence "Draft workflow
   candidate (N1 reference corpus, not admitted procedure — see
   `docs/icm/convention.md` §2). Use when: …" is rewritten to admitted
   framing — model `repo-to-icm/index.md`'s own body (states what the
   workflow does, cites its milestone/candidate id, points at
   `CONTEXT.md`/`workflow.toml`, and — since `provenance.md` is leaving the
   tree, see item 5 — points at this spec plus the archived provenance copy
   instead of a same-directory `provenance.md`).
4. **`workflow.toml` header comment** (the `#`-prefixed prose above
   `[workflow]`, never the table itself): rewritten from "Draft workflow —
   N1 reference corpus … Not admitted … Promotion … is a distinct,
   human-reviewed act" to an admitted-style header citing the milestone,
   the N1 candidate id (`provenance.md`'s own "candidate **W##**" line),
   and where the citation trail now lives — model `repo-to-icm/
   workflow.toml`'s header.
5. **`provenance.md` leaves the promoted tree.** Convention §2 rule 2
   permits "dropping or archiving … as the review record dictates";
   `repo-to-icm` (the only precedent) has none in its own tree. This spec's
   rule: archive verbatim (unedited — see §2) to
   `docs/gauntlet/promoted-provenance/<name>.md` in the same commit as the
   `git mv`, so the citation trail survives the move as a readable file,
   not only as a pre-move git blob a reviewer has to know to dig for.
6. **Root catalog.** `.sergeant/index.md` gains one row per promoted
   workflow (name / `published` / pointer to `workflows/<name>/index.md`)
   — convention §1 rule 1: a published workflow absent from the catalog is
   itself a violation, independent of anything else being correct.
7. **`_config/`, `references/`, `scripts/` carry across unchanged** where
   present, and are **not** speculatively added where a package has none —
   convention §1 says both are OPTIONAL per workflow/stage; a package with
   nothing stable to reference correctly omits them.
8. **Everything inside a stage `CONTEXT.md` or `output/README.md` carries
   across byte-for-byte** — Inputs tables and dispositions are already
   correct as authored (see the finalize caveat below for the one
   documented, not-required exception).

**One honest, corpus-wide observation, not a mandated edit.**
`repo-to-icm`'s own closing stage (`90-reconcile`) names a finalize step
and ships `scripts/finalize.py`, per convention §1a's D9 working rule
("a workflow that declares any output ends with a deterministic finalize
step"). Checking all 34 drafts' *true* closing stage (from `workflow.toml`'s
own stage order, not directory-listing order) against that rule: only 3
(`drain-fleet`, `respond-to-worker`, `to-spec`) actually name a finalize
step at their real last stage; `dispatch`'s one "finalize" mention sits at
`80-monitor`, not its true last stage `90-reconcile-fleet`. The other 30
declare `promote`/`evidence` outputs with no finalize step at all. D9 lives
under convention §1a's **"Open questions (recorded, not resolved)"**
heading, not its numbered Rules — so this is not, on the convention's own
text, a promotion blocker. But curation must not silently launder it either:
where a curator promotes a package with declared outputs and no finalize
step, the archived provenance note (item 5) should say so in one line —
disposition is applied by human review at merge time for that package, not
mechanically. This gap is what most of §5's classification turns on.

---

## 2. What is FORBIDDEN to change

Curation is packaging, not re-authoring. The adjudicated behavioral content
below is off-limits — editing any of it is not a curation act, it is
re-litigating N1's adjudication outside the process that owns it:

- **Stage boundaries.** `workflow.toml`'s `stages` array (identity, order,
  count) and the corresponding directory names/ordinals. Every split/merge
  decision is recorded in that package's own `provenance.md` under an
  "Adjudication A#" heading (e.g. `research`'s A4 folding
  `10-write-findings` into `00-investigate`) — curation does not re-run
  that judgment, it packages its outcome.
- **Behavior-unit citations.** Every `BU-P#-###` id, its quoted statement,
  and its source path/line in a stage's "Behavior contract" section — the
  evidentiary chain back to `reference/sergeant-upstream`. Not edited,
  paraphrased, or re-cited to a different line range.
- **Ladder-rung / "Judgment required" classification.** Whether a stage is
  an actor stage (§6.4) or a folded helper invocation (§6.5) is an
  adjudicated call. Curation does not reclassify a stage, invent a
  "Judgment required" section, or remove one.
- **`## Delegation` target names.** The literal workflow name cited (e.g.
  "grilling", "drain-fleet") is not changed to a different or renamed
  target. If a name collision or rename is ever unavoidable, every package
  that delegates to the renamed target must be updated in the same commit
  — an exceptional, logged act, not routine packaging.
- **Output dispositions** (`promote` / `evidence`) — an adjudicated policy
  call about what merges vs. what is Work-branch-only evidence. Not
  flipped to tidy up a package's merge footprint.
- **Engine-gap notes** (G1, G3, G5, G6, G7 …) and their survives/rejected
  verdicts (`reference-corpus/synthesis.md` §5,
  `reference-corpus/engine-pressure.md`) — historical record of what was
  proposed and ruled on. Curation reads these to understand a package; it
  does not edit, remove, or "resolve" them.
- **Stage prose** — "Purpose", "What must become true here", "Behavior
  contract", "Helper invocation(s)", "Additional note" sections: the
  adjudicated procedure itself.
- **`workflow.toml`'s `[workflow]` table body** (name, stages). `version`
  is bumped only by a real future content change to the workflow, never by
  the promotion act itself — promotion is not a content change.

What curation **may** touch, stated explicitly so the forbidden list above
is not read as blocking the whole file: the `workflow.toml` header
*comment* (never the table), `index.md`'s `status` field and its
introductory sentence, the root catalog listing, `provenance.md`'s
*location* (moved/archived, never its content), and — only where §1's
finalize gap applies — one added documentation line at the closing stage.

---

## 3. Engine-acceptance procedure (verbatim)

Run once per package, in a package-private scratch subject repo and a
package-private data dir, never reused across packages and never run
against `/home/miztertea/sergeant-rs`, `sergeant-harvest`, or `sergeant-runb`
themselves.

```sh
B=/home/miztertea/sergeant-runb/target/debug/sgt
SCRATCH=<this session's scratchpad>/promotion
NAME=<package name>              # == the promoted directory's own basename

# 1. Fresh scratch subject repo
rm -rf "$SCRATCH/subject-$NAME" "$SCRATCH/dd-$NAME"
mkdir -p "$SCRATCH/subject-$NAME"
cd "$SCRATCH/subject-$NAME"
git init -q -b main .
git config user.email test@example.invalid
git config user.name  "promotion test"

# 2. Install the PROMOTED package (post-§1 edits — provenance.md already
#    archived out, status already published) under .sergeant/workflows/
mkdir -p .sergeant/workflows
cp -r <path to the promoted package tree> .sergeant/workflows/$NAME
git add -A && git commit -qm "subject repo: promoted $NAME"

# 3. SGT_FAKE_SCRIPT MUST be unset — this is what makes every stage
#    complete instead of pausing (FakeBackend::from_env -> FakeBackend::new,
#    src/backend/fake.rs, when the env var is absent).
unset SGT_FAKE_SCRIPT

# 4. Submit with a plausible intent naming/matching the workflow's own
#    trigger, explicit --workflow so there is no ambiguity with the
#    embedded default ("software-change"), explicit --backend fake.
"$B" --data-dir "$SCRATCH/dd-$NAME" --json run \
  "<a one-sentence intent matching this package's own trigger text>" \
  --workflow "$NAME" --backend fake
```

Assert, from the JSON reply and the journal at
`$SCRATCH/dd-$NAME/journal/*.ndjson` filtered to the returned `work.id`:

1. `work.state == "completed"`. Under the unscripted fake backend none of
   the 34 packages are entitled to pause (only `SGT_FAKE_SCRIPT` or a real
   backend's `needs_input` signal produces a pause) — any other terminal
   state is a gate FAILURE.
2. Exactly one `workflow.bound` event whose `stage_bindings` list, in
   order, equals `workflow.toml`'s `stages` array exactly — same ids, same
   order, same count. This is "workflow.bound pinned the full stage list."
3. For every stage in that order, one `stage.entered` followed (allowing
   intervening `execution.*` events) by one `stage.completed` for the same
   `stage_id`; the sequence of entered/completed pairs across the whole
   journal matches `stage_bindings`' order exactly.
4. Exactly one terminal `work.completed` whose `stages` count equals
   `len(workflow.toml.stages)`.
5. (Reused from `scripts/demo.sh`'s own check, worth applying to every
   multi-stage package) each stage's `execution.started` carries a
   distinct `execution_id` — proves each stage is a fresh execution, not a
   reused turn.

Then, unconditionally, before touching the next package:

```sh
PID=$(python3 -c "import json;print(json.load(open('$SCRATCH/dd-$NAME/runtime.json'))['pid'])")
kill -TERM "$PID"
# poll kill -0 "$PID" for up to ~10s; SIGKILL if it survives
pgrep -af "debug/sgt \[-\]-data-dir $SCRATCH/dd-$NAME" || echo "confirmed gone"
```

The bracketed `[-]-data-dir` in the check is deliberate (CLAUDE.md) — it
makes the pgrep pattern non-self-matching. Never delete `$SCRATCH/dd-$NAME`
or start the next package's daemon until that line prints "confirmed gone."

**Caveat, recorded rather than omitted:** this gate is mechanical only. It
proves the four journal properties above and nothing else. It does **not**
exercise a package's `needs_input`/`respond` path — no unscripted fake run
ever pauses — so it cannot validate the two packages whose adjudicated
content specifically depends on that path (`grilling`,
`sergeant-setup`'s `30-project-interview`; both engine-gap **G5**, the
re-enterable-interview case). Those need a second, scripted
(`SGT_FAKE_SCRIPT="needs_input:…;complete:…"`) or real-backend acceptance
pass before they should be trusted beyond "the engine can enter and leave
every stage in order" — which is exactly why they classify NEEDS-JUDGMENT
in §5 rather than being cleared by this gate alone. Nor does this gate
check that a `## Delegation` target actually exists in the library — the
engine has no concept of cross-workflow references, so a dangling
delegation target passes this gate silently; that check belongs to §2/§4's
naming discipline, enforced by review, not by `sgt run`.

**What was actually run.** `research` (1 stage), `worker-mission`
(4 stages), and `sergeant-setup` (8 stages) each went through the exact
recipe above, on 2026-08-11, against
`/home/miztertea/sergeant-runb/target/debug/sgt`: all three reached
`work.state == "completed"`, all three journals showed `workflow.bound`
pinning the full stage list, every stage's `stage.entered`/`stage.completed`
pair appeared in declared order with a distinct `execution_id`, and all
three daemons were confirmed gone by the bracketed `pgrep` before the next
package started. `sergeant-setup`'s clean pass is itself the caveat's
illustration: its `30-project-interview` stage's real, adjudicated content
depends on a multi-round `needs_input` loop this unscripted run never
touched, and the run reads as unremarkably successful regardless.

---

## 4. Naming rule

`.sergeant/workflows/<name>` where `<name>` is **exactly**
`reference-corpus/draft-workflows/<name>`'s own directory basename — no
case change, no re-hyphenation, no synonym.

Two independent reasons this must be exact rather than "close enough":

- `sgt run --workflow <name>` and the workspace-default resolution path
  match the directory basename literally.
- 10 of the 34 packages already name another corpus member by this exact
  bare string in a `## Delegation` section (`to-tickets`, `worker-mission`,
  `implement`, `triage`, `direct-implementation`, `wayfinder`,
  `cross-repo-work`, `task-intake-and-route`, `grill-with-docs`,
  `dispatch` — see §5). Renaming a promoted package breaks every package
  that already delegates to it, invisibly to both the structural shape
  (Delegation prose is not machine-parsed) and to §3's engine-acceptance
  gate (delegation is actor judgment at runtime, not engine-wired) — only a
  literal name-for-name check across the promoted library catches it.

Practical consequence: promote the 34 as one name-preserving batch, or, if
promoting incrementally, promote a delegation *target* no later than the
package that names it, so no window has a dangling reference.

**Ordering-window record (Cerberus promotion-fixer pass, 2026-08-11, F5).**
This rule was broken once during incremental promotion: `cross-repo-work`
landed (58e368f) two commits before its `## Delegation` target `dispatch`
(24cbccf), leaving a transient window with a dangling reference. Mitigated
at the time it happened — `cross-repo-work`'s own `index.md` at 58e368f
makes no "published in this library" claim about `dispatch` (unlike
`dispatch`'s and `worker-mission`'s own `index.md`, which do assert target
presence and are correct) — and closed as of `dispatch`'s own promotion
two commits later. All 10 delegating packages' `## Delegation` targets were
re-checked against the final 35-directory listing at HEAD (2026-08-11) and
every bold-named target resolves; the only unmatched tokens in any
`## Delegation` section are the English words "whichever"/"chosen"/
"selected", not package names. No further action: the defect was a
transient window in committed history, already self-corrected by the time
promotion completed, and does not exist at HEAD. Recorded rather than left
unremarked, since the class of mistake — batch-promoting with delegation
fan-out — recurs for any future incremental promotion pass.

---

## 5. Classification of all 34 packages

**Method.** Two objective, machine-checkable signals, both gathered by
direct scan of every package in the corpus (not sampled): (a) the
package's own `CONTEXT.md` names an engine-gap (G1/G3/G5/G6/G7) it is
deliberately routing around or citing as live pressure — meaning a curator
must understand the workaround, not just move the directory; (b) the
package's own `CONTEXT.md` contains a `## Delegation` section naming
another workflow by bare name — meaning promotion correctness depends on a
second package's presence and name, which neither the structural shape nor
§3's engine gate can verify. A package with either signal is
NEEDS-JUDGMENT; a package with neither — the majority, 20 of 34 — is a
clean directory-move-plus-status-flip with nothing further for a curator to
adjudicate.

**STRAIGHTFORWARD (20):** `code-review`, `deepen-module`, `diagnose-bug`,
`drain-fleet`, `load-project`, `monitor-fleet`, `project-graph`,
`prototype`, `reconcile-and-cleanup-fleet`, `recover-stalled-worker`,
`research`, `resolving-merge-conflicts`, `respond-to-worker`,
`route-review-findings`, `sergeant-help`, `tdd`, `to-spec`,
`validate-and-ship`, `vet-external-skill`, `wiki-digest`. (Several of these
— `drain-fleet`, `respond-to-worker`, `load-project`, `code-review`,
`tdd`, `validate-and-ship` — are delegation *targets* named by other
packages; being targeted does not itself impose curation judgment on the
target's own packaging, only on the package that names it.)

**NEEDS-JUDGMENT (14):**

- `cross-repo-work` — delegates to `dispatch` at `50-handoff-or-stop`;
  `60-reconcile` separately cites engine-gap G6 as live pressure (it
  explicitly does *not* invoke `dispatch` as a real child-workflow, only
  names it for the reader) — curator must confirm `dispatch` is promoted
  and not mistake the citation for an actual delegation.
- `deliver-external-callback` — its sole surviving stage (`00-seal`) is the
  corpus's own lowest-confidence stage-vs-helper placement, kept only
  because a materializable workflow needs at least one actor-stage
  directory; its own text calls this "low confidence" and ties it to
  engine-gap G3 ("this may not be a workflow with actor judgment at all").
- `dispatch` — engine-gap-flagged at `80-monitor`, and delegates to both
  `drain-fleet` (`15-check-admission`) and `respond-to-worker`
  (`80-monitor`) — a three-package dependency to verify.
- `direct-implementation` — delegates its entire shipping-gate stage to
  `validate-and-ship`; also one of only two packages with a `_config/`, so
  curation must confirm (as this spec did) its standing-constraints don't
  overlap `sergeant-setup`'s before leaving both un-consolidated.
- `grilling` — the corpus's canonical engine-gap **G5** case
  (re-enterable `needs_input` for a multi-round interview); §3's
  unscripted acceptance gate never exercises this path, so a curator must
  read and understand the re-entry design by hand.
- `grill-with-docs` — delegates to `grilling` twice (`00-interview-loop`,
  `10-confirm-understanding`), inheriting G5's re-enterable-interview shape
  by composition, plus the ordinary delegation-target check.
- `implement` — delegates to `tdd` (`10-implement-with-tdd`) and
  `code-review` (`30-review`) — two-package dependency check.
- `sergeant-setup` — `30-project-interview` is the corpus's other G5 case
  (multi-round interview reading back accumulated Work-response history);
  same "the mechanical gate doesn't exercise this" caveat as `grilling` —
  confirmed directly: its §3 run completed cleanly without ever touching
  this stage's real behavior.
- `task-intake-and-route` — the corpus's router: delegates to
  `load-project` and to whichever of `direct-implementation`/`dispatch`
  was chosen at its own `03-choose-mode` — the widest *named-alternative*
  fan after `worker-mission`, and all named targets must exist for the
  routing to mean anything.
- `to-tickets` — delegates to `load-project`; the smallest instance of the
  pattern, still requires the target present.
- `triage` — delegates to `grilling` (so inherits the G5 caveat by
  composition); its own text separately notes a *rejected* engine-gap
  claim for its transition graph (upheld, not open) — worth distinguishing
  from `grilling`'s own live G5 when a curator reads it.
- `wake-and-resume` — its sole surviving stage is the direct source of
  engine-gap **G1** (periodic re-evaluation scheduling with no live billed
  process) — a curator must understand that this stage's durable outcome
  depends on an external re-trigger the engine does not yet own natively,
  not something the stage completes unaided.
- `wayfinder` — delegates to `grilling` (`00-name-destination`);
  `40-regraduate-fog`'s fog-loop is represented as *resubmitting a fresh
  Work* rather than an engine-level loop (engine-gap G7 was considered and
  **rejected**, not accommodated) — non-obvious behavior a curator should
  understand before publishing it as ordinary procedure.
- `worker-mission` — delegates to **five** possible targets chosen
  dynamically at `20-implement` (`diagnose-bug`, `prototype`, `tdd`,
  `implement`, `deepen-module`, whichever `10-triage-and-route` selects) —
  the widest fan-out in the corpus — plus its own engine-gap **G6** note
  (child-procedure invocation) at `10-triage-and-route`.

---

## Summary for curators

Per package: `git mv` into `.sergeant/workflows/`, flip `index.md` status
and its admission sentence, rewrite the `workflow.toml` header *comment*
only, archive `provenance.md` verbatim to
`docs/gauntlet/promoted-provenance/<name>.md`, add the package to
`.sergeant/index.md`'s catalog table, run §3's engine-acceptance recipe
with a private data dir, stop and pgrep-verify the daemon. Touch nothing
under §2's forbidden list. For the 14 NEEDS-JUDGMENT packages, read the
one-line reason above before promoting — most resolve to "confirm the
named delegation target(s) are also in the library" and two
(`grilling`, `sergeant-setup`) additionally need a scripted or real-backend
acceptance pass before their G5 stage is trusted.
