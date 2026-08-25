# Sergeant workflow catalog

Root catalog (dev-corpus convention record §1 rule 1; §1 rule 2). Lists
every `status: published` workflow under
`.sergeant/workflows/`. This file is the list, not an entry — it carries no
`kind: workflow` front matter of its own, and no per-workflow authored field
(description, tags) is duplicated here beyond a short pointer; each
workflow's own `index.md` is the source for those (dev-corpus convention
record §3 rule 2, the same anti-duplication rule that governs `AGENTS.md`).

7 packages (down from 19 at the 2026-08-22 distro content rebuild — down
from 35 at the MVP-5 F2 execution-surface re-triage, 2026-08-12; down from
23 at ICM-R2, 2026-08-16; down from 20 at ICM-R3, 2026-08-16; up to 19 at
the 2026-08-20 wave-4 backlog close-out, which authored and published
`validate-intent` (#201) and `record-decisions` (#88)). The 2026-08-22
rebuild's authority is the owner's rulings of that date and the dev-corpus design proposal (W2);
its design record and derivation for every package below is dev-corpus
provenance, kept in this project's private development record, not
shipped here.

Prior retirements: 12 at the pre-2026-08-22 waves — 9 CLI-SURFACE, 1
OPERATOR-SKILL, and the 2 R-NS-6-dissolved `grilling`/`grill-with-docs` —
to `skills/` (operator skills, this repository's canonical skill root) or
to a dev-corpus re-homing record (CLI-verb candidates and
engine gaps); ICM-R2 (dev-corpus provenance, owner ruling
ICM-R0): `task-intake-and-route`
(ABSORBED), `sergeant-setup` (SPLIT, into `skills/estate-navigation/SKILL.md`
and `AGENTS.md`'s Guardrails), `direct-implementation` (HARVEST, into
`AGENTS.md`'s "When NOT to use `sgt`"); ICM-R3
(dev-corpus provenance): `tdd`
(REHOME, into `.sergeant/common/contexts/tdd.md`/`test-quality.md` —
themselves consolidated away at this round, see below), `to-spec`
(REHOME, into `skills/to-spec/SKILL.md`), `load-project` (ABSORBED, into
`estate-navigation`).

## The 2026-08-22 retirement log

Eighteen packages leave `.sergeant/workflows/` at this round, all
`git rm -r`'d; provenance for every one survives in git history.

| Package | Disposition | Destination |
|---|---|---|
| `code-review` | reshape | → `review-change`; `references/smell-baseline.md` moves to `review-change/20-panel/references/` |
| `cross-repo-work` | dissolve | Ownership/dependency-order planning is Captain intent-shaping; the fan-out is runtime. Multi-repo is topology, not a workflow |
| `deepen-module` | retire | → roadmap technique (the `tdd` precedent exactly) |
| `diagnose-bug` | reshape | → `fix-defect` |
| `dispatch` | cut | Its `05-classify-risk` safety-keyword routing is a live gap (named in the head PR's ratify list); `80-monitor`'s reconciliation doctrine is engine-true prose that leaves with it |
| `implement` | absorb | → `implement-change` (its two stages are the loop's middle) |
| `prototype` | retire | Zero live bindings; a spike is a change with a throwaway intent. Retiring it required a second, undeclared `tests/` edit beyond the wave's one named exception (`tests/f_doctrine_skew.rs`, §7.5): `tests/m8_estate_cli.rs`'s `run_workflow_resolves_against_a_freshly_initialized_estate` retargeted its `--workflow` argument from `prototype` to `investigate` (commit `89fa3a52`), or `cargo test --locked` would 422 against a retired package. Named here, on the head PR's ratify list |
| `record-decisions` | absorb | → `author-document` profile (a section, not a construct — see that package's own Notes for reviewers) |
| `recover-stalled-worker` | retire | → engine lead (recovering sergeant's own stalled worker is engine behavior) |
| `repo-to-icm` | relocate | → the private workspace repo's own `.sergeant/local/workflows/`; takes the library's only `kind = "execute"` stage with it — this library no longer exercises `kind = "execute"` anywhere |
| `research` | absorb | → `investigate` |
| `resolving-merge-conflicts` | retire | Its one rule becomes `@@resolve-conflicts` (`.sergeant/common/contexts/resolve-conflicts.md`) |
| `to-tickets` | retire | → roadmap (zero live bindings) |
| `triage` | retire | → roadmap (zero live bindings) |
| `validate-intent` | absorb | → W3's `define-acceptance`; spec-fidelity survives as a panel axis, one stage later than it would have fired |
| `vet-external-skill` | retire | Vetting a skill is a Captain act; zero bindings |
| `wayfinder` | absorb | → `investigate`; its `00-name-destination` stage is deleted, not carried — naming a destination requires a live interview, which R-NS-6 places in Captain, before dispatch |
| `worker-mission` | absorb | → `implement-change` (the loop with a brief as input) |

Also at this round: `.sergeant/common/contexts/tdd.md` and
`test-quality.md` are deleted, consolidated into a single
`.sergeant/common/contexts/test-first.md` (both were already dangling
references to the retired `tdd` package's own earlier rehome).

## The seven packages

| Workflow | Status | Index |
|---|---|---|
| `author-document` | published | [`workflows/author-document/index.md`](workflows/author-document/index.md) |
| `fix-defect` | published | [`workflows/fix-defect/index.md`](workflows/fix-defect/index.md) |
| `implement-change` | published | [`workflows/implement-change/index.md`](workflows/implement-change/index.md) |
| `investigate` | published | [`workflows/investigate/index.md`](workflows/investigate/index.md) |
| `remediate-findings` | published | [`workflows/remediate-findings/index.md`](workflows/remediate-findings/index.md) |
| `review-change` | published | [`workflows/review-change/index.md`](workflows/review-change/index.md) |
| `validate-and-ship` | published | [`workflows/validate-and-ship/index.md`](workflows/validate-and-ship/index.md) |

## The unnamed case

Omitting `--workflow` binds `software-change`, the four-stage loop
compiled into the binary (`00-prepare`, `10-implement`, `20-review`,
`30-close`), recorded on the Work with `source: embedded`. It is not
listed above because it is not a package under `.sergeant/workflows/` —
there is nothing on disk to fork or version. It is the default this
catalog's seven named packages are alternatives to, and selecting one of
them is a choice that should be made and stated rather than defaulted
into.

`.sergeant/drafts/workflows/` holds generated, human-reviewable candidates —
never admitted procedure, never listed here (dev-corpus convention record §2).

Operator skills (never dispatched as Work; loaded directly by the harness)
live at `skills/<name>/SKILL.md`; see `skills/` for the current fourteen.
