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
    /// Subcommand to run.
    #[command(subcommand)]
    command: Command,
}

/// Top-level `sgt` subcommands (§31 subset).
#[derive(Subcommand, Debug)]
enum Command {
    /// Run the daemon in the foreground until SIGINT/SIGTERM.
    Daemon,
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
        /// Workspace to scope the work to.
        #[arg(long)]
        workspace: Option<String>,
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
}

/// CLI error: a message for stderr and a nonzero exit.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
struct CliError(String);

impl CliError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

impl From<daemon::DaemonError> for CliError {
    fn from(e: daemon::DaemonError) -> Self {
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
        Err(e) => {
            eprintln!("sgt: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Resolve the data dir: `--data-dir` flag, `SGT_DATA_DIR`, then
/// `$XDG_DATA_HOME/sergeant` or `~/.local/share/sergeant`.
fn resolve_data_dir(flag: Option<PathBuf>) -> Result<PathBuf, CliError> {
    if let Some(dir) = flag {
        return Ok(dir);
    }
    if let Some(dir) = std::env::var_os("SGT_DATA_DIR") {
        return Ok(PathBuf::from(dir));
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
    match sgt.command {
        Command::Daemon => {
            tracing_subscriber::fmt().init();
            daemon::run_until_signal(&data_dir).await?;
            Ok(())
        }
        Command::Status => {
            let client = Client::ensure_daemon(&data_dir).await?;
            let system = client.get("/v1/system").await?;
            let works = client.get("/v1/work").await?;
            let mut counts = std::collections::BTreeMap::<String, u64>::new();
            let mut total = 0u64;
            if let Some(list) = works["works"].as_array() {
                for work in list {
                    if let Some(state) = work["state"].as_str() {
                        *counts.entry(state.to_string()).or_insert(0) += 1;
                        total += 1;
                    }
                }
            }
            if sgt.json {
                print_json(
                    &json!({"system": system, "work_total": total, "work_by_state": counts}),
                );
            } else {
                println!(
                    "daemon ok — version {} api {} data dir {}",
                    system["version"].as_str().unwrap_or("?"),
                    system["api_revision"].as_str().unwrap_or("?"),
                    system["data_dir"].as_str().unwrap_or("?"),
                );
                println!("work: {total} total");
                for (state, n) in counts {
                    println!("  {state}: {n}");
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
            workspace,
        } => {
            let client = Client::ensure_daemon(&data_dir).await?;
            let body = json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "intent": intent,
                "workflow": workflow,
                "backend": backend,
                "profile": profile,
                "repositories": repositories,
                "workspace": workspace,
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
            let client = Client::ensure_daemon(&data_dir).await?;
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
            let client = Client::ensure_daemon(&data_dir).await?;
            let body = json!({"command_id": ulid::Ulid::generate().to_string()});
            let result = client.post(&format!("/v1/work/{id}/retry"), &body).await?;
            if sgt.json {
                print_json(&result);
            } else {
                print_work_line("retried", &result);
            }
            Ok(())
        }
        Command::Work { command } => {
            let client = Client::ensure_daemon(&data_dir).await?;
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
                            for key in ["stage", "surface", "execution", "workflow", "backend"] {
                                if !result[key].is_null() {
                                    object.insert(key.to_string(), result[key].clone());
                                }
                            }
                        }
                        print_json(&work);
                    }
                    Ok(())
                }
            }
        }
        Command::Analytics { name } => {
            let client = Client::ensure_daemon(&data_dir).await?;
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
            let client = Client::ensure_daemon(&data_dir).await?;
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

/// Thin authenticated HTTP client over the daemon's v1 API.
struct Client {
    http: reqwest::Client,
    endpoint: String,
    token: String,
}

impl Client {
    /// Connect to the daemon for `data_dir`, auto-spawning one if needed.
    async fn ensure_daemon(data_dir: &Path) -> Result<Self, CliError> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()?;

        let stale = if let Some(descriptor) = daemon::read_descriptor(data_dir)? {
            if healthz_ok(&http, &descriptor.endpoint).await {
                return Ok(Self::from_descriptor(http, descriptor));
            }
            if daemon::pid_alive(descriptor.pid) {
                // Ambiguous: something with that PID is alive but the
                // endpoint does not answer. Spawning a second daemon here
                // could race a slow-but-live one; fail closed instead.
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
            // between judging the file stale and removing it, a daemon
            // another client just spawned can publish its fresh descriptor at
            // the same path, and unlinking that would leave a healthy,
            // lock-holding daemon permanently undiscoverable (it only writes
            // the descriptor at startup). Leaving the stale bytes in place
            // costs nothing — the wait loop below never accepts a descriptor
            // that does not answer `/healthz`.
            Some(descriptor)
        } else {
            None
        };

        spawn_daemon(data_dir)?;

        // Wait for a healthy descriptor. It may be written by our child or
        // by a concurrently racing client's child — either is fine; the
        // daemon lock guarantees at most one winner. The descriptor already
        // judged stale is skipped by identity rather than re-probed, so a
        // stale endpoint that hangs instead of refusing cannot eat the wait
        // budget one health timeout at a time.
        let deadline = Instant::now() + SPAWN_WAIT;
        while Instant::now() < deadline {
            if let Ok(Some(descriptor)) = daemon::read_descriptor(data_dir)
                && !is_stale_descriptor(stale.as_ref(), &descriptor)
                && healthz_ok(&http, &descriptor.endpoint).await
            {
                return Ok(Self::from_descriptor(http, descriptor));
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

    fn from_descriptor(http: reqwest::Client, descriptor: RuntimeDescriptor) -> Self {
        Self {
            http,
            endpoint: descriptor.endpoint,
            token: descriptor.token,
        }
    }

    /// Authenticated GET; non-2xx becomes a [`CliError`] carrying the
    /// server's structured error message.
    async fn get(&self, path: &str) -> Result<Value, CliError> {
        let response = self
            .http
            .get(format!("{}{path}", self.endpoint))
            .bearer_auth(&self.token)
            .send()
            .await?;
        Self::into_value(response).await
    }

    /// Authenticated POST with a JSON body.
    async fn post(&self, path: &str, body: &Value) -> Result<Value, CliError> {
        let response = self
            .http
            .post(format!("{}{path}", self.endpoint))
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await?;
        Self::into_value(response).await
    }

    async fn into_value(response: reqwest::Response) -> Result<Value, CliError> {
        let status = response.status();
        let body: Value = response.json().await.unwrap_or(Value::Null);
        if status.is_success() {
            Ok(body)
        } else {
            let message = body["error"]["message"]
                .as_str()
                .map(str::to_string)
                .unwrap_or_else(|| format!("HTTP {status}"));
            Err(CliError::new(format!("{status}: {message}")))
        }
    }
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
