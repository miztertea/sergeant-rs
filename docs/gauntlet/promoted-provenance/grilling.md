# Provenance — Grilling

Maps every stage (and every workflow-level citation) to the behavior units that justify it, per `docs/gauntlet/contracts/N1.md` and `docs/icm/record-shapes.md` §3. Source snapshot: `reference/sergeant-upstream` at the SHA recorded in `reference/UPSTREAM.md`. Synthesis basis: `reference-corpus/synthesis.md` §1, candidate **W28** `grilling`.

## Workflow-level citations

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-005` | grilling is a workflow that interviews the user to stress-test a plan, decision, or idea, triggered by an explicit request or grill trigger phrases. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (frontmatter: description) |

## Stages

### `00-interview-loop`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-006` | The interview proceeds systematically down a decision tree, resolving dependent decisions in order, with the actor offering a recommended answer alongside each question. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 6) |
| `BU-P3-007` | Within the interview loop, only one question is posed at a time, and the actor waits for the user's answer before asking the next. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 8) |
| `BU-P3-008` | The interview loop draws a firm line: facts discoverable by exploring the environment must be looked up by the actor; only genuine decisions are put to the user, and the actor waits for the user's answer on each. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 10) |

### `10-confirm-understanding`

| Unit | Statement | Source |
|---|---|---|
| `BU-P3-009` | The workflow may not proceed to action until the user explicitly confirms shared understanding has been reached. | `reference/sergeant-upstream/.agents/skills/grilling/SKILL.md` (body line 12) |

## Curation note (promotion, 2026-08-11)

**NEEDS-JUDGMENT — canonical G5 case, resolved by scripted acceptance run.**
`docs/icm/promotion-spec-2026-08-11.md` §5 flags `grilling` as the corpus's
canonical engine-gap **G5** case (re-enterable `needs_input` for a
multi-round interview, `reference-corpus/synthesis.md` §5's G5 entry,
"survives, narrowed") and notes §3's unscripted acceptance gate never
exercises this path. Per that spec's caveat, a second, scripted
(`SGT_FAKE_SCRIPT`) acceptance pass was run in a package-private scratch
subject/data-dir before promotion, distinct from and in addition to the
standard §3 unscripted run (which also passed cleanly: `work.completed`,
`stage_bindings` == `workflow.toml`'s two-stage order, distinct
`execution_id` per stage).

Script: `needs_input:...;needs_input:...;needs_input:...;complete:...;
needs_input:...;complete:...` — three sequential dependent questions
inside `00-interview-loop`, then its completion, then one confirmation
question inside `10-confirm-understanding`, then its completion. Four
`sgt respond` round trips were driven against the running work.

**Observed mechanism (journal evidence, not asserted from docs):** each
`needs_input` round inside `00-interview-loop` produced
`stage.needs_input` → `work.needs_input` → (respond) →
`stage.input_received` (the answer, journaled) → `work.resumed` →
`stage.resumed`, all three rounds under **one** `execution.started` /
`execution_id` and **one** `stage.entered` (`attempt: 1`) — the engine's
built `begin_input`/`PendingSend` path (`src/runtime/engine.rs`,
`src/api.rs` `work_input`) resumes the *same* execution via a fresh
backend `send`, it does not open a new execution per question. This is
the durable-session/fresh-turn shape CLAUDE.md's own architecture invariant
names ("a Claude session is a durable conversation identity; the OS
process exists per turn") rather than the literal "re-entered as a fresh
[Sergeant] execution" phrasing in `synthesis.md`'s G5 narrowing — the two
read as the same underlying design once "execution" is read as "turn," not
as Sergeant's own `execution_id`. The workflow reached `work.completed`
(`stages: 2`) with no structural anomaly (no duplicate `stage.entered`, no
orphaned execution, `stage.input_received` correctly attributed to
`00-interview-loop` for all three answers and to `10-confirm-understanding`
for its own).

This confirms `00-interview-loop`'s own "Additional note" ("each unanswered
question may end the stage in `needs_input` and re-enter it") is
mechanically true of the engine as built today, for a bounded three-round
case. It is **not** a claim that this exhausts G5's full narrowed scope
(unbounded rounds, real-backend `--resume` semantics under this same
primitive) — only that the shape this package depends on is not vaporware.
No package content was changed to reach this conclusion; the resolution is
this verification record, per
`docs/icm/promotion-spec-2026-08-11.md`'s NEEDS-JUDGMENT handling.

**Promotion note (`docs/icm/promotion-spec-2026-08-11.md` §1).** This
package declares a `promote` output disposition
(`10-confirm-understanding/output/README.md`) at its true closing stage
with no finalize step — one of the 30 of 34 N1 packages in that shape, not
one of the 3 (`drain-fleet`, `respond-to-worker`, `to-spec`) that name one.
Recorded here per the spec's finalize-gap rule rather than silently
promoted; disposition is left to human review at merge time, not applied
mechanically by this curation act.

