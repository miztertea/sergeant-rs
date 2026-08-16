# Package adjudication: sergeant-setup

Producer pass, ICM-R2 pilot (`docs/adr/0013-icm-r0-owner-rulings.md` decisions
8-9; `reference/proposal-icm-r-procedure-authority.md` §8). Draft for
independent review — not a landed change (ADR 0013 decision 6).

## Original intention

Capture a complete project definition through interview and track any
capability gap discovered along the way as approved work — historically the
judgment half of "bring an installation from any partial state to a
verified-complete state," with the mechanical bootstrap/repair half already
split out to `sgt init`/`sgt doctor` at a prior pass (MVP-5 F2
execution-surface re-triage, 2026-08-12, `docs/icm/retriage-2026-08-11.md`,
`docs/icm/re-homing-record-2026-08-12.md`). That prior SPLIT left this
package as a two-stage "workflow core": `05-file-capability-gaps` and
`30-project-interview`.

## Current trigger and outcome

Per `.sergeant/workflows/sergeant-setup/CONTEXT.md` and `index.md` (status:
`published`, version 3): triggered by first install, a new project/repository
to register, a broken or incomplete installation, or a verification request.
Outcome: each unsupported capability becomes an approved tracked issue or a
reported gap, and a complete project definition is captured, previewed, and
written only after confirmation.

**Already-dead in practice.** `AGENTS.md`'s own trigger→skill/workflow
routing table (lines 42-49) has no row for `sergeant-setup` at all. Its two
relevant triggers are already claimed by other surfaces: "The estate isn't
set up yet, or `sgt doctor` reports a fixable install/config fault" routes to
`sgt init`/`sgt doctor` directly ("not a skill — CLI verbs"), and "Before
acting in an estate whose repos/groups/health aren't already confirmed this
session" routes to `estate-navigation`. `AGENTS.md` states explicitly:
"setup/repair routes to `sgt init`/`sgt doctor`, not a skill, either way"
(`BU-1262`). `sergeant-setup` survives only as a catalog row in
`.sergeant/index.md` (`published`) that nothing in current operating
doctrine ever routes a harness to.

## Driver and admission boundary

As authored: `stage-actor`, admission boundary `always` (an ordinary admitted
workflow, two actor stages, `.sergeant/workflows/sergeant-setup/workflow.toml`).
Neither surviving stage carries the `## Bounded judgment` section
`docs/icm/convention.md` §6.1 (ADR 0013 decision 4) now requires of every
actor stage — both instead carry an older `## Judgment required` section
predating that requirement. Moot given the disposition below, but recorded
as an independent convention-compliance defect found in the same pass.

## Behavior-unit dispositions

