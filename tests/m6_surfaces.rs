//! M6 acceptance tests (sergeant-rs-workspace's knowledge/evidence/gauntlet/contracts/M6.md).
//!
//! Acceptance 2 (the embedded dashboard) and its tests are gone: ADR 0011
//! (D7) deletes `src/web.rs`, `web/`, and the `sgt web` verb outright — a
//! disabled stub carrying open reactivation issues was rejected as a
//! maintenance claim this repo would not honor, deletion is the lower rung.
//! What is left, renumbered:
//!
//! 1. TUI data/render: against a live daemon with seeded work, the fleet and
//!    detail screens render the contracted fields (ratatui `TestBackend`,
//!    content assertions rather than pixels); an SSE-driven update is
//!    reflected in a re-render; the TUI's write keys actually work through
//!    the API; and `tui.rs` imports nothing but the API client.
//! 2. Doctor: a healthy environment exits 0 with every check green; a broken
//!    data dir and an absent claude each produce their named failing check,
//!    a remedy line, and a nonzero exit; `--json` is stable.
//! 3. Demo: `scripts/demo.sh` exits 0 in a clean temp environment and its
//!    printed evidence pointers resolve — the journal, the graph answer, and
//!    the analytics answer are all re-read from the kept directory, and the
//!    graph's cited seqs are checked against the journal, rather than the
//!    script being graded on the prose it printed. The non-pausing arc
//!    `--real-claude` takes is run too, with a fake backend, so the flag's
//!    control flow is exercised without tokens.
//! 4. Clients-are-equal: `tui.rs` reaches state only through the API's own
//!    types — a path scan over every crate-rooted path it names (`crate::`
//!    and `super::`, brace groups walked). The scan is itself tested,
//!    because an instrument nobody has tried to fool is a claim rather than
//!    a measurement: `use crate::{…}` and `use super::daemon::…` each hid a
//!    real reach into daemon internals from the earlier version.
//!
//! **TUI test approach** (the contract's Unknown). Content assertions on a
//! `TestBackend` buffer proved workable and are used as the primary
//! instrument: the buffer is flattened to text lines and searched for the
//! contracted fields. Exact-frame snapshots were rejected — they break on
//! every layout tweak and assert mostly about box-drawing.
//!
//! The view-model layer is unit tested as well, not merely as a fallback, and
//! here is exactly where, because a claim about where tests live is itself a
//! test claim:
//!
//! - `src/tui.rs`'s own `#[cfg(test)] mod tests` covers the projection field
//!   by field (`fleet_rows` → `WorkRow`, including the `-`-for-missing rule
//!   and the one-based stage position), the keymap (`App::on_key`: bounded
//!   navigation, the two-keystroke write keys), `App::observe`'s
//!   refresh-worthy set, and the durable liveness indicator in the header;
//! - `stage_label` and `field_text` are *not* the TUI's — they live in
//!   `src/api.rs`, unit tested there next to their own definitions (moved
//!   from `src/web.rs`'s tests when the dashboard was deleted, ADR 0011 —
//!   the rule they encode outlives the second client that used to share it).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::api::ApiClient;
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::event::{EventDraft, EventSource};
use sergeant_rs::runtime::git::canonical_git_common_dir;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::tui::{self, Action, App, Destination};

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------- helpers

fn ulid() -> String {
    ulid::Ulid::generate().to_string()
}

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client")
}

/// An estate root with exactly one repository, and that repository's derived
/// mount — the only shape a Work can bind to now.
///
/// Estate-root §6.1 removed `[[repo]] path`: a mount is always
/// `<estate-root>/repos/<name>`, so a bare `TempDir` holding a git repo is no
/// longer anything the engine will plan against. `support::scaffold_solo_estate`
/// is the shared builder for that shape; the mount comes back because
/// `origin.cwd` on a submission is recorded evidence only now (§13.3), and the
/// mount is the honest thing to record.
///
/// One repository on purpose, in every caller: a single-repository estate
/// infers its sole repository on an empty scope, which is what keeps the
/// submissions below scope-free.
fn solo_estate(name: &str) -> (TempDir, PathBuf) {
    let root = TempDir::new().expect("tempdir");
    let (mount, _head) = support::scaffold_solo_estate(root.path(), name);
    (root, mount)
}

/// An estate root with a valid `[estate]` manifest and no repositories yet —
/// the state a fresh `sgt init` leaves, and all a daemon needs to be
/// *admitted* (§4.1 admission is structural: `Estate::admit` never
/// resolves mounts, so "nothing declared yet" is not an estate-identity
/// fault). Used by the tests whose subject is the daemon's own lifecycle
/// rather than anything it would plan.
fn empty_estate(root: &Path, name: &str) {
    std::fs::create_dir_all(root).expect("estate root");
    std::fs::write(
        root.join("sergeant.toml"),
        format!("[estate]\nname = {name:?}\n"),
    )
    .expect("write sergeant.toml");
}

/// A two-stage workflow so the stage coordinate is non-trivial on screen.
fn write_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/tiny");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"tiny\"\nversion = \"1\"\nstages = [\"00-first\", \"10-second\"]\n",
    )
    .expect("workflow.toml");
    for stage in ["00-first", "10-second"] {
        std::fs::create_dir_all(dir.join(stage)).expect("stage dir");
        std::fs::write(dir.join(stage).join("CONTEXT.md"), "context").expect("CONTEXT.md");
    }
}

/// A one-stage workflow — T1c's mutation tests each drive exactly one Work
/// through exactly one stage, so a single-stage fixture keeps each test's
/// `FakeStep` script trivially "one step per turn", with no risk of a
/// second Work's launch or a retry's re-entry consuming the wrong entry off
/// a queue shared with anything else.
fn write_solo_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/solo");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"solo\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::create_dir_all(dir.join("00-only")).expect("stage dir");
    std::fs::write(dir.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");
}

/// Start an in-process host daemon over `data_dir`.
///
/// **H1: no estate binding.** The daemon comes up serving zero estates and
/// admits `estate_root` the first time a request addresses it — which is why
/// [`submit`] below sends `estate_root` explicitly. A submission that names
/// no estate still plans against nothing and sits on `pending` forever, so
/// the addressing is exactly as load-bearing as the old binding was; it just
/// lives on the request now.
///
/// `estate_root` is kept as a parameter — unused by the start itself — so
/// every call site still reads as "this rig's estate is that one", and the
/// helper stays the one place a multi-estate variant would grow from.
async fn start_fake(
    data_dir: &Path,
    _estate_root: &Path,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, FakeBackend) {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

/// D4: a submission names the estate it addresses and, separately, the cwd
/// it came from. Keeping them two parameters is the point — every call site
/// here passes a *mount* as `cwd`, which is exactly the thing §5.2 forbids
/// being read as topology, and a rig that conflated them would stop proving
/// that.
/// **The two-estate sibling of [`start_fake`]** (brief deliverable 10):
/// one in-process host daemon, no estates bound, ready to admit whichever
/// roots its requests address.
///
/// Additive — [`start_fake`] and every test on it are untouched, because
/// single-estate is a special case of host mode rather than a removed one.
/// The only difference is what the rig *says*: this one takes no estate at
/// all, which is what a host daemon actually is.
async fn start_fake_host(
    data_dir: &Path,
    script: impl IntoIterator<Item = FakeStep>,
) -> (DaemonHandle, FakeBackend) {
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, script);
    let registry = BackendRegistry::new().with(Arc::new(fake.clone()));
    let handle = daemon::start_with(
        data_dir,
        DaemonConfig {
            backends: Arc::new(registry),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    (handle, fake)
}

async fn submit(
    handle: &DaemonHandle,
    estate_root: &Path,
    cwd: &Path,
    intent: &str,
    workflow: &str,
) -> String {
    let body = json!({
        "command_id": ulid(),
        "intent": intent,
        "workflow": workflow,
        "estate_root": estate_root,
        "origin": {"client": "cli", "cwd": cwd},
    });
    let response = http()
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("submit");
    assert_eq!(response.status(), 201, "submit must be accepted");
    let value: Value = response.json().await.expect("json");
    value["work"]["id"].as_str().expect("work id").to_string()
}

/// D4: a client addresses the estate its estate-scoped requests mean. The
/// daemon binds none, so a client that named none would be refused by name
/// on every estate-scoped route — which is the contract, not a rig quirk.
fn client_for(handle: &DaemonHandle, estate_root: &Path) -> ApiClient {
    ApiClient::new(&handle.endpoint, &handle.token)
        .expect("client")
        .with_estate_root(estate_root)
}

/// The `TestBackend` buffer as plain text lines — the content assertions in
/// this file are about *what the screen says*, never about where the box
/// characters land.
fn screen_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| {
                    buffer
                        .cell((x, y))
                        .map(|cell| cell.symbol())
                        .unwrap_or(" ")
                        .to_string()
                })
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn screen_text(terminal: &Terminal<TestBackend>) -> String {
    screen_lines(terminal).join("\n")
}

fn render(app: &App) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(200, 44)).expect("test terminal");
    terminal
        .draw(|frame| tui::draw(frame, app))
        .expect("draw the screen");
    terminal
}

fn assert_shows(terminal: &Terminal<TestBackend>, needle: &str, what: &str) {
    let text = screen_text(terminal);
    assert!(
        text.contains(needle),
        "the {what} must appear on screen ({needle:?} not found in):\n{text}"
    );
}

// ------------------------------------------------------------------- 1. TUI

/// Acceptance 1. The TUI's Fleet browser and canonical-Work stub render the
/// contracted fields from a live daemon, a live SSE event drives a
/// re-render, and Home's New Work submission is a real verb against the API
/// — not decoration. (T1a replaces the M6-era two-screen Fleet/Detail TUI
/// with `Home / Fleet / Workflows / Estate` navigation over a canonical Work
/// stub; respond/retry/extend/cancel mutations are T1c's job, §20.2, so this
/// acceptance test no longer drives them.)
#[tokio::test]
async fn t1_the_tui_renders_and_drives_the_fleet_over_the_api() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    // The workflow package belongs to the *estate*, not to a mount: the
    // daemon reads `.sergeant/workflows/` under the estate it is bound to.
    write_workflow(estate.path());

    // Two works: one that finishes, one that stops for an answer, so Fleet
    // has to show two different states and the canonical Work stub has
    // something to say about stage and workflow.
    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [
            FakeStep::complete_with("first stage done"),
            FakeStep::complete(),
            FakeStep::needs_input("which retry budget?"),
        ],
    )
    .await;
    let client = client_for(&handle, estate.path());

    let done = submit(&handle, estate.path(), &mount, "finish this one", "tiny").await;
    let asking = submit(&handle, estate.path(), &mount, "ask me something", "tiny").await;

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");

    // --- Fleet ---------------------------------------------------------
    app.on_key(ratatui::crossterm::event::KeyCode::Esc); // leave Home's form focus
    app.on_key(ratatui::crossterm::event::KeyCode::Char('2')); // Fleet
    assert_eq!(app.destination, Destination::Fleet);

    let terminal = render(&app);
    // Fleet's id column is §10.2's lowest-priority, "short id" field — it
    // may truncate a full ULID under real terminal widths, so this checks a
    // distinguishing prefix rather than the whole thing. The canonical Work
    // stub below asserts the untruncated id.
    assert_shows(&terminal, &done[..12], "completed work's id (prefix)");
    assert_shows(&terminal, &asking[..12], "waiting work's id (prefix)");
    assert_shows(&terminal, "completed", "completed state");
    assert_shows(&terminal, "needs_input", "needs_input state");
    assert_shows(&terminal, "finish this one", "intent");
    assert_shows(&terminal, "00-first 1/2", "stage coordinate");
    assert_shows(&terminal, FAKE_BACKEND_NAME, "resolved backend");
    assert_shows(&terminal, "Fleet — 2 works", "fleet header");

    // --- the canonical Work surface (§13, T1b) --------------------------
    let asking_row = app
        .rows
        .iter()
        .position(|row| row.id == asking)
        .expect("the waiting work is in the fleet");
    app.fleet.selected = asking_row;
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    assert_eq!(
        action,
        Action::OpenWork(asking.clone()),
        "opening the real surface asks for the fetch WorkScreen::load makes"
    );
    assert!(
        app.open_work.is_none(),
        "not open until the fetch actually runs"
    );
    let outcome = app.execute(&client, action).await;
    assert_eq!(outcome, Action::None);
    assert_eq!(
        app.open_work,
        Some(tui::OpenWork {
            id: asking.clone(),
            from: Destination::Fleet,
        })
    );

    // Thread (default tab): the initial intent, the pinned workflow's own
    // question surfaced as a gold ask card or, absent one from this fake
    // backend, at minimum the header's above-the-fold pin.
    let terminal = render(&app);
    assert_shows(&terminal, &asking, "work id");
    assert_shows(&terminal, "ask me something", "intent");
    assert_shows(&terminal, "needs_input", "state");
    assert_shows(&terminal, "tiny", "workflow name");
    assert_shows(&terminal, "00-first", "stage");
    assert_shows(
        &terminal,
        "which retry budget?",
        "the stage's question, pinned above the fold",
    );
    assert_shows(&terminal, "Thread", "the tab bar names Thread");
    assert_shows(&terminal, "Workflow", "the tab bar names Workflow");
    assert_shows(&terminal, "Evidence", "the tab bar names Evidence");
    assert_shows(&terminal, "Graph", "the tab bar names Graph");
    assert_shows(&terminal, "Details", "the tab bar names Details");

    // Workflow: the pinned stage rail, not a percent bar.
    app.on_key(ratatui::crossterm::event::KeyCode::Tab);
    let terminal = render(&app);
    assert_shows(&terminal, "00-first", "the rail names the first stage");
    assert_shows(&terminal, "10-second", "the rail names the second stage");
    assert!(
        !screen_text(&terminal).contains('%'),
        "never a percent bar (Decision T2-50)"
    );

    // Evidence: the raw journal window for this Work alone.
    app.on_key(ratatui::crossterm::event::KeyCode::Tab);
    let terminal = render(&app);
    assert_shows(&terminal, "stage.entered", "a raw journal kind");

    // Esc returns to exactly where it was opened from, and drops the fetch.
    app.on_key(ratatui::crossterm::event::KeyCode::Esc);
    assert!(app.open_work.is_none(), "Esc closes the surface");
    assert!(app.work_screen.is_none());
    assert_eq!(
        app.destination,
        Destination::Fleet,
        "back to exactly where it was opened from"
    );

    // --- an SSE event drives a re-render ------------------------------------
    let mut stream = client
        .stream_events(app.last_seq, None)
        .await
        .expect("live tail attaches");
    let fresh = submit(
        &handle,
        estate.path(),
        &mount,
        "arrived while watching",
        "tiny",
    )
    .await;
    assert!(
        !screen_text(&render(&app)).contains(&fresh),
        "the new work must not be on screen before the TUI hears about it"
    );

    let deadline = Instant::now() + Duration::from_secs(20);
    let mut asked_to_refresh = false;
    while Instant::now() < deadline {
        let event = tokio::time::timeout(Duration::from_secs(5), stream.next_event())
            .await
            .expect("the live tail must deliver")
            .expect("no malformed frame")
            .expect("the stream must stay open");
        let event = serde_json::to_value(event).expect("event as json");
        if app.observe(&event) {
            asked_to_refresh = true;
        }
        if event["kind"] == "work.submitted" && event["work_id"] == Value::String(fresh.clone()) {
            break;
        }
    }
    assert!(
        asked_to_refresh,
        "a work.submitted event must be classified as state-bearing"
    );
    app.refresh(&client).await.expect("refresh after the event");
    assert_shows(
        &render(&app),
        &fresh[..12],
        "work that arrived over SSE while the TUI was open",
    );

    // --- Home's New Work submission is a real POST /v1/work, not decoration
    app.on_key(ratatui::crossterm::event::KeyCode::Char('1')); // Home; re-focuses the form
    for c in "submitted from the tui".chars() {
        app.on_key(ratatui::crossterm::event::KeyCode::Char(c));
    }
    // Tab past workflow/backend/profile/group/repositories/turns/ceiling
    // to the `[ Run Work ]` control, then submit.
    for _ in 0..8 {
        app.on_key(ratatui::crossterm::event::KeyCode::Tab);
    }
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    let Action::Submit(body) = action else {
        panic!("expected a submission, got {action:?}");
    };
    assert_eq!(body["intent"], "submitted from the tui");
    assert_eq!(body["created_by"], "tui");
    let outcome = app.execute(&client, Action::Submit(body)).await;
    assert_eq!(outcome, Action::Refresh);
    assert!(
        app.home.last_error.is_none(),
        "the daemon must accept it: {:?}",
        app.home.last_error
    );
    assert!(
        app.home.intent.is_empty(),
        "the draft clears once the daemon accepts it"
    );

    let fleet = client.fleet().await.expect("read back");
    assert!(
        fleet["works"]
            .as_array()
            .expect("works")
            .iter()
            .any(|w| w["intent"] == "submitted from the tui"),
        "the TUI's New Work form must actually reach the daemon: {fleet}"
    );

    // …and so does the scope the operator typed (estate-root proposal
    // §13.3): a repository this estate never declared must come back as
    // §15's refusal naming it, which can only happen if the Repositories
    // field actually reaches `scope.repos` and the daemon's own
    // `Engine::resolve_scope` — the same resolution `sgt run --repo` gets.
    // A form that dropped the field would be accepted here instead, since a
    // one-repository estate infers its sole repository on an empty scope.
    app.home.intent.set_text("scoped from the tui");
    app.home.repositories = "not-a-declared-repo".to_string();
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    let Action::Submit(scoped) = action else {
        panic!("expected a submission, got {action:?}");
    };
    assert_eq!(scoped["scope"]["repos"], json!(["not-a-declared-repo"]));
    let outcome = app.execute(&client, Action::Submit(scoped)).await;
    assert_eq!(outcome, Action::Refresh);
    let refused = app
        .home
        .last_error
        .clone()
        .expect("an undeclared repository must be refused");
    assert!(
        refused.contains("not-a-declared-repo"),
        "the refusal must name what the form actually sent: {refused}"
    );

    handle.shutdown().await;
}

// -------------------------------------------------------- T1c: mutations

/// Respond (§15.1's `ANSWER` composer, §13.10's action matrix) reaches the
/// real `POST /v1/work/{id}/input` — driven through the actual keymap
/// (`r`, type the answer, Ctrl+Enter), not a hand-built `Action::Respond`.
#[tokio::test]
async fn t1c_respond_reaches_the_real_api_through_the_answer_composer() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_solo_workflow(estate.path());

    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [
            FakeStep::needs_input("which retry budget?"),
            FakeStep::complete(),
        ],
    )
    .await;
    let client = client_for(&handle, estate.path());
    let id = submit(&handle, estate.path(), &mount, "ask a question", "solo").await;

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    let action = app.execute(&client, Action::OpenWork(id.clone())).await;
    assert_eq!(action, Action::None);
    assert_eq!(
        app.work_screen.as_ref().unwrap().work()["work"]["state"],
        "needs_input"
    );

    assert_eq!(
        app.on_key(ratatui::crossterm::event::KeyCode::Char('r')),
        Action::None,
        "'r' focuses the answer composer, it does not itself mutate"
    );
    for c in "forty turns".chars() {
        app.on_key(ratatui::crossterm::event::KeyCode::Char(c));
    }
    let ctrl_enter = ratatui::crossterm::event::KeyEvent::new(
        ratatui::crossterm::event::KeyCode::Enter,
        ratatui::crossterm::event::KeyModifiers::CONTROL,
    );
    let action = app.on_key(ctrl_enter);
    assert_eq!(
        action,
        Action::Respond {
            id: id.clone(),
            input: "forty turns".to_string(),
        }
    );
    let outcome = app.execute(&client, action).await;
    assert_eq!(
        outcome,
        Action::Refresh,
        "a mutation always asks to refresh"
    );
    assert!(
        app.status.contains(&format!("responded to {id}")),
        "{}",
        app.status
    );

    let work = client.work(&id).await.expect("read back");
    assert_eq!(
        work["work"]["state"], "completed",
        "the real POST /v1/work/{{id}}/input resumed the turn, which the \
         script's next step then completes — a decoration would have left \
         this Work parked on needs_input forever: {work}"
    );

    handle.shutdown().await;
}

/// Cancel (§13.10/§15.5) reaches the real `POST /v1/work/{id}/cancel`,
/// driven through the confirmation overlay's own deliberate Enter.
#[tokio::test]
async fn t1c_cancel_reaches_the_real_api_through_the_confirmation() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_solo_workflow(estate.path());

    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [FakeStep::waiting("queued upstream")],
    )
    .await;
    let client = client_for(&handle, estate.path());
    let id = submit(&handle, estate.path(), &mount, "wait around", "solo").await;

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    app.execute(&client, Action::OpenWork(id.clone())).await;
    assert_eq!(
        app.work_screen.as_ref().unwrap().work()["work"]["state"],
        "waiting"
    );

    assert_eq!(
        app.on_key(ratatui::crossterm::event::KeyCode::Char('c')),
        Action::None
    );
    assert!(
        app.overlay.is_some(),
        "'c' must open the cancel confirmation before anything is sent"
    );
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    assert_eq!(action, Action::Cancel(id.clone()));
    let outcome = app.execute(&client, action).await;
    assert_eq!(outcome, Action::Refresh);
    assert!(app.overlay.is_none(), "success closes the confirmation");

    let work = client.work(&id).await.expect("read back");
    assert_eq!(work["work"]["state"], "canceled", "{work}");

    handle.shutdown().await;
}

/// Retry (§13.10/§15.5) reaches the real `POST /v1/work/{id}/retry`.
#[tokio::test]
async fn t1c_retry_reaches_the_real_api_through_the_confirmation() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_solo_workflow(estate.path());

    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [FakeStep::fail("boom"), FakeStep::complete()],
    )
    .await;
    let client = client_for(&handle, estate.path());
    let id = submit(&handle, estate.path(), &mount, "fail then retry", "solo").await;

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    app.execute(&client, Action::OpenWork(id.clone())).await;
    assert_eq!(
        app.work_screen.as_ref().unwrap().work()["work"]["state"],
        "failed"
    );

    app.on_key(ratatui::crossterm::event::KeyCode::Char('t'));
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    assert_eq!(action, Action::Retry(id.clone()));
    let outcome = app.execute(&client, action).await;
    assert_eq!(outcome, Action::Refresh);

    let work = client.work(&id).await.expect("read back");
    assert_eq!(
        work["work"]["state"], "completed",
        "retry re-entered the stage, which the script's next step completes: {work}"
    );

    handle.shutdown().await;
}

/// Extend (§13.10/§15.5) reaches the real `POST /v1/work/{id}/extend` with
/// the explicit turns the operator typed, and never implicitly retries
/// (§13.9's closing rule) — the Work stays `blocked` afterward.
#[tokio::test]
async fn t1c_extend_reaches_the_real_api_with_the_typed_turn_count() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_solo_workflow(estate.path());

    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [FakeStep::blocked("envelope exhausted")],
    )
    .await;
    let client = client_for(&handle, estate.path());
    let id = submit(&handle, estate.path(), &mount, "run out of turns", "solo").await;

    let before = client.work(&id).await.expect("read back");
    assert_eq!(before["work"]["state"], "blocked");
    let cap_before = before["envelope"]["turn_cap"].as_u64().expect("turn_cap");

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    app.execute(&client, Action::OpenWork(id.clone())).await;

    app.on_key(ratatui::crossterm::event::KeyCode::Char('e'));
    for c in "5".chars() {
        app.on_key(ratatui::crossterm::event::KeyCode::Char(c));
    }
    app.on_key(ratatui::crossterm::event::KeyCode::Tab); // field -> Confirm
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    assert_eq!(
        action,
        Action::Extend {
            id: id.clone(),
            additional_turns: 5,
        }
    );
    let outcome = app.execute(&client, action).await;
    assert_eq!(outcome, Action::Refresh);
    assert!(app.overlay.is_none(), "success closes the confirmation");

    let after = client.work(&id).await.expect("read back");
    assert_eq!(
        after["envelope"]["turn_cap"].as_u64(),
        Some(cap_before + 5),
        "{after}"
    );
    assert_eq!(
        after["work"]["state"], "blocked",
        "extend must not implicitly retry: {after}"
    );

    handle.shutdown().await;
}

