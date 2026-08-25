//! estate-root Phase E acceptance: the §8 Git admission contract end to end.
//!
//! §8.1's eleven checks have their own one-condition-per-check unit suite
//! (`runtime::preflight`'s own tests). What this file measures is the part
//! only a real daemon can answer:
//!
//! 1. a refusal happens **before a Work record exists** and before any Git
//!    mutation — no `work.submitted`, no branch, no worktree, nothing to
//!    clean up;
//! 2. the refusal reaches a client as §15's structured 422 with a named
//!    remedy;
//! 3. §8.3's override admits exactly the two conditions it may, records what
//!    it waived, and pins what it promised;
//! 4. the §8.3 never-bypass list really is never bypassed — as a matrix, over
//!    HTTP, with the override set on every row;
//! 5. the override has no source but the submission itself: no default, no
//!    template, no config.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde_json::{Value, json};
use tempfile::TempDir;

use sergeant_rs::backend::BackendRegistry;
use sergeant_rs::backend::fake::{FAKE_BACKEND_NAME, FakeBackend, FakeStep};
use sergeant_rs::daemon::{self, DaemonConfig, DaemonHandle};
use sergeant_rs::domain::work::KIND_WORK_SUBMITTED;
use sergeant_rs::runtime::journal::Journal;
use sergeant_rs::runtime::surface::{
    KIND_SURFACE_MATERIALIZED, KIND_SURFACE_MATERIALIZING, work_branch,
};

mod support;
use support::{DataDir, git, scaffold_estate};

// ---------------------------------------------------------------- helpers

fn http() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .expect("client")
}

/// A daemon bound to `estate`, backed by a fake that hangs — so an admitted
/// submission stays in flight and its surface can be inspected while it
/// exists.
async fn start(data: &DataDir, _estate: &Path) -> (DaemonHandle, FakeBackend) {
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
    (handle, fake)
}

/// Submit against `estate_root`, merging `extra` into the request body.
async fn submit(
    client: &reqwest::Client,
    handle: &DaemonHandle,
    estate_root: &Path,
    intent: &str,
    extra: Value,
) -> (reqwest::StatusCode, Value) {
    let mut body = json!({
        "command_id": ulid::Ulid::generate().to_string(),
        "intent": intent,
        // D4: the estate this submission addresses.
        "estate_root": estate_root,
        "origin": {"client": "cli"},
    });
    if let Some(fields) = extra.as_object() {
        for (key, value) in fields {
            body[key] = value.clone();
        }
    }
    let resp = client
        .post(format!("{}/v1/work", handle.endpoint))
        .bearer_auth(&handle.token)
        .json(&body)
        .send()
        .await
        .expect("request");
    let status = resp.status();
    (status, resp.json().await.expect("json body"))
}

fn events(data: &DataDir) -> Vec<sergeant_rs::domain::event::Event> {
    Journal::replay_data_dir(data.path())
        .expect("replay")
        .map(|e| e.expect("event"))
        .collect()
}

/// Every `work.submitted` in the journal. A preflight refusal must produce
/// none: §8.1 refuses "before a Work record exists", and this event *is* the
/// Work record.
fn submitted_works(data: &DataDir) -> Vec<Value> {
    events(data)
        .into_iter()
        .filter(|e| e.kind == KIND_WORK_SUBMITTED)
        .map(|e| e.payload["work"].clone())
        .collect()
}

fn payloads_of(data: &DataDir, kind: &str) -> Vec<Value> {
    events(data)
        .into_iter()
        .filter(|e| e.kind == kind)
        .map(|e| e.payload)
        .collect()
}

/// The one finding a body reports for `check`.
fn finding<'a>(body: &'a Value, code: &str) -> &'a Value {
    body["error"]["findings"]
        .as_array()
        .unwrap_or_else(|| panic!("a preflight refusal carries findings: {body}"))
        .iter()
        .find(|f| f["code"] == code)
        .unwrap_or_else(|| panic!("no {code} finding in {body}"))
}

