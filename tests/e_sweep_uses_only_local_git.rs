//! #159, via a recording Git binary: a sweep only ever *reads* an estate's
//! refs, and the only thing it ever writes is `git branch -D` on a
//! `sergeant/*` branch it just proved redundant.
//!
//! The classification pass is advertised as mutating nothing and the deletion
//! pass as touching nothing outside `refs/heads/sergeant/`. Both are claims
//! about which git verbs run, and the only honest way to check a claim of
//! that shape is to watch every invocation: a shim records working directory
//! and full argument vector, then `exec`s the real Git, so a *complete*
//! sweep — classify, then delete — runs end to end and the recording is of
//! the real thing (`tests/e_admission_uses_no_network_git.rs` establishes
//! this rig and shares its mechanics through `tests/support`).
//!
//! **This must stay the only test in this file.** `std::env::set_var` is
//! process-global, not thread-scoped, and `runtime::git::command` reads the
//! override fresh on every invocation, so a sibling test in this process
//! would see the shim too and would race the `TempDir` holding it.

use std::collections::BTreeSet;
use std::sync::Arc;

use serde_json::json;
use tempfile::TempDir;

use sergeant_rs::api::ApiClient;
use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig};
use sergeant_rs::runtime::git::GIT_BIN_ENV;
use sergeant_rs::runtime::sweep::SweepTarget;

mod support;
use support::{Invocation, parse_log, real_git, scaffold_solo_estate, write_recording_shim};

/// Every git verb a sweep — classification *and* deletion — is allowed to
/// run. All local, all either a read or a `sergeant/*` ref deletion:
///
/// - `rev-parse` — mount validation (`Estate::resolve`) and the repository
///   lock identity the deletion takes (`§2.7`'s common directory);
/// - `symbolic-ref` — the mount's own default branch, which is what every
///   `redundant` verdict is relative to (§6.4: never a remote's idea of one);
/// - `for-each-ref` — the `refs/heads/sergeant/` enumeration;
/// - `merge-base` — the one containment proof deletion is authorized on;
/// - `rev-list` — the unique-commit count a `retained` branch reports;
/// - `worktree` — the prunable-registration count (`list --porcelain`);
/// - `branch` — the deletion itself, asserted below to be `-D` and nothing
///   else.
const ALLOWED_VERBS: &[&str] = &[
    "rev-parse",
    "symbolic-ref",
    "for-each-ref",
    "merge-base",
    "rev-list",
    "worktree",
    "branch",
];

/// Verbs that must never appear anywhere in a sweep. Network verbs because a
/// sweep decides redundancy from local ancestry alone; history-rewriting and
/// checkout verbs because the mount is the operator's and a sweep only reads
/// it.
const FORBIDDEN: &[&str] = &[
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
    "reset",
    "commit",
    "gc",
    "prune",
];