/// A two-level nested workflow package (S2 W1 / decision E1) in the estate:
/// `10-investigate` is a stage directory that holds its own `workflow.toml`,
/// so it is a container and its leaves flatten to composed ids.
fn write_nested_workflow(root: &Path) {
    let dir = root.join(".sergeant/workflows/nested");
    std::fs::create_dir_all(&dir).expect("workflow dir");
    std::fs::write(
        dir.join("workflow.toml"),
        "[workflow]\nname = \"nested\"\nversion = \"1\"\n\
         stages = [\"00-orient\", \"10-investigate\", \"20-implement\"]\n",
    )
    .expect("workflow.toml");
    for stage in ["00-orient", "20-implement"] {
        std::fs::create_dir_all(dir.join(stage)).expect("stage dir");
        std::fs::write(dir.join(stage).join("CONTEXT.md"), "context").expect("CONTEXT.md");
    }
    let investigate = dir.join("10-investigate");
    std::fs::create_dir_all(&investigate).expect("container dir");
    std::fs::write(
        investigate.join("workflow.toml"),
        "[workflow]\nname = \"10-investigate\"\nversion = \"1\"\n\
         stages = [\"00-lead\", \"10-code\"]\n",
    )
    .expect("nested workflow.toml");
    for stage in ["00-lead", "10-code"] {
        std::fs::create_dir_all(investigate.join(stage)).expect("nested stage dir");
        std::fs::write(investigate.join(stage).join("CONTEXT.md"), "context").expect("CONTEXT.md");
    }
}

/// S2 W1 / decision E10, end to end against a live daemon: a Work bound to a
/// nested workflow renders its rail as a **tree** — one indented header per
/// container, leaves under it — while the header's stage coordinate names
/// the current leaf by its full composed id.
///
/// Both halves are deliberately asserted here rather than only in
/// `work_view.rs`'s own unit tests. The wire shape stayed flat
/// (`workflow.stages` is still one array of leaf ids, and `stage_label`
/// still does `index + 1` of `of`), so the tree exists only in the render —
/// which means a rig that unit-tested `workflow_lines` alone could not tell
/// whether the API actually serves ids with `/` in them at all.
#[tokio::test]
async fn t1e_a_nested_workflow_renders_as_an_indented_tree_over_the_live_api() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_nested_workflow(estate.path());

    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [
            FakeStep::complete(),                 // 00-orient
            FakeStep::needs_input("which lead?"), // 10-investigate/00-lead parks here
        ],
    )
    .await;
    let client = client_for(&handle, estate.path());
    let work = submit(
        &handle,
        estate.path(),
        &mount,
        "investigate deeply",
        "nested",
    )
    .await;

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    let action = Action::OpenWork(work.clone());
    assert_eq!(app.execute(&client, action).await, Action::None);

    // The header coordinate: `stage_label` renders a composed id as-is,
    // with the flat position arithmetic E10 preserved.
    let terminal = render(&app);
    assert_shows(
        &terminal,
        "10-investigate/00-lead",
        "the current stage's composed id",
    );

    // The Workflow tab (§13.4's rail), now a tree. `Tab` rather than `2`,
    // the same way t1 above reaches it: Home's own form still owns the digit
    // keys here, and leaving it would close the open Work.
    app.on_key(ratatui::crossterm::event::KeyCode::Tab);
    let terminal = render(&app);
    // Asserted as whole-line substrings (the rail's rows sit inside a
    // bordered pane, so a row never *begins* at column 0): the container
    // header carries its own glyph and a plain done/total over the leaves it
    // owns, each of those leaves is indented one level under it, and the
    // container's sibling stays at the top level.
    for row in [
        "✓ 00-orient",
        "⠹ 10-investigate/  0/2",
        "  ⠹ 00-lead",
        "  · 10-code",
        "· 20-implement",
    ] {
        assert_shows(&terminal, row, "a row of the nested rail");
    }
    let lines = screen_lines(&terminal);
    let leaf = lines
        .iter()
        .find(|line| line.contains("⠹ 00-lead"))
        .expect("the current leaf's row");
    assert!(
        !leaf.contains("10-investigate/00-lead"),
        "a nested leaf shows its own name under its container header, not the composed id \
         again: {leaf:?}"
    );
    assert!(
        !screen_text(&terminal).contains('%'),
        "never a percent bar, containers included (Decision T2-50)"
    );

    handle.shutdown().await;
}

/// §12.1 end to end: the Add-repo form's real `POST /v1/estate/repos` —
/// kicked off the render loop rather than awaited inline (the spinner/
/// elapsed rule), per T3's own routes over `crate::domain::manifest::
/// add_repo`. `App::poll_background` (called every loop tick in production,
/// polled directly here) is what notices the clone finished and closes the
/// overlay.
#[tokio::test]
async fn t3_estate_add_repo_reaches_the_real_api_off_the_render_loop() {
    // Estate-root §5.1: `src/api.rs`'s `resolve_estate_root` reads the estate
    // the daemon was *bound* to at start, rather than walking up from its
    // data dir — so the binding below, not the nesting, is what makes
    // `POST /v1/estate/repos` resolvable. The default
    // `<estate_root>/.sergeant/data` layout is kept anyway because that is
    // where a real `sgt daemon` at this root would put it, and a route that
    // silently depended on the two agreeing would be invisible otherwise.
    // No `[[repo]]` entries yet — declaring the first one is the subject.
    let estate_root = TempDir::new().expect("tempdir");
    empty_estate(estate_root.path(), "t3-tui-estate");
    let data_dir = estate_root.path().join(".sergeant/data");
    std::fs::create_dir_all(&data_dir).expect("data dir");
    let source = TempDir::new().expect("tempdir");
    support::init_repo(source.path());

    let (handle, _fake) = start_fake(&data_dir, estate_root.path(), []).await;
    let client = client_for(&handle, estate_root.path());

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");

    app.on_key(ratatui::crossterm::event::KeyCode::Esc); // leave Home's form focus first
    app.on_key(ratatui::crossterm::event::KeyCode::Char('4'));
    assert_eq!(app.destination, Destination::Estate);
    let opened = app.on_key(ratatui::crossterm::event::KeyCode::Char('a'));
    assert_eq!(
        opened,
        Action::None,
        "opening the form is local, not a mutation"
    );

    for c in "svc-a".chars() {
        app.on_key(ratatui::crossterm::event::KeyCode::Char(c));
    }
    app.on_key(ratatui::crossterm::event::KeyCode::Tab); // -> origin
    for c in source.path().to_str().expect("utf8 path").chars() {
        app.on_key(ratatui::crossterm::event::KeyCode::Char(c));
    }
    app.on_key(ratatui::crossterm::event::KeyCode::Tab); // -> instructions
    app.on_key(ratatui::crossterm::event::KeyCode::Tab); // -> confirm
    let action = app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    assert_eq!(
        action,
        Action::AddRepo {
            name: "svc-a".to_string(),
            origin: Some(source.path().to_str().expect("utf8 path").to_string()),
            instructions: None,
        }
    );

    let outcome = app.execute(&client, action).await;
    assert_eq!(
        outcome,
        Action::None,
        "the clone runs off the render loop — execute must return immediately, not await it"
    );
    assert!(
        app.overlay.is_some(),
        "the overlay stays open showing the spinner until the background add finishes"
    );

    let deadline = Instant::now() + Duration::from_secs(10);
    let needs_refresh = loop {
        let needs_refresh = app.poll_background();
        if app.estate.pending_repo_add.is_none() {
            break needs_refresh;
        }
        assert!(
            Instant::now() < deadline,
            "the background add-repo task must finish and be noticed"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    assert!(
        needs_refresh,
        "a successful add must ask for a refresh: {}",
        app.status
    );
    assert!(app.overlay.is_none(), "success closes the add-repo overlay");
    assert!(app.status.contains("repo added"), "{}", app.status);

    app.refresh(&client).await.expect("re-read after the add");
    assert!(
        app.estate.repos.iter().any(|r| r["name"] == "svc-a"),
        "the real POST must have declared the repo, not just returned success: {:?}",
        app.estate.repos
    );

    handle.shutdown().await;
}

/// §12.3: Health renders the real `GET /v1/doctor` report — status, check
/// name, and detail per check, reached by navigating there through the
/// keymap (Estate's own Tab cycles Repositories → Groups → Health) rather
/// than poking internal state the TUI itself cannot reach any other way.
#[tokio::test]
async fn t3_estate_health_renders_the_real_doctor_report() {
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    empty_estate(estate.path(), "t3-health-estate");
    let (handle, _fake) = start_fake(data.path(), estate.path(), []).await;
    let client = client_for(&handle, estate.path());

    let mut app = App::new();
    app.on_key(ratatui::crossterm::event::KeyCode::Esc); // leave Home's form focus first
    app.on_key(ratatui::crossterm::event::KeyCode::Char('4'));
    app.on_key(ratatui::crossterm::event::KeyCode::Tab); // Repositories -> Groups
    app.on_key(ratatui::crossterm::event::KeyCode::Tab); // Groups -> Health
    app.refresh(&client).await.expect("first read");

    let terminal = render(&app);
    assert_shows(&terminal, "git", "the doctor report's git check");
    assert_shows(
        &terminal,
        "disk_pressure",
        "the doctor report's disk_pressure check",
    );

    handle.shutdown().await;
}

/// Acceptance 1's structural half, stated for the TUI alone: the terminal UI
/// reaches state through the API client and nothing else. (Acceptance 5
/// applies the same rule to both clients at once; this is the §30 sentence
/// checked where §30 puts it.)
#[test]
fn t1_the_tui_has_no_private_shortcut() {
    let tui = code_only(&read_tui_source());
    assert_eq!(
        crate_paths(&tui),
        vec!["api".to_string()],
        "the tui module tree may name crate::api and nothing else — a private \
         shortcut here means the API is incomplete (§30)"
    );
    assert!(
        tui.contains("ApiClient"),
        "…and it must actually use the client, or the rule above is vacuous"
    );
}

/// ADR 0010 (D6): bare `sgt` is a homepage now, not the TUI, and it must
/// touch no daemon at all — that is what dissolves ADR 0009's TUI carve-out
/// debate instead of answering it. A build that still wired bare `sgt` to
/// the TUI (§30, the prior behavior) would either spawn a daemon here or
/// fail at terminal setup; this build must do neither.
#[test]
fn t1_bare_sgt_prints_the_homepage_and_touches_no_daemon() {
    let data = DataDir::new();
    // Outside any estate. Under exact-root admission (§4.1) that is now a
    // property of one directory rather than of a whole ancestry: a fresh
    // tempdir is not an estate root, full stop, and no walk can make it one.
    // The isolation still matters because the test process's own cwd is the
    // checkout — which is itself an estate under clone-is-distro (README.md)
    // — and would take the estate-aware branch the sibling test below covers.
    let cwd = TempDir::new().expect("tempdir outside any estate");
    let output = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .current_dir(cwd.path())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run bare sgt");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        output.status.success(),
        "the homepage must not fail (status {:?}, stderr {:?})",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("sgt init") && stdout.contains("sgt doctor"),
        "outside an estate, the homepage must orient a stranger toward \
         `sgt init`/`sgt doctor`: {stdout:?}"
    );
    assert!(
        daemon::read_descriptor(data.path())
            .expect("descriptor readable")
            .is_none(),
        "bare `sgt` must not spawn a daemon — it is a homepage, not a client (ADR 0010)"
    );
}

/// ADR 0010's open question (estate-aware vs. static banner) resolved in the
/// estate-aware direction: inside an estate, the homepage names it and its
/// declared repository count — reading `sergeant.toml` is not observing the
/// daemon, so this still must not spawn one.
#[test]
fn t1_bare_sgt_inside_an_estate_names_it_and_touches_no_daemon() {
    let data = DataDir::new();
    let estate = TempDir::new().expect("tempdir");
    let init = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .current_dir(estate.path())
        .arg("init")
        .arg("--name")
        .arg("probe-estate")
        .output()
        .expect("run sgt init");
    assert!(
        init.status.success(),
        "sgt init: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    let output = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .current_dir(estate.path())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run bare sgt inside the estate");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        output.status.success(),
        "the homepage must not fail (status {:?}, stderr {:?})",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("probe-estate"),
        "inside an estate, the homepage must name it: {stdout:?}"
    );
    assert!(
        daemon::read_descriptor(data.path())
            .expect("descriptor readable")
            .is_none(),
        "bare `sgt` must not spawn a daemon even inside an estate (ADR 0010)"
    );
}

/// ADR 0009's no-spawn set, on the TUI specifically: `sgt tui` with no
/// daemon running refuses rather than materializing one just to have
/// something to render, and names `sgt doctor` as the remedy the way every
/// other fail-closed path in this codebase does.
///
/// Run from a real estate root (estate-root §4.2/§4.3): `tui` is
/// estate-scoped, and root validation happens *before* descriptor lookup —
/// from anywhere else this would refuse with §4.4's no-estate diagnostic
/// instead, which is a different refusal by a different mechanism and would
/// leave ADR 0009's rule unmeasured.
#[test]
fn t1_sgt_tui_refuses_without_a_daemon_and_names_the_remedy() {
    let data = DataDir::new();
    let estate = TempDir::new().expect("tempdir");
    empty_estate(estate.path(), "tui-refusal-estate");
    let output = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .arg("tui")
        .current_dir(estate.path())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run sgt tui");
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    assert!(
        !output.status.success(),
        "sgt tui must refuse rather than auto-spawn with no daemon running"
    );
    assert!(
        stderr.contains("sgt doctor"),
        "the refusal must name the remedy: {stderr:?}"
    );
    assert!(
        daemon::read_descriptor(data.path())
            .expect("descriptor readable")
            .is_none(),
        "sgt tui must not have spawned a daemon on its way to refusing"
    );
}

/// ADR 0009's no-spawn set, on the rest of it: `status`, `work list`,
/// `work show`, `work transcript`, `work retained`, `work reap` (its
/// unconfirmed preview only — the `--yes` disposal path still mutates and
/// still belongs to `ensure_daemon`), and `analytics` all refuse rather than
/// auto-spawn with no daemon running, and each names `sgt doctor` as the
/// remedy — the same shape `t1_sgt_tui_refuses_without_a_daemon_and_names_the_remedy`
/// pins for the TUI. A build that still routed any one of these through
/// `ensure_daemon` (the pre-ADR-0009 behavior every one of them used to
/// share) would exit 0 here and leave a daemon behind instead. `work reap`
/// specifically pins the no-mistakes review fix that had its unconfirmed
/// preview path calling `ensure_daemon` — auto-spawning a daemon just to
/// preview a disposal nothing asked to happen yet.
///
/// All seven run from a real estate root, for the reason the TUI sibling
/// above states: estate-root §4.3 puts exact-root validation ahead of
/// descriptor lookup, so outside an estate every one of these would refuse
/// with §4.4's diagnostic before ADR 0009's rule ever came into it — seven
/// tests passing for a reason that has nothing to do with what they pin.
#[test]
fn t1_observation_verbs_refuse_without_a_daemon_and_name_the_remedy() {
    let data = DataDir::new();
    let estate = TempDir::new().expect("tempdir");
    empty_estate(estate.path(), "observation-refusal-estate");
    for args in [
        vec!["status"],
        vec!["work", "list"],
        vec!["work", "show", "01SOMENONEXISTENTWORKID000"],
        vec!["work", "transcript", "01SOMENONEXISTENTWORKID000"],
        vec!["work", "retained"],
        vec!["work", "reap", "01SOMENONEXISTENTWORKID000"],
        vec!["analytics"],
    ] {
        let output = Command::new(SGT)
            .arg("--data-dir")
            .arg(data.path())
            .args(&args)
            .current_dir(estate.path())
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap_or_else(|e| panic!("run sgt {}: {e}", args.join(" ")));
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        assert!(
            !output.status.success(),
            "sgt {} must refuse rather than auto-spawn with no daemon running",
            args.join(" ")
        );
        assert!(
            stderr.contains("sgt doctor"),
            "sgt {}'s refusal must name the remedy: {stderr:?}",
            args.join(" ")
        );
        assert!(
            daemon::read_descriptor(data.path())
                .expect("descriptor readable")
                .is_none(),
            "sgt {} must not have spawned a daemon on its way to refusing",
            args.join(" ")
        );
    }
}