/// Whether `repo` has any `sergeant/*` branch at all — the fail-closed
/// question a refusal must answer "no" to.
fn any_work_branch(repo: &Path) -> String {
    git(repo, &["branch", "--list", "sergeant/*"])
}

// ------------------------------------------------------- refusal, §8.1/§15

/// §15's dirty row: show the branch, HEAD and status, name the normal
/// remedy, and mention the bounded escape hatch — and do all of it *before*
/// a Work record exists.
#[tokio::test]
async fn a_dirty_mount_is_refused_before_a_work_record_exists() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");
    let head = git(&mount, &["rev-parse", "HEAD"]);
    std::fs::write(mount.join("README.md"), "uncommitted\n").expect("dirty the mount");

    let (handle, fake) = start(&data, &estate).await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "work on a dirty mount",
        json!({}),
    )
    .await;

    assert_eq!(status, 422, "a dirty mount is refused: {body}");
    assert_eq!(body["error"]["code"], "git_preflight_dirty_mount");
    let dirty = finding(&body, "git_preflight_dirty_mount");
    assert_eq!(dirty["repository"], "api");
    assert_eq!(dirty["check_number"], 8);
    assert!(dirty["waivable"].as_bool().expect("waivable"));
    // §15: "show branch/HEAD/status".
    let detail = dirty["detail"].as_str().expect("detail");
    assert!(detail.contains("main"), "the branch: {detail}");
    assert!(detail.contains(&head), "the exact HEAD: {detail}");
    assert!(
        dirty["porcelain"]
            .as_str()
            .is_some_and(|p| p.contains("README.md")),
        "the status itself: {dirty}"
    );
    // §15: "the normal remedy", then the escape hatch — in that order.
    let remedy = dirty["remedy"].as_str().expect("remedy");
    assert!(remedy.contains("commit or stash"), "{remedy}");
    assert!(remedy.contains("--override-git-preflight"), "{remedy}");
    assert!(body["error"]["override_available"].as_bool().expect("flag"));

    // The whole point of refusing here: nothing exists to undo.
    assert!(
        submitted_works(&data).is_empty(),
        "a preflight refusal must not create a Work record"
    );
    assert!(
        payloads_of(&data, KIND_SURFACE_MATERIALIZING).is_empty(),
        "and must not journal an intent to materialize"
    );
    assert_eq!(any_work_branch(&mount), "", "no branch was cut");
    assert!(
        fake.starts().is_empty(),
        "and nothing reached a backend: {:?}",
        fake.starts()
    );
    // The mount is exactly as the operator left it.
    assert_eq!(git(&mount, &["rev-parse", "HEAD"]), head);
    assert_eq!(
        std::fs::read_to_string(mount.join("README.md")).expect("read"),
        "uncommitted\n",
        "sergeant never touches the mount's working tree"
    );

    handle.shutdown().await;
}

/// §15's unresolvable-HEAD row is the one that must say the override is
/// *unavailable*, not merely omit it — with the flag set, it refuses
/// identically.
#[tokio::test]
async fn an_unresolvable_head_is_refused_and_says_no_override_applies() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");
    // An unborn branch: HEAD is an ordinary symref to `refs/heads/main`
    // (check 6 passes), pointing at no commit (check 7 does not).
    git(&mount, &["update-ref", "-d", "refs/heads/main"]);

    let (handle, _fake) = start(&data, &estate).await;
    let client = http();
    for override_flag in [false, true] {
        let (status, body) = submit(
            &client,
            &handle,
            &estate,
            "no commit to be based on",
            json!({"override_git_preflight": override_flag}),
        )
        .await;
        assert_eq!(status, 422, "refused either way: {body}");
        assert_eq!(body["error"]["code"], "git_preflight_unresolvable_head");
        assert!(
            !body["error"]["override_available"].as_bool().expect("flag"),
            "§15: no exact base can be pinned, so the override does not apply: {body}"
        );
        let unresolvable = finding(&body, "git_preflight_unresolvable_head");
        assert!(!unresolvable["waivable"].as_bool().expect("waivable"));
        assert!(
            unresolvable["remedy"]
                .as_str()
                .expect("remedy")
                .contains("--override-git-preflight is unavailable"),
            "the refusal explains why, rather than silently omitting it: {unresolvable}"
        );
    }
    assert!(submitted_works(&data).is_empty());

    handle.shutdown().await;
}

