//! `sgt` CLI (proposal §31 subset, deviation D1): clap parsing, daemon
//! auto-spawn, and a thin HTTP client over the daemon's v1 API.
//!
//! The CLI never touches runtime state directly — every command except
//! `sgt daemon` goes through the daemon's loopback API. When no daemon is
//! running, the client spawns one detached (`sgt --data-dir <dir> daemon`),
//! waits for the runtime descriptor plus a passing `/healthz`, and proceeds.
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
    /// Data directory (default: $SGT_DATA_DIR, then ~/.local/share/sergeant).
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    /// Emit machine-readable JSON instead of human text.
    #[arg(long, global = true)]
    json: bool,
    /// Subcommand to run. Omitted, `sgt` opens the TUI (§30).
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
        /// Workflow to run (default: the workspace's, else `software-change`).
        #[arg(long)]
        workflow: Option<String>,
        /// Backend to run on (§13's explicit tier).
        #[arg(long)]
        backend: Option<String>,
        /// Launch profile (§14).
        #[arg(long)]
        profile: Option<String>,
        /// Targeted repository (repeatable).
        #[arg(long = "repo")]
        repositories: Vec<String>,
        /// A declared `[group.<name>]`'s repositories, expanded client-side
        /// into the same selection `--repo` builds (R-MVP1-5(b): group
        /// membership gets no new engine surface — this is pure CLI-side
        /// flag expansion over the existing `repositories` field). Combines
        /// with any `--repo` flags given alongside it (union, declaration
        /// order, duplicates dropped).
        #[arg(long)]
        group: Option<String>,
        /// Workspace to scope the work to.
        #[arg(long)]
        workspace: Option<String>,
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
    /// only changes what the next retry is checked against).
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
    /// Print the embedded dashboard's URL, token included (§29).
    Web {
        /// Also open it in a browser (`$BROWSER`, else `xdg-open`/`open`).
        #[arg(long)]
        open: bool,
    },
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

impl From<crate::domain::workspace::WorkspaceError> for CliError {
    fn from(e: crate::domain::workspace::WorkspaceError) -> Self {
        Self(e.to_string())
    }
}

