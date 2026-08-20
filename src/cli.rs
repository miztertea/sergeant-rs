//! `sgt` CLI (proposal §31 subset, deviation D1): clap parsing, daemon
//! auto-spawn, and a thin HTTP client over the daemon's v1 API.
//!
//! The CLI never touches runtime state directly — every command except
//! `sgt daemon` goes through the daemon's loopback API. `sgt`'s verbs split
//! into two sets (ADR 0009, D5): the ones that mutate durable state (`run`,
//! `respond`, `retry`, `extend`, `cancel`) spawn a detached daemon
//! (`sgt --data-dir <dir> daemon`) on demand, wait for the runtime
//! descriptor plus a passing `/healthz`, and proceed; the ones that only
//! observe (`status`, `work show`/`list`/`transcript`, `analytics`,
//! `watch`, `tui`, `doctor`, `daemon stop`) never do — observation must not
//! materialize the thing observed, so a cold estate gets a refusal naming
//! `sgt doctor` as the remedy, not a daemon started just to have something
//! to report. Bare `sgt` (no subcommand) is a third thing again: a static
//! homepage that never touches the daemon at all (ADR 0010). `sgt
//! claude`/`codex`/`opencode`/`goose` are a fourth thing: they never reach
//! the daemon either, because they never return to this process at all —
//! they compose an environment and `exec` into the harness (ADR 0006, D2;
//! see `crate::harness`).
//!
//! Stale-descriptor policy (contract): endpoint refuses *and* PID is dead →
//! stale, replace it; PID alive but endpoint unresponsive → ambiguous, fail
//! closed with a diagnostic rather than risk a second daemon. Replacement is
//! the new daemon's atomic descriptor write, never an unlink by the client —
//! a client that removes the path can delete a descriptor a *successor*
//! daemon just published and leave it undiscoverable.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use serde_json::{Value, json};

use crate::api::{ApiClient, ClientError};
use crate::daemon::{self, RuntimeDescriptor};
use crate::domain::estate::{Estate, EstateRoot, RootSource};
use crate::runtime::sweep::SweepTarget;

/// How long the client waits for a spawned daemon to publish a healthy
/// descriptor before giving up.
const SPAWN_WAIT: Duration = Duration::from_secs(10);
/// Poll interval while waiting for the descriptor.
const SPAWN_POLL: Duration = Duration::from_millis(50);
/// Per-request timeout for CLI API calls.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Timeout for a single `/healthz` probe.
const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