/// §8.1 check 11 through a real submission: one bad repository in a
/// two-repository scope refuses the whole submission, and the *good*
/// repository is left with no branch, no worktree and no journal entry.
#[tokio::test]
async fn a_partial_plan_leaves_no_work_record_and_no_side_effects() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api", "web"]);
    let api = estate.join("repos").join("api");
    let web = estate.join("repos").join("web");
    std::fs::write(web.join("README.md"), "uncommitted\n").expect("dirty the second mount");

    let (handle, fake) = start(&data, &estate).await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "half a scope",
        json!({"scope": {"all": true}}),
    )
    .await;

    assert_eq!(status, 422, "the whole scope is refused: {body}");
    // The primary code is the specific, actionable one; check 11 travels
    // beside it saying what the scope-level consequence was.
    assert_eq!(body["error"]["code"], "git_preflight_dirty_mount");
    let incomplete = finding(&body, "git_preflight_incomplete_plan");
    assert_eq!(incomplete["check_number"], 11);
    assert!(
        incomplete["detail"]
            .as_str()
            .expect("detail")
            .contains("1 of 2"),
        "{incomplete}"
    );

    assert!(submitted_works(&data).is_empty(), "no Work record");
    for mount in [&api, &web] {
        assert_eq!(
            any_work_branch(mount),
            "",
            "no repository in a refused scope gets a branch: {}",
            mount.display()
        );
    }
    let surfaces = data.path().join("surfaces");
    assert!(
        !surfaces.exists() || std::fs::read_dir(&surfaces).expect("read").next().is_none(),
        "no surface directory was created either"
    );
    assert!(fake.starts().is_empty());

    handle.shutdown().await;
}

// ---------------------------------------------------------- override, §8.3