/// Issue #3, end to end and on a real terminal: a TUI whose pty hangs up
/// exits, instead of outliving its screen at ~100% of a core.
///
/// **Why a pty and not a unit test.** The spin is crossterm's — its
/// `event::poll` never returns once the pty master closes — and no in-process
/// test can produce that, because it is precisely the library call the fix
/// arranges never to make. `src/tui.rs`'s unit tests pin the composition (the
/// tick the reader is handed refuses to poll a hung-up terminal; shutdown
/// leaves a wedged reader behind; the watch arm ends the session). This is
/// the measurement they stand in for, and it is the shape the bug was
/// reported in: run the binary under a pty, kill the process holding the
/// master, require the TUI to be gone.
///
/// Measured while this was written: pre-fix, the orphan was still there 15 s
/// later having burned 614 CPU ticks, and it ignored SIGTERM (the shutdown
/// path was parked in an unbounded `reader.join()` behind the spinning
/// thread); fixed, it exits within a few seconds.
///
/// `script(1)` is the pty allocator because it is util-linux and already
/// present wherever these suites run; when it is absent the test says so and
/// stops rather than inventing a weaker claim.
#[cfg(unix)]
#[test]
fn t1_a_tui_whose_terminal_hangs_up_does_not_outlive_it() {
    if Command::new("script").arg("--version").output().is_err() {
        eprintln!("skipping: script(1) is not installed, so no pty can be allocated");
        return;
    }
    let data = DataDir::new();
    let cwd = TempDir::new().expect("tempdir");
    // Estate-root §4.2: `daemon` and `tui` are both estate-scoped, so the
    // daemon is started *at* an estate root and the session below names that
    // same root with `-C`. `script(1)` inherits this process's own working
    // directory, which is the crate checkout and not an estate, so `-C` is
    // the only way to name the root without moving a global (§4.1: the flag
    // names an exact root, and is never searched from either).
    empty_estate(cwd.path(), "tui-hangup-estate");
    // ADR 0009: `sgt tui` is in the no-spawn set now, so a daemon has to
    // already be running for it to attach to — a bare `sgt tui` with none
    // would refuse before ever touching the terminal, which is not what
    // this measures.
    let _daemon = SpawnedDaemon::start(&data, cwd.path(), &[]);
    let typescript_dir = TempDir::new().expect("tempdir");
    let typescript = typescript_dir.path().join("session");

    // `script` allocates the pty and makes it the child's controlling
    // terminal; killing `script` closes the master, which is what a terminal
    // emulator dying does. Wrapped so that an assertion failing before the
    // planned kill does not leave the pty (and the session on it) running.
    //
    // **First-contact macOS fix (path-to-mac.md trip, 2026-08-15).** `-f -c
    // <command> <file>` is util-linux `script`'s syntax; BSD/macOS `script`
    // has no `-c` or `-f` flag at all and refuses both outright ("illegal
    // option"). Its own usage is `script [-aeFkpqr] [-t time] [file
    // [command ...]]` — the typescript file is positional and comes first,
    // and the command follows as trailing positional args (measured
    // directly against this host's `/usr/bin/script`).
    let sgt_tui_command = format!(
        "{SGT} -C {} --data-dir {} tui",
        cwd.path().display(),
        data.display()
    );
    let mut script_command = Command::new("script");
    script_command.arg("-q");
    if cfg!(target_os = "macos") {
        script_command
            .arg(&typescript)
            .arg("sh")
            .arg("-c")
            .arg(&sgt_tui_command);
    } else {
        script_command
            .arg("-f")
            .arg("-c")
            .arg(&sgt_tui_command)
            .arg(&typescript);
    }
    let mut pty = Reaped(
        script_command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("allocate a pty with script(1)"),
    );

    // Wait until the session is actually up — the alternate-screen switch in
    // the typescript is the terminal half; the process itself is found the
    // same way the daemon reaper finds daemons.
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut tui = None;
    while Instant::now() < deadline {
        let painted = std::fs::read(&typescript)
            .map(|bytes| String::from_utf8_lossy(&bytes).contains("[?1049h"))
            .unwrap_or(false);
        if painted {
            tui = tui_pid(data.path());
            if tui.is_some() {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let tui = tui.expect("the TUI must come up under the pty");
    // The watch installs itself as the loop starts; hang up after it has.
    std::thread::sleep(Duration::from_secs(1));
    assert!(
        pid_alive(tui),
        "the session must still be running when its terminal dies, or this \
         measures nothing"
    );

    pty.reap();

    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline && pid_alive(tui) {
        std::thread::sleep(Duration::from_millis(100));
    }
    let survived = pid_alive(tui);
    if survived {
        // Killed before the assertion, not after: a failing assert would
        // otherwise leave the orphan this test is about running on the box.
        let _ = Command::new("kill")
            .arg("-KILL")
            .arg(tui.to_string())
            .status();
    }
    assert!(
        !survived,
        "the TUI outlived the terminal it was drawing on (pid {tui}) — that is \
         issue #3: a process nobody can see, spinning, deaf to SIGTERM"
    );
}

/// A child process this test must not leave behind, on any path out.
#[cfg(unix)]
struct Reaped(std::process::Child);

#[cfg(unix)]
impl Reaped {
    /// Kill it now — the hangup itself, when the child is holding a pty
    /// master. Idempotent with the `Drop` below.
    fn reap(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[cfg(unix)]
impl Drop for Reaped {
    fn drop(&mut self) {
        self.reap();
    }
}

/// The pid of the `sgt` *client* on this data dir — the TUI — as distinct
/// from the daemon it spawned. Argv-matched like `support::daemon_pids`,
/// which this deliberately mirrors rather than extends: the reaper's job is
/// daemons, and widening it would make every suite's cleanup depend on a
/// classification only this test needs.
///
/// **First-contact macOS fix (path-to-mac.md trip, 2026-08-15).** This used
/// to read `/proc` directly with no macOS arm — same bug `support::daemon_pids`
/// had (this function's own doc comment already names that one as the
/// pattern being mirrored, but the fix landed there without also reaching
/// this copy). `.ok()?` on the failed `read_dir("/proc")` made it return
/// `None` silently on macOS rather than panic, which is the more dangerous
/// failure shape: every caller here reads "no TUI client pid" instead of
/// "this platform can't answer that". Reuses
/// [`sergeant_rs::platform::process::running_processes`], the same
/// Ponytail R1 fix applied to `support::daemon_pids`.
#[cfg(unix)]
fn tui_pid(data_dir: &Path) -> Option<u32> {
    let wanted = data_dir.to_string_lossy().to_string();
    let processes = sergeant_rs::platform::process::running_processes()?;
    for process in processes {
        let is_sgt = process
            .argv
            .first()
            .map(PathBuf::from)
            .and_then(|program| program.file_name().map(|name| name == "sgt"))
            .unwrap_or(false);
        let names_dir = process
            .argv
            .windows(2)
            .any(|pair| pair[0] == "--data-dir" && pair[1] == wanted);
        if is_sgt && names_dir && !process.argv.iter().any(|arg| arg == "daemon") {
            return Some(process.pid);
        }
    }
    None
}

/// Whether a pid is still *running* — `/proc` on Linux, like everything else
/// in this rig, so nothing has to be signalled to find out; `ps` on macOS,
/// which has no `/proc`.
///
/// A zombie is not alive: once the pty's owner is killed the session is
/// reparented, and whether its exit status has been collected yet is the
/// reaper's business, not evidence that the process is still there.
///
/// **First-contact macOS arm (path-to-mac.md trip, 2026-08-15).** This used
/// to read `/proc/{pid}/stat` unconditionally — on macOS that path never
/// exists, so every pid read as dead, including the TUI this test had just
/// confirmed was up (`the TUI must come up under the pty` passing right
/// before this check fails is the tell: the process is there, the read
/// isn't). Unlike `platform::process::process_alive` (existence only, no
/// zombie distinction — this test needs the distinction, so that helper
/// isn't a fit here), there is no existing production fact to reuse; this is
/// new decision logic, not a second copy of one that already exists.
#[cfg(target_os = "linux")]
fn pid_alive(pid: u32) -> bool {
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return false;
    };
    // The comm field is parenthesized and may contain spaces; the state
    // character is the first field after it.
    stat.rsplit_once(") ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
        .is_some_and(|state| state != "Z")
}

/// macOS arm: `ps -o state= -p <pid>` prints just the state column (no
/// header, `-o state=`), one word like `Ss` or `Z` — a zombie's state starts
/// with `Z` on BSD `ps` too. Empty output (nothing printed at all) means `ps`
/// found no such pid.
#[cfg(target_os = "macos")]
fn pid_alive(pid: u32) -> bool {
    let Ok(output) = std::process::Command::new("ps")
        .args(["-o", "state=", "-p", &pid.to_string()])
        .output()
    else {
        return false;
    };
    let state = String::from_utf8_lossy(&output.stdout);
    let state = state.trim();
    !state.is_empty() && !state.starts_with('Z')
}

/// The API extension the detail screens needed, tested where it lives: this
/// milestone added `work_id` and `limit` to `/v1/events` rather than letting
/// a client read the journal (§30's rule, applied).
/// H1 §11 criterion 1, in process: **two exact-root estates submit Work to
/// one daemon**, and each Work materializes its surface under its *own*
/// estate's mount.
///
/// This is the fact the whole wave turns on, and the in-process shape is
/// what makes it checkable at the surface level rather than only over the
/// CLI: if `Engine::plan` were still reading one pinned estate, one of these
/// two Works would have been planned against the other's manifest — the same
/// mount name, `solo`, in a different estate, which is exactly the silent
/// confusion a pinned field produces (and exactly why D1 makes the estate
/// coordinate a canonical *root*, never a display name).
#[tokio::test]
async fn h1_two_estates_submit_work_to_one_daemon_and_keep_their_own_surfaces() {
    let data = TempDir::new().expect("tempdir");
    let (payments, payments_mount) = solo_estate("solo");
    let (billing, billing_mount) = solo_estate("solo");
    write_workflow(payments.path());
    write_workflow(billing.path());
    assert_ne!(
        payments.path(),
        billing.path(),
        "the fixture must really be two estates"
    );

    let (handle, _fake) = start_fake_host(
        data.path(),
        [FakeStep::complete_with("done"), FakeStep::complete()],
    )
    .await;

    let first = submit(
        &handle,
        payments.path(),
        &payments_mount,
        "payments work",
        "tiny",
    )
    .await;
    let second = submit(
        &handle,
        billing.path(),
        &billing_mount,
        "billing work",
        "tiny",
    )
    .await;
    assert_ne!(first, second);

    let client = client_for(&handle, payments.path());
    // Each Work's surface must be **cut from its own estate's mount**. Both
    // estates declare a repository called `solo`, deliberately: a name is not
    // an identity (D1), and a pinned engine field would have cut both Works
    // from whichever estate it happened to hold.
    let mut sources = Vec::new();
    for (work_id, estate) in [(&first, payments.path()), (&second, billing.path())] {
        let shown = client
            .work(work_id)
            .await
            .unwrap_or_else(|e| panic!("show {work_id}: {e}"));
        let source = shown["surface"]["bindings"][0]["canonical_top_level"]
            .as_str()
            .unwrap_or_else(|| panic!("{work_id} has no surface: {shown}"))
            .to_string();
        let expected = std::fs::canonicalize(estate.join("repos").join("solo"))
            .expect("canonical mount")
            .to_string_lossy()
            .into_owned();
        assert_eq!(
            source, expected,
            "{work_id} must be cut from its own estate's mount"
        );
        sources.push(source);
    }
    assert_ne!(
        sources[0], sources[1],
        "the two estates' identically-named mounts must not collapse into one"
    );

    // H1 §4: the registry observed exactly the two roots that were addressed
    // — no more (nothing was discovered) and no fewer (both were admitted).
    let listed = client.estates().await.expect("GET /v1/estates");
    let mut roots: Vec<String> = listed["estates"]
        .as_array()
        .expect("estates array")
        .iter()
        .map(|e| e["root"].as_str().expect("root").to_string())
        .collect();
    roots.sort();
    let mut expected = vec![
        std::fs::canonicalize(payments.path())
            .expect("canonical")
            .to_string_lossy()
            .into_owned(),
        std::fs::canonicalize(billing.path())
            .expect("canonical")
            .to_string_lossy()
            .into_owned(),
    ];
    expected.sort();
    assert_eq!(roots, expected, "{listed}");

    // H1 §11 criterion 5: one journal, and a replay tells the two estates
    // apart by their own coordinates.
    let mut by_estate: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for event in sergeant_rs::runtime::journal::Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
    {
        if let (Some(root), Some(work_id)) = (&event.workspace_id, &event.work_id) {
            let works = by_estate.entry(root.clone()).or_default();
            if !works.contains(work_id) {
                works.push(work_id.clone());
            }
        }
    }
    assert_eq!(
        by_estate.len(),
        2,
        "one journal, two estates: {by_estate:?}"
    );
    for (root, works) in &by_estate {
        assert_eq!(
            works.len(),
            1,
            "{root} owns exactly its own Work: {works:?}"
        );
    }

    // D4/D6: `GET /v1/events?estate_root=` narrows the shared stream to one
    // estate, server-side. Without it, "estate-wide watch" would silently
    // become "host-wide watch" for every existing `sgt watch` the moment a
    // second estate is admitted — this field is the wire half of keeping
    // that from happening; routing the client onto it is W3's.
    let payments_root = std::fs::canonicalize(payments.path())
        .expect("canonical")
        .to_string_lossy()
        .into_owned();
    let scoped = client
        .get(&format!(
            "/v1/events?estate_root={}",
            payments_root.replace('/', "%2F")
        ))
        .await
        .expect("filtered history");
    let scoped = scoped["events"].as_array().expect("events").clone();
    assert!(!scoped.is_empty(), "the filter must not empty the stream");
    for event in &scoped {
        assert_eq!(
            event["workspace_id"], payments_root,
            "an estate filter must return only that estate's events: {event}"
        );
    }
    let whole = client.get("/v1/events").await.expect("whole history");
    assert!(
        whole["events"].as_array().expect("events").len() > scoped.len(),
        "and it must actually remove events from the shared stream"
    );

    handle.shutdown().await;
}

/// W4d deliverable 4: a host-scoped client — one that addresses **no**
/// fixed estate at all, `ApiClient::new` with no `.with_estate_root` — sees
/// both estates' Work in Fleet, the estate filter cycles all/payments/
/// billing correctly, and the Estate screen's picker lists both. Reuses
/// [`start_fake_host`] (already landed for the daemon/journal seat's own
/// acceptance test above) rather than adding a second multi-estate daemon
/// fixture — the brief's "start_fake_host-style sibling" already exists.
#[tokio::test]
async fn t5_fleet_and_estate_screen_span_two_estates() {
    let data = TempDir::new().expect("tempdir");
    let (payments, payments_mount) = solo_estate("payments");
    let (billing, billing_mount) = solo_estate("billing");
    write_workflow(payments.path());
    write_workflow(billing.path());

    let (handle, _fake) =
        start_fake_host(data.path(), [FakeStep::complete(), FakeStep::complete()]).await;

    submit(
        &handle,
        payments.path(),
        &payments_mount,
        "payments work",
        "tiny",
    )
    .await;
    submit(
        &handle,
        billing.path(),
        &billing_mount,
        "billing work",
        "tiny",
    )
    .await;

    // Host-scoped: addresses no estate at all — the exact shape `sgt tui`
    // builds under H1 (`ApiClient::new` with no `.with_estate_root`).
    let client = ApiClient::new(&handle.endpoint, &handle.token).expect("client");

    // D1: the estate filter's own identity is the canonical root the daemon
    // folds per Work (`estate_root`), never the manifest's display name —
    // two estates could share one.
    let payments_root = std::fs::canonicalize(payments.path())
        .expect("canonical")
        .to_string_lossy()
        .into_owned();
    let billing_root = std::fs::canonicalize(billing.path())
        .expect("canonical")
        .to_string_lossy()
        .into_owned();

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    app.on_key(ratatui::crossterm::event::KeyCode::Esc); // leave Home's form focus
    app.on_key(ratatui::crossterm::event::KeyCode::Char('2')); // Fleet
    assert_eq!(app.destination, Destination::Fleet);
    // The Attention drawer is a global, unfiltered cross-cut (both Works
    // are non-terminal-free/`completed`... it lists recently-finished ones
    // regardless of Fleet's own local filter) — closed here so this test's
    // "must actually narrow" assertions read Fleet's own table alone,
    // the same `drawer_open = false` every other Fleet geometry fixture
    // already sets.
    app.drawer_open = false;

    // --- Fleet shows both estates' Work -------------------------------
    let terminal = render(&app);
    assert_shows(&terminal, "payments work", "payments estate's work");
    assert_shows(&terminal, "billing work", "billing estate's work");
    assert_shows(
        &terminal,
        "estate all",
        "the header's default all-estates filter",
    );

    // --- the estate filter cycles all -> <one> -> <other> -> all ------
    // `next_estate_filter` sorts distinct estate roots, and each estate's
    // root is an independent fresh tempdir (`solo_estate` per estate), so
    // which one sorts first is not fixed across runs — compare directly
    // rather than asserting a specific order.
    let (first_root, first_intent, second_root, second_intent) = if billing_root < payments_root {
        (
            &billing_root,
            "billing work",
            &payments_root,
            "payments work",
        )
    } else {
        (
            &payments_root,
            "payments work",
            &billing_root,
            "billing work",
        )
    };

    app.on_key(ratatui::crossterm::event::KeyCode::Char('e'));
    assert_eq!(
        app.fleet.filters.estate.as_deref(),
        Some(first_root.as_str())
    );
    let terminal = render(&app);
    assert_shows(&terminal, first_intent, "the filtered-in row");
    assert!(
        !screen_text(&terminal).contains(second_intent),
        "the filter must actually narrow the fleet: {}",
        screen_text(&terminal)
    );

    app.on_key(ratatui::crossterm::event::KeyCode::Char('e'));
    assert_eq!(
        app.fleet.filters.estate.as_deref(),
        Some(second_root.as_str())
    );
    let terminal = render(&app);
    assert_shows(&terminal, second_intent, "the filtered-in row");
    assert!(!screen_text(&terminal).contains(first_intent));

    app.on_key(ratatui::crossterm::event::KeyCode::Char('e'));
    assert_eq!(app.fleet.filters.estate, None, "cycles back to all-estates");

    // --- the Estate screen's picker lists both --------------------------
    app.on_key(ratatui::crossterm::event::KeyCode::Char('4')); // Estate
    assert_eq!(app.destination, Destination::Estate);
    app.refresh(&client).await.expect("estate refresh");
    assert!(
        app.estate.picked.is_none(),
        "a host-scoped client must not silently address either estate on its own"
    );
    let terminal = render(&app);
    assert_shows(&terminal, "payments", "the picker's payments entry");
    assert_shows(&terminal, "billing", "the picker's billing entry");

    // Picking one addresses that estate's own repos/groups/doctor below.
    app.on_key(ratatui::crossterm::event::KeyCode::Enter);
    assert!(
        app.estate.picked.is_some(),
        "Enter picks the highlighted entry"
    );
    app.refresh(&client).await.expect("post-pick refresh");
    let terminal = render(&app);
    assert_shows(&terminal, "Repositories", "sub-tabs appear once picked");

    handle.shutdown().await;
}

#[tokio::test]
async fn t1_the_events_endpoint_filters_and_tails_by_work() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_workflow(estate.path());

    let (handle, _fake) = start_fake(data.path(), estate.path(), []).await;
    let client = client_for(&handle, estate.path());
    let first = submit(&handle, estate.path(), &mount, "first work", "tiny").await;
    let second = submit(&handle, estate.path(), &mount, "second work", "tiny").await;

    let all = client.get("/v1/events").await.expect("history");
    let all = all["events"].as_array().expect("events").len();

    let mine = client
        .work_events(&first, 1000)
        .await
        .expect("filtered history");
    let mine = mine["events"].as_array().expect("events").clone();
    assert!(!mine.is_empty(), "the first work has events");
    assert!(mine.len() < all, "filtering must actually remove events");
    assert!(
        mine.iter()
            .all(|e| e["work_id"] == Value::String(first.clone())),
        "no other work's events may leak into the filter"
    );
    assert!(
        !mine
            .iter()
            .any(|e| e["work_id"] == Value::String(second.clone())),
        "the second work's events must not appear"
    );

    let tail = client.work_events(&first, 3).await.expect("tailed history");
    let tail = tail["events"].as_array().expect("events").clone();
    assert_eq!(tail.len(), 3, "limit must bound the answer");
    assert_eq!(
        tail,
        mine[mine.len() - 3..].to_vec(),
        "the limit must keep the *newest* events, which is what a tail means"
    );

    handle.shutdown().await;
}

// ------------------------------------------------------------ TUI resilience

/// A TUI holding the canonical Work stub open for a work the daemon no
/// longer has falls back to Fleet instead of painting a corpse.
///
/// The data dir can be replaced under a running client (a restore, a wiped
/// scratch dir, a `--data-dir` pointed somewhere else) and `App::open_work`
/// is the one part of `App` that keeps asking about a specific id — T1a's
/// stub does that purely from the already-loaded Fleet projection (no
/// per-work API call yet; that is T1b's job), so the fallback rule is
/// "still in the refreshed Fleet rows or not", checked in `App::refresh`.
#[tokio::test]
async fn a_work_view_whose_work_vanished_falls_back_to_fleet() {
    let data = TempDir::new().expect("tempdir");
    let (estate, mount) = solo_estate("svc");
    write_workflow(estate.path());

    let (handle, _fake) = start_fake(
        data.path(),
        estate.path(),
        [FakeStep::needs_input("which budget?")],
    )
    .await;
    let work_id = submit(&handle, estate.path(), &mount, "still here", "tiny").await;
    let client = client_for(&handle, estate.path());

    let mut app = App::new();
    app.refresh(&client).await.expect("first read");
    assert!(
        app.rows.iter().any(|row| row.id == work_id),
        "the work is in the fleet before it vanishes"
    );
    app.open_work = Some(tui::OpenWork {
        id: "01NOSUCHWORKATALL".to_string(),
        from: Destination::Fleet,
    });

    // The work the stub is showing is gone; the fleet behind it is not.
    app.refresh(&client)
        .await
        .expect("a work that vanished is not a client failure");

    assert_eq!(
        app.open_work, None,
        "the surface falls back rather than painting a corpse"
    );
    assert!(
        app.work_screen.is_none(),
        "its fetched data must not survive the fallback either"
    );
    assert!(
        app.rows.iter().any(|row| row.id == work_id),
        "the fleet it fell back to is the live one"
    );
    assert!(
        !screen_text(&render(&app)).contains("01NOSUCHWORKATALL"),
        "and the screen says nothing about the id that is gone"
    );

    handle.shutdown().await;
}

// ---------------------------------------------------------------- 2. doctor

/// Acceptance 3. Doctor is green on a healthy install, names the failing
/// check and its remedy on a broken one, and its `--json` shape is stable.
#[test]
fn t3_doctor_names_every_fault_and_its_remedy() {
    let data = TempDir::new().expect("tempdir");
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    // TH-02 (MVP-2 D3 fixer pass): a scripted `docker`, the same reason
    // `claude` above is scripted — this test's "every check must be ok"
    // assertion below must not depend on whether *this host* happens to
    // have a reachable Docker Engine (N4.md's own probe-gating rule for
    // Docker facts on the GH runner / cloud container).
    let docker = stub_docker(bin.path());
    // W4c: same treatment for `host_service_manager` — this test's "every
    // check must be ok" assertion must not depend on whether *this host*
    // happens to have a reachable native service manager either.
    let service_manager = stub_service_manager(bin.path());
    // Same treatment for the `environment` check's own `$HOME`: a fixture
    // dir with neither `.cargo/bin` nor `.local/bin` on disk, so it reads a
    // deterministic "ok" regardless of what toolchain directories the
    // test-running host's real $HOME happens to have missing from PATH.
    let home = TempDir::new().expect("tempdir");
    let home = home.path().display().to_string();
    // Estate-root §4.2: `doctor` still runs outside an estate, but it now
    // reports a **failing** `estate_root` row when it is not at one — so
    // "healthy" and "outside an estate" are no longer compatible, and a
    // healthy install is by definition one being reported on from its own
    // estate root. The dedicated fixture root (rather than the test
    // process's cwd) is still what makes this hermetic, for a sharper
    // reason than #165's original one: there is no ancestor walk left to
    // wander into ADR 0008(c)'s self-hosting estate, but cwd is global
    // process state and `doctor_in` is what names the root here.
    //
    // The estate is fully furnished on purpose — one declared repository
    // present at its derived `repos/<name>` mount (§6.1), one workflow
    // package — because `estate` and `workflows` both read `warn` on an
    // estate that has neither, and "every check must be green" is the
    // claim this arm makes.
    let estate = TempDir::new().expect("tempdir");
    support::scaffold_estate(estate.path(), "healthy-fixture-estate", &["svc"]);
    write_workflow(estate.path());

    // --- healthy -------------------------------------------------------------
    let (code, stdout, _) = doctor_in(
        estate.path(),
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("SGT_SYSTEMCTL_BIN", &service_manager),
            ("SGT_LAUNCHCTL_BIN", &service_manager),
            ("HOME", &home),
        ],
        false,
    );
    assert_eq!(code, Some(0), "a healthy install must exit 0:\n{stdout}");
    let (code, healthy_json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("SGT_SYSTEMCTL_BIN", &service_manager),
            ("SGT_LAUNCHCTL_BIN", &service_manager),
            ("HOME", &home),
        ],
        true,
    );
    assert_eq!(code, Some(0));
    let report: Value = serde_json::from_str(&healthy_json).expect("doctor --json is json");
    assert_eq!(report["healthy"], true);
    let names: Vec<&str> = report["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .map(|c| c["name"].as_str().expect("name"))
        .collect();
    assert_eq!(
        names,
        vec![
            "git",
            "claude",
            // ADR 0006's residual hole (#100): whether *this* process's own
            // environment carries the toolchain PATH enrichment `sgt
            // claude`/etc. compose, independent of the data dir below.
            "environment",
            // #67: data_dir is checked before docker — the docker adapter's
            // blob store and disk_pressure's `df` call both live inside the
            // data dir, so this check runs first and both defer to it by
            // name when it fails rather than re-diagnosing the same fault.
            "data_dir",
            // #85, ADR 0003 D6: whether this data dir's filesystem supports
            // reliable advisory locking — independent of data_dir's own
            // writability, so it sits beside rather than behind it.
            "filesystem",
            "docker",
            "journal",
            "projection",
            // S3 X4 (F8): Atlas's coverage row, beside the projection row it
            // is deliberately unlike — one is disposable and refolded, the
            // other persists and is re-derived only by re-reading sources.
            "atlas",
            "daemon",
            // H1 §2/#276, W4c: is a native per-user service manager
            // reachable at all — a distinct question from `daemon`'s own
            // per-data-dir descriptor health above.
            "host_service_manager",
            // Estate-root §4.2/§4.4: is the directory this `sgt doctor` was
            // run from an estate root at all? It is the row the three
            // beneath it read their answer from — a single admission,
            // threaded down the way `data_dir_ok` already is, rather than
            // three independent searches — which is also why it sits
            // immediately above them rather than beside `estate`.
            "estate_root",
            // H1 §6, W4c: the cutover gate — daemon.lock/runtime.json/
            // journal/ still present under this estate's own
            // `.sergeant/data` from before a host runtime root existed.
            // Threaded off `estate_root` the same way `permission_mode`
            // below it is.
            "legacy_estate_runtime",
            "permission_mode",
            // #262: the per-profile `network_access` override, right beside
            // `permission_mode` — same estate-root threading, same
            // fail-closed-at-load guarantee.
            "network_access",
            // MVP-3: manifest health beyond "it parses" — declared repos
            // missing on disk (origin as the remedy) and directories under
            // `repos/` the manifest never declared. Green here because the
            // fixture estate's one declared repository is really at its
            // derived mount.
            "estate",
            // #165: how many workflow packages the estate declares. Green
            // here because the fixture estate declares one; an estate with
            // none reads `warn`, which is `t3f`'s subject.
            "workflows",
            // S2 V4 (owner-directed 2026-08-27): a directory inside a
            // workflow package that looks like a stage but isn't declared
            // in that package's `stages` list. Green here because the
            // fixture estate's one package declares everything it has.
            "workflow_stage_declarations",
            // #261: the installed corpus (AGENTS.md, skills/*/SKILL.md,
            // .sergeant/workflows/**, .sergeant/common/contexts/*.md)
            // cites only routes that resolve inside this estate.
            "doc_routes",
            // #261 owner ruling point 5: the installed distro's own
            // `edition` front matter agrees with the running binary.
            "distro_edition",
            // estate-root §12.2: the cheap Git-surface summary — journal
            // plus retained-artifact filesystem metadata only, never a
            // per-branch git walk. Green here because the fixture estate
            // has no journal at all yet ("no journal yet" reads `ok`, the
            // same as a fresh install with nothing run in it).
            "git_surfaces",
            "disk_pressure",
            // W4: retention's own growth axis — segments, floor,
            // retained-Work count against the cap, prune stall, and startup
            // rebuild time. Runs last, after disk pressure, per
            // `Report::run`'s own registration order.
            "journal_growth",
        ],
        "the --json check list and its order are the stable part of this contract"
    );
    for check in report["checks"].as_array().expect("checks") {
        // #85: `filesystem` is the one check this repo cannot make report
        // `ok` on a healthy macOS install today. `src/platform/fs_locking.rs`
        // always answers `Unknown` there by design (its own doc comment: the
        // fail-closed-*safe* direction, since guessing at unmeasured
        // `statfs`/`getmntinfo`/`diskutil` detection would be worse than
        // admitting it can't tell) — `doctor` surfaces that as `warn`, not a
        // fault to remedy. First measured end-to-end on the MacBook Pro M3
        // Pro arrival trip, 2026-08-15; #85 stays open until real macOS
        // detection ships, which is separate feature work, not this
        // measurement pass.
        if check["name"] == "filesystem" && cfg!(target_os = "macos") {
            assert_eq!(
                check["status"], "warn",
                "the filesystem check must warn, not fail, when detection is merely unmeasured: {check}"
            );
        } else {
            assert_eq!(
                check["status"], "ok",
                "every check must be green on a healthy install: {check}"
            );
            assert!(
                check["remedy"].is_null(),
                "a green check has nothing to remedy: {check}"
            );
        }
        assert!(
            check["detail"].as_str().is_some_and(|d| !d.is_empty()),
            "every check must say what it measured: {check}"
        );
    }

    // Healthy, but with history. The check above ran against a data dir
    // nothing had ever used, where the journal and projection checks take
    // their "nothing here yet" branch — so on its own it would leave the
    // interesting half of both checks unmeasured. Give the dir a real journal
    // and require the same verdict, now with real numbers behind it.
    let used = TempDir::new().expect("tempdir");
    seed_journal(used.path(), 5);
    let (code, stdout, _) = doctor_in(
        estate.path(),
        used.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        false,
    );
    assert_eq!(
        code,
        Some(0),
        "a populated healthy install must exit 0:\n{stdout}"
    );
    let (_, used_json, _) = doctor_in(
        estate.path(),
        used.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&used_json).expect("json");
    assert_eq!(report["healthy"], true);
    let journal = named_check(&report, "journal");
    assert_eq!(journal["status"], "ok");
    assert!(
        journal["detail"]
            .as_str()
            .is_some_and(|d| d.contains("5 events") && d.contains("head seq 5")),
        "the journal check must report what it actually replayed: {journal}"
    );
    let projection = named_check(&report, "projection");
    assert_eq!(projection["status"], "ok");
    assert!(
        projection["detail"]
            .as_str()
            .is_some_and(|d| d.contains("seq 5")),
        "the projection check must report the fold it actually completed: {projection}"
    );
    // …and a journal it cannot replay is a failure, not a shrug.
    corrupt_journal(used.path());
    let (code, stdout, _) = doctor_in(
        estate.path(),
        used.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        false,
    );
    assert_ne!(code, Some(0), "an unreplayable journal must exit nonzero");
    let (_, torn_json, _) = doctor_in(
        estate.path(),
        used.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&torn_json).expect("json");
    assert_eq!(named_check(&report, "journal")["status"], "fail");
    assert!(
        named_check(&report, "journal")["remedy"].is_string(),
        "the journal failure must name its remedy"
    );
    assert!(stdout.contains("remedy:"));
    // …and the check downstream of it must decline rather than blame itself:
    // a projection that cannot be folded because the journal is torn is not a
    // projection fault, and saying so is the whole reason that branch exists.
    let projection = named_check(&report, "projection");
    assert_eq!(
        projection["status"], "warn",
        "a torn journal must leave the projection check declining, not failing: {projection}"
    );
    assert!(
        projection["detail"]
            .as_str()
            .is_some_and(|d| d.contains("not attempted")),
        "the projection check must say it did not run, and why: {projection}"
    );

    // Stability: the same environment produces the same shape (keys, names,
    // order, statuses) — only details may move.
    let (_, first_again, _) = doctor_in(
        estate.path(),
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        true,
    );
    let first_again: Value = serde_json::from_str(&first_again).expect("json");
    let healthy_report: Value =
        serde_json::from_str(&healthy_json).expect("the first healthy report reparses");
    assert_eq!(
        shape(&healthy_report),
        shape(&first_again),
        "the --json shape must not vary between runs"
    );

    // --- an absent claude -----------------------------------------------------
    let missing = bin.path().join("no-such-claude").display().to_string();
    let (code, stdout, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &missing), ("SGT_DOCKER_BIN", &docker)],
        false,
    );
    assert_ne!(code, Some(0), "an unusable claude must exit nonzero");
    let (_, absent_json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &missing), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&absent_json).expect("json");
    assert_eq!(report["healthy"], false);
    let check = named_check(&report, "claude");
    assert_eq!(
        check["status"], "fail",
        "the claude check must be the one that fails"
    );
    assert!(
        check["remedy"]
            .as_str()
            .is_some_and(|r| r.contains("Claude CLI")),
        "the failing check must name its remedy: {check}"
    );
    assert!(
        stdout.contains("remedy:"),
        "the human report must print the remedy line:\n{stdout}"
    );
    // Nothing else broke: a fault must not smear across the report.
    assert_eq!(named_check(&report, "git")["status"], "ok");
    assert_eq!(named_check(&report, "journal")["status"], "ok");

    // --- an absent git --------------------------------------------------------
    //
    // git is resolved off PATH, and it is present in every environment these
    // tests run in — so without this the git check could be a hardcoded `ok`
    // and nothing would notice. Emptying PATH for one doctor run is the whole
    // fault: sergeant materializes every work surface with `git worktree`, so
    // this is a hard failure, not a warning.
    let no_git = bin.path().join("empty-path");
    std::fs::create_dir_all(&no_git).expect("mkdir");
    let (code, stdout, _) = doctor_in(
        estate.path(),
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("PATH", &no_git.display().to_string()),
        ],
        false,
    );
    assert_ne!(
        code,
        Some(0),
        "an unusable git must exit nonzero:\n{stdout}"
    );
    let (_, no_git_json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("PATH", &no_git.display().to_string()),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&no_git_json).expect("json");
    let check = named_check(&report, "git");
    assert_eq!(check["status"], "fail", "the git check must fail: {check}");
    assert!(
        check["remedy"].as_str().is_some_and(|r| r.contains("git")),
        "the failing check must name its remedy: {check}"
    );
    assert!(
        stdout.contains("remedy:"),
        "the human report must print the remedy line:\n{stdout}"
    );

    // --- the daemon check, in all three of its states -------------------------
    //
    // Everything above runs against a data dir with no descriptor, where
    // `daemon_check` takes its "no daemon running" branch — which means the
    // check could have been a constant and every assertion so far would still
    // pass (probed: replacing its body with that one literal left the whole
    // suite green). Its other three branches are where an operator actually
    // needs it, so they are exercised here.
    let live_data = DataDir::new();
    // §4.2: `daemon` is estate-scoped, so the rig's daemon has to be started
    // *at* an estate root. It is deliberately not `estate` above: the
    // daemon check reads the descriptor in `live_data`, and nothing about
    // this arm should depend on the daemon and the doctor sharing an estate.
    let live_cwd = TempDir::new().expect("tempdir");
    empty_estate(live_cwd.path(), "doctor-live-daemon-estate");
    {
        let running = SpawnedDaemon::start(&live_data, live_cwd.path(), &[]);
        let (code, stdout, _) = doctor_in(
            estate.path(),
            live_data.path(),
            &[
                ("SGT_CLAUDE_BIN", &claude),
                ("SGT_DOCKER_BIN", &docker),
                ("HOME", &home),
            ],
            false,
        );
        assert_eq!(
            code,
            Some(0),
            "a healthy install with a daemon behind it must exit 0:\n{stdout}"
        );
        let (_, live_json, _) = doctor_in(
            estate.path(),
            live_data.path(),
            &[
                ("SGT_CLAUDE_BIN", &claude),
                ("SGT_DOCKER_BIN", &docker),
                ("HOME", &home),
            ],
            true,
        );
        let report: Value = serde_json::from_str(&live_json).expect("json");
        let check = named_check(&report, "daemon");
        assert_eq!(
            check["status"], "ok",
            "a live daemon is not a fault: {check}"
        );
        let detail = check["detail"].as_str().expect("detail");
        assert!(
            detail.contains("serving") && detail.contains(&running.endpoint),
            "the check must name the endpoint it actually reached: {detail}"
        );
        assert!(
            detail.contains(&format!("pid {}", running.child.id())),
            "…and the pid behind it, which is what makes the answer checkable: {detail}"
        );
    }
    // The daemon is gone now, but its descriptor may or may not have been
    // cleaned up on the way out; either way, a descriptor whose pid is gone is
    // the *stale* case, and it is a warning, not a failure — a mutating verb
    // (or `sgt daemon`) republishes it; observation verbs refuse instead
    // (ADR 0009).
    let stale = TempDir::new().expect("tempdir");
    write_descriptor(stale.path(), dead_pid(), "http://127.0.0.1:1");
    let (code, stdout, _) = doctor_in(
        estate.path(),
        stale.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        false,
    );
    assert_eq!(
        code,
        Some(0),
        "a stale descriptor is harmless and must not fail the install:\n{stdout}"
    );
    let (_, stale_json, _) = doctor_in(
        estate.path(),
        stale.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&stale_json).expect("json");
    let check = named_check(&report, "daemon");
    assert_eq!(
        check["status"], "warn",
        "a gone pid is stale, not broken: {check}"
    );
    assert!(
        check["detail"]
            .as_str()
            .is_some_and(|d| d.contains("stale")),
        "the warning must say what it saw: {check}"
    );

    // A pid that is alive while the endpoint refuses is the case a client
    // cannot resolve on its own, so the doctor calls it a failure.
    let occupied = TempDir::new().expect("tempdir");
    let mut squatter = Command::new("sleep")
        .arg("30")
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("spawn a process to stand in for a wedged daemon");
    write_descriptor(occupied.path(), squatter.id(), "http://127.0.0.1:1");
    let (code, stdout, _) = doctor_in(
        estate.path(),
        occupied.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        false,
    );
    assert_ne!(
        code,
        Some(0),
        "a live pid behind a dead endpoint must exit nonzero:\n{stdout}"
    );
    let (_, wedged_json, _) = doctor_in(
        estate.path(),
        occupied.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&wedged_json).expect("json");
    let check = named_check(&report, "daemon");
    assert_eq!(check["status"], "fail", "{check}");
    assert!(
        check["detail"]
            .as_str()
            .is_some_and(|d| d.contains("does not answer /healthz")),
        "the failure must say which half is wrong: {check}"
    );
    assert!(
        check["remedy"]
            .as_str()
            .is_some_and(|r| r.contains(&squatter.id().to_string())),
        "the remedy must name the pid the operator has to go look at: {check}"
    );
    let _ = squatter.kill();
    let _ = squatter.wait();

    // --- a broken data dir ----------------------------------------------------
    //
    // A regular file where the data dir should be. This is the fault that
    // reproduces as the same fault for root and non-root alike (permission
    // bits do not stop uid 0, and these tests run as root in this container).
    let blocked = bin.path().join("not-a-directory");
    std::fs::write(&blocked, b"this is a file").expect("write file");
    let (code, stdout, _) = doctor_in(
        estate.path(),
        &blocked,
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        false,
    );
    assert_ne!(code, Some(0), "an unusable data dir must exit nonzero");
    let (_, broken_json, _) = doctor_in(
        estate.path(),
        &blocked,
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&broken_json).expect("json");
    assert_eq!(report["healthy"], false);
    let check = named_check(&report, "data_dir");
    assert_eq!(
        check["status"], "fail",
        "the data_dir check must be the one that fails: {report}"
    );
    assert!(
        check["remedy"]
            .as_str()
            .is_some_and(|r| r.contains("SGT_DATA_DIR") || r.contains("--data-dir")),
        "the remedy must tell the operator where to point it instead: {check}"
    );
    assert!(
        stdout.contains("remedy:"),
        "human report must carry a remedy"
    );
}

