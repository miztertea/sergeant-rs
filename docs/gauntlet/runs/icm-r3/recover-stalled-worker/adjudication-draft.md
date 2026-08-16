# Package adjudication: recover-stalled-worker

ICM-R3 full-reconciliation pass, `reference/proposal-icm-r-procedure-
authority.md` §10.4; method per §8; record shape per `docs/icm/record-
shapes.md` §6. Producer pass only — independent review is a separate step
(§8.11 of the proposal; §6.2/6.3 of `docs/icm/convention.md`) and has not
run yet. This record is itself draft — it does not self-promote (ADR 0013
decision 6).

## Original intention

One bounded recovery attempt for a stalled worker: converge on a
replacement or escalate — never guess
(`.sergeant/workflows/recover-stalled-worker/CONTEXT.md` "Purpose";
`index.md` description). Promoted into the N1 reference corpus as
candidate **W11** (`docs/gauntlet/contracts/N1.md`,
`reference-corpus/synthesis.md` §1), with a full behavior-unit citation
trail archived at `docs/gauntlet/promoted-provenance/
recover-stalled-worker.md`. This ICM-R3 pass does not re-run that N1
extraction; it applies the Placement and Bounded-Judgment ladders on top
of the already-cited N1 content and checks the package's compliance with
ADR 0013's rulings, the same method already exercised at ICM-R2 on
`validate-and-ship` (`docs/gauntlet/runs/icm-r2/validate-and-ship/
adjudication-draft.md`, read as the worked example for this pass).

The upstream source is `reference/sergeant-upstream/bin/sgt-recover`, a
CLI tool invoked explicitly as `sgt-recover <task-id> <repo>` after an
operator has already diagnosed a stall via `sgt-watch` and the
troubleshooting doc's four-signal procedure
(`reference/sergeant-upstream/docs/troubleshooting.md` L52-68). `sgt-watch`
only classifies and records the stall diagnostic; it never invokes
`sgt-recover` itself (checked directly — `sgt-recover` does not appear in
`reference/sergeant-upstream/bin/sgt-watch`). The destructive action this
package performs (kill the stalled process, launch a replacement) is
therefore already gated, in its source mechanism, on an explicit,
separately-issued human action naming the exact task/repo — not an
automatic reaction to the watcher's own diagnostic.

## Current trigger and outcome

Three-stage linear workflow (`workflow.toml`: `00-collect-signals`,
`40-escalate-on-second-attempt`, `50-escalate-undocumented`).

Trigger (workflow-level, `CONTEXT.md`): "A worker is `in_progress` with a
stall classification recorded by the watcher." Read together with the
source mechanism above, this trigger names the *precondition* the
workflow requires, not an automatic dispatch rule — an operator (or
Captain, on the operator's behalf) still supplies the specific
task/repo when admitting the Work, exactly as the upstream CLI required
positional arguments naming which worker to recover. Nothing in this
package's own content currently states this explicitly (see BU-RSW-14
below).

Outcome: either exactly one bounded recovery attempt (preflight validated,
replacement launched and proven live, original retired only after) with a
durable stamp guaranteeing a second stall on the same worker escalates
instead of retrying, or an immediate escalation to `needs_input` when the
stall classification is undocumented/unrecognized.

## Driver and admission boundary

