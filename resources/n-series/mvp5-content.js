export const meta = {
  name: 'mvp5-content',
  description: 'MVP-5 CONTENT: AGENTS.md rewrite from the invariant corpus, symlink, operator skills, library re-homing, README — then the assembled-product ship gate from a fresh clone + fresh context',
  phases: [
    { title: 'Content', detail: 'lane F1: AGENTS.md + symlink + README; lane F2: skills port + library re-homing' },
    { title: 'Panel', detail: 'Opus critics: vision-fidelity + content-honesty over the OS layer' },
    { title: 'ShipGate', detail: 'fresh clone, fresh harness context, the full colleague path, no rescue' },
  ],
}
const REPO = '/home/miztertea/sergeant-rs'
const BASE = '14909fd'
const G = `You are building sergeant-rs MVP-5 (CONTENT — the OS layer) on Cerberus, branch cerberus/mvp-1. Binding, read in order: ${REPO}/NORTH-STAR.md (the destination + R-NS rules; AGENTS.md is the canonical boot contract), ${REPO}/docs/gauntlet/notes/mvp-bucketing-2026-08-11.md MVP-5 section, ${REPO}/docs/gauntlet/goal-prompt-mvp.md §MVP-5, ${REPO}/reference/sergeant-upstream/AGENTS.md (the SHAPE to re-imagine: routing table + standard workflow loop — with sgt verbs replacing bin/ scripts), ${REPO}/docs/icm/retriage-2026-08-11.md (re-homing verdicts, both passes), ${REPO}/docs/environments/cerberus.md. PATH="$HOME/.cargo/bin:$PATH". SCRATCH DISCIPLINE: never ~/. Gates: cargo fmt --check && clippy -D warnings && cargo test (content changes must not break the suite — the workflow-catalog tests read .sergeant/). L10 separable commits; commits end "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Fixes #53/#57 on the commits that close them. Do NOT push.`
const BUILD = { type: 'object', required: ['commits', 'gates_green', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean' }, notes: { type: 'string' } } }

phase('Content')
const f1 = await agent(`${G}
Lane F1 — the boot contract:
1. REWRITE AGENTS.md as the canonical front door (upstream's shape, sergeant-rs's verbs): what sergeant is (North Star destination, 3 sentences max), the trigger->skill/workflow routing table, the standard workflow loop (load estate context via sgt doctor/repo list -> check running work via sgt status/work list -> shape intent (structured fields optional) -> choose workflow -> sgt run with envelope flags -> monitor via work show/transcript -> respond to needs_input -> collect via the output pointer), when NOT to use sgt (single-turn asks — the OS owns the routing judgment per North Star ruling 4), and the working-on-sergeant-itself row (the dev rulebook is repo content: CLAUDE.md's dev sections). CONSUME the 126 adjudicated agents-invariant units: docs/gauntlet/runs/n2-run4/.sergeant/workflows/repo-to-icm/40-classify/output/classifications.ndjson (representation=agents-invariant) — each invariant either lands in AGENTS.md (cited by BU id in an HTML comment), lands in a skill, or is recorded as not-adopted in a dispositions appendix doc (docs/icm/agents-invariant-dispositions.md). No invariant silently dropped.
2. CLAUDE.md -> AGENTS.md as git symlink is the RULING — but CLAUDE.md currently holds the dev rulebook this repo's sessions need. Resolution per North Star + clone-is-distro: move the dev rulebook content to a clearly-named repo doc (docs/DEVELOPMENT.md), AGENTS.md's routing table points "working on sergeant-rs itself" at it, THEN replace CLAUDE.md with the symlink (git mv semantics: rm + ln -s AGENTS.md CLAUDE.md, committed). Verify the suite still passes (tests that read CLAUDE.md's text may need their paths updated to docs/DEVELOPMENT.md — do it deliberately).
3. README.md recentered on clone-and-work: the colleague path (gh repo clone; cargo install --path . --bin sgt — honestly noting build time; sgt init; open your harness; speak an intent), what sergeant is (distro framing, engine as the core), pointers to AGENTS.md and the workflow catalog. Keep the dev/record links (GAUNTLET, NORTH-STAR) in a contributors section.`,
  { label: 'F1:boot-contract', phase: 'Content', model: 'sonnet', effort: 'high', schema: BUILD })
log(`F1: ${f1 ? f1.commits.length + ' commits, green=' + f1.gates_green : 'FAILED'}`)
if (!f1 || !f1.gates_green) throw new Error('F1 not green')

const f2 = await agent(`${G}
Lane F2 — skills + library re-homing (execute the retriage verdicts, both passes, docs/icm/retriage-2026-08-11.md):
1. Create the operator-skills layer: skills/<name>/SKILL.md at repo root (upstream's convention). Port sergeant-help (from .sergeant/workflows/sergeant-help against the REAL current CLI — verify every verb it names exists in sgt --help), re-home grilling + grill-with-docs as skills (conversation is the harness's job — R-NS-6; note their workflow packages retire), and port the surviving upstream operator skills' CONTENT where the function map ruled SKILL (sgt-context/sgt-sync equivalents become estate-navigation guidance referencing sgt repo list/doctor).
2. Retire the ABSORBED/CLI-SURFACE packages from .sergeant/workflows/ per the verdicts: respond-to-worker (absorbed: sgt respond), monitor-fleet, project-graph, reconcile-and-cleanup-fleet, wake-and-resume, deliver-external-callback, drain-fleet (absorbed by MVP-3's drain or gated), route-review-findings, wiki-digest (parked), load-project + sergeant-setup per their SPLIT verdicts (workflow cores stay, CLI slices retire), monitor-fleet->helper. Each retirement: git rm the package + one line in a docs/icm/re-homing-record-2026-08-12.md with the verdict citation; provenance preserved in git history. The 21 surviving workflows + splits stay.
3. Fix #53 (partition-checkpoint-protocol.md's wrong retry verb -> the two measured resume paths; Fixes #53) and #57 (validate-and-ship S4 defect; Fixes #57) in passing.
4. Run the §9.7 validator (--admitted mode) over the final library; all survivors pass or are fixed.`,
  { label: 'F2:skills-rehoming', phase: 'Content', model: 'sonnet', effort: 'high', schema: BUILD })
log(`F2: ${f2 ? f2.commits.length + ' commits, green=' + f2.gates_green : 'FAILED'}`)
if (!f2 || !f2.gates_green) throw new Error('F2 not green')

phase('Panel')
const FINDINGS = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }
const axes = [
  { key: 'vision-fidelity', brief: `Axis VISION-FIDELITY: does AGENTS.md teach the North Star's actual loop (estate -> intent -> workflow -> walk away -> return, R-NS rules respected)? Does every verb/flag it names EXIST in the current CLI (run each against sgt --help / the binary — a boot contract citing phantom verbs is the harness-amnesia problem reborn)? Is the when-NOT-to-use-sgt judgment present (North Star ruling 4)? Does README's colleague path actually work command-by-command on this host?` },
  { key: 'content-honesty', brief: `Axis CONTENT-HONESTY: were the 126 invariant units genuinely consumed (spot-check 15 BU ids end-to-end: unit -> AGENTS.md/skill/dispositions appendix)? Are the re-homing retirements exactly the retriage verdicts (no extra deletions, none skipped)? Do #53's protocol fix and #57's fix match their issues? Does the validator actually pass over the final library (re-run it)?` },
]
const crits = await parallel(axes.map(a => () => agent(
  `Blind critic, sergeant-rs MVP-5 content. Read-only in ${REPO} (running sgt --help / the validator is allowed; NEVER edit). Read: NORTH-STAR.md, the MVP-5 bucketing section, retriage doc, GAUNTLET register (L3). Diff: git diff ${BASE}..HEAD. ${a.brief} Findings with file:line evidence.`,
  { label: `critic:${a.key}`, phase: 'Panel', model: 'opus', effort: 'high', schema: FINDINGS })))
const all = crits.flatMap((c, i) => (c ? c.findings.map(f => ({ ...f, id: `${axes[i].key}:${f.id}` })) : []))
log(`panel: ${all.length} findings (${all.filter(f => f.severity === 'error').length} error)`)
let fix = null
const actionable = all.filter(f => f.severity !== 'info')
if (actionable.length) {
  fix = await agent(`${G}\nMVP-5 fixer. Close every finding below or record a rejection reason; ledger entry for the pass.\nFINDINGS: ${JSON.stringify(actionable, null, 2)}`,
    { label: 'fixer', phase: 'Panel', model: 'sonnet', effort: 'high', schema: BUILD })
  if (!fix || !fix.gates_green) throw new Error('fixer not green')
}

phase('ShipGate')
const ship = await agent(`You are the SHIP GATE operator for sergeant-rs — the assembled-product test, per docs/gauntlet/notes/mvp-bucketing-2026-08-11.md §A7 and the goal prompt's MVP-5 exit. You simulate a colleague with a FRESH context: you may read ONLY what the product itself surfaces (README, AGENTS.md, skills, CLI output, doctor remedies) — NOT the gauntlet record, NOT the contracts, NOT this prompt's history. Any time you need knowledge the product didn't give you, that is a FINDING (the product failed), not a thing to work around silently.
The literal path, in a scratch dir (/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/shipgate/, never ~/):
1. Fresh clone of ${REPO} (file:// clone of the current branch). Install per README's documented method. Time each step with real timestamps.
2. sgt init per README/AGENTS.md. Register two repos (create two tiny scratch git repos as the estate's working set) via the documented verbs.
3. Follow AGENTS.md as a harness would: shape an intent for a small real task in one of the repos, pick a workflow from the catalog per the routing table, submit with envelope flags per the docs.
4. Let it run (real claude backend, sonnet profile per the docs' guidance — if the docs don't say how to configure the profile, FINDING). Detach (no polling for 5+ min). Return via sgt status / work show / work transcript; find the branch + outputs via the output pointer.
5. Restart resilience: sgt daemon stop mid-idle, restart via any client command, verify state survives.
6. Verdict per the gate's own rules: NO hand-editing the manifest beyond documented verbs, NO journal decoding, NO orchestrator rescue. Every deviation = a finding with severity. PASS only if the whole path completed on product surfaces alone.
Write docs/gauntlet/runs/shipgate-2026-08-12/report.md: timed step table (measured timestamps), findings list, PASS/FAIL verdict with the one-sentence reason. Commit on cerberus/mvp-1 ("MVP-5 ship gate: <verdict>"), do NOT push. Teardown: daemon stopped, bracketed pgrep clean, scratch preserved. Return: verdict, step times, findings count, top 3 findings.`,
  { label: 'shipgate', phase: 'ShipGate', model: 'sonnet', effort: 'high', schema: { type: 'object', required: ['verdict', 'findings_count', 'top_findings', 'notes'], properties: { verdict: { type: 'string' }, findings_count: { type: 'number' }, top_findings: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } } } })
return { f1: f1.commits, f2: f2.commits, panel: { findings: all.length, actionable: actionable.length }, fix: fix ? fix.commits : 'none', shipgate: ship ? { verdict: ship.verdict, findings: ship.findings_count, top: ship.top_findings } : 'FAILED' }
