export const meta = {
  name: 'n3-build-wave1',
  description: 'N3 wave 1: tagged stage definitions (B1), then the two-phase boundary + ask primitive (B2)',
  phases: [
    { title: 'B1', detail: 'domain: tagged stages, pinning, content identity, §22.3 matrix' },
    { title: 'B2', detail: 'engine/backend: two-phase boundary, #14, GP-2 ask, §22.5/22.6', model: 'opus' },
  ],
}
const REPO = '/home/user/sergeant-rs'
const G = `You are building sergeant-rs milestone N3. Read IN ORDER: ${REPO}/docs/gauntlet/contracts/N3.md (binding), ${REPO}/CLAUDE.md, ${REPO}/GAUNTLET.md register+backlog tables, ${REPO}/LESSONS.md, proposal §§11-14 + §22.3-22.6 in ${REPO}/reference/proposal-next-iteration-icm-workflows.md. Repo rules bind: gates before done (cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test), tests in the per-milestone suites via tests/support DataDir guard, every behavior pinned by a test that fails when reverted (L7), adjacent-append windows designed+tested (L6), fail closed on ambiguity. Commit your work in separable commits with clear messages ending "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>" — use Fixes #NN trailers ONLY on a commit that fully closes that issue. Do NOT push. Leak check before finishing: pgrep -af "debug/sgt --data-dir" | grep -v "bash -c" empty.`
const S = { type: 'object', required: ['commits', 'tests_added', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, tests_added: { type: 'number' }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } }

phase('B1')
const b1 = await agent(`${G}
Your scope — N3 Outcome 2 (tagged stage definitions), touching src/domain/workflow.rs (+ loader path) and tests/m3_execution.rs primarily:
- Legacy workflow.toml parses identically (existing tests stay green untouched where possible).
- Optional [stage."<id>"] tables: kind = "actor" (only legal kind this milestone), harness, profile. Unknown kinds, unknown fields, tables for undeclared stage ids, actor-only fields misplaced: all fail closed with structured errors (§22.3).
- Resolved WorkflowDefinition pins the full executor spec per stage; workflow.bound journals it; editing files after bind cannot change a running Work (existing pinning tests extended).
- Workflow content identity: a stable hash over execution-relevant resolved content (descriptor, stage order, contexts, executor metadata), journaled with workflow.bound; changes when any execution-relevant field changes (§22.3 pin).
Do NOT touch engine routing/execution (B2 builds on your types — keep StageDefinition's new fields additive and defaulted). Full §22.3 test matrix in m3.`,
  { label: 'B1:tagged-stages', phase: 'B1', model: 'sonnet', schema: S })
log(`B1: ${b1 ? b1.commits.length + ' commits, gates=' + b1.gates_green : 'FAILED'}`)
if (!b1 || !b1.gates_green) throw new Error('B1 not green — stop wave')

phase('B2')
const b2 = await agent(`${G}
Your scope — N3 Outcomes 1, 4, 5 (the engine seam). This is the milestone's heart; take the time to design before editing. Touching src/runtime/engine.rs, src/backend/mod.rs + claude.rs + fake.rs, src/runtime/recovery.rs, src/api.rs, tests/m4_backends.rs (+m2 where lock-discipline instrumentation lands):
1. Two-phase boundary per contract Outcome 1: execution.reserved appended under the core lock with pinned spec + allocated id; external launch/observe outside the lock; verified result append + transition back under the lock (id+attempt currency checks; late results subordinate to durable state per §14.5).
2. §15 trait evolution: prepare(request)->PreparedExecution then launch(prepared); stop/interrupt return join/completion handles the daemon awaits AFTER releasing the core guard — this closes #14/B3; put "Fixes #14" on exactly the commit that lands it with its pinning test (a stalled archive join must not block an independent request — instrumented test).
3. Claude start-window: session id journaled via reservation before spawn; recovery classifies the reserved-but-unlaunched window fail-closed.
4. GP-2/#42 ask primitive: BackendSignal pathway for an execution to park its stage needs_input on an actor-authored question, resumable via existing respond to the SAME execution (U1 semantics, docs/gauntlet/notes/n2-fake-backend-semantics.md). Fake backend: scriptable ask step + contract tests ("Fixes #42" on that commit). Claude adapter: measure what 2.1.226 stream-json offers for mid-turn/turn-end questions; if a clean mapping exists, implement + capability flag + opt-in contract test; if not measurable, flag stays false with the measurement recorded (L8) — do not fake it.
5. §22.5 crash-injection matrix over the new lifecycle (all eight windows, fake backend; "Fixes #20" on the commit completing the matrix) + §22.6 lock-discipline instrumentation with the N0 budget bounds.
Respect B1's landed types (read its commits first: git log). Keep fix/feature commits separable (L10).`,
  { label: 'B2:two-phase-engine', phase: 'B2', model: 'opus', effort: 'high', schema: S })
log(`B2: ${b2 ? b2.commits.length + ' commits, gates=' + b2.gates_green : 'FAILED'}`)
return { b1, b2 }