/// `sgt` — sergeant-rs command line.
#[derive(Parser, Debug)]
#[command(
    name = "sgt",
    version,
    about = "sergeant-rs: local agent execution surface"
)]
struct Sgt {
    /// Estate root to address, instead of the current directory (C10).
    ///
    /// Names an **exact** root: no search happens from it, and it is
    /// validated by exactly the same rule the current directory is (§4.1).
    /// The CLI is agent-first — an agent should not have to mutate its own
    /// working directory to address an estate.
    #[arg(short = 'C', global = true, value_name = "ESTATE_ROOT")]
    chdir: Option<PathBuf>,
    /// Data directory (default: $SGT_DATA_DIR, then the estate root's
    /// .sergeant/data, then $XDG_DATA_HOME/sergeant, then
    /// ~/.local/share/sergeant).
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    /// Emit machine-readable JSON instead of human text.
    #[arg(long, global = true)]
    json: bool,
    /// Subcommand to run. Omitted, `sgt` prints the homepage (ADR 0010,
    /// deviation from proposal §30 — see `GAUNTLET.md`'s deviation
    /// register): a logo and a condensed quickstart, touching no daemon.
    /// The TUI is `sgt tui`, an explicit verb like any other.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Top-level `sgt` subcommands (§31 subset).
#[derive(Subcommand, Debug)]
enum Command {
    /// Run the daemon in the foreground until SIGINT/SIGTERM, or (with a
    /// subcommand) manage one already running.
    Daemon {
        #[command(subcommand)]
        command: Option<DaemonCommand>,
    },
    /// Daemon health and work counts.
    Status,
    /// Submit new work.
    Run {
        /// The intent to submit.
        intent: String,
        /// Workflow to run (default: the estate's, else `software-change`).
        #[arg(long)]
        workflow: Option<String>,
        /// Backend to run on (§13's explicit tier).
        #[arg(long)]
        backend: Option<String>,
        /// Launch profile (§14).
        #[arg(long)]
        profile: Option<String>,
        /// Targeted repository (repeatable). §7.2: forwarded verbatim as
        /// `scope.repos` — the daemon resolves scope against its own bound
        /// estate; the CLI performs no expansion of its own.
        #[arg(long = "repo")]
        repositories: Vec<String>,
        /// A declared `[group.<name>]`, forwarded verbatim as `scope.group`
        /// (§7.2's core-owned resolution: the daemon expands group
        /// membership against its own bound manifest, not the CLI — this
        /// lets the TUI or a direct API caller submit the identical scope
        /// JSON without reimplementing group semantics). May combine with
        /// `--repo` (union, dedup, decided by the daemon).
        #[arg(long)]
        group: Option<String>,
        /// Explicit whole-estate selection (§7.1's third scope form,
        /// forwarded as `scope.all`) — required to target every repository
        /// in a multi-repository estate; the daemon never infers it. Owner
        /// ruling (2026-08-20): mutually exclusive with `--repo`/`--group` —
        /// combining them is refused here by clap, and again (the
        /// authoritative check, for a direct API caller) by the daemon's own
        /// `Engine::resolve_scope`.
        #[arg(long, conflicts_with_all = ["repositories", "group"])]
        all: bool,
        /// R-MVP1-7's turn envelope, overridden for this one Work
        /// (checkpoint-friction item — the daemon-wide `Engine::turn_cap`/
        /// `SGT_TURN_CAP` default applies otherwise). Journaled with the
        /// submission and enforced by the engine at every turn-spawning
        /// verb, the same as the daemon-wide default.
        #[arg(long)]
        turns: Option<u32>,
        /// R-MVP1-7's per-turn wall-clock ceiling in seconds, overridden for
        /// this one Work (the daemon-wide `Engine::turn_ceiling` default
        /// applies otherwise).
        #[arg(long)]
        ceiling_secs: Option<u64>,
        /// estate-root §8.3's one bounded override of the Git admission
        /// preflight. Waives exactly two normal cautions, and only because an
        /// exact commit can still be pinned in both: a **dirty** mount (the
        /// Work is based on the committed HEAD, the full porcelain evidence
        /// is journaled, and the uncommitted changes are explicitly excluded)
        /// and a **detached** mount (the exact HEAD is pinned and no named
        /// base branch is recorded).
        ///
        /// It never waives an invalid estate, unresolved scope, an unknown
        /// repository, an aliased mount, an unresolvable top level/common
        /// dir/commit, a lock conflict, an existing Work ref or path, a
        /// failed surface construction, or a backend/workflow/profile
        /// failure. "Override may waive policy caution. It may not replace a
        /// missing fact or overcome mechanical impossibility."
        ///
        /// Deliberately a flag on this one verb and nothing else: §8.3 makes
        /// it unavailable from run defaults and run templates, so there is no
        /// config key, no `[estate]` key and no profile field that can supply
        /// it — the operator types it for that submission or it is not set.
        #[arg(long)]
        override_git_preflight: bool,
    },
    /// Inspect work items.
    Work {
        /// What to do with work items.
        #[command(subcommand)]
        command: WorkCommand,
    },
    /// Answer a work item that is waiting for input.
    Respond {
        /// Work id to answer.
        id: String,
        /// The answer.
        input: String,
    },
    /// Retry the current stage of a failed, blocked or waiting work item.
    Retry {
        /// Work id to retry.
        id: String,
    },
    /// R-MVP1-10's exit door for R-MVP1-7's envelope-exhausted `blocked`
    /// landing: raise a work's turn envelope, then `sgt retry` to actually
    /// re-enter the stage (extending alone has no effect on its own — it
    /// only changes what the next retry is checked against). The same door
    /// also reopens a `blocked` landing from a per-turn wall-clock ceiling
    /// interrupt (#90) — `sgt retry` alone is usually enough there, since a
    /// ceiling crossing rarely also exhausts the turn envelope; `extend` only
    /// matters if it happens to have.
    Extend {
        /// Work id whose envelope is being raised.
        id: String,
        /// How many additional turns to allow, on top of whatever this work
        /// already has (cumulative across repeated extensions).
        additional_turns: u32,
    },
    /// Cancel a work item.
    Cancel {
        /// Work id to cancel.
        id: String,
    },
    /// Block until a Work needs attention or reaches a terminal state, then
    /// print the current snapshot and exit — the harness's return path after
    /// `sgt run` (`docs/gauntlet/contracts/WATCH.md`), replacing
    /// `sgt work show` polling with one call. With no `WORK_ID`, watches
    /// future matching transitions across the whole estate instead of one
    /// Work (WATCH-02); either way this is silent until a match — no
    /// heartbeat, no "connected" line. Deliberately does **not** auto-spawn a
    /// daemon (R-WATCH-3, joining `sgt doctor`/`sgt daemon stop`'s own
    /// no-auto-spawn set): observation must not materialize the thing
    /// observed. The global `--json` flag means JSONL here — one
    /// `sergeant.watch/v1` object per line, never a wrapping array.
    Watch {
        /// Work id to watch (omit for an estate-wide watch).
        id: Option<String>,
        /// Remain attached after a nonterminal match (`needs_input`,
        /// `waiting`, `blocked`, `failed`) instead of exiting after the
        /// first notice. A scoped `--follow` still exits once the Work
        /// reaches `completed`/`canceled`; an estate-wide `--follow` stays
        /// attached until interrupted, the stream closes, or an
        /// unrecoverable error occurs. A Work that fails and is never
        /// retried or canceled leaves a scoped `--follow` watcher attached
        /// indefinitely after it has already emitted the `failed` notice
        /// (R-WATCH-8) — a caller that wants to exit on `failed` uses
        /// one-shot mode and re-arms.
        #[arg(long)]
        follow: bool,
    },
    /// Ask the daemon one of the canned §22 analytical questions.
    ///
    /// With no name, list the questions this daemon can answer and the row
    /// counts of the disposable DuckDB projection behind them. Clients never
    /// open that file — they ask the daemon, which is what keeps the
    /// one-owner architecture true (§22).
    Analytics {
        /// Query name (omit to list what is available).
        name: Option<String>,
    },
    /// Open the TUI cockpit (ADR 0010, D6): a client like any other, so it
    /// never auto-spawns a daemon (ADR 0009's no-spawn set) — with none
    /// running it refuses and names `sgt doctor` as the remedy, rather than
    /// materializing one just to have something to render.
    Tui,
    /// Diagnose this installation (§31): tools, data dir, journal, projection,
    /// daemon. Every failing check names the remedy.
    Doctor,
    /// Scaffold an estate at the current directory (MVP-3): `[estate]` in
    /// `sergeant.toml`, `repos/`, `.gitignore` entries for `.sergeant/data`
    /// and `repos/`. Idempotent — a second run on an already-initialized
    /// estate changes nothing. Runs the same `sgt doctor` checks afterward
    /// (no daemon is spawned, matching `sgt doctor`'s own rule).
    Init {
        /// Estate name (default: this directory's own name). Ignored on a
        /// re-run that finds `[estate]` already present — init never renames
        /// an existing estate.
        #[arg(long)]
        name: Option<String>,
    },
    /// Manage the estate's declared repositories (`[[repo]]` in
    /// `sergeant.toml`). A pure manifest edit — no daemon involved.
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    /// Manage the estate's declared groups (`[group.<name>]` in
    /// `sergeant.toml`). A pure manifest edit — no daemon involved.
    Group {
        #[command(subcommand)]
        command: GroupCommand,
    },
    /// Manage the estate's local workflow forks (§6 Phase 3). A pure
    /// filesystem copy — no daemon involved.
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommand,
    },
    /// Launch `claude` bound to this estate (ADR 0006, D2): compose an
    /// environment, bind the estate, then **exec** — replacing this `sgt`
    /// process with `claude`'s, never forking and supervising it. Everything
    /// after `--` passes straight through: `sgt claude -- --model opus`
    /// reaches `claude` as `--model opus`, unexamined. A human-facing
    /// surface like `sgt init`/`sgt doctor`, not something a workflow drives.
    Claude {
        /// Arguments to pass through to `claude`, verbatim, after `--`.
        #[arg(last = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch `codex` bound to this estate — the same passthrough as `sgt
    /// claude` (ADR 0006, D2), for a different harness. Not an adapter: sgt
    /// gives this harness a home without measuring or driving it (ADR 0001,
    /// #25's precedent still gates that).
    Codex {
        /// Arguments to pass through to `codex`, verbatim, after `--`.
        #[arg(last = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch `opencode` bound to this estate — the same passthrough as `sgt
    /// claude` (ADR 0006, D2).
    Opencode {
        /// Arguments to pass through to `opencode`, verbatim, after `--`.
        #[arg(last = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Launch `goose` bound to this estate — the same passthrough as `sgt
    /// claude` (ADR 0006, D2).
    Goose {
        /// Arguments to pass through to `goose`, verbatim, after `--`.
        #[arg(last = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// `sgt daemon ...` subcommands (MVP-3, E4).
#[derive(Subcommand, Debug)]
enum DaemonCommand {
    /// Gracefully stop the daemon on this data dir: pause admission (new
    /// `sgt run` submissions are refused), wait for in-flight work to
    /// finish, then the same SIGTERM shutdown `sgt daemon` (foreground)
    /// already answers to — journals `daemon.stopped`, removes the runtime
    /// descriptor. A clean stop, not a kill: it refuses nothing itself, and
    /// nothing about it destroys work — a Work still draining when the
    /// bounded wait runs out is simply picked back up by ordinary restart
    /// recovery (§25) the next time a daemon starts on this data dir.
    /// Idempotent: stopping a daemon that is not running, or one already
    /// mid-drain from an earlier `sgt daemon stop`, is not an error.
    Stop,
}

/// `sgt repo ...` subcommands (MVP-3).
#[derive(Subcommand, Debug)]
enum RepoCommand {
    /// Declare a repository: clone `--origin` into `repos/<name>` if the
    /// directory does not exist yet, or verify it is already a git
    /// repository if it does, then add `[[repo]]`.
    Add {
        /// Repository name (mounts at `repos/<name>`).
        name: String,
        /// Clone source. Required unless `repos/<name>` already exists.
        #[arg(long)]
        origin: Option<String>,
        /// R-MVP1-4 instruction policy: `local` or `suppress` (default:
        /// unset, which resolves to `suppress`).
        #[arg(long)]
        instructions: Option<String>,
    },
    /// Undeclare a repository. Refuses (naming the group) while any group
    /// still lists it as a member. Never deletes `repos/<name>` from disk.
    Remove {
        /// Repository name to undeclare.
        name: String,
    },
    /// List declared repositories.
    List,
}

/// `sgt group ...` subcommands (MVP-3).
#[derive(Subcommand, Debug)]
enum GroupCommand {
    /// Declare or extend a group. mkdir-p semantics: creating an existing
    /// group unions the given repositories into its membership rather than
    /// erroring; every member must already be a declared repository (fail
    /// closed, naming which one is not and the remedy).
    Add {
        /// Group name.
        name: String,
        /// Member repository names (repeatable positionally).
        repos: Vec<String>,
        /// One orientation line (AI-facing).
        #[arg(long)]
        brief: Option<String>,
    },
    /// Remove a group, or specific members from it. With no repository
    /// arguments, removes the whole group; with one or more, removes just
    /// those members (each must actually belong to the group).
    Remove {
        /// Group name.
        name: String,
        /// Member repository names to drop (omit to remove the whole group).
        repos: Vec<String>,
    },
    /// List declared groups and their members.
    List,
}

/// `sgt workflow ...` subcommands (§6 Phase 3).
#[derive(Subcommand, Debug)]
enum WorkflowCommand {
    /// Copy a stock workflow package (`.sergeant/workflows/<name>/`) into
    /// `.sergeant/local/workflows/<name>/`, where it shadows stock by name
    /// (Ponytail R4/R7 — the one genuinely new verb §6 Phase 3 adds; see
    /// `WorkflowForkError`'s variants for why R2–R6 don't already cover
    /// this). Refuses if a local package of that name already exists —
    /// never silently overwrites a user's fork — or if there is no stock
    /// package of that name to fork from.
    Fork {
        /// Name of the stock workflow package to fork.
        name: String,
    },
}

/// `sgt work ...` subcommands.
#[derive(Subcommand, Debug)]
enum WorkCommand {
    /// List all work items.
    List,
    /// Show one work item.
    Show {
        /// Work id to show.
        id: String,
        /// Render the work's §23 graph neighborhood instead of its record.
        #[arg(long)]
        graph: bool,
    },
    /// Decode a work's conversation from the journal, in causal order
    /// (MVP-3). Read-only: never mutates daemon state. Use the global
    /// `--json` flag for the raw structured turns instead of plain text.
    Transcript {
        /// Work id whose conversation to decode.
        id: String,
    },
    /// #109's inspect verb: every repository binding any terminal work's
    /// teardown left something on disk for — a captured dirty-state patch,
    /// or, in the submodule/capture-failure fallback, the whole worktree
    /// directory — with why and how large. Read-only: never mutates daemon
    /// state, and never destroys anything by itself. `sgt work reap`
    /// disposes of what this names.
    Retained,
    /// #109's dispose verb: permanently discard a work's retained dirty
    /// state (the captured patch, or — the submodule/capture-failure
    /// fallback — the worktree directory itself). Never touches the
    /// retained *branch*: teardown's branch-retention guarantee is
    /// unconditional (§11) and this verb has no path that reaches it.
    /// Refuses without `--yes`: instead of guessing intent, it prints
    /// exactly what `--yes` would destroy (`sgt work retained`'s own view,
    /// scoped to this work) and stops there.
    Reap {
        /// Work id whose retained dirty state to reap.
        id: String,
        /// Actually perform the deletion. Without it, this only previews
        /// what would be destroyed.
        #[arg(long)]
        yes: bool,
    },
    /// §12.3's deliberate sweep (#159): classify every `sergeant/*` branch
    /// in every mount of this estate — `active` (a Work is still on it),
    /// `redundant` (its tip is already contained in the mount's default
    /// branch, so it holds no unique commits), `retained` (a terminal Work's
    /// unique work), `orphan` (a `sergeant/*` ref sergeant never journaled)
    /// — and report each mount's prunable worktree registrations.
    ///
    /// Deliberately not part of `sgt doctor`: this walks every mount's ref
    /// store and asks git about ancestry per branch, where doctor's
    /// git-surfaces row is a cheap count off the journal.
    ///
    /// Reports only, unless `--delete-redundant --yes` is given. Nothing but
    /// a `redundant` branch is ever deletable, and the daemon re-proves that
    /// per branch at deletion time.
    Sweep {
        /// Delete the branches classified `redundant`. Without `--yes` this
        /// only prints exactly what would be deleted.
        #[arg(long)]
        delete_redundant: bool,
        /// Actually perform the deletion (requires `--delete-redundant`).
        #[arg(long, requires = "delete_redundant")]
        yes: bool,
    },
}

/// CLI error: a message for stderr and a nonzero exit.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct CliError(String);

impl CliError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    /// A nonzero exit with nothing to say: the command already printed its
    /// own report (`sgt doctor`), and repeating it on stderr adds noise, not
    /// information.
    fn silent() -> Self {
        Self(String::new())
    }
}

impl From<ClientError> for CliError {
    fn from(e: ClientError) -> Self {
        Self(e.to_string())
    }
}

impl From<crate::tui::TuiError> for CliError {
    fn from(e: crate::tui::TuiError) -> Self {
        Self(e.to_string())
    }
}

impl From<daemon::DaemonError> for CliError {
    fn from(e: daemon::DaemonError) -> Self {
        Self(e.to_string())
    }
}

impl From<crate::domain::estate::EstateError> for CliError {
    fn from(e: crate::domain::estate::EstateError) -> Self {
        Self(e.to_string())
    }
}

impl From<crate::domain::manifest::ManifestError> for CliError {
    fn from(e: crate::domain::manifest::ManifestError) -> Self {
        Self(e.to_string())
    }
}

impl From<crate::domain::workflow::WorkflowForkError> for CliError {
    fn from(e: crate::domain::workflow::WorkflowForkError) -> Self {
        Self(e.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        Self(e.to_string())
    }
}

/// Binary entry point: parse, run, map errors to exit code 1.
pub fn main() -> ExitCode {
    let sgt = Sgt::parse();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("sgt: cannot start async runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match runtime.block_on(dispatch(sgt)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError(message)) if message.is_empty() => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("sgt: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The exact directory this invocation addresses (§4.1, C10): `-C <path>`
/// when given, else the process's own working directory. Never searched
/// from, in either case.
fn effective_root(chdir: Option<&Path>) -> Result<(PathBuf, RootSource), CliError> {
    match chdir {
        Some(path) => Ok((path.to_path_buf(), RootSource::Flag)),
        None => Ok((std::env::current_dir()?, RootSource::Cwd)),
    }
}

/// `SGT_ESTATE_ROOT`, when it names a *valid* estate root strictly above
/// `dir` — §4.4's second diagnostic block, and the only thing this variable
/// is ever consulted for. **It never waives the exact-root check** (§5.3):
/// it can only make a refusal more informative, never turn one into an
/// admission.
fn bound_root_above(dir: &Path) -> Option<PathBuf> {
    let bound = std::env::var_os("SGT_ESTATE_ROOT")?;
    let bound = std::fs::canonicalize(PathBuf::from(bound)).ok()?;
    let here = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if here == bound || !here.starts_with(&bound) {
        return None;
    }
    Estate::admit(&bound).ok().map(|admitted| admitted.path)
}

/// §4.1/§4.3's gate: admit the exact root, or refuse with §4.4's diagnostic.
///
/// Every estate-scoped command calls this **first** — before data-dir
/// resolution, descriptor lookup, spawn, API call, repository inspection, or
/// harness exec — so a directory mistake can never attach to or spawn the
/// wrong daemon.
fn admit_root(dir: &Path, source: RootSource) -> Result<EstateRoot, CliError> {
    Estate::admit(dir).map_err(|e| {
        let e = if source == RootSource::Flag {
            e.via_flag()
        } else {
            e
        };
        let e = match bound_root_above(dir) {
            Some(bound) => e.with_bound_root(bound),
            None => e,
        };
        CliError::new(e.to_string())
    })
}

/// Which rung of [`resolve_data_dir`]'s ladder produced the path — owner
/// ruling #80: `--data-dir` and `SGT_DATA_DIR` outrank a manifest
/// declaration and the estate-local default unconditionally, which in turn
/// outrank the pre-estate platform fallback. Naming free, kept private to
/// this file: nothing outside `resolve_data_dir` and its two immediate
/// callers needs to distinguish these by name today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DataDirSource {
    /// `--data-dir`.
    Flag,
    /// `SGT_DATA_DIR`.
    Env,
    /// The estate manifest's own `[estate] data_dir` (ADR 0008(b)).
    Manifest,
    /// `<estate_root>/.sergeant/data` — no manifest declaration, but `root`
    /// is an estate root.
    EstateDefault,
    /// The pre-estate platform fallback ([`crate::platform::data_dir`]).
    PlatformFallback,
}

impl DataDirSource {
    /// ADR 0008: `$XDG_DATA_HOME` is set in the environment, but the
    /// winning rung was one estate-first resolution reached *before* the
    /// platform fallback tail — the one rung that would actually have
    /// consulted it — was ever tried. `Flag`/`Env` also outrank it, but
    /// deliberately, by design the operator invoked directly; this is
    /// narrower, naming only the case where a declared or estate-local
    /// default silently won a race the operator's environment suggests
    /// they expected `$XDG_DATA_HOME` to run.
    fn xdg_outranked(self) -> bool {
        matches!(self, DataDirSource::Manifest | DataDirSource::EstateDefault)
            && std::env::var_os("XDG_DATA_HOME").is_some()
    }
}

/// Resolve the data dir: `--data-dir` flag, `SGT_DATA_DIR` — both unchanged,
/// unconditional precedence — then (U-R2, MVP-3's estate-resolved default)
/// the estate at `root`, when `root` is one. The default there is the
/// manifest's own `[estate] data_dir` if it declares one (ADR 0008(b)), else
/// `<estate_root>/.sergeant/data` — the same path `sgt init` scaffolds a
/// `.gitignore` entry for ([`crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR`]).
/// Both `data_dir` and the hardcoded default sit *below* the flag and
/// `SGT_DATA_DIR`: ADR 0008(a) upholds estate-first precedence over the
/// pre-estate platform fallback unchanged, and does not touch the flag/env
/// rungs above it — a manifest declaration is consulted only once the estate
/// has already won that race, narrowing what it would otherwise default to,
/// exactly as `surfaces_dir` narrows the daemon's own default rather than
/// outranking an explicit override. Only once `root` is not an estate root
/// (the unscoped commands' case — an estate-scoped one never gets this far,
/// §4.3) does resolution fall through to the pre-estate default, a platform
/// fact ([`crate::platform::data_dir`], #82).
///
/// **Estate-root Phase D:** the "discovered estate" rung is now the *exact*
/// root, never an upward walk. The flag/env rungs above it are untouched —
/// they locate a data dir, and have never substituted for root validation.
///
/// Returns the winning rung alongside the path (#80) so a caller can surface
/// *why* this path won, not just what it is.
fn resolve_data_dir(
    flag: Option<PathBuf>,
    root: &Path,
) -> Result<(PathBuf, DataDirSource), CliError> {
    if let Some(dir) = flag {
        return Ok((dir, DataDirSource::Flag));
    }
    if let Some(dir) = std::env::var_os("SGT_DATA_DIR") {
        return Ok((PathBuf::from(dir), DataDirSource::Env));
    }
    if Estate::is_estate_root(root)? {
        return Ok(match Estate::root_data_dir_override(root)? {
            Some(dir) => (dir, DataDirSource::Manifest),
            None => (
                root.join(crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR),
                DataDirSource::EstateDefault,
            ),
        });
    }
    crate::platform::data_dir::fallback_dir(|name| std::env::var_os(name))
        .map(|dir| (dir, DataDirSource::PlatformFallback))
        .map_err(CliError::new)
}

/// §4.2: which commands require an exact estate root, and which are
/// deliberately usable outside one.
///
/// Estate-scoped: `run`, `status`, `work *`, `respond`/`retry`/`extend`/
/// `cancel`, `watch`, `analytics`, `tui`, `daemon`(+`stop`), `repo *`,
/// `group *`, `workflow *`, and the four harnesses. Unscoped: bare `sgt`,
/// `--help`, `--version`, `init`, `doctor` — nothing else.
fn is_estate_scoped(command: &Command) -> bool {
    !matches!(command, Command::Doctor | Command::Init { .. })
}

async fn dispatch(sgt: Sgt) -> Result<(), CliError> {
    let data_dir_flag = sgt.data_dir.clone();
    let (root, root_source) = effective_root(sgt.chdir.as_deref())?;
    let Some(command) = sgt.command else {
        // ADR 0010 (D6, deviation from proposal §30 registered in
        // `GAUNTLET.md`): bare `sgt` is a homepage, not the TUI, and it
        // contacts no daemon at all — that is what dissolves ADR 0009's
        // hardest case (a human-facing surface with nothing to reconnect
        // to) rather than answering it. `sgt tui` is the explicit verb.
        print_homepage(&root);
        return Ok(());
    };
    // §4.3: root validation precedes data-directory resolution, runtime-
    // descriptor lookup, daemon auto-spawn, API calls, repository
    // inspection, and harness preparation or exec. A directory mistake
    // therefore cannot attach to or spawn the wrong daemon.
    let estate = if is_estate_scoped(&command) {
        Some(admit_root(&root, root_source)?)
    } else {
        None
    };
    let (data_dir, data_dir_source) = resolve_data_dir(
        sgt.data_dir,
        estate.as_ref().map(|e| e.path.as_path()).unwrap_or(&root),
    )?;
    match command {
        Command::Daemon { command: None } => {
            tracing_subscriber::fmt().init();
            daemon::run_until_signal(&data_dir, estate.as_ref().map(|e| e.path.as_path())).await?;
            Ok(())
        }
        Command::Daemon {
            command: Some(DaemonCommand::Stop),
        } => daemon_stop(&data_dir, &estate_root(&estate), sgt.json).await,
        Command::Status => {
            let client = observe_connect(&data_dir, &estate_root(&estate)).await?;
            let system = client.get("/v1/system").await?;
            let works = client.get("/v1/work").await?;
            let mut counts = std::collections::BTreeMap::<String, u64>::new();
            let mut total = 0u64;
            // MVP-3's envelope-visibility item: how much of R-MVP1-7's turn
            // budget each currently-active work has spent, so an operator
            // checking in on a walked-away run does not have to decode the
            // journal (or run `sgt work show` per id) to see whether it is
            // close to its cap.
            let mut active_envelopes: Vec<(String, Value)> = Vec::new();
            if let Some(list) = works["works"].as_array() {
                for work in list {
                    if let Some(state) = work["state"].as_str() {
                        *counts.entry(state.to_string()).or_insert(0) += 1;
                        total += 1;
                        if state == "active" {
                            active_envelopes.push((
                                work["id"].as_str().unwrap_or("?").to_string(),
                                work["envelope"].clone(),
                            ));
                        }
                    }
                }
            }
            let admission_paused = system["admission_paused"].as_bool().unwrap_or(false);
            if sgt.json {
                print_json(&json!({
                    "system": system,
                    "work_total": total,
                    "work_by_state": counts,
                    "active_envelopes": active_envelopes.iter().map(|(id, e)| json!({"id": id, "envelope": e})).collect::<Vec<_>>(),
                }));
            } else {
                println!(
                    "daemon ok — version {} api {} data dir {}",
                    system["version"].as_str().unwrap_or("?"),
                    system["api_revision"].as_str().unwrap_or("?"),
                    system["data_dir"].as_str().unwrap_or("?"),
                );
                if admission_paused {
                    println!(
                        "admission: PAUSED — new work is refused (draining for `sgt daemon stop`)"
                    );
                }
                println!("work: {total} total");
                for (state, n) in counts {
                    println!("  {state}: {n}");
                }
                for (id, envelope) in &active_envelopes {
                    println!(
                        "  {id}: turns {}/{} · ceiling {}s",
                        envelope["turns_spawned"],
                        envelope["turn_cap"],
                        envelope["turn_ceiling_secs"],
                    );
                }
            }
            Ok(())
        }
        Command::Run {
            intent,
            workflow,
            backend,
            profile,
            repositories,
            group,
            all,
            turns,
            ceiling_secs,
            override_git_preflight,
        } => {
            // estate-root proposal §7.2: scope resolution is core-owned. The
            // CLI forwards `--repo`/`--group`/`--all` verbatim as
            // `scope.{repos,group,all}` and does no expansion of its own —
            // the daemon resolves them against its own bound manifest
            // (`runtime::engine::Engine::resolve_scope`), the same function
            // a direct API caller or the TUI submitting the identical scope
            // JSON reaches. Before this, `--group` was pure CLI-side
            // expansion (MVP-3 finding MVP3-C2, R-MVP1-5(b)); §7.2's own
            // rule — "a client surface adds usability, never functionality"
            // — is exactly what retires that.
            let client = ensure_daemon(&data_dir, &estate_root(&estate)).await?;
            let envelope = if turns.is_some() || ceiling_secs.is_some() {
                Some(json!({"turn_cap": turns, "ceiling_secs": ceiling_secs}))
            } else {
                None
            };
            let body = json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "intent": intent,
                "workflow": workflow,
                "backend": backend,
                "profile": profile,
                "scope": {
                    "repos": repositories,
                    "group": group,
                    "all": all,
                },
                "envelope": envelope,
                // §8.3: forwarded verbatim from the flag, with no default
                // layer of any kind between the two — the daemon reads it off
                // this one request field.
                "override_git_preflight": override_git_preflight,
                "created_by": "cli",
                "origin": origin(),
            });
            let result = client.post("/v1/work", &body).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_work_line("submitted", &result);
            }
            Ok(())
        }
        Command::Respond { id, input } => {
            let client = ensure_daemon(&data_dir, &estate_root(&estate)).await?;
            let body = json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "input": input,
            });
            let result = client.post(&format!("/v1/work/{id}/input"), &body).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_work_line("answered", &result);
            }
            Ok(())
        }
        Command::Retry { id } => {
            let client = ensure_daemon(&data_dir, &estate_root(&estate)).await?;
            let body = json!({"command_id": ulid::Ulid::generate().to_string()});
            let result = client.post(&format!("/v1/work/{id}/retry"), &body).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_work_line("retried", &result);
            }
            Ok(())
        }
        Command::Extend {
            id,
            additional_turns,
        } => {
            let client = ensure_daemon(&data_dir, &estate_root(&estate)).await?;
            let body = json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "additional_turns": additional_turns,
            });
            let result = client.post(&format!("/v1/work/{id}/extend"), &body).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_work_line("extended", &result);
            }
            Ok(())
        }
        Command::Work {
            command: WorkCommand::Reap { id, yes },
        } => {
            // #109's dispose verb mutates durable state (it deletes retained
            // artifacts and journals the outcome), so it joins
            // `cancel`/`retry`/`extend`'s bucket rather than the read-only
            // `work list`/`show`/`transcript`/`retained` one. The preview
            // path below never mutates, though, so it connects like
            // `retained` (ADR 0009: observation must not materialize the
            // thing observed) rather than auto-spawning a daemon.
            if !yes {
                let client = observe_connect(&data_dir, &estate_root(&estate)).await?;
                let retained = client.retained().await?;
                let mine: Vec<&Value> = retained["retained"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|r| r["work_id"].as_str() == Some(id.as_str()))
                    .collect();
                if sgt.json {
                    print_json(&json!({"would_reap": mine}));
                } else if mine.is_empty() {
                    println!("nothing retained for {id}");
                } else {
                    println!("--yes would permanently discard:");
                    for r in &mine {
                        println!(
                            "  {}  {}  {}  {}",
                            r["repository"].as_str().unwrap_or("?"),
                            r["disposition"].as_str().unwrap_or("?"),
                            doctor::human_bytes(r["bytes"].as_u64().unwrap_or(0)),
                            r["path"].as_str().unwrap_or("?"),
                        );
                    }
                    println!("re-run with --yes to actually reap this");
                }
                return Ok(());
            }
            let client = ensure_daemon(&data_dir, &estate_root(&estate)).await?;
            let result = client.reap(&id, true).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_reap_report(&result);
            }
            Ok(())
        }
        Command::Work {
            command:
                WorkCommand::Sweep {
                    delete_redundant,
                    yes,
                },
        } => {
            // Classification mutates nothing, so it connects like `retained`
            // (ADR 0009: observation must not materialize the thing
            // observed). Only the confirmed deletion joins `reap`'s bucket
            // and may spawn a daemon.
            let client = if delete_redundant && yes {
                ensure_daemon(&data_dir, &estate_root(&estate)).await?
            } else {
                observe_connect(&data_dir, &estate_root(&estate)).await?
            };
            let report = client.sweep().await?;
            if !delete_redundant {
                if sgt.json {
                    print_json(&report);
                } else {
                    print_sweep(&report);
                }
                return Ok(());
            }
            let redundant = redundant_branches(&report);
            if !yes {
                if sgt.json {
                    let targets: Vec<&SweepTarget> = redundant.iter().map(|(t, _)| t).collect();
                    print_json(&json!({"would_delete": targets}));
                } else if redundant.is_empty() {
                    println!("nothing is classified redundant — nothing to delete");
                } else {
                    println!("--yes would permanently delete:");
                    for (target, tip) in &redundant {
                        println!("  {}  {}  {tip}", target.repository, target.branch);
                    }
                    println!("re-run with --yes to actually delete these");
                }
                return Ok(());
            }
            if redundant.is_empty() {
                if sgt.json {
                    print_json(&json!({"deleted": []}));
                } else {
                    println!("nothing is classified redundant — nothing to delete");
                }
                return Ok(());
            }
            // The daemon re-classifies every one of these before deleting
            // any of it; this list is a request, not a grant.
            let targets: Vec<SweepTarget> = redundant.into_iter().map(|(t, _)| t).collect();
            let result = client.sweep_delete(&targets, true).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_sweep_deletion(&result);
            }
            Ok(())
        }
        Command::Work { command } => {
            let client = observe_connect(&data_dir, &estate_root(&estate)).await?;
            match command {
                WorkCommand::List => {
                    let result = client.get("/v1/work").await?;
                    if sgt.json {
                        print_json(&result);
                    } else {
                        match result["works"].as_array() {
                            Some(list) if !list.is_empty() => {
                                for work in list {
                                    println!(
                                        "{}  {}  {}",
                                        work["id"].as_str().unwrap_or("?"),
                                        list_state_label(work),
                                        work["intent"].as_str().unwrap_or("?"),
                                    );
                                }
                            }
                            _ => println!("no work"),
                        }
                    }
                    Ok(())
                }
                WorkCommand::Show { id, graph: true } => {
                    let result = client.get(&format!("/v1/graph/work/{id}")).await?;
                    if sgt.json {
                        print_json(&result);
                    } else {
                        print_graph(&result);
                    }
                    Ok(())
                }
                WorkCommand::Show { id, graph: false } => {
                    let result = client.get(&format!("/v1/work/{id}")).await?;
                    if sgt.json {
                        print_json(&result);
                    } else {
                        // Human form is one record: the §10 work fields plus
                        // the run coordinates the M3 contract asks show to
                        // include. They stay named keys rather than being
                        // folded into `state` — stage is orthogonal (§10).
                        let mut work = result["work"].clone();
                        if let Some(object) = work.as_object_mut() {
                            for key in [
                                "stage",
                                "surface",
                                "execution",
                                "workflow",
                                "backend",
                                "output",
                                "teardown",
                                // §11 / C5: the integrity disposition and
                                // the §11.3 findings behind it, folded the
                                // same way `teardown` and `output` are —
                                // "was my output where it should be" is
                                // answerable from this one command.
                                "integrity",
                                // MVP-3's envelope-visibility item: turns
                                // spent/capped and the effective ceiling,
                                // folded in the same way every other key
                                // here is (see the comment on `output`
                                // below for why this stays one JSON blob).
                                "envelope",
                            ] {
                                if !result[key].is_null() {
                                    object.insert(key.to_string(), result[key].clone());
                                }
                            }
                        }
                        // R-MVP1-2's output pointer (source repo, retained
                        // branch, worktree path, finalize commit) already
                        // rides along inside this same object at `output`
                        // (folded in above) — "where did my deliverable
                        // land" is answerable from this one `work show`
                        // without a second command, and every other key
                        // here folds the identical way, so it stays inside
                        // the one JSON blob rather than breaking it up with
                        // separate prose the way a caller that treats this
                        // "human form" as parseable JSON (several tests do)
                        // would choke on.
                        print_json(&work);
                    }
                    Ok(())
                }
                WorkCommand::Transcript { id } => {
                    let result = client.work_transcript(&id).await?;
                    if sgt.json {
                        print_json(&result);
                    } else {
                        print_transcript(&result);
                    }
                    Ok(())
                }
                WorkCommand::Retained => {
                    let result = client.retained().await?;
                    if sgt.json {
                        print_json(&result);
                    } else {
                        print_retained(&result);
                    }
                    Ok(())
                }
                // Both are handled by their own dedicated arms above —
                // never reachable here.
                WorkCommand::Reap { .. } => unreachable!("reap is matched before this arm"),
                WorkCommand::Sweep { .. } => unreachable!("sweep is matched before this arm"),
            }
        }
        Command::Analytics { name } => {
            let client = observe_connect(&data_dir, &estate_root(&estate)).await?;
            let result = match &name {
                Some(name) => client.get(&format!("/v1/analytics/{name}")).await?,
                None => client.get("/v1/analytics").await?,
            };
            if sgt.json {
                print_json(&result);
            } else if name.is_some() {
                print_table(&result);
            } else {
                print_analytics_index(&result);
            }
            Ok(())
        }
        Command::Cancel { id } => {
            let client = ensure_daemon(&data_dir, &estate_root(&estate)).await?;
            let body = json!({"command_id": ulid::Ulid::generate().to_string()});
            let result = client.post(&format!("/v1/work/{id}/cancel"), &body).await?;
            if sgt.json {
                print_json(&result);
            } else {
                println!(
                    "canceled {} ({})",
                    result["work"]["id"].as_str().unwrap_or("?"),
                    result["work"]["state"].as_str().unwrap_or("?"),
                );
            }
            Ok(())
        }
        Command::Watch { id, follow } => {
            // R-WATCH-3 / ADR 0009: never `ensure_daemon` — this verb
            // refuses rather than spawns.
            let client = observe_connect(&data_dir, &estate_root(&estate)).await?;
            let options = crate::watch::WatchOptions {
                work_id: id,
                follow,
            };
            let json = sgt.json;
            crate::watch::watch(&client, &options, |notice| {
                if json {
                    // §9.1: JSONL — one compact object per line, no wrapping
                    // array, no keep-alive/heartbeat lines (§10).
                    println!(
                        "{}",
                        serde_json::to_string(notice).expect("watch notice serializes")
                    );
                } else {
                    println!("{}", crate::watch::render_human(notice));
                }
            })
            .await
            .map(|_exit| ())
            .map_err(|e| CliError::new(e.to_string()))
        }
        Command::Tui => {
            // ADR 0009/0010: the TUI never auto-spawns — it is in the
            // no-spawn set like every other observation surface, and bare
            // `sgt` no longer falls into it by default.
            let client = observe_connect(&data_dir, &estate_root(&estate)).await?;
            crate::tui::run(client).await.map_err(CliError::from)
        }
        Command::Doctor => {
            let report = doctor::run(&data_dir, &root, data_dir_source.xdg_outranked()).await;
            if sgt.json {
                print_json(&report.to_json());
            } else {
                report.print();
            }
            if report.healthy() {
                Ok(())
            } else {
                // The report has already said what is wrong and what to do
                // about it on stdout; a second summary on stderr would be
                // noise. The nonzero exit is the machine-readable half.
                Err(CliError::silent())
            }
        }
        Command::Init { name } => {
            let cwd = root.clone();
            let outcome = crate::domain::manifest::init_estate(&cwd, name.as_deref())?;
            // ADR 0014 decision 1: the distro (AGENTS.md, skills/,
            // .sergeant/common/contexts/, .sergeant/workflows/) is embedded
            // in the binary and written here, not cloned. Per-file
            // idempotent (`domain::distro::write_distro`), so this never
            // turns a second `sgt init` on an already-initialized estate
            // into anything but a no-op, and it writes only within `cwd`.
            let distro_outcome = crate::domain::distro::write_distro(&cwd)?;
            // Re-resolve now that `sergeant.toml` exists: the `data_dir`
            // computed above ran before `init_estate` created it, so on a
            // fresh estate estate discovery had nothing to find yet and
            // this report would otherwise self-check the pre-estate
            // fallback and call it `[ok]` (#164).
            let (data_dir, data_dir_source) = resolve_data_dir(data_dir_flag, &cwd)?;
            let report = doctor::run(&data_dir, &cwd, data_dir_source.xdg_outranked()).await;
            if sgt.json {
                print_json(&json!({
                    "outcome": {
                        "manifest_created": outcome.manifest_created,
                        "estate_section_added": outcome.estate_section_added,
                        "repos_dir_created": outcome.repos_dir_created,
                        "gitignore_updated": outcome.gitignore_updated,
                        "distro_files_written": distro_outcome.written.len(),
                        "distro_files_already_present": distro_outcome.skipped.len(),
                        "changed": outcome.changed() || distro_outcome.changed(),
                    },
                    "doctor": report.to_json(),
                }));
            } else {
                if outcome.changed() || distro_outcome.changed() {
                    println!("initialized estate at {}", cwd.display());
                    if outcome.manifest_created {
                        println!("  created sergeant.toml");
                    }
                    if outcome.estate_section_added && !outcome.manifest_created {
                        println!("  added [estate] to the existing sergeant.toml");
                    }
                    if outcome.repos_dir_created {
                        println!("  created repos/");
                    }
                    if outcome.gitignore_updated {
                        println!("  updated .gitignore");
                    }
                    if distro_outcome.changed() {
                        println!(
                            "  wrote {} distro file(s) (AGENTS.md, skills/, \
                             .sergeant/common/contexts/, .sergeant/workflows/)",
                            distro_outcome.written.len()
                        );
                    }
                } else {
                    println!(
                        "estate at {} is already initialized — nothing to do",
                        cwd.display()
                    );
                }
                println!();
                report.print();
            }
            if report.healthy_for_init() {
                Ok(())
            } else {
                Err(CliError::silent())
            }
        }
        Command::Repo { command } => repo_command(sgt.json, &estate_root(&estate), command).await,
        Command::Group { command } => group_command(sgt.json, &estate_root(&estate), command).await,
        Command::Workflow { command } => {
            workflow_command(sgt.json, &estate_root(&estate), command).await
        }
        Command::Claude { args } => exec_harness("claude", &args, &estate_root(&estate), &data_dir),
        Command::Codex { args } => exec_harness("codex", &args, &estate_root(&estate), &data_dir),
        Command::Opencode { args } => {
            exec_harness("opencode", &args, &estate_root(&estate), &data_dir)
        }
        Command::Goose { args } => exec_harness("goose", &args, &estate_root(&estate), &data_dir),
    }
}