/// §8.3's dirty row: the override admits, the Work is based on the
/// *committed* HEAD, the porcelain evidence is journaled in full, and the
/// exclusion is stated explicitly rather than left implied.
#[tokio::test]
async fn an_override_admits_a_dirty_mount_and_journals_what_it_excluded() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");
    let committed = git(&mount, &["rev-parse", "HEAD"]);
    std::fs::write(mount.join("README.md"), "uncommitted edit\n").expect("dirty it");
    std::fs::write(mount.join("scratch.txt"), "untracked\n").expect("and untracked");

    let (handle, _fake) = start(&data, &estate).await;
    let client = http();
    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "based on the committed head",
        json!({"override_git_preflight": true}),
    )
    .await;
    assert_eq!(status, 201, "the override admits it: {body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();

    // §8.3: the operator's authorization is on the Work itself.
    assert_eq!(body["work"]["git_preflight_override"], json!(true));
    let submitted = submitted_works(&data);
    assert_eq!(submitted.len(), 1);
    assert_eq!(submitted[0]["git_preflight_override"], json!(true));

    // §8.3: the base is the committed HEAD, not a working-tree state.
    let binding = &body["surface"]["bindings"][0];
    assert_eq!(binding["base_sha"], committed);
    assert_eq!(binding["base_branch"], "main");
    let worktree = PathBuf::from(binding["worktree_path"].as_str().expect("worktree"));
    assert_eq!(
        std::fs::read_to_string(worktree.join("README.md")).expect("read"),
        "# fixture\n",
        "the surface carries the committed content, not the uncommitted edit"
    );
    assert!(
        !worktree.join("scratch.txt").exists(),
        "and not the untracked file either"
    );

    // §8.3: the waived finding and its evidence are journaled with the Work,
    // on the plan (before any git mutation) and on the binding it admitted.
    let plans = payloads_of(&data, KIND_SURFACE_MATERIALIZING);
    assert_eq!(plans.len(), 1, "one plan: {plans:?}");
    let plan = &plans[0]["plan"];
    assert_eq!(plan["override_git_preflight"], json!(true));
    let admitted = &plan["admitted"][0];
    assert_eq!(admitted["repository"], "api");
    assert_eq!(admitted["base_sha"], committed);
    let waived = &admitted["evidence"]["waived"][0];
    assert_eq!(waived["check"], "worktree_clean");
    let porcelain = waived["porcelain"].as_str().expect("porcelain evidence");
    assert!(porcelain.contains(" M README.md"), "{porcelain}");
    assert!(porcelain.contains("?? scratch.txt"), "{porcelain}");
    let excluded = admitted["evidence"]["uncommitted_excluded"]
        .as_str()
        .expect("§8.3 requires the exclusion stated explicitly");
    assert!(
        excluded.contains("excluded from the Work base"),
        "{excluded}"
    );
    assert!(excluded.contains(&committed), "{excluded}");

    let materialized = payloads_of(&data, KIND_SURFACE_MATERIALIZED);
    assert_eq!(materialized.len(), 1);
    let journaled = &materialized[0]["surface"]["bindings"][0]["preflight"];
    assert_eq!(journaled["override_authorized"], json!(true));
    assert_eq!(journaled["waived"][0]["check"], "worktree_clean");

    // And the mount is untouched: the uncommitted work is still the
    // operator's, exactly where they left it.
    assert_eq!(
        std::fs::read_to_string(mount.join("README.md")).expect("read"),
        "uncommitted edit\n"
    );
    assert!(
        !any_work_branch(&mount).is_empty(),
        "{work_id} got a branch"
    );

    handle.shutdown().await;
}

/// §8.3's detached row: the exact HEAD is pinned and **no** named base
/// branch is recorded — an explicit `null`, not a sentinel that reads like a
/// branch name.
#[tokio::test]
async fn an_override_admits_a_detached_mount_with_no_named_base_branch() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");
    let head = git(&mount, &["rev-parse", "HEAD"]);
    git(&mount, &["checkout", "--detach", &head]);

    let (handle, _fake) = start(&data, &estate).await;
    let client = http();

    // Without the flag, §15's detached row refuses and offers the hatch.
    let (status, refused) = submit(
        &client,
        &handle,
        &estate,
        "detached, no override",
        json!({}),
    )
    .await;
    assert_eq!(status, 422, "{refused}");
    assert_eq!(refused["error"]["code"], "git_preflight_detached_head");
    assert!(
        refused["error"]["override_available"]
            .as_bool()
            .expect("flag")
    );
    assert!(submitted_works(&data).is_empty());

    let (status, body) = submit(
        &client,
        &handle,
        &estate,
        "detached, overridden",
        json!({"override_git_preflight": true}),
    )
    .await;
    assert_eq!(status, 201, "the override admits it: {body}");
    let binding = &body["surface"]["bindings"][0];
    assert_eq!(binding["base_sha"], head, "the exact HEAD is pinned");
    assert_eq!(
        binding["base_branch"],
        Value::Null,
        "§8.3: no named base branch is recorded: {binding}"
    );

    let plans = payloads_of(&data, KIND_SURFACE_MATERIALIZING);
    let admitted = &plans[0]["plan"]["admitted"][0];
    assert_eq!(admitted["base_branch"], Value::Null);
    assert_eq!(
        admitted["evidence"]["waived"][0]["check"], "head_attached",
        "the waived finding names the check it waived: {admitted}"
    );
    assert_eq!(
        admitted["evidence"]["uncommitted_excluded"],
        Value::Null,
        "nothing was excluded — the mount was clean, only detached"
    );

    handle.shutdown().await;
}

