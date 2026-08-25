# Sergeant

<p align="center"><img src="assets/logo.png" alt="Sergeant" width="760"></p>
<p align="center"><strong>Define intent with Captain. Execute it with Sergeant.</strong></p>

Sergeant is a local-first AgentOS distro and durable intent-execution engine. Your coding harness becomes **Captain**: it helps define intent, choose repository scope, and select procedure. Sergeant turns accepted intent into durable **Work**, runs explicit stages in isolated Git worktrees, and preserves the evidence and output branches needed to inspect or recover it.

Sergeant supports Linux, Apple Silicon macOS, and Windows through WSL2. It is designed for one developer per installation and is currently pre-1.0.

## Install

```sh
curl -fsSL https://github.com/miztertea/sergeant-rs/releases/latest/download/sergeant-rs-installer.sh | sh
sgt --version
```

You need Git and a supported coding harness. Docker is required only for deterministic execute stages. See [Install Sergeant](docs/getting-started/install.md) for verification, upgrades, and platform details.

## Five-minute start

```sh
mkdir -p ~/estates/my-product
cd ~/estates/my-product
sgt init
sgt claude                 # or: codex, opencode, goose, agy
```

Tell Captain which repositories belong to the product and what outcome you want. Captain verifies the estate, shapes acceptance, names a workflow, and dispatches durable Work. Follow the [Captain-first](docs/getting-started/captain-first.md) or [CLI-first](docs/getting-started/cli-first.md) tutorial.

## The operating model

| Captain | Sergeant |
|---|---|
| conversation, ambiguity, and intent | durable completion |
| estate discovery and scope selection | Work-owned Git surfaces |
| interactive skills | pinned staged workflows |
| user authority and decisions | journal, recovery, and evidence |

An **estate** is the exact directory containing `sergeant.toml`. It can hold one repository or hundreds without combining histories. Each selected repository receives a Work-owned linked worktree and durable `sergeant/<work-id>` branch. A **workflow** is ordered filesystem procedure; each stage is a fresh actor or deterministic Docker execution.

Sergeant journals transitions so Work can outlive terminals, daemon processes, model windows, and restarts. If recovery evidence is insufficient, it blocks with a reason instead of guessing.

## Capabilities

- exact-root, one-to-many estates and reusable repository groups;
- Captain launchers for Claude Code, Codex, OpenCode, Goose, and Antigravity;
- actor execution through Claude, Codex, OpenCode, and Agy, plus fake and Docker execution;
- seven published workflows plus an embedded software-change default;
- retry, response, extension, cancellation, recovery, and retained evidence;
- isolated Git surfaces, integrity reporting, transcripts, and provenance graphs;
- TUI, blocking watch, JSON, loopback HTTP/SSE API, analytics, and OpenTelemetry.

Git surfaces are mutation-attribution boundaries, not OS sandboxes. See [Security and trust](docs/concepts/security-and-trust.md).

## Documentation

- [Getting started](docs/index.md#start-here)
- [Guides](docs/index.md#operate-sergeant)
- [Concepts](docs/index.md#understand-the-product)
- [Workflow catalog](docs/workflows/index.md)
- [Reference](docs/index.md#exact-contracts)

`sgt --help` is the authority for current command grammar. [AGENTS.md](AGENTS.md) is Captain's constitution; [.sergeant/index.md](.sergeant/index.md) is the executable workflow catalog.

## Project

[Releases](https://github.com/miztertea/sergeant-rs/releases) · [Changelog](CHANGELOG.md) · [Contributing](CONTRIBUTING.md) · [Security](SECURITY.md) · [MIT License](LICENSE)

Sergeant-rs is a clean-room Rust reimagining of ideas explored by [kunchenguid/firstmate](https://github.com/kunchenguid/firstmate) and [callmeradical/sergeant](https://github.com/callmeradical/sergeant), rebuilt around durable Work, explicit workflows, a journaled runtime, and local Git surfaces.
