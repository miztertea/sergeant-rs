//! §6.4/§17, via a recording Git binary: "Sergeant does not fetch, pull,
//! reset, rebase, switch to main, or infer a remote default during Work
//! admission."
//!
//! Phase 0 made this observable by giving `runtime::git` an injectable binary
//! ([`GIT_BIN_ENV`]); §18 asks for the test that uses it. A shim records every
//! invocation — working directory and full argument vector — and then `exec`s
//! the real Git, so a *complete, successful* admission runs end to end and the
//! recording is of the real thing rather than of a stub's idea of it.
//!
//! **This must stay the only test in this file**, for exactly the reason
//! `tests/c11_injectable_git.rs` documents: `std::env::set_var` is
//! process-global, not thread-scoped, and `runtime::git::command` reads the
//! override fresh on every invocation. A sibling test in this process would
//! see the shim too, and would also race the `TempDir` holding it. Cargo gives
//! every integration-test file its own process, so a file with one test has no
//! sibling to leak into.
//!
//! # What "forbidden" means precisely
//!
//! Two different rules, because they are two different claims:
//!
//! - **Network and history-rewriting verbs are forbidden outright**, in any
//!   directory: `fetch`, `pull`, `push`, `remote`, `ls-remote`, `clone`,
//!   `rebase`, `merge`, `cherry-pick`, `switch`, `checkout`. None of them has
//!   any business on an admission path at all.
//! - **`reset` is forbidden in the mounted checkout.** Materialization does
//!   run one `git reset --hard HEAD` — inside the *linked worktree it just
//!   created*, to populate the files `worktree add --no-checkout`
//!   deliberately skipped (`surface::checkout_worktree` explains why `reset`
//!   rather than `checkout`). That touches Sergeant's own surface and no
//!   branch; §6.4 is about the checkout the operator owns. The assertion is
//!   therefore scoped by working directory rather than waved away, and the
//!   test proves the mount saw none of it.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::runtime::git::GIT_BIN_ENV;

mod support;
use support::{DataDir, Invocation, parse_log, real_git, scaffold_estate, write_recording_shim};

/// Verbs that must never appear on an admission path, in any directory.
const FORBIDDEN_ANYWHERE: &[&str] = &[
    "fetch",
    "pull",
    "push",
    "remote",
    "ls-remote",
    "clone",
    "rebase",
    "merge",
    "cherry-pick",
    "switch",
    "checkout",
];

/// Verbs that must never be run *against the mounted checkout*. See the
/// module doc for why `reset` is on this list rather than the one above.
const FORBIDDEN_IN_THE_MOUNT: &[&str] = &["reset"];

#[tokio::test]
async fn a_whole_admission_never_runs_a_network_or_branch_changing_git_command() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    // Two repositories, explicitly scoped, so the recording covers a
    // multi-repository admission rather than the single-mount easy case.
    scaffold_estate(&estate, "payments", &["api", "web"]);
    let api = std::fs::canonicalize(estate.join("repos").join("api")).expect("canonical api");
    let web = std::fs::canonicalize(estate.join("repos").join("web")).expect("canonical web");

    let shim_dir = TempDir::new().expect("tempdir");
    let log = shim_dir.path().join("git-invocations.log");
    let shim = shim_dir.path().join("recording-git");
    write_recording_shim(&shim, &log, &real_git());

    // SAFETY (test-only): this binary runs exactly one test, so there is no
    // other thread in this process to observe the mutation — the whole reason
    // this file holds one test. Set before the daemon starts (its own estate
    // admission is part of what is being recorded) and removed only once
    // every git invocation is finished, with the shim's `TempDir` still
    // alive throughout.
    unsafe { std::env::set_var(GIT_BIN_ENV, &shim) };

    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [FakeStep::hang()]);
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake.clone()))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            claude: None,
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client");
    let response = client
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&json!({
            "command_id": ulid::Ulid::generate().to_string(),
            "intent": "admit two clean mounts",
            "scope": {"all": true},
            "estate_root": estate,
            "origin": {"client": "cli"},
        }))
        .send()
        .await
        .expect("request");
    let status = response.status();
    let body: Value = response.json().await.expect("json body");

    handle.shutdown().await;
    unsafe { std::env::remove_var(GIT_BIN_ENV) };

    assert_eq!(status, 201, "the admission must succeed: {body}");
    assert_eq!(body["work"]["state"], "active", "{body}");
    assert_eq!(
        body["surface"]["bindings"]
            .as_array()
            .expect("bindings")
            .len(),
        2,
        "both repositories were materialized: {body}"
    );

    let recorded = parse_log(&std::fs::read_to_string(&log).expect("the shim recorded something"));
    assert!(
        recorded.len() > 5,
        "the shim must have intercepted the real admission, not a handful of strays: {recorded:?}"
    );
    // The shim really did stand in for git everywhere: the verbs a normal
    // admission does use are all present.
    let verbs: BTreeSet<&str> = recorded.iter().filter_map(Invocation::verb).collect();
    for expected in ["rev-parse", "status", "show-ref", "worktree"] {
        assert!(
            verbs.contains(expected),
            "admission is supposed to run `git {expected}`; the recording shows {verbs:?}"
        );
    }

    for invocation in &recorded {
        let verb = invocation.verb().unwrap_or("");
        assert!(
            !FORBIDDEN_ANYWHERE.contains(&verb),
            "§6.4: admission ran `git {verb}` in {} — no network or branch-changing git \
             command may run on an admission path at all ({:?})",
            invocation.cwd.display(),
            invocation.args
        );
        let in_a_mount = invocation.cwd == api || invocation.cwd == web;
        assert!(
            !(in_a_mount && FORBIDDEN_IN_THE_MOUNT.contains(&verb)),
            "§6.4: admission ran `git {verb}` in the mounted checkout {} ({:?}) — the mount is \
             the operator's, and sergeant only ever reads it",
            invocation.cwd.display(),
            invocation.args
        );
    }

    // The one `reset` materialization does run is inside the surface it just
    // created, never in a mount — asserted positively so the carve-out above
    // is a measured fact rather than a loophole nothing exercises.
    let resets: Vec<&Invocation> = recorded
        .iter()
        .filter(|i| i.verb() == Some("reset"))
        .collect();
    assert_eq!(resets.len(), 2, "one per materialized worktree: {resets:?}");
    let surfaces = std::fs::canonicalize(data.path().join("surfaces")).expect("surfaces root");
    for reset in resets {
        assert!(
            reset.cwd.starts_with(&surfaces),
            "the only `git reset` on this path populates sergeant's own worktree: {reset:?}"
        );
    }

    // §8.2's other half, measured the same way: no invocation asked git what
    // any remote thinks, so nothing about the base could have been inferred
    // from one.
    for invocation in &recorded {
        assert!(
            !invocation
                .args
                .iter()
                .any(|arg| arg == "origin" || arg.starts_with("refs/remotes/")),
            "§6.4: admission must not infer a remote default: {invocation:?}"
        );
    }
}