/// TH-2 (BS2 test-honesty finding): t3 above pins the `permission_mode`
/// check's *name*, its position, `status == ok` and a non-empty `detail` —
/// but it runs doctor from a repo with no `sergeant.toml`, so the
/// profile-reporting branch of `permission_mode_check` never executes and
/// nothing ever reads what `detail` actually says. A doctor that reported a
/// constant ("unspecified -> no flag") for every profile regardless of its
/// real `permission_mode` passed the whole suite before this test existed.
/// This drives doctor from a estate with two profiles — one unspecified,
/// one `plan` — and asserts the detail names both profiles and their real
/// effective modes (#47's surfacing half).
#[test]
fn t3b_doctor_reports_the_effective_permission_mode_per_profile() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    // §6.1: `[[repo]] path` is gone and the mount is derived, so the
    // repository lives at `repos/solo` rather than being the estate root
    // itself (`path = "."`, the old shape). The manifest is written by hand
    // rather than scaffolded because the two `[[profile]]` tables are the
    // subject and `support::scaffold_estate` declares repositories only.
    support::init_repo(&estate.path().join("repos").join("solo"));
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\n\
         [[repo]]\nname = \"solo\"\n\n\
         [[profile]]\nname = \"quiet\"\nbackend = \"claude\"\n\n\
         [[profile]]\nname = \"careful\"\nbackend = \"claude\"\n\
         [profile.options]\npermission_mode = \"plan\"\n",
    )
    .expect("write sergeant.toml");

    let (code, stdout, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        false,
    );
    assert_eq!(code, Some(0), "a healthy install must exit 0:\n{stdout}");
    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "permission_mode");
    assert_eq!(check["status"], "ok", "{check}");
    let detail = check["detail"].as_str().expect("detail is a string");
    assert!(
        detail.contains("quiet") && detail.contains("careful"),
        "the detail must name every profile, not just count them: {detail}"
    );
    assert!(
        detail.contains("careful=plan"),
        "the profile with an explicit mode must report that mode's real CLI value: {detail}"
    );
    assert!(
        detail.contains("quiet=unspecified"),
        "the unspecified profile must be reported as unspecified/no-flag, \
         never silently folded into a mode it never chose: {detail}"
    );
    assert!(
        !detail.contains("quiet=plan") && !detail.contains("careful=unspecified"),
        "the two profiles' effective modes must not be swapped or constant-folded: {detail}"
    );
}

/// #262: `sgt doctor` reports the effective `network_access` override per
/// profile, mirroring `t3b_doctor_reports_the_effective_permission_mode_
/// per_profile` above — same reason: a misconfigured `network_access` must
/// be visible before any launch, not just refused at load.
#[test]
fn t3e_doctor_reports_the_effective_network_access_per_profile() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    support::init_repo(&estate.path().join("repos").join("solo"));
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\n\
         [[repo]]\nname = \"solo\"\n\n\
         [[profile]]\nname = \"quiet\"\nbackend = \"codex\"\n\n\
         [[profile]]\nname = \"careful\"\nbackend = \"codex\"\n\
         [profile.options]\nnetwork_access = \"true\"\n",
    )
    .expect("write sergeant.toml");

    let (code, stdout, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        false,
    );
    assert_eq!(code, Some(0), "a healthy install must exit 0:\n{stdout}");
    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "network_access");
    assert_eq!(check["status"], "ok", "{check}");
    let detail = check["detail"].as_str().expect("detail is a string");
    assert!(
        detail.contains("quiet") && detail.contains("careful"),
        "the detail must name every profile, not just count them: {detail}"
    );
    assert!(
        detail.contains("careful=true"),
        "the profile with an explicit override must report that value: {detail}"
    );
    assert!(
        detail.contains("quiet=unspecified"),
        "the unspecified profile must be reported as unspecified, \
         never silently folded into a value it never chose: {detail}"
    );
}

/// #262: a `claude` profile that sets `network_access` validates fine
/// (`claude.rs` never reads the option, so nothing about it can be
/// "unrecognized") but must not be reported as if it took effect —
/// `claude.rs`'s launch grammar has no sandbox network knob at all.
#[test]
fn t3e_doctor_reports_network_access_has_no_effect_on_a_claude_profile() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    support::init_repo(&estate.path().join("repos").join("solo"));
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\n\
         [[repo]]\nname = \"solo\"\n\n\
         [[profile]]\nname = \"terra\"\nbackend = \"claude\"\n\
         [profile.options]\nnetwork_access = \"false\"\n",
    )
    .expect("write sergeant.toml");

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "network_access");
    let detail = check["detail"].as_str().expect("detail is a string");
    assert!(
        detail.contains("terra="),
        "the detail must name the claude profile: {detail}"
    );
    assert!(
        !detail.contains("terra=false"),
        "a claude profile must never be rendered as if network_access took effect: {detail}"
    );
    assert!(
        detail.contains("false") && detail.contains("no effect"),
        "the configured value must still be named, alongside a plain statement that the claude \
         backend does not read it: {detail}"
    );
}

/// #262: a `codex` profile that sets `permission_mode` validates fine
/// (`codex.rs` never reads the option, so nothing about it can be
/// "unrecognized") but must not be reported as if it took effect —
/// `codex.rs`'s launch grammar has no `--permission-mode` equivalent at all.
/// Before this fix `permission_mode_check` rendered every profile's
/// configured mode identically regardless of backend, which is exactly the
/// dishonesty issue #262 filed: `sgt doctor` reported the profile healthy
/// from the string in `sergeant.toml` alone, never from what the adapter
/// actually does with it.
#[test]
fn t3d_doctor_reports_permission_mode_has_no_effect_on_a_codex_profile() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    support::init_repo(&estate.path().join("repos").join("solo"));
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\n\
         [[repo]]\nname = \"solo\"\n\n\
         [[profile]]\nname = \"terra\"\nbackend = \"codex\"\n\
         [profile.options]\npermission_mode = \"bypassPermissions\"\n",
    )
    .expect("write sergeant.toml");

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "permission_mode");
    let detail = check["detail"].as_str().expect("detail is a string");
    assert!(
        detail.contains("terra="),
        "the detail must name the codex profile: {detail}"
    );
    assert!(
        !detail.contains("terra=bypassPermissions"),
        "a codex profile must never be rendered as if bypassPermissions took effect: {detail}"
    );
    assert!(
        detail.contains("bypassPermissions") && detail.contains("no effect"),
        "the configured value must still be named, alongside a plain statement that the codex \
         backend does not read it: {detail}"
    );
}

/// MVP-3: `sgt doctor`'s estate check names a declared-but-missing
/// repository by name and reports its declared `origin` as the remedy — the
/// exact fact an operator needs ("clone it from here"), not a generic
/// "something is missing".
///
/// guard-map: a mutation that reports the missing repo's name but drops the
/// origin from the remedy (or reports a fixed "clone it" string) survives
/// every other doctor test but fails this one.
#[test]
fn t3c_doctor_estate_check_names_a_missing_repository_with_its_origin_as_the_remedy() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\n\
         [[repo]]\nname = \"payments\"\n\
         origin = \"https://example.com/payments.git\"\n",
    )
    .expect("write sergeant.toml");
    // No `path` key (§6.1 removed it): the mount this check reports missing
    // is the derived `repos/payments`, which is deliberately never created —
    // that absence is the point. §4.1 admission is structural and does not
    // resolve mounts, so the estate is still admitted and the `estate` row
    // is still the one that has to notice.

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "estate");
    assert_eq!(check["status"], "warn", "{check}");
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains("payments") && detail.contains("missing on disk"),
        "the missing repo must be named: {detail}"
    );
    let remedy = check["remedy"].as_str().expect("remedy");
    assert!(
        remedy.contains("https://example.com/payments.git"),
        "the declared origin must be the remedy, not a generic instruction: {remedy}"
    );
}

/// MVP-3: a directory under `repos/` the manifest never declared is flagged
/// by name, distinct from (and not confused with) the missing-repository
/// case above.
///
/// guard-map: dropping the `repos/` directory scan, or failing to exclude
/// declared names from it, makes this fail — either no fault is reported, or
/// the declared-and-present `known` repo is wrongly flagged too.
#[test]
fn t3d_doctor_estate_check_flags_an_undeclared_directory_under_repos() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    support::init_repo(&estate.path().join("repos").join("known"));
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\n[[repo]]\nname = \"known\"\n",
    )
    .expect("write sergeant.toml");
    // On disk but never declared in sergeant.toml. Both directories sit
    // under `repos/` because that is now the *only* place a mount can be
    // (§6.1) — which is what makes "declared but absent" and "present but
    // undeclared" two readings of one directory listing rather than two
    // unrelated searches.
    support::init_repo(&estate.path().join("repos").join("stray"));

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "estate");
    assert_eq!(check["status"], "warn", "{check}");
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains("stray") && detail.contains("not declared"),
        "the undeclared directory must be named: {detail}"
    );
    assert!(
        !detail.contains("known is") && !detail.contains("known/"),
        "the declared, present repository must not also be flagged: {detail}"
    );
    let remedy = check["remedy"].as_str().expect("remedy");
    assert!(
        remedy.contains("sgt repo add stray"),
        "the remedy must name the fixing command: {remedy}"
    );
}

/// MVP-3/E5: a manifest that fails to parse reports file *and* line — via
/// `toml::de::Error`'s own diagnostic riding `EstateError::Malformed`'s
/// `Display` — never a generic "invalid config".
///
/// **Estate-root §4.1/§4.2 moved which row carries it.** Admission is now the
/// first thing doctor asks, and a manifest that does not parse cannot be
/// admitted, so the `estate` row below it reads the already-decided "not an
/// estate root" answer and has nothing to say. The `estate_root` row is where
/// the parser's own diagnostic surfaces, split across `detail` (the
/// refusal's first line) and `remedy` (the rest, §4.4's own words). The
/// claim is unchanged and so is the mutation it kills: a doctor that
/// reported "invalid config" without the file or the line leaves an operator
/// with a broken estate and nowhere to look.
#[test]
fn t3e_doctor_estate_root_check_names_file_and_line_on_a_malformed_manifest() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n\nthis is not valid toml >>>\n",
    )
    .expect("write sergeant.toml");

    let (code, _, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        false,
    );
    assert_ne!(code, Some(0), "a broken manifest must exit nonzero");
    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "estate_root");
    assert_eq!(check["status"], "fail", "{check}");
    // The diagnostic is one §4.4 message split at its first newline, so the
    // file/line claim is about the row as a whole, not about which half the
    // renderer happened to put them in.
    let detail = check["detail"].as_str().expect("detail");
    let remedy = check["remedy"].as_str().expect("remedy");
    let rendered = format!("{detail}\n{remedy}");
    assert!(
        rendered.contains("sergeant.toml"),
        "the offending file must be named: {rendered}"
    );
    assert!(
        rendered.to_lowercase().contains("line"),
        "the parse error must carry a line number, not just \"invalid\": {rendered}"
    );
    // …and the row beneath it must not re-diagnose the same fault under its
    // own name: it reads `estate_root`'s already-decided answer, and there
    // is no admitted estate to check.
    let estate = named_check(&report, "estate");
    assert_eq!(
        estate["status"], "ok",
        "the estate row defers to the estate_root row above it: {estate}"
    );
}

/// #165: a freshly scaffolded estate with no `.sergeant/workflows/`
/// packages must be visible in `sgt doctor` as `0 workflow packages`, not
/// silent — the fact a submitter otherwise only learns from a 422 on
/// `sgt run --workflow <name>`. Once a package is declared, the check turns
/// `ok` and names it.
///
/// guard-map: dropping the `workflows` check (or wiring it to always report
/// `ok`) survives every other doctor test but fails this one; so does
/// counting a directory with no `workflow.toml` in it as a package.
#[test]
fn t3f_doctor_workflows_check_reports_zero_then_a_declared_package() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n",
    )
    .expect("write sergeant.toml");

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "workflows");
    assert_eq!(check["status"], "warn", "{check}");
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains("0 workflow packages"),
        "a fresh estate must report zero packages, not silence: {detail}"
    );
    assert!(check["remedy"].as_str().is_some(), "{check}");

    // A directory with no `workflow.toml` must not count as a package.
    std::fs::create_dir_all(estate.path().join(".sergeant/workflows/not-a-package"))
        .expect("mkdir");
    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "workflows");
    assert_eq!(
        check["status"], "warn",
        "a directory with no workflow.toml is not a package: {check}"
    );

    // A real package flips the check to `ok` and names it.
    let package_dir = estate.path().join(".sergeant/workflows/research");
    std::fs::create_dir_all(&package_dir).expect("mkdir");
    std::fs::write(
        package_dir.join("workflow.toml"),
        "[workflow]\nname = \"research\"\nversion = 1\nstages = []\n",
    )
    .expect("write workflow.toml");

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "workflows");
    assert_eq!(check["status"], "ok", "{check}");
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains("research") && detail.contains('1'),
        "the declared package must be named: {detail}"
    );
}