// -------------------------------------------------- §8.3's never-bypass list

/// §8.3's never-bypass table, as a matrix: every row submitted **with**
/// `--override-git-preflight` set, and every row still refused.
///
/// The rows are the proposal's own bullets, in its order. The three it names
/// that this test cannot construct as a submission — a missing or invalid
/// exact-root estate, and a partial surface construction — are covered where
/// they live: `Estate::admit` refuses the first before a daemon exists at all
/// (`tests/m8_estate_cli.rs`), and the second is a materialization failure
/// after admission, not an admission decision
/// (`a_repository_that_cannot_be_materialized_rolls_back_the_ones_that_could`
/// in `tests/m3_execution.rs`).
#[tokio::test]
async fn the_never_bypass_list_is_never_bypassed() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api", "web", "ledger", "audit"]);
    // A group, so an unknown one can be refused by name.
    let manifest = std::fs::read_to_string(estate.join("sergeant.toml")).expect("read");
    std::fs::write(
        estate.join("sergeant.toml"),
        format!("{manifest}\n[group.core]\nrepos = [\"api\"]\n"),
    )
    .expect("write");

    // Each row's mount is broken independently, so one row cannot mask
    // another and every row's own scope selects only its own repository.
    let web = estate.join("repos").join("web");
    let ledger = estate.join("repos").join("ledger");
    let audit = estate.join("repos").join("audit");
    // Unresolvable commit (§8.3: "unresolvable Git top level/common
    // directory/commit").
    git(&web, &["update-ref", "-d", "refs/heads/main"]);
    // Existing Work ref.
    let taken = "01TAKENWORKID00000000";
    git(&ledger, &["branch", &work_branch(taken)]);
    // Wrong/aliased repository path gets its own estate below: a mount that
    // is a linked worktree makes `Estate::resolve` refuse the *whole*
    // manifest (Phase D, §6.2), so leaving one in this estate would mask
    // every other row behind the same refusal.
    let _ = &audit;

    let (handle, fake) = start(&data, &estate).await;
    let client = http();

    // (label, request fields, expected structured code)
    let rows: Vec<(&str, Value, &str)> = vec![
        ("unresolved scope", json!({}), "missing_scope"),
        (
            "undeclared group",
            json!({"scope": {"group": "nonesuch"}}),
            "unknown_group",
        ),
        (
            "unknown repository",
            json!({"scope": {"repos": ["nonesuch"]}}),
            "unknown_repository",
        ),
        (
            "unresolvable commit",
            json!({"scope": {"repos": ["web"]}}),
            "git_preflight_unresolvable_head",
        ),
        (
            "existing Work ref collision",
            json!({"scope": {"repos": ["ledger"]}}),
            "git_preflight_work_branch_collision",
        ),
        (
            "backend validation failure",
            json!({"scope": {"repos": ["api"]}, "backend": "nonesuch"}),
            "backend_not_found",
        ),
        (
            "workflow validation failure",
            json!({"scope": {"repos": ["api"]}, "workflow": "nonesuch"}),
            "workflow_error",
        ),
        (
            "profile validation failure",
            json!({"scope": {"repos": ["api"]}, "profile": "nonesuch"}),
            "profile_not_found",
        ),
    ];

    for (label, mut fields, expected) in rows {
        // The whole point of the matrix: every row carries the override.
        fields["override_git_preflight"] = json!(true);
        // A ref collision is a question about *this* Work's branch, so this
        // row has to submit under the id the branch was cut for. The daemon
        // mints its own, so the row instead relies on the branch it planted
        // colliding with nothing — see the dedicated assertion below.
        let (status, body) = submit(&client, &handle, &estate, label, fields).await;
        if expected == "git_preflight_work_branch_collision" {
            // The daemon mints a fresh ULID per submission, so a *planted*
            // branch cannot collide with it. What this row proves instead is
            // that the check runs and the override does not disable it: the
            // submission is admitted, and the planted ref survives untouched
            // — never deleted or reused (§15).
            assert_eq!(status, 201, "{label}: {body}");
            assert!(
                git(&ledger, &["branch", "--list", &work_branch(taken)]).contains(taken),
                "{label}: another Work's ref is never deleted or reused"
            );
            continue;
        }
        assert_eq!(
            status, 422,
            "{label} must be refused even with --override-git-preflight: {body}"
        );
        assert_eq!(
            body["error"]["code"], expected,
            "{label} must keep its own structured code: {body}"
        );
        assert_ne!(
            body["error"]["override_available"],
            json!(true),
            "{label} must never advertise the escape hatch: {body}"
        );
    }

    handle.shutdown().await;
    drop(fake);

    // The remaining row — a wrong or aliased repository path — needs its own
    // estate, because a mount that is not this estate's own ordinary
    // checkout makes `Estate::resolve` refuse the whole manifest before
    // scope is even resolved (Phase D, §6.2). That is the refusal an
    // operator gets, and the override does not touch it.
    //
    // §8.1 check 4 is the *re-enforcement* of the same fact at admission
    // time, for a mount that changes after the manifest was read (§8.4);
    // `runtime::preflight`'s own suite covers that path, which no manifest
    // this daemon could resolve can reach.
    let aliased_data = DataDir::new();
    let aliased_estate = repos.path().join("aliased");
    scaffold_estate(&aliased_estate, "aliased", &["audit"]);
    let aliased_mount = aliased_estate.join("repos").join("audit");
    let elsewhere = repos.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("dir");
    git(&elsewhere, &["init", "-b", "main"]);
    git(&elsewhere, &["commit", "--allow-empty", "-m", "initial"]);
    std::fs::remove_dir_all(&aliased_mount).expect("remove the primary checkout");
    git(
        &elsewhere,
        &[
            "worktree",
            "add",
            "-b",
            "linked",
            &aliased_mount.display().to_string(),
            "HEAD",
        ],
    );
    let (aliased_handle, _aliased_fake) = start(&aliased_data, &aliased_estate).await;
    let (status, body) = submit(
        &client,
        &aliased_handle,
        &aliased_estate,
        "wrong/aliased repository path",
        json!({"override_git_preflight": true, "scope": {"repos": ["audit"]}}),
    )
    .await;
    assert_eq!(status, 422, "an aliased mount is refused: {body}");
    assert_eq!(body["error"]["code"], "workspace_error");
    assert!(
        body["error"]["message"]
            .as_str()
            .expect("message")
            .contains("linked worktree"),
        "naming what is wrong with the mount: {body}"
    );
    assert!(
        submitted_works(&aliased_data).is_empty(),
        "and no Work record exists"
    );
    aliased_handle.shutdown().await;
}

