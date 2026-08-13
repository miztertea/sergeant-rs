# MVP-5 ship gate — 2026-08-13

Assembled-product acceptance test (docs/gauntlet/notes/mvp-bucketing-2026-08-11.md
§A7 / goal prompt's MVP-5 exit), run as a fresh colleague using **only**
product surfaces (README.md, AGENTS.md, `skills/`, CLI `--help`/output,
`sgt doctor` remedies, `.sergeant/index.md`) — never GAUNTLET.md,
`docs/gauntlet/contracts/`, or journal internals. One environment fact was
assumed per the operator brief: `PATH="$HOME/.cargo/bin:$PATH"`.

Working tree: `/tmp/claude-1001/.../scratchpad/shipgate2/` (fresh clone of
`file:///home/miztertea/sergeant-rs` on `cerberus/mvp-1`, commit `d62578c`).

## Verdict

**PASS** — the full documented path (install → init → register two repos →
doctor green → real intent submitted on the `claude` backend → durable
execution across a restart → completed Work with a verifiable branch/diff
output) completed end-to-end using only documented `sgt` verbs, with zero
manifest hand-edits and zero journal decoding. Two real documentation/UX
gaps were found along the way (F2, F3 below) and worked around exactly the
way the product's own guardrail language prescribes (name the gap, don't
invent syntax) rather than papered over.

## Step-time table

