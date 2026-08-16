# Package adjudication: load-project

ICM-R3 full-reconciliation pass (`docs/adr/0013-icm-r0-owner-rulings.md`;
`reference/proposal-icm-r-procedure-authority.md` §8, §10.4; record shape
per `docs/icm/record-shapes.md` §6). Producer pass only — independent
review is a separate step (§8.11 of the proposal; §6.2/6.3 of
`docs/icm/convention.md`) and has not run yet. This record is itself draft
and does not self-promote (ADR 0013 decisions 6-7). No file under
`.sergeant/workflows/load-project/`, `.sergeant/index.md`,
`skills/estate-navigation/SKILL.md`, or `.sergeant/workflows/to-tickets/`
is edited by this pass.

## Original intention

"Establish, before any mutation, which repositories own the requested
outcome, where they are, what instructions govern them, and what state
they are in" (`.sergeant/workflows/load-project/CONTEXT.md` "Purpose"),
by wrapping upstream's `~/.config/sergeant/<project>.yaml` multi-project
registry: look up a project by exact registered name (`sgt-list`), resolve
its owning repositories/paths/clone-state/roles/groups/layered
instructions (`sgt-context`), and register or edit a project's definition
in that registry file, gated on schema-read, path-rule, and post-write
verification (`sgt-list`/`sgt-context` again), with restore-on-failure.
Promoted into the N1 reference corpus as candidate **W1**
(`docs/gauntlet/contracts/N1.md`), full behavior-unit citation trail at
`docs/gauntlet/promoted-provenance/load-project.md`. A prior pass (MVP-5
F2 execution-surface re-triage, 2026-08-12,
`docs/icm/retriage-2026-08-11.md` line 60, `docs/icm/re-homing-record-
2026-08-12.md` line 39) already split out four command-surface functions
(`list-projects`, `project-status`, `project-sync`, `project-task-list`)
to `sgt repo list`/`sgt doctor`/`sgt repo add`, leaving `00-resolve-
project-name`, `10-resolve-context`, and `20-register-or-edit`'s own
register/edit judgment as the surviving "workflow core" this pass
adjudicates. This ICM-R3 pass does not re-run N1 extraction; it applies
the Placement and Bounded-Judgment ladders to the already-cited content
and checks the package's compliance with ADR 0013's rulings, per this
Work's brief.

## Current trigger and outcome

Three ordinary actor stages (`workflow.toml`: `00-resolve-project-name`,
`10-resolve-context`, `20-register-or-edit`), `status: published`,
`version: 3`, listed in `.sergeant/index.md`. Trigger (per `CONTEXT.md`
and `index.md`): "A project is named, registered, edited, synced, or
listed; or repository ownership is not already established." Outcome: an
exact registered project name is bound (or the run stops to ask whether
to register), owning repos/paths/clone-state/roles/groups/instructions are
recorded as governing context, and a project definition is written to the
Sergeant-owned config path and validated (or restored on failure).

**Already-dead in practice, the same finding this package's own sibling
pilot made for `sergeant-setup` at ICM-R2.** `AGENTS.md`'s own trigger→
skill/workflow routing table (lines 42-51) has no row for `load-project`.
Its only two live triggers are already claimed: "Before acting in an
estate whose repos/groups/health aren't already confirmed this session"
routes to `estate-navigation` (`AGENTS.md` line 48), and registration
("register a new repo/group", "set up this estate") is explicitly listed
in `estate-navigation`'s own "When to use" section. `load-project`
survives only as a catalog row in `.sergeant/index.md` (`published`) that
nothing in current operating doctrine routes a harness to directly —
except one live delegation, `to-tickets/00-load-project-context`, which
still names it (see "Cross-package consequence" below).

## Driver and admission boundary

As authored: `stage-actor`, admission boundary `always` — three ordinary
admitted stages, fresh execution each, no live dialogue about what Work
should exist. Every stage's own table already labels itself "actor-stage
(§6.4, judgment)". None of the three stage `CONTEXT.md` files carries the
`## Bounded judgment` section `docs/icm/convention.md` §6.1 (ADR 0013
decision 4) now requires of every actor stage; all three instead carry the
older `## Judgment required` boilerplate paragraph predating that
requirement — the same authoring-format gap `validate-and-ship`'s ICM-R2
pass found (BU-VAS-13) and left as an in-place amendment. Recorded here as
an independent finding; moot for `load-project` given the disposition
below.

## The obsolete-mechanism finding (governs every unit below)