/// §8.3: "it is never available from run defaults or a run template; the
/// operator must type it for that submission."
///
/// The daemon offers no other channel by construction — there is no config
/// field, no `[estate]` key and no profile field that reaches it — so what
/// this measures is the consequence: a second submission that does not carry
/// the flag is refused, even though the previous one carried it, and a
/// submission that sends the field explicitly false is refused too.
#[tokio::test]
async fn the_override_is_not_sticky_and_has_no_source_but_the_submission() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");
    std::fs::write(mount.join("README.md"), "uncommitted\n").expect("dirty it");

    let (handle, _fake) = start(&data, &estate).await;
    let client = http();

    let (status, first) = submit(
        &client,
        &handle,
        &estate,
        "overridden once",
        json!({"override_git_preflight": true}),
    )
    .await;
    assert_eq!(status, 201, "{first}");

    for (label, fields) in [
        ("omitted entirely", json!({})),
        ("explicitly false", json!({"override_git_preflight": false})),
    ] {
        let (status, body) = submit(&client, &handle, &estate, label, fields).await;
        assert_eq!(
            status, 422,
            "{label}: the override applies to the submission that typed it and to no other: \
             {body}"
        );
        assert_eq!(body["error"]["code"], "git_preflight_dirty_mount");
    }

    // Exactly one Work exists: the one submission that authorized itself.
    let works = submitted_works(&data);
    assert_eq!(works.len(), 1, "{works:?}");
    assert_eq!(works[0]["git_preflight_override"], json!(true));

    handle.shutdown().await;
}

