//! W5 brief deliverable 1(a): the periodic multi-estate sweep caller named
//! but not built by `runtime::sweep::classify`'s own `// W4a seam:` comment —
//! `maybe_run_periodic_sweep` iterating `EstateRegistry::entries()` and
//! calling `classify` once per admitted estate.
//!
//! Real daemon, two admitted estates, each with a merged `sergeant/*`
//! branch. The decisive property: after the periodic pass has had a chance
//! to run over *both* estates, neither mount's ref store has moved —
//! classification never mutates (`runtime::sweep::classify`'s own doc),
//! whether it runs once via `GET /v1/sweep` or on a background tick.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::json;
use tempfile::TempDir;
use tracing_subscriber::layer::SubscriberExt;

use sergeant_rs::api::ApiClient;
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};

mod support;
use support::{git, scaffold_solo_estate};

/// Captures the `estate` field of every `"periodic sweep"` info-level event
/// emitted by [`sergeant_rs::api`]'s periodic caller, so a test can assert
/// the caller actually *ran* against a given mount rather than merely
/// inferring it from an absence of mutation (which also holds if the caller
/// never fires — `classify` mutates nothing under correct behavior either
/// way).
#[derive(Clone, Default)]
struct SweepLog(Arc<Mutex<Vec<String>>>);

impl SweepLog {
    fn swept_estates(&self) -> Vec<String> {
        self.0.lock().expect("sweep log lock poisoned").clone()
    }
}

struct SweepLogLayer(SweepLog);

#[derive(Default)]
struct SweepEventVisitor {
    is_periodic_sweep: bool,
    estate: Option<String>,
}

impl tracing::field::Visit for SweepEventVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        match field.name() {
            "message" if rendered.contains("periodic sweep") => self.is_periodic_sweep = true,
            "estate" => self.estate = Some(rendered.trim_matches('"').to_string()),
            _ => {}
        }
    }
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for SweepLogLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = SweepEventVisitor::default();
        event.record(&mut visitor);
        if visitor.is_periodic_sweep
            && let Some(estate) = visitor.estate
        {
            self.0
                .0
                .lock()
                .expect("sweep log lock poisoned")
                .push(estate);
        }
    }
}

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

fn refs_in(mount: &Path) -> String {
    git(mount, &["for-each-ref", "--format=%(refname:short)"])
}

async fn completed_work(client: &ApiClient, cwd: &Path) -> String {
    let submitted = client
        .post(
            "/v1/work",
            &json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "intent": "leave a branch behind",
                "workflow": "solo",
                "estate_root": cwd,
                "origin": {"client": "cli", "cwd": cwd},
            }),
        )
        .await
        .expect("submit");
    let id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    for _ in 0..200 {
        let work = client
            .get(&format!("/v1/work/{id}"))
            .await
            .expect("read work");
        if matches!(
            work["work"]["state"].as_str(),
            Some("completed" | "completed_dirty" | "failed" | "canceled")
        ) {
            return id;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("work {id} never reached a terminal state");
}

/// guard-map: two admitted estates, each with a redundant `sergeant/*`
/// branch, survive several periodic-sweep-eligible ticks unmutated.
/// Mutation this kills: a periodic caller that (a) never fires at all, so
/// this would pass vacuously even with a caller that deletes on sight, or
/// (b) mutates instead of only classifying.
#[tokio::test]
async fn periodic_sweep_walks_every_admitted_estate_and_mutates_nothing() {
    let sweep_log = SweepLog::default();
    let subscriber = tracing_subscriber::registry().with(SweepLogLayer(sweep_log.clone()));
    let _tracing_guard = tracing::subscriber::set_default(subscriber);

    let root_a = TempDir::new().expect("estate a tempdir");
    let (mount_a, _) = scaffold_solo_estate(root_a.path(), "svc-a");
    write_solo_workflow(root_a.path());
    let root_b = TempDir::new().expect("estate b tempdir");
    let (mount_b, _) = scaffold_solo_estate(root_b.path(), "svc-b");
    write_solo_workflow(root_b.path());

    let data = TempDir::new().expect("data tempdir");
    let fake = FakeBackend::scripted(
        FAKE_BACKEND_NAME,
        [FakeStep::complete(), FakeStep::complete()],
    );
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            completion_poll: Duration::from_millis(15),
            // Always due: this test cares that the caller reaches every
            // admitted estate, not about the production throttle.
            sweep_interval: Duration::ZERO,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");

    let client = ApiClient::new(&handle.endpoint, &handle.token).expect("client");
    completed_work(&client, root_a.path()).await;
    completed_work(&client, root_b.path()).await;

    let before_a = refs_in(&mount_a);
    let before_b = refs_in(&mount_b);
    assert!(before_a.contains("sergeant/"), "{before_a}");
    assert!(before_b.contains("sergeant/"), "{before_b}");

    // Several completion-poll ticks, each one periodic-sweep-eligible —
    // enough for the background caller to have walked both estates at least
    // once if it iterates the registry at all.
    tokio::time::sleep(Duration::from_millis(200)).await;

    assert_eq!(
        before_a,
        refs_in(&mount_a),
        "estate A's ref store must be byte-identical after periodic sweep ticks"
    );
    assert_eq!(
        before_b,
        refs_in(&mount_b),
        "estate B's ref store must be byte-identical after periodic sweep ticks"
    );

    // Decisive: the periodic caller must actually have executed against
    // both mounts, not merely have left them unmutated by never firing at
    // all. Reverting the `maybe_run_periodic_sweep` wiring in
    // `drive_completions` makes this fail while the ref-store assertions
    // above keep passing vacuously.
    let swept = sweep_log.swept_estates();
    let root_a_display = root_a.path().display().to_string();
    let root_b_display = root_b.path().display().to_string();
    assert!(
        swept.iter().any(|e| e == &root_a_display),
        "periodic sweep caller never logged running against estate A ({root_a_display}); swept: {swept:?}"
    );
    assert!(
        swept.iter().any(|e| e == &root_b_display),
        "periodic sweep caller never logged running against estate B ({root_b_display}); swept: {swept:?}"
    );

    handle.shutdown().await;
}