/// `sgt <harness>`'s dispatch (ADR 0006, D2, §5.3): the exact root has
/// already been admitted by [`dispatch`] before this runs (§4.3 — validation
/// precedes harness preparation *and* exec), so this composes the
/// environment, binds `estate_root`/`data_dir`, and execs.
/// [`crate::harness::exec`] only ever returns on failure; a successful exec
/// replaces this process and never returns here at all, which is the whole
/// point of the boundary (module docs).
fn exec_harness(
    binary: &str,
    args: &[String],
    estate_root: &Path,
    data_dir: &Path,
) -> Result<(), CliError> {
    let command = crate::harness::prepare(binary, args, estate_root, data_dir);
    let err = crate::harness::exec(command);
    Err(CliError::new(format!("cannot exec `{binary}`: {err}")))
}

/// The admitted root, for the estate-scoped arms of [`dispatch`].
///
/// Infallible by construction: [`dispatch`] admits the root for every
/// command [`is_estate_scoped`] answers `true` for, before any of these arms
/// can run — §4.3's ordering, expressed in the types rather than re-checked.
fn estate_root(estate: &Option<EstateRoot>) -> PathBuf {
    estate
        .as_ref()
        .expect("dispatch admits the root for every estate-scoped command")
        .path
        .clone()
}

/// `sgt repo add/remove/list` (MVP-3): a pure manifest edit/read, no daemon
/// involved — repository declarations are estate topology (§9), not runtime
/// state.
async fn repo_command(
    json: bool,
    estate_root: &Path,
    command: RepoCommand,
) -> Result<(), CliError> {
    match command {
        RepoCommand::Add {
            name,
            origin,
            instructions,
        } => {
            let policy = match instructions.as_deref() {
                None => None,
                Some("local") => Some(crate::domain::estate::InstructionPolicy::Local),
                Some("suppress") => Some(crate::domain::estate::InstructionPolicy::Suppress),
                Some(other) => {
                    return Err(CliError::new(format!(
                        "--instructions {other:?} is not recognized (use \"local\" or \"suppress\")"
                    )));
                }
            };
            crate::domain::manifest::add_repo(estate_root, &name, origin.as_deref(), policy)?;
            if json {
                print_json(&json!({"added": name}));
            } else {
                println!(
                    "added repo {name} at {}",
                    crate::domain::estate::mount_path(estate_root, &name).display()
                );
            }
            Ok(())
        }
        RepoCommand::Remove { name } => {
            crate::domain::manifest::remove_repo(estate_root, &name)?;
            if json {
                print_json(&json!({"removed": name}));
            } else {
                println!("removed repo {name}");
            }
            Ok(())
        }
        RepoCommand::List => {
            let estate = crate::domain::estate::Estate::from_config_allow_empty(
                &estate_root.join(crate::domain::estate::MANIFEST_FILE),
            )?;
            if json {
                print_json(&json!({
                    "repositories": estate.repositories.iter().map(|r| json!({
                        "name": r.name,
                        "path": r.path,
                        "instructions": estate.instruction_policy(&r.name).as_str(),
                        "origin": estate.repository_origin(&r.name),
                    })).collect::<Vec<_>>(),
                }));
            } else if estate.repositories.is_empty() {
                println!("no repositories declared");
            } else {
                for r in &estate.repositories {
                    println!(
                        "{}  {}  instructions={}  origin={}",
                        r.name,
                        r.path.display(),
                        estate.instruction_policy(&r.name),
                        estate.repository_origin(&r.name).unwrap_or("-"),
                    );
                }
            }
            Ok(())
        }
    }
}

/// `sgt group add/remove/list` (MVP-3): a pure manifest edit/read, same
/// rationale as [`repo_command`].
async fn group_command(
    json: bool,
    estate_root: &Path,
    command: GroupCommand,
) -> Result<(), CliError> {
    match command {
        GroupCommand::Add { name, repos, brief } => {
            crate::domain::manifest::add_group(estate_root, &name, &repos, brief.as_deref())?;
            if json {
                print_json(&json!({"group": name}));
            } else {
                println!("group {name} updated");
            }
            Ok(())
        }
        GroupCommand::Remove { name, repos } => {
            crate::domain::manifest::remove_group(estate_root, &name, &repos)?;
            if json {
                print_json(&json!({"group": name}));
            } else if repos.is_empty() {
                println!("removed group {name}");
            } else {
                println!("removed {} from group {name}", repos.join(", "));
            }
            Ok(())
        }
        GroupCommand::List => {
            let estate = crate::domain::estate::Estate::from_config_allow_empty(
                &estate_root.join(crate::domain::estate::MANIFEST_FILE),
            )?;
            if json {
                print_json(&json!({
                    "groups": estate.groups.iter().map(|(name, g)| json!({
                        "name": name,
                        "repos": g.repos,
                        "brief": g.brief,
                    })).collect::<Vec<_>>(),
                }));
            } else if estate.groups.is_empty() {
                println!("no groups declared");
            } else {
                for (name, g) in &estate.groups {
                    let brief = g
                        .brief
                        .as_deref()
                        .map(|b| format!("  {b}"))
                        .unwrap_or_default();
                    println!("{}  [{}]{}", name, g.repos.join(", "), brief);
                }
            }
            Ok(())
        }
    }
}

/// `sgt workflow fork` (§6 Phase 3): a pure filesystem copy under the
/// estate root, same rationale as [`repo_command`]/[`group_command`] — no
/// daemon involved.
async fn workflow_command(
    json: bool,
    estate_root: &Path,
    command: WorkflowCommand,
) -> Result<(), CliError> {
    match command {
        WorkflowCommand::Fork { name } => {
            let forked = crate::domain::workflow::fork(estate_root, &name)?;
            if json {
                print_json(&json!({"forked": name, "path": forked.display().to_string()}));
            } else {
                println!("forked workflow {name} to {}", forked.display());
            }
            Ok(())
        }
    }
}

/// One line per edge: the relation, its endpoints, and — the point of §23 —
/// the journal seq that justifies it, so a reader can go and check.
fn print_graph(result: &Value) {
    let labels: std::collections::BTreeMap<&str, &str> = result["nodes"]
        .as_array()
        .map(|nodes| {
            nodes
                .iter()
                .filter_map(|n| Some((n["node_id"].as_str()?, n["label"].as_str().unwrap_or(""))))
                .collect()
        })
        .unwrap_or_default();
    let empty = Vec::new();
    let edges = result["edges"].as_array().unwrap_or(&empty);
    println!(
        "{} — {} nodes, {} edges",
        result["work_id"].as_str().unwrap_or("?"),
        result["nodes"].as_array().map_or(0, Vec::len),
        edges.len(),
    );
    for edge in edges {
        let from = edge["from_node"].as_str().unwrap_or("?");
        let to = edge["to_node"].as_str().unwrap_or("?");
        println!(
            "  seq {:>5}  {} --{}--> {}   ({} → {})",
            edge["source_seq"],
            from,
            edge["relation"].as_str().unwrap_or("?"),
            to,
            labels.get(from).copied().unwrap_or(""),
            labels.get(to).copied().unwrap_or(""),
        );
    }
}

/// `sgt work transcript`'s plain-text rendering: one block per turn, in the
/// causal order `GET /v1/work/{id}/transcript` already returns them in. A
/// turn recovered from the raw archive of an interrupted turn (rather than
/// its own journaled event) is called out, since it is lower-fidelity than
/// the rest — never presented as if it were an ordinary completed turn.
fn print_transcript(result: &Value) {
    print!("{}", render_transcript(result));
}

/// #96: the journal timestamps every conversation event
/// (`transcript_turns`'s `"ts"` field, read straight off `event.timestamp`);
/// the human rendering used to drop it, so an operator reading assistant text
/// with no wall-clock anchor mistook "recent-looking" for "recent" and missed
/// a turn that had already been ceiling-interrupted two minutes earlier (#90).
///
/// Prints the RFC3339 timestamp the journal already carries, verbatim — no
/// elapsed-time arithmetic against "now": that would be *deriving* a fact
/// this surface has no business computing, not reading one the journal
/// already recorded. `--json` (`transcript_turns`) already carries the same
/// field per entry, so only this rendering needed the fix.
///
/// Factored out from `print_transcript` (matching `watch::render_human`'s
/// split) so the rendering is a pure function a test can pin without a
/// daemon.
fn render_transcript(result: &Value) -> String {
    let empty = Vec::new();
    let turns = result["turns"].as_array().unwrap_or(&empty);
    if turns.is_empty() {
        return "no conversation recorded for this work\n".to_string();
    }
    let mut out = String::new();
    for turn in turns {
        let label = match turn["role"].as_str().unwrap_or("?") {
            "user" => "User",
            "assistant" => "Assistant",
            "ask" => "Ask",
            other => other,
        };
        let recovered = if turn["source"].as_str() == Some("blob_decode") {
            " (recovered from the interrupted turn's raw archive)"
        } else {
            ""
        };
        let ts = turn["ts"].as_str().unwrap_or("?");
        out.push_str(&format!("[{ts}] {label}{recovered}:\n"));
        out.push_str(turn["text"].as_str().unwrap_or(""));
        out.push_str("\n\n");
    }
    out
}

