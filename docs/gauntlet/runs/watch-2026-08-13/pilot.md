# WATCH §16.4 product pilot — 2026-08-13

The human-facing gate: the consuming harness (Claude Code, this session,
acting as the AGENTS.md orchestrator) delegates real Work and waits on
`sgt watch` alone — no polling. Binary under test: `target/release/sgt`
at `74a218c` (the branch's own build; the owner's installed `~/.cargo/bin/sgt`
was not touched).

## Path taken (product surfaces only)

1. **Estate**: `sgt init` in the checkout (doctor: healthy, 10 ok / 1 warn
   that cleared once a repo was declared). `sgt repo add sergeant-rs
   --origin /home/miztertea/sergeant-rs` — self-hosting, the estate's clone
   landed on `cerberus/watch`. A `[[profile]]` named `sonnet`
   (`default_model = "sonnet"`) hand-edited into `sergeant.toml` (the
   canonical pen) per the model-economy doctrine.
2. **Submit** 19:48:15Z: `sgt run "<DEVELOPMENT.md build-dir rule
   subsection>" --repo sergeant-rs --backend claude --profile sonnet
   --turns 8` → `01KZYAP2B84ZKJJ9BNZ78W7D45` (active).
3. **Wait** — the feature consuming itself: `sgt --json watch
   01KZYAP2B84ZKJJ9BNZ78W7D45` armed one-shot under the harness's Monitor
   facility (foreground process, silent while blocked). **Zero `sgt`
   inspection commands were run between submit and notice.**
4. **Notice** 19:50:58Z (2m43s submit→notice): exactly one
   `sergeant.watch/v1` JSONL object on stdout, exit 0.
   `reason: state_transition`, trigger `stage.completed` seq 101,
   snapshot `work.state: completed`, stage `30-close`, envelope
   `turns_spawned: 4 / turn_cap: 8`.
5. **R-WATCH-9's terminal lag observed live**: the notice's snapshot
   carried `output: null` — the teardown cascade had not yet landed at
   emission, exactly as the contract documents (no wait, no fabricated
   settledness). `sgt work show` moments later was settled:
   `clean: true`, retained branch `sergeant/01KZYAP2B84ZKJJ9BNZ78W7D45`,
   finalize commit `dbf3c487`, worktree removed.
6. **Collect**: the delivered commit (`docs(DEVELOPMENT): pin
   probe/verify-agent build-dir placement rule`, Sonnet trailer, exactly
   the asked shape) cherry-picked onto `cerberus/watch` as `b33eccc`.
7. **Teardown**: `sgt daemon stop` → "daemon stopped"; process table
   clean. The estate (manifest, clone, journal) is left in place as
   working state for the owner plus this pilot's evidence.

## Verdict

**PASS.** The whole loop ran on product surfaces: submit → detach →
one authoritative notice → collect via the output pointer. The envelope
spent: 4 of 8 turns, one stage-attempt, no retries, no needs_input.

## Environment evidence (recorded, not part of the acceptance result)

- Facility: Claude Code `Monitor` running the one-shot foreground
  `sgt --json watch` — chosen over a bare foreground tool call because
  this harness's foreground calls cap at 10 minutes (the R-WATCH-5
  guidance in AGENTS.md matches what this harness actually needed).
- Claude CLI drifted to 2.1.229 on this host (doctor row; L14
  drift-is-a-fact, not chased).

## Findings

1. **sergeant.toml is permanent untracked noise in the distro checkout**
   (#71): `sgt init`'s .gitignore entries cover `.sergeant/data`,
   `repos/`, and the manifest's temp files — not the manifest itself,
   which is per-installation and can never be committed. Every
   `git status` in a colleague's checkout shows `?? sergeant.toml`
   forever. Filed for adjudication, not fixed here.
2. No other deviations: no daemon was hand-started (submit auto-spawned
   it; `watch` was pointed at a live daemon), no manifest edits beyond
   the documented pens, no journal decoding.