/// §6 Phase 3 (Ponytail R2 — reuse this check surface): `sgt doctor` reports
/// how many `.sergeant/local/workflows/` packages shadow a same-named stock
/// package, and flags any whose `edition` (ADR 0016) is older than the
/// binary's own version.
///
/// guard-map: dropping the drift computation (or comparing against the wrong
/// field) survives `t3f_doctor_workflows_check_reports_zero_then_a_declared_
/// package` but fails this one.
#[test]
fn t3g_doctor_workflows_check_reports_local_shadow_and_edition_drift() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let estate = TempDir::new().expect("tempdir");
    std::fs::write(
        estate.path().join("sergeant.toml"),
        "[estate]\nname = \"w\"\n",
    )
    .expect("write sergeant.toml");

    let stock = estate.path().join(".sergeant/workflows/research");
    std::fs::create_dir_all(&stock).expect("mkdir stock");
    std::fs::write(
        stock.join("workflow.toml"),
        "[workflow]\nname = \"research\"\nversion = 1\nstages = []\n",
    )
    .expect("write workflow.toml");

    // A local fork at the current binary edition: shadows stock, no drift.
    let current_edition = env!("CARGO_PKG_VERSION");
    let fresh_local = estate.path().join(".sergeant/local/workflows/research");
    std::fs::create_dir_all(&fresh_local).expect("mkdir local");
    std::fs::write(
        fresh_local.join("workflow.toml"),
        "[workflow]\nname = \"research\"\nversion = 1\nstages = []\n",
    )
    .expect("write workflow.toml");
    std::fs::write(
        fresh_local.join("index.md"),
        format!("---\nstatus: published\ndescription: d\nedition: {current_edition}\n---\n"),
    )
    .expect("write index.md");

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "workflows");
    assert_eq!(check["status"], "ok", "{check}");
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains("1 local package(s) shadow stock"),
        "must report the shadow count: {detail}"
    );
    assert!(
        detail.contains("research"),
        "must name the shadowing package: {detail}"
    );

    // Now make the local fork stale: an edition older than the binary's own.
    std::fs::write(
        fresh_local.join("index.md"),
        "---\nstatus: published\ndescription: d\nedition: 0.0.1\n---\n",
    )
    .expect("rewrite index.md with a stale edition");

    let (_, json, _) = doctor_in(
        estate.path(),
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "workflows");
    assert_eq!(
        check["status"], "warn",
        "a stale local edition must warn, not fail (it still runs): {check}"
    );
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains("1 older than the current edition"),
        "must report the drift count: {detail}"
    );
    assert!(
        detail.contains("research"),
        "must name the drifted package: {detail}"
    );
    let remedy = check["remedy"].as_str().expect("remedy");
    assert!(
        remedy.contains("sgt workflow fork"),
        "the remedy must name the fork verb: {remedy}"
    );
}

/// #67: an unwritable *parent* of the data dir used to produce two
/// confusing, apparently-independent downstream symptoms — `docker` failing
/// to open its blob store under the data dir (EPERM) and `disk_pressure`'s
/// `df` call failing because the data dir never got created (ENOENT,
/// misreported as "could not be measured on this platform", which isn't
/// even true — the platform can measure free space fine). Both are the same
/// root cause. This drives `sgt doctor` against a data dir whose parent is
/// stripped of write permission and requires that fault collapse into one
/// `data_dir` FAIL row naming the actual offending parent directory, with
/// `docker` and `disk_pressure` declining rather than re-diagnosing it under
/// their own, more confusing names.
///
/// guard-map: reverting the fix restores the old check order (`docker`
/// before `data_dir`) and the un-consolidated `docker`/`disk_pressure`
/// bodies — this fails on the reverted code because `data_dir`'s detail
/// never names the parent directory and `docker`/`disk_pressure` print
/// their own confusing messages instead of deferring.
#[test]
fn t3f_doctor_names_an_unwritable_parent_as_one_remedy_row() {
    use std::os::unix::fs::PermissionsExt;

    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());

    let parent = TempDir::new().expect("tempdir");
    // The data dir itself must not exist yet — `create_dir_all` has to walk
    // up past it and find this unwritable directory stalling the walk.
    let data_dir = parent.path().join("child").join("data");

    std::fs::set_permissions(parent.path(), std::fs::Permissions::from_mode(0o555))
        .expect("chmod parent read+exec only");
    struct RestorePerms(PathBuf);
    impl Drop for RestorePerms {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o755));
        }
    }
    let _restore = RestorePerms(parent.path().to_path_buf());

    // Environment probe (CONTRIBUTING.md's testing rules): the root dev
    // container silently ignores permission-bit restrictions, so this
    // fixture cannot be armed there — skip loudly rather than assert
    // something that cannot hold in that environment.
    if std::fs::create_dir(parent.path().join("probe")).is_ok() {
        eprintln!(
            "SKIPPED-ENV: permission bits are not enforced on this host (root container \
             shape) — the unwritable-parent fault cannot be armed here"
        );
        return;
    }

    let (code, stdout, _) = doctor(
        &data_dir,
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        false,
    );
    assert_ne!(
        code,
        Some(0),
        "an unwritable parent must exit nonzero:\n{stdout}"
    );

    let (_, json, _) = doctor(
        &data_dir,
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");

    let parent_str = parent.path().display().to_string();

    let data_dir_check = named_check(&report, "data_dir");
    assert_eq!(data_dir_check["status"], "fail", "{data_dir_check}");
    let detail = data_dir_check["detail"].as_str().expect("detail");
    assert!(
        detail.contains(&parent_str),
        "the data_dir check must name the actual offending parent directory, not just the \
         data dir it could not create: {detail}"
    );
    let remedy = data_dir_check["remedy"].as_str().expect("remedy");
    assert!(
        remedy.contains(&parent_str),
        "the remedy must point at the parent directory to fix, not a generic instruction: \
         {remedy}"
    );

    // The two downstream co-failures decline instead of re-diagnosing the
    // same fault under their own, more confusing names.
    let docker_check = named_check(&report, "docker");
    assert_ne!(
        docker_check["status"], "ok",
        "docker cannot succeed when the data dir it opens a blob store under does not exist: \
         {docker_check}"
    );
    assert!(
        !docker_check["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("could not initialize the Docker adapter"),
        "docker must defer to the data_dir check above rather than re-diagnosing the same \
         EPERM under its own name: {docker_check}"
    );

    let disk_pressure_check = named_check(&report, "disk_pressure");
    assert_ne!(disk_pressure_check["status"], "ok", "{disk_pressure_check}");
    assert!(
        !disk_pressure_check["detail"]
            .as_str()
            .unwrap_or_default()
            .contains("could not be measured on this platform"),
        "disk_pressure must not claim the platform can't measure free space when the real \
         cause is the parent directory data_dir already named: {disk_pressure_check}"
    );
}

/// ADR 0006's residual hole (#100): a toolchain directory that exists on
/// disk but is missing from `PATH` is #60's exact failure shape — an actor
/// that cannot find `cargo` and misreads it as a permissions fault. This
/// arms it deterministically by pointing `HOME` at a fixture directory that
/// has `.cargo/bin` on disk but never touching `PATH`, so the real test
/// process's PATH (which does not contain this fresh tempdir) still cannot
/// see it.
///
/// guard-map: a mutation that always reports `ok` regardless of PATH, or one
/// that checks a different directory than
/// `harness::toolchain_path_dirs` actually composes, survives every other
/// doctor test but fails this one.
#[test]
fn t3g_doctor_environment_check_flags_a_toolchain_dir_missing_from_path() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    let home = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(home.path().join(".cargo").join("bin")).expect("mkdir cargo bin");

    let (_, json, _) = doctor(
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home.path().display().to_string()),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "environment");
    assert_eq!(check["status"], "warn", "{check}");
    let cargo_bin = home.path().join(".cargo").join("bin").display().to_string();
    let detail = check["detail"].as_str().expect("detail");
    assert!(
        detail.contains(&cargo_bin),
        "the missing directory must be named: {detail}"
    );
    let remedy = check["remedy"].as_str().expect("remedy");
    assert!(
        remedy.contains("sgt claude"),
        "the remedy must name the passthrough that would have composed it: {remedy}"
    );
}

/// The control for [`t3g_doctor_environment_check_flags_a_toolchain_dir_missing_from_path`]:
/// when neither toolchain directory exists on disk at all, there is nothing
/// to enrich and the check must read `ok`, not warn about a directory the
/// host never had in the first place.
#[test]
fn t3h_doctor_environment_check_is_ok_when_no_toolchain_dirs_exist() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    // A fresh tempdir with nothing under it — no `.cargo/bin`, no `.local/bin`.
    let home = TempDir::new().expect("tempdir");

    let (_, json, _) = doctor(
        data.path(),
        &[
            ("SGT_CLAUDE_BIN", &claude),
            ("SGT_DOCKER_BIN", &docker),
            ("HOME", &home.path().display().to_string()),
        ],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("doctor --json is json");
    let check = named_check(&report, "environment");
    assert_eq!(check["status"], "ok", "{check}");
    assert!(
        check["remedy"].is_null(),
        "a green check has nothing to remedy: {check}"
    );
}

/// The journal check's *other* failure arm: `corrupt_journal` above only
/// ever reaches "replay failed after N events" (a line the replay gets to
/// and cannot parse). `Journal::replay_data_dir` can also fail before it
/// reads a single line — the journal directory itself is unusable — and that
/// is a different branch in `journal_check` with a different message
/// ("cannot open the journal" vs. "replay failed after…"). A regular file
/// where `journal/` should be a directory is the dir-as-file trick that
/// reaches it: `.exists()` is still true (so the check does not take the
/// fresh-install "no journal yet" branch), but `read_dir` on it fails.
#[test]
fn doctor_reports_a_journal_it_cannot_even_open() {
    let bin = TempDir::new().expect("tempdir");
    let claude = stub_claude(bin.path());
    let docker = stub_docker(bin.path());
    let data = TempDir::new().expect("tempdir");
    std::fs::create_dir_all(data.path()).expect("data dir");
    std::fs::write(data.path().join("journal"), b"not a directory").expect("write file");

    let (code, stdout, _) = doctor(
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        false,
    );
    assert_ne!(
        code,
        Some(0),
        "a journal directory that is a file must fail the install"
    );
    let (_, json, _) = doctor(
        data.path(),
        &[("SGT_CLAUDE_BIN", &claude), ("SGT_DOCKER_BIN", &docker)],
        true,
    );
    let report: Value = serde_json::from_str(&json).expect("json");
    assert_eq!(report["healthy"], false);
    let check = named_check(&report, "journal");
    assert_eq!(check["status"], "fail", "{check}");
    assert!(
        check["detail"]
            .as_str()
            .is_some_and(|d| d.contains("cannot open the journal")),
        "this is the open failure, not the replay-failed-partway-through one: {check}"
    );
    assert!(
        check["remedy"].is_string(),
        "the open failure must name a remedy too: {check}"
    );
    assert!(stdout.contains("remedy:"));

    // The check downstream of it declines for the same reason a torn
    // journal makes it decline: nothing was replayed to build a projection
    // from.
    let projection = named_check(&report, "projection");
    assert_eq!(projection["status"], "warn", "{projection}");
    assert!(
        projection["detail"]
            .as_str()
            .is_some_and(|d| d.contains("not attempted")),
        "{projection}"
    );
}

/// The parts of a doctor report that must not move between runs.
fn shape(report: &Value) -> Value {
    json!({
        "keys": report.as_object().expect("object").keys().collect::<Vec<_>>(),
        "checks": report["checks"].as_array().expect("checks").iter().map(|c| json!({
            "keys": c.as_object().expect("object").keys().collect::<Vec<_>>(),
            "name": c["name"],
            "status": c["status"],
        })).collect::<Vec<_>>(),
    })
}

/// Give a data dir a real journal, without a daemon: doctor is a read-only
/// diagnostic and must work on a data dir whose owner is not running.
fn seed_journal(data_dir: &Path, events: u32) {
    let mut journal = Journal::open(data_dir).expect("open journal");
    for n in 0..events {
        journal
            .append(EventDraft::new(
                EventSource::new("test", "doctor-fixture"),
                "test.seeded",
                json!({"n": n}),
            ))
            .expect("append");
    }
}

/// Corrupt the journal's newest segment so replay cannot validate it.
fn corrupt_journal(data_dir: &Path) {
    let mut segments: Vec<PathBuf> = std::fs::read_dir(data_dir.join("journal"))
        .expect("journal dir")
        .filter_map(|entry| {
            let path = entry.expect("entry").path();
            (path.extension().is_some_and(|e| e == "ndjson")).then_some(path)
        })
        .collect();
    segments.sort();
    let segment = segments.pop().expect("at least one segment");
    let mut text = std::fs::read_to_string(&segment).expect("read segment");
    text.push_str("{ this is not an event }\n");
    std::fs::write(&segment, text).expect("write segment");
}

/// Every event seq in a journal directory, read straight off the NDJSON —
/// the same durable record the graph's edges cite.
fn journal_seqs(journal: &Path) -> Vec<u64> {
    let mut seqs = Vec::new();
    for entry in std::fs::read_dir(journal).expect("journal dir") {
        let path = entry.expect("entry").path();
        if path.extension().is_none_or(|e| e != "ndjson") {
            continue;
        }
        for line in std::fs::read_to_string(&path)
            .expect("read segment")
            .lines()
        {
            if line.trim().is_empty() {
                continue;
            }
            let event: Value = serde_json::from_str(line).expect("a journal line is an event");
            seqs.push(event["seq"].as_u64().expect("every event carries a seq"));
        }
    }
    seqs
}

/// Publish a runtime descriptor by hand, so the doctor's three daemon states
/// can be produced without three daemons.
fn write_descriptor(data_dir: &Path, pid: u32, endpoint: &str) {
    std::fs::create_dir_all(data_dir).expect("mkdir");
    let descriptor = json!({
        "schema": daemon::DESCRIPTOR_SCHEMA,
        "endpoint": endpoint,
        "pid": pid,
        "api_revision": "v1",
        "token": ulid(),
    });
    std::fs::write(
        daemon::descriptor_path(data_dir),
        serde_json::to_vec_pretty(&descriptor).expect("serialize"),
    )
    .expect("write descriptor");
}

/// A pid that is certainly not running: a child, reaped, and its number then
/// re-checked. (Linux allocates pids sequentially, so a just-freed number is
/// the last one the kernel will hand out again.)
fn dead_pid() -> u32 {
    let mut child = Command::new("true").spawn().expect("spawn");
    let pid = child.id();
    child.wait().expect("wait");
    let deadline = Instant::now() + Duration::from_secs(5);
    while daemon::pid_alive(pid) {
        assert!(Instant::now() < deadline, "pid {pid} never went away");
        std::thread::sleep(Duration::from_millis(20));
    }
    pid
}

fn named_check<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("no {name} check in {report}"))
}

fn doctor(data_dir: &Path, env: &[(&str, &str)], as_json: bool) -> (Option<i32>, String, String) {
    doctor_in_opt(None, data_dir, env, as_json)
}

/// [`doctor`], run with the subprocess's cwd set to `estate`.
///
/// Estate-root §4.1: the effective root of an invocation is `-C <path>` if
/// given, else the process's own working directory — and `sgt doctor` asks
/// `Estate::admit` about exactly that directory, with no ancestor walk to
/// fall back on. So the cwd is what decides whether the `estate_root`,
/// `permission_mode`, `estate` and `workflows` rows have anything to report,
/// and setting it on the subprocess (rather than the test process, which is
/// global state) is how these tests choose.
fn doctor_in(
    estate: &Path,
    data_dir: &Path,
    env: &[(&str, &str)],
    as_json: bool,
) -> (Option<i32>, String, String) {
    doctor_in_opt(Some(estate), data_dir, env, as_json)
}

fn doctor_in_opt(
    estate: Option<&Path>,
    data_dir: &Path,
    env: &[(&str, &str)],
    as_json: bool,
) -> (Option<i32>, String, String) {
    let mut command = Command::new(SGT);
    if let Some(estate) = estate {
        command.current_dir(estate);
    }
    command.arg("--data-dir").arg(data_dir);
    if as_json {
        command.arg("--json");
    }
    command.arg("doctor");
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("run sgt doctor");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// ------------------------------------------------------------------ 3. demo

/// Acceptance 4. The §39 walkthrough runs, exits 0, and the evidence it
/// pointed at is really there.
#[test]
fn t4_the_section_39_demo_runs_and_its_evidence_resolves() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("scripts/demo.sh");
    assert!(script.exists(), "scripts/demo.sh must exist");

    let home = TempDir::new().expect("tempdir");
    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        // A clean environment: no inherited data dir, no inherited fake
        // script, and a HOME of its own so nothing reaches the developer's
        // real `~/.local/share/sergeant`.
        .env_remove("SGT_DATA_DIR")
        .env_remove("SGT_FAKE_SCRIPT")
        .env("HOME", home.path())
        .env("SGT_BIN", SGT)
        .env("KEEP_DEMO_DIR", "1")
        .output()
        .expect("run scripts/demo.sh");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "the walkthrough must exit 0\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    );

    // The whole §39 arc, in order — a script that exits 0 having skipped half
    // the walkthrough is not the deliverable.
    let mut cursor = 0usize;
    for marker in [
        "a developer clones a repository",
        "they ask for work",
        "sergeant routed it, cut a work surface",
        "the first stage runs and stops to ask a question",
        "the developer answers",
        // §39's arc is "stages", plural, and this is the element that makes
        // it so: the review is dispatched as another execution. It always
        // ran; until this marker existed the walkthrough never said so, and
        // nothing would have noticed it going quiet again.
        "the review ran as a second, independent execution",
        "the surface was retired",
        "where the evidence lives",
        "the walkthrough held",
    ] {
        let found = stdout[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("the walkthrough never reached {marker:?}:\n{stdout}"));
        cursor += found + marker.len();
    }
    assert!(
        stdout.contains("submit → surface → stages → needs_input → respond → completed → retire"),
        "the closing line states the arc that was proven"
    );

    // Independent resolution of the pointers it printed.
    let demo_dir = stdout
        .lines()
        .find_map(|line| line.strip_prefix("demo directory kept at "))
        .expect("KEEP_DEMO_DIR must report where it kept things")
        .trim();
    let demo_dir = PathBuf::from(demo_dir);
    let journal = demo_dir.join("data/journal");
    assert!(
        journal.is_dir(),
        "the journal pointer must resolve: {journal:?}"
    );
    assert!(
        !journal_seqs(&journal).is_empty(),
        "the journal the demo pointed at must hold events"
    );
    // The other three pointers, resolved from the artifacts the run actually
    // fetched rather than from the prose it printed about them.
    //
    // The daemon is stopped by the script's own trap before this test resumes,
    // so its endpoint cannot be re-queried — but every answer it gave was
    // written to a file under the kept directory, and those files are outside
    // the script's narration. Grepping stdout for a sentence the script itself
    // emitted grades the script on its own homework; reading `graph.json` and
    // checking its edges against the journal does not.
    let work_id = stdout
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("the client auto-spawned a daemon, then submitted work ")
        })
        .expect("the run must name the work it submitted")
        .trim()
        .to_string();

    let graph: Value = serde_json::from_str(
        &std::fs::read_to_string(demo_dir.join("graph.json")).expect("the graph answer was kept"),
    )
    .expect("graph.json is json");
    assert_eq!(graph["work_id"], Value::String(work_id.clone()));
    let edges = graph["edges"].as_array().expect("edges");
    assert!(
        !edges.is_empty(),
        "the graph pointer answered with no edges"
    );
    let seqs: Vec<u64> = journal_seqs(&journal);
    for edge in edges {
        let seq = edge["source_seq"].as_u64().unwrap_or_else(|| {
            panic!("every edge must carry the journal seq that justifies it: {edge}")
        });
        assert!(
            seqs.contains(&seq),
            "edge {edge} cites journal seq {seq}, which is not in the journal the run kept"
        );
    }

    // The stage narration, resolved the same way: every line the step printed
    // names an execution and a journal seq, and both must be real. A run that
    // printed one stage, or printed the same execution twice, would be
    // describing a single prompt while calling it a workflow.
    let narrated: Vec<(String, u64)> = stdout
        .lines()
        .filter_map(|line| line.trim().strip_prefix("stage "))
        .filter_map(|line| {
            let execution = line.split(" — execution ").nth(1)?;
            let (execution, rest) = execution.split_once(" on ")?;
            let seq = rest.split("journal seq ").nth(1)?.trim_end_matches(')');
            Some((execution.to_string(), seq.parse::<u64>().ok()?))
        })
        .collect();
    assert_eq!(
        narrated.len(),
        2,
        "the walkthrough must narrate both stages of the two-stage workflow:\n{stdout}"
    );
    assert_ne!(
        narrated[0].0, narrated[1].0,
        "§39's review is *another execution*, not a later turn of the first one"
    );
    for (execution, seq) in &narrated {
        assert!(
            seqs.contains(seq),
            "the narration cites journal seq {seq} for execution {execution}, which is \
             not in the journal the run kept"
        );
    }

    let analytics: Value = serde_json::from_str(
        &std::fs::read_to_string(demo_dir.join("analytics.json"))
            .expect("the analytics answer was kept"),
    )
    .expect("analytics.json is json");
    assert_eq!(
        analytics["question"], "How long does work remain blocked?",
        "the kept answer must be the query the script named"
    );
    let rows = analytics["rows"].as_array().expect("rows");
    assert!(
        !rows.is_empty(),
        "the analytics pointer must have *answered*, not just printed its question — \
         the question prints ahead of the rows, so zero rows looks identical in stdout: \
         {analytics}"
    );
    assert!(
        rows.iter()
            .any(|row| row[0] == Value::String(work_id.clone())),
        "the projection must have been fed this run's work: {analytics}"
    );

    // The demo daemon is asked to stop by the script's own trap; it must not
    // still be holding the data dir it printed about.
    assert!(
        !demo_dir.join("data/runtime.json").exists(),
        "the walkthrough must not leave a daemon advertising a temp data dir"
    );
    std::fs::remove_dir_all(&demo_dir).expect("clean up the kept demo dir");

    // --- the `--real-claude` shape, without spending a token ------------------
    //
    // The flag's own guard first: it must refuse before it can spend anything.
    let refused = Command::new("bash")
        .arg(&script)
        .arg("--real-claude")
        .current_dir(&root)
        .env_remove("SERGEANT_CLAUDE_TESTS")
        .env("HOME", home.path())
        .env("SGT_BIN", SGT)
        .output()
        .expect("run scripts/demo.sh --real-claude");
    assert!(
        !refused.status.success(),
        "--real-claude must refuse without the token budget flag"
    );
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("SERGEANT_CLAUDE_TESTS=1"),
        "…and must say which discipline it is enforcing: {}",
        String::from_utf8_lossy(&refused.stderr)
    );

    // Then the control flow that flag selects. `--real-claude` differs from
    // the default run in exactly two ways — a raised client timeout, and a
    // walkthrough that does not insist on the `needs_input` pause, because
    // P0's Claude adapter emits no such signal (only the fake backend scripts
    // one, and this milestone does not touch the adapter). The second is the
    // branch that used to be unreachable in every test: the script's
    // unconditional `[ "$STATE" = "needs_input" ]` meant a non-pausing backend
    // could only ever reach `fail`. Supplying a non-pausing fake script takes
    // the same branch for free, so the arc is proven, not asserted.
    let straight = TempDir::new().expect("tempdir");
    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .env_remove("SGT_DATA_DIR")
        .env("HOME", straight.path())
        .env("SGT_BIN", SGT)
        .env("SGT_FAKE_SCRIPT", "complete:implemented;complete:reviewed")
        .output()
        .expect("run scripts/demo.sh with a non-pausing backend");
    let straight_stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        output.status.success(),
        "the walkthrough must hold for a backend that never pauses — this is the path \
         --real-claude takes\n--- stdout ---\n{straight_stdout}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        straight_stdout.contains("submit → surface → stages → completed → retire"),
        "…and it must state the arc it actually proved, not the one with a pause \
         in it:\n{straight_stdout}"
    );
    assert!(
        straight_stdout.contains("without asking"),
        "…and say out loud that the pause did not happen, rather than skip it \
         quietly:\n{straight_stdout}"
    );
}