Before classifying any individual behavior unit, the PL-0 rung must be
checked against the *current product*, not against upstream
(`docs/icm/convention.md` §2a rule 4; proposal §5.2). Three independent,
already-landed sources converge on the same fact:

1. **`skills/estate-navigation/SKILL.md`'s own header** (live file, ICM-R2
   landed): "`sgt-context`... and `sgt-sync`... were each ruled **SKILL**
   by owner pre-ruling... Their upstream mechanism (a
   `~/.config/sergeant/*.yaml` project registry, `yq`-parsed, with
   `defaults → group → repo` instruction layering the harness composed
   itself) **does not exist in sergeant-rs and is not re-created here**."
   `sgt-context` and `sgt-sync` are exactly the two binaries
   `load-project`'s own `10-resolve-context` stage and its folded
   sync helper wrap (`BU-P5-093`, `BU-P5-095`).
2. **`docs/gauntlet/contracts/MVP-1.md`** (R-MVP1-3, R-MVP1-4, R-MVP1-5,
   lines 88-140): sergeant-rs's estate model is one `sergeant.toml` per
   estate, found by walking up from the working directory and bounded at
   `$HOME` — `[estate]`/`[[repo]]`/`[group.<name>]`, not a multi-project
   registry. `[[repo]] instructions = "local" | "suppress"` is declared in
   the manifest and **resolved and pinned by `sgt` itself at Work bind
   time**, not composed by a harness reading a layered YAML stack the way
   `load-project`'s `10-resolve-context` describes. There is no "project
   name" to register, list, or look up by exact match anywhere in the
   current schema.