| # | Step | Start (UTC) | End (UTC) | Result |
|---|---|---|---|---|
| 1 | `git clone file:///home/miztertea/sergeant-rs` | 00:48:09 | 00:48:10 | ok, branch `cerberus/mvp-1` checked out |
| 2 | Create two scratch git repos (`repo-alpha`, `repo-beta`) outside the estate | 00:48:43 | 00:48:43 | ok |
| 3 | `cargo install --path . --bin sgt` (README "Get it") | 00:48:44 (launched) | build itself: 3m 44s per cargo's own log | ok, `sgt` on `~/.cargo/bin` — **see F1 (operator dormancy) and F2 (silent binary replacement)** |
| 4 | `sgt init` (in the fresh clone) | 02:08:46 | 02:08:47 | ok — scaffolded `sergeant.toml`, `repos/`, `.gitignore`; auto-ran doctor, one expected `[warn]` (no repos declared yet) |
| 5 | `sgt repo add repo-alpha --origin file://…` / `repo-beta` | 02:08:54 | 02:08:54 | ok, both cloned into `repos/` and declared |
| 6 | `sgt doctor` | 02:08:58 | 02:08:58 | **all `[ok]`, `healthy`** — git, claude 2.1.229, Docker, data_dir, journal, projection, daemon, permission_mode, estate (2 repos), disk_pressure (7.2 GiB free). No EPERM/blob-store or disk-pressure warning was observed in this run (checked for per the coordinator's issue #67 note — not reproduced here) |
| 7 | Investigate model/profile config for "sonnet" (README, AGENTS.md, `sgt run --help`, `sgt doctor`, deliberate E5 probe) | 02:08:58 | 02:09:43 | **no documented mechanism found — F3** |
| 8 | `sgt run "Add a short docstring to the greet function in greet.py…" --backend claude --repo repo-alpha` | 02:09:43 | 02:09:43 | ok, submitted `01KZWE3VE3QM3VZ8ES5GM5JF6J`, state `active`, stage `00-prepare` |
| 9 | Detach — foreground `sleep 300` | 02:09:49 | 02:14:49 | no polling |
| 10 | `sgt status` / `sgt work show <id>` | 02:14:53 | 02:14:53 | `state: completed`, 4/4 stages (`00-prepare`→`10-implement`→`20-review`→`30-close`), envelope `turns_spawned: 4` of `turn_cap: 12` |
| 11 | `sgt work transcript <id>` | 02:14:59 | 02:14:59 | full causal 4-stage conversation, coherent with the diff |
| 12 | Verify output pointer (branch `sergeant/01KZWE3VE3QM3VZ8ES5GM5JF6J`, commit `89a30b4`) directly via `git show` | 02:15:xx | 02:15:xx | real docstring diff present in `repo-alpha`, exactly matching the intent |
| 13 | `sgt daemon stop` → `sgt status` (respawn) | 02:15:12 | 02:15:12 | daemon stopped cleanly; next client command auto-respawned it; `work list`/`repo list` unchanged — **state survived the restart** |
| 14 | Teardown: second `sgt daemon stop`, bracketed `pgrep`, docker check | 02:15:20 | 02:15:20 | daemon stopped, `pgrep -f "sgt [-]-data-dir"` empty, no sergeant-related Docker containers/images (two unrelated pre-existing `hello-world` containers, untouched) |

Active gate execution time (excluding the dormancy gap in step 3): roughly
10-11 minutes, dominated by the ~4-minute cold DuckDB/release build and the
mandated 5-minute detach sleep.

## Findings

**F1 — Operator dormancy during the background install (process finding, not a product defect).**
Severity: medium (process integrity of this gate run, not `sgt` itself).
After launching `cargo install` in the background, the operator ended its
turn to wait for the automatic completion notification instead of chaining
foreground `sleep 300` calls as instructed. Root cause: the runtime's own
guard explicitly blocked the instructed pattern (`sleep 300` immediately
followed by a status-check command) with `"Blocked: ... To wait for a
command you started, use run_in_background: true. Do not chain shorter
sleeps to work around this block."` — leaving no available compliant
mechanism to satisfy the absolute anti-dormancy rule for a >5-minute
background build via chained sleeps. Net effect was an unmonitored ~80
minute gap before an orchestrator nudge resumed the gate. Recommend the
gate procedure explicitly reconcile "foreground sleeps only" with the
harness's own sleep-chaining guard for any step expected to exceed one
sleep call (e.g., permit `run_in_background` + a single terminal
notification for the *install* step specifically, reserving the literal
foreground-sleep requirement for the step it was written for — the 5-minute
work-item detach, which *was* executed as a real foreground sleep in step
9).

**F2 — `cargo install --path . --bin sgt` silently replaces a different checkout's installed binary.**
Severity: medium (data/workflow-affecting for any developer with more than
one sergeant-rs checkout). Confirmed directly in the install log:

```
  Installing sergeant-rs v0.1.0 (/tmp/.../scratchpad/shipgate2/sergeant-rs)
  ...
   Replacing /home/miztertea/.cargo/bin/sgt
    Replaced package `sergeant-rs v0.1.0 (/home/miztertea/sergeant-rs)` with
    `sergeant-rs v0.1.0 (/tmp/.../scratchpad/shipgate2/sergeant-rs)`
    (executable `sgt`)
```

README's "Get it" section documents exactly `cargo install --path . --bin
sgt` with no warning that running it from a second checkout (e.g. this
ship-gate's disposable clone) silently overwrites whichever `sgt` binary a
different checkout — such as the user's primary, day-to-day
`/home/miztertea/sergeant-rs` — had installed to the same shared
`$CARGO_HOME/bin`. There is no confirmation prompt, no version/checkout
provenance check, and no note in README about the collision. This is the
same class of hazard CLAUDE.md's dev-facing "probe-copy" note already
warns internal contributors about, but ordinary README-following users get
no equivalent warning. Suggested remedy: README should either warn about
the shared-`$CARGO_HOME/bin` collision explicitly, or suggest
`--root <estate-local-dir>` for anyone who keeps more than one checkout.

**F3 — No documented way to select/pin the Claude model (e.g. "sonnet") for a Work item.**
Severity: medium-high (directly blocked the gate's literal instruction to
submit on a named model profile). Checked every product surface available:

- `sgt run --help` — no `--model` flag; only `--backend`, `--profile`,
  `--turns`, `--ceiling-secs`, etc.
- README's only description of what a profile configures: *"A profile can
  also pin the permission mode Claude turns launch with (`permission_mode
  = "acceptEdits"` in the profile's `options` table...)"* — permission mode
  only; model is never mentioned anywhere in README.
- `AGENTS.md` never mentions model selection (its one "model" occurrence is
  unrelated: "each model turn's raw output").
- `sgt doctor`'s `permission_mode` check reports each declared profile's
  effective *permission mode* — nothing about model.
- Deliberately probing the E5-shaped path: `sgt run "..." --backend claude
  --profile sonnet --repo repo-alpha` fails with `sgt: 422: no profile
  named "sonnet" in this workspace (has: )` — this names the absence of
  the profile but not the TOML syntax to declare one, let alone whether a
  profile can pin a model at all.

Net effect: real work in step 8 was submitted via `--backend claude` with
no profile and whatever model the `claude` CLI defaults to — the only path
actually supported by documented product surfaces. Inventing
`sergeant.toml` profile syntax to force a "sonnet" pin would have violated
the gate's own PASS bar ("no manifest hand-edits beyond documented
verbs"), so this is filed as a finding rather than worked around.

**F4 — Cosmetic rustup toolchain warning on a clean `cargo install`.**
Severity: low. On the very first `cargo install --path . --bin sgt` from a
brand-new clone, cargo/rustup emit: `warning: default toolchain implicitly
overridden with 'stable-x86_64-unknown-linux-gnu' by rustup toolchain
file... use 'cargo +stable install' if you meant to use the stable
toolchain`. Build still succeeds; a first-time README follower sees an
unexplained warning immediately with no note about whether it's expected.

## What worked cleanly (no findings)

- `sgt init` / `sgt repo add` / `sgt doctor` matched README/AGENTS.md
  documentation exactly, including the documented pre-estate doctor
  caveat.
- The full 4-stage `software-change` workflow (`00-prepare` →
  `10-implement` → `20-review` → `30-close`) ran unattended on the real
  `claude` backend and produced a correct, reviewed, committed diff
  matching the intent verbatim — verified independently via `git show` on
  the retained branch, not just trusted from `sgt work show`.
- `sgt work show`/`sgt work transcript` gave an honest, causally-ordered
  account of what happened, including the model's own review commentary
  (it flagged an untracked `__pycache__/` directory as out-of-scope rather
  than silently committing it).
- Restart resilience: `sgt daemon stop` drained and stopped cleanly; the
  very next client command (`sgt status`) auto-respawned the daemon with
  zero data loss — the completed Work item, its full record, and both
  declared repos were all present, exactly as `README.md`'s
  rebuild-from-journal model promises.

## Teardown

- Clone's daemon: stopped (`sgt daemon stop`, idempotent second call
  confirmed clean).
- `pgrep -f "sgt [-]-data-dir"`: empty (bracketed pattern, non-self-matching).
- Docker: no sergeant-related containers or images; two unrelated
  pre-existing `hello-world` containers left untouched.
- Scratch preserved at
  `/tmp/claude-1001/-home-miztertea-sergeant-rs/6c77471b-11a6-41b6-a88b-5d09cea538ff/scratchpad/shipgate2/`
  (clone, install log, two scratch repos, completed Work's retained branch
  and diff).