/// The §34 TUI stack, as the milestone actually spends it.
///
/// The contract's dependency budget names "ratatui + crossterm". Only
/// `ratatui` is declared: a second `crossterm` entry is a second copy of the
/// same crate that Cargo may resolve to a different version, at which point
/// the `KeyEvent` this code matches on is not the one ratatui's backend
/// produces. The narrowing is deliberate and rung-logged in `Cargo.toml`;
/// this is what keeps it that way — adding the direct dependency, or reaching
/// for crossterm by any path other than ratatui's re-export, fails here and
/// has to be argued for.
#[test]
fn the_tui_stack_is_ratatui_with_crossterm_reached_through_it() {
    let manifest =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("read Cargo.toml");
    let declared: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("crossterm"))
        .collect();
    assert!(
        declared.is_empty(),
        "crossterm must not be a direct dependency — it is ratatui's, and one \
         resolution of it is the whole point: {declared:?}"
    );
    assert!(
        manifest.contains("\nratatui = "),
        "ratatui is the §34-named TUI dependency and must stay declared"
    );

    let tui = code_only(&read_tui_source());
    assert!(
        tui.contains("ratatui::crossterm::"),
        "the TUI must reach crossterm through ratatui's re-export"
    );
    for line in tui.lines().map(str::trim) {
        assert!(
            !line.starts_with("use crossterm"),
            "a bare `crossterm` path would compile only against a second, separately \
             resolved copy of the crate: {line}"
        );
    }
}

/// The one client knob the walkthrough sets, checked by name.
///
/// `SGT_CLIENT_TIMEOUT_SECS` is read in exactly one place (`api::client_timeout`)
/// and written in exactly one place (`scripts/demo.sh`, inside the
/// `--real-claude` branch this suite deliberately does not run). Nothing else
/// would notice the two spellings drifting apart, and the failure mode is
/// silent: the demo exports a variable nobody reads and the run dies on the
/// ten-second default instead. The parse itself is unit tested in `src/api.rs`;
/// this is the other half — that the two ends still name the same knob.
#[test]
fn t4_the_demo_and_the_client_name_the_same_timeout_knob() {
    let script =
        std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/demo.sh"))
            .expect("read scripts/demo.sh");
    let knob = sergeant_rs::api::CLIENT_TIMEOUT_ENV;
    assert!(
        script.contains(&format!("export {knob}=")),
        "scripts/demo.sh must export {knob} — the variable the client actually \
         reads — for the path that waits on a real model"
    );
}

/// …and an operator who sets that knob and does not get it is *told*, on the
/// stderr of the process they ran (issue #24).
///
/// The rule itself (raise-only) and the warning's wording are unit tested in
/// `src/api.rs`. What only a real client process can show is that the warning
/// is printed at all: `client_timeout` can compute it and drop it on the
/// floor, which restores exactly the silence the issue is about — a knob that
/// behaves identically whether you set it, mistype it, or leave it alone,
/// with nothing anywhere naming the timeout in force. So this runs a client
/// with the override set to a value below the default and reads its stderr.
#[test]
fn t4_a_client_says_out_loud_when_it_ignores_the_timeout_knob() {
    let data = DataDir::new();
    let knob = sergeant_rs::api::CLIENT_TIMEOUT_ENV;
    // `run` rather than `status`: ADR 0009 moved `status` into the no-spawn
    // set, and with no daemon running it would refuse before ever
    // constructing the `ApiClient` this warning comes from. A mutating verb
    // still auto-spawns and reaches the same `ApiClient::new` call.
    //
    // From an estate root, because §4.3 puts exact-root validation ahead of
    // the auto-spawn: outside one, `run` refuses before `ensure_daemon` ever
    // builds a client, and this test would be asserting about a warning no
    // code path had the chance to emit. The estate declares no repositories,
    // which is enough to be admitted (§4.1) and enough to reach the client —
    // the daemon's own 422 afterward is not this test's subject.
    let estate = TempDir::new().expect("tempdir");
    empty_estate(estate.path(), "timeout-knob-estate");
    let output = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .arg("run")
        .arg("timeout knob probe")
        .current_dir(estate.path())
        .env(knob, "5")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run a client with the override set");
    data.reap();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let said: Vec<&str> = stderr.lines().filter(|line| line.contains(knob)).collect();
    assert_eq!(
        said.len(),
        1,
        "a client that ignored {knob} must say so exactly once on stderr — \
         once, because the warning is about this process's configuration, not \
         about each client it builds. stderr was: {stderr:?}"
    );
    let said = said[0];
    assert!(
        said.contains("\"5\"") && said.contains("10s"),
        "the warning must name the value it ignored and the timeout actually in \
         force, or the operator is no better off: {said}"
    );
}

/// The same client, with nothing to complain about, complains about nothing.
///
/// Half of the value of a warning is that it is not always there; a client
/// that printed this line unconditionally would be noise an operator learns
/// to skip past, and the assertion above would still pass.
#[test]
fn t4_a_client_that_applies_the_knob_is_quiet_about_it() {
    let data = DataDir::new();
    let knob = sergeant_rs::api::CLIENT_TIMEOUT_ENV;
    // Same reasoning as the sibling test above, on both counts: `run`, not
    // `status`, and from an estate root so the client is actually built.
    // This half needs the estate more than the other one does — a refusal
    // before `ApiClient::new` produces no warning either, so outside an
    // estate "an override that was applied is not a complaint" would hold
    // vacuously against a build that never applied anything.
    let estate = TempDir::new().expect("tempdir");
    empty_estate(estate.path(), "timeout-knob-estate");
    let output = Command::new(SGT)
        .arg("--data-dir")
        .arg(data.path())
        .arg("run")
        .arg("timeout knob probe")
        .current_dir(estate.path())
        .env(knob, "300")
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run a client with the override set");
    data.reap();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        !stderr.contains(knob),
        "an override that *was* applied is not a complaint: {stderr:?}"
    );
}

// -------------------------------------------------------- 4. clients equal

/// Acceptance 4. §7/§30's clients-are-equal rule: `tui.rs` reaches state
/// only through the API's own types.
///
/// `web.rs` used to be scanned alongside it here, with a second,
/// compile-time half pinning `ApiViews`'s private field and exact method
/// set — both gone with the dashboard (ADR 0011). What remains is the path
/// scan alone, the same shape M5's t2 used for the DuckDB owner, which
/// catches an import however it is spelled (see [`crate_paths`], and the
/// test that fools it, `t5b` below).
///
/// The trailing token denylist is a backstop only. It is not the net: the net
/// is the path scan, because a name-based list can only forbid the daemon
/// internals somebody thought to name, and `BlobStore` was not one of them.
#[test]
fn t5_the_tui_is_a_client_like_any_other() {
    let source = code_only(&read_tui_source());
    let paths = crate_paths(&source);
    assert_eq!(
        paths,
        vec!["api".to_string()],
        "the tui module tree may reach the crate only through `crate::api` — \
         the API client surface — but names: {paths:?}"
    );
    for forbidden in [
        "ApiState",
        "registry",
        "Journal",
        "Analytics",
        "Engine",
        "blocking_lock",
    ] {
        assert!(
            !names_token(&source, forbidden),
            "tui.rs names {forbidden}: a client must not know the daemon's insides"
        );
    }

    // The positive half: the file must actually be a client, or the rule
    // above is satisfied by a module that renders nothing.
    assert!(
        names_token(&source, "ApiClient"),
        "tui.rs must reach state through the API client"
    );
}

/// The guard above, guarded. An instrument nobody has tried to fool is a
/// claim, not a measurement — so the spellings that *would* hide a reach into
/// daemon internals are stated here as data and required to be visible.
///
/// Each of these was first written into a disposable copy of `src/tui.rs`,
/// where it compiled and ran; the braced form in particular left the
/// structural test green while the TUI held a handle on the daemon's blob
/// store. That is the regression this locks down.
#[test]
fn t5b_the_structural_scan_sees_every_spelling_of_a_path() {
    for (source, expected) in [
        ("use crate::api::ApiClient;", vec!["api"]),
        (
            "use crate::{api::{ApiClient, ClientError}, runtime::blob::BlobStore};",
            vec!["api", "runtime"],
        ),
        (
            "use crate::{\n    runtime::journal::Journal,\n};",
            vec!["runtime"],
        ),
        ("use super::daemon::descriptor_path;", vec!["daemon"]),
        (
            "use super::*;\nfn f(v: &crate::api::EventStream) {}",
            vec!["api"],
        ),
        ("let x = crate::daemon::pid_alive(7);", vec!["daemon"]),
        ("use crate::{self as me};", vec!["self"]),
    ] {
        assert_eq!(
            crate_paths(source),
            expected,
            "the scan must see {source:?} for what it is"
        );
    }
    // …and it must not invent a reach out of a test module's `use super::*`,
    // which the client has.
    assert!(
        crate_paths("mod tests { use super::*; use super::field; }").is_empty(),
        "`super::` inside a module's own tests names that module, not the crate"
    );
}

/// The SSE stream publishes a vocabulary, and exactly one place states it.
///
/// `send_sse` names every frame with the journal's event kind, so a client
/// that wants the whole tail must enumerate the kinds. Before this test the
/// embedded dashboard (deleted, ADR 0011) kept its own copy of the list and
/// it was five kinds stale — `surface.materializing` reached no listener at
/// all. The list lives in `api::SSE_EVENT_KINDS`, stated once for every
/// current and future browser-based consumer of the stream; this is what
/// keeps it complete: add a `KIND_*` to the crate without adding it here and
/// this fails.
#[test]
fn t6_the_sse_vocabulary_is_stated_once_and_stays_complete() {
    let declared = declared_event_kinds();
    assert!(
        declared.len() > 30,
        "the scan found suspiciously few kinds ({}); it has stopped matching the \
         declarations it is meant to read",
        declared.len()
    );
    for (kind, origin) in &declared {
        assert!(
            sergeant_rs::api::SSE_EVENT_KINDS.contains(&kind.as_str()),
            "{origin} declares the event kind {kind:?}, which the daemon will name an SSE \
             frame with, but api::SSE_EVENT_KINDS does not list it — every client that \
             enumerates frame names would silently drop it"
        );
    }
    let names: Vec<&str> = declared.iter().map(|(kind, _)| kind.as_str()).collect();
    for kind in sergeant_rs::api::SSE_EVENT_KINDS.iter() {
        assert!(
            names.contains(kind),
            "api::SSE_EVENT_KINDS lists {kind:?}, which no KIND_* constant declares"
        );
    }
    let mut seen = sergeant_rs::api::SSE_EVENT_KINDS.to_vec();
    seen.sort_unstable();
    let before = seen.len();
    seen.dedup();
    assert_eq!(before, seen.len(), "the vocabulary must not repeat a kind");
}

/// W4 §1.1.2: the SSE stream's two control frames (`sergeant.floor`,
/// `sergeant.stream_error`) are deliberately outside `SSE_EVENT_KINDS`'s
/// vocabulary — neither is a `KIND_*` constant, and neither appears in the
/// journaled-kinds list. Extends `t6`'s own bidirectional scan rather than
/// duplicating it.
#[test]
fn sse_control_frames_are_disjoint_from_kind_star_and_from_sse_event_kinds() {
    let declared = declared_event_kinds();
    let names: Vec<&str> = declared.iter().map(|(kind, _)| kind.as_str()).collect();
    for frame in sergeant_rs::api::SSE_CONTROL_FRAMES {
        assert!(
            !names.contains(frame),
            "{frame:?} is in SSE_CONTROL_FRAMES but also declared as a KIND_* constant — a \
             control frame must never collide with a journaled event kind"
        );
        assert!(
            !sergeant_rs::api::SSE_EVENT_KINDS.contains(frame),
            "{frame:?} is in SSE_CONTROL_FRAMES but also listed in SSE_EVENT_KINDS — the two \
             vocabularies must stay disjoint"
        );
    }
}

/// Every `pub const KIND_… : &str = "…";` the crate declares, with the file
/// that declares it.
fn declared_event_kinds() -> Vec<(String, String)> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|e| e == "rs") {
                let source = std::fs::read_to_string(&path).expect("read source");
                for line in source.lines() {
                    let Some(rest) = line.trim().strip_prefix("pub const KIND_") else {
                        continue;
                    };
                    let Some((_, literal)) = rest.split_once(" = \"") else {
                        continue;
                    };
                    let Some((kind, _)) = literal.split_once('"') else {
                        continue;
                    };
                    found.push((
                        kind.to_string(),
                        path.file_name()
                            .expect("name")
                            .to_string_lossy()
                            .to_string(),
                    ));
                }
            }
        }
    }
    found
}

/// §22.6, structurally: every external effect the engine performs lives
/// inside a pending-effect performer, and nowhere a `&mut Core` can reach.
///
/// INV-N3-08's finding about the wave-1 verdict was that the instrument could
/// only park the fake inside LAUNCH and inside a stop `Completion`, so "no
/// external I/O, process wait or thread join under the core lock" was a claim
/// about two paths reported as a claim about the lock. Timing tests can only
/// ever cover the effects someone thought to gate. This one covers the class:
/// the engine's git and harness calls must appear only in
/// `PendingSurface::perform`, `PendingLaunch::perform`/`launch` and
/// `PendingSend::perform`, which take no `Core` and therefore cannot be called
/// with the guard held by construction.
///
/// Like t5's `ApiViews` pin, widening this is meant to be deliberate: moving
/// an effect back into a `&mut Core` path fails here with the call site named.
///
/// Round-2 finding N3R2-03: the enumeration was itself incomplete. OBSERVE
/// and §15 RESUME are external effects — a handle the Claude adapter has no
/// memory of sends it walking `/proc`, and §22.6 names "reading a large
/// output stream" beside the spawn — and both performers were changed to take
/// their observation outside the guard for exactly that reason. But neither
/// verb was on this list, so an OBSERVE inside a `&mut Core` function was
/// invisible here, and one was: `drive` used to fall back to observing itself
/// whenever a caller handed it no observation. Both verbs are enumerated now,
/// and every remaining call site is named below as a single-owner path.
#[test]
fn t11_external_effects_live_only_in_the_out_of_lock_performers() {
    let source = code_only(&read_source("runtime/engine.rs"));
    // The performers, by the exact signatures that make them safe: none of
    // them can see a `Core`.
    let performers = [
        (
            "impl PendingSurface",
            "pub fn perform(&self) -> SurfaceOutcome",
        ),
        (
            "impl PendingLaunch",
            "pub fn perform(&self) -> LaunchOutcome",
        ),
        ("impl PendingSend", "pub fn perform(&self) -> SendOutcome"),
        // Issue #46's addition. The daemon's completion driver has to ask a
        // question no request asked — "did that turn end?" — and OBSERVE is an
        // external effect like any other, so it lands in a performer of its
        // own rather than in the driver's loop, where a `&mut Core` could
        // reach it. Widening this list is the deliberate act the test's own
        // doc comment describes; the settle it feeds re-checks §14.5.
        (
            "impl PendingObserve",
            "pub fn perform(&self) -> ObserveOutcome",
        ),
        // R-MVP1-7's per-turn wall-clock ceiling. INTERRUPT is an external
        // effect exactly like STOP/OBSERVE are, so it gets its own performer
        // rather than joining `stop_execution`'s reviewed under-the-lock
        // exemption below — nothing about the ceiling needs INTERRUPT to run
        // synchronously with a journal append the way STOP's kill-then-latch
        // does.
        (
            "impl PendingInterrupt",
            "pub fn perform(&self) -> InterruptOutcome",
        ),
    ];
    let mut ranges = Vec::new();
    for (block, signature) in performers {
        let start = source
            .find(block)
            .unwrap_or_else(|| panic!("engine.rs must declare `{block}`"));
        assert!(
            source[start..].contains(signature),
            "`{block}` must expose `{signature}` — the performer's whole safety \
             property is that it never receives a Core"
        );
        ranges.push((start, block_end(&source, start)));
    }
    // `PendingLaunch::launch` is the raw effect the two-phase tests drive by
    // hand; it lives in the same block and is covered by the range above.
    //
    // The call sites deliberately outside them, named here rather than left
    // to be discovered. Every one is a single-owner path: startup recovery,
    // which runs between journal replay and the first served request, and the
    // inline drivers recovery and the deterministic tests use, which hold the
    // only reference to the Core there is (see `run_inline`'s doc comment).
    // Nothing is queueing behind the guard in any of them, because nothing is
    // being served. Adding to this list is the deliberate act; adding to it
    // for a path the daemon serves requests from would be a lie the timing
    // tests (m2's t9/t9b/t10/t11/t11c) would then catch.
    for single_owner in [
        "pub fn reconcile_terminal_surface",
        "fn run_inline",
        "pub fn resume(&self",
        "pub fn reconcile_work",
        "fn reserved_identity_liveness",
        "fn reattach",
    ] {
        let start = source
            .find(single_owner)
            .unwrap_or_else(|| panic!("engine.rs must declare `{single_owner}`"));
        ranges.push((start, block_end(&source, start)));
    }

    // Round-2 finding INV-R2-02: `backend.stop()` is a real external effect —
    // a synchronous kill + reap on the Claude adapter (`ClaudeBackend::stop`
    // is `self.interrupt(handle)?.wait()`) — and unlike everything else on
    // this list, it runs under the *request-path* guard too, not only in a
    // single-owner context. That is not new here: issue #14/B3 already
    // reviewed and kept it there deliberately (only the adapter's
    // transcript-archive tail is deferred out from under the lock; the STOP
    // request and its outcome commit are not), and `api::cancel_work`'s own
    // comment says so ("The STOP request and the teardown land under this
    // guard"). What #44 changed is the durability ordering behind that
    // decision: the kill now precedes the flush that makes
    // `execution.stopped`/`execution.abandoned` durable, where before group
    // commit every commit was durable the instant it landed — so a crash
    // between the kill and the flush can leave a killed or orphaned native
    // context with no journal record. This is a known, accepted exemption,
    // not an unreviewed one — named here rather than left invisible to this
    // test. See `CoreGuard`'s doc for the durability guarantee stated
    // honestly against it.
    //
    // INV-R1-01 (N4/M7, MVP-2 D3 fixer pass): N4 added a second backend to
    // this same `fn stop_execution` call site — `DockerBackend::stop` — and
    // this rationale, written entirely about the Claude adapter's cheap
    // signal-send, did not automatically cover it: `DockerBackend::stop`
    // originally shelled out to `docker inspect` + `docker rm -f`
    // synchronously (measured ~70 ms combined, over this contract's entire
    // single-submit p50 budget), which is not the "synchronous kill + reap"
    // shape this exemption was reviewed for. Fixed by making
    // `DockerBackend::stop` defer its whole Docker Engine interaction
    // through `Completion`'s tail-work mechanism, the same "kill now, join
    // later" split `ClaudeBackend::stop` already uses — so both backends
    // registered under this fixed call site now do genuinely cheap,
    // in-memory work synchronously, and the region stays honestly
    // whitelisted rather than silently widened in scope by a new backend.
    for stop_under_the_request_guard in ["fn stop_execution", "pub fn settle_launch"] {
        let start = source
            .find(stop_under_the_request_guard)
            .unwrap_or_else(|| panic!("engine.rs must declare `{stop_under_the_request_guard}`"));
        ranges.push((start, block_end(&source, start)));
    }

    for effect in [
        "materialize(",
        // estate-root Phase E: the submission path calls this instead of
        // `materialize` (it hands over §8.1's admitted base rather than
        // re-reading each mount). It is the same external effect and must
        // stay inside the same out-of-lock performer, so it is named here —
        // a new entry point must not be able to slip the guard this test is.
        "materialize_admitted(",
        "rematerialize(",
        "teardown(",
        "backend.launch(",
        "backend.send(",
        "backend.observe(",
        "backend.resume(",
        "backend.stop(",
        "backend.interrupt(",
        "pending.perform(",
    ] {
        for (index, _) in source.match_indices(effect) {
            // `self.tear_down…`/`…rematerialize` as part of a longer
            // identifier is a different symbol; require a boundary.
            let preceded_by_ident = source[..index]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_');
            if preceded_by_ident {
                continue;
            }
            assert!(
                ranges
                    .iter()
                    .any(|(start, end)| index > *start && index < *end),
                "src/runtime/engine.rs calls `{effect}` outside every out-of-lock \
                 performer, near: {:?}",
                &source[index.saturating_sub(120)..(index + 60).min(source.len())]
            );
        }
    }
}

/// The journal's commit path issues exactly the one fsync it accounts for.
///
/// A-N3-1's whole cost story is per-event fsyncs — the two-phase boundary
/// added two per work, that is the 25% the budget was amended over, and #44's
/// fix (landed) is one fsync per lock hold instead of one per event. So "an
/// extra fsync per append" is *the* named throughput regression, and round-2
/// finding N3R2-04 showed the throughput guard could not see it: on that
/// container an fsync costs ~0.08 ms, so a second one per append is ~5% of a
/// submit — inside the noise of any floor that does not flake (measured: 38.2
/// → 36.8 works/s at burst 25).
///
/// Timing is the wrong instrument for it; counting is the right one, and
/// `Journal::fsync_count` already exists for that. But the counter is derived
/// from *one* `sync_data` call, so a second durability syscall beside it is
/// invisible to the counter as well — which is exactly the mutation the
/// finding used. This closes both: every `sync_data`/`sync_all` in the module
/// must live in a named block, and `sync_now` — the only one on the
/// group-commit path — must contain exactly one, the accounted one.
///
/// Post-#44 this test is doing *more* work than before, not less: with the
/// fsync moved out of `append_event`, an fsync smuggled back into the append
/// path would restore the per-event cost the whole change exists to remove,
/// and the block list below is what makes that un-smugglable.
///
/// m1's `crash_tail…` tests assert the counter's value per group; this
/// asserts the counter is the whole truth about what the journal syncs.
#[test]
fn t11b_the_append_path_issues_exactly_the_fsync_it_accounts_for() {
    let source = code_only(&read_source("runtime/journal.rs"));
    // Where a durability syscall may appear, and why.
    let allowed = [
        // The accounted group-commit sync (#44).
        "fn sync_now",
        // The rollback after a failed append: re-syncs a truncated segment so
        // a torn tail cannot be concatenated onto. Not an acknowledged append.
        "pub fn append_event",
        // Startup quarantine of a torn tail, and the dirent sync that makes a
        // new segment survive its own creation. Neither is on the hot path.
        "fn recover_tail",
        "fn sync_dir",
        // The best-effort close of a group whose handle is going away. Not on
        // any acknowledged path — the daemon's groups are closed by
        // `CoreGuard`, and this only exists so a dropped handle is not a
        // second, silent way to lose a written line.
        "impl Drop for Journal",
        // The module's own `#[cfg(test)]` block. It is not the production
        // path and it is where the fault-injection probes live — one of them
        // has to call `sync_data` on a deliberately unsyncable descriptor to
        // find out whether this host can express the injection at all.
        "mod tests",
    ];
    let ranges: Vec<(usize, usize)> = allowed
        .iter()
        .map(|block| {
            let start = source
                .find(block)
                .unwrap_or_else(|| panic!("journal.rs must declare `{block}`"));
            (start, block_end(&source, start))
        })
        .collect();

    for effect in [".sync_data(", ".sync_all("] {
        for (index, _) in source.match_indices(effect) {
            assert!(
                ranges
                    .iter()
                    .any(|(start, end)| index > *start && index < *end),
                "src/runtime/journal.rs calls `{effect}` outside every accounted \
                 durability site, near: {:?}",
                &source[index.saturating_sub(120)..(index + 60).min(source.len())]
            );
        }
    }

    // And the group-commit path syncs once, through the expression that
    // counts it.
    let (start, end) = ranges[0];
    let body = &source[start..end];
    let syncs = body.matches(".sync_data(").count() + body.matches(".sync_all(").count();
    assert_eq!(
        syncs, 1,
        "`sync_now` must issue exactly one durability syscall — a second one \
         is an fsync per group that `fsync_count` cannot see and no throughput \
         floor on a fast filesystem can feel: {body}"
    );
    assert!(
        body.contains("self.fsync_count += self.segment_file.sync_data()"),
        "the one sync must be the counted one, so the counter cannot drift \
         from the syscall: {body}"
    );
}