impl From<crate::domain::manifest::ManifestError> for CliError {
    fn from(e: crate::domain::manifest::ManifestError) -> Self {
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

/// Resolve the data dir: `--data-dir` flag, `SGT_DATA_DIR` — both unchanged,
/// unconditional precedence — then (U-R2, MVP-3's estate-resolved default) an
/// estate discovered by walking upward from the current directory, mirroring
/// R-MVP1-12: filesystem-first, crossing git boundaries, bounded at `$HOME`.
/// When one is found, the default is `<estate_root>/.sergeant/data` — the
/// same path `sgt init` scaffolds a `.gitignore` entry for
/// ([`crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR`]). This is a new
/// fallback *rung*, not a replacement: only once no estate is found (or the
/// current directory cannot even be read) does resolution fall through to
/// the pre-estate default, `$XDG_DATA_HOME/sergeant` or
/// `~/.local/share/sergeant`, unchanged.
fn resolve_data_dir(flag: Option<PathBuf>) -> Result<PathBuf, CliError> {
    if let Some(dir) = flag {
        return Ok(dir);
    }
    if let Some(dir) = std::env::var_os("SGT_DATA_DIR") {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(cwd) = std::env::current_dir()
        && let Some(estate_root) = crate::domain::workspace::Workspace::estate_root(&cwd, None)?
    {
        return Ok(estate_root.join(crate::domain::manifest::DEFAULT_ESTATE_DATA_DIR));
    }
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return Ok(PathBuf::from(xdg).join("sergeant"));
    }
    match std::env::var_os("HOME") {
        Some(home) => Ok(PathBuf::from(home).join(".local/share/sergeant")),
        None => Err(CliError::new(
            "cannot resolve data dir: set --data-dir, SGT_DATA_DIR, or HOME",
        )),
    }
}

async fn dispatch(sgt: Sgt) -> Result<(), CliError> {
    let data_dir = resolve_data_dir(sgt.data_dir)?;
    let Some(command) = sgt.command else {
        // §30: bare `sgt` is the TUI. It is a client like any other, so it
        // gets the same auto-spawn path the CLI commands get.
        let client = ensure_daemon(&data_dir).await?;
        return crate::tui::run(client).await.map_err(CliError::from);
    };
    match command {
        Command::Daemon { command: None } => {
            tracing_subscriber::fmt().init();
            daemon::run_until_signal(&data_dir).await?;
            Ok(())
        }
        Command::Daemon {
            command: Some(DaemonCommand::Stop),
        } => daemon_stop(&data_dir, sgt.json).await,
        Command::Status => {
            let client = ensure_daemon(&data_dir).await?;
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
            mut repositories,
            group,
            workspace,
            turns,
            ceiling_secs,
        } => {
            // R-MVP1-5(b): group membership gets no new engine surface —
            // `--group` is pure CLI-side expansion into the same
            // `repositories` selection `--repo` already builds, over the
            // estate discovered from the current directory (bounded at this
            // daemon's own data dir, mirroring every other client-side
            // workspace read in this binary).
            //
            // MVP-3 invariants finding MVP3-C2: this reads group membership
            // through the on-disk-free structural parser
            // (`declared_groups_scoped`), not the strict `discover_scoped`
            // (which resolves every declared `[[repo]]` through git) — group
            // membership is just declared names, so an unrelated missing
            // repository must not block a group whose own members are all
            // fine, the same coupling `domain::manifest`'s edit pens no
            // longer have (MVP3-C1).
            if let Some(group_name) = &group {
                let cwd = std::env::current_dir()?;
                let groups = crate::domain::workspace::Workspace::declared_groups_scoped(
                    &cwd,
                    Some(&data_dir),
                )?;
                let members = groups.get(group_name).ok_or_else(|| {
                    let available: Vec<&str> = groups.keys().map(String::as_str).collect();
                    CliError::new(format!(
                        "no group {group_name:?} declared in this estate (declared: {}); \
                         declare it first with `sgt group add {group_name} <repo>...`",
                        if available.is_empty() {
                            "none".to_string()
                        } else {
                            available.join(", ")
                        }
                    ))
                })?;
                for repo in &members.repos {
                    if !repositories.contains(repo) {
                        repositories.push(repo.clone());
                    }
                }
            }
            let client = ensure_daemon(&data_dir).await?;
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
                "repositories": repositories,
                "workspace": workspace,
                "envelope": envelope,
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
            let client = ensure_daemon(&data_dir).await?;
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
            let client = ensure_daemon(&data_dir).await?;
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
            let client = ensure_daemon(&data_dir).await?;
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
        Command::Work { command } => {
            let client = ensure_daemon(&data_dir).await?;
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
                                        work["state"].as_str().unwrap_or("?"),
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
            }
        }
        Command::Analytics { name } => {
            let client = ensure_daemon(&data_dir).await?;
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
            let client = ensure_daemon(&data_dir).await?;
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
        Command::Web { open } => {
            // §29's handoff. `sgt web` is a client command like any other, so
            // it auto-spawns the daemon: asking for the dashboard of a daemon
            // that is not running should start it, not lecture about it.
            let client = ensure_daemon(&data_dir).await?;
            let url = client.dashboard_url();
            if sgt.json {
                print_json(&json!({"url": url, "endpoint": client.endpoint()}));
            } else {
                println!("{url}");
                println!(
                    "the token is in that URL — it is a secret; loopback only, \
                     and it changes every time the daemon restarts"
                );
            }
            if open {
                open_in_browser(&url)?;
            }
            Ok(())
        }
        Command::Doctor => {
            let report = doctor::run(&data_dir).await;
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
            let cwd = std::env::current_dir()?;
            let outcome = crate::domain::manifest::init_estate(&cwd, name.as_deref())?;
            let report = doctor::run(&data_dir).await;
            if sgt.json {
                print_json(&json!({
                    "outcome": {
                        "manifest_created": outcome.manifest_created,
                        "estate_section_added": outcome.estate_section_added,
                        "repos_dir_created": outcome.repos_dir_created,
                        "gitignore_updated": outcome.gitignore_updated,
                        "changed": outcome.changed(),
                    },
                    "doctor": report.to_json(),
                }));
            } else {
                if outcome.changed() {
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
                } else {
                    println!(
                        "estate at {} is already initialized — nothing to do",
                        cwd.display()
                    );
                }
                println!();
                report.print();
            }
            if report.healthy() {
                Ok(())
            } else {
                Err(CliError::silent())
            }
        }
        Command::Repo { command } => repo_command(sgt.json, &data_dir, command).await,
        Command::Group { command } => group_command(sgt.json, &data_dir, command).await,
    }
}

/// The estate root every `sgt repo`/`sgt group` verb edits: discovered from
/// the current directory (R-MVP1-12's own walk, bounded at `$HOME` and at
/// this daemon's own data dir), never created by these verbs — that is
/// `sgt init`'s job.
fn discover_estate_root(data_dir: &Path) -> Result<PathBuf, CliError> {
    let cwd = std::env::current_dir()?;
    crate::domain::workspace::Workspace::estate_root(&cwd, Some(data_dir))?.ok_or_else(|| {
        CliError::new(
            "no estate found above the current directory (bounded at $HOME) — run `sgt init` \
             first, at the directory that should become the estate root",
        )
    })
}

/// `sgt repo add/remove/list` (MVP-3): a pure manifest edit/read, no daemon
/// involved — repository declarations are estate topology (§9), not runtime
/// state.
async fn repo_command(json: bool, data_dir: &Path, command: RepoCommand) -> Result<(), CliError> {
    let estate_root = discover_estate_root(data_dir)?;
    match command {
        RepoCommand::Add {
            name,
            origin,
            instructions,
        } => {
            let policy = match instructions.as_deref() {
                None => None,
                Some("local") => Some(crate::domain::workspace::InstructionPolicy::Local),
                Some("suppress") => Some(crate::domain::workspace::InstructionPolicy::Suppress),
                Some(other) => {
                    return Err(CliError::new(format!(
                        "--instructions {other:?} is not recognized (use \"local\" or \"suppress\")"
                    )));
                }
            };
            crate::domain::manifest::add_repo(&estate_root, &name, origin.as_deref(), policy)?;
            if json {
                print_json(&json!({"added": name}));
            } else {
                println!(
                    "added repo {name} at {}",
                    estate_root.join("repos").join(&name).display()
                );
            }
            Ok(())
        }
        RepoCommand::Remove { name } => {
            crate::domain::manifest::remove_repo(&estate_root, &name)?;
            if json {
                print_json(&json!({"removed": name}));
            } else {
                println!("removed repo {name}");
            }
            Ok(())
        }
        RepoCommand::List => {
            let workspace = crate::domain::workspace::Workspace::from_config_allow_empty(
                &estate_root.join(crate::domain::workspace::WORKSPACE_FILE),
            )?;
            if json {
                print_json(&json!({
                    "repositories": workspace.repositories.iter().map(|r| json!({
                        "name": r.name,
                        "path": r.path,
                        "instructions": workspace.instruction_policy(&r.name).as_str(),
                        "origin": workspace.repository_origin(&r.name),
                    })).collect::<Vec<_>>(),
                }));
            } else if workspace.repositories.is_empty() {
                println!("no repositories declared");
            } else {
                for r in &workspace.repositories {
                    println!(
                        "{}  {}  instructions={}  origin={}",
                        r.name,
                        r.path.display(),
                        workspace.instruction_policy(&r.name),
                        workspace.repository_origin(&r.name).unwrap_or("-"),
                    );
                }
            }
            Ok(())
        }
    }
}

/// `sgt group add/remove/list` (MVP-3): a pure manifest edit/read, same
/// rationale as [`repo_command`].
async fn group_command(json: bool, data_dir: &Path, command: GroupCommand) -> Result<(), CliError> {
    let estate_root = discover_estate_root(data_dir)?;
    match command {
        GroupCommand::Add { name, repos, brief } => {
            crate::domain::manifest::add_group(&estate_root, &name, &repos, brief.as_deref())?;
            if json {
                print_json(&json!({"group": name}));
            } else {
                println!("group {name} updated");
            }
            Ok(())
        }
        GroupCommand::Remove { name, repos } => {
            crate::domain::manifest::remove_group(&estate_root, &name, &repos)?;
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
            let workspace = crate::domain::workspace::Workspace::from_config_allow_empty(
                &estate_root.join(crate::domain::workspace::WORKSPACE_FILE),
            )?;
            if json {
                print_json(&json!({
                    "groups": workspace.groups.iter().map(|(name, g)| json!({
                        "name": name,
                        "repos": g.repos,
                        "brief": g.brief,
                    })).collect::<Vec<_>>(),
                }));
            } else if workspace.groups.is_empty() {
                println!("no groups declared");
            } else {
                for (name, g) in &workspace.groups {
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

/// Hand a URL to the user's browser: `$BROWSER` if set, else the platform
/// opener. A failure here is reported, never silent — the URL has already
/// been printed, so the user still has it.
fn open_in_browser(url: &str) -> Result<(), CliError> {
    let opener = std::env::var("BROWSER").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "open".to_string()
        } else {
            "xdg-open".to_string()
        }
    });
    let status = std::process::Command::new(&opener)
        .arg(url)
        .status()
        .map_err(|e| CliError::new(format!("cannot run {opener}: {e}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(CliError::new(format!("{opener} exited with {status}")))
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
    let empty = Vec::new();
    let turns = result["turns"].as_array().unwrap_or(&empty);
    if turns.is_empty() {
        println!("no conversation recorded for this work");
        return;
    }
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
        println!("{label}{recovered}:");
        println!("{}", turn["text"].as_str().unwrap_or(""));
        println!();
    }
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
/// the directory workspace discovery should start from. The client owns the
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

/// Connect to the daemon for `data_dir`, auto-spawning one if needed.
///
/// This is the CLI's half of the client contract — descriptor discovery,
/// staleness judgement, detached spawn — and it hands back the crate's one
/// API client ([`ApiClient`], defined next to the router it speaks to). Every
/// front end in this binary, TUI included, comes through here.
async fn ensure_daemon(data_dir: &Path) -> Result<ApiClient, CliError> {
    let http = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let stale = if let Some(descriptor) = daemon::read_descriptor(data_dir)? {
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

    spawn_daemon(data_dir)?;

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
            && healthz_ok(&http, &descriptor.endpoint).await
        {
            return client_for(&descriptor);
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
fn spawn_daemon(data_dir: &Path) -> Result<(), CliError> {
    std::fs::create_dir_all(data_dir)?;
    let exe = std::env::current_exe()?;
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("daemon.log"))?;
    let mut command = std::process::Command::new(exe);
    command
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
async fn daemon_stop(data_dir: &Path, json: bool) -> Result<(), CliError> {
    let Some(descriptor) = daemon::read_descriptor(data_dir)? else {
        report_daemon_stop(
            json,
            "not_running",
            "no daemon is running for this data dir",
        );
        return Ok(());
    };
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
/// Doctor deliberately does **not** auto-spawn a daemon. Every other client
/// command starts one on demand; this one is asking whether the installation
/// is sound, and starting the thing under examination would answer a
/// different question.
mod doctor {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use serde_json::{Value, json};

    use crate::backend::Backend;
    use crate::backend::claude::{
        CLAUDE_BIN_ENV, ClaudeBackend, ClaudeConfig, MIN_TRUSTED_VERSION,
    };
    use crate::backend::docker::{self, DockerBackend, DockerConfig};
    use crate::daemon;
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
        /// check that is not `Ok`.
        pub fn print(&self) {
            println!("sergeant doctor — {}", self.data_dir.display());
            for check in &self.checks {
                println!(
                    "  [{}] {:<12} {}",
                    check.status.marker(),
                    check.name,
                    check.detail
                );
                if let Some(remedy) = &check.remedy {
                    println!("         remedy: {remedy}");
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

    /// Run every check against `data_dir`.
    pub async fn run(data_dir: &Path) -> Report {
        let mut checks = vec![
            git_check(),
            claude_check(data_dir),
            docker_check(data_dir),
            data_dir_check(data_dir),
        ];
        // The journal is the only durable fact in the installation, so it is
        // checked before anything derived from it; a projection rebuild over
        // an unreadable journal would report the journal's fault under the
        // projection's name.
        let (journal_check, journal_ok) = journal_check(data_dir);
        checks.push(journal_check);
        checks.push(projection_check(data_dir, journal_ok));
        checks.push(daemon_check(data_dir).await);
        checks.push(permission_mode_check(data_dir));
        checks.push(estate_check(data_dir));
        // N4/#23 (retention Rule B): disk pressure inside the data dir. Runs
        // after everything above regardless of their outcome — knowing "is
        // this installation about to run out of disk" does not depend on the
        // daemon, the journal, or Docker being reachable.
        checks.push(disk_pressure_check(data_dir));
        Report {
            data_dir: data_dir.to_path_buf(),
            checks,
        }
    }

    /// §31, #47: the effective `--permission-mode` behavior each declared
    /// profile launches with — the same question `a_profile_is_launch_
    /// configuration_carried_to_the_claude_adapter` pins in code, surfaced
    /// where an operator reading `sgt doctor` will actually see it.
    ///
    /// Workspace discovery, not the data dir: profiles live in
    /// `sergeant.toml` at the repository the doctor is *run from*, which is
    /// also why a malformed `permission_mode` here reads as this check's own
    /// failure rather than a mysterious daemon-side refusal later — #47's
    /// fail-closed load already ran by the time this string is built.
    fn permission_mode_check(data_dir: &Path) -> Check {
        use crate::domain::workspace::{Workspace, WorkspaceError};

        let cwd = match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(e) => {
                return Check::warn(
                    "permission_mode",
                    format!("cannot read the current directory: {e}"),
                    "run `sgt doctor` from inside the workspace whose profiles you want reported",
                );
            }
        };
        // R-MVP1-12's data-dir scope: bound the upward estate walk at this
        // installation's own data dir, never above it.
        match Workspace::discover_scoped(&cwd, Some(data_dir)) {
            Ok(workspace) if workspace.profiles.is_empty() => Check::ok(
                "permission_mode",
                "no profiles declared — every execution launches with no --permission-mode \
                 flag at all (the CLI's own default)",
            ),
            Ok(workspace) => {
                let modes: Vec<String> = workspace
                    .profiles
                    .iter()
                    .map(|p| {
                        // Already validated at load (#47): from_config would
                        // have refused the workspace before this ran.
                        let effective = match p.permission_mode().ok().flatten() {
                            Some(mode) => mode.as_cli_value().to_string(),
                            None => "unspecified -> no flag (CLI default)".to_string(),
                        };
                        format!("{}={effective}", p.name)
                    })
                    .collect();
                Check::ok("permission_mode", modes.join(", "))
            }
            Err(WorkspaceError::NotARepository { .. }) => Check::ok(
                "permission_mode",
                "not inside a workspace — nothing to report",
            ),
            Err(e) => Check::warn(
                "permission_mode",
                format!("cannot read this workspace's profiles: {e}"),
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
    /// `WorkspaceError` variant already names its file and the offending
    /// key, and `Malformed` additionally carries `toml::de::Error`'s own
    /// line/column, so nothing here needs to reconstruct that.
    ///
    /// Once the manifest parses, this looks past what execution's own
    /// strict loader (`Workspace::from_config`) would ever tell you, because
    /// that loader fails closed at the *first* problem it finds — right for
    /// launching a Work, useless for "what is wrong with my estate right
    /// now": it uses [`crate::domain::workspace::Workspace::declared_repos`]
    /// instead, which names every declared repository regardless of whether
    /// earlier ones are missing, then cross-checks two directions —
    /// declared-but-absent (remedy: the declared `origin`, when there is
    /// one, else the `sgt repo add` command that would set one) and
    /// present-but-undeclared (a directory under `repos/` no `[[repo]]`
    /// entry names).
    fn estate_check(data_dir: &Path) -> Check {
        use crate::domain::workspace::{WORKSPACE_FILE, Workspace};

        let cwd = match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(e) => {
                return Check::warn(
                    "estate",
                    format!("cannot read the current directory: {e}"),
                    "run `sgt doctor` from inside the estate you want checked",
                );
            }
        };
        let estate_root = match Workspace::estate_root(&cwd, Some(data_dir)) {
            Ok(root) => root,
            Err(e) => {
                return Check::fail(
                    "estate",
                    e.to_string(),
                    "fix sergeant.toml at the file and location named above",
                );
            }
        };
        let Some(estate_root) = estate_root else {
            return Check::ok("estate", "not inside an estate — nothing to check");
        };
        let manifest_path = estate_root.join(WORKSPACE_FILE);
        let declared = match Workspace::declared_repos(&manifest_path) {
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

        let repos_dir = estate_root.join("repos");
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
                    "repos/{name} is on disk but not declared in {WORKSPACE_FILE}"
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
    fn docker_check(data_dir: &Path) -> Check {
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
    fn disk_pressure_check(data_dir: &Path) -> Check {
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
    /// losslessly.
    fn human_bytes(bytes: u64) -> String {
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

    /// §31: the data dir exists and this user can write it.
    ///
    /// Probed by actually creating and removing a file. Permission *bits* are
    /// not the question — read-only mounts, full disks and SELinux all answer
    /// "writable" by inspection and "no" in practice.
    fn data_dir_check(data_dir: &Path) -> Check {
        let remedy = format!(
            "create {} and make it writable by this user (or point --data-dir / SGT_DATA_DIR \
             somewhere that is)",
            data_dir.display()
        );
        if let Err(e) = std::fs::create_dir_all(data_dir) {
            return Check::fail(
                "data_dir",
                format!("cannot create {}: {e}", data_dir.display()),
                remedy,
            );
        }
        let probe = data_dir.join(".doctor-write-probe");
        match std::fs::write(&probe, b"doctor") {
            Ok(()) => {
                let _ = std::fs::remove_file(&probe);
                Check::ok("data_dir", format!("{} is writable", data_dir.display()))
            }
            Err(e) => Check::fail(
                "data_dir",
                format!("cannot write inside {}: {e}", data_dir.display()),
                remedy,
            ),
        }
    }

    /// §31: the journal opens and validates.
    ///
    /// Read-only, by full validating replay across segments — the same walk
    /// the daemon does at startup, which is what makes a green check mean
    /// "the daemon will start". It never takes the journal's writer lock, so
    /// running this against a live daemon is safe.
    ///
    /// Returns the check and whether the replay succeeded, so the projection
    /// check below can say "not attempted" instead of blaming itself.
    fn journal_check(data_dir: &Path) -> (Check, bool) {
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
            );
        }
        let replay = match Journal::replay_data_dir(data_dir) {
            Ok(replay) => replay,
            Err(e) => {
                return (
                    Check::fail("journal", format!("cannot open the journal: {e}"), remedy),
                    false,
                );
            }
        };
        let mut count = 0u64;
        let mut last_seq = 0u64;
        for event in replay {
            match event {
                Ok(event) => {
                    count += 1;
                    last_seq = event.seq;
                }
                Err(e) => {
                    return (
                        Check::fail(
                            "journal",
                            format!("replay failed after {count} events: {e}"),
                            remedy,
                        ),
                        false,
                    );
                }
            }
        }
        (
            Check::ok(
                "journal",
                format!("{count} events replay cleanly (head seq {last_seq})"),
            ),
            true,
        )
    }

    /// §31: the disposable projection can be rebuilt from the journal.
    ///
    /// Built into a scratch directory, not the data dir: a live daemon owns
    /// the real DuckDB file, and a diagnostic must not touch state it does
    /// not own. What this proves is the property that matters — the fold from
    /// journal to projection completes — which is exactly what the daemon
    /// does on every start (§40: projections are disposable).
    fn projection_check(data_dir: &Path, journal_ok: bool) -> Check {
        if !journal_ok {
            return Check::warn(
                "projection",
                "not attempted: the journal did not replay",
                "fix the journal check above first",
            );
        }
        let scratch = std::env::temp_dir().join(format!("sgt-doctor-{}", ulid::Ulid::generate()));
        let outcome = if journal_dir(data_dir).exists() {
            Journal::replay_data_dir(data_dir)
                .map_err(|e| e.to_string())
                .and_then(|replay| {
                    Analytics::rebuild(&scratch, replay)
                        .map(|analytics| analytics.last_seq())
                        .map_err(|e| e.to_string())
                })
        } else {
            // No journal yet: the fold has nothing to read, but the store
            // itself must still open — which is the half of this check that
            // a fresh install can actually be wrong about.
            Analytics::rebuild(&scratch, std::iter::empty())
                .map(|analytics| analytics.last_seq())
                .map_err(|e| e.to_string())
        };
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
                    "no daemon running; the next client command starts one",
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
                "harmless: the next client command spawns a daemon, which republishes it",
            )
        }
    }
}