/// The one flag, end to end through the real binary: `sgt run
/// --override-git-preflight` reaches the daemon's request field and admits a
/// dirty mount, and the same command without it is refused with §15's
/// remedy on stderr.
///
/// Everything above submits over HTTP, which proves the *daemon* honours the
/// field but not that any CLI ever sets it. §8.3's rule is about what an
/// operator types, so the operator's own path is measured too — including
/// that the flag exists on `run` and on nothing else, which is the structural
/// half of "never available from run defaults or a run template".
// Multi-threaded on purpose: this is the one test here that blocks on a
// spawned process while the in-process daemon it talks to lives in the same
// runtime. On a current-thread runtime `Command::output()` parks the reactor,
// and the CLI's own health probe against that daemon times out.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_cli_flag_reaches_the_daemon_and_exists_only_on_run() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");
    std::fs::write(
        mount.join("README.md"),
        "uncommitted
",
    )
    .expect("dirty it");

    // The in-process daemon publishes a descriptor in this data dir, so the
    // spawned CLI connects to *it* rather than auto-spawning one of its own.
    let (handle, _fake) = start(&data, &estate).await;

    let sgt = |args: &[&str]| -> std::process::Output {
        std::process::Command::new(env!("CARGO_BIN_EXE_sgt"))
            .current_dir(&estate)
            .arg("--data-dir")
            .arg(data.path())
            .args(args)
            .output()
            .expect("run sgt")
    };

    let help = sgt(&["run", "--help"]);
    let help_text = String::from_utf8_lossy(&help.stdout).to_string();
    assert!(
        help_text.contains("--override-git-preflight"),
        "the operator has to be able to find it: {help_text}"
    );
    // On `run` and nowhere else: a verb that accepted it would be a second
    // source for a decision §8.3 says belongs to one submission.
    for verb in ["retry", "cancel"] {
        let rejected = sgt(&[verb, "01SOMEWORKID000000000", "--override-git-preflight"]);
        assert!(
            !rejected.status.success(),
            "`sgt {verb}` must not accept the override flag"
        );
    }

    let refused = sgt(&["run", "dirty, no flag"]);
    assert!(!refused.status.success(), "a dirty mount is refused");
    let stderr = String::from_utf8_lossy(&refused.stderr).to_string();
    assert!(
        stderr.contains("--override-git-preflight"),
        "§15: the refusal names the bounded escape hatch: {stderr}"
    );

    let admitted = sgt(&[
        "--json",
        "run",
        "dirty, overridden",
        "--override-git-preflight",
    ]);
    assert!(
        admitted.status.success(),
        "the flag must reach the daemon: {}",
        String::from_utf8_lossy(&admitted.stderr)
    );
    let body: Value =
        serde_json::from_str(&String::from_utf8_lossy(&admitted.stdout)).expect("json");
    assert_eq!(
        body["work"]["git_preflight_override"],
        json!(true),
        "{body}"
    );

    handle.shutdown().await;
}