/// The `state` column of `sgt work list`, with §11.5's integrity axis folded
/// in — amendment C5's acceptance criterion, that a terminal-dirty Work is
/// distinguishable in *default* output and not only in `--json`.
///
/// A dirty completion already reads `completed_dirty`: the API's own
/// `reported_state` is where that compact label is minted, and re-appending
/// anything to it here would say the same thing twice. `failed` and
/// `canceled` keep their true state strings (§11.5 adds no label for them and
/// no transition target), so the axis is appended to the column instead —
/// `failed/dirty` — which is exactly how §11.5 writes the orthogonal pair.
fn list_state_label(work: &Value) -> String {
    let state = work["state"].as_str().unwrap_or("?");
    let dirty = work["integrity"]["disposition"].as_str() == Some("dirty");
    if dirty && !state.ends_with("_dirty") {
        format!("{state}/dirty")
    } else {
        state.to_string()
    }
}

/// #109's inspect verb (`GET /v1/retained`, `sgt work retained`): every
/// repository binding some terminal work's teardown left something on disk
/// for, with a running total so "how much disk is this actually costing me"
/// is answerable at a glance — the question #109 itself was filed over.
fn print_retained(result: &Value) {
    let empty = Vec::new();
    let entries = result["retained"].as_array().unwrap_or(&empty);
    if entries.is_empty() {
        println!("nothing retained");
        return;
    }
    let mut total = 0u64;
    for entry in entries {
        let bytes = entry["bytes"].as_u64().unwrap_or(0);
        total += bytes;
        println!(
            "{}  {}  {}  {}  {}",
            entry["work_id"].as_str().unwrap_or("?"),
            entry["repository"].as_str().unwrap_or("?"),
            entry["disposition"].as_str().unwrap_or("?"),
            doctor::human_bytes(bytes),
            entry["path"].as_str().unwrap_or("?"),
        );
    }
    println!("total: {}", doctor::human_bytes(total));
}

/// #109's dispose verb's own result (`POST /v1/work/{id}/reap`, `sgt work
/// reap --yes`): what actually happened to each binding, not just that the
/// call succeeded.
fn print_reap_report(result: &Value) {
    let empty = Vec::new();
    let bindings = result["report"]["bindings"].as_array().unwrap_or(&empty);
    if bindings.is_empty() {
        println!("nothing to reap");
        return;
    }
    for binding in bindings {
        let outcome = &binding["outcome"];
        let repository = binding["repository"].as_str().unwrap_or("?");
        match outcome["outcome"].as_str() {
            Some("reaped") => println!(
                "{repository}: reaped {}",
                doctor::human_bytes(outcome["bytes"].as_u64().unwrap_or(0)),
            ),
            Some("nothing_retained") => println!("{repository}: nothing retained"),
            Some("skipped") => println!(
                "{repository}: skipped — {}",
                outcome["reason"].as_str().unwrap_or("?"),
            ),
            Some("failed") => println!(
                "{repository}: failed — {}",
                outcome["detail"].as_str().unwrap_or("?"),
            ),
            _ => println!("{repository}: {outcome}"),
        }
    }
}

/// §12.3's sweep report (`GET /v1/sweep`, `sgt work sweep`): every mount's
/// `sergeant/*` refs with what each one is, the default branch every
/// `redundant` verdict is relative to, and the mount's prunable worktree
/// registrations with git's own remedy.
fn print_sweep(report: &Value) {
    let empty = Vec::new();
    let repositories = report["repositories"].as_array().unwrap_or(&empty);
    if repositories.is_empty() {
        println!("no repositories declared — nothing to sweep");
        return;
    }
    let mut redundant = 0usize;
    for repository in repositories {
        let name = repository["repository"].as_str().unwrap_or("?");
        match repository["default_branch"].as_str() {
            Some(default) => println!("{name} (default branch: {default})"),
            None => println!("{name} (HEAD is detached — redundancy cannot be proven here)"),
        }
        if let Some(unreadable) = repository["unreadable"].as_str() {
            println!("  cannot read this mount: {unreadable}");
            continue;
        }
        let branches = repository["branches"].as_array().unwrap_or(&empty);
        if branches.is_empty() {
            println!("  no sergeant/* branches");
        }
        for branch in branches {
            let classification = branch["classification"].as_str().unwrap_or("?");
            if classification == "redundant" {
                redundant += 1;
            }
            println!(
                "  {:<9} {}  {}  {}",
                classification,
                branch["branch"].as_str().unwrap_or("?"),
                short_sha(branch["tip"].as_str().unwrap_or("?")),
                branch["detail"].as_str().unwrap_or(""),
            );
        }
        let prunable = repository["prunable_worktrees"].as_u64().unwrap_or(0);
        if prunable > 0 {
            println!(
                "  {prunable} prunable worktree registration(s) — remedy: `git -C {} worktree \
                 prune`",
                repository["path"].as_str().unwrap_or("?"),
            );
        }
    }
    if let Some(note) = report["note"].as_str() {
        println!("note: {note}");
    }
    if redundant > 0 {
        println!(
            "{redundant} branch(es) classified redundant; preview the deletion with `sgt work \
             sweep --delete-redundant`"
        );
    }
}

/// The sweep's deletion result (`POST /v1/sweep`, `sgt work sweep
/// --delete-redundant --yes`): what actually happened to each requested
/// branch, including every one the daemon's own re-classification refused.
fn print_sweep_deletion(result: &Value) {
    let empty = Vec::new();
    let deleted = result["deleted"].as_array().unwrap_or(&empty);
    if deleted.is_empty() {
        println!("nothing was deleted");
        return;
    }
    for entry in deleted {
        let where_ = format!(
            "{}  {}",
            entry["repository"].as_str().unwrap_or("?"),
            entry["branch"].as_str().unwrap_or("?"),
        );
        match entry["outcome"].as_str() {
            Some("deleted") => println!(
                "{where_}: deleted at {}",
                short_sha(entry["tip"].as_str().unwrap_or("?")),
            ),
            Some("refused") => println!(
                "{where_}: refused — {}",
                entry["reason"].as_str().unwrap_or("?"),
            ),
            Some("failed") => println!(
                "{where_}: failed — {}",
                entry["detail"].as_str().unwrap_or("?"),
            ),
            _ => println!("{where_}: {}", entry["outcome"]),
        }
    }
}