3. **The sergeant-setup ICM-R2 pilot's own finding**
   (`docs/gauntlet/runs/icm-r2/sergeant-setup/adjudication-draft.md`,
   "Validation evidence" / B7 section): "`load-project`'s three stages
   never conduct a field-by-field interactive interview, never preview a
   full file, and never back up before overwrite... What is genuinely
   duplicated without delegation is the *write-and-validate-a-project-
   definition* boundary itself: both packages independently own 'persist
   a project definition, gated on confirmation, to the Sergeant-owned
   config path,' with zero delegation between them... fixing that
   duplication by delegation (B7's proposed remedy) would not actually
   resolve it, because both packages' write targets are already obsolete
   under the current estate model... `load-project` is not one of this
   pilot's nine packages and is not rewritten here; this is recorded as a
   cross-package note for whoever adjudicates it next." This pass is that
   next adjudication.

`docs/icm/retriage-2026-08-11.md` line 60 (MVP-5 F2, 2026-08-11, predating
both the Placement/Bounded-Judgment ladders and `estate-navigation`'s
current content) ruled `load-project`'s core "stays **WORKFLOW**:
resolving owning repos, roles, and a layered instruction set is real
judgment, differs per project." That verdict is superseded, not merely
disagreed with: it reasoned from upstream's multi-project abstraction
before `estate-navigation` existed and before the owner pre-ruling above
was recorded in this file's own header. Per the proposal's evidence
hierarchy (§2.5: "owner rulings and current measured Sergeant behavior >
... > committed proposals, gauntlet records, and retrospectives"), the
owner pre-ruling and the current schema outrank a pre-ladder retriage
note. This is not a new classification invented by this pass — it is the
same PL-0 check `docs/icm/convention.md` §2a rule 4 requires of "every
future classification pass," applied to the one package the sergeant-setup
pilot explicitly deferred.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-P5-090`/`091`, `BU-P1-132` (package identity: resolve project ownership/config/paths before work; trigger set) | `CONTEXT.md`, `index.md`; upstream `SKILL.md` lines 8, 12-13; `AGENTS.md` (upstream) L113 | PL-0 | J0 (registry concept absent from current product) | RETIRE | none — durable intent ("confirm repo/estate context before mutation") already ABSORBED at `skills/estate-navigation/SKILL.md` header and `AGENTS.md`'s live routing-table row 48 |
| `BU-P5-092` (unknown project name → `sgt-list`, require exact registered name) | `00-resolve-project-name/CONTEXT.md` | PL-0 | J0 | RETIRE | none — no "project name" concept exists to require; `AGENTS.md` Standard workflow loop step 1 ("never infer which repo or estate you're in from the current directory") already covers the underlying "don't proceed on an assumed target" intent for the current single-estate model |
| `BU-P5-108` (unregistered project → stop and ask to register) | `00-resolve-project-name/CONTEXT.md` | PL-0 | J0 | RETIRE | none — `estate-navigation`'s "Registering repos and groups interactively" section (already landed, ICM-R2) is the current analog, gated per-repository/group rather than per-project |
| `BU-P5-093` (`sgt-context`: record owning repos, paths, clone state, roles/groups, layered instructions, Graphify config) | `10-resolve-context/CONTEXT.md` | PL-0 | J0 | RETIRE / ABSORBED | `sgt-context` does not exist in sergeant-rs (estate-navigation header, above); the repo/group/health half is ABSORBED by `estate-navigation`'s "Resolving estate context" section (`sgt doctor`/`sgt repo list`/`sgt group list`); the instruction-layering half is ABSORBED at a *stronger* rung — `sgt` itself resolves and pins `[[repo]] instructions` at Work bind time (R-MVP1-4), so no actor judgment is needed here at all, not merely relocated judgment |
| `BU-P5-094` (raw YAML read only as fallback when `sgt-context` omits a field) | `10-resolve-context/CONTEXT.md` | PL-0 | J0 | RETIRE | none — no resolved-view-vs-raw-source duality exists; `sergeant.toml` is read directly if ever needed, there is no synthesized `sgt-context` view to prefer over it |
| `BU-P5-096`, `BU-P6-021` (completion evidence = context block showing every repo cloned, in one of three explicit clone states) | `10-resolve-context/CONTEXT.md` | PL-0 | J0 | RETIRE | none — `sgt-context`'s specific three-state output shape is an upstream binary's implementation detail; `sgt repo list`/`sgt doctor` (estate-navigation) are the current completion-evidence surfaces, with their own (Rust-owned, not workflow-content-owned) output shape |
| `BU-P6-022` (context block reports whether a built Graphify report exists or names the build command) | `10-resolve-context/CONTEXT.md` | PL-0 | J0 (out of this package's scope) | RETIRE | none for `load-project` — Graphify is its own separately-extracted N1 candidate (`docs/gauntlet/promoted-provenance/project-graph.md`); this citation was never more than a passthrough note inside `10-resolve-context` and has no `project-graph` package to hand off to today (that package's own admission is a separate, not-yet-run adjudication) |
| `BU-P5-111` (`sgt-context`/raw-YAML disagreement is blocking) | `10-resolve-context/CONTEXT.md` | PL-0 | J0 | RETIRE | none — no `sgt-context`/raw-source pair exists to disagree |
| `BU-P5-097`/`098`/`099`/`101`/`103` (read schema+existing YAML before edit; write only to `~/.config/sergeant/<project>.yaml`, no secrets; absolute-or-`dev_root`-relative paths; post-write verify via `sgt-list`+`sgt-context`; restore-prior-YAML or leave uncommitted on failure) | `20-register-or-edit/CONTEXT.md` | PL-0 | J5 (no-secrets is governing) + J0 (the rest: registry concept absent) | RETIRE / ABSORBED | The multi-project registry write target does not exist. The protective *intents* are already owned elsewhere at a stronger rung than a re-hosted workflow stage would provide: no-secrets-in-config is `AGENTS.md` Guardrails (`BU-0055`/`BU-0259`); "gated, individually-reversible, no monolithic preview-and-confirm ceremony" is `estate-navigation`'s "Registering repos and groups interactively" section (already landed, ICM-R2, itself HARVESTed from `sergeant-setup`'s own duplicate of this same boundary — see B7 finding above); atomic-write/no-half-written-state is now the CLI verb's own responsibility (`sgt repo add`'s idempotent, refusal-on-conflict mechanics), not something a wrapping workflow stage must implement |
| `BU-P5-095`/`102`/`109`/`110`, `BU-P6-013`/`014` (folded "sync repositories" helper: clone-if-missing, fast-forward-only pull, missing-URL/missing-executable diagnostics) | `20-register-or-edit/CONTEXT.md` "Retired helper content" | PL-0/PL-6 | J5 (governing: never force-merge/rebase a diverged branch) | STAND — already correctly re-homed | `sgt repo add <name> --origin <url>` (clone-if-missing half) plus the honestly-named "no pull verb yet" gap in `skills/estate-navigation/SKILL.md` (`docs/icm/re-homing-record-2026-08-12.md` line 39); no further action this pass, re-homing already executed at MVP-5 F2 |
| `BU-P6-012`/`035` (folded "report state" helper: per-repo clone/branch/cleanliness/ahead-behind status; open-task listing) | `20-register-or-edit/CONTEXT.md` "Retired helper content" | PL-0/PL-6 | J5 (never mutate while reporting) | STAND — already correctly re-homed | `sgt repo list` + `sgt doctor` (status/listing half); `sgt project status/sync`-style task-list slice recorded NET-NEW-SURFACE/unbuilt at `docs/icm/retriage-2026-08-11.md` lines 71/116, not re-litigated here |
| `CONTEXT.md` "Provenance" section's own admission — cites a `provenance.md` "never actually created for `load-project`," redirecting to `docs/gauntlet/promoted-provenance/load-project.md` | `CONTEXT.md` | N/A (authoring-hygiene, not a placement question) | J0 (self-documented dangling reference) | moot under RETIRE — would otherwise be `FOLD` (correct the pointer in place) if the package survived | n/a |

## Cross-package consequence: `to-tickets/00-load-project-context`

`to-tickets/CONTEXT.md`'s "Relationships to other workflows" section and
`00-load-project-context/CONTEXT.md`'s own "Delegation" section both name
**load-project** by identity: "This stage's outcome is produced by running
**load-project** to its own completion." No other live package under
`.sergeant/workflows/` or `skills/` names `load-project` (verified: grep
across `.sergeant/workflows/`, `skills/`, `AGENTS.md`,
`docs/DEVELOPMENT.md`, `docs/icm/` for the exact token, cross-checked
against `cross-repo-work/CONTEXT.md`, which despite an early false-positive
grep match contains no actual reference).

This pass does not edit `to-tickets` — it is a different package, out of
this Work's assigned surface, and the retirement itself has not been
reconciled/published yet (ADR 0013 decisions 6-7: a producer's own
adjudication does not self-promote). Recorded here as the concrete
follow-on this disposition creates, in the same shape the sergeant-setup
pilot recorded its own cross-package note: **when `load-project` is
actually retired at the reconcile-and-publish step, `to-tickets/00-load-
project-context/CONTEXT.md`'s "Delegation" section must be corrected in
the same change** to name `estate-navigation` (`skills/estate-navigation/
SKILL.md`) instead — the "Project context is loaded" outcome
`00-load-project-context` needs (repos, groups, health confirmed) is
exactly what `estate-navigation`'s "Resolving estate context" section
already produces. A dangling delegation to a retired package identity
left unresolved past reconciliation would itself be the same class of
defect `docs/icm/convention.md` §4 rule 1 already flags for a broken
`@@name` reference.

## Surviving package design

No workflow content survives. Every behavior unit above is either RETIRE
(no current-product analog — the multi-project registry concept itself)
or ABSORBED (an existing, already-landed surface — `estate-navigation`,
`AGENTS.md` Guardrails, or the `sgt` CLI's own mechanics — already owns
the protective or judgment intent, in most cases at a stronger rung than a
re-hosted workflow stage would provide). The two "already correctly
re-homed" rows (folded sync/report-state helpers) require no further
action; that re-homing was executed at MVP-5 F2 and is not revisited here.

Because no unit's destination requires new or substantially rewritten
content — every destination surface already exists and already carries
the relevant text — this disposition needs no `draft/` directory
(consistent with `task-intake-and-route`'s ICM-R2 precedent, ABSORBED,
which also produced no draft content; contrast `sergeant-setup`'s SPLIT,
which genuinely added new sections to `estate-navigation` and required
draft text for review). The only mechanical follow-on is the
`to-tickets` delegation correction above, which belongs to a different
package's own file and to the reconcile-and-publish step, not to this
producer's draft output.

## Inputs and outputs

Not applicable in the surviving sense — no package content survives.
Existing declared Inputs (all three stages already comply with
`record-shapes.md` §1a) and `output/README.md` Layer-4 declarations (all
three `evidence`-dispositioned) were read in full during Inventory; no
violation was found in either before concluding they retire along with
the package.

## Review and promotion policy

This adjudication record is draft producer output; it does not self-
promote (ADR 0013 decisions 6-7; `docs/icm/convention.md` §6.2). Landing
the retirement itself — removing `.sergeant/workflows/load-project/`,
updating `.sergeant/index.md`'s catalog (count and retirement note,
matching the existing `task-intake-and-route`/`sergeant-setup`/
`direct-implementation` entries there), and correcting `to-tickets/00-
load-project-context/CONTEXT.md`'s delegation — is the reconcile-and-
publish step's job (§8.12 of the proposal), gated on this record's own
independent review first.

## Alternatives considered

- **STAND**, per `docs/icm/retriage-2026-08-11.md`'s prior "stays
  WORKFLOW" verdict. Rejected: that verdict predates both ladders and
  `estate-navigation`'s current content, reasoned from upstream's
  multi-project abstraction, and is outranked by the owner pre-ruling
  recorded in `estate-navigation`'s own header plus the current
  `sergeant.toml` schema — the same PL-0-before-everything-else check
  `docs/icm/convention.md` §2a rule 4 requires and the sergeant-setup
  pilot already applied to its sibling package.
- **HARVEST the register/edit judgment into `estate-navigation`**, as
  `sergeant-setup`'s `30-project-interview` fragment was. Rejected as
  redundant, not wrong in kind: `estate-navigation`'s "Registering repos
  and groups interactively" section already covers this exact boundary
  (it was HARVESTed from `sergeant-setup`'s duplicate of the same
  write-and-validate intent at ICM-R2). Adding a second, near-identical
  section sourced from `load-project`'s own citations would itself be the
  duplication B7 originally flagged, recreated one hop later.
- **REHOME the whole package to `estate-navigation` as one lump.**
  Rejected for the same reason `sergeant-setup`'s pilot rejected it for
  its own package (`record-shapes.md` §6 rule 4, proposal §8.8:
  file-shape mirroring is not evidence of correctness) — and doubly so
  here, since there is no surviving new content to relocate at all; a
  REHOME record would misstate that something moves when nothing does.
- **Fix B7 literally as first proposed: make `20-register-or-edit`
  delegate to (or be delegated to by) a live registration procedure.**
  Rejected: the sergeant-setup pilot already found that literal fix
  insufficient because both packages' write targets were obsolete: this
  pass confirms `load-project`'s own write target independently, closing
  the cross-package note the pilot left open, rather than wiring two
  obsolete mechanisms together.

## Final disposition
ABSORBED

## Validation evidence

- **Source-valid:** every citation in `load-project`'s three stage
  `CONTEXT.md` files and its `CONTEXT.md`/`index.md` was read in full and
  cross-checked against its already-archived N1 provenance
  (`docs/gauntlet/promoted-provenance/load-project.md`); no new citation
  fabricated.
- **Placement-valid:** PL-0 was checked first, against the current
  product (three independent sources: `estate-navigation`'s own header,
  `docs/gauntlet/contracts/MVP-1.md`'s schema rulings, and the
  sergeant-setup ICM-R2 pilot's own B7 finding), before any lower rung was
  considered — per `docs/icm/convention.md` §2a rule 4's requirement that
  "every future classification pass lists the engine's capability surface
  beside the candidates."
- **Authority-valid:** moot under RETIRE/ABSORBED for the package itself;
  the destination surfaces (`estate-navigation`, `AGENTS.md`) already
  carry their own `## Bounded judgment`/Guardrails content, verified
  present (not re-derived) by reading `skills/estate-navigation/SKILL.md`
  in full, including its own ICM-R2 "Bounded judgment" section.
- **Structurally valid:** `workflow.toml`'s three-stage order, the three
  stage directories, and their `output/README.md` declarations agree
  (`docs/icm/convention.md` §1 rule 4) — verified directly before
  concluding retirement, not assumed.
- **Execution-valid:** out of scope for this producer pass (proposal
  §9.3's execution-validation claims are not re-run here); moot in
  practice once the package retires, since nothing will execute it going
  forward.
- **Package-specific hint check (per this Work's brief):** "No prior
  relationships-section delegation found. to-tickets (wave 2) delegates to
  this package" — confirmed: `to-tickets/00-load-project-context/
  CONTEXT.md` is the sole live delegator, addressed above under
  "Cross-package consequence." "GAUNTLET.md backlog item B7's finding
  (closed at ICM-R2, sergeant-setup): confirm load-project genuinely has
  no interview/preview/backup logic, matching what the sergeant-setup
  pilot pass found" — confirmed by direct re-reading of all three current
  stage `CONTEXT.md` files this pass, not merely cited from the prior
  pass's own record: `load-project` never conducts a field-by-field
  interactive interview, never previews a full file before write, and
  never backs up before overwrite (its own protective mechanism is
  read-schema-then-write, verify, restore-on-failure, a different and
  narrower shape than the interview `sergeant-setup` used to run).
- This record itself is a draft producer output, not yet independently
  reviewed (ADR 0013 decisions 6-7); it does not self-promote.
