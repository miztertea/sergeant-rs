# Package adjudication: task-intake-and-route

ICM-R2 pilot package (`docs/adr/0013-icm-r0-owner-rulings.md` decisions
8–9). Producer pass, per `reference/proposal-icm-r-procedure-authority.md`
§8 (Contract, Inventory, Harvest, Normalize, Placement classification,
Authority classification, Synthesis). Behavior units classified first;
package verdict synthesized from the units afterward
(`docs/icm/record-shapes.md` §6 rule 1).

## Original intention

Per the package's own `CONTEXT.md`/`index.md`: "The standing entry
procedure every task passes through before any implementation workflow
starts: it turns a user request into a chosen, scoped execution mode."
Extracted (N1, candidate W5) from `reference/sergeant-upstream/AGENTS.md`'s
nine numbered standing-entry steps, decomposed into six admitted actor
stages (three of the original nine folded as helpers at N1 adjudication
A4). Historical provenance: `docs/gauntlet/promoted-provenance/
task-intake-and-route.md`.

## Current trigger and outcome

Trigger (as authored): "Any task the user brings." Bounded outcome (as
authored): repositories/instructions/dependencies known, mode chosen,
unresolved risk decisions confirmed, control handed to
`direct-implementation` or `dispatch`, decision gates handled, and
PRs/merge order/deployments/cleanup settled — six actor stages,
`workflow.toml` v2, `status: published`.

## Driver and admission boundary

