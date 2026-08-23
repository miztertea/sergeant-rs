# Sergeant

<p align="center">
  <img src="docs/img/logo.png" alt="Sergeant" width="760">
</p>

<p align="center">
  <strong>Define intent with Captain. Execute it with Sergeant.</strong>
</p>

<p align="center">
  Sergeant turns your coding harness into <strong>Captain</strong> using <code>AGENTS.md</code> and skills. Captain helps you understand the work, define what you want, and delegate it. Sergeant executes that intent through durable workflows across one repository or an entire software estate.
</p>

<p align="center">
  <a href="https://github.com/miztertea/sergeant-rs/releases"><img alt="Release" src="https://img.shields.io/github/v/release/miztertea/sergeant-rs"></a>
  <a href="https://github.com/miztertea/sergeant-rs/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/miztertea/sergeant-rs/actions/workflows/ci.yml/badge.svg?event=pull_request"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20WSL-blue">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue"></a>
</p>

Sergeant is local-first, designed for one developer per installation, and currently pre-1.0. It runs on Linux, macOS, and Windows through WSL2.

## Get started

You need:

- **Git**
- **[Claude Code](https://claude.com/claude-code), [Codex](https://developers.openai.com/codex/cli), or [OpenCode](https://opencode.ai)** — agent workflow stages execute on any of the three; everything else works without them via a deterministic test backend
- **Docker**, if your workflows include deterministic container stages — optional otherwise, and `sgt doctor` will tell you either way

Install the latest release:

```sh
curl -fsSL https://github.com/miztertea/sergeant-rs/releases/latest/download/sergeant-rs-installer.sh | sh
```

Prebuilt binaries cover x86_64 Linux and Apple Silicon macOS; other platforms build from source with Rust (see [CONTRIBUTING.md](CONTRIBUTING.md)). The installer verifies its download against published checksums (v0.1.2 and later); to verify provenance independently, run `gh attestation verify` against the [release](https://github.com/miztertea/sergeant-rs/releases) assets.

Create an estate and launch your coding harness:

```sh
mkdir -p ~/estates/my-product
cd ~/estates/my-product
sgt init
sgt claude
# or: sgt codex / sgt opencode / sgt goose
```

Then talk to Captain:

> This estate contains our payments API, auth service, customer web app, and engineering knowledge repo. I can give you the local locations or Git remotes for each one.

Captain will help register the repositories, organize them into groups when useful, and verify the estate. Then describe the work:

> Add account lockout after five failed login attempts. Update the web UI to explain why the account is locked, and record the new behavior in our engineering knowledge.

Captain shapes that conversation into intent, selects the appropriate workflow and repository scope, and hands the work to Sergeant.

You do not need to learn Sergeant's CLI before using Sergeant. Captain is an expert in operating it on your behalf.

## Captain defines. Sergeant executes.

Coding harnesses are excellent at conversation and judgment. Terminal sessions, model processes, and context windows are poor places to keep durable execution state. Sergeant separates those responsibilities:

| Captain | Sergeant |
|---|---|
| Your coding harness + `AGENTS.md` + skills | The `sgt` binary + its estate-bound daemon |
| Talks with you | Executes accepted intent |
| Explores, interviews, and clarifies | Runs durable, ordered workflows |
| Decides what should become Work | Owns Work state and lifecycle |
| Chooses scope and workflow | Creates Git surfaces and drives stages |
| Brings decisions back to you | Journals what happened and why |

The split carries into how each side is extended. A Captain *skill* is an interactive way of working inside your current conversation — building an estate, grilling an incomplete idea, turning a discussion into a specification. A Sergeant *workflow* begins after intent has been accepted: a self-contained execution procedure with explicit stages, context, inputs, and outputs.

```text
You
 │
 ▼
Captain — coding harness + AGENTS.md + skills
 │
 │  accepted intent
 ▼
Sergeant — durable execution engine
 │
 ▼
Workflow
 ├── stage → fresh agent execution
 ├── stage → deterministic container execution
 ├── stage → fresh agent execution
 └── stage → review or close-out
 │
 ▼
Your estate — one repository or many
```

## Work across an estate

An estate is any directory initialized by Sergeant: the exact directory containing `sergeant.toml`.

```text
my-product/
├── sergeant.toml
├── AGENTS.md
├── skills/
├── .sergeant/
└── repos/
    ├── payments-api/
    ├── auth-service/
    ├── customer-web/
    └── engineering-knowledge/
```

An estate can contain one repository or hundreds. It gives Sergeant a light-monorepo view of repositories that belong together, without requiring you to combine them into a monorepo. A single Work might touch an API, an authentication service, a frontend, and documentation: Sergeant binds all selected repositories to the same Work, gives each one a Work-owned Git surface, and carries the intent and workflow across the complete scope.

There is no prescribed estate size. Keep one broad estate divided into groups, or several purpose-built estates (`~/estates/client-a`, `~/estates/homelab`, …). Each estate has its own manifest, runtime state, journal, retention policy, and daemon binding, and multiple estates can be active simultaneously. `sergeant.toml` declares the estate's repositories, Git origins, groups, routing profiles, and retention policy; Captain handles the normal setup conversation, and the manifest remains available when you want direct control.

## Bring your knowledge with the work

The `engineering-knowledge` repository in the example estate above is not decoration. When a Work should update your architecture notes, runbooks, or decision records, Captain includes the knowledge repository in the Work scope — and it receives the same branch, workflow context, review, and evidence trail as the source repositories. Documentation stops drifting from the systems it describes, because it ships in the same Work.

Sergeant does not impose a memory system of its own: the estate carries durable knowledge as ordinary repositories, and Captain keeps whatever working memory your harness already uses.

## Make good work repeatable

A workflow is a standard way of working expressed as a directory:

```text
.sergeant/workflows/implement-change/
├── workflow.toml
├── CONTEXT.md
├── 00-orient/
│   └── CONTEXT.md
├── 05-baseline/
│   └── CONTEXT.md
├── 10-implement/
│   └── CONTEXT.md
├── 15-validate/
│   └── CONTEXT.md
├── 20-panel/
│   └── CONTEXT.md
├── 25-refute/
│   └── CONTEXT.md
├── 30-fix-confirmed/
│   └── CONTEXT.md
├── 35-re-verify/
│   └── CONTEXT.md
└── 40-close/
    └── CONTEXT.md
```

Every stage is its own execution with its own contract. A stage declares the context and inputs it needs, produces explicit outputs, and ends before the next stage begins — context moves between stages through declared artifacts, not through one enormous agent conversation that must remain alive forever. A stage can be a fresh agent execution when judgment is needed, a Docker execution for deterministic work, repository-native scripts and tests, or a human decision when authority runs out. Agents where reasoning helps. Containers where determinism helps.

Sergeant ships with seven named workflows — implementing a change (with an in-loop review panel, refuters, and a re-verify pass over the fix), fixing a defect, investigating a bounded question, reviewing a diff that arrives from outside a Work, remediating a typed set of findings, authoring a document, and validating and shipping — the [workflow catalog](.sergeant/index.md) lists them all. Captain discovers them through the catalog and selects one based on the accepted intent; omitting `--workflow` binds the embedded default loop, which the catalog is a set of deliberate alternatives to, never a silent fallback.

### Write your own workflows

Custom workflows are encouraged. Tell Captain how your team works and ask it to create one. New candidates land under `.sergeant/drafts/workflows/<name>/`, where they are reviewable but not runnable; after review, they are promoted into `.sergeant/workflows/<name>/`. That publication boundary is deliberate: generated procedure never becomes runnable merely because an agent wrote it. The complete authoring model is the [ICM filesystem convention](docs/icm/convention.md).

## Work survives the session

A Sergeant Work is not a terminal, a model process, or a chat session — it is durable state. You can:

- close the terminal
- lose your network connection
- put the laptop to sleep
- accidentally kill the daemon
- hit a model usage window
- restart the machine

Sergeant records meaningful state transitions in a crash-tolerant journal. When the estate comes back, Sergeant reconstructs Work from that evidence: if recovery is unambiguous, execution continues; if the evidence is not sufficient, Sergeant blocks with a stated reason rather than guessing.

That makes resuming conversational:

> My usage window reset. Resume the account-lockout Work.

Captain inspects the recorded state and uses Sergeant's retry, extension, response, and recovery surfaces as appropriate. It does not have to reconstruct the run from memory.

The durability promise is not that a process survives every interruption. The promise is that retained Work is faithfully resumable after the process does not.

## A lot happens after you say "go"

| Sergeant handles | What that gives you |
|---|---|
| Durable Work identity | Work outlives terminals, processes, and individual model turns |
| Append-only journal and replay | State is reconstructed from recorded evidence, never guessed |
| One Git surface per Work and repository | Parallel work never shares an ordinary working tree |
| Explicit `needs_input` state | Agents ask instead of inventing an answer |
| Turn and time envelopes | An execution cannot consume unbounded agent turns by accident |
| Retry, extension, cancellation, and recovery | A failed stage does not restart the entire effort |
| Dirty-state accounting | Uncommitted, misplaced, or unaccounted-for Git results stay visible |
| Transcripts and provenance graphs | You can inspect the conversation and causal history of any Work |

Sergeant treats a Work's Git worktrees as its declared mutation surface. It records integrity problems and estate drift, but it is not an operating-system security boundary. For stronger isolation, run Sergeant and your harness inside a VM, container, restricted account, or sandbox of your choice.

## Stay informed without babysitting

Humans, Captain, scripts, and observability systems all see the same engine through different surfaces.

**For you: the TUI.**

```sh
sgt tui
```

The TUI exposes the estate, workflows, active and historical Work, current stages, attention states, and journal activity.

![Sergeant TUI — fleet view](docs/img/tui-fleet.png)
*(capture from an earlier build; the current cockpit adds Home, Workflows, and Estate views)*

**For Captain: `sgt watch`.**

```sh
sgt --json watch <work-id> --follow
```

`sgt watch` blocks quietly until Work completes, fails, blocks, or needs input — one blocking call instead of a polling loop. Captain can delegate the Work, keep talking with you or go idle, and react when Sergeant has something meaningful to report. Omit the id for an estate-wide watch.

**For automation: CLI, JSON, and API.** Every major CLI surface supports structured output via a global `--json`. The CLI and TUI are both plain clients of the daemon's loopback HTTP/SSE API — neither holds private state.

**For operations: OpenTelemetry.** OTLP/HTTP export is optional and off by default:

```sh
SGT_OTEL=1 SGT_OTLP_ENDPOINT=http://localhost:4318 sgt daemon
```

Sergeant exports the Work → stage → execution → tool hierarchy with execution, wait, journal, failure, and token metrics. Telemetry is a disposable projection; the journal remains the source of truth.

## Bring your environment

Sergeant runs as your user and inherits the environment, tools, credentials, and permissions available to your coding harness. It does not prescribe how your estate builds, tests, deploys, or publishes software, and its Git URLs are opaque and forge-neutral — GitHub, GitLab, Gitea, or a bare remote all look the same to it.

Whatever toolchains your repositories use, Sergeant itself needs only Git (repositories and Work surfaces are its execution language), a supported coding harness to act as Captain, and Docker when workflows include container stages. Rust is required only to build Sergeant from source.

## Current targets

| Surface | Current state |
|---|---|
| Platforms | Linux, macOS, and Windows through WSL2 |
| Harness launchers | `sgt claude`, `sgt codex`, `sgt opencode`, `sgt goose` |
| Primary Captain harnesses | Claude Code and Codex |
| Agent workflow execution | Claude, Codex, and OpenCode (`--backend claude\|codex\|opencode`); stages that stop to ask a question run on Claude or OpenCode's serve transport — Codex is not yet a supported backend for those |
| Deterministic workflow execution | Docker |
| Planned | Goose and Antigravity as full Captain and backend targets |

Sergeant is pre-1.0. Configuration and interfaces may change between releases while the product model settles.

## The CLI is still yours

Captain normally operates Sergeant for you, but nothing is hidden behind an agent-only interface.

```sh
sgt                              # product homepage
sgt doctor                       # installation and estate health, every fault named with a remedy
sgt status                       # daemon health and Work counts
sgt tui                          # interactive estate cockpit
sgt work list                    # active and historical Work
sgt work show <id>               # state, stage, surface, output, recent events
sgt work show <id> --graph       # provenance graph
sgt work transcript <id>         # reconstructed conversation
sgt watch [<id>] [--follow]      # wait for attention or a terminal result
sgt respond <id> "<answer>"      # answer a waiting stage
sgt retry <id>                   # retry failed, blocked, or waiting Work
sgt extend <id> <turns>          # grant more turns to an exhausted envelope, then retry
sgt cancel <id>                  # cancel Work
sgt analytics                    # ask operational questions of your execution history
```

You can also submit directly:

```sh
sgt run "add retry handling to the settlement worker" \
  --workflow implement-change \
  --group payments
```

Agent stages run on the backend you route to (`--backend claude|codex|opencode`, or a routing profile in `sergeant.toml`). Add `--backend fake` to any `sgt run` to try the whole loop deterministically without spending tokens, or run `scripts/demo.sh` from a source checkout for a guided end-to-end walkthrough. `sgt --help` and each subcommand's `--help` are the authority on the current command surface.

## Bounded history, never silent deletion

Each estate declares how much terminal Work history it retains — 1,000 terminal Works by default, with a configurable minimum of 64. Past that policy, Sergeant prunes eligible old Work and blobs automatically: Work-aware, crash-recoverable, visible through the journal, and reported by `sgt doctor`. A request for a pruned Work is answered by name and policy rather than being made indistinguishable from a Work that never existed. And every Work's output branch (`sergeant/<work-id>`) is retained after every terminal outcome — nothing deletes it automatically.

## Where Sergeant came from

The name sergeant-rs is intentional.

[kunchenguid/firstmate](https://github.com/kunchenguid/firstmate) demonstrated that a coding harness can be specialized by a portable collection of instructions, skills, tools, and operating conventions. [callmeradical/sergeant](https://github.com/callmeradical/sergeant) built on that idea and made multi-repository project topology central: give an agent an understanding of the repositories that belong together, then let it coordinate work across them.

Sergeant-rs is a clean-room Rust reimagining of that lineage. It keeps the agent-distro and multi-repository ideas, but changes the execution substrate: the distro is embedded in the released binary and written by `sgt init`; Work is owned by a purpose-built durable runtime; workflows are explicit staged execution packages; state is journaled rather than inferred from terminal sessions; and tmux-based process orchestration is not part of the runtime. The original ideas — and the problems they exposed — shaped many of Sergeant-rs's contracts and regression tests.

## Documentation

- [AGENTS.md](AGENTS.md) — Captain's operating doctrine and routing model
- [Workflow catalog](.sergeant/index.md) — published Sergeant workflows
- [ICM workflow convention](docs/icm/convention.md) — workflow and stage authoring
- [Glossary](docs/glossary.md) — precise definitions of Sergeant concepts
- [NORTH-STAR.md](NORTH-STAR.md) — product direction and governing decisions
- [CHANGELOG.md](CHANGELOG.md) — release history
- [CONTRIBUTING.md](CONTRIBUTING.md) — contributing and development setup
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — architecture invariants, tests, and development rules

## Contributing

Sergeant is written in Rust and dogfoods its own execution model. See [CONTRIBUTING.md](CONTRIBUTING.md) before changing the project. The core local gates are:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

Sergeant is available under the [MIT License](LICENSE).
