export const meta = {
  name: 'bs2b-probe-env',
  description: 'probe-env.sh: session-start environment facts probe (retro item 1), lean multi-axis loop in its own worktree',
  phases: [
    { title: 'Build', detail: 'Sonnet builder writes scripts/probe-env.sh + runs it' },
    { title: 'Review', detail: 'Opus critic: honesty + portability axes' },
    { title: 'Fix', detail: 'fixer over confirmed findings' },
  ],
}

const WT = '/home/miztertea/sergeant-probe-env'
const G = `You are working on sergeant-rs in the dedicated worktree ${WT} (branch cerberus/probe-env — NEVER touch /home/miztertea/sergeant-rs, the main checkout; another loop owns it). cargo is not needed for this work and you must not run it. Read first: ${WT}/docs/gauntlet/notes/session-retro-n-series-2026-08-11.md item 1 (the mandate), ${WT}/docs/environments/README.md (the convention the script serves), ${WT}/docs/environments/cerberus.md + claude-code-cloud.md + github-runner.md (the facts tables the script must be able to emit), ${WT}/CLAUDE.md. Commit in ${WT} with messages ending "Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>". Do NOT push.`

const BUILD_SCHEMA = { type: 'object', required: ['commits', 'gates_green', 'sample_output', 'notes'], properties: { commits: { type: 'array', items: { type: 'string' } }, gates_green: { type: 'boolean', description: 'shellcheck clean (if installed) + script runs green on this host' }, sample_output: { type: 'string', description: 'the script\'s full stdout from a real run on this host' }, notes: { type: 'string' } } }
const FINDINGS_SCHEMA = { type: 'object', required: ['findings', 'summary'], properties: { summary: { type: 'string' }, findings: { type: 'array', items: { type: 'object', required: ['id', 'severity', 'claim', 'evidence'], properties: { id: { type: 'string' }, severity: { type: 'string', enum: ['error', 'warning', 'info'] }, claim: { type: 'string' }, evidence: { type: 'string' } } } } } }

phase('Build')
const build = await agent(`${G}
Write ${WT}/scripts/probe-env.sh — the retro-item-1 deliverable: run once at session start on ANY host, measure and print the environment facts table for that host in the docs/environments/ format (a markdown table a session can paste into the host's fact file). Measure, never assume:
- uid/user/groups; whether DAC is enforced (chmod 000 self-read probe in a temp dir);
- CAP_LINUX_IMMUTABLE viability (chattr +i on a temp file, cleaned up ALWAYS — trap-based cleanup);
- disk: free space on \$HOME and \$TMPDIR filesystems, quota tooling presence;
- O_DIRECT open behavior on \$TMPDIR and on \$HOME's fs (python3 one-liner probe is fine — python3 presence is itself a measured fact with a graceful degrade);
- network posture: proxy env vars; HTTPS reachability of https://api.github.com and https://raw.githubusercontent.com (HTTP code, bounded timeout, failure is a fact not an error);
- Docker: present? daemon reachable? storage driver, cgroup version, user-in-docker-group; do NOT pull images or run containers (session-start must be cheap and offline-safe) — report "runtime lifecycle unprobed (use the docs/environments checklist)";
- claude CLI: presence, --version, auth status via claude auth status --json if available (loggedIn + authMethod fields only, never tokens);
- cargo/rustc presence and versions, noting the PATH caveat if found only in ~/.cargo/bin;
- cores, kernel, whether the host looks like a container (/.dockerenv, /proc/1/cgroup heuristics — report the evidence, not a guess);
- IS_SANDBOX and root: if uid==0, state the claude skip-flag refusal fact and the IS_SANDBOX lever (cite src/backend/claude.rs module docs).
Rules: every probe fails soft (a missing tool prints "unmeasurable: <why>" and continues — the script NEVER exits nonzero because a fact is absent; it exits nonzero only on its own bugs, set -u discipline); no writes outside mktemp dirs; cleanup on EXIT trap; output is pure stdout, two sections: the markdown table, then a one-line "paste destination" hint naming docs/environments/<hostname>.md. Run it on this host; verify its facts against docs/environments/cerberus.md — every overlapping row must agree (they were measured independently this session); disagreement is a bug in your script OR a stale fact file: investigate which and say so in notes. If shellcheck is installed, it must pass clean. ONE commit.`,
  { label: 'build:probe-env', phase: 'Build', model: 'sonnet', effort: 'high', schema: BUILD_SCHEMA })
log(`build: ${build ? build.commits.length + ' commits, green=' + build.gates_green : 'FAILED'}`)
if (!build || !build.gates_green) throw new Error('probe-env build not green')

phase('Review')
const review = await agent(`You are a blind critic (R-S0-12: code is code — even a probe script gets fresh eyes). Work read-only in ${WT}; the diff is the last commit (git show HEAD). Read the mandate first: docs/gauntlet/notes/session-retro-n-series-2026-08-11.md item 1, docs/environments/README.md, and GAUNTLET.md's register (L3). Axes, both yours:
(1) MEASUREMENT HONESTY: does every printed fact come from a probe actually executed in this run? Any row that could print a stale/assumed/hardcoded value on some host is an error. Any probe whose failure mode could print a WRONG fact (rather than "unmeasurable") is an error. Check the chattr/temp cleanup paths actually run under set -u + early exits (trace the trap).
(2) PORTABILITY/SAFETY: run the script yourself (it is cheap and read-only by design — verify that claim while you're at it: any write outside mktemp dirs is an error, any image pull or container run is an error, any nonzero exit on a merely-missing tool is an error). Does it degrade gracefully with tools absent (simulate: PATH= subsets)? Would it behave on a root container and a GH runner per their fact files' recorded shapes?
Findings with file:line + executed evidence. Do not edit any file.`,
  { label: 'critic:probe-env', phase: 'Review', model: 'opus', effort: 'high', schema: FINDINGS_SCHEMA })
const errors = review ? review.findings.filter(f => f.severity === 'error') : []
log(`review: ${review ? review.findings.length : 0} findings, ${errors.length} errors`)

phase('Fix')
let fix = null
if (review && review.findings.length > 0) {
  fix = await agent(`${G}
You are the fixer. Close every error finding; warnings fixed or declined-with-reason in notes; info acknowledged. Findings from the blind critic:
${JSON.stringify(review.findings, null, 2)}
Re-run the script after fixing; verify the cerberus.md agreement check still holds; shellcheck clean if installed; separable commit(s).`,
    { label: 'fix:probe-env', phase: 'Fix', model: 'sonnet', effort: 'medium', schema: BUILD_SCHEMA })
  log(`fix: ${fix ? fix.commits.length + ' commits' : 'none needed/FAILED'}`)
}

return { build: { commits: build.commits, notes: build.notes }, findings: review ? review.findings.length : 0, errors: errors.length, fix: fix ? fix.commits : null }
