export const meta = {
  name: 's1-instrument-gauntlet',
  description: 'S1 phase 1: instrument repairs, repo hygiene, coverage harness — one Opus builder, gates green, pins revert-probed',
  phases: [
    { title: 'Instrument', detail: 'Opus builder: repairs + hygiene + harness per S1 contract phase 1' },
  ],
}

const REPO = '/home/user/sergeant-rs'

const BUILD_SCHEMA = {
  type: 'object',
  properties: {
    gates_green: { type: 'boolean' },
    summary: { type: 'string' },
    measurements: { type: 'array', items: { type: 'string' } },
    design_decisions: { type: 'array', items: { type: 'string' } },
    pin_probes: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          repair: { type: 'string' },
          probe: { type: 'string' },
          probe_failed_suite_as_expected: { type: 'boolean' },
        },
        required: ['repair', 'probe', 'probe_failed_suite_as_expected'],
      },
    },
    contract_ambiguities: { type: 'array', items: { type: 'string' } },
  },
  required: ['gates_green', 'summary', 'pin_probes'],
}

phase('Instrument')
const build = await agent(
  `You are the S1 phase-1 builder for sergeant-rs. Work in ${REPO} on the current
branch (never switch branches, never push). Read fully, in order:
docs/gauntlet/contracts/S1-COVERAGE.md (your contract — phase 1 is your entire
scope), docs/gauntlet/contracts/S0.md (rulings R-S0-1..11, binding),
reference/proposal-s-series-stabilization.md sections 5-9 and 14-15, LESSONS.md
(L1/L5/L7/L10 binding), CLAUDE.md, tests/support/mod.rs, the SpawnedDaemon impl in
tests/m6_surfaces.rs (around line 2245-2300), and scripts/demo.sh's cleanup.

Deliver EXACTLY the contract's phase-1 list, nothing more:
1. SpawnedDaemon::drop SIGTERM-first teardown (TERM, bounded wait, KILL fallback)
   + a pinning test asserting the dropped daemon exited via SIGTERM not SIGKILL.
2. reap_daemons reports TERM-vs-KILL per pid; extend the existing m2 reaper test
   to pin it.
3. scripts/demo.sh teardown: TERM, 10s grace, KILL escalation, verify-gone before
   rm -rf, exit nonzero if the daemon survives (fail closed). Record in a comment
   that this pin is partial and why.
4. .gitattributes: linguist-vendored on reference/ reference-corpus/ .sergeant/;
   linguist-generated on Cargo.lock. .gitignore: add __pycache__/ and *.pyc. Do
   NOT untrack the existing vendored .pyc (ruling R-S0-8).
5. .github/workflows/ci.yml: change trigger to push: branches: [main] plus
   pull_request. Change nothing else in the job.
6. .github/ISSUE_TEMPLATE/coverage-gap.yml with labels ["coverage"], mirroring
   the perf template's evidence discipline (uncovered BEHAVIOR, not lines).
7. scripts/coverage/: stage scripts embedding the R-S0-3 command lines verbatim
   (show-env + absoluteness check as stage C0; per-suite --no-report stages; the
   report stage; the F1/F2 census runners), each script: bash, set -euo pipefail,
   disk pre-flight refusing under 10GB free, rustc -vV + cargo-llvm-cov version
   recorded into the artifacts dir, per-stage profraw accounting
   (produced/merged/discarded counts), post-stage hygiene sweep
   (pgrep -f "llvm-cov-target/debug/sgt --data-dir" must find nothing), and a
   harness doc scripts/coverage/README.md that ALSO enumerates every wall-clock
   deadline in tests/ (grep for Duration::from_secs / from_millis in tests/ and
   the support module) ranked by measured-or-unknown headroom. Artifacts dir:
   docs/coverage/artifacts-2026-08-10/ (gitignored? NO — committed, it is
   evidence; keep artifacts small: logs and counts, never profraws or html).
   IMPORTANT measured-not-assumed (L1): verify cargo-llvm-cov 0.8.7's actual
   behavior for #[cfg(test)] code, tests/-file inclusion in reports, and clean
   --profraw-only BY RUNNING the tool (cargo llvm-cov --help / show-env /
   trivial probes are cheap; a full instrumented build is NOT needed to answer
   these), and record what you measured in the harness README.
Constraints: do NOT touch src/ or Cargo.toml (rulings), do NOT touch GAUNTLET.md
/ LESSONS.md / README.md / reference/ / docs/gauntlet/ beyond nothing at all —
your writes are tests/, scripts/, .github/, .gitattributes, .gitignore,
scripts/coverage/, docs/coverage/. The two opt-in real-Claude tests are never
run. The repo's cold build may still be settling — if cargo complains the target
dir is locked, wait for the lock, do not create alternate target dirs.
Gates green before returning: cargo fmt --check && cargo clippy --all-targets --
-D warnings && cargo test. Then revert-probe EVERY pin (L7) in a disposable git
worktree (git worktree add, never the main tree, L5): revert the repair, run the
pinning test, confirm it fails, restore, record each probe in pin_probes. After
worktree probes, rebuild the main checkout before any final measurement
(CLAUDE.md's shared-cache rule).
Return: gates_green, summary, measurements (each tool behavior you measured),
design_decisions (rung-tagged), pin_probes, contract_ambiguities.`,
  { label: 'build:s1-instrument', model: 'opus', effort: 'high', phase: 'Instrument', schema: BUILD_SCHEMA },
)

return { build }