As authored, this package is `PL-4` (a Sergeant workflow: admitted,
sequential, fresh execution per stage). Applying the placement ladder's
own discriminator (proposal §3.3, §5.4) to what the six stages actually
do: every stage's job is to decide **whether a Work should exist and in
what form**, or to conduct the pre/around-Work conversation for a Work
that may not exist yet — before Work admission (stages 01/03/05), at the
moment of admission (06), or narrating/interpreting an already-admitted
Work's progress back to the user (08/09). None of the six stages executes
a bounded procedure *given* an already-admitted intent the way PL-4
requires (§5.6: "given an already-defined intent... can Sergeant execute
this procedure durably from admission to a terminal result"). The actual
driver throughout is the interactive harness (Captain) shaping and
steering, not Sergeant executing an admitted Work — this is exactly the
proposal's own §1 finding about this package (lines 63–69) and the
`AGENTS.md`/North-Star split it cites (R-NS-6: "execution ≠ dialogue";
"the harness ... owns that routing judgment," `AGENTS.md` "When NOT to use
`sgt`"). Admission boundary: pre-work throughout stages 01/03/05/06;
during-Work narration in 08/09, still Captain-driven per R-NS-6 (sgt owns
message mechanics, the harness owns the conversation).

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| BU-P1-026 (01-load-context: identify owning repo(s), inherited instructions, cross-repo deps before mode selection) | `AGENTS.md` L136, step 1 | PL-0 | N/A — invariant, no delegated actor judgment | ABSORBED | `AGENTS.md` "Standard workflow loop" step 1 (`sgt doctor`/`sgt repo list`, "never infer which repo or estate you're in from the current directory") plus `skills/estate-navigation/SKILL.md`, which explicitly names step 1 as the policy it specializes |
| BU-P1-027 (folded helper, ex-`02-check-queue`: run sgt-td-list, reuse or create a task) | `AGENTS.md` L137, step 2 | PL-0 | N/A — invariant | ABSORBED | `AGENTS.md` step 2 ("Check running work," `sgt status`/`sgt work list`, "reuse or resume a matching Work item instead of creating a duplicate") |
| BU-P1-028, BU-P1-003, BU-P1-108, BU-P8-053, BU-P8-054 (03-choose-mode: direct-vs-dispatch criteria) | `AGENTS.md` L138 step 3; `docs/what-is-sergeant.md` L68-72; `docs/using-sergeant.md` L18-19, L30-33 | PL-0 | N/A — invariant; the *act* of applying it is Captain judgment already assigned by `AGENTS.md` itself ("the harness ... owns that routing judgment; sergeant's core makes no claim about it") | ABSORBED | `AGENTS.md` "When NOT to use `sgt`" section — the same four dispatch-mode criteria and the same direct-mode criteria are already stated there near-verbatim as a stable invariant, with routing judgment already assigned to the harness |
| BU-P1-030 (05-confirm-decisions: ask only genuinely unresolved scope/risk decisions; never reconfirm a settled mode/plan/tradeoff) | `AGENTS.md` L140, step 5 | PL-0 | J0 already-canonical (`.sergeant/common/contexts/bounded-judgment.md` §J0: scope/policy/security/privacy/destructive/irreversible/public-behavior/promotion triggers `needs_input`/a live question; every other decision is settled) | ABSORBED | `bounded-judgment.md`, ratified as the canonical ladder every Captain skill and stage must already apply (ADR 0013 decision 1, `docs/icm/convention.md` §6.1) — this unit restates J0's own test in workflow-specific prose, it does not add a new rule |
| BU-P1-029 (folded helper, ex-`04-reconcile-state`: `sgt-watch --sync-all`, inspect active workers/branches/worktrees/gates before starting) | `AGENTS.md` L139, step 4 | PL-0 | N/A — invariant | ABSORBED | `AGENTS.md` step 2 (reuse/resume before creating a duplicate) and step 6 ("Monitor," `sgt --json watch`, journal-backed reconciliation) together already state this policy |
| BU-P1-031 (06-execute: direct mode implements in-session; dispatch mode runs `sgt-dispatch`) | `AGENTS.md` L141-143, step 6 | PL-0 | N/A — invariant | ABSORBED | `AGENTS.md` steps 4–5 ("Choose a workflow," `sgt run "<intent>"` with envelope flags) already describe exactly this fork; the two destinations (`direct-implementation`, `dispatch`) are themselves already-admitted workflows in this pilot corpus that own their own launch preconditions |
| BU-P1-033 (08-handle-decisions: for needs_input/blocked/ask-user gates, obtain only genuinely missing decisions, record, continue without re-asking) | `AGENTS.md` L145, step 8 | PL-0 | N/A — invariant | ABSORBED | `AGENTS.md` step 7 ("Respond to `needs_input`," "reserved for genuine human-judgment gates ... not relayed for findings a workflow could apply itself") |
| BU-P1-038 (08: use `sgt-respond`/`sgt-wake`/recovery only after reconciling status, response generation, identity, and handoff evidence) + folded helper BU-P1-032 (ex-`07-monitor`: require recent meaningful events or an active child operation plus exact process identity, not parent-process liveness alone) | `AGENTS.md` L148 (resume preconditions); L144, step 7 | PL-0 | N/A — invariant | ABSORBED | `AGENTS.md` step 6 verbatim: "a Work item isn't progressing merely because a process for it exists; trust the journal-backed state these surfaces read, not liveness alone" — the same identity/liveness policy this package's own "Notes for reviewers" already says must be read as "the durable execution or session identity this project already journals," i.e. it is describing product that already exists and already states the rule |
| BU-P1-034 (09-reconcile-deliver: surface PRs and merge order, complete approved merges/deployments, run cleanup only after terminal state and preserved evidence verified) | `AGENTS.md` L146, step 9 | PL-0 | N/A — invariant | ABSORBED | Duplicated, more specifically, by the two workflows `06-execute` already delegates to: `direct-implementation/06-pr-and-merge` (`BU-P1-013`/`BU-P1-014` — PR/CI/review/merge-authorization gate, then record handoff/PR/merge/deployment/cleanup outcomes) and `dispatch/90-reconcile-fleet` (`BU-P5-070`/`BU-P5-071`/`BU-P1-006` — per-repo gate list including dependency merge order, "never reconciled merely because a PR exists"); `AGENTS.md` step 8 ("Collect") covers the cross-mode remainder (output pointer, spend). A stage sequenced *after* `06-execute` hands off to a workflow that already performs this same reconciliation internally is pure duplication, not new value |

No unit required a `HARVEST`/`SPLIT` destination distinct from what
already exists: every citation traces to policy `AGENTS.md`'s current
"Standard workflow loop" (or, for one unit, the canonical
`bounded-judgment.md`, or, for the last, the two downstream workflows this
package itself delegates to) already states, in most cases in closely
matching language. `alternatives_considered` for every row: `stage`
(rejected — no independent durable checkpoint distinguishes it from the
Captain conversation already required to reach it) and, for BU-P1-034
specifically, `shared-context`/`shared-helper` (rejected — the behavior is
not a reusable method invoked *by* other packages, it is a duplicate of
what two of those packages already do as their own terminal stage).

## Surviving package design

None. No behavior unit requires a new or retained surface — see
"Destination" column above. There is no `06-...`-shaped or other stage
list to propose; a rewritten package would either restate `AGENTS.md`'s
existing standard loop in workflow form (reintroducing the PL-4-vs-PL-2
conflation §5.4's discriminator exists to catch) or restate
`bounded-judgment.md`'s own J0 test in local prose (a second, drift-prone
copy of already-canonical text, contrary to §7.1's no-duplication rule).

## Inputs and outputs

Not applicable — no surviving package. The behaviors this package cited
continue to be read from their existing owning surfaces
(`AGENTS.md`, `bounded-judgment.md`, `direct-implementation`, `dispatch`,
`skills/estate-navigation/SKILL.md`); none of those files change as part
of this adjudication.

## Review and promotion policy

This record is a draft adjudication only (ADR 0013 decision 6: independent
review is required before any promotable effect). It does not itself
retire, edit, or unpublish `.sergeant/workflows/task-intake-and-route/` —
that is a Captain reconcile-and-publish action (proposal §8.11–8.13,
`docs/icm/convention.md` §6.2) gated on an independent reviewer confirming
this classification, not a self-promotion by this producer pass.

## Alternatives considered

- **STAND** — rejected. Every cited behavior unit duplicates content
  already live in `AGENTS.md`'s standard workflow loop, the canonical
  `bounded-judgment.md`, or the two downstream workflows this package
  delegates to; retaining a seventh, parallel restatement is exactly the
  drift §7.1 (no-duplication) and §5.4's PL-2/PL-4 discriminator exist to
  prevent.
- **SPLIT/HARVEST** (the proposal's own SS12.1 hypothesis: PL-2 Captain +
  PL-0 absorbed + a possible PL-3 reusable routing method) — tested unit by
  unit and not confirmed as authored. No unit survives as a genuinely new
  Captain-skill artifact or a genuinely new shared method: the mode-
  decision criteria (would-be PL-2 content) are already stated in
  `AGENTS.md` itself as an invariant with routing judgment already
  assigned to the harness, and the would-be PL-3 candidate
  (09-reconcile-deliver) turned out to be a verbatim duplicate of two
  other packages' own terminal stages rather than a method those packages
  invoke. The hypothesis's shape was directionally right (this package is
  not a workflow) but its predicted surviving destinations do not exist
  once checked against current package content — the current product has
  already absorbed everything this package describes.
- **FOLD** (fold surviving units into the two destination workflows'
  own stages) — rejected per-unit: nothing here is missing from
  `direct-implementation`/`dispatch`/`AGENTS.md` for there to fold in.
- **RETIRE** — considered as the literal outcome (no surviving package),
  but `ABSORBED` is the more precise modifier per §5.10: the behavior is
  not merely absent of value, it is actively owned today by named,
  existing product surfaces, which `RETIRE`'s definition ("no surviving
  behavior remains after normalization") does not capture as precisely as
  `ABSORBED`'s ("existing product surface already owns the behavior").

## Final disposition

ABSORBED

## Validation evidence

- Every file under `.sergeant/workflows/task-intake-and-route/` read in
  full (`CONTEXT.md`, `index.md`, `workflow.toml`, all six stages'
  `CONTEXT.md` and `output/README.md`).
- Both delegation targets read: `.sergeant/workflows/direct-implementation/`
  (`CONTEXT.md` and `06-pr-and-merge/CONTEXT.md`) and
  `.sergeant/workflows/dispatch/` (`CONTEXT.md` and
  `90-reconcile-fleet/CONTEXT.md`) — confirmed each already performs the
  behavior `09-reconcile-deliver` restates.
- `AGENTS.md` (current, top-level) read in full and cross-checked line by
  line against every stage's behavior contract; `skills/estate-navigation/
  SKILL.md` read in full for the `01-load-context` cross-check.
- `.sergeant/common/contexts/bounded-judgment.md` read in full; its J0
  section checked against `BU-P1-030`'s stated criteria.
- `NORTH-STAR.md` read for R-NS-6 (execution ≠ dialogue) and the
  Captain/OS ownership split cited in the driver classification above.
- `.sergeant/workflows/load-project/` spot-checked: `01-load-context`
  delegates to it, but `load-project`'s own `CONTEXT.md` records it as an
  unreconciled legacy artifact (still describing upstream's
  `~/.config/sergeant/<project>.yaml` registry, "no sergeant-rs analog
  yet") — corroborating evidence that this package's citations trail the
  current product rather than describing it, not itself part of this
  package's disposition (a separate package, out of this pilot's scope).
- No package-specific content was found that contradicts the proposal's
  §1 finding about this package or the SS12.1 hint; the hint's predicted
  *destinations* (a surviving Captain skill, a surviving shared method)
  were tested against actual current content and not confirmed — recorded
  above under "Alternatives considered."