#[tokio::test]
async fn a_whole_sweep_reads_refs_and_deletes_nothing_but_a_redundant_sergeant_branch() {
    let estate = TempDir::new().expect("estate tempdir");
    let (mount, _head) = scaffold_solo_estate(estate.path(), "svc");
    let workflow = estate.path().join(".sergeant/workflows/solo");
    std::fs::create_dir_all(workflow.join("00-only")).expect("workflow dirs");
    std::fs::write(
        workflow.join("workflow.toml"),
        "[workflow]\nname = \"solo\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("workflow.toml");
    std::fs::write(workflow.join("00-only").join("CONTEXT.md"), "context").expect("CONTEXT.md");

    let data = TempDir::new().expect("data tempdir");
    let fake = FakeBackend::scripted(FAKE_BACKEND_NAME, [FakeStep::complete()]);
    let handle = daemon::start_with(
        data.path(),
        DaemonConfig {
            backends: Arc::new(BackendRegistry::new().with(Arc::new(fake))),
            default_backend: Some(FAKE_BACKEND_NAME.to_string()),
            ..DaemonConfig::default()
        },
    )
    .await
    .expect("daemon start");
    // D4: the sweep addresses the estate whose `sergeant/*` refs it means.
    let client = ApiClient::new(&handle.endpoint, &handle.token)
        .expect("client")
        .with_estate_root(estate.path());

    // A real Work, under the real git, so the branch the sweep classifies is
    // one a materialization actually left behind.
    let submitted = client
        .post(
            "/v1/work",
            &json!({
                "command_id": ulid::Ulid::generate().to_string(),
                "intent": "leave a branch behind",
                "workflow": "solo",
                "estate_root": estate.path(),
                "origin": {"client": "cli", "cwd": estate.path()},
            }),
        )
        .await
        .expect("submit");
    let id = submitted["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    let mut terminal = false;
    for _ in 0..200 {
        let work = client
            .get(&format!("/v1/work/{id}"))
            .await
            .expect("read work");
        if work["work"]["state"]
            .as_str()
            .is_some_and(|s| s.starts_with("completed") || s == "failed" || s == "canceled")
        {
            terminal = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    assert!(terminal, "work {id} never reached a terminal state");
    let work_branch = format!("sergeant/{id}");

    let shim_dir = TempDir::new().expect("tempdir");
    let log = shim_dir.path().join("git-invocations.log");
    let shim = shim_dir.path().join("recording-git");
    write_recording_shim(&shim, &log, &real_git());

    // SAFETY (test-only): this binary runs exactly one test, so there is no
    // other thread in this process to observe the mutation — the whole
    // reason this file holds one test. Set only for the sweep itself, so the
    // recording is of the sweep and of nothing that set it up, and removed
    // with the shim's `TempDir` still alive.
    unsafe { std::env::set_var(GIT_BIN_ENV, &shim) };
    let report = client.sweep().await.expect("classification pass");
    let deletion = client
        .sweep_delete(
            &[SweepTarget {
                repository: "svc".to_string(),
                branch: work_branch.clone(),
            }],
            true,
        )
        .await
        .expect("deletion pass");
    unsafe { std::env::remove_var(GIT_BIN_ENV) };

    handle.shutdown().await;

    assert_eq!(
        report["repositories"][0]["branches"][0]["classification"], "redundant",
        "{report}"
    );
    assert_eq!(deletion["deleted"][0]["outcome"], "deleted", "{deletion}");

    let recorded = parse_log(&std::fs::read_to_string(&log).expect("the shim recorded something"));
    assert!(
        recorded.len() > 3,
        "the shim must have intercepted the real sweep, not a handful of strays: {recorded:?}"
    );
    let verbs: BTreeSet<&str> = recorded.iter().filter_map(Invocation::verb).collect();
    // The shim really did stand in for git: the verbs a sweep must run to do
    // its job at all are all present.
    for expected in ["for-each-ref", "symbolic-ref", "merge-base", "branch"] {
        assert!(
            verbs.contains(expected),
            "a sweep is supposed to run `git {expected}`; the recording shows {verbs:?}"
        );
    }
    for verb in &verbs {
        assert!(
            !FORBIDDEN.contains(verb),
            "a sweep ran `git {verb}` — it decides redundancy from local ancestry alone and \
             only ever reads the operator's mount: {recorded:?}"
        );
        assert!(
            ALLOWED_VERBS.contains(verb),
            "`git {verb}` is not in the sweep's declared verb set {ALLOWED_VERBS:?}; widen it \
             deliberately, with the reason, or do not run it"
        );
    }

    // The one mutating verb, pinned to exactly the one shape it is allowed:
    // a forced delete of a `sergeant/*` branch, nothing else.
    let mutations: Vec<&Invocation> = recorded
        .iter()
        .filter(|i| i.verb() == Some("branch"))
        .collect();
    assert_eq!(mutations.len(), 1, "{mutations:?}");
    for mutation in mutations {
        assert!(
            mutation.args.iter().any(|a| a == "-D"),
            "sweep's only `git branch` is the deletion: {mutation:?}"
        );
        assert!(
            mutation.args.contains(&work_branch),
            "the deletion names the branch it proved redundant: {mutation:?}"
        );
        assert_eq!(
            mutation.cwd,
            std::fs::canonicalize(&mount).expect("canonical mount"),
            "the deletion runs in the estate's own mount: {mutation:?}"
        );
    }
}