/// Every core-lock hold goes through the one guard that closes its group
/// (#44).
///
/// The group commit is only a durability contract if there is exactly one
/// door into the core: a hold taken with a bare `core.lock()` would append
/// events into `Core`'s open group and then release the mutex without
/// fsyncing or publishing them — they would sit there until some *later*,
/// unrelated hold happened to flush them, which is silent delayed durability
/// and a genuinely worse failure than the per-append fsync this replaced.
///
/// `CoreGuard::drop` cannot be forgotten; *acquiring the mutex another way*
/// can. So the rule is structural, like t5 and t11 beside it: a `tokio::sync::Mutex`
/// hold on the core — spelled `core.lock(`, `.blocking_lock(`, `.try_lock(`,
/// `.lock_owned(`, `.try_lock_owned(` or `.blocking_lock_owned(` — may appear
/// only inside the two `CoreGuard` constructors. Anything else — production
/// code, a handler added later, a test fixture — fails here with the call
/// site printed.
///
/// Round-2 findings INV-R2-04/TH-R2-04 closed two of this test's three
/// coverage gaps: it now walks every file under `src/` (not a fixed list of
/// six) and checks every lock-taking spelling `tokio::sync::Mutex` exposes,
/// not just the two group-commit itself uses. The third — a receiver renamed
/// away from `core` losing coverage entirely — is closed for the one shape
/// that exists in this tree today: `daemon.rs` upgrades the sink's `Weak`
/// reference into a local it calls `live` before acquiring through it
/// (`let Some(live) = core.upgrade()`), so a bare `live.lock()` there would
/// otherwise be invisible; every identifier bound from `core.upgrade()` is
/// derived below and scanned as its own receiver, in the same file, for
/// exactly that reason. A rename introduced *elsewhere* in a future edit
/// (a new `Weak<Mutex<Core>>` clone under some other name, reached some
/// other way) is still outside what a textual scan can promise — the honest
/// residual, not a hidden one.
#[test]
fn t11c_every_core_lock_hold_ends_in_the_group_commit_guard() {
    let door_methods = [
        ".lock(",
        ".blocking_lock(",
        ".try_lock(",
        ".lock_owned(",
        ".try_lock_owned(",
        ".blocking_lock_owned(",
    ];

    let mut checked_api_rs = false;
    for path in all_src_files() {
        let module = path
            .strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"))
            .expect("under src/")
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let source = code_only(&std::fs::read_to_string(&path).expect("read source"));

        let allowed: Vec<(usize, usize)> = ["pub async fn acquire", "pub fn acquire_blocking"]
            .iter()
            .filter_map(|block| {
                let start = source.find(block)?;
                Some((start, block_end(&source, start)))
            })
            .collect();
        if module == "api.rs" {
            assert_eq!(
                allowed.len(),
                2,
                "api.rs must declare both `CoreGuard` constructors — they are \
                 the only sanctioned way to take the core lock"
            );
            checked_api_rs = true;
        }

        // Every receiver that could hold the *core*'s mutex: the field/local
        // literally named `core`, plus anything bound straight off
        // `core.upgrade()` (the `Weak<Mutex<Core>>` the sink thread holds —
        // see `daemon::journaling_sink`) under whatever name that binding
        // gave it.
        let mut receivers = vec!["core".to_string()];
        for (index, _) in source.match_indices("core.upgrade()") {
            let prefix = &source[..index];
            let line_start = prefix.rfind('\n').map(|i| i + 1).unwrap_or(0);
            let line = &prefix[line_start..];
            if let Some(let_at) = line.rfind("let ") {
                let binder = line[let_at + 4..].trim();
                // `let Some(name) = core.upgrade()` or `let name = core.upgrade()`.
                let ident: String = binder
                    .trim_start_matches("Some(")
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !ident.is_empty() && !receivers.contains(&ident) {
                    receivers.push(ident);
                }
            }
        }

        for receiver in &receivers {
            for method in door_methods {
                let door = format!("{receiver}{method}");
                for (index, _) in source.match_indices(door.as_str()) {
                    // A bare method call (`.lock(`) must not also match a
                    // qualified one already counted (`core.lock(` inside a
                    // longer receiver expression) — require a non-identifier
                    // boundary immediately before the receiver name.
                    let boundary_at = index;
                    let preceded_by_ident = source[..boundary_at]
                        .chars()
                        .next_back()
                        .is_some_and(|c| c.is_alphanumeric() || c == '_');
                    if preceded_by_ident {
                        continue;
                    }
                    assert!(
                        allowed
                            .iter()
                            .any(|(start, end)| index > *start && index < *end),
                        "src/{module} takes the core lock outside `CoreGuard` \
                         (via `{door}`), so that hold's appends are never \
                         fsynced or published at the end of it (#44), near: {:?}",
                        char_boundary_snippet(&source, index)
                    );
                }
            }
        }
    }
    assert!(checked_api_rs, "the walk must have reached src/api.rs");
}

/// A ~200-byte window of `source` around `index`, snapped to char
/// boundaries — `source[a..b]` panics if either `a` or `b` lands inside a
/// multi-byte character (this codebase's doc comments use `§` freely), which
/// would turn a real violation this test caught into an unrelated slicing
/// panic instead of the assertion failure that names the call site.
fn char_boundary_snippet(source: &str, index: usize) -> &str {
    let target_start = index.saturating_sub(140);
    let start = source
        .char_indices()
        .rev()
        .find(|&(i, _)| i <= target_start)
        .map_or(0, |(i, _)| i);
    let target_end = (index + 60).min(source.len());
    let end = source
        .char_indices()
        .find(|&(i, _)| i >= target_end)
        .map_or(source.len(), |(i, _)| i);
    &source[start..end]
}

/// Every `.rs` file under `src/`, recursively.
fn all_src_files() -> Vec<PathBuf> {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_some_and(|e| e == "rs") {
                found.push(path);
            }
        }
    }
    found
}

/// The end of the `{ … }` block that starts at or after `from`.
fn block_end(source: &str, from: usize) -> usize {
    let open = source[from..].find('{').expect("a block") + from;
    let mut depth = 0usize;
    for (offset, c) in source[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return open + offset;
                }
            }
            _ => {}
        }
    }
    source.len()
}

fn read_source(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// The whole `src/tui/` module tree, concatenated.
///
/// T1a restructured the M6-era single `src/tui.rs` into a module tree
/// (`src/tui/mod.rs` plus submodules) — the structural scans below
/// (`t5`/`t1_the_tui_has_no_private_shortcut`) must see every file in it, not
/// just `mod.rs`, or a submodule could reach past `crate::api` invisibly to
/// them. This is the file-discovery half of that split; the scans'
/// assertions themselves (what paths are allowed, what tokens are forbidden)
/// are unchanged.
fn read_tui_source() -> String {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tui");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {dir:?}: {e}"))
        .map(|entry| {
            entry
                .unwrap_or_else(|e| panic!("read dir entry under {dir:?}: {e}"))
                .path()
        })
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    assert!(
        files.len() > 1,
        "expected the tui module tree to have split into more than one file under {dir:?}, found {files:?}"
    );
    files
        .into_iter()
        .map(|path| std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The same file with its `//`-comments removed.
///
/// The structural rules below are about *code*, and a module that documents
/// what it is forbidden to do — which the two clients here deliberately do —
/// must not fail its own guard for saying so. String literals are tracked so
/// a `"http://…"` cannot be mistaken for the start of a comment.
fn code_only(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let bytes: Vec<char> = line.chars().collect();
        let mut in_string = false;
        let mut escaped = false;
        let mut cut = bytes.len();
        for (index, c) in bytes.iter().enumerate() {
            if escaped {
                escaped = false;
                continue;
            }
            match c {
                '\\' if in_string => escaped = true,
                '"' => in_string = !in_string,
                '/' if !in_string && bytes.get(index + 1) == Some(&'/') => {
                    cut = index;
                    break;
                }
                _ => {}
            }
        }
        out.extend(&bytes[..cut]);
        out.push('\n');
    }
    out
}

/// Every distinct crate-root module a source file reaches into, sorted.
///
/// A path scan rather than a fixed grep: however the path is spelled — in a
/// `use`, inline in an expression, inside a type — the module being reached
/// into is the head of the path. Three spellings the naive version of this
/// scan could not see, each of which was probed against it and passed:
///
/// - `use crate::{api::…, runtime::blob::…}` — a brace group names *several*
///   modules, and a scan that read one identifier after `crate::` saw an
///   empty string and dropped it. Groups are walked, at any nesting.
/// - `use super::daemon::…` — `tui.rs` is a top-level module, so `super::`
///   is `crate::` with a different name. Both roots are scanned.
/// - `super::` inside these files' own `mod tests` means the module itself,
///   not the crate, so a `super::` head only counts when it names a real
///   crate-root module — read from `src/lib.rs`, not listed here.
fn crate_paths(source: &str) -> Vec<String> {
    let modules = crate_modules();
    let mut found = Vec::new();
    for (index, _) in source.match_indices("crate::") {
        path_heads(&source[index + "crate::".len()..], &mut found);
    }
    let mut relative = Vec::new();
    for (index, _) in source.match_indices("super::") {
        path_heads(&source[index + "super::".len()..], &mut relative);
    }
    found.extend(relative.into_iter().filter(|head| modules.contains(head)));
    found.sort();
    found.dedup();
    found
}

/// The head segment(s) of a path expression: `api::ApiClient` names `api`,
/// and `{api::X, runtime::blob::Y}` names both `api` and `runtime`.
fn path_heads(rest: &str, out: &mut Vec<String>) {
    let rest = rest.trim_start();
    if let Some(group) = rest.strip_prefix('{') {
        for branch in brace_branches(group) {
            path_heads(&branch, out);
        }
        return;
    }
    let head: String = rest
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if !head.is_empty() {
        out.push(head);
    }
}

/// The comma-separated branches of a brace group, given the text just past
/// its opening brace. Nested groups travel inside their branch.
fn brace_branches(group: &str) -> Vec<String> {
    let mut branches = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    for c in group.chars() {
        match c {
            '}' if depth == 0 => break,
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            ',' if depth == 0 => branches.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    branches.push(current);
    branches.retain(|branch| !branch.trim().is_empty());
    branches
}

/// The crate's top-level modules, read from `src/lib.rs`.
fn crate_modules() -> Vec<String> {
    let lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let source = std::fs::read_to_string(&lib).expect("read src/lib.rs");
    let modules: Vec<String> = source
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(str::to_string)
        })
        .collect();
    assert!(
        modules.contains(&"api".to_string()) && modules.contains(&"daemon".to_string()),
        "src/lib.rs must still declare its modules as `pub mod …;` for this scan to see them: \
         {modules:?}"
    );
    modules
}

/// Whether an identifier appears as a whole token (not inside a longer word).
fn names_token(source: &str, token: &str) -> bool {
    source
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|word| word == token)
}

/// The rig's own teardown, pinned: SIGTERM, and the daemon exits by itself.
///
/// An instrument nobody has tried is a claim (the m2 reaper test's rule,
/// applied to this file's rig). The assertion that matters is not "the daemon
/// is gone" — SIGKILL achieves that too — but *how*: the daemon must receive
/// SIGTERM, run `run_until_signal`'s shutdown path, and return from `main`,
/// which is the only exit that runs anything registered to happen at exit.
/// A rig that reverted to `child.kill()` would leave every assertion in the
/// suite green while silently dropping this daemon's coverage profile, so the
/// exit status is read and checked rather than discarded.
#[test]
fn the_spawned_daemon_rig_stops_its_daemon_with_sigterm() {
    use std::os::unix::process::ExitStatusExt;

    let data = DataDir::new();
    let cwd = TempDir::new().expect("tempdir");
    empty_estate(cwd.path(), "sigterm-rig-estate");
    let mut spawned = SpawnedDaemon::start(&data, cwd.path(), &[]);
    let pid = spawned.child.id();
    assert!(
        daemon::pid_alive(pid),
        "the rig must hand back a live daemon, or this test measures nothing"
    );

    let stop = spawned.stop();

    assert_eq!(
        stop.signal,
        StopSignal::Term,
        "the rig must stop its daemon with SIGTERM and only escalate if the {}s grace runs \
         out; a SIGKILL here means the polite path was skipped or the daemon slept through it",
        DAEMON_TERM_GRACE.as_secs()
    );
    let status = stop
        .status
        .expect("the rig must wait for the daemon it signalled, not leave a zombie");
    assert_eq!(
        status.signal(),
        None,
        "the daemon must exit on its own after SIGTERM, not die *of* a signal — killed by \
         {:?} means nothing it registered at exit ran (a coverage profile among them)",
        status.signal()
    );
    assert!(
        status.success(),
        "the daemon's SIGTERM shutdown must return from main cleanly: {status:?}"
    );
    assert!(
        !daemon::pid_alive(pid),
        "after stop() the daemon pid {pid} must be gone"
    );
    assert_eq!(
        spawned.stop(),
        stop,
        "stop() must be idempotent: Drop calls it again on the way out"
    );
}

/// guard-map: MVP-3's admission drain flag (`sgt daemon stop`, scoped
/// exactly to that use) — one journaled `admission.paused`/
/// `admission.resumed` event pair plus a submit refusal while paused — and
/// the L6 crash window CONTRIBUTING.md names explicitly for this class of
/// mechanism: a daemon killed *after* `admission.paused` is durable but
/// *before* it ever gets to stop must not leave the data dir stuck refusing
/// work forever.
///
/// This drives the fault directly at the journal/engine level (raw
/// `POST /v1/admission/pause`, then a real `SIGKILL`) rather than through
/// `sgt daemon stop`'s own CLI-side drain wait, because the hazard L6 names
/// is about journal durability across a crash, not about the CLI's polling
/// loop — the crash window exists the instant `admission.paused` commits,
/// independent of whatever the caller was doing next.
///
/// Mutation this kills: dropping `submit_work`'s `admission_paused` check
/// (a paused daemon would keep accepting work); dropping `start_with`'s
/// unconditional startup resume (a fresh daemon would replay
/// `admission_paused: true` forever, since nothing else ever resumes it);
/// or making `pause_admission` non-idempotent (a second pause would journal
/// a redundant event, which this test's second `pause` call would still
/// pass today but a stricter follow-on assertion on event counts — added
/// below — would not).
#[test]
fn r_mvp3_admission_pause_survives_a_kill_between_pause_and_stop() {
    let data = DataDir::new();
    let cwd = TempDir::new().expect("tempdir");
    // A furnished estate, not a bare directory: `daemon` is estate-scoped
    // (§4.2) so the rig needs a root to start at at all, and this test's two
    // submissions have to be *admissible* for the pause to be the only thing
    // that can refuse one. Against a repository-less estate both would be
    // refused by the manifest, and "a submit while admission is paused must
    // fail" would hold no matter what the admission gate did.
    support::scaffold_estate(cwd.path(), "admission-rig-estate", &["svc"]);
    let mut first = SpawnedDaemon::start(&data, cwd.path(), &[]);
    let client = ApiClient::new(&first.endpoint, &first.token).expect("client");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    rt.block_on(async {
        // Pause admission — the durable half of the crash window: everything
        // after this commit must be recoverable no matter when the process
        // dies.
        let paused = client
            .post("/v1/admission/pause", &json!({"command_id": ulid()}))
            .await
            .expect("pause admission");
        assert_eq!(
            paused["admission"]["paused"], true,
            "pause_admission must report paused: {paused}"
        );

        // A genuinely new submission is refused while paused, never silently
        // accepted or queued.
        let refused = client
            .post(
                "/v1/work",
                &json!({"command_id": ulid(), "intent": "refused while admission is paused"}),
            )
            .await;
        assert!(
            refused.is_err(),
            "a submit while admission is paused must fail, not succeed: {refused:?}"
        );

        // Idempotent re-pause: a `daemon stop` retry against a still-live,
        // already-paused daemon must not double-pause. Journaled state is
        // opaque to this client, but the response must be stable and never
        // itself an error — a caller retrying `sgt daemon stop` cannot be
        // punished for the retry.
        let paused_again = client
            .post("/v1/admission/pause", &json!({"command_id": ulid()}))
            .await
            .expect("re-pause must succeed, not error");
        assert_eq!(paused_again["admission"]["paused"], true);
    });

    // The fault: kill the daemon directly, simulating a death mid-drain —
    // before `sgt daemon stop` (or anything else) ever got to send its own
    // SIGTERM or resume admission itself. `Child::kill` is SIGKILL, exactly
    // like `SpawnedDaemon::stop`'s own escalation path.
    let pid = first.child.id();
    first.child.kill().expect("kill the daemon");
    let status = first.child.wait().expect("wait for the killed daemon");
    first.stopped = Some(DaemonStop {
        signal: StopSignal::Kill,
        status: Some(status),
    });
    assert!(
        !daemon::pid_alive(pid),
        "the killed daemon must actually be gone before restarting on the same data dir"
    );

    // The killed daemon never got to remove its descriptor (only a clean
    // shutdown does that), so a stale one is still sitting in the data dir.
    // `SpawnedDaemon::start_at`'s wait loop only checks for *presence*, not
    // freshness (unlike the real `ensure_daemon`, which also checks
    // `/healthz` and compares identity) — it would hand back that stale
    // descriptor immediately rather than waiting for the second daemon's
    // own. Removing it is test-rig hygiene only: production never needs
    // this, because a fresh daemon's own atomic descriptor write
    // overwrites the same path regardless of what is already there.
    std::fs::remove_file(daemon::descriptor_path(data.path()))
        .expect("remove the killed daemon's stale descriptor");

    // Restart on the same data dir. This process was never mid-drain itself
    // — the L6 argument `KIND_ADMISSION_RESUMED`'s own doc makes — so
    // admission must come back unconditionally, not stay stuck.
    let second = SpawnedDaemon::start(&data, cwd.path(), &[]);
    let client2 = ApiClient::new(&second.endpoint, &second.token).expect("client");
    rt.block_on(async {
        let system = client2.get("/v1/system").await.expect("system");
        assert_eq!(
            system["admission_paused"], false,
            "a fresh daemon must never inherit an admission pause from a predecessor that died \
             mid-drain — it would refuse all work forever with no operator ever having asked \
             for that: {system}"
        );
        let accepted = client2
            .post(
                "/v1/work",
                &json!({"command_id": ulid(), "intent": "accepted after the restart resumes admission"}),
            )
            .await
            .expect("submit after restart must succeed");
        // Admitted, which is the whole claim — not `pending`, which it used
        // to be able to assert only because the daemon planned against
        // nothing. §5.1 binds this daemon to the estate it was started at,
        // so a submission is planned inside the request that carried it and
        // the state that comes back is wherever the run got to. The id is
        // the durable evidence that admission happened.
        assert!(
            accepted["work"]["id"].as_str().is_some_and(|id| !id.is_empty()),
            "the post-restart submission must actually be admitted: {accepted}"
        );
    });

    data.reap();
}

/// The same teardown, composed the way the rig's real users get it: `Drop`.
///
/// The pin above never executes `Drop` at all — it calls `stop()` first, which
/// latches `stopped`, so `Drop`'s body is not the thing under test. Measured,
/// not reasoned about: with `Drop::drop` reverted to the pre-repair
/// `child.kill(); child.wait()` and `stop()` left intact, that test and the
/// whole m6 suite still pass. And `Drop` is the *only* teardown the two
/// production users of this rig have — the embedded-assets probe and doctor's
/// live-daemon arm both `start()` and never `stop()`. So the part was pinned
/// and the composition, which is what §6.1's loss site 1 actually rides on,
/// was not.
///
/// This test therefore reads evidence the rig cannot author, because the
/// *daemon* writes it. Measured on this container with two temp data dirs,
/// one `kill -TERM` and one `kill -KILL`: after SIGTERM the daemon runs
/// `run_until_signal`'s shutdown tail — journals `daemon.stopped`, removes
/// `runtime.json` — and after SIGKILL it journals nothing and leaves the
/// descriptor behind. `DataDir`'s own `Drop` cannot substitute for this: it
/// asserts only that no daemon *survived*, and a SIGKILLed daemon has not
/// survived.
#[test]
fn the_dropped_spawned_daemon_leaves_the_evidence_of_a_clean_shutdown() {
    let data = DataDir::new();
    let cwd = TempDir::new().expect("tempdir");
    empty_estate(cwd.path(), "drop-rig-estate");

    let pid = {
        let spawned = SpawnedDaemon::start(&data, cwd.path(), &[]);
        let pid = spawned.child.id();
        assert!(
            daemon::pid_alive(pid),
            "the rig must hand back a live daemon, or this test measures nothing"
        );
        assert!(
            daemon::descriptor_path(data.path()).exists(),
            "a serving daemon must have published its descriptor first, or its absence \
             below would prove nothing"
        );
        pid
        // Scope ends here, and `Drop` is the only teardown that runs — no
        // `stop()` call, exactly like the rig's two production users.
    };

    assert!(
        !daemon::pid_alive(pid),
        "the dropped rig must not leave daemon pid {pid} running"
    );
    assert!(
        !daemon::descriptor_path(data.path()).exists(),
        "the dropped daemon must have removed its own runtime.json, which only the \
         SIGTERM shutdown path does. The descriptor is still there, so the daemon was \
         killed rather than asked — and a SIGKILLed process flushes nothing at exit, \
         its coverage profile included (§6.1 loss site 1)"
    );
    let stopped = Journal::replay_data_dir(data.path())
        .expect("replay the journal")
        .filter_map(Result::ok)
        .filter(|event| event.kind == daemon::KIND_DAEMON_STOPPED)
        .count();
    assert_eq!(
        stopped,
        1,
        "the dropped daemon must have journaled exactly one {} event; {stopped} means \
         its shutdown path did not run to the end",
        daemon::KIND_DAEMON_STOPPED
    );
}

/// The daemon installs its shutdown-signal handlers **before** anything makes
/// it reachable.
///
/// The two tests above both rest on "SIGTERM makes the daemon run its shutdown
/// path", and that is only true once a handler exists: until then SIGTERM's
/// default disposition applies and the kernel simply terminates the process —
/// no `daemon.stopped`, a descriptor left pointing at a dead pid, nothing
/// registered at exit run. `SpawnedDaemon::start` returns the instant the
/// descriptor appears and its callers signal immediately, so every instruction
/// between the descriptor write and the handler install is a window they can
/// land in.
///
/// Measured, 2026-08-11 (Cerberus): the SIGTERM pin above failed with
/// `status.signal() == Some(15)` on run 3 of 40 m6 runs executed against a
/// concurrently running suite — killed *by* the signal, which is the exact
/// failure the window produces. It is not a rig bug and no amount of retrying
/// in the rig would fix it; the ordering is the fix.
///
/// Structural rather than behavioural on purpose. The behavioural witness is
/// the flake itself, which is ~2.5% per run and therefore cannot be a gate;
/// this reads the ordering directly, so a future edit that moves the install
/// back after `start_with` fails here, deterministically, with the reason.
#[test]
fn the_daemon_installs_its_signal_handlers_before_it_publishes_anything() {
    let source = code_only(&read_source("daemon.rs"));
    let start = source
        .find("pub async fn run_until_signal")
        .expect("daemon.rs must declare `run_until_signal`");
    let body = &source[start..block_end(&source, start)];
    let install = body
        .find("ShutdownSignals::install()")
        .expect("run_until_signal must install its signal handlers");
    let serving = body
        .find("start_with(")
        .expect("run_until_signal must start the daemon");
    assert!(
        install < serving,
        "run_until_signal installs its shutdown handlers at byte {install} of its body \
         and calls start_with at {serving}: every instruction in between is a window in \
         which SIGTERM kills this daemon outright instead of shutting it down — and \
         `start_with` publishes the runtime descriptor, which is precisely the thing \
         clients wait for before they signal"
    );
    // …and the install must be the real registration, not a future nobody has
    // polled: `ctrl_c()` registers SIGINT on first poll, which would put the
    // handler back after the descriptor however early the call site sits.
    assert!(
        source.contains("SignalKind::interrupt()") && source.contains("SignalKind::terminate()"),
        "both signals must be registered explicitly at install time"
    );
}