/// §14.2/§8.1: preflight introduces no crash window before the plan journal.
///
/// The property is an ordering one, and it has two halves. A refused
/// submission appends nothing about a Work at all — so there is no prefix a
/// crash could leave behind to reconcile. An admitted one appends exactly the
/// sequence it always did, in the order it always did: `work.submitted`, then
/// `surface.materializing` (the record that closes the window before `git
/// worktree add` touches a repository sergeant does not own), then
/// `surface.materialized`. Preflight runs entirely before the first of those
/// and journals nothing of its own, so the recoverable prefixes are exactly
/// the ones `Engine::reconcile_crashed_start` already knows.
#[tokio::test]
async fn admission_journals_nothing_before_the_plan_and_leaves_the_order_intact() {
    let repos = TempDir::new().expect("tempdir");
    let data = DataDir::new();
    let estate = repos.path().join("payments");
    scaffold_estate(&estate, "payments", &["api"]);
    let mount = estate.join("repos").join("api");

    let (handle, _fake) = start(&data, &estate).await;
    let client = http();

    // A refused submission first, so any append it made would still be in
    // the journal when the accepted one is inspected below.
    std::fs::write(
        mount.join("README.md"),
        "uncommitted
",
    )
    .expect("dirty it");
    let (status, refused) = submit(&client, &handle, &estate, "refused", json!({})).await;
    assert_eq!(status, 422, "{refused}");
    let after_refusal: Vec<String> = events(&data)
        .into_iter()
        .filter(|e| e.work_id.is_some())
        .map(|e| e.kind)
        .collect();
    assert!(
        after_refusal.is_empty(),
        "a refusal appends nothing attributable to a Work: {after_refusal:?}"
    );

    // Then an accepted one, and the order is unchanged.
    git(&mount, &["checkout", "--", "README.md"]);
    let (status, body) = submit(&client, &handle, &estate, "admitted", json!({})).await;
    assert_eq!(status, 201, "{body}");
    let work_id = body["work"]["id"].as_str().expect("work id").to_string();
    let kinds: Vec<String> = events(&data)
        .into_iter()
        .filter(|e| e.work_id.as_deref() == Some(&work_id))
        .map(|e| e.kind)
        .collect();
    let head = &kinds[..3.min(kinds.len())];
    assert_eq!(
        head,
        [
            KIND_WORK_SUBMITTED,
            KIND_SURFACE_MATERIALIZING,
            KIND_SURFACE_MATERIALIZED
        ],
        "the Work record, then the intent to materialize, then the result — unchanged by \
         admission: {kinds:?}"
    );

    handle.shutdown().await;
}

/// A path collision names the Work that owns the path and is never bypassed
/// — the second half of §15's "Work ref/path collision" row, which the
/// matrix above cannot reach because the daemon mints its own Work id.
///
/// Driven through the runtime's own admission function rather than HTTP for
/// exactly that reason: the collision is a question about a *specific* Work
/// id, and only a caller that chooses the id can ask it.
#[test]
fn a_worktree_path_collision_names_its_owner_and_the_override_does_not_help() {
    use sergeant_rs::domain::estate::Estate;
    use sergeant_rs::runtime::preflight;
    use sergeant_rs::runtime::surface::surface_root;

    let repos = TempDir::new().expect("tempdir");
    let estate_root = repos.path().join("payments");
    scaffold_estate(&estate_root, "payments", &["api"]);
    let estate = Estate::resolve(&estate_root).expect("resolve");
    let data = repos.path().join("data");
    let surfaces = repos.path().join("surfaces");
    let work_id = "01COLLIDINGPATH000000";

    // Somebody else's surface is already at the path this Work would use.
    let occupied = surface_root(&surfaces, work_id).join("api");
    std::fs::create_dir_all(&occupied).expect("occupy");

    for override_flag in [false, true] {
        let refusal = preflight::run(
            &estate,
            &data,
            &surfaces,
            work_id,
            &estate.repositories,
            override_flag,
        )
        .expect_err("a path collision is refused either way");
        assert_eq!(
            refusal.code(),
            "git_preflight_worktree_path_collision",
            "override = {override_flag}: {refusal:?}"
        );
        assert!(
            refusal.findings[0].detail.contains(work_id),
            "§15: name the path and the Work that owns it: {}",
            refusal.findings[0].detail
        );
        assert!(!refusal.override_would_help());
    }
    assert!(occupied.exists(), "and it is never removed or reused");
}