/// Every branch the daemon's classification pass called `redundant`, paired
/// with the tip it currently points at — what the unconfirmed preview prints
/// and what `--yes` submits.
fn redundant_branches(report: &Value) -> Vec<(SweepTarget, String)> {
    let empty = Vec::new();
    report["repositories"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .flat_map(|repository| {
            let name = repository["repository"].as_str().unwrap_or("?").to_string();
            repository["branches"]
                .as_array()
                .unwrap_or(&empty)
                .iter()
                .filter(|branch| branch["classification"].as_str() == Some("redundant"))
                .map(|branch| {
                    (
                        SweepTarget {
                            repository: name.clone(),
                            branch: branch["branch"].as_str().unwrap_or("?").to_string(),
                        },
                        branch["tip"].as_str().unwrap_or("?").to_string(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// A commit for a report column: git's own abbreviation length, with a
/// short-or-missing value passed through as-is rather than sliced blindly.
fn short_sha(sha: &str) -> &str {
    sha.get(..7).unwrap_or(sha)
}

/// A canned query's answer as a plain aligned table (M6 owns presentation).
fn print_table(result: &Value) {
    let empty = Vec::new();
    let columns = result["columns"].as_array().unwrap_or(&empty);
    if let Some(question) = result["question"].as_str() {
        println!("{question}");
    }
    let header: Vec<String> = columns
        .iter()
        .map(|c| c.as_str().unwrap_or("?").to_string())
        .collect();
    println!("{}", header.join("\t"));
    for row in result["rows"].as_array().unwrap_or(&empty) {
        let cells: Vec<String> = row
            .as_array()
            .unwrap_or(&empty)
            .iter()
            .map(cell_text)
            .collect();
        println!("{}", cells.join("\t"));
    }
}

/// The questions this daemon answers, and how populated the projection is.
fn print_analytics_index(result: &Value) {
    let empty = Vec::new();
    println!("queries:");
    for query in result["queries"].as_array().unwrap_or(&empty) {
        println!(
            "  {:<26} {}",
            query["name"].as_str().unwrap_or("?"),
            query["question"].as_str().unwrap_or(""),
        );
    }
    println!("projection (rebuilt from the journal, disposable):");
    for table in result["tables"].as_array().unwrap_or(&empty) {
        println!(
            "  {:<26} {} rows",
            table["table"].as_str().unwrap_or("?"),
            table["rows"],
        );
    }
}

fn cell_text(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(_) => println!("{value}"),
    }
}

/// One human line about a work: id, §10 state, and — because the stage is a
/// separate coordinate, not a state — the stage it is on when there is one.
fn print_work_line(verb: &str, result: &Value) {
    let stage = result["stage"]["stage_id"]
        .as_str()
        .map(|id| format!(" [{id}]"))
        .unwrap_or_default();
    println!(
        "{verb} {} ({}){stage}",
        result["work"]["id"].as_str().unwrap_or("?"),
        result["work"]["state"].as_str().unwrap_or("?"),
    );
}

/// §13's origin metadata for this invocation: which front end is asking, and
/// the directory estate discovery should start from. The client owns the
/// cwd — the daemon has none — so discovery input has to travel with the
/// request. `SGT_ORIGIN_CLIENT` lets a front-end harness declare itself; a
/// bare terminal is just `cli`, which names no backend and therefore falls
/// through origin affinity to the configured default (§13's own table).
fn origin() -> Value {
    let client = std::env::var("SGT_ORIGIN_CLIENT").unwrap_or_else(|_| "cli".to_string());
    json!({
        "client": client,
        "cwd": std::env::current_dir().ok(),
    })
}

/// ASCII-art wordmark for the bare-`sgt` homepage (ADR 0010, D6).
const LOGO: &str = r"
   ___ ___ ___ __  __
  / __/ __| __|  \/  |___
  \__ \__ \ _|| |\/| / -_)   local agent execution surface
  |___/___/___|_|  |_\___|
";

/// Bare `sgt` (no subcommand): a homepage, not the TUI (ADR 0010, D6 —
/// deviation from proposal §30, registered in `GAUNTLET.md`'s deviation
/// register).
///
/// Touches no daemon at all, which is the point: this is what dissolves ADR
/// 0009's TUI carve-out debate instead of answering it — there is no daemon
/// question to settle for a command that never asks the daemon anything.
/// Reading `sergeant.toml` is not observing the daemon (ADR 0010's own
/// framing), so this *is* estate-aware, resolving the ADR's open question in
/// that direction: outside an estate it points at `sgt init`; inside one it
/// names the estate and its declared repository count, the same client-side
/// manifest read every other estate-aware verb in this binary does.
fn print_homepage(root: &Path) {
    println!("{}", LOGO.trim_end_matches('\n'));
    println!();
    // §4.1: the exact directory, never a parent. Bare `sgt` is unscoped, so
    // "this is not an estate root" is information here, not a refusal.
    match Estate::admit(root) {
        Ok(admitted) => {
            match Estate::from_config_allow_empty(&admitted.manifest_path) {
                Ok(estate) => println!(
                    "estate {:?} at {} — {} repositories declared",
                    estate.name,
                    admitted.path.display(),
                    estate.repositories.len(),
                ),
                Err(e) => println!(
                    "estate at {} could not be read: {e}",
                    admitted.path.display()
                ),
            }
            println!();
            println!("  sgt doctor              diagnose this installation");
            println!("  sgt status              daemon health and work counts");
            println!("  sgt run \"<intent>\"      submit work");
            println!("  sgt tui                 open the cockpit");
        }
        Err(_) => {
            println!("{} is not an estate root", root.display());
            println!("(sergeant does not search parent directories for one)");
            println!();
            println!("  sgt init                scaffold one here");
            println!("  sgt -C <estate-root>    address an estate elsewhere");
            println!("  sgt doctor              diagnose this installation");
        }
    }
    println!();
    println!("sgt <command> --help for the full verb list");
}

/// Connect to the daemon for `data_dir`, auto-spawning one if needed.
///
/// This is the CLI's half of the client contract — descriptor discovery,
/// staleness judgement, detached spawn — and it hands back the crate's one
/// API client ([`ApiClient`], defined next to the router it speaks to). Only
/// the mutating verbs (ADR 0009) come through here; every observation verb,
/// TUI included, goes through [`observe_connect`] instead.
async fn ensure_daemon(data_dir: &Path, estate_root: &Path) -> Result<ApiClient, CliError> {
    let http = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let stale = if let Some(descriptor) = daemon::read_descriptor(data_dir)? {
        // §5.1: verify the binding **before** the endpoint is used for
        // anything, and before any decision that could spawn. A daemon bound
        // to another estate is a named refusal — never a connection, and
        // never a second daemon over the same data dir.
        check_binding(&descriptor, data_dir, estate_root)?;
        if healthz_ok(&http, &descriptor.endpoint).await {
            return client_for(&descriptor);
        }
        if daemon::pid_alive(descriptor.pid) {
            // Ambiguous: something with that PID is alive but the endpoint
            // does not answer. Spawning a second daemon here could race a
            // slow-but-live one; fail closed instead.
            return Err(CliError::new(format!(
                "daemon descriptor at {} names PID {} (alive) but {} does not answer /healthz; \
                 refusing to spawn a second daemon. If the daemon is truly gone, remove the \
                 descriptor file and retry.",
                daemon::descriptor_path(data_dir).display(),
                descriptor.pid,
                descriptor.endpoint,
            )));
        }
        // Stale: endpoint dead and PID dead. Replacement is the spawned
        // daemon's atomic descriptor write below, *not* an unlink here:
        // between judging the file stale and removing it, a daemon another
        // client just spawned can publish its fresh descriptor at the same
        // path, and unlinking that would leave a healthy, lock-holding daemon
        // permanently undiscoverable (it only writes the descriptor at
        // startup). Leaving the stale bytes in place costs nothing — the wait
        // loop below never accepts a descriptor that does not answer
        // `/healthz`.
        Some(descriptor)
    } else {
        None
    };

    spawn_daemon(data_dir, estate_root)?;

    // Wait for a healthy descriptor. It may be written by our child or by a
    // concurrently racing client's child — either is fine; the daemon lock
    // guarantees at most one winner. The descriptor already judged stale is
    // skipped by identity rather than re-probed, so a stale endpoint that
    // hangs instead of refusing cannot eat the wait budget one health timeout
    // at a time.
    let deadline = Instant::now() + SPAWN_WAIT;
    while Instant::now() < deadline {
        if let Ok(Some(descriptor)) = daemon::read_descriptor(data_dir)
            && !is_stale_descriptor(stale.as_ref(), &descriptor)
        {
            // The winner of the spawn race may be another client's child,
            // and §5.1 applies to it exactly as it did above: a daemon bound
            // elsewhere is refused, not adopted.
            check_binding(&descriptor, data_dir, estate_root)?;
            if healthz_ok(&http, &descriptor.endpoint).await {
                return client_for(&descriptor);
            }
        }
        tokio::time::sleep(SPAWN_POLL).await;
    }
    Err(CliError::new(format!(
        "spawned a daemon for {} but it did not become healthy within {:?} \
         (see daemon.log in the data dir)",
        data_dir.display(),
        SPAWN_WAIT,
    )))
}

fn client_for(descriptor: &RuntimeDescriptor) -> Result<ApiClient, CliError> {
    ApiClient::new(&descriptor.endpoint, &descriptor.token).map_err(CliError::from)
}

/// §5.1/§15: refuse a descriptor bound to a different estate, naming both
/// roots. Runs before the endpoint is probed, connected to, or used to
/// decide whether to spawn — "never connect, never a second daemon on the
/// same data dir" is only true if the check comes first.
fn check_binding(
    descriptor: &RuntimeDescriptor,
    data_dir: &Path,
    estate_root: &Path,
) -> Result<(), CliError> {
    descriptor
        .check_estate_root(data_dir, Some(estate_root))
        .map_err(|e| CliError::new(e.to_string()))
}

/// Every observation verb's daemon connection (ADR 0009, D5; ruled first for
/// `watch` alone by R-WATCH-3) — the three detection branches
/// `sgt doctor`/`daemon_stop` already use (`src/cli.rs:1266-1293` is this
/// ruling's own citation for the precedent), mirrored here rather than
/// shared because `observe_connect` must do the one thing every mutating
/// command in this file deliberately does not: **never call
/// [`spawn_daemon`]**. Observation must not materialize the thing observed —
/// fail-closed at both ends of the daemon's life, matching R-WATCH-3's own
/// framing. Used by `watch`, `status`, `work show`/`list`/`transcript`/
/// `retained`, `work reap`'s unconfirmed preview (its `--yes` disposal path
/// still mutates and still goes through [`ensure_daemon`]), `analytics`,
/// and `tui` — every verb ADR 0009 moved into the no-spawn set.
///
/// 1. A healthy descriptor → attach, exactly like [`ensure_daemon`].
/// 2. No descriptor, or a descriptor whose PID is dead → refuse, naming the
///    data dir and the remedy (`sgt doctor`), exit nonzero, spawn nothing.
/// 3. A descriptor naming a live PID that does not answer `/healthz` → the
///    same ambiguous, fail-closed refusal `ensure_daemon` gives, spawn
///    nothing.
async fn observe_connect(data_dir: &Path, estate_root: &Path) -> Result<ApiClient, CliError> {
    let path = daemon::descriptor_path(data_dir);
    let Some(descriptor) = daemon::read_descriptor(data_dir)? else {
        return Err(CliError::new(format!(
            "no daemon is running for {}; observation commands refuse to start one \
             (`sgt doctor` for the full picture) — start one with a mutating verb instead \
             (e.g. `sgt run`) or `sgt daemon`",
            data_dir.display(),
        )));
    };
    // §5.1, before the endpoint is touched at all — the same gate
    // `ensure_daemon` applies, for the same reason.
    check_binding(&descriptor, data_dir, estate_root)?;
    let http = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;
    if healthz_ok(&http, &descriptor.endpoint).await {
        return client_for(&descriptor);
    }
    if daemon::pid_alive(descriptor.pid) {
        return Err(CliError::new(format!(
            "daemon descriptor at {} names PID {} (alive) but {} does not answer /healthz; \
             refusing to spawn a second daemon and refusing to guess. If it is truly gone, \
             remove the descriptor file and retry, or run `sgt doctor` to check.",
            path.display(),
            descriptor.pid,
            descriptor.endpoint,
        )));
    }
    Err(CliError::new(format!(
        "no daemon is running for {} (descriptor at {} is stale — pid {} is gone); \
         observation commands refuse to start one (`sgt doctor` for the full picture) — \
         start one with a mutating verb instead (e.g. `sgt run`) or `sgt daemon`",
        data_dir.display(),
        path.display(),
        descriptor.pid,
    )))
}

/// Whether `candidate` is still the exact descriptor already judged stale.
/// Identity is endpoint + PID + token; a daemon publishes a fresh random
/// token, so a successor's descriptor can never compare equal to it.
fn is_stale_descriptor(stale: Option<&RuntimeDescriptor>, candidate: &RuntimeDescriptor) -> bool {
    stale.is_some_and(|stale| {
        stale.endpoint == candidate.endpoint
            && stale.pid == candidate.pid
            && stale.token == candidate.token
    })
}

/// One `/healthz` probe with a short timeout.
async fn healthz_ok(http: &reqwest::Client, endpoint: &str) -> bool {
    matches!(
        http.get(format!("{endpoint}/healthz"))
            .timeout(HEALTH_TIMEOUT)
            .send()
            .await,
        Ok(resp) if resp.status().is_success()
    )
}

/// Spawn a detached `sgt daemon` for `data_dir`: own process group, stdio to
/// `daemon.log` in the data dir. The child is *not* waited on — it outlives
/// this client by design; losing the daemon-lock race makes it exit on its
/// own.
fn spawn_daemon(data_dir: &Path, estate_root: &Path) -> Result<(), CliError> {
    std::fs::create_dir_all(data_dir)?;
    let exe = std::env::current_exe()?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("daemon.log"))?;
    let mut command = std::process::Command::new(exe);
    // §5.1, C10: the spawned daemon is bound to the estate this client
    // already admitted — named explicitly with `-C` rather than inherited
    // from a cwd the detached child does not reliably keep.
    command
        .arg("-C")
        .arg(estate_root)
        .arg("--data-dir")
        .arg(data_dir)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log.try_clone()?))
        .stderr(std::process::Stdio::from(log));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // New process group: the daemon must survive the client's terminal
        // and not receive the client's signals.
        command.process_group(0);
    }
    command.spawn()?;
    Ok(())
}

/// How long `sgt daemon stop` waits for `active` work to settle before
/// giving up on the drain and proceeding to SIGTERM anyway.
///
/// A bound, not a promise: R-MVP1-7's own per-turn ceiling can run to
/// minutes, so waiting out every possible in-flight turn unconditionally
/// would make this command itself an unbounded hang on an interactive
/// terminal — exactly the kind of thing a *stop* command must never be. The
/// admission pause this function journals first is what actually protects
/// the operator's intent (no new work starts), and a Work still draining
/// when this deadline passes is simply left for ordinary restart recovery
/// (§25) to pick back up the next time a daemon starts on this data dir —
/// nothing about proceeding here destroys it. An operator who wants to wait
/// longer can just run `sgt daemon stop` again; pausing an already-paused
/// daemon journals nothing new (`pause_admission`'s own idempotence).
const DRAIN_TIMEOUT: Duration = Duration::from_secs(60);
/// Poll interval while draining.
const DRAIN_POLL: Duration = Duration::from_millis(200);
/// How long `sgt daemon stop` waits for the daemon to actually exit after
/// SIGTERM before reporting that it did not. Mirrors the test rig's own
/// SIGTERM grace (`tests/support::TERM_GRACE`) — the same shutdown path
/// `the_spawned_daemon_rig_stops_its_daemon_with_sigterm` already proves the
/// daemon meets, reached here from a client instead of a test harness.
const STOP_TERM_GRACE: Duration = Duration::from_secs(15);
/// Poll interval while waiting for the daemon to exit.
const STOP_POLL: Duration = Duration::from_millis(100);

/// `sgt daemon stop` (MVP-3, E4; the bucketing doc's "cheap-now" item):
/// pause admission, drain in-flight work, then the same graceful SIGTERM
/// shutdown `sgt daemon` (foreground) already answers to.
///
/// Deliberately does **not** auto-spawn a daemon — asking to stop something
/// that is not running is answered "already stopped", never "let me start
/// one so I can stop it", mirroring `sgt doctor`'s own no-auto-spawn rule.
async fn daemon_stop(data_dir: &Path, estate_root: &Path, json: bool) -> Result<(), CliError> {
    let Some(descriptor) = daemon::read_descriptor(data_dir)? else {
        report_daemon_stop(
            json,
            "not_running",
            "no daemon is running for this data dir",
        );
        return Ok(());
    };
    // §5.1, before the endpoint is touched: stopping is *using* a daemon,
    // and a daemon bound to another estate is no more this estate's to stop
    // than it is this estate's to submit to.
    check_binding(&descriptor, data_dir, estate_root)?;
    let http = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;
    if !healthz_ok(&http, &descriptor.endpoint).await {
        if daemon::pid_alive(descriptor.pid) {
            // Same ambiguity `ensure_daemon` refuses to guess through: a
            // live PID that will not answer `/healthz` might be this
            // daemon mid-startup, or an unrelated process that happens to
            // reuse the pid. Fail closed rather than signal it.
            return Err(CliError::new(format!(
                "daemon descriptor at {} names PID {} (alive) but {} does not answer /healthz; \
                 refusing to signal it. If it is truly gone, remove the descriptor and retry.",
                daemon::descriptor_path(data_dir).display(),
                descriptor.pid,
                descriptor.endpoint,
            )));
        }
        report_daemon_stop(
            json,
            "not_running",
            "no daemon is running for this data dir (stale descriptor)",
        );
        return Ok(());
    }
    let client = client_for(&descriptor)?;

    // 1. Pause admission — MVP-3's drain flag, scoped exactly to this verb.
    // Idempotent: a retry against a still-live, already-paused daemon
    // journals nothing new (`pause_admission`'s own doc).
    let pause_body = json!({"command_id": ulid::Ulid::generate().to_string()});
    client.post("/v1/admission/pause", &pause_body).await?;

    // 2. Drain: wait for every `active` work to settle, bounded by
    // `DRAIN_TIMEOUT` above.
    let drain_deadline = Instant::now() + DRAIN_TIMEOUT;
    loop {
        let works = client.get("/v1/work").await?;
        let active = works["works"]
            .as_array()
            .map(|list| list.iter().filter(|w| w["state"] == "active").count())
            .unwrap_or(0);
        if active == 0 || Instant::now() >= drain_deadline {
            break;
        }
        tokio::time::sleep(DRAIN_POLL).await;
    }

    // 3. The ordinary, already-tested graceful shutdown path: SIGTERM, then
    // wait for the daemon to exit and journal `daemon.stopped` on its own.
    // `kill(1)` rather than a signals crate — this binary has none, and the
    // test rig (`tests/support::reap_daemons`) already leans on the exact
    // same external command for the exact same reason (no dependency worth
    // adding for one syscall's worth of shelling out).
    let status = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(descriptor.pid.to_string())
        .status()
        .map_err(|e| CliError::new(format!("cannot signal pid {}: {e}", descriptor.pid)))?;
    if !status.success() {
        // `kill` fails when the pid is already gone — treat that as success
        // (idempotent stop), not an error.
        if daemon::pid_alive(descriptor.pid) {
            return Err(CliError::new(format!(
                "kill -TERM {} exited with {status}",
                descriptor.pid
            )));
        }
        report_daemon_stop(json, "stopped", "daemon already stopped");
        return Ok(());
    }
    let term_deadline = Instant::now() + STOP_TERM_GRACE;
    while daemon::pid_alive(descriptor.pid) && Instant::now() < term_deadline {
        tokio::time::sleep(STOP_POLL).await;
    }
    if daemon::pid_alive(descriptor.pid) {
        return Err(CliError::new(format!(
            "sent SIGTERM to pid {} but it did not exit within {STOP_TERM_GRACE:?} — it may \
             still be tearing down a slow effect; check daemon.log in the data dir, or retry \
             `sgt daemon stop`",
            descriptor.pid,
        )));
    }
    report_daemon_stop(json, "stopped", "daemon stopped");
    Ok(())
}

/// `sgt daemon stop`'s one report shape, human or `--json`.
fn report_daemon_stop(json: bool, status: &str, message: &str) {
    if json {
        print_json(&json!({"status": status, "message": message}));
    } else {
        println!("{message}");
    }
}

/// `sgt doctor` (proposal §31): diagnose one installation.
///
/// The rule every check here obeys: **a failing check names its remedy.** A
/// diagnostic that reports a fault without saying what to do about it has
/// moved the problem, not diagnosed it. The checks run in a fixed order and
/// the `--json` shape is stable — one object per check, always the same keys,
/// always the same names — because the first consumer of a doctor is a bug
/// report and the second is a script.
///
/// Doctor deliberately does **not** auto-spawn a daemon — it joins the rest
/// of ADR 0009's no-spawn set (`status`, `work`, `analytics`, `watch`,
/// `tui`, `daemon stop`). Only the mutating verbs (`run`, `respond`,
/// `retry`, `extend`, `cancel`) start one on demand; this one is asking
/// whether the installation is sound, and starting the thing under
/// examination would answer a different question.
pub(crate) mod doctor {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use serde_json::{Value, json};

    use crate::backend::Backend;
    use crate::backend::claude::{
        CLAUDE_BIN_ENV, ClaudeBackend, ClaudeConfig, MIN_TRUSTED_VERSION,
    };
    use crate::backend::docker::{self, DockerBackend, DockerConfig};
    use crate::daemon;
    use crate::domain::estate::MANIFEST_FILE;
    use crate::domain::event::Event;
    use crate::runtime::analytics::Analytics;
    use crate::runtime::journal::Journal;

    /// A check's verdict.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Status {
        /// Working as it should.
        Ok,
        /// Usable, but something is off and will bite later.
        Warn,
        /// Broken: this installation cannot do its job.
        Fail,
    }

    impl Status {
        fn as_str(self) -> &'static str {
            match self {
                Status::Ok => "ok",
                Status::Warn => "warn",
                Status::Fail => "fail",
            }
        }

        fn marker(self) -> &'static str {
            match self {
                Status::Ok => "ok  ",
                Status::Warn => "warn",
                Status::Fail => "FAIL",
            }
        }
    }

    /// One diagnostic.
    #[derive(Debug, Clone)]
    pub struct Check {
        /// Stable machine name.
        pub name: &'static str,
        /// Verdict.
        pub status: Status,
        /// What was measured.
        pub detail: String,
        /// What to do about it — present whenever the status is not `Ok`.
        pub remedy: Option<String>,
    }

    impl Check {
        fn ok(name: &'static str, detail: impl Into<String>) -> Self {
            Self {
                name,
                status: Status::Ok,
                detail: detail.into(),
                remedy: None,
            }
        }

        fn warn(name: &'static str, detail: impl Into<String>, remedy: impl Into<String>) -> Self {
            Self {
                name,
                status: Status::Warn,
                detail: detail.into(),
                remedy: Some(remedy.into()),
            }
        }

        fn fail(name: &'static str, detail: impl Into<String>, remedy: impl Into<String>) -> Self {
            Self {
                name,
                status: Status::Fail,
                detail: detail.into(),
                remedy: Some(remedy.into()),
            }
        }
    }

    /// The full diagnosis.
    #[derive(Debug, Clone)]
    pub struct Report {
        /// Data dir the report is about.
        pub data_dir: PathBuf,
        /// Checks, in a fixed order.
        pub checks: Vec<Check>,
    }

    impl Report {
        /// Whether the installation can do its job. Warnings do not make an
        /// installation unhealthy — they make it worth reading about.
        pub fn healthy(&self) -> bool {
            self.checks.iter().all(|c| c.status != Status::Fail)
        }

        /// `sgt init`'s narrower question: did init's own responsibilities —
        /// scaffolding plus the estate-fundamental checks (`data_dir`,
        /// `journal`, `projection`) — succeed. Harness rows (`claude`,
        /// `docker`) are advisory at init time: §17.5's degraded-daemon
        /// doctrine and NORTH-STAR's day-one loop both say a missing harness
        /// narrows which backends can run work later, it must not brick
        /// estate setup. A colleague without the `claude` CLI installed must
        /// still be able to `sgt init` — the row still prints as `FAIL` with
        /// its remedy (that part stays honest), it just does not gate this
        /// exit code. `sgt doctor` run standalone keeps [`Self::healthy`]
        /// unchanged: an operator asking "is everything healthy" deserves
        /// the hard answer.
        pub fn healthy_for_init(&self) -> bool {
            self.checks
                .iter()
                .all(|c| c.status != Status::Fail || matches!(c.name, "claude" | "docker"))
        }

        /// The stable `--json` shape.
        pub fn to_json(&self) -> Value {
            json!({
                "healthy": self.healthy(),
                "data_dir": self.data_dir,
                "checks": self.checks.iter().map(|c| json!({
                    "name": c.name,
                    "status": c.status.as_str(),
                    "detail": c.detail,
                    "remedy": c.remedy,
                })).collect::<Vec<_>>(),
            })
        }

        /// The human report: one line per check, remedy indented under any
        /// check that is not `Ok`. A `detail` that itself spans several
        /// lines (§12.2's `git_surfaces` block is the one row that does)
        /// gets the same continuation treatment remedy already has below,
        /// indented to the column its own first line starts at
        /// (`"  [{marker}] {name:<12} "`'s fixed width) rather than back to
        /// column zero — so the block reads as one row's own detail, not as
        /// stray output between checks.
        pub fn print(&self) {
            const DETAIL_COLUMN: usize = 22;
            println!("sergeant doctor — {}", self.data_dir.display());
            for check in &self.checks {
                let mut lines = check.detail.lines();
                println!(
                    "  [{}] {:<12} {}",
                    check.status.marker(),
                    check.name,
                    lines.next().unwrap_or("")
                );
                for line in lines {
                    println!("{:DETAIL_COLUMN$}{line}", "");
                }
                if let Some(remedy) = &check.remedy {
                    // A remedy may be several lines — §4.4's estate-root
                    // diagnostic is a whole block. Continuation lines are
                    // indented to the same column as the first so the
                    // remedy still reads as one indented thing under its
                    // check, rather than spilling back to column zero.
                    let mut lines = remedy.lines();
                    if let Some(first) = lines.next() {
                        println!("         remedy: {first}");
                        for line in lines {
                            if line.is_empty() {
                                println!();
                            } else {
                                println!("                 {line}");
                            }
                        }
                    }
                }
            }
            println!(
                "{}",
                if self.healthy() {
                    "healthy"
                } else {
                    "unhealthy — see the remedies above"
                }
            );
        }
    }

    /// Run every check against `data_dir`. `xdg_outranked` is #80's own
    /// fact from `cli::resolve_data_dir`'s resolution — whether
    /// `$XDG_DATA_HOME` was set but never reached because estate-first
    /// resolution already won; threaded down to `data_dir_check` so its
    /// `ok` detail can disclose the precedence, not just the outcome.
    pub async fn run(data_dir: &Path, root: &Path, xdg_outranked: bool) -> Report {
        let mut checks = vec![git_check(), claude_check(data_dir), environment_check()];
        // #67: the data dir's own existence and writability is checked
        // before anything that lives *inside* it — the docker adapter's
        // blob store and disk-pressure's `df` call both fail the instant
        // the data dir cannot be created, and running them first surfaced
        // that single fault as two unrelated-looking downstream symptoms.
        // Threading `data_dir_ok` down mirrors `journal_ok` below: the
        // checks that depend on it decline by name instead of re-deriving
        // (and re-explaining) the same root cause.
        let (data_dir_check, data_dir_ok) = data_dir_check(data_dir, xdg_outranked);
        checks.push(data_dir_check);
        // #85, ADR 0003 D6: whether this data dir's filesystem supports
        // reliable advisory locking, independent of `data_dir_ok` — this
        // answers a question about the mount underneath, not about
        // writability, and (via `fs_locking::detect_for_path`'s own ancestor
        // walk) works whether or not the data dir has been created yet.
        checks.push(filesystem_check(data_dir));
        checks.push(docker_check(data_dir, data_dir_ok));
        // The journal is the only durable fact in the installation, so it is
        // checked before anything derived from it; a projection rebuild over
        // an unreadable journal would report the journal's fault under the
        // projection's name.
        let (journal_check, journal_ok, journal_events) = journal_check(data_dir);
        checks.push(journal_check);
        checks.push(projection_check(journal_ok, journal_events.as_deref()));
        checks.push(daemon_check(data_dir).await);
        // §4.2: `sgt doctor` is usable outside an estate and never searches
        // upward. The estate-root row is the one that says out loud whether
        // this directory is an estate root at all — a failing row with the
        // §4.4 remedy when it is not, rather than a walk that quietly finds
        // some other estate and reports on that instead. Threaded down to
        // the rows beneath it exactly the way `data_dir_ok` is: they decline
        // by name instead of each re-deriving the same answer.
        let (estate_root_check, admitted) = estate_root_check(root);
        checks.push(estate_root_check);
        checks.push(permission_mode_check(admitted.as_deref()));
        checks.push(estate_check(admitted.as_deref()));
        checks.push(workflows_check(admitted.as_deref()));
        // §12.2: cheap, bounded — journal plus retained-artifact filesystem
        // metadata, never a per-branch git walk. Same estate-root threading
        // as the two rows above.
        checks.push(git_surfaces_check(
            data_dir,
            admitted.as_deref(),
            journal_events.as_deref(),
        ));
        // N4/#23 (retention Rule B): disk pressure inside the data dir. Runs
        // after everything above regardless of their outcome — knowing "is
        // this installation about to run out of disk" does not depend on the
        // daemon, the journal, or Docker being reachable.
        checks.push(disk_pressure_check(data_dir, data_dir_ok));
        Report {
            data_dir: data_dir.to_path_buf(),
            checks,
        }
    }

    /// §4.2/§4.4: is the directory this `sgt doctor` was run from an estate
    /// root at all?
    ///
    /// `sgt doctor` is one of the five commands usable outside an estate,
    /// and §4.2 is explicit about what it must do there: "reports
    /// installation health and a failing estate-root row with the remedy to
    /// cd or initialize. It does not start a daemon." So this row **never
    /// searches upward** — it asks [`Estate::admit`] about exactly one
    /// directory — and answers with §4.4's own diagnostic when the answer is
    /// no, rather than quietly reporting on some other estate found above.
    ///
    /// Returns the admitted root alongside the row, so the three estate rows
    /// beneath it read one already-decided answer instead of each deriving
    /// their own (the same threading `data_dir_ok` uses).
    fn estate_root_check(root: &Path) -> (Check, Option<PathBuf>) {
        use crate::domain::estate::Estate;

        match Estate::admit(root) {
            Ok(admitted) => {
                let check = Check::ok(
                    "estate_root",
                    format!("{} is an estate root", admitted.path.display()),
                );
                (check, Some(admitted.path))
            }
            Err(e) => {
                let e = match super::bound_root_above(root) {
                    Some(bound) => e.with_bound_root(bound),
                    None => e,
                };
                // The diagnostic already carries the expected path, the
                // "no parent search" explanation, and the remedy, in §4.4's
                // own words. Splitting it into detail/remedy would either
                // duplicate it or lose half of it, so the detail is its
                // first line and the remedy is the rest.
                let rendered = e.to_string();
                let (first, rest) = rendered.split_once('\n').unwrap_or((&rendered, ""));
                (
                    Check::fail("estate_root", first.to_string(), rest.trim().to_string()),
                    None,
                )
            }
        }
    }

    /// §31, #47: the effective `--permission-mode` behavior each declared
    /// profile launches with — the same question `a_profile_is_launch_
    /// configuration_carried_to_the_claude_adapter` pins in code, surfaced
    /// where an operator reading `sgt doctor` will actually see it.
    ///
    /// The estate manifest, not the data dir: profiles live in
    /// `sergeant.toml` at the estate root the doctor is *run from*, which is
    /// also why a malformed `permission_mode` here reads as this check's own
    /// failure rather than a mysterious daemon-side refusal later — #47's
    /// fail-closed load already ran by the time this string is built.
    ///
    /// `estate_root` is [`estate_root_check`]'s already-admitted answer
    /// (§4.1's exact root, `None` when this directory is not one) — never a
    /// second, independent search.
    fn permission_mode_check(estate_root: Option<&Path>) -> Check {
        use crate::domain::estate::Estate;

        let Some(estate_root) = estate_root else {
            return Check::ok("permission_mode", "not an estate root — nothing to report");
        };
        match Estate::from_config_allow_empty(&estate_root.join(MANIFEST_FILE)) {
            Ok(estate) if estate.profiles.is_empty() => Check::ok(
                "permission_mode",
                "no profiles declared — every execution launches with no --permission-mode \
                 flag at all (the CLI's own default)",
            ),
            Ok(estate) => {
                let modes: Vec<String> = estate
                    .profiles
                    .iter()
                    .map(|p| {
                        // Already validated at load (#47): from_config would
                        // have refused the estate before this ran.
                        let effective = match p.permission_mode().ok().flatten() {
                            Some(mode) => mode.as_cli_value().to_string(),
                            None => "unspecified -> no flag (CLI default)".to_string(),
                        };
                        format!("{}={effective}", p.name)
                    })
                    .collect();
                Check::ok("permission_mode", modes.join(", "))
            }
            Err(e) => Check::warn(
                "permission_mode",
                format!("cannot read this estate's profiles: {e}"),
                "fix sergeant.toml at the location the error names",
            ),
        }
    }

    /// MVP-3: the estate manifest's own health, beyond whether it merely
    /// parses. Bounded at this installation's own data dir, same discovery
    /// as [`permission_mode_check`]; silent (`ok`) outside any estate.
    ///
    /// A manifest that fails to parse at all (malformed TOML, R-MVP1-3's
    /// legacy-vocabulary refusal, a duplicate or invalid repository name)
    /// reports that failure's own message as this check's detail — every
    /// `EstateError` variant already names its file and the offending
    /// key, and `Malformed` additionally carries `toml::de::Error`'s own
    /// line/column, so nothing here needs to reconstruct that.
    ///
    /// Once the manifest parses, this looks past what execution's own
    /// strict loader (`Estate::from_config`) would ever tell you, because
    /// that loader fails closed at the *first* problem it finds — right for
    /// launching a Work, useless for "what is wrong with my estate right
    /// now": it uses [`crate::domain::estate::Estate::declared_repos`]
    /// instead, which names every declared repository regardless of whether
    /// earlier ones are missing, then cross-checks two directions —
    /// declared-but-absent (remedy: the declared `origin`, when there is
    /// one, else the `sgt repo add` command that would set one) and
    /// present-but-undeclared (a directory under `repos/` no `[[repo]]`
    /// entry names).
    fn estate_check(estate_root: Option<&Path>) -> Check {
        use crate::domain::estate::Estate;

        let Some(estate_root) = estate_root else {
            return Check::ok("estate", "not an estate root — nothing to check");
        };
        let manifest_path = estate_root.join(MANIFEST_FILE);
        // Estate-root Phase D: a manifest that cannot be parsed at all no
        // longer reaches this row. `estate_root_check` already ran
        // `Estate::admit`, which runs the identical parse — TOML syntax,
        // the R-MVP1-3 legacy-vocabulary refusal, §6.1's removed `path` key,
        // duplicate or invalid repository names — and only hands this row an
        // `estate_root` when all of it passed. Kept as a defensive arm rather
        // than an `expect`, because "the manifest changed between two reads
        // of the same doctor run" is a race, not an invariant, and a doctor
        // that panics on it would be the worst possible answer.
        let declared = match Estate::declared_repos(&manifest_path) {
            Ok(declared) => declared,
            Err(e) => {
                return Check::fail(
                    "estate",
                    e.to_string(),
                    format!(
                        "fix {} at the location named above",
                        manifest_path.display()
                    ),
                );
            }
        };

        let mut declared_names: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        let mut details = Vec::new();
        let mut remedies = Vec::new();
        for repo in &declared {
            declared_names.insert(repo.name.clone());
            if repo.path.exists() {
                continue;
            }
            details.push(format!(
                "{} is declared at {} but missing on disk",
                repo.name,
                repo.path.display()
            ));
            remedies.push(match &repo.origin {
                Some(origin) => format!(
                    "{}: clone it — `sgt repo add {} --origin {origin}`",
                    repo.name, repo.name
                ),
                None => format!(
                    "{}: no origin is declared — either place a git checkout at {} \
                     yourself or declare one with `sgt repo add {} --origin <url>`",
                    repo.name,
                    repo.path.display(),
                    repo.name
                ),
            });
        }

        let repos_dir = estate_root.join(crate::domain::estate::REPOS_DIR);
        if let Ok(entries) = std::fs::read_dir(&repos_dir) {
            let mut undeclared: Vec<String> = entries
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| !declared_names.contains(name))
                .collect();
            undeclared.sort();
            for name in undeclared {
                details.push(format!(
                    "repos/{name} is on disk but not declared in {MANIFEST_FILE}"
                ));
                remedies.push(format!(
                    "{name}: declare it — `sgt repo add {name}` (or remove the directory if it \
                     should not be there)"
                ));
            }
        }

        if details.is_empty() {
            Check::ok(
                "estate",
                format!(
                    "{} repositories declared, all present on disk, no undeclared directories \
                     under repos/",
                    declared.len()
                ),
            )
        } else {
            Check::warn("estate", details.join("; "), remedies.join("; "))
        }
    }

    /// #165 (Ponytail R2 — visibility, not the fix): how many workflow
    /// packages the estate declares under `.sergeant/workflows/`. A fresh
    /// `sgt init` writes none — `WorkflowDefinition::resolve`
    /// (`src/domain/workflow.rs`) only ever falls back to the binary's one
    /// embedded package (`software-change`, [`crate::domain::workflow::
    /// DEFAULT_WORKFLOW`]), so any other explicitly named workflow 422s at
    /// submit with nothing in this report to explain why. The full fix —
    /// `sgt init` writing real packages, or resolution falling back to a
    /// binary-embedded set with more than one member — is Phase 3 embedding
    /// (`reference/proposal-product-estate-split.md`), out of scope
    /// here; this check exists so "zero" is something `sgt doctor` says out
    /// loud instead of a fact a submitter only learns from a 422.
    ///
    /// §6 Phase 3 (Ponytail R2 — reuse this existing check surface rather
    /// than add a sibling): also reports how many `.sergeant/local/
    /// workflows/` packages shadow a same-named stock package, and, among
    /// those, how many carry an `edition` (ADR 0016) older than the binary's
    /// own version — the drift ADR 0016 makes checkable by plain string
    /// comparison, without a `sgt workflow diff` verb (ADR 0014 decision 4).
    /// A local package's `edition` is set once, at fork time, to whatever
    /// the stock copy it forked from carried — which `sgt init`/update
    /// always writes as the current distro version — so comparing a fork's
    /// `edition` against the binary's own version is the same comparison
    /// ADR 0016 describes against "the currently-shipped stock package's
    /// edition", not a different one.
    ///
    /// Same discovery as [`estate_check`]; silent (`ok`) outside any
    /// estate. Zero stock packages is `warn`, not `fail`: the embedded
    /// default still runs unnamed dispatch (§30), so an estate with none is
    /// degraded, not broken. Drift among local packages is also `warn`, not
    /// `fail`: a stale fork still runs — it is stale, not broken.
    fn workflows_check(estate_root: Option<&Path>) -> Check {
        use crate::domain::workflow::{
            LOCAL_WORKFLOW_ROOT, WORKFLOW_ROOT, read_index_front_matter,
        };

        let Some(estate_root) = estate_root else {
            return Check::ok("workflows", "not an estate root — nothing to check");
        };

        let workflows_dir = estate_root.join(WORKFLOW_ROOT);
        let mut names: Vec<String> = match std::fs::read_dir(&workflows_dir) {
            Ok(entries) => entries
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
                .filter(|entry| entry.path().join("workflow.toml").is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        };
        names.sort();

        let local_workflows_dir = estate_root.join(LOCAL_WORKFLOW_ROOT);
        let mut local_names: Vec<String> = match std::fs::read_dir(&local_workflows_dir) {
            Ok(entries) => entries
                .flatten()
                .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
                .filter(|entry| entry.path().join("workflow.toml").is_file())
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => Vec::new(),
        };
        local_names.sort();

        let stock: std::collections::BTreeSet<&str> = names.iter().map(String::as_str).collect();
        let shadowing: Vec<&String> = local_names
            .iter()
            .filter(|n| stock.contains(n.as_str()))
            .collect();
        let current_edition = env!("CARGO_PKG_VERSION");
        let drifted: Vec<&String> = shadowing
            .iter()
            .filter(|n| {
                read_index_front_matter(&local_workflows_dir.join(n))
                    .and_then(|fm| fm.edition)
                    .is_some_and(|edition| edition != current_edition)
            })
            .copied()
            .collect();

        let drift_suffix = if shadowing.is_empty() {
            String::new()
        } else if drifted.is_empty() {
            format!(
                "; {} local package(s) shadow stock, all at the current edition {current_edition:?}: {}",
                shadowing.len(),
                shadowing
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        } else {
            format!(
                "; {} local package(s) shadow stock, {} older than the current edition {current_edition:?}: {}",
                shadowing.len(),
                drifted.len(),
                drifted
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        };

        if names.is_empty() {
            Check::warn(
                "workflows",
                format!(
                    "0 workflow packages declared under {} — only the built-in {:?} runs \
                     (unnamed dispatch, or `--workflow {:?}` explicitly); any other \
                     `--workflow <name>` will 422{drift_suffix}",
                    workflows_dir.display(),
                    crate::domain::workflow::DEFAULT_WORKFLOW,
                    crate::domain::workflow::DEFAULT_WORKFLOW,
                ),
                format!(
                    "write a package to {}/<name>/workflow.toml before dispatching `--workflow \
                     <name>`, or re-run `sgt init` — it writes the embedded distro's stock \
                     packages for any that are missing (#165), without touching one already \
                     present",
                    workflows_dir.display()
                ),
            )
        } else if !drifted.is_empty() {
            Check::warn(
                "workflows",
                format!(
                    "{} workflow package(s) declared: {}{drift_suffix}",
                    names.len(),
                    names.join(", ")
                ),
                format!(
                    "run `sgt workflow fork <name>` again for {} to pick up the current stock \
                     edition {current_edition:?} (ADR 0016 — this never overwrites a local \
                     package by itself; remove the stale one first if you want the refork)",
                    drifted
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            )
        } else {
            Check::ok(
                "workflows",
                format!(
                    "{} workflow package(s) declared: {}{drift_suffix}",
                    names.len(),
                    names.join(", ")
                ),
            )
        }
    }

    /// estate-root proposal §12.2: a cheap, bounded Git-surface summary
    /// derived from the journal and retained-artifact filesystem metadata
    /// only — never a per-branch `git` ancestry walk (§12.3's expensive
    /// reconciliation, unbuilt on purpose, owns that). Same discovery as
    /// [`estate_check`]/[`workflows_check`]; silent (`ok`) outside any
    /// estate.
    ///
    /// "Active" here is the git-surface sense — a Work's surface has a
    /// `surface.materialized` event with no later `surface.torn_down` for
    /// the same work id — not [`crate::domain::work::WorkState::Active`];
    /// a Work sitting in `waiting`/`needs_input`/`blocked` still holds an
    /// open surface and counts here. A rematerialization (crash-recovery
    /// replay reissuing `surface.materialized` for a work id already torn
    /// down) clears the earlier teardown and its integrity verdict for that
    /// work id, mirroring `work_registry_reducer`'s own `run.teardown =
    /// None`/`run.integrity = None` on the same event.
    ///
    /// "journaled Work branches" counts every binding any Work's surface
    /// was ever materialized with, active or long since torn down — §12.1's
    /// branches are durable and never deleted here, so this is the
    /// journal's own count of what was created, not a walk of what a
    /// mounted repository still has a ref for.
    ///
    /// "retained worktrees"/"retained patches"/"retained artifact size"
    /// reuse [`crate::runtime::surface::retained_bindings`] — the same live
    /// filesystem read `sgt work retained` answers from — over every
    /// journaled teardown, classifying each entry by whether its retained
    /// path is the captured patch file or the whole worktree directory
    /// fallback. "terminal dirty Works" counts work ids whose most recent
    /// teardown recorded [`crate::runtime::integrity::IntegrityDisposition::Dirty`].
    fn git_surfaces_check(
        data_dir: &Path,
        estate_root: Option<&Path>,
        events: Option<&[Event]>,
    ) -> Check {
        use crate::runtime::integrity::IntegrityDisposition;
        use crate::runtime::surface::{
            KIND_SURFACE_MATERIALIZED, KIND_SURFACE_TORN_DOWN, TeardownReport, WorkSurface,
            retained_bindings,
        };

        if estate_root.is_none() {
            return Check::ok("git_surfaces", "not an estate root — nothing to check");
        }
        if !journal_dir(data_dir).exists() {
            return Check::ok(
                "git_surfaces",
                "no journal yet — nothing has run in this data dir",
            );
        }
        // The journal dir exists, so `journal_check` above attempted a
        // replay; `None` here means that replay failed — its Check already
        // names the fault, this one just declines by pointing at it.
        let Some(events) = events else {
            return Check::warn(
                "git_surfaces",
                "not attempted: the journal did not replay",
                "see the journal check above",
            );
        };

        // work id -> binding count of its most recent `surface.materialized`.
        let mut materialized: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        // work id -> its most recent `surface.torn_down` report.
        let mut torn_down: std::collections::BTreeMap<String, TeardownReport> =
            std::collections::BTreeMap::new();
        // work id -> the integrity verdict that same teardown recorded.
        let mut integrity: std::collections::BTreeMap<String, IntegrityDisposition> =
            std::collections::BTreeMap::new();

        for event in events {
            let Some(work_id) = event.work_id.clone() else {
                continue;
            };
            match event.kind.as_str() {
                KIND_SURFACE_MATERIALIZED => {
                    if let Ok(surface) =
                        serde_json::from_value::<WorkSurface>(event.payload["surface"].clone())
                    {
                        materialized.insert(work_id.clone(), surface.bindings.len());
                        torn_down.remove(&work_id);
                        integrity.remove(&work_id);
                    }
                }
                KIND_SURFACE_TORN_DOWN => {
                    if let Ok(report) =
                        serde_json::from_value::<TeardownReport>(event.payload["report"].clone())
                    {
                        match serde_json::from_value::<IntegrityDisposition>(
                            event.payload["integrity"].clone(),
                        ) {
                            Ok(disposition) => {
                                integrity.insert(work_id.clone(), disposition);
                            }
                            Err(_) => {
                                integrity.remove(&work_id);
                            }
                        }
                        torn_down.insert(work_id, report);
                    }
                }
                _ => {}
            }
        }

        let journaled_branches: usize = materialized.values().sum();
        let active_work_ids: Vec<&str> = materialized
            .keys()
            .filter(|id| !torn_down.contains_key(id.as_str()))
            .map(String::as_str)
            .collect();
        let active_works = active_work_ids.len();
        let active_worktrees: usize = active_work_ids.iter().map(|id| materialized[*id]).sum();

        let mut retained_worktrees = 0u64;
        let mut retained_patches = 0u64;
        let mut retained_bytes = 0u64;
        let mut terminal_dirty_works = 0u64;

        for (work_id, report) in &torn_down {
            for binding in retained_bindings(report) {
                retained_bytes += binding.bytes;
                if binding.reason == "retained_dirty" && binding.path.is_file() {
                    retained_patches += 1;
                } else {
                    retained_worktrees += 1;
                }
            }
            if integrity.get(work_id) == Some(&IntegrityDisposition::Dirty) {
                terminal_dirty_works += 1;
            }
        }

        let detail = [
            "git surfaces".to_string(),
            format!("  active works:              {active_works}"),
            format!("  active linked worktrees:   {active_worktrees}"),
            format!("  retained worktrees:        {retained_worktrees}"),
            format!("  retained patches:          {retained_patches}"),
            format!(
                "  retained artifact size:    {}",
                human_bytes(retained_bytes)
            ),
            format!("  journaled Work branches:   {journaled_branches}"),
            format!("  terminal dirty Works:      {terminal_dirty_works}"),
        ]
        .join("\n");

        if retained_worktrees == 0 && retained_patches == 0 && terminal_dirty_works == 0 {
            Check::ok("git_surfaces", detail)
        } else {
            // §12.3 stays future work: this names the separate inspection/
            // cleanup remedy rather than classifying anything as merged,
            // redundant, or safe to delete, and deletes nothing itself.
            Check::warn(
                "git_surfaces",
                detail,
                "inspect with `sgt work retained` (bytes, path, and why, for everything \
                 retained); `sgt work show <id>` explains a specific terminal-dirty Work's \
                 integrity findings; `sgt work reap --yes <id>` explicitly discards one \
                 reviewed retained binding — this row classifies nothing as merged or \
                 redundant and deletes nothing on its own",
            )
        }
    }

    /// Where the journal's segments live inside a data dir.
    fn journal_dir(data_dir: &Path) -> PathBuf {
        data_dir.join("journal")
    }

    /// §31: git presence and version. Sergeant shells out to the installed
    /// git for every work surface (§34), so an absent git is a hard failure.
    fn git_check() -> Check {
        match Command::new("git").arg("--version").output() {
            Ok(output) if output.status.success() => Check::ok(
                "git",
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
            ),
            Ok(output) => Check::fail(
                "git",
                format!("`git --version` exited with {}", output.status),
                "install a working git; sergeant materializes every work surface with `git worktree`",
            ),
            Err(e) => Check::fail(
                "git",
                format!("cannot run `git --version`: {e}"),
                "install git and make sure it is on the daemon's PATH",
            ),
        }
    }

    /// §31: the claude CLI's presence *and* the adapter's own version gate.
    ///
    /// This runs the adapter's probe rather than a second copy of the rule:
    /// a doctor that says "fine" while the daemon refuses the same binary is
    /// worse than no doctor at all.
    fn claude_check(data_dir: &Path) -> Check {
        let config = ClaudeConfig::new(data_dir);
        let executable = config.executable.display().to_string();
        let report = ClaudeBackend::new(config).probe();
        let detail = report
            .detail
            .unwrap_or_else(|| "no detail reported".to_string());
        if report.available {
            Check::ok("claude", format!("{executable}: {detail}"))
        } else {
            Check::fail(
                "claude",
                format!("{executable}: {detail}"),
                format!(
                    "install the Claude CLI at >= {}.{}.{} (or point {CLAUDE_BIN_ENV} at one); \
                     until then only the `fake` backend can run work",
                    MIN_TRUSTED_VERSION.0, MIN_TRUSTED_VERSION.1, MIN_TRUSTED_VERSION.2
                ),
            )
        }
    }

    /// ADR 0006's residual hole, named in the ADR and in proposal §5.2:
    /// `sgt claude`/`codex`/`opencode`/`goose` only fix the environment for
    /// sessions that go through the front door. `sgt run` from a terminal
    /// that never did returns to #60. This is the complement the ADR names —
    /// issue #100 — checking the *current* process's own environment against
    /// the exact list [`crate::harness::toolchain_path_dirs`] composes, so
    /// this check and the passthrough it is checking can never silently
    /// disagree about what "the environment" means.
    ///
    /// Deliberately `Warn`, not `Fail`: a missing toolchain dir narrows what
    /// an actor launched from here can build, the same posture `docker`'s
    /// row above takes for a capability gap rather than a broken
    /// installation (§17.5's degraded-daemon doctrine).
    fn environment_check() -> Check {
        let Some(home) = std::env::var_os("HOME") else {
            return Check::warn(
                "environment",
                "$HOME is not set — cannot check toolchain PATH enrichment",
                "set HOME, or run through `sgt claude` (or codex/opencode/goose), which composes \
                 PATH before it needs HOME to be set at all",
            );
        };
        let dirs = crate::harness::toolchain_path_dirs(Path::new(&home));
        let missing = crate::harness::dirs_missing_from_path(
            std::env::var_os("PATH").as_ref(),
            &dirs,
            |dir| dir.exists(),
        );
        if missing.is_empty() {
            Check::ok(
                "environment",
                "PATH already includes the toolchain directories `sgt claude` (and codex/\
                 opencode/goose) compose — or none of them exist on this host",
            )
        } else {
            let names: Vec<String> = missing.iter().map(|d| d.display().to_string()).collect();
            Check::warn(
                "environment",
                format!(
                    "{} exist{} on disk but {} not on PATH — this is #60's failure shape: a \
                     tool an actor needs (cargo, a CLI harness itself) can go missing and read \
                     as a permissions fault instead",
                    names.join(", "),
                    if names.len() == 1 { "s" } else { "" },
                    if names.len() == 1 { "is" } else { "are" },
                ),
                format!(
                    "run this session through `sgt claude` (or codex/opencode/goose), which \
                     composes PATH before exec'ing, or add {} to PATH yourself",
                    names.join(", ")
                ),
            )
        }
    }

    /// N4/§17.4/§16.3: whether the local Docker Engine is reachable, *and*
    /// whether the full container lifecycle actually works — not just
    /// whether something answered a version ping.
    ///
    /// INV-R1-06 (MVP-2 D3 fixer pass): before this fix, this check only
    /// ever ran `backend.probe()` (the cheap `docker version` ping
    /// `Engine::bind_stages` also runs on every submission), and
    /// `DockerBackend::lifecycle_probe` — §16.3's real round trip: create,
    /// bind-mount, start, write, read the write back on the host, remove —
    /// was dead code the module's own doc incorrectly claimed was already
    /// wired in here. §16.3's whole point is that a version ping proves only
    /// that *something* answered: a daemon whose socket responds but whose
    /// bind-mount plumbing is broken (a misconfigured storage driver, a
    /// permission wall on the mount path) would report `ok` forever under
    /// the cheap probe alone. Now the real round trip runs whenever the
    /// cheap probe succeeds, and its own failure (not just the ping's)
    /// downgrades this check to `Warn` — still not `Fail`, for the same
    /// reason as before: §17.5 makes an execute-workflow submission the
    /// thing that actually refuses, and an actor-only installation is fully
    /// healthy either way. This row exists so an operator can tell *why* an
    /// execute submission would be refused before trying one, with real
    /// evidence rather than a ping's optimism.
    fn docker_check(data_dir: &Path, data_dir_ok: bool) -> Check {
        // #67: `DockerBackend::new` opens a blob store under `data_dir` and
        // fails with the same EPERM the `data_dir` check above already
        // named and gave a remedy for — re-running it here would just print
        // that fault a second time under a different, more confusing name
        // ("could not initialize the Docker adapter") without adding
        // information.
        if !data_dir_ok {
            return Check::warn(
                "docker",
                "not attempted: the data dir is not writable",
                "fix the data_dir check above first",
            );
        }
        let backend = match DockerBackend::new(DockerConfig::new(data_dir)) {
            Ok(backend) => backend,
            Err(e) => {
                return Check::warn(
                    "docker",
                    format!("could not initialize the Docker adapter: {e}"),
                    "check that the data dir is writable; this is the same failure data_dir \
                     would report",
                );
            }
        };
        let report = backend.probe();
        let ping_detail = report
            .detail
            .unwrap_or_else(|| "no detail reported".to_string());
        if !report.available {
            return Check::warn(
                "docker",
                ping_detail,
                "install Docker and make sure this user can reach its socket (the `docker` \
                 group on Linux); until then only actor-only workflows can run — a workflow \
                 with a `kind = \"execute\"` stage is refused at submit, before any Work exists",
            );
        }
        let probe = backend.lifecycle_probe(docker::PROD_PROBE_IMAGE);
        if probe.available {
            Check::ok(
                "docker",
                format!("{ping_detail}; bind-mount round trip confirmed"),
            )
        } else {
            Check::warn(
                "docker",
                format!(
                    "{ping_detail}; the container lifecycle round trip failed: {}",
                    probe.detail.as_deref().unwrap_or("no detail reported")
                ),
                "the Docker Engine answers a version ping but a real container round trip \
                 (create, bind-mount, start, write, read back, remove) did not succeed — check \
                 the storage driver and that this user's containers can actually write through \
                 a bind mount; until fixed, only actor-only workflows can run",
            )
        }
    }

    /// N4/#23 (retention Rule B): data-dir size, the blob store's share of
    /// it, and headroom on the filesystem it lives on. Runs regardless of
    /// Docker's availability — this is a core disk concern (the blob store
    /// exists independent of any execute stage ever running), folded into
    /// this module because Docker-captured stdout/stderr is the evidence
    /// class most likely to grow it fast (§16.9, §22.8).
    fn disk_pressure_check(data_dir: &Path, data_dir_ok: bool) -> Check {
        // #67: when the data dir does not exist, `df` on it fails ENOENT and
        // `free_space` reports `None` — which this check used to print as
        // "free space could not be measured on this platform", a claim
        // that's simply false (the platform is fine; the path is missing)
        // and misdirects an operator away from the real, already-named
        // fault above.
        if !data_dir_ok {
            return Check::warn(
                "disk_pressure",
                "not attempted: the data dir is not writable",
                "fix the data_dir check above first",
            );
        }
        let report = docker::measure_disk_pressure(data_dir);
        let detail = format!(
            "data dir {} ({} blobs), {}",
            human_bytes(report.data_dir_bytes),
            human_bytes(report.blob_bytes),
            match report.free_bytes {
                Some(free) => format!("{} free on its filesystem", human_bytes(free)),
                None => "free space could not be measured on this platform".to_string(),
            }
        );
        const FAIL_BELOW: u64 = 100 * 1024 * 1024; // 100 MiB: imminent and actionable
        const WARN_BELOW: u64 = 1024 * 1024 * 1024; // 1 GiB
        match report.free_bytes {
            Some(free) if free < FAIL_BELOW => Check::fail(
                "disk_pressure",
                detail,
                format!(
                    "free {} disk space urgently — the data dir has {} of headroom left; the \
                     blob store never deletes on its own (no blob GC this milestone), so freeing \
                     space elsewhere on the same filesystem is the only lever today",
                    human_bytes(WARN_BELOW),
                    human_bytes(free)
                ),
            ),
            Some(free) if free < WARN_BELOW => Check::warn(
                "disk_pressure",
                detail,
                format!(
                    "only {} free on the data dir's filesystem; watch it, especially if \
                     execute-stage workflows are capturing large output (§22.8)",
                    human_bytes(free)
                ),
            ),
            Some(_) => Check::ok("disk_pressure", detail),
            None => Check::warn(
                "disk_pressure",
                detail,
                "free space could not be measured on this platform (no `df`); watch data dir \
                 growth manually",
            ),
        }
    }

    /// Render a byte count the way an operator reading a terminal wants it,
    /// not the raw integer `disk_pressure_check`'s JSON already carries
    /// losslessly. `pub(super)`: `sgt work retained`/`sgt work reap` (#109)
    /// reuse this rather than a second byte formatter — one declaration,
    /// one resolution.
    pub(super) fn human_bytes(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
        let mut value = bytes as f64;
        let mut unit = 0;
        while value >= 1024.0 && unit < UNITS.len() - 1 {
            value /= 1024.0;
            unit += 1;
        }
        if unit == 0 {
            format!("{bytes} {}", UNITS[unit])
        } else {
            format!("{value:.1} {}", UNITS[unit])
        }
    }

    /// §31/#67: the data dir exists and this user can write it — checked
    /// before anything that lives *inside* it (`docker_check`,
    /// `disk_pressure_check`), so the boolean this returns lets both decline
    /// instead of re-diagnosing the same fault under a more confusing name.
    ///
    /// Probed by actually creating and removing a file. Permission *bits* are
    /// not the question — read-only mounts, full disks and SELinux all answer
    /// "writable" by inspection and "no" in practice.
    ///
    /// #67: when `create_dir_all` fails, the leaf path named in the raw io
    /// error is often not the directory an operator actually needs to fix —
    /// it never got created, so the error is really about wherever the walk
    /// stalled. Walking up to the nearest ancestor that *does* exist and
    /// probing that instead turns "permission denied" on a path that was
    /// never created into one remedy row naming the actual offending parent.
    ///
    /// #80/ADR 0008: `xdg_outranked` only ever adds a detail line to the
    /// `ok` outcome — a set-but-outranked `$XDG_DATA_HOME` is disclosure,
    /// not a fault; it never demotes this row to `warn` and never adds a
    /// row of its own.
    fn data_dir_check(data_dir: &Path, xdg_outranked: bool) -> (Check, bool) {
        let generic_remedy = format!(
            "create {} and make it writable by this user (or point --data-dir / SGT_DATA_DIR \
             somewhere that is)",
            data_dir.display()
        );
        if let Err(e) = std::fs::create_dir_all(data_dir) {
            if let Some(ancestor) = nearest_existing_ancestor(data_dir)
                && !probe_writable(&ancestor)
            {
                return (
                    Check::fail(
                        "data_dir",
                        format!(
                            "{} does not exist and cannot be created: its nearest existing \
                             parent, {}, is not writable by this user ({e})",
                            data_dir.display(),
                            ancestor.display()
                        ),
                        format!(
                            "make {} writable by this user — e.g. `chmod u+w {}` — or point \
                             --data-dir / SGT_DATA_DIR at a location under a writable parent",
                            ancestor.display(),
                            ancestor.display()
                        ),
                    ),
                    false,
                );
            }
            return (
                Check::fail(
                    "data_dir",
                    format!("cannot create {}: {e}", data_dir.display()),
                    generic_remedy,
                ),
                false,
            );
        }
        let probe = data_dir.join(".doctor-write-probe");
        match std::fs::write(&probe, b"doctor") {
            Ok(()) => {
                let _ = std::fs::remove_file(&probe);
                let mut detail = format!("{} is writable", data_dir.display());
                if xdg_outranked {
                    detail.push_str(
                        " (resolved via the estate root; $XDG_DATA_HOME is set but outranked \
                         here — estate-first resolution, ADR 0008)",
                    );
                }
                (Check::ok("data_dir", detail), true)
            }
            Err(e) => (
                Check::fail(
                    "data_dir",
                    format!("cannot write inside {}: {e}", data_dir.display()),
                    generic_remedy,
                ),
                false,
            ),
        }
    }

    /// #85, ADR 0003 D6: does this data dir's filesystem support reliable
    /// advisory locking. The fail-closed asymmetry the ADR calls for: a
    /// *confirmed* bad filesystem is `Fail` (this is not in
    /// `healthy_for_init`'s `claude`/`docker` advisory exemption, so it
    /// blocks `sgt init`'s exit code the same way `data_dir` above does); an
    /// *inconclusive* probe — today, always true on macOS, #85's own open
    /// question — is `Warn`, never `Fail`, because refusing on a probe this
    /// crate cannot yet run would brick `sgt init` on exactly the platform
    /// whose detection is unmeasured.
    fn filesystem_check(data_dir: &Path) -> Check {
        check_from_reliability(
            data_dir,
            crate::platform::fs_locking::detect_for_path(data_dir),
        )
    }

    /// Split from [`filesystem_check`] so the three-way mapping from a
    /// [`crate::platform::fs_locking::Reliability`] verdict to a `Check` is
    /// testable without a real `drvfs`/NFS/SMB mount (`platform::fs_locking`'s
    /// own tests cover the detection half).
    fn check_from_reliability(
        data_dir: &Path,
        reliability: crate::platform::fs_locking::Reliability,
    ) -> Check {
        use crate::platform::fs_locking::Reliability;
        match reliability {
            Reliability::Reliable => Check::ok(
                "filesystem",
                format!("{} supports reliable advisory locking", data_dir.display()),
            ),
            Reliability::Unreliable { filesystem } => Check::fail(
                "filesystem",
                format!(
                    "{} is on a {filesystem} filesystem, where advisory locking (flock) is \
                     unreliable",
                    data_dir.display()
                ),
                crate::platform::fs_locking::remedy(&filesystem),
            ),
            Reliability::Unknown { reason } => Check::warn(
                "filesystem",
                format!(
                    "cannot determine whether {} supports reliable advisory locking: {reason}",
                    data_dir.display()
                ),
                "this could not be measured on this platform; if the estate lives on a WSL2 \
                 /mnt/c mount, an NFS share, or an SMB share, move it to a native filesystem as \
                 a precaution",
            ),
        }
    }

    /// Walk up from `path` to the nearest ancestor that already exists on
    /// disk — the directory `create_dir_all` actually stalled at, and the
    /// one worth naming as the fault (`data_dir_check`, #67).
    fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
        let mut current = path;
        loop {
            if current.exists() {
                return Some(current.to_path_buf());
            }
            current = current.parent()?;
        }
    }

    /// Same real-write probe `data_dir_check` uses on the data dir itself,
    /// reused on an ancestor directory: permission bits are not the
    /// question, actually creating and removing a file is.
    fn probe_writable(dir: &Path) -> bool {
        let probe = dir.join(format!(".sgt-doctor-writable-probe-{}", std::process::id()));
        match std::fs::write(&probe, b"doctor") {
            Ok(()) => {
                let _ = std::fs::remove_file(&probe);
                true
            }
            Err(_) => false,
        }
    }

    /// §31: the journal opens and validates.
    ///
    /// Read-only, by full validating replay across segments — the same walk
    /// the daemon does at startup, which is what makes a green check mean
    /// "the daemon will start". It never takes the journal's writer lock, so
    /// running this against a live daemon is safe.
    ///
    /// Returns the check, whether the replay succeeded, and — on success —
    /// the events it replayed, so every check derived from the journal
    /// (`projection_check`, `git_surfaces_check`) can fold that same
    /// `Vec<Event>` instead of each re-opening and re-replaying the journal
    /// for itself (#12: one replay shared across doctor's checks). `None`
    /// on both the no-journal and the replay-failure paths — callers that
    /// need to tell those two apart still have the bool for that; this
    /// function keeps sole ownership of the structured `Malformed` detail
    /// either way.
    fn journal_check(data_dir: &Path) -> (Check, bool, Option<Vec<Event>>) {
        let remedy = "the journal is the durable record — do not delete it. Inspect the \
                      reported segment by hand; a torn tail from a crash is quarantined \
                      automatically the next time the daemon opens it";
        if !journal_dir(data_dir).exists() {
            // A data dir nothing has ever run in is healthy, not broken. The
            // daemon creates the journal on its first start; reporting a
            // fresh install as a fault would train operators to ignore this
            // check, which is worse than not having it.
            return (
                Check::ok(
                    "journal",
                    "no journal yet — nothing has run in this data dir",
                ),
                true,
                None,
            );
        }
        let replay = match Journal::replay_data_dir(data_dir) {
            Ok(replay) => replay,
            Err(e) => {
                return (
                    Check::fail("journal", format!("cannot open the journal: {e}"), remedy),
                    false,
                    None,
                );
            }
        };
        let mut events = Vec::new();
        let mut last_seq = 0u64;
        for event in replay {
            match event {
                Ok(event) => {
                    last_seq = event.seq;
                    events.push(event);
                }
                Err(e) => {
                    return (
                        Check::fail(
                            "journal",
                            format!("replay failed after {} events: {e}", events.len()),
                            remedy,
                        ),
                        false,
                        None,
                    );
                }
            }
        }
        (
            Check::ok(
                "journal",
                format!(
                    "{} events replay cleanly (head seq {last_seq})",
                    events.len()
                ),
            ),
            true,
            Some(events),
        )
    }

    /// §31: the disposable projection can be rebuilt from the journal.
    ///
    /// Built into a scratch directory, not the data dir: a live daemon owns
    /// the real DuckDB file, and a diagnostic must not touch state it does
    /// not own. What this proves is the property that matters — the fold from
    /// journal to projection completes — which is exactly what the daemon
    /// does on every start (§40: projections are disposable).
    fn projection_check(journal_ok: bool, events: Option<&[Event]>) -> Check {
        if !journal_ok {
            return Check::warn(
                "projection",
                "not attempted: the journal did not replay",
                "fix the journal check above first",
            );
        }
        let scratch = std::env::temp_dir().join(format!("sgt-doctor-{}", ulid::Ulid::generate()));
        // `events` is `None` on a fresh install with no journal at all, same
        // as `Some(&[])` for a journal that exists but has nothing in it —
        // either way the fold has nothing to read, but the store itself
        // must still open, which is the half of this check that a fresh
        // install can actually be wrong about.
        let outcome = Analytics::rebuild(&scratch, events.unwrap_or(&[]).iter().cloned().map(Ok))
            .map(|analytics| analytics.last_seq())
            .map_err(|e| e.to_string());
        let _ = std::fs::remove_dir_all(&scratch);
        match outcome {
            Ok(last_seq) => Check::ok(
                "projection",
                format!("rebuilds from the journal to seq {last_seq}"),
            ),
            Err(e) => Check::fail(
                "projection",
                format!("rebuild failed: {e}"),
                "the analytical projection is disposable: stop the daemon, delete \
                 `projections/` in the data dir, and start it again. If it still fails, the \
                 journal is the source of truth and nothing has been lost",
            ),
        }
    }

    /// §31: the runtime descriptor and whether a daemon is actually behind it.
    ///
    /// The three states mirror the client's own stale-descriptor policy: no
    /// descriptor is fine (clients spawn on demand); descriptor plus a
    /// healthy endpoint is fine; a descriptor whose PID is alive while the
    /// endpoint refuses is the ambiguous case a client cannot resolve on its
    /// own, so it is the one the doctor calls a failure.
    async fn daemon_check(data_dir: &Path) -> Check {
        let path = daemon::descriptor_path(data_dir);
        let descriptor = match daemon::read_descriptor(data_dir) {
            Ok(Some(descriptor)) => descriptor,
            Ok(None) => {
                return Check::ok(
                    "daemon",
                    "no daemon running; a mutating verb (`run`, `respond`, `retry`, `extend`, \
                     `cancel`) starts one on demand — `status`, `work`, `analytics`, `watch`, \
                     and `tui` refuse instead rather than materializing one just to observe it \
                     (ADR 0009)",
                );
            }
            Err(e) => {
                return Check::fail(
                    "daemon",
                    format!("{} is unreadable: {e}", path.display()),
                    format!(
                        "stop any running daemon and remove {} — it is republished at startup",
                        path.display()
                    ),
                );
            }
        };
        let healthy = match reqwest::Client::builder()
            .timeout(super::HEALTH_TIMEOUT)
            .build()
        {
            Ok(http) => super::healthz_ok(&http, &descriptor.endpoint).await,
            Err(_) => false,
        };
        if healthy {
            return Check::ok(
                "daemon",
                format!(
                    "serving {} (pid {}, api {})",
                    descriptor.endpoint, descriptor.pid, descriptor.api_revision
                ),
            );
        }
        if daemon::pid_alive(descriptor.pid) {
            Check::fail(
                "daemon",
                format!(
                    "pid {} is alive but {} does not answer /healthz",
                    descriptor.pid, descriptor.endpoint
                ),
                format!(
                    "a client will refuse to start a second daemon in this state. Check what \
                     pid {} is; if it is not sergeant, remove {}",
                    descriptor.pid,
                    path.display()
                ),
            )
        } else {
            Check::warn(
                "daemon",
                format!(
                    "descriptor at {} is stale (pid {} is gone)",
                    path.display(),
                    descriptor.pid
                ),
                "harmless: a mutating verb (`run`, `respond`, `retry`, `extend`, `cancel`) or \
                 `sgt daemon` spawns a fresh daemon, which republishes it — observation verbs \
                 refuse instead rather than spawning one just to observe it (ADR 0009)",
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::platform::fs_locking::Reliability;

        /// #85, ADR 0003 D6: a confirmed-bad filesystem is `Fail`, and the
        /// remedy names the filesystem. Reverting the `Unreliable` match arm
        /// to `Check::warn` (or `ok`) fails this immediately, and — because
        /// `healthy_for_init` only exempts `claude`/`docker` — would also
        /// silently stop this check from blocking `sgt init`'s exit code.
        #[test]
        fn a_confirmed_bad_filesystem_is_a_failing_check_naming_the_remedy() {
            let check = check_from_reliability(
                Path::new("/some/data/dir"),
                Reliability::Unreliable {
                    filesystem: "drvfs".to_string(),
                },
            );
            assert_eq!(check.status, Status::Fail);
            assert!(check.detail.contains("drvfs"));
            let remedy = check.remedy.expect("a failing check must name its remedy");
            assert!(remedy.contains("drvfs"));
        }

        /// The fail-closed asymmetry's other half: an inconclusive probe is
        /// `Warn`, never `Fail` — a `Fail` here would gate `healthy_for_init`
        /// and brick `sgt init` on exactly the platform (macOS, today) whose
        /// detection is unmeasured.
        #[test]
        fn an_inconclusive_probe_is_a_warning_never_a_failure() {
            let check = check_from_reliability(
                Path::new("/some/data/dir"),
                Reliability::Unknown {
                    reason: "macOS detection is unmeasured".to_string(),
                },
            );
            assert_eq!(check.status, Status::Warn);
            assert!(check.detail.contains("macOS detection is unmeasured"));
        }

        #[test]
        fn an_ordinary_filesystem_is_ok() {
            let check = check_from_reliability(Path::new("/some/data/dir"), Reliability::Reliable);
            assert_eq!(check.status, Status::Ok);
            assert!(check.remedy.is_none());
        }

        /// §12.2/§18: a scratch data dir, no estate — the row must not even
        /// attempt to open a journal that was never asked for.
        #[test]
        fn git_surfaces_is_silent_outside_an_estate() {
            let check = git_surfaces_check(Path::new("/nonexistent/data/dir"), None, None);
            assert_eq!(check.status, Status::Ok);
            assert_eq!(check.detail, "not an estate root — nothing to check");
        }

        /// §12.2/§18: a fresh estate with a journal but nothing ever run in
        /// it — every count is zero and the row stays `Ok`, same as every
        /// other doctor row on a fresh install.
        #[test]
        fn git_surfaces_on_an_empty_journal_is_all_zero_and_ok() {
            let data_dir = std::env::temp_dir()
                .join(format!("sgt-git-surfaces-test-{}", ulid::Ulid::generate()));
            Journal::open(&data_dir).expect("journal opens");
            let events: Vec<Event> = Journal::replay_data_dir(&data_dir)
                .expect("journal replays")
                .collect::<Result<Vec<_>, _>>()
                .expect("events parse");
            let check =
                git_surfaces_check(&data_dir, Some(Path::new("/some/estate")), Some(&events));
            let _ = std::fs::remove_dir_all(&data_dir);
            assert_eq!(check.status, Status::Ok);
            assert!(check.remedy.is_none());
            for row in [
                "active works:              0",
                "active linked worktrees:   0",
                "retained worktrees:        0",
                "retained patches:          0",
                "retained artifact size:    0 B",
                "journaled Work branches:   0",
                "terminal dirty Works:      0",
            ] {
                assert!(
                    check.detail.contains(row),
                    "missing {row:?} in {:?}",
                    check.detail
                );
            }
        }

        /// §12.2/§18: the full row set, seeded straight into the journal —
        /// no daemon, no real git worktree, exactly the "journal + manifest
        /// + retained-artifact filesystem metadata" the row promises.
        ///
        /// Four works, four different git-surface fates:
        /// - `work-a`: materialized, two bindings, never torn down — an
        ///   *active* work with two active linked worktrees.
        /// - `work-b`: materialized (one binding), torn down clean — a
        ///   durable branch that leaves no residue.
        /// - `work-c`: torn down with a captured, still-on-disk dirty patch
        ///   and a `dirty` integrity verdict — one retained patch, one
        ///   terminal dirty work.
        /// - `work-d`: torn down `retained_error` with the worktree
        ///   directory itself still on disk and a `dirty` verdict — one
        ///   retained worktree, a second terminal dirty work.
        ///
        /// Expected tally: 1 active work, 2 active linked worktrees, 1
        /// retained worktree, 1 retained patch, 2 terminal dirty works, and
        /// 5 journaled Work branches (2 + 1 + 1 + 1 across the four works) —
        /// none of it requiring a single `git` invocation.
        #[test]
        fn git_surfaces_tallies_a_seeded_journal_across_four_works() {
            use crate::domain::event::{EventDraft, EventSource};
            use crate::runtime::surface::{KIND_SURFACE_MATERIALIZED, KIND_SURFACE_TORN_DOWN};

            let data_dir = std::env::temp_dir()
                .join(format!("sgt-git-surfaces-test-{}", ulid::Ulid::generate()));
            std::fs::create_dir_all(&data_dir).expect("scratch data dir");

            // On-disk evidence `retained_bindings` reads live, per its own
            // "report what is actually on disk right now" rule.
            let patch_path = data_dir.join("work-c-patch.diff");
            std::fs::write(&patch_path, b"diff --git a/x b/x\n").expect("write patch fixture");
            let worktree_path = data_dir.join("work-d-worktree");
            std::fs::create_dir_all(&worktree_path).expect("worktree fixture dir");
            std::fs::write(worktree_path.join("dirty.txt"), b"uncommit")
                .expect("worktree fixture file");

            let source = EventSource::new("test", "git_surfaces");
            let mut journal = Journal::open(&data_dir).expect("journal opens");

            let materialized = |repos: &[&str]| {
                json!({
                    "surface": {
                        "work_id": "placeholder",
                        "root": data_dir.join("surfaces/placeholder"),
                        "bindings": repos.iter().map(|r| json!({
                            "repository": r,
                            "source_path": data_dir.join("repos").join(r),
                            "base_sha": "0".repeat(40),
                            "worktree_path": data_dir.join("surfaces/placeholder").join(r),
                            "work_branch": format!("sergeant/{r}"),
                            "head_sha": "0".repeat(40),
                        })).collect::<Vec<_>>(),
                    }
                })
            };

            // work-a: two bindings, active (no teardown).
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_MATERIALIZED,
                        materialized(&["one", "two"]),
                    )
                    .with_work_id("work-a"),
                )
                .expect("append work-a materialized");

            // work-b: one binding, torn down clean.
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_MATERIALIZED,
                        materialized(&["solo"]),
                    )
                    .with_work_id("work-b"),
                )
                .expect("append work-b materialized");
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_TORN_DOWN,
                        json!({
                            "report": {
                                "work_id": "work-b",
                                "clean": true,
                                "bindings": [{
                                    "repository": "solo",
                                    "worktree_path": data_dir.join("surfaces/work-b/solo"),
                                    "work_branch": "sergeant/work-b",
                                    "disposition": "removed",
                                }],
                            },
                            "integrity": "clean",
                        }),
                    )
                    .with_work_id("work-b"),
                )
                .expect("append work-b torn down");

            // work-c: torn down with a retained, on-disk patch; dirty.
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_MATERIALIZED,
                        materialized(&["patched"]),
                    )
                    .with_work_id("work-c"),
                )
                .expect("append work-c materialized");
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_TORN_DOWN,
                        json!({
                            "report": {
                                "work_id": "work-c",
                                "clean": false,
                                "bindings": [{
                                    "repository": "patched",
                                    "worktree_path": data_dir.join("surfaces/work-c/patched"),
                                    "work_branch": "sergeant/work-c",
                                    "disposition": "retained_dirty",
                                    "changes": " M dirty.rs",
                                    "patch": {"path": patch_path, "bytes": 19},
                                }],
                            },
                            "integrity": "dirty",
                        }),
                    )
                    .with_work_id("work-c"),
                )
                .expect("append work-c torn down");

            // work-d: torn down `retained_error`, worktree dir still on
            // disk; dirty.
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_MATERIALIZED,
                        materialized(&["stuck"]),
                    )
                    .with_work_id("work-d"),
                )
                .expect("append work-d materialized");
            journal
                .append(
                    EventDraft::new(
                        source.clone(),
                        KIND_SURFACE_TORN_DOWN,
                        json!({
                            "report": {
                                "work_id": "work-d",
                                "clean": false,
                                "bindings": [{
                                    "repository": "stuck",
                                    "worktree_path": worktree_path,
                                    "work_branch": "sergeant/work-d",
                                    "disposition": "retained_error",
                                    "detail": "worktree remove refused: bare repository",
                                }],
                            },
                            "integrity": "dirty",
                        }),
                    )
                    .with_work_id("work-d"),
                )
                .expect("append work-d torn down");
            drop(journal);

            let events: Vec<Event> = Journal::replay_data_dir(&data_dir)
                .expect("journal replays")
                .collect::<Result<Vec<_>, _>>()
                .expect("events parse");
            let check =
                git_surfaces_check(&data_dir, Some(Path::new("/some/estate")), Some(&events));
            let _ = std::fs::remove_dir_all(&data_dir);

            assert_eq!(
                check.status,
                Status::Warn,
                "residue must not read as Ok: {check:?}"
            );
            let remedy = check.remedy.expect("nonzero residue must name a remedy");
            assert!(remedy.contains("sgt work retained"), "remedy: {remedy}");
            assert!(remedy.contains("sgt work reap"), "remedy: {remedy}");

            for row in [
                "active works:              1",
                "active linked worktrees:   2",
                "retained worktrees:        1",
                "retained patches:          1",
                "journaled Work branches:   5",
                "terminal dirty Works:      2",
            ] {
                assert!(
                    check.detail.contains(row),
                    "missing {row:?} in {:?}",
                    check.detail
                );
            }
            // 19 (the patch's recorded size) + 8 ("uncommit"'s live directory
            // size) = 27 bytes total, well under a KiB — `human_bytes` still
            // renders it in bytes.
            assert!(
                check.detail.contains("retained artifact size:    27 B"),
                "{:?}",
                check.detail
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #96: a human-rendered transcript must show *when* each turn
    /// happened, not just what it said — the exact gap that let a
    /// ceiling-interrupted turn (#90) read as still in progress.
    /// Reverting `render_transcript` to drop the `[{ts}]` prefix passes
    /// every other assertion here and fails only this one.
    #[test]
    fn render_transcript_prints_each_turns_timestamp() {
        let result = json!({
            "turns": [
                {
                    "seq": 1,
                    "ts": "2026-08-14T15:24:00.000Z",
                    "role": "user",
                    "text": "let's wait for both background runs to complete",
                    "source": "event",
                },
                {
                    "seq": 2,
                    "ts": "2026-08-14T15:26:55.874Z",
                    "role": "assistant",
                    "text": "still waiting",
                    "source": "event",
                },
            ],
        });
        let rendered = render_transcript(&result);
        assert!(
            rendered.contains("[2026-08-14T15:24:00.000Z] User:"),
            "missing user timestamp: {rendered}"
        );
        assert!(
            rendered.contains("[2026-08-14T15:26:55.874Z] Assistant:"),
            "missing assistant timestamp: {rendered}"
        );
    }

    /// A turn recovered from the interrupted raw archive still needs its
    /// timestamp — that is the exact turn #90's transcript misdiagnosis
    /// involved (a `blob_decode` entry from a ceiling-interrupted turn).
    #[test]
    fn render_transcript_timestamps_a_recovered_turn_too() {
        let result = json!({
            "turns": [{
                "seq": 3,
                "ts": "2026-08-14T15:26:55.874Z",
                "role": "assistant",
                "text": "partial output before the cut",
                "source": "blob_decode",
                "interrupted": true,
            }],
        });
        let rendered = render_transcript(&result);
        assert!(
            rendered.contains(
                "[2026-08-14T15:26:55.874Z] Assistant (recovered from the interrupted turn's raw archive):"
            ),
            "missing recovered-turn timestamp: {rendered}"
        );
    }

    #[test]
    fn render_transcript_reports_no_conversation_without_a_timestamp_line() {
        let rendered = render_transcript(&json!({"turns": []}));
        assert_eq!(rendered, "no conversation recorded for this work\n");
    }

    fn listed_work(state: &str, disposition: Option<&str>) -> Value {
        let mut work = json!({"id": "w-1", "state": state, "intent": "do the thing"});
        if let Some(disposition) = disposition {
            work["integrity"] = json!({"disposition": disposition});
        }
        work
    }

    /// C5's acceptance criterion for the two terminal states §11.5 mints no
    /// compact label for: a dirty `failed` or `canceled` Work has to be
    /// distinguishable in `sgt work list`'s *default* output, not only under
    /// `--json`. Garbling or inverting the suffix branch in
    /// `list_state_label` fails here and nowhere else — every other
    /// dirty-work assertion in the suite reads the `/v1/work` JSON body.
    #[test]
    fn list_state_label_appends_the_integrity_axis_to_failed_and_canceled() {
        assert_eq!(
            list_state_label(&listed_work("failed", Some("dirty"))),
            "failed/dirty"
        );
        assert_eq!(
            list_state_label(&listed_work("canceled", Some("dirty"))),
            "canceled/dirty"
        );
    }

    /// The other half of the branch: `completed_dirty` is minted by the API's
    /// own `reported_state`, so the column must not say it twice.
    #[test]
    fn list_state_label_does_not_re_suffix_a_state_already_carrying_the_axis() {
        assert_eq!(
            list_state_label(&listed_work("completed_dirty", Some("dirty"))),
            "completed_dirty"
        );
    }

    /// A clean or not-assessed Work keeps its true state string — absent
    /// integrity is "not assessed" (C3), never a dirty rendering.
    #[test]
    fn list_state_label_leaves_clean_and_unassessed_states_alone() {
        assert_eq!(
            list_state_label(&listed_work("completed", Some("clean"))),
            "completed"
        );
        assert_eq!(list_state_label(&listed_work("failed", None)), "failed");
        assert_eq!(list_state_label(&listed_work("running", None)), "running");
    }
}
