export const meta = {
  name: 'm0-bootstrap-gauntlet',
  description: 'Scaffold sergeant-rs crate + CI per M0 contract, then blind-critique',
  phases: [
    { title: 'Build', detail: 'Sonnet builder scaffolds crate and CI, drives gates green' },
    { title: 'Critique', detail: 'Opus blind critic grades diff vs contract' },
    { title: 'Verify', detail: 'Opus refuters adversarially verify findings' },
  ],
}

const REPO = '/home/user/sergeant-rs'

phase('Build')
const FINDINGS_SCHEMA = {
  type: 'object',
  properties: {
    findings: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          axis: { type: 'string' },
          file: { type: 'string' },
          summary: { type: 'string' },
          evidence: { type: 'string' },
          severity: { type: 'string', enum: ['error', 'warning', 'info'] },
        },
        required: ['axis', 'summary', 'evidence', 'severity'],
      },
    },
  },
  required: ['findings'],
}

const buildReport = await agent(
  `You are the M0 builder for the sergeant-rs prototype. Work in ${REPO}.

Read docs/gauntlet/contracts/M0.md — it is your contract. Also read section 35 of
reference/proposal-depot-rust-execution-surface.md for the module layout it cites.

Outcome that must be true when you finish:
- A single Rust crate at the repo root: package name "sergeant-rs", binary named "sgt"
  (use [[bin]] name = "sgt", path src/main.rs).
- Module skeleton per the contract: src/main.rs, src/cli.rs, src/daemon.rs, src/api.rs,
  src/tui.rs, src/web.rs, src/telemetry.rs, src/domain/ (work.rs, execution.rs,
  workspace.rs, workflow.rs, event.rs + mod.rs), src/runtime/ (journal.rs, projection.rs,
  router.rs, recovery.rs, surface.rs + mod.rs), src/backend/ (mod.rs, claude.rs, codex.rs,
  fake.rs). Stubs only — modules compile, contain at most doc comments and empty item
  declarations; NO speculative types or logic. main.rs wires clap minimally: "sgt" with a
  --version flag and a stub "sgt status" subcommand that prints a placeholder line.
- Dependencies: only serde (derive), serde_json, clap (derive), tokio (rt-multi-thread,
  macros), ulid, blake3, toml, tracing, thiserror. Use cargo add so versions resolve to
  current. No other deps.
- .gitignore for Rust (target/, plus .DS_Store).
- rust-toolchain.toml pinning channel "stable".
- .github/workflows/ci.yml: on push + pull_request; ubuntu-latest; stable toolchain with
  clippy + rustfmt components; steps: cargo fmt --check, cargo clippy --all-targets --
  -D warnings, cargo test. Use dtolnay/rust-toolchain@stable and Swatinem/rust-cache@v2.
- Do NOT touch reference/, GAUNTLET.md, LESSONS.md, docs/, or README.md. Do not run any
  git commands — the orchestrator owns git.

Drive the gates green yourself before returning: cargo build, cargo fmt --check,
cargo clippy --all-targets -- -D warnings, cargo test. Fix until all pass.

Return raw JSON: {"gates_green": bool, "gate_output_tail": "<last lines of each gate>",
"files_created": [paths]}.`,
  { label: 'build:scaffold', model: 'sonnet', effort: 'medium', phase: 'Build' },
)

phase('Critique')
const critique = await agent(
  `You are a blind critic in a gauntlet loop. You did not write this code. Work in ${REPO}.

Grade the CURRENT UNCOMMITTED CHANGES (run: git status --short, then read every new
file except anything under reference/) against docs/gauntlet/contracts/M0.md on two axes:

- spec-fidelity: does the scaffold match the contract exactly (crate name, binary name,
  module layout, dependency list, CI gates)? Any extra or missing pieces?
- simplicity: is there ANY speculative structure, premature abstraction, logic in stubs,
  or dependency beyond the contract's list? Minimality is a contract term here.

Rules: evidence-only — every finding must cite a file and what you actually saw in it.
Do not grade style preferences. Do not propose new features. An empty findings list is a
valid and welcome outcome if the work genuinely passes.`,
  { label: 'critic:spec+simplicity', model: 'opus', effort: 'high', phase: 'Critique', schema: FINDINGS_SCHEMA },
)

phase('Verify')
const confirmed = []
if (critique && critique.findings.length > 0) {
  const verdicts = await parallel(
    critique.findings.map((f, i) => () =>
      agent(
        `Adversarially verify this code-review finding in ${REPO}. Try to REFUTE it by
reading the actual files and the contract docs/gauntlet/contracts/M0.md. Finding:
${JSON.stringify(f)}
Return raw JSON: {"refuted": bool, "reason": "<one sentence grounded in what you read>"}.
Default to refuted=true if the evidence does not clearly support the finding.`,
        {
          label: `refute:${i}`,
          model: 'opus',
          effort: 'medium',
          phase: 'Verify',
          schema: {
            type: 'object',
            properties: { refuted: { type: 'boolean' }, reason: { type: 'string' } },
            required: ['refuted', 'reason'],
          },
        },
      ).then((v) => ({ finding: f, verdict: v })),
    ),
  )
  for (const v of verdicts.filter(Boolean)) {
    if (v.verdict && !v.verdict.refuted) confirmed.push(v)
  }
}

return { buildReport, critiqueCount: critique ? critique.findings.length : 0, confirmed }