Driver: **stage actor**, all three stages (matches the package's own
stage table in `CONTEXT.md`, each row already labeled "actor-stage (§6.4,
judgment)"). Admission boundary: **in-Work** — the workflow receives an
already-admitted Work intent that names the specific stalled worker
(task-id/repo), mirroring the upstream CLI's own explicit two-argument
invocation. It passes the execution-surface test (`convention.md` §2a):
"would a human type `sgt run 'recover the stalled worker for <task> in
<repo>' --workflow recover-stalled-worker`?" — yes, after the human has
already seen the watcher's stall diagnostic. This is not pre-Work Captain
dialogue about what Work should exist; the decision "recover this
specific worker" is made before the Work is admitted, by the same
mechanism the source CLI already required (naming task-id and repo as
arguments).

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-RSW-01 | `CONTEXT.md` (Purpose) — one bounded recovery attempt: converge on a replacement or escalate, never guess | PL-4 | J5 (contract-level: exactly one bounded attempt, ever; never an open-ended retry loop, `BU-P7-095`) | STAND | `recover-stalled-worker` (workflow) |
| BU-P8-095 | `00-collect-signals/CONTEXT.md` — four signals (fleet status/log mtime, process identity + activity timestamp, progress timestamp/stall diagnostic, td handoff + branch/worktree state) collected together before any kill/relaunch decision | PL-5 | J5 (governing: no kill/relaunch decision on partial evidence) + J2 (delegated: reconciling a nonterminal stall diagnostic through the documented progress rules) | STAND | `00-collect-signals` |
| BU-P8-099 | `00-collect-signals/CONTEXT.md` — a repeated notification is compared on task, repo, state generation, message digest, and timestamp before acting | PL-5 | J2 (delegated: investigate the specific cause — stale fleet record, unconsumed response, or misreclassified expected-blocked worker) + J5 (governing: never produce a duplicate task or response) | STAND | `00-collect-signals` |
| BU-P6-071 | `40-escalate-on-second-attempt/CONTEXT.md` (own checkpoint) and its Helper §1 (folded `10-preflight`) — stall proof required (in_progress + watcher-written stall marker); every invocation stamped so a second attempt always escalates | PL-5 (checkpoint); PL-6 (the stamping mechanism itself) | J5 (governing: gates the attempt on concrete proof; guarantees exactly one bounded relaunch) | STAND | `40-escalate-on-second-attempt` |
| BU-P6-073 | Helper §1 (folded `10-preflight`) — refuse while an unfinished action-lease exists unless the owner is provably dead; anything else fails closed | PL-6 | J5 (governing: fail-closed default preserves exact-once delivery evidence; only provable death, never mere idleness, overrides it) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P6-075 | Helper §1 (folded `10-preflight`) — every pre-flight validation runs to completion before the attempt is stamped as made | PL-6 | J5 (governing: the one-shot budget is consumed only on actual commitment to relaunch, never by a failed pre-flight check) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P7-092 | Helper §1 (folded `10-preflight`) — recovery is refused while a drain is active | PL-6 | J5 (governing: respects the same fleet-wide admission-control boundary as ordinary dispatch/respond relaunches) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P7-093 | Helper §1 (folded `10-preflight`) — lease-owner liveness/staleness adjudicated (provably dead → proceed; live, reused identifier, or unprovable → fail closed); no raw shell error leaked to stderr | PL-6 | J5 (governing: fail-closed on unprovable ownership) + J1 (local: error-message text quality) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P6-072 | Helper §2 (folded `20-launch-replacement`) — replacement only launched, identity validated, before the original is ever terminated | PL-6 | J5 (governing: strict ordering — a failed relaunch sequence leaves the original process intact) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P7-094 | Helper §2 (folded `20-launch-replacement`) — validate liveness, published identity, and notification-target creation before killing the original; every abort path restores fleet state to the surviving original | PL-6 | J5 (governing: the destructive step is strictly ordered after the replacement is proven viable; no abort path may lose the recorded worker identity) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P7-095 | Helper §3 (folded `30-retire-original`) — exactly one bounded recovery attempt per invocation: terminate, relaunch, atomically update fleet metadata, deliver notification | PL-6 | J5 (governing: a single bounded operation, not an open-ended retry loop) | STAND | `40-escalate-on-second-attempt` (helper) |
| BU-P8-109 | `50-escalate-undocumented/CONTEXT.md` — search existing docs via sergeant-help first, then create a td task with exact reproduction, expected behavior, preserved state, and acceptance criteria | PL-5 | J5 (governing: an undocumented/unrecognized stall class is never guessed at — it always escalates) + J2 (delegated: how to conduct the search and compose the task's contents) | STAND | `50-escalate-undocumented` |
| BU-RSW-13 | All three stage `CONTEXT.md` files — uniform `## Judgment required` boilerplate paragraph; no stage names J2 delegations, J1 local choices, or J0 escalation triggers in the canonical shape | N/A (authoring-format compliance, not a placement question) | J5 (ADR 0013 decision 4 + `docs/icm/convention.md` §6.1: every actor stage's `CONTEXT.md` carries a `## Bounded judgment` section "always... omission is never ambiguous" — a governing requirement this package predates and does not yet satisfy; the ICM-R2 pilot packages, e.g. `validate-and-ship`, `code-review`, `research`, already carry the real section in place, confirming this is a rollout gap, not an open design question) | STAND (package identity correct; in-place content amendment required — see Surviving package design) | all three stage `CONTEXT.md` files |
| BU-RSW-14 | `CONTEXT.md` (L1) — no `## Authority envelope` section exists; nothing in the package states explicitly that a specific task/repo must already be named at admission (see Original intention / Driver and admission boundary above) | N/A | J5 (`convention.md` §6.1: every workflow Layer-1 `CONTEXT.md` carries an `## Authority envelope` section) | STAND, in-place amendment required | `CONTEXT.md` |
| BU-RSW-15 | `CONTEXT.md`, `index.md` — both state "See `provenance.md`" for the complete stage-to-behavior-unit mapping, but no `provenance.md` exists anywhere under this package's own directory (the actual file lives at `docs/gauntlet/promoted-provenance/recover-stalled-worker.md`) | N/A (dangling in-package reference, not a placement question) | J5 (`docs/icm/record-shapes.md` §1a rule 1: an unresolved reference is exactly what review must catch, same category as `validate-and-ship`'s dangling `route-review-findings` reference, BU-VAS-10) | **FOLD** (correct the reference in place; no placement change to this package) | `CONTEXT.md`, `index.md` |

## Relationships to other workflows (checked, not found)

The dispatch brief for this pass flagged `recover-stalled-worker` as
thematically adjacent to `dispatch` and `worker-mission` (both worker-
lifecycle packages) with no prior relationships-section delegation on
record, and asked whether the package's current isolation is correct.
Checked directly against the current content of all three packages:

- `dispatch/CONTEXT.md` and `worker-mission/CONTEXT.md` each carry a
  "Relationships to other workflows" section (the former delegates to
  `drain-fleet` and `respond-to-worker`; the latter delegates to
  `diagnose-bug`/`prototype`/`tdd`/`implement`/`deepen-module`). Neither
  names `recover-stalled-worker`, and `recover-stalled-worker/CONTEXT.md`
  names neither of them.
- `dispatch/80-monitor/CONTEXT.md` cites `BU-P5-089` — "Recovering a
  stuck, stale, or orphaned dispatched worker is done only through the
  response-delivery command or an equivalent explicit action" — but this
  is the *response-delivery* recovery concern (a worker that has not
  consumed a delivered response), sourced from
  `reference/sergeant-upstream/skills/dispatch/SKILL.md` lines 221-229,
  and already routed to `respond-to-worker` (per `dispatch/CONTEXT.md`'s
  own delegation). It is a different failure mode from the `in_progress`-
  but-stalled condition this package addresses (sourced from
  `bin/sgt-recover` and the troubleshooting doc's four-signal procedure),
  with no shared source citation between the two.
- The source mechanisms do not call each other: `sgt-recover` is invoked
  standalone by task-id/repo (checked above); `sgt-watch`, the diagnostic
  producer both packages could in principle consume, never invokes
  `sgt-recover` itself. Checked directly (`grep -rl sgt-recover
  reference/sergeant-upstream/bin reference/sergeant-upstream/skills`):
  six files mention the string, but every occurrence outside `bin/
  sgt-recover` itself is a comment or an error-message suggestion telling
  a human operator to run it manually (`bin/sgt-respond` L312's `_die`
  text; comments in `bin/_sgt-drain.sh`, `bin/sgt-interactive-worker`,
  `bin/sgt-watch`, `bin/_sgt-response-lock.sh`) — none is a call.

**Conclusion:** the package's current isolation from `dispatch` and
`worker-mission` is correct — the thematic adjacency (all three concern
workers) is not a shared behavioral contract, and adding a delegation
either direction would assert a dependency the source evidence does not
support. No relationships-section change is warranted.

## Surviving package design

No stage moves, merges, splits, or renames. The three-stage sequence,
its N1 adjudication A4 helper folds (`10-preflight`,
`20-launch-replacement`, `30-retire-original` into
`40-escalate-on-second-attempt`), and every already-cited N1 behavior
unit remain correctly placed at PL-4 (package) / PL-5 (each stage) / PL-6
(each folded helper). The package requires **in-place content
amendment**, not restructuring:

1. Add a `## Bounded judgment` section (per `convention.md` §7.3 /
   `bounded-judgment.md`) to each of the three stage `CONTEXT.md` files,
   replacing the current `## Judgment required` boilerplate with named J2
   delegations, J1 local choices, and J0 escalation triggers specific to
   that stage — this is a direct restatement of judgment content this
   package's Behavior contract sections already carry informally (see the
   J boundary column above, derived from that existing prose). For
   `40-escalate-on-second-attempt` this includes naming its Helper
   invocations' own governing (J5) constraints inline, not only in prose.
2. Add a `## Authority envelope` section to the workflow-level
   `CONTEXT.md` (per `convention.md` §7.2), stating explicitly that the
   workflow requires an already-named task/repo at admission (BU-RSW-14)
   — the concrete fact that keeps this package's destructive action from
   being an unauthorized-dispatch gap the way `validate-and-ship`'s
   push/pr/ci gap was (see Alternatives considered).
3. Correct the dangling `provenance.md` reference in `CONTEXT.md` and
   `index.md` (BU-RSW-15) to point at
   `docs/gauntlet/promoted-provenance/recover-stalled-worker.md`, the
   file that actually carries the stage-to-behavior-unit mapping.

None of these three amendments changes which package owns the behavior,
so none triggers the ADR's REHOME/SPLIT/HARVEST draft-and-rehome step
(`docs/adr/0013-icm-r0-owner-rulings.md` decision 6; task brief). They
are recorded here as the concrete remediation this adjudication found,
for the owner/reviewer to schedule — per this task's own instructions,
this producer pass does not apply them in place.

## Inputs and outputs

Inputs: as declared in each stage's own Inputs table — `00-collect-
signals` correctly declares only `../CONTEXT.md` (L1, first stage);
`40-escalate-on-second-attempt` correctly declares `00-collect-signals`'s
`output/README.md`; `50-escalate-undocumented` correctly declares
`40-escalate-on-second-attempt`'s `output/README.md`. All three comply
with `record-shapes.md` §1a (verified during Inventory). No contract-
bearing dependency was found undeclared.

Outputs: `output/README.md` in each stage declares its expected artifact
and disposition. `00-collect-signals` and `40-escalate-on-second-attempt`
are `evidence` (Work-branch record only); `50-escalate-undocumented`'s is
`promote` (workflow deliverable), correctly reflecting that it is the
terminal stage. `40-escalate-on-second-attempt`'s output README correctly
notes it preserves the three folded stages' own pre-A4 `evidence`
disposition. `50-escalate-undocumented`'s output README already records,
per the same D9 working-rule pattern noted at `validate-and-ship`, that
this workflow declares a `promote` output with no deterministic finalize
step named at its closing stage — not a promotion blocker, already
recorded in place rather than silently laundered.

## Review and promotion policy

This package's own content is already `status: published` under
`.sergeant/workflows/` (not a draft) — its structural and provenance
identity does not change. The three remediation items above are ordinary
content edits to an admitted workflow and should go through this
repository's normal review path for workflow content changes, not a new
draft-and-promote cycle, per `docs/icm/convention.md` §2 (the
draft/admitted split governs *new or substantially rewritten* content;
adding a required section to an already-admitted stage's `CONTEXT.md` is
neither). Per ADR 0013 decision 6, only the promotable form of this
change (once actually made) needs independent review before it lands —
this adjudication record itself, being ICM-R3 evidence, needs this
workstream's own reviewer step
(`reference/proposal-icm-r-procedure-authority.md` §8.11) before its
findings are treated as settled.

## Alternatives considered

- **REHOME or SPLIT this package into `dispatch` or `worker-mission`**,
  on the theory that worker-lifecycle behavior belongs in one place.
  Rejected: checked directly (see "Relationships to other workflows"
  above) — no shared source mechanism, no shared behavior-unit citation,
  and a different failure mode (in-progress-but-stalled vs. response-
  delivery non-consumption) from either sibling package. Merging would
  assert a dependency the evidence does not support and would be exactly
  the file-shape-mirroring failure §8.8 of the proposal warns against, in
  reverse (collapsing distinct behavior into one directory because the
  directories are thematically adjacent, rather than splitting one
  directory that mirrors a single source file).
- **Treat the admission-authority gap (BU-RSW-14) as a J0/needs-input
  finding analogous to `validate-and-ship`'s push/pr/ci gap (BU-VAS-15).**
  Rejected after checking the source mechanism: unlike `validate-and-
  ship`'s dispatched-Work path (which genuinely has no stage-level
  instruction gating push/PR/CI), this package's upstream CLI already
  required the destructive action's authorization as a structural
  precondition — `sgt-recover` cannot be invoked without already knowing
  which task/repo to recover, and `sgt-watch` never calls it
  automatically (checked directly, both cited above). The gap here is
  that this package's own content does not yet *state* that precondition
  explicitly, which is a documentation completeness finding (BU-RSW-14,
  disposition: in-place amendment) — not an unresolved authority question
  requiring `needs_input` from this producer.
- **Silently add the missing `## Bounded judgment` and `## Authority
  envelope` sections on this producer's own authority**, resolving BU-
  RSW-13/14 rather than just recording them. Rejected per this task's
  explicit instruction: revised content for a STAND disposition is not
  written to the live package by the producer; it is recorded here for
  independent review and a later reconcile-and-publish pass, matching
  exactly how `validate-and-ship`'s identical-shaped findings (BU-VAS-13/
  14) were handled at ICM-R2 and only landed in-place after that pilot's
  own review cycle (confirmed live: `.sergeant/workflows/validate-and-
  ship/CONTEXT.md` now carries the real `## Authority envelope` section).

## Final disposition
STAND

## Validation evidence

- Source-valid: every existing behavior-unit citation in this package's
  three stage `CONTEXT.md` files was read in full and traced to its
  already-archived N1 provenance
  (`docs/gauntlet/promoted-provenance/recover-stalled-worker.md`); no new
  citation was fabricated for this pass. The provenance file's own
  "Adjudication A4" and "Curation note" sections were also read in full.
- Placement-valid: every stage's already-recorded PL-5 rung
  ("actor-stage (§6.4, judgment)") and every folded helper's PL-6 rung
  were independently re-derived from the Placement Ladder in this pass
  and confirmed, not merely copied from the package's own table.
- Authority-valid: **not yet** — this is precisely what BU-RSW-13/14
  found missing. The package cannot be called authority-valid
  (`reference/proposal-icm-r-procedure-authority.md` §9.1 claim 3) until
  the three remediation items under "Surviving package design" land.
- Structurally valid: all three stage directories, their `output/
  README.md` declarations, and `workflow.toml`'s stage order agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly. One dangling
  reference found and dispositioned (BU-RSW-15, `provenance.md`).
- Execution-valid: **out of scope for this producer pass** — this
  adjudication is a content/citation review, not a re-run of the
  package; `reference/proposal-icm-r-procedure-authority.md` §9.3's
  execution-validation claims remain to be measured separately. The
  package's own engine-acceptance gate already ran once, at N1 promotion
  (`docs/gauntlet/promoted-provenance/recover-stalled-worker.md`
  "Curation note", 2026-08-11) — that evidence is prior-art, not
  re-verified here.
- This record itself is a draft producer output, not yet independently
  reviewed (`docs/adr/0013-icm-r0-owner-rulings.md` decisions 6-7); it
  does not self-promote.