// -------------------------------------------------- coverage-harness pins

/// `scripts/coverage/` grades the S1 baseline; something has to grade it.
///
/// It lives in this suite because this suite already owns the `scripts/`
/// surface — `t4` runs `scripts/demo.sh` the same way, for the same reason:
/// a shell script outside the gate is a script whose checks are claims. The
/// selftest itself needs no instrumented build and no collection run; it
/// drives `common.sh`'s accounting against a scratch directory of empty
/// `.profraw` files and asserts the two things that were measurably
/// unenforceable before: that a stage which destroys an earlier stage's
/// profiles fails (it used to print "stage ok" and exit 0), and that C4's
/// committed `profraw_merged` number is the profraw-list's line count rather
/// than that count glued to the digits in its own path.
#[test]
fn the_coverage_harness_grades_its_own_accounting() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("scripts/coverage/selftest.sh");
    assert!(script.exists(), "scripts/coverage/selftest.sh must exist");

    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .output()
        .expect("run scripts/coverage/selftest.sh");
    assert!(
        output.status.success(),
        "the coverage harness's selftest must pass\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ---------------------------------------------------- rig-reaper self-test

/// #113: eight `TempDir`-shaped rigs, 1.7 GB, left behind by suite runs that
/// got `SIGKILL`ed mid-run — `Drop` never runs for those, so the fix is a
/// reaper that runs at the start of the *next* run rather than another
/// destructor. `support::reap_orphaned_rigs` is what does that sweep; this
/// grades it directly rather than trusting a mechanism nothing exercises.
#[test]
fn the_rig_reaper_removes_only_orphans_whose_owner_process_is_actually_dead() {
    let base = TempDir::new().expect("scratch base for the reaper test");
    let base = base.path();

    // An orphan: its marker names a pid this test can prove is dead, because
    // it spawned and `wait()`ed on it itself — no ambiguity about whether
    // the process table still holds it.
    let mut dead = Command::new("true")
        .spawn()
        .expect("spawn a short-lived process");
    let dead_pid = dead.id();
    dead.wait()
        .expect("wait for the short-lived process to exit");
    let orphan = base.join("orphan-rig");
    std::fs::create_dir_all(&orphan).expect("create fake orphaned rig");
    std::fs::write(
        orphan.join(support::RIG_OWNER_PID_FILE),
        dead_pid.to_string(),
    )
    .expect("write dead owner-pid marker");

    // A live rig: its marker names this very test process, which is alive
    // for the whole test — reaping it would be exactly the "eats a live
    // run's state" defect the issue calls out as worse than the leak.
    let live = base.join("live-rig");
    std::fs::create_dir_all(&live).expect("create fake live rig");
    std::fs::write(
        live.join(support::RIG_OWNER_PID_FILE),
        std::process::id().to_string(),
    )
    .expect("write live owner-pid marker");

    // A pre-#113 rig: no marker at all. Left alone rather than guessed
    // about.
    let unmarked = base.join("unmarked-rig");
    std::fs::create_dir_all(&unmarked).expect("create fake unmarked rig");

    let reaped = support::reap_orphaned_rigs(base);

    assert_eq!(
        reaped,
        vec![orphan.clone()],
        "exactly the dead-owner rig must be reaped"
    );
    assert!(!orphan.exists(), "the orphaned rig must be gone");
    assert!(
        live.exists(),
        "a rig with a live owner must survive the sweep"
    );
    assert!(unmarked.exists(), "an unmarked rig must survive the sweep");
}

/// The same sweep, exercised through `DataDir::new()` itself rather than the
/// bare function — proving the wiring, not just the mechanism. Runs against
/// the real disk-backed base (or its `SGT_TEST_TMPDIR` override) because
/// that is what every suite's `DataDir::new()` actually sweeps; the planted
/// orphan's pid is independently proven dead the same way as above, so this
/// is safe to run alongside any other suite sharing that base.
#[test]
fn data_dir_new_reaps_an_orphaned_rig_left_by_a_prior_killed_run() {
    let Some(base) = support::disk_backed_tmp_base() else {
        eprintln!("SKIPPED-ENV: no disk-backed tmp base on this host");
        return;
    };
    std::fs::create_dir_all(&base).expect("create disk-backed test tmp base dir");

    let mut dead = Command::new("true")
        .spawn()
        .expect("spawn a short-lived process");
    let dead_pid = dead.id();
    dead.wait()
        .expect("wait for the short-lived process to exit");
    let orphan = base.join(format!("sgt-rs-tests-orphan-pin-{}", std::process::id()));
    std::fs::create_dir_all(&orphan).expect("create fake orphaned rig");
    std::fs::write(
        orphan.join(support::RIG_OWNER_PID_FILE),
        dead_pid.to_string(),
    )
    .expect("write dead owner-pid marker");

    let _data_dir = DataDir::new();

    assert!(
        !orphan.exists(),
        "DataDir::new() must reap an orphaned rig left under its base at start"
    );
}

/// W7 (#113 platform-correctness fixer): `pid_is_alive` must decide macOS
/// liveness through the platform boundary's own, already-cfg-gated
/// `process_alive` (`src/platform/process.rs`), not a bare, unguarded
/// `/proc` read. The bug this pins: `Path::new("/proc").join(pid.to_string())
/// .is_dir()` with no `cfg` guard is `false` for *every* pid on a host with
/// no `/proc` at all — macOS — so the reaper would delete the rig of a run
/// that is still using it, on exactly the platform this sprint exists to
/// reach.
///
/// This can't be pinned by running the suite here: this host *has* `/proc`,
/// so the old bare read and the fixed `process_alive` delegation answer
/// identically for any real pid on Linux (the sibling test above already
/// exercises that shared behavior). What differs only on a platform this
/// suite cannot run on is not observable by any assertion over live
/// process state — so, same shape as `t5` above pinning `tui.rs`'s client
/// boundary via a source scan rather than a runtime probe, this pins the
/// *wiring*: revert `pid_is_alive` back to the bare `/proc` read (verified
/// in a disposable `/var/tmp` worktree per L7) and this fails.
///
/// Scoped to the function *body*, not the whole file: an earlier version of
/// this test scanned the full source, which made it pass against a reverted
/// body anyway, because `pid_is_alive`'s own doc comment (this file,
/// `tests/support/mod.rs`) names `platform::process::process_alive` in a
/// rustdoc link — a revert that leaves that comment in place, which is what
/// a real revert of just the function body does, left the substring in the
/// file and the test green. Caught by actually reverting in a disposable
/// `/var/tmp` worktree per L7, not by inspection.
#[test]
fn the_reaper_liveness_check_goes_through_the_platform_boundary_not_a_bare_proc_read() {
    let source = read_test_support_source();
    let body = pid_is_alive_body(&source);
    assert!(
        body.contains("process_alive"),
        "tests/support/mod.rs's pid_is_alive must delegate to \
         sergeant_rs::platform::process::process_alive (src/platform/process.rs), \
         whose macOS arm is cfg-gated — a bare, unguarded `/proc` read reports \
         every pid dead on a platform with no `/proc`. Body was: {body:?}"
    );
    assert!(
        !body.contains("/proc"),
        "tests/support/mod.rs's pid_is_alive must not read /proc directly — that bare \
         read is the bug this test pins. Body was: {body:?}"
    );
}

/// The braces-delimited body of `fn pid_is_alive(pid: u32) -> bool { ... }`,
/// so the assertions above scan only the implementation and not any doc
/// comment above it (see the test's own doc comment for why that distinction
/// is load-bearing). Assumes a body with no nested braces, true of both the
/// delegating form and the bare-`/proc`-read form this test must catch.
fn pid_is_alive_body(source: &str) -> &str {
    let sig = "fn pid_is_alive(pid: u32) -> bool {";
    let start = source
        .find(sig)
        .unwrap_or_else(|| panic!("tests/support/mod.rs must declare `{sig}`"));
    let after_sig = &source[start + sig.len()..];
    let end = after_sig
        .find('}')
        .expect("pid_is_alive's body must close with a `}`");
    &after_sig[..end]
}

fn read_test_support_source() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/support/mod.rs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

// -------------------------------------------------- perf clock guard pin

/// #95: `scripts/perf/common.sh`'s `perf_now_ns`/`perf_mark` must fail
/// loudly when neither clock branch works (macOS: `EPOCHREALTIME` needs
/// bash 5.0+, BSD `date` has no `%N`), instead of feeding a value like
/// `"1786728341N"` into arithmetic. Picking the replacement clock is out of
/// scope (needs real macOS timing); this only pins the guard, exercised on
/// Linux by forcing the fallback branch (`scripts/perf/common-selftest.sh`'s
/// own header explains the reproduction). Same pattern as the coverage
/// harness self-test above: a shell script outside the gate is a script
/// whose checks are claims, and this suite already owns `scripts/`.
#[test]
fn the_perf_clock_guard_fails_loudly_instead_of_a_malformed_timestamp() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("scripts/perf/common-selftest.sh");
    assert!(
        script.exists(),
        "scripts/perf/common-selftest.sh must exist"
    );

    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .output()
        .expect("run scripts/perf/common-selftest.sh");
    assert!(
        output.status.success(),
        "the perf clock guard's selftest must pass\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// -------------------------------------------------- shipping-gate #120 pin

/// #120: no-mistakes resolves its diff base by running `git ls-remote
/// --symref origin HEAD` against this repo's `origin` remote (a live,
/// non-bare working copy, not a fixed-default-branch host) and rebasing
/// onto it; when a checkout was cut from that same tip, the resulting diff
/// is genuinely empty and no-mistakes silently fast-paths to `outcome:
/// passed` without running review/test/document/lint. The fix (this repo's
/// own `sergeant-rs-workspace's knowledge/evidence/gauntlet/runs/t-series-build-2026-08-16/plan.md` names it
/// explicitly) is a pre-flight guard in `scripts/gate.sh` that replicates
/// that same base detection and refuses loudly instead. Same pattern as the
/// coverage harness and perf clock guard self-tests above: a shell script
/// outside the gate is a script whose checks are claims, and this suite
/// already owns `scripts/`.
#[test]
fn the_shipping_gate_refuses_rather_than_silently_fast_pathing_on_120s_empty_diff() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let script = root.join("scripts/gate-guard-selftest.sh");
    assert!(script.exists(), "scripts/gate-guard-selftest.sh must exist");

    let output = Command::new("bash")
        .arg(&script)
        .current_dir(&root)
        .output()
        .expect("run scripts/gate-guard-selftest.sh");
    assert!(
        output.status.success(),
        "the shipping gate's #120 empty-diff guard selftest must pass\n--- stdout ---\n{}\n--- stderr ---\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// ------------------------------------------------------- spawned-daemon rig

/// How the rig's own daemon ended, and what the kernel said about it.
///
/// Recorded rather than assumed: the whole point of the polite teardown is
/// that the daemon gets to run its shutdown path, and "it is gone" is not
/// evidence that it did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DaemonStop {
    /// The strongest signal the rig had to send.
    signal: StopSignal,
    /// What `wait(2)` reported. `None` only if the wait itself failed.
    status: Option<std::process::ExitStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopSignal {
    Term,
    Kill,
}

/// A `sgt daemon` running as a real child process, from a chosen cwd.
///
/// **`cwd` must be an estate root.** `daemon` is estate-scoped (§4.2), root
/// validation runs before anything else in dispatch (§4.3), and there is no
/// ancestor walk to rescue a directory that is merely near one — so a rig
/// pointed at a bare `TempDir` never publishes a descriptor and `start_at`
/// below panics on the timeout rather than failing where the mistake is.
/// The root the daemon is started at is also the estate it is *bound* to
/// (§5.1): it plans against that estate and refuses clients resolving a
/// different one.
struct SpawnedDaemon {
    endpoint: String,
    token: String,
    data_dir: PathBuf,
    child: std::process::Child,
    /// Set by the first `stop()`, so `Drop` neither repeats it nor waits on
    /// a child that has already been reaped.
    stopped: Option<DaemonStop>,
}

impl SpawnedDaemon {
    /// Start a daemon on a guarded data dir.
    ///
    /// The [`DataDir`] is what makes the process accounting complete: this
    /// type reaps the child it owns, but a client command run against the
    /// same data dir can spawn a *second*, detached daemon that no `Child`
    /// handle points at, and that is the shape the leak measurement found.
    fn start(data_dir: &DataDir, cwd: &Path, env: &[(&str, &str)]) -> Self {
        Self::start_at(data_dir.path(), cwd, env)
    }
}

impl SpawnedDaemon {
    // The child is reaped by this type's `Drop`, which stops and waits. The
    // lint cannot see across that boundary; the timeout path below reaps
    // explicitly because it never constructs the value that owns the Drop —
    // and it stays a `kill()`, because a daemon that never published a
    // descriptor has nothing to shut down politely.
    #[allow(clippy::zombie_processes)]
    fn start_at(data_dir: &Path, cwd: &Path, env: &[(&str, &str)]) -> Self {
        let mut command = Command::new(SGT);
        command
            .arg("--data-dir")
            .arg(data_dir)
            .arg("daemon")
            .current_dir(cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        for (key, value) in env {
            command.env(key, value);
        }
        let mut child = command.spawn().expect("spawn sgt daemon");
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Ok(Some(descriptor)) = daemon::read_descriptor(data_dir) {
                return Self {
                    endpoint: descriptor.endpoint,
                    token: descriptor.token,
                    data_dir: data_dir.to_path_buf(),
                    child,
                    stopped: None,
                };
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("the spawned daemon never published a descriptor");
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Stop the daemon the way an operator stops it — SIGTERM, a bounded
    /// wait, SIGKILL only if the wait runs out — and report which of the two
    /// it took. Idempotent; `Drop` calls it for tests that do not.
    ///
    /// This rig used to open with `child.kill()`, i.e. SIGKILL. The daemon
    /// died either way and every test stayed green, so the difference was
    /// invisible from inside the suite — but SIGKILL runs nothing registered
    /// at exit. Measured on this container: an instrumented process that
    /// handles SIGTERM and returns from `main` writes its `.profraw`; the
    /// same process under SIGKILL writes none at all. Both of this rig's
    /// daemons were therefore contributing nothing to a coverage run, which
    /// is what §6.1 of the S-series proposal registered as loss site 1.
    fn stop(&mut self) -> DaemonStop {
        if let Some(stop) = self.stopped {
            return stop;
        }
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(self.child.id().to_string())
            .status();
        let deadline = Instant::now() + DAEMON_TERM_GRACE;
        let stop = loop {
            if let Ok(Some(status)) = self.child.try_wait() {
                break DaemonStop {
                    signal: StopSignal::Term,
                    status: Some(status),
                };
            }
            if Instant::now() >= deadline {
                // The escalation still exists — a rig that could be outlived
                // by its own daemon would be a worse bug than a lost profile.
                let _ = self.child.kill();
                break DaemonStop {
                    signal: StopSignal::Kill,
                    status: self.child.wait().ok(),
                };
            }
            std::thread::sleep(Duration::from_millis(50));
        };
        self.stopped = Some(stop);
        stop
    }
}

/// How long the rig's daemon gets to shut down on SIGTERM before SIGKILL.
///
/// Deliberately the same ten seconds as `support::TERM_GRACE`: two teardown
/// paths that disagreed about "too slow" would make a slow shutdown look like
/// a different failure depending on which one ran first.
const DAEMON_TERM_GRACE: Duration = Duration::from_secs(10);

impl Drop for SpawnedDaemon {
    fn drop(&mut self) {
        let stop = self.stop();
        if stop.signal == StopSignal::Kill {
            eprintln!(
                "SpawnedDaemon: the daemon on {:?} ignored SIGTERM for {}s and needed SIGKILL \
                 — it flushed nothing at exit (coverage profiles included)",
                self.data_dir,
                DAEMON_TERM_GRACE.as_secs()
            );
        }
    }
}

fn write_script(dir: &Path, name: &str, body: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, body).expect("write script");
    let mut permissions = std::fs::metadata(&path).expect("stat").permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).expect("chmod");
    // A file another process still holds open for writing cannot be exec'd;
    // absorb the ETXTBSY window before handing it over.
    support::wait_until_executable(&path);
    path.display().to_string()
}

/// A stand-in `claude` that passes the adapter's version and flag gates, so
/// the doctor tests measure the doctor rather than this container.
fn stub_claude(dir: &Path) -> String {
    let flags = "--print --verbose --output-format --session-id --resume --setting-sources \
                 --model --permission-mode --dangerously-skip-permissions";
    write_script(
        dir,
        "claude-stub",
        &format!(
            "#!/bin/sh\ncase \"$1\" in\n  --version) echo '2.1.226 (Claude Code)';;\n  \
             --help) echo '{flags}';;\n  *) cat >/dev/null;;\nesac\n"
        ),
    )
}

/// A stand-in `systemctl`/`launchctl` that always reports success — same
/// reasoning as `stub_docker`/`stub_claude`: `host_service_manager`'s
/// "every check must be green on a healthy install" assertion below must
/// not depend on whether *this host* happens to have a reachable native
/// service manager (W4c, following TH-02's own probe-gating precedent).
/// Wired in through `SGT_SYSTEMCTL_BIN`/`SGT_LAUNCHCTL_BIN`
/// (`platform::service`'s injection points, the `SGT_GIT_BIN` precedent).
fn stub_service_manager(dir: &Path) -> String {
    write_script(dir, "service-manager-stub", "#!/bin/sh\nexit 0\n")
}

/// A stand-in `docker` that passes both `DockerBackend::probe`'s cheap
/// version check (`docker version --format {{.Server.Version}}`) and
/// `DockerBackend::lifecycle_probe`'s real bind-mount round trip (INV-R1-06:
/// `docker_check` now runs both, so the doctor tests need a stub that fakes
/// both, not just the ping), so the doctor tests measure the doctor rather
/// than whether *this host* happens to have a reachable Docker Engine
/// (TH-02, MVP-2 D3 fixer pass: N4.md's probe-gating rule for Docker facts,
/// applied to the one doctor row that previously had no override analogous
/// to `SGT_CLAUDE_BIN`). For `run`, it parses the `--mount`
/// `type=bind,source=...,target=...` value straight out of argv and writes
/// the probe marker there itself, standing in for what a real container
/// would do — good enough to prove the doctor *asks* the lifecycle
/// question, without needing a real container runtime in this suite.
fn stub_docker(dir: &Path) -> String {
    write_script(
        dir,
        "docker-stub",
        "#!/bin/sh\n\
         case \"$1\" in\n\
         \x20 version) echo '99.0.0';;\n\
         \x20 run)\n\
         \x20   for arg in \"$@\"; do\n\
         \x20     case \"$arg\" in\n\
         \x20       type=bind,source=*)\n\
         \x20         src=$(echo \"$arg\" | sed -n 's/.*source=\\([^,]*\\),target=.*/\\1/p')\n\
         \x20         echo sergeant-probe > \"$src/probe-marker\"\n\
         \x20         ;;\n\
         \x20     esac\n\
         \x20   done\n\
         \x20   exit 0\n\
         \x20   ;;\n\
         \x20 rm) exit 0;;\n\
         \x20 *) exit 1;;\n\
         esac\n",
    )
}

/// Split-hardening W5, review finding #1b (`src/backend/codex.rs`'s
/// `worktree_git_common_dir` doc comment carries the full reasoning): the
/// #259 fix now grants a codex actor's linked worktree write access to the
/// repository's whole shared `.git` common directory, not merely its own
/// per-worktree admin subdirectory — a real `git commit` needs the wider
/// grant (see that function and `tests/codex_backend.rs`'s
/// `the_first_turn_grants_the_works_own_git_common_dir_as_an_add_dir_root`).
/// This is a strictly *wider* filesystem grant than before, so what it costs
/// the isolation story needs to be stated as plainly as what the narrower
/// grant used to promise. This test states both halves in git terms alone —
/// no codex-specific code is reachable from this integration crate, so it
/// proves the underlying git facts the grant is built on, not the adapter's
/// own plumbing (`src/runtime/surface.rs`'s own unit tests are the only
/// place with access to that).
///
/// **Holds, structurally**: two different repositories never share a common
/// dir. A Work's grant is built only from its own bindings, so this alone is
/// what keeps a Work confined to the repositories it was actually admitted
/// into — nothing here depends on trusting the actor.
///
/// **No longer holds, structurally**: two Works on the *same* repository —
/// each in its own linked worktree, on its own distinct branch
/// (`runtime::surface`'s own admission always cuts a new branch per Work) —
/// resolve to the exact same common directory. A sandbox scoped to that
/// grant cannot tell "this Work's own branch ref" from "the other Work's
/// branch ref, or the object store backing every branch in the repository"
/// -- both live under the one directory both Works were handed write access
/// to. `runtime::repolock` only serializes the `git worktree add` span
/// (proven by `tests/c4_repo_lock.rs`), never a Work's whole lifetime, so
/// two such Works can legitimately be running concurrently when this
/// applies. What still keeps them apart is the same-named branch convention
/// (`runtime::surface` never hands two live Works the same branch) plus
/// after-the-fact detection (branch-tip assertions, `common_dir_finding`) —
/// trust and audit, not a filesystem boundary.
#[test]
fn git_worktree_common_dir_grant_is_shared_within_a_repository_but_never_crosses_repositories() {
    let root = TempDir::new().expect("tempdir");

    // Two Works on the same repository: distinct worktrees, distinct
    // branches, same source.
    let repo_a = root.path().join("repo-a");
    support::init_repo(&repo_a);
    let work_a1 = root.path().join("work-a1");
    let work_a2 = root.path().join("work-a2");
    support::git(
        &repo_a,
        &[
            "worktree",
            "add",
            "-b",
            "sergeant/work-a1",
            work_a1.to_str().unwrap(),
        ],
    );
    support::git(
        &repo_a,
        &[
            "worktree",
            "add",
            "-b",
            "sergeant/work-a2",
            work_a2.to_str().unwrap(),
        ],
    );

    let grant_a1 = canonical_git_common_dir(&work_a1).expect("work-a1 common dir");
    let grant_a2 = canonical_git_common_dir(&work_a2).expect("work-a2 common dir");
    assert_eq!(
        grant_a1, grant_a2,
        "two Works on the same repository resolve to the same shared common dir -- the grant \
         itself no longer distinguishes them; only the branch-naming convention does"
    );

    // A distinct branch each, still true -- the convention half of the
    // isolation story, not the filesystem half.
    let branch_a1 = support::git(&work_a1, &["symbolic-ref", "--quiet", "--short", "HEAD"]);
    let branch_a2 = support::git(&work_a2, &["symbolic-ref", "--quiet", "--short", "HEAD"]);
    assert_ne!(
        branch_a1.trim(),
        branch_a2.trim(),
        "each Work still lands on its own named branch, even though the grant no longer \
         enforces that separation by itself"
    );

    // A second, unrelated repository: its common dir must never equal
    // either of repo-a's Works' grants. This is the half that IS still a
    // structural (not merely conventional) boundary.
    let repo_b = root.path().join("repo-b");
    support::init_repo(&repo_b);
    let work_b1 = root.path().join("work-b1");
    support::git(
        &repo_b,
        &[
            "worktree",
            "add",
            "-b",
            "sergeant/work-b1",
            work_b1.to_str().unwrap(),
        ],
    );
    let grant_b1 = canonical_git_common_dir(&work_b1).expect("work-b1 common dir");
    assert_ne!(
        grant_a1, grant_b1,
        "a Work bound only to repo-a must never resolve a grant equal to repo-b's common dir \
         -- cross-repository isolation is never trust-based, only cross-Work isolation within \
         one repository is"
    );
}