| Unit | Source | PL rung | J boundary | Disposition | Destination |
|---|---|---:|---|---|---|
| `BU-P5-004` (write only to `~/.config/sergeant/config.yaml` / `<project>.yaml`) | `.sergeant/workflows/sergeant-setup/_config/standing-constraints.md` | PL-0 | J5 (governing constraint) | ABSORBED | `AGENTS.md` Guardrails ("`sgt init`/`sgt repo add`/`sgt group add` write only within the estate they scaffold... never to another harness's own configuration" — `BU-1263`/`BU-1264`/`BU-1295`) |
| `BU-P5-005` (never write other tools' config surfaces) | same | PL-0 | J5 | ABSORBED | same `AGENTS.md` guardrail |
| `BU-P7-036` (skill must explicitly say "Never write to" those paths) | same | PL-0 | J5 | ABSORBED | same `AGENTS.md` guardrail |
| `BU-P5-006` (td/Graphify/Treehouse never auto-initialized without consent) | same | PL-1 | J5 | HARVEST | `AGENTS.md` Guardrails, new bullet — orphaned: its owning stages (`60-task-tracking-init`, `70-optional-capabilities`) already retired to CLI-SURFACE at MVP-5 F2 and this policy was never carried into `docs/icm/re-homing-record-2026-08-12.md`; it currently constrains no live behavior anywhere in the corpus |
| `BU-P5-012` (unsupported capability → drafted td issue, y/yes gated, else reported) | `05-file-capability-gaps/CONTEXT.md` | PL-2 | J2 (delegated: draft issue content, judge severity/necessity) | HARVEST | `skills/estate-navigation/SKILL.md`, new section continuing its existing "a missing tool or capability surfaces as `sgt doctor`'s named remedy" coverage — this is exactly PL-2's own listed example, "decides whether work should remain direct or become durable Work," conducted live rather than as an admitted background Work |
| `BU-P5-023` (skip new-project interview in favor of "Phase 5" repair) | `30-project-interview/CONTEXT.md` | PL-0 | J0 (dangling reference) | RETIRE | none — cites a stage (`40-repair-existing`, "Phase 5") retired at MVP-5 F2; the redirect target no longer exists in this workflow. The underlying intent (don't re-run a fresh-registration interview against something already registered) needs re-derivation against `sgt repo add`'s already-idempotent, no-op-on-existing behavior, not restatement of a dead stage name — parked as an open item, not silently resolved |
| `BU-P5-024` (strict-order interview: project name, per-repo name/path/URL/role/group, group description + shared `agent_instructions`, default instructions, GitHub identity, Graphify path) | same | PL-0 (mostly) / PL-2 (fragment) | J0 (schema mismatch) | RETIRE, with one HARVEST fragment | Most of this citation targets fields (`agent_instructions` free text, per-repo role, project-level GitHub identity) that do not exist in the current `sergeant.toml` schema (`[estate]`/`[[repo]]`/`[group.<name>]`, `docs/gauntlet/contracts/MVP-1.md` R-MVP1-3/R-MVP1-4/R-MVP1-5 — only `instructions = "local"\|"suppress"` per repo, no free text) — parked pending a schema decision this docs-only pass does not make. The transplantable fragment (ask for repo name + clone origin + group membership, iterate until the user is done) HARVESTs into `estate-navigation` alongside `BU-P5-012` |
| `BU-P5-026` (preview full file + explicit confirmation before write) | same | PL-0 | J5 | ABSORBED | No single "write the whole project file" moment survives under the current incremental `sgt repo add`/`sgt group add` model; the protective intent is already covered by `AGENTS.md`'s destructive-state guardrail (`BU-0050`: standing authorization never extends to destroying preserved state) and by `sgt init`'s already-idempotent no-op-not-reset behavior |
| `BU-P5-027` (timestamped `.bak` before overwrite) | same | PL-0 | J5 | ABSORBED | same reasoning as `BU-P5-026` — no whole-file overwrite moment remains to protect |
| `BU-P8-045` (validation: name matches filename, unique repo names/paths, clone URLs present, real roles/groups, non-vague instructions) | same | PL-0 (mostly) | J0 (schema mismatch) | RETIRE | Same schema mismatch as `BU-P5-024`; the "instructions must state commands and observable constraints, not vague quality slogans" quality bar is worth re-applying if/when a free-text instructions field exists in `sergeant.toml`, but nothing to attach it to today |

## Surviving package design

No workflow survives. The package's behavior is absorbed, harvested, or
retired as tabulated above:

- **`estate-navigation` (`skills/estate-navigation/SKILL.md`)** gains two new
  sections: (1) a capability-gap-to-tracked-work procedure (from
  `BU-P5-012`), consent-gated exactly as originally specified; (2) an
  interactive repo/group-registration walkthrough targeting the actual
  current schema (from the transplantable fragment of `BU-P5-024`), replacing
  the retired multi-project-registry interview. See
  `docs/gauntlet/runs/icm-r2/sergeant-setup/draft/skills/estate-navigation/SKILL.md`
  for the full revised text.
- **`AGENTS.md`** gains one new Guardrails bullet (`BU-P5-006`, orphaned
  consent-gating policy); its other absorbed units already have a citation
  there and need no edit. See
  `docs/gauntlet/runs/icm-r2/sergeant-setup/draft/AGENTS.md-guardrails-addendum.md`
  for the proposed bullet in isolation (not a full-file rewrite — `AGENTS.md`
  is out of this pass's file scope beyond this one additive bullet, and even
  that addition is left for the reconcile-and-publish step to accept, not
  applied here).
- **`.sergeant/workflows/sergeant-setup/`** retires. Draft retirement content
  (index.md/CONTEXT.md/workflow.toml) mirroring the live tree is at
  `docs/gauntlet/runs/icm-r2/sergeant-setup/draft/.sergeant/workflows/sergeant-setup/`.
  `.sergeant/index.md`'s catalog row updates to `retired` pointing at this
  adjudication, in the same change that lands the retirement (not performed
  here — draft only).
- **Two parked items** (`BU-P5-023`/`024`'s dangling-reference and
  schema-mismatch fragments, `BU-P8-045`) are not resolved by this pass —
  they require a `sergeant.toml` schema decision (free-text instructions?
  GitHub identity? Graphify path?) this docs-only ICM-R workstream has no
  authority to make (runtime freeze, ADR 0013 decision 10). Recorded as an
  open item for whoever next touches the estate manifest schema.

## Inputs and outputs

Retired package: no inputs/outputs survive. Draft `estate-navigation`
addendum's own inputs/outputs are unchanged from the existing skill (live
`sgt doctor`/`sgt repo add`/`sgt group add`/`sgt repo list`/`sgt group list`
command results — no new Layer-4 artifacts, matching the skill's existing
pattern of consuming live command output rather than declared workflow
inputs).

## Review and promotion policy

Per `docs/icm/convention.md` §6.2/§6.3 (ADR 0013 decisions 6-7): this
adjudication and its draft content are not promotable by their own producer.
Independent review (a fresh execution, explicit inputs, review-only
contract, no edit authority) must check: source fidelity of every citation
above; whether the `estate-navigation` draft addendum correctly reflects the
current `sergeant.toml` schema and not an invented one; whether the B7
backlog finding (below) is characterized accurately; and whether RETIRE is
the right call for the schema-mismatched fragments versus a more aggressive
attempt to map them. Only after independent review and owner reconciliation
does any of this draft content land.

## Alternatives considered

- **REHOME the whole package to `estate-navigation` as one lump.** Rejected:
  record-shapes.md §6 rule 4 and proposal §8.8 both warn against file-shape
  mirroring; the package's units have genuinely different destinations
  (`AGENTS.md` for the write-scope invariant, `estate-navigation` for live
  interview content, RETIRE for schema-mismatched fragments) — a single
  REHOME target would either drop the `AGENTS.md`-bound units or force them
  into `estate-navigation` where they don't belong topically.
- **Keep `30-project-interview` as a PL-4/PL-5 workflow stage, per its own
  file's "as extracted" classification.** Rejected: this is the corpus's own
  named U3/engine-gap-G5 case (`reference-corpus/synthesis.md` §5) — a
  multi-round, data-dependent human interview needing a re-enterable
  `needs_input` stage the current engine does not yet support faithfully.
  Reclassifying the interview as PL-2 (live Captain-session dialogue)
  resolves G5 as a side effect: a Captain skill talks to the user turn by
  turn in one live session with no engine-side re-entry mechanism required.
  This is not claimed as a general solution to G5 (other packages' cases may
  still need it) — only that this specific instance's PL-2 fit removes the
  need for it here.
- **Fix B7 as originally proposed: make `30-project-interview` delegate to
  `load-project`'s `20-register-or-edit` stage instead of reimplementing
  it.** Rejected as the terminal fix, though the underlying observation is
  confirmed (see next section) — `load-project` itself still describes the
  same obsolete `~/.config/sergeant/<project>.yaml` multi-project-registry
  model (its own `CONTEXT.md` says so explicitly: "sergeant-rs's estate model
  is `sergeant.toml`, per-directory, not a multi-project registry... out of
  this re-homing pass's scope"). Wiring `sergeant-setup` to delegate to a
  target that is itself built on an obsolete model would not fix the
  underlying defect, only relocate it. `load-project` is not one of this
  pilot's nine packages and is not rewritten here; this is recorded as a
  cross-package note for whoever adjudicates it next.
- **Leave `05-file-capability-gaps` as a workflow stage** (it has a real
  durable artifact — a filed td issue — unlike the interview's engine-gap
  problem). Considered seriously; rejected because its own trigger set
  (first install / broken install / verification request) is the same live,
  conversational, pre-durable-Work context as the interview, and its
  behavior contract's own shape — "decide whether an unsupported capability
  becomes tracked Work" — is PL-2's own textbook example almost verbatim.

## Final disposition
SPLIT

## Validation evidence

- **B7 backlog item (`GAUNTLET.md` line 54, `docs/icm/retriage-2026-08-11.md`
  line 61): "`30-project-interview` duplicates `load-project`'s registration
  job wholesale... instead of delegating."** Checked directly against
  current content (`load-project/00-resolve-project-name/CONTEXT.md`,
  `10-resolve-context/CONTEXT.md`, `20-register-or-edit/CONTEXT.md`, all read
  in full this pass). **Confirmed, with one correction to the original
  framing's precision:** the word "wholesale" overstates the literal overlap
  — `load-project`'s three stages never conduct a field-by-field interactive
  interview, never preview a full file, and never back up before overwrite;
  that content is unique to `sergeant-setup`. What is genuinely duplicated
  without delegation is the *write-and-validate-a-project-definition*
  boundary itself: both packages independently own "persist a project
  definition, gated on confirmation, to the Sergeant-owned config path," with
  zero delegation between them (`30-project-interview`'s own file admits
  this — "this stage's own 'duplicates load-project's registration job
  wholesale' defect... is unresolved by this pass too — flagged, not
  fixed," carried unchanged from the MVP-5 F2 pass into the live file today).
  The deeper finding this pass adds: fixing that duplication by delegation
  (B7's proposed remedy) would not actually resolve it, because both
  packages' write targets are already obsolete under the current estate
  model — see the SPLIT verdict and alternatives above.
- **SS12.3 package-specific hypothesis, checked point by point against
  current content, not assumed:**
  - *"Spans PL-0 retirement (setup/repair absorbed by `sgt init`/`sgt
    doctor`)"* — **already executed**, prior to this pass, at MVP-5 F2
    (`docs/icm/retriage-2026-08-11.md`, `docs/icm/re-homing-record-2026-08-12.md`);
    this pass's own PL-0 findings above (the six `ABSORBED`/`RETIRE` rows)
    extend that retirement to the remaining two stages' content, which the
    F2 pass had not reached.
  - *"PL-2 skill behavior (live project interview)"* — **confirmed**, but the
    live evidence for it is stronger and more direct than the hypothesis
    states: it is not merely that the interview *resembles* Captain-skill
    material — `AGENTS.md`'s own routing table already routes both of this
    package's triggers ("estate isn't set up" and "repos/groups... not
    confirmed") to `sgt init`/`sgt doctor` and `estate-navigation`
    respectively, and states outright that "setup/repair routes to `sgt
    init`/`sgt doctor`, not a skill, either way" (`BU-1262`) — meaning
    current operating doctrine has already stopped sending a harness to this
    workflow, independent of this adjudication.
  - *"Capability-gap judgment"* — **confirmed live and re-homed**, not
    retired: `BU-P5-012` survives as real, still-needed judgment (see
    dispositions table), HARVESTed into `estate-navigation` rather than
    retired, because `sgt doctor` only detects/remedies what it mechanically
    can — the td-issue-or-report decision on what it can't remains actor
    judgment.
  - *"Historical upstream project-YAML concepts that don't map to the current
    estate model"* — **confirmed, and the strongest single finding of this
    pass**: `skills/estate-navigation/SKILL.md` already states, in its own
    header, that the upstream `~/.config/sergeant/*.yaml` project registry
    "does not exist in sergeant-rs and is not re-created here," naming the
    real model (`sergeant.toml`'s `[estate]`/`[[repo]]`/`[group.<name>]`,
    `docs/gauntlet/contracts/MVP-1.md`) as its replacement. Every citation in
    `_config/standing-constraints.md` and most of `30-project-interview`'s
    citations describe the retired model by name (`~/.config/sergeant/
    <project>.yaml`, `dev_root`, a free-text `agent_instructions` field, a
    project-level GitHub identity field) — none of which exist in the schema
    `estate-navigation` and `docs/gauntlet/contracts/MVP-1.md` actually
    document.
- **Files read in full this pass:** `.sergeant/workflows/sergeant-setup/{CONTEXT.md,index.md,workflow.toml,_config/standing-constraints.md,05-file-capability-gaps/CONTEXT.md,05-file-capability-gaps/output/README.md,30-project-interview/CONTEXT.md,30-project-interview/output/README.md}`;
  `.sergeant/workflows/load-project/{CONTEXT.md,00-resolve-project-name/CONTEXT.md,10-resolve-context/CONTEXT.md,20-register-or-edit/CONTEXT.md}`;
  `skills/estate-navigation/SKILL.md`; `AGENTS.md` (routing table and
  Guardrails sections); `docs/icm/retriage-2026-08-11.md`;
  `docs/gauntlet/runs/n2-run4/.sergeant/drafts/workflows/sergeant-setup/provenance.md`
  (the only extant `provenance.md` found for this package — the live
  package's own `CONTEXT.md` cites a `provenance.md` that does not exist at
  `.sergeant/workflows/sergeant-setup/provenance.md`; the archived
  `docs/gauntlet/promoted-provenance/sergeant-setup.md` referenced by
  `index.md` is the real trail and was consulted for stage-history context,
  not re-read line by line since the retired stages are out of scope for
  reclassification);
  `docs/gauntlet/contracts/MVP-1.md` (schema sections R-MVP1-3/4/5);
  `GAUNTLET.md` (B7 row); `reference-corpus/synthesis.md` (G5/U3/X8).
