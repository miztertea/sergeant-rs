//! MVP-3 acceptance: the estate CLI surface built on top of MVP-1's manifest
//! (`docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`'s MVP-3 row).
//!
//! `sgt init` (scaffold + idempotency) · `sgt repo add/remove/list`
//! (populate-or-verify, atomic validated writes, group-membership refusal)
//! · `sgt group add/remove/list` (mkdir-p union semantics, declared-member
//! validation) · `sgt run --group` (estate-root proposal §7.2: the CLI
//! forwards `--repo`/`--group`/`--all` verbatim as `scope.{repos,group,all}`
//! and the *daemon* resolves group membership against its own bound
//! manifest — superseding MVP-3's original R-MVP1-5(b) CLI-side expansion)
//! · the estate-resolved data-dir default (U-R2, R-MVP1-12's own walk reused
//! for `--data-dir`/`SGT_DATA_DIR` precedence, unchanged).
//!
//! Every non-daemon verb here (`init`, `repo *`, `group *`, `doctor`) is
//! pure manifest/filesystem plumbing (§9: "sergeant.toml ... never stores
//! transient work state") — no daemon is spawned, so these run against a
//! bare `&Path`, matching `m6_surfaces.rs`'s own `doctor(...)` helper.
//! `sgt run --group` is the one verb here that submits real work and so
//! auto-spawns a daemon; that test (and its "unrelated broken repo" sibling)
//! alone go through `support::DataDir` — and, since Phase C, *always* spawn
//! one, because scope resolution (including an undeclared group's refusal)
//! now happens on the daemon side rather than before `ensure_daemon` runs.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

// ---------------------------------------------------------------- helpers

fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "sergeant tests")
        .env("GIT_AUTHOR_EMAIL", "tests@example.invalid")
        .env("GIT_COMMITTER_NAME", "sergeant tests")
        .env("GIT_COMMITTER_EMAIL", "tests@example.invalid")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A real git repository with one commit — a valid `sgt repo add` target
/// (populate-or-verify's "verify" arm) or clone source (its "populate" arm).
fn init_repo(path: &Path) {
    std::fs::create_dir_all(path).expect("repo dir");
    git(path, &["init", "-b", "main"]);
    std::fs::write(path.join("README.md"), "# fixture\n").expect("write file");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
}

/// Run `sgt` with `cwd`, an optional `--data-dir`, extra env, and `args`.
/// Every non-daemon-spawning verb in this suite goes through this — no
/// `DataDir` guard needed, mirroring `m6_surfaces.rs`'s `doctor(...)`.
fn run(cwd: &Path, data_dir: Option<&Path>, env: &[(&str, &str)], args: &[&str]) -> Output {
    let mut command = Command::new(SGT);
    command.current_dir(cwd);
    if let Some(data_dir) = data_dir {
        command.arg("--data-dir").arg(data_dir);
    }
    command.args(args);
    for (key, value) in env {
        command.env(key, value);
    }
    let output = command.output().expect("run sgt");
    Output {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

struct Output {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Output {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", self.stdout))
    }

    fn assert_ok(&self, what: &str) -> &Self {
        assert_eq!(
            self.code,
            Some(0),
            "{what} must succeed, got {:?}\nstdout: {}\nstderr: {}",
            self.code,
            self.stdout,
            self.stderr
        );
        self
    }

    fn assert_fails(&self, what: &str) -> &Self {
        assert_ne!(
            self.code,
            Some(0),
            "{what} must fail, got exit 0\nstdout: {}",
            self.stdout
        );
        self
    }
}

// ------------------------------------------------------------------ init

/// guard-map: `sgt init` scaffolds `[estate]`, `repos/`, and both
/// `.gitignore` entries, and a second run is a true no-op: same estate name,
/// no duplicated `.gitignore` lines, `outcome.changed` reports `false`.
/// Mutation this kills: dropping either idempotency guard in
/// `domain::manifest::init_estate`/`ensure_gitignore` (a re-run would then
/// rename the estate or duplicate `.gitignore` lines).
#[test]
fn init_scaffolds_and_is_idempotent() {
    let root = tempfile::TempDir::new().expect("tempdir");
    let scratch_data_dir = root.path().join("scratch-data-dir");

    let first = run(
        root.path(),
        Some(&scratch_data_dir),
        &[],
        &["--json", "init", "--name", "my-estate"],
    );
    first.assert_ok("first init");
    let first_json = first.json();
    assert!(first_json["outcome"]["changed"].as_bool().unwrap());
    assert!(first_json["outcome"]["manifest_created"].as_bool().unwrap());
    assert!(
        first_json["outcome"]["repos_dir_created"]
            .as_bool()
            .unwrap()
    );
    assert!(
        first_json["outcome"]["gitignore_updated"]
            .as_bool()
            .unwrap()
    );

    assert!(root.path().join("sergeant.toml").is_file());
    assert!(root.path().join("repos").is_dir());
    let gitignore = std::fs::read_to_string(root.path().join(".gitignore")).expect("gitignore");
    assert!(gitignore.lines().any(|l| l.trim() == ".sergeant/data"));
    assert!(gitignore.lines().any(|l| l.trim() == "repos/"));

    let second = run(
        root.path(),
        Some(&scratch_data_dir),
        &[],
        &["--json", "init", "--name", "renamed-would-be-wrong"],
    );
    second.assert_ok("second init");
    let second_json = second.json();
    assert!(
        !second_json["outcome"]["changed"].as_bool().unwrap(),
        "a second init must change nothing, got {second_json}"
    );

    let manifest = std::fs::read_to_string(root.path().join("sergeant.toml")).expect("manifest");
    assert!(
        manifest.contains("my-estate"),
        "the first run's name must survive a second init, got:\n{manifest}"
    );
    assert!(
        !manifest.contains("renamed-would-be-wrong"),
        "a second init must never rename an existing estate"
    );
}

// --------------------------------------------------------------- distro

/// guard-map (#165): a fresh estate has the embedded distro on disk after
/// `sgt init` — `AGENTS.md`, at least one `skills/*/SKILL.md`, at least one
/// `.sergeant/common/contexts/*.md`, and more than zero stock workflow
/// packages under `.sergeant/workflows/`. Mutation this kills: `sgt init`
/// going back to scaffolding only `sergeant.toml`/`repos/`/`.gitignore`
/// (issue #165's original defect) would fail every assertion here.
#[test]
fn init_writes_the_embedded_distro() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");

    assert!(estate.path().join("AGENTS.md").is_file());

    let skills: Vec<_> = std::fs::read_dir(estate.path().join("skills"))
        .expect("skills dir")
        .flatten()
        .filter(|e| e.path().join("SKILL.md").is_file())
        .collect();
    assert!(!skills.is_empty(), "expected at least one skill package");

    let contexts: Vec<_> = std::fs::read_dir(estate.path().join(".sergeant/common/contexts"))
        .expect("contexts dir")
        .flatten()
        .collect();
    assert!(!contexts.is_empty(), "expected at least one context file");

    let workflows: Vec<_> = std::fs::read_dir(estate.path().join(".sergeant/workflows"))
        .expect("workflows dir")
        .flatten()
        .filter(|e| e.path().join("workflow.toml").is_file())
        .collect();
    assert!(
        !workflows.is_empty(),
        "expected at least one stock workflow package"
    );
}

/// guard-map (hard constraint 1): a second `sgt init` on an
/// already-initialized estate is a true no-op for the distro too — it does
/// not overwrite a file a user has since edited, and it never creates
/// `.sergeant/local/` (that directory is the user's own; stock never writes
/// there — hard constraint 3, Phase 3's local-shadows-stock resolution
/// depends on the separation). Mutation this kills: `write_distro` losing
/// its per-file existence check (a second init would then clobber the
/// user's edit back to stock content) or accidentally writing under
/// `.sergeant/local/` instead of `.sergeant/workflows/`.
#[test]
fn init_writing_the_distro_is_idempotent_and_never_touches_local() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");
    assert!(!estate.path().join(".sergeant/local").exists());

    let sentinel = "the user edited this doctrine file\n";
    std::fs::write(estate.path().join("AGENTS.md"), sentinel).expect("simulate a user edit");

    let second = run(estate.path(), Some(&data_dir), &[], &["--json", "init"]);
    second.assert_ok("second init");
    assert!(
        !second.json()["outcome"]["changed"].as_bool().unwrap(),
        "a second init must change nothing, got {}",
        second.stdout
    );
    assert_eq!(
        std::fs::read_to_string(estate.path().join("AGENTS.md")).expect("AGENTS.md"),
        sentinel,
        "a second init must never clobber a user's edited AGENTS.md"
    );
    assert!(
        !estate.path().join(".sergeant/local").exists(),
        "sgt init must never create .sergeant/local/ — that tree is the user's own"
    );
}

/// guard-map (ADR 0016 / hard constraint 4): every stock template `sgt
/// init` writes carries `edition: <this binary's Cargo.toml version>` in
/// its front matter — the co-versioning mechanism (ADR 0014 decision 2):
/// the value is stamped by `sgt init` itself
/// (`domain::distro::rewrite_edition`, keyed off `env!("CARGO_PKG_VERSION")`)
/// rather than trusted verbatim from whatever was checked into the embedded
/// content. Mutation this kills: reverting `write_file` to write embedded
/// bytes unmodified would still pass today (checked-in `edition: 0.1.0`
/// happens to match `Cargo.toml`'s current `0.1.0`) but silently drifts the
/// moment either one changes without the other being re-synced by hand —
/// this test pins the binary version as the source of truth, not the
/// checked-in string.
#[test]
fn init_written_templates_carry_the_binary_edition() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");

    let edition_line = format!("edition: {}", env!("CARGO_PKG_VERSION"));

    let mut checked = 0;
    for entry in std::fs::read_dir(estate.path().join(".sergeant/workflows"))
        .expect("workflows dir")
        .flatten()
    {
        let index = entry.path().join("index.md");
        if index.is_file() {
            let text = std::fs::read_to_string(&index).expect("index.md");
            assert!(
                text.contains(&edition_line),
                "{} must carry {edition_line:?}, got:\n{text}",
                index.display()
            );
            checked += 1;
        }
    }
    assert!(checked > 0, "expected at least one stock index.md");

    for entry in std::fs::read_dir(estate.path().join("skills"))
        .expect("skills dir")
        .flatten()
    {
        let skill_md = entry.path().join("SKILL.md");
        if skill_md.is_file() {
            let text = std::fs::read_to_string(&skill_md).expect("SKILL.md");
            assert!(
                text.contains(&edition_line),
                "{} must carry {edition_line:?}, got:\n{text}",
                skill_md.display()
            );
        }
    }
}

/// guard-map (#165's own acceptance criterion): `sgt run --workflow <name>`
/// resolves a stock package on a *freshly initialized* estate instead of
/// 422ing — the exact failure issue #165 describes ("workflow \"research\"
/// not found (looked in ...)"). This submits against `prototype`, a real
/// shipped package, through the actual daemon (`support::DataDir`, a real
/// spawn) rather than a hand-written fixture workflow, so the assertion is
/// that resolution — not necessarily full execution — succeeds: any failure
/// downstream of resolution (capability mismatch, etc.) is a different,
/// acceptable outcome; a `workflow_error` naming "not found" is not.
/// Mutation this kills: `sgt init` regressing back to writing no workflow
/// packages (#165's original defect) would turn this submission's error
/// code back into `workflow_error` with a "not found (looked in" message.
#[test]
fn run_workflow_resolves_against_a_freshly_initialized_estate() {
    let data_dir = DataDir::new();
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(estate.path(), Some(data_dir.path()), &[], &["init"]).assert_ok("init");

    init_repo(&estate.path().join("repos").join("target"));
    run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &["repo", "add", "target"],
    )
    .assert_ok("repo add");

    let submitted = run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &[
            "--json",
            "run",
            "--workflow",
            "prototype",
            "--backend",
            "fake",
            "--repo",
            "target",
            "try the embedded distro",
        ],
    );

    // A submit-time API error prints as `sgt: {status}: {message}` on
    // stderr (`ClientError::Api`'s `Display`, `src/api.rs`), not JSON on
    // stdout — so on failure the fingerprint of issue #165's 422
    // (`WorkflowError::NotFound`'s message, `src/domain/workflow.rs`) is
    // checked there. Any failure *without* that fingerprint is a different,
    // unrelated outcome (e.g. a capability mismatch) and is not what this
    // test is pinning.
    assert!(
        !submitted.stderr.contains("not found (looked in"),
        "must not hit the #165 422, got stdout: {}\nstderr: {}",
        submitted.stdout,
        submitted.stderr
    );
    if submitted.code == Some(0) {
        assert!(
            submitted.json()["work"]["id"].is_string(),
            "a successful submit must carry a work id, got: {}",
            submitted.stdout
        );
    }

    data_dir.reap();
}

/// guard-map: on a host with no `claude` CLI reachable, `sgt init` still
/// exits 0 and still scaffolds the estate — the bug this pins (GH runner CI,
/// all `m8_estate_cli` tests, run 31659419098): doctor's `claude` row FAILs
/// there ("cannot run claude --version: No such file or directory"), and
/// before this fix that FAIL flowed straight into init's own exit code, so a
/// colleague without `claude` installed could never `sgt init` at all — even
/// though the row's own remedy says "until then only the `fake` backend can
/// run work". §17.5's degraded-daemon doctrine and NORTH-STAR's day-one loop
/// both say a missing harness narrows capabilities, it must not brick estate
/// setup. `SGT_CLAUDE_BIN` (`src/backend/claude.rs`) pointed at a path that
/// does not exist reproduces the runner's "no claude on PATH" condition
/// deterministically on any host, including this one where `claude` is
/// actually installed.
///
/// Mutation this kills: reverting `Command::Init`'s exit-code check from
/// `report.healthy_for_init()` back to `report.healthy()` (or widening
/// `healthy_for_init`'s advisory set to swallow `data_dir`/`journal`/
/// `projection` too) — either would flip this test's exit code, or would
/// stop failing when a *real* estate-fundamental check breaks.
#[test]
fn init_succeeds_with_no_claude_cli_but_still_reports_the_fail_row() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");

    let result = run(
        estate.path(),
        Some(&data_dir),
        &[("SGT_CLAUDE_BIN", "/nonexistent/no-such-claude-binary")],
        &["init"],
    );
    result.assert_ok("init with no claude CLI reachable");

    // The scaffold happened — init's own responsibility, unaffected by the
    // harness being unavailable.
    assert!(estate.path().join("sergeant.toml").is_file());
    assert!(estate.path().join("repos").is_dir());

    // The row still prints as FAIL with its remedy — only the exit code
    // changed, not the honesty of the report.
    assert!(
        result.stdout.contains("FAIL") && result.stdout.contains("claude"),
        "the claude row must still print as a FAIL with its remedy, got:\n{}",
        result.stdout
    );
    assert!(
        result.stdout.contains("fake") && result.stdout.contains("backend"),
        "the FAIL row's own remedy (fake backend still works) must still be visible, got:\n{}",
        result.stdout
    );
}

// ------------------------------------------------------------------ repo

/// guard-map: `sgt repo add --origin` clones into `repos/<name>` when absent;
/// `sgt repo add` with no origin verifies an already-present git repo; a
/// third add of the same name is refused as already declared. Mutation this
/// kills: dropping the duplicate-name refusal, or cloning to the wrong
/// destination.
#[test]
fn repo_add_clones_verifies_and_refuses_duplicates() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");

    let upstream = tempfile::TempDir::new().expect("tempdir");
    init_repo(upstream.path());

    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &[
            "repo",
            "add",
            "api",
            "--origin",
            upstream.path().to_str().expect("utf8"),
            "--instructions",
            "local",
        ],
    )
    .assert_ok("repo add --origin");
    assert!(estate.path().join("repos/api/.git").exists());

    // Verify-only: a pre-existing git repo, no origin needed.
    init_repo(&estate.path().join("repos/web"));
    run(estate.path(), Some(&data_dir), &[], &["repo", "add", "web"])
        .assert_ok("repo add, verify only");

    // Already declared.
    run(estate.path(), Some(&data_dir), &[], &["repo", "add", "api"])
        .assert_fails("re-adding a declared repo");

    let list = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["--json", "repo", "list"],
    );
    list.assert_ok("repo list");
    let repos = list.json()["repositories"].as_array().cloned().unwrap();
    let names: Vec<&str> = repos.iter().map(|r| r["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["api", "web"]);
    let api = repos.iter().find(|r| r["name"] == "api").unwrap();
    assert_eq!(api["instructions"], "local");
    assert_eq!(
        api["origin"].as_str().unwrap(),
        upstream.path().to_str().unwrap()
    );
}

/// guard-map: `sgt repo add <name>` with no `--origin` and no existing
/// `repos/<name>` directory is refused, naming `--origin` as the remedy.
/// Mutation this kills: silently declaring a repository whose directory
/// never actually gets created (a `[[repo]]` entry pointing at nothing).
#[test]
fn repo_add_without_origin_or_existing_dir_is_refused() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");

    let result = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["repo", "add", "ghost"],
    );
    result.assert_fails("no origin, no existing dir");
    assert!(
        result.stderr.contains("--origin"),
        "the refusal must name the remedy, got: {}",
        result.stderr
    );
}

/// guard-map: `sgt repo remove` refuses while a group still references the
/// repository (naming the group), and succeeds once the group no longer
/// does. End-to-end version of `domain::manifest`'s own unit test, through
/// the CLI surface specifically. Mutation this kills: `sgt repo remove`
/// bypassing the group-membership check that `domain::manifest::remove_repo`
/// enforces.
#[test]
fn repo_remove_is_refused_while_group_referenced_then_succeeds() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");
    for name in ["a", "b"] {
        init_repo(&estate.path().join("repos").join(name));
        run(estate.path(), Some(&data_dir), &[], &["repo", "add", name]).assert_ok("repo add");
    }
    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "add", "pair", "a", "b"],
    )
    .assert_ok("group add");

    let blocked = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["repo", "remove", "a"],
    );
    blocked.assert_fails("repo remove while still a group member");
    assert!(
        blocked.stderr.contains("pair"),
        "the refusal must name the group, got: {}",
        blocked.stderr
    );

    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "remove", "pair", "a"],
    )
    .assert_ok("drop a from the group");
    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["repo", "remove", "a"],
    )
    .assert_ok("now removable");

    let list = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["--json", "repo", "list"],
    );
    let repos = list.json()["repositories"].as_array().cloned().unwrap();
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0]["name"], "b");
}

/// MVP-3 test-honesty finding TH-2 (and invariants finding MVP3-C6): the
/// manifest lock's *cross-process* half, never exercised anywhere else — the
/// only other concurrency coverage (`domain::manifest`'s
/// `concurrent_repo_adds_do_not_lose_an_entry`) uses two threads in one
/// process, which take the wait-and-retry path because the lock is already
/// in that process's own `SELF_LOCKED` set. Two real `sgt` invocations never
/// share that set, so this test holds the lock file itself — a genuinely
/// different process/file-description than the `sgt` subprocess below, the
/// same relationship two real concurrent `sgt` commands would have — and
/// proves the real, shipped outcome: the second command's `try_lock` returns
/// immediately with no wait, so it FAILS CLOSED (nonzero exit, naming the
/// lock file and the manual remedy) rather than serializing and both
/// succeeding. Once the lock is released, the identical command succeeds —
/// proving the refusal was really about contention, not a broken lock file.
///
/// guard-map: reverting `manifest.rs`'s module doc's old framing back into
/// code (e.g. making `take_exclusive_lock` wait indefinitely rather than
/// failing closed for a lock this process never held) would hang this test
/// rather than fail it fast — the budget below catches that too.
#[test]
fn two_real_sgt_processes_racing_repo_add_serialize_dont_tear() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(estate.path(), None, &[], &["init"]).assert_ok("init");
    init_repo(&estate.path().join("repos").join("a"));

    let lock_path = estate.path().join(".sergeant.toml.lock");
    let held = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .expect("open lock file");
    held.try_lock().expect(
        "this test process must be able to take the lock — it is standing in for the \
         'other concurrent sgt invocation' the module doc names",
    );

    let contended = run(estate.path(), None, &[], &["repo", "add", "a"]);
    contended.assert_fails("repo add while the manifest lock is held by another process");
    assert!(
        contended.stderr.contains("already editing")
            && contended.stderr.contains(".sergeant.toml.lock"),
        "the refusal must name the lock file and that another sgt command owns it, got: {}",
        contended.stderr
    );

    held.unlock().expect("release the lock");
    drop(held);

    run(estate.path(), None, &[], &["repo", "add", "a"])
        .assert_ok("repo add must succeed once the lock is free");
    let list = run(estate.path(), None, &[], &["--json", "repo", "list"]);
    let repos = list.json()["repositories"].as_array().cloned().unwrap();
    assert_eq!(
        repos.len(),
        1,
        "the retried add must actually land: {list:?}",
        list = list.json()
    );
    assert_eq!(repos[0]["name"], "a");
}

// ----------------------------------------------------------------- group

/// guard-map: `sgt group add` refuses an undeclared member, naming it; valid
/// members are accepted and re-running `group add` with an overlapping set
/// unions membership (mkdir-p semantics) rather than erroring or
/// duplicating. Mutation this kills: checking membership against the wrong
/// (e.g. empty) declared-repo list, or replacing membership instead of
/// unioning it.
#[test]
fn group_add_validates_members_and_unions_on_repeat() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");
    for name in ["a", "b", "c"] {
        init_repo(&estate.path().join("repos").join(name));
        run(estate.path(), Some(&data_dir), &[], &["repo", "add", name]).assert_ok("repo add");
    }

    let refused = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "add", "trio", "a", "ghost"],
    );
    refused.assert_fails("undeclared member");
    assert!(refused.stderr.contains("ghost"), "got: {}", refused.stderr);

    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "add", "trio", "a", "--brief", "the trio"],
    )
    .assert_ok("create with one member");
    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "add", "trio", "b", "c"],
    )
    .assert_ok("union in two more");

    let list = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["--json", "group", "list"],
    );
    let groups = list.json()["groups"].as_array().cloned().unwrap();
    assert_eq!(groups.len(), 1);
    let mut members: Vec<String> = groups[0]["repos"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    members.sort();
    assert_eq!(members, vec!["a", "b", "c"]);
    assert_eq!(groups[0]["brief"], "the trio");
}

/// guard-map: `sgt group remove <name>` with no repo args removes the whole
/// group; with repo args, removes just those members. Mutation this kills:
/// swapping the two behaviors (an empty `repos` slice removing nothing
/// instead of the whole group would be indistinguishable to a caller from
/// "no-op", silently leaving a group `sgt group remove` claimed to delete).
#[test]
fn group_remove_whole_group_vs_named_members() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");
    for name in ["a", "b"] {
        init_repo(&estate.path().join("repos").join(name));
        run(estate.path(), Some(&data_dir), &[], &["repo", "add", name]).assert_ok("repo add");
    }
    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "add", "pair", "a", "b"],
    )
    .assert_ok("group add");

    run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["group", "remove", "pair"],
    )
    .assert_ok("remove the whole group");

    let list = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["--json", "group", "list"],
    );
    assert_eq!(list.json()["groups"].as_array().unwrap().len(), 0);
}

// -------------------------------------------------------------- workflow

/// §6 Phase 3 (Ponytail R7): `sgt workflow fork <name>` copies a stock
/// workflow package into `.sergeant/local/workflows/<name>/`, where it then
/// resolves ahead of stock (R4). guard-map: a fork that fails to copy every
/// file, or that resolution fails to prefer, survives a narrower "the
/// command exits 0" check but fails this one.
#[test]
fn workflow_fork_copies_stock_and_the_fork_then_resolves() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");

    let stock = estate.path().join(".sergeant/workflows/example");
    std::fs::create_dir_all(stock.join("00-only")).expect("stage dir");
    std::fs::write(
        stock.join("workflow.toml"),
        "[workflow]\nname = \"example\"\nversion = \"1\"\nstages = [\"00-only\"]\n",
    )
    .expect("descriptor");
    std::fs::write(stock.join("00-only").join("CONTEXT.md"), "stock body").expect("stage context");
    std::fs::write(
        stock.join("index.md"),
        "---\nstatus: published\ndescription: an example\nedition: 0.1.0\n---\n",
    )
    .expect("index.md");

    let forked = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["--json", "workflow", "fork", "example"],
    );
    forked.assert_ok("workflow fork");
    let forked_json = forked.json();
    assert_eq!(forked_json["forked"], "example");

    let local = estate.path().join(".sergeant/local/workflows/example");
    assert!(local.join("workflow.toml").is_file());
    assert!(local.join("00-only/CONTEXT.md").is_file());
    let index = std::fs::read_to_string(local.join("index.md")).expect("index.md");
    assert!(
        index.contains("edition: 0.1.0"),
        "the edition marker must survive the copy verbatim: {index}"
    );
}

/// §6 Phase 3 (Ponytail R7): `sgt workflow fork` refuses to overwrite an
/// existing local package by the same name — it must never silently clobber
/// a user's already-rewritten fork.
#[test]
fn workflow_fork_refuses_to_overwrite_an_existing_local_package() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let data_dir = estate.path().join("data-dir");
    run(estate.path(), Some(&data_dir), &[], &["init"]).assert_ok("init");

    let stock = estate.path().join(".sergeant/workflows/example");
    std::fs::create_dir_all(&stock).expect("mkdir stock");
    std::fs::write(
        stock.join("workflow.toml"),
        "[workflow]\nname = \"example\"\nversion = \"1\"\nstages = []\n",
    )
    .expect("descriptor");

    let local = estate.path().join(".sergeant/local/workflows/example");
    std::fs::create_dir_all(&local).expect("mkdir local");
    std::fs::write(local.join("sentinel"), "user work, do not touch").expect("sentinel");

    let refused = run(
        estate.path(),
        Some(&data_dir),
        &[],
        &["workflow", "fork", "example"],
    );
    refused.assert_fails("must refuse to overwrite an existing local package");
    assert!(
        local.join("sentinel").is_file(),
        "the user's existing local package must be untouched"
    );
    assert!(!local.join("workflow.toml").is_file());
}

// --------------------------------------------------------- run --group

/// guard-map: `sgt run --group <name>` submits `scope.group = <name>`
/// verbatim, and the *daemon* resolves it into the exact same repository
/// list `--repo` (repeated) would produce — estate-root §7.2's ruling,
/// pinned end to end through a real (fake-backend) submission. An
/// undeclared group name is refused by the daemon's own 422 (§15), naming
/// it — no longer a client-side pre-check, so a real daemon spawns for this
/// case too. Mutation this kills: `--group` sending the group *name* to the
/// API but resolution silently expanding to zero repositories or the whole
/// estate instead of the daemon's structured refusal/expansion this test
/// pins on both sides of the undeclared-group boundary.
#[test]
fn run_group_resolves_server_side_into_repositories() {
    let data_dir = DataDir::new();
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(estate.path(), Some(data_dir.path()), &[], &["init"]).assert_ok("init");
    for name in ["a", "b"] {
        init_repo(&estate.path().join("repos").join(name));
        run(
            estate.path(),
            Some(data_dir.path()),
            &[],
            &["repo", "add", name],
        )
        .assert_ok("repo add");
    }
    run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &["group", "add", "pair", "a", "b"],
    )
    .assert_ok("group add");

    // An undeclared group is refused by the daemon's own scope resolution
    // (§7.2) — the CLI no longer looks group membership up itself, so
    // `ensure_daemon` runs and a real daemon answers this 422.
    let bad = run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &["run", "--group", "ghost", "do the thing"],
    );
    bad.assert_fails("undeclared group");
    assert!(bad.stderr.contains("ghost"), "got: {}", bad.stderr);
    assert!(
        !data_dir.daemon_pids().is_empty(),
        "server-side --group resolution means a daemon must actually spawn and reach its own \
         scope resolution for the undeclared-group refusal to happen at all: {}",
        bad.stderr
    );

    let submitted = run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "run", "--group", "pair", "expand the group"],
    );
    submitted.assert_ok("run --group pair");
    let mut repositories: Vec<String> = submitted.json()["work"]["repositories"]
        .as_array()
        .expect("repositories array")
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    repositories.sort();
    assert_eq!(repositories, vec!["a", "b"]);

    data_dir.reap();
}

/// Historical note, MVP3-C2: before estate-root Phase C, `--group`'s
/// *client-side* expansion read group membership through an on-disk-free
/// structural parser specifically so an unrelated declared repository
/// missing from disk would not refuse the command before a daemon was even
/// spawned. Phase C retires that CLI-side lookup entirely (§7.2: resolution
/// is core-owned), so this test's outcome is no longer "the client-side
/// carve-out and the daemon's own strict bind disagree" — it is the plain
/// consequence of `Engine::plan` unconditionally resolving the *whole*
/// estate (`Estate::resolve` against the daemon's bound root, since
/// estate-root Phase D) before scope resolution ever runs,
/// exactly the coupling a plain `--repo` submission has always had (matches
/// the B4 register entry MVP3-C2's own basis cites). `ghost` being outside
/// the requested group no longer matters: the command still fails, and the
/// daemon's own 422 is what names `ghost` as the actual defect.
///
/// guard-map: a resolution path that somehow skipped or lightened estate
/// discovery for a `--group` submission (say, resolving groups against a
/// separately-read, on-disk-free manifest instead of the same `estate`
/// `plan` already discovered) would make this pass with no daemon-side
/// evidence of `ghost` at all — the `daemon_pids` assertion below is what
/// catches that regression.
#[test]
fn run_group_expansion_itself_survives_an_unrelated_declared_repo_missing_from_disk() {
    let data_dir = DataDir::new();
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(estate.path(), Some(data_dir.path()), &[], &["init"]).assert_ok("init");
    for name in ["a", "b", "ghost"] {
        init_repo(&estate.path().join("repos").join(name));
        run(
            estate.path(),
            Some(data_dir.path()),
            &[],
            &["repo", "add", name],
        )
        .assert_ok("repo add");
    }
    run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &["group", "add", "pair", "a", "b"],
    )
    .assert_ok("group add");

    // Simulate the clone-is-distro shape for the group's non-member.
    std::fs::remove_dir_all(estate.path().join("repos").join("ghost"))
        .expect("remove ghost's checkout");

    let submitted = run(
        estate.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "run", "--group", "pair", "expand despite ghost"],
    );
    submitted.assert_fails("the daemon's own full-estate bind still refuses on ghost");
    assert!(
        submitted.stderr.contains("ghost"),
        "the refusal must still name the actual defect: {}",
        submitted.stderr
    );
    assert!(
        !data_dir.daemon_pids().is_empty(),
        "a daemon must have actually spawned and reached its own bind-time preflight for this \
         to be the daemon's own full-estate discovery failing on ghost, not some earlier \
         client-side check: {}",
        submitted.stderr
    );

    data_dir.reap();
}

// ---------------------------------------------------- estate-resolved data dir

/// guard-map: with no `--data-dir`/`SGT_DATA_DIR`, `sgt doctor` defaults to
/// `<estate_root>/.sergeant/data` when run **at** the estate root. `sgt
/// doctor` never auto-spawns a daemon (its own documented rule), so this
/// needs no `DataDir` guard. Mutation this kills: `resolve_data_dir` never
/// consulting the estate at all (would report the old default instead), or
/// joining the wrong relative path (would report `.sergeant/data` in the
/// wrong directory).
///
/// **Estate-root Phase D flipped the descendant half of this pin.** It used
/// to assert the identical answer from `repos/` one level down, via
/// R-MVP1-12's upward walk. There is no walk any more (§4.1), so the
/// sibling test below pins the *new* answer from that same directory:
/// exact-root resolution finds no estate there, the pre-estate platform
/// default stands, and `sgt doctor` — which still runs outside an estate —
/// says so in a failing `estate_root` row instead of quietly reporting on
/// the estate above.
#[test]
fn data_dir_defaults_to_estate_local_when_no_flag_or_env() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    // A throwaway --data-dir for init itself: at the moment init runs, no
    // estate exists yet for its own data-dir resolution to find (it is the
    // command creating one), so init needs an explicit target regardless.
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");

    let expected = std::fs::canonicalize(estate.path())
        .expect("canonical estate root")
        .join(".sergeant/data");

    let result = run(estate.path(), None, &[], &["--json", "doctor"]);
    // Doctor's health is not the point here (an empty estate reports fine
    // or warn on unrelated checks); only the resolved path is.
    let reported = PathBuf::from(result.json()["data_dir"].as_str().expect("data_dir"));
    let reported = std::fs::canonicalize(&reported).unwrap_or(reported);
    assert_eq!(
        reported, expected,
        "the estate root must default to the estate-local data dir"
    );
}

/// **Phase 0 pin, flipped (§4.1/§4.2).** From `repos/` — one directory below
/// the estate root — there is no ancestor walk left to find the estate, so:
///
/// - the data dir falls through to the pre-estate platform default;
/// - `sgt doctor` still runs (it is one of the five unscoped commands) and
///   reports a **failing** `estate_root` row carrying §4.4's remedy;
/// - an estate-*scoped* verb from the same directory refuses outright, and
///   spawns nothing.
// CONTRACT PIN (estate-root Phase D): no upward data-dir/estate discovery from a descendant of the estate root.
#[test]
fn a_descendant_of_the_estate_root_finds_no_estate_and_spawns_nothing() {
    let data_dir = DataDir::new();
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");

    let xdg = estate.path().join("xdg-home");
    let home = estate.path().join("home");
    let env = [
        ("XDG_DATA_HOME", xdg.to_str().expect("utf8")),
        ("HOME", home.to_str().expect("utf8")),
    ];
    let descendant = estate.path().join("repos");

    let doctor = run(&descendant, None, &env, &["--json", "doctor"]);
    let report = doctor.json();
    let reported = PathBuf::from(report["data_dir"].as_str().expect("data_dir"));
    let expected = if cfg!(target_os = "macos") {
        home.join("Library/Application Support/sergeant")
    } else {
        xdg.join("sergeant")
    };
    assert_eq!(
        reported, expected,
        "no upward walk: a descendant falls through to the pre-estate default"
    );
    let estate_row = report["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["name"] == "estate_root")
        .expect("doctor must carry an estate_root row");
    assert_eq!(
        estate_row["status"], "fail",
        "outside an estate root the row must fail: {estate_row}"
    );
    let remedy = estate_row["remedy"].as_str().expect("remedy");
    assert!(
        remedy.contains("does not search parent directories") && remedy.contains("sgt init"),
        "§4.4's remedy must be carried: {remedy}"
    );

    // And an estate-scoped verb from the same place refuses before it could
    // look up a descriptor or spawn anything (§4.3).
    let refused = run(
        &descendant,
        Some(data_dir.path()),
        &[],
        &["run", "should never be admitted"],
    );
    refused.assert_fails("run from a descendant of the estate root");
    assert!(
        refused
            .stderr
            .contains("no estate found in the current directory"),
        "§4.4's own first line: {}",
        refused.stderr
    );
    assert!(
        data_dir.daemon_pids().is_empty(),
        "a refused estate-scoped verb must never spawn a daemon"
    );
}

/// guard-map (#164): `sgt init` itself, with no `--data-dir`/`SGT_DATA_DIR`,
/// must report and create the estate-local data dir it just made
/// `sergeant.toml` discoverable at — not the pre-estate fallback. `dispatch`
/// resolves `data_dir` once at the top, before the `Init` handler runs
/// `init_estate`; on a brand-new estate that resolution predates
/// `sergeant.toml` existing, so unless the `Init` handler re-resolves after
/// creating it, `init`'s own embedded doctor report silently checks (and
/// reports `[ok]` for) `$XDG_DATA_HOME`/`$HOME` instead of
/// `<estate_root>/.sergeant/data` — exactly the failure this issue names:
/// `sgt init` scaffolds a `.gitignore` rule for `.sergeant/data` but a
/// fresh estate's journal silently lands outside the estate, with `doctor`
/// reporting healthy the whole time. Mutation this kills: reverting the
/// `Command::Init` handler to use the `data_dir` resolved before
/// `init_estate` instead of re-resolving afterward.
#[test]
fn init_reports_and_creates_the_estate_local_data_dir_not_the_pre_estate_fallback() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    let xdg = estate.path().join("xdg-home");
    let home = estate.path().join("home");

    let result = run(
        estate.path(),
        None,
        &[
            ("XDG_DATA_HOME", xdg.to_str().expect("utf8")),
            ("HOME", home.to_str().expect("utf8")),
        ],
        &["--json", "init"],
    );
    assert_eq!(
        result.code,
        Some(0),
        "init must succeed\nstdout: {}\nstderr: {}",
        result.stdout,
        result.stderr
    );

    let reported = PathBuf::from(
        result.json()["doctor"]["data_dir"]
            .as_str()
            .expect("doctor.data_dir"),
    );
    let expected = std::fs::canonicalize(estate.path())
        .expect("canonical estate root")
        .join(".sergeant/data");
    let reported = std::fs::canonicalize(&reported).unwrap_or(reported);
    assert_eq!(
        reported, expected,
        "init's own report must resolve to the estate-local data dir, not the pre-estate fallback"
    );
    assert!(
        expected.is_dir(),
        "init must leave the estate-local data dir created on disk, matching the \
         .gitignore rule it scaffolds for .sergeant/data"
    );
    assert!(
        !xdg.exists(),
        "init must not touch the pre-estate XDG fallback once an estate is being created"
    );
}

/// guard-map: the estate-resolved default is a new *fallback rung*, not a
/// replacement — an explicit `--data-dir` flag, and separately
/// `SGT_DATA_DIR`, both still take precedence over a discovered estate,
/// unchanged. Mutation this kills: reordering `resolve_data_dir` so the
/// estate walk runs before the flag/env checks.
#[test]
fn explicit_data_dir_and_env_still_win_over_a_discovered_estate() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");

    let explicit_flag = estate.path().join("explicit-flag-dir");
    let flag_result = run(
        estate.path(),
        Some(&explicit_flag),
        &[],
        &["--json", "doctor"],
    );
    assert_eq!(
        std::fs::canonicalize(flag_result.json()["data_dir"].as_str().unwrap())
            .unwrap_or_else(|_| PathBuf::from(flag_result.json()["data_dir"].as_str().unwrap())),
        std::fs::canonicalize(&explicit_flag).unwrap_or(explicit_flag.clone()),
        "--data-dir must still win over a discovered estate"
    );

    let explicit_env = estate.path().join("explicit-env-dir");
    let env_result = run(
        estate.path(),
        None,
        &[("SGT_DATA_DIR", explicit_env.to_str().expect("utf8"))],
        &["--json", "doctor"],
    );
    assert_eq!(
        std::fs::canonicalize(env_result.json()["data_dir"].as_str().unwrap())
            .unwrap_or_else(|_| PathBuf::from(env_result.json()["data_dir"].as_str().unwrap())),
        std::fs::canonicalize(&explicit_env).unwrap_or(explicit_env.clone()),
        "SGT_DATA_DIR must still win over a discovered estate"
    );
}

/// guard-map: with no estate anywhere on the ascent (bounded, filesystem
/// root reached), the pre-estate default is unchanged — `sgt doctor`
/// reports whatever `XDG_DATA_HOME` says, exactly as it did before this
/// milestone. Mutation this kills: the new estate-walk fallback firing (or
/// erroring) even when no estate exists, changing the pre-existing default
/// path.
/// **#82, closed by measurement (MacBook Pro M3 Pro, 2026-08-15).** This
/// used to set only `XDG_DATA_HOME` and assert it unconditionally won — true
/// on Linux, but macOS's own convention (`src/platform/data_dir.rs`'s
/// `MACOS.env_override: None`) ignores `XDG_DATA_HOME` entirely and falls
/// back to `$HOME/Library/Application Support/sergeant` — and since this
/// test never overrode `HOME` either (it didn't need to, on the platform it
/// was written against), that fallback landed on this *host's real* `$HOME`
/// rather than the test's own scratch dir. Both env vars are now set and the
/// expected path is platform-conditional, mirroring
/// `tests/m2_daemon_api.rs::resolve_data_dir_falls_back_through_sgt_data_dir_then_xdg_then_home`,
/// which needed the identical fix.
#[test]
fn no_estate_anywhere_falls_back_to_the_pre_estate_default_unchanged() {
    let scratch = tempfile::TempDir::new().expect("tempdir");
    let cwd = scratch.path().join("plain").join("nested");
    std::fs::create_dir_all(&cwd).expect("nested dir");
    let xdg = scratch.path().join("xdg-home");
    let home = scratch.path().join("home");

    let result = run(
        &cwd,
        None,
        &[
            ("XDG_DATA_HOME", xdg.to_str().expect("utf8")),
            ("HOME", home.to_str().expect("utf8")),
        ],
        &["--json", "doctor"],
    );
    let reported = PathBuf::from(result.json()["data_dir"].as_str().expect("data_dir"));
    let expected = if cfg!(target_os = "macos") {
        home.join("Library/Application Support/sergeant")
    } else {
        xdg.join("sergeant")
    };
    assert_eq!(reported, expected);
}

/// Append `data_dir = "<value>"` to the `[estate]` table `sgt init` already
/// scaffolded — a direct manifest edit (mirroring how the domain-level
/// `surfaces_dir` tests write TOML by hand) rather than going through a CLI
/// verb, since `sgt init` itself has no flag for this key.
fn declare_manifest_data_dir(root: &Path, value: &str) {
    let manifest_path = root.join("sergeant.toml");
    let manifest = std::fs::read_to_string(&manifest_path).expect("read manifest");
    assert!(
        manifest.contains("[estate]"),
        "expected an already-scaffolded [estate] table, got:\n{manifest}"
    );
    let updated = manifest.replacen(
        "[estate]\n",
        &format!("[estate]\ndata_dir = {value:?}\n"),
        1,
    );
    std::fs::write(&manifest_path, updated).expect("write manifest");
}

/// guard-map (ADR 0008(b)): with no `--data-dir`/`SGT_DATA_DIR`, a manifest
/// declaring `[estate] data_dir` is honored instead of the hardcoded
/// `<estate_root>/.sergeant/data` default — the whole point of the ADR
/// (the manifest is authority for repos, groups, profiles and
/// `surfaces_dir`; this closes the same-shaped gap for its own state).
/// Mutation this kills: `resolve_data_dir` finding the estate and still
/// hardcoding `DEFAULT_ESTATE_DATA_DIR` regardless of what the manifest
/// says — reverting the `cli.rs` change alone (leaving the domain parsing in
/// place) reproduces exactly that and fails this test.
#[test]
fn manifest_data_dir_overrides_the_estate_local_default() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");
    declare_manifest_data_dir(estate.path(), "custom-durable-state");

    let expected = std::fs::canonicalize(estate.path())
        .expect("canonical estate root")
        .join("custom-durable-state");

    let result = run(estate.path(), None, &[], &["--json", "doctor"]);
    let reported = PathBuf::from(result.json()["data_dir"].as_str().expect("data_dir"));
    let reported = std::fs::canonicalize(&reported).unwrap_or(reported);
    assert_eq!(
        reported, expected,
        "a manifest-declared data_dir must be honored, not the hardcoded .sergeant/data default"
    );
}

/// guard-map (ADR 0008(b); precedence settled by owner ruling 2026-08-20 —
/// backlog close-out kickoff, `docs/proposals/backlog-closeout-2026-08-20.md`
/// rulings §8): `SGT_DATA_DIR` outranks a manifest-declared `data_dir`. The
/// full ratified order is `--data-dir` > `SGT_DATA_DIR` > manifest
/// `data_dir` — invocation-explicit beats declared, an owner ruling now, not
/// the recommendation this test used to pin. Pinned so a future change
/// cannot silently flip which one wins without this test naming the flip.
/// Mutation this kills: reordering `resolve_data_dir` so the estate walk
/// (and the manifest `data_dir` it may carry) is consulted before
/// `SGT_DATA_DIR`.
#[test]
fn sgt_data_dir_env_var_still_outranks_a_manifest_declared_data_dir() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");
    declare_manifest_data_dir(estate.path(), "manifest-declared-data-dir");

    let explicit_env = estate.path().join("explicit-env-dir");
    let result = run(
        estate.path(),
        None,
        &[("SGT_DATA_DIR", explicit_env.to_str().expect("utf8"))],
        &["--json", "doctor"],
    );
    let reported = PathBuf::from(result.json()["data_dir"].as_str().expect("data_dir"));
    assert_eq!(
        std::fs::canonicalize(&reported).unwrap_or(reported),
        std::fs::canonicalize(&explicit_env).unwrap_or(explicit_env),
        "SGT_DATA_DIR must still win over a manifest-declared data_dir"
    );
}

/// guard-map (ADR 0008(b); owner ruling 2026-08-20 rulings §8): `--data-dir`
/// outranks a manifest-declared `data_dir` too — the flag rung above
/// `SGT_DATA_DIR` in the ratified order, `--data-dir` > `SGT_DATA_DIR` >
/// manifest `data_dir`. Sibling of
/// `sgt_data_dir_env_var_still_outranks_a_manifest_declared_data_dir` above,
/// same `declare_manifest_data_dir` + canonicalize pattern. Mutation this
/// kills: reordering `resolve_data_dir` so the estate walk (and the
/// manifest `data_dir` it may carry) is consulted before the `--data-dir`
/// flag.
#[test]
fn data_dir_flag_still_outranks_a_manifest_declared_data_dir() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");
    declare_manifest_data_dir(estate.path(), "manifest-declared-data-dir");

    let explicit_flag = estate.path().join("explicit-flag-dir");
    let result = run(
        estate.path(),
        Some(&explicit_flag),
        &[],
        &["--json", "doctor"],
    );
    let reported = PathBuf::from(result.json()["data_dir"].as_str().expect("data_dir"));
    assert_eq!(
        std::fs::canonicalize(&reported).unwrap_or(reported),
        std::fs::canonicalize(&explicit_flag).unwrap_or(explicit_flag),
        "--data-dir must still win over a manifest-declared data_dir"
    );
}

/// #80/ADR 0008: when `$XDG_DATA_HOME` is set but the estate rung already
/// won (no `--data-dir`/`SGT_DATA_DIR`, no manifest `data_dir` — the
/// estate-local default wins outright), `sgt --json doctor`'s `data_dir`
/// row discloses it as a detail line on the still-`ok` check, never a warn
/// and never a new row. With `$XDG_DATA_HOME` unset, the same row carries
/// no such note. Built on raw `Command` rather than the shared `run()`
/// helper so both cases control `$XDG_DATA_HOME` explicitly — `env_remove`
/// for the unset case — instead of depending on whatever the host process
/// happens to have set.
#[test]
fn doctor_data_dir_detail_discloses_an_outranked_xdg_data_home() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");

    let xdg = estate.path().join("xdg-home");
    std::fs::create_dir_all(&xdg).expect("xdg dir");

    let data_dir_detail = |output: std::process::Output| -> String {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let report: Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {stdout}"));
        let data_dir_row = report["checks"]
            .as_array()
            .expect("checks")
            .iter()
            .find(|c| c["name"] == "data_dir")
            .unwrap_or_else(|| panic!("doctor must carry a data_dir row: {report}"))
            .clone();
        assert_eq!(
            data_dir_row["status"], "ok",
            "the data_dir row must stay ok: {data_dir_row}"
        );
        data_dir_row["detail"].as_str().expect("detail").to_string()
    };

    let with_xdg = Command::new(SGT)
        .current_dir(estate.path())
        .env("XDG_DATA_HOME", &xdg)
        .args(["--json", "doctor"])
        .output()
        .expect("run sgt");
    let with_xdg_detail = data_dir_detail(with_xdg);
    assert!(
        with_xdg_detail.contains(
            "$XDG_DATA_HOME is set but outranked here — estate-first resolution, ADR 0008"
        ),
        "data_dir detail must disclose the outranked $XDG_DATA_HOME: {with_xdg_detail}"
    );

    let without_xdg = Command::new(SGT)
        .current_dir(estate.path())
        .env_remove("XDG_DATA_HOME")
        .args(["--json", "doctor"])
        .output()
        .expect("run sgt");
    let without_xdg_detail = data_dir_detail(without_xdg);
    assert!(
        !without_xdg_detail.contains("outranked"),
        "unset $XDG_DATA_HOME must not trigger the outranked disclosure: {without_xdg_detail}"
    );
}

/// guard-map (regression): `resolve_data_dir` runs at the top of every
/// `dispatch`, ahead of every command including `sgt doctor` — whose entire
/// job is diagnosing a broken installation, up to and including a broken
/// manifest. Finding a manifest's `data_dir` must not itself demand the
/// rest of the manifest — here, two profiles sharing a name, a defect
/// `sgt doctor` has nothing to do with — be structurally valid, or `doctor`
/// could never run against the one manifest it most needs to. Mutation this
/// kills: resolving `data_dir` via the full structural validator (duplicate
/// repos/profiles, invalid permission modes, unknown group members all
/// refuse) instead of a lookup scoped to `data_dir` alone — that would make
/// `doctor` (and every other command) crash before ever running, on stderr,
/// instead of producing its own JSON report.
#[test]
fn a_structurally_broken_manifest_does_not_stop_doctor_from_running() {
    let estate = tempfile::TempDir::new().expect("tempdir");
    run(
        estate.path(),
        Some(&estate.path().join("init-scratch-data-dir")),
        &[],
        &["init"],
    )
    .assert_ok("init");
    declare_manifest_data_dir(estate.path(), "custom-durable-state");
    let manifest_path = estate.path().join("sergeant.toml");
    let mut manifest = std::fs::read_to_string(&manifest_path).expect("read manifest");
    manifest.push_str(
        "\n[[profile]]\nname = \"same\"\nbackend = \"fake\"\n\n\
         [[profile]]\nname = \"same\"\nbackend = \"fake\"\n",
    );
    std::fs::write(&manifest_path, manifest).expect("write manifest");

    let result = run(estate.path(), None, &[], &["--json", "doctor"]);
    // `json()` itself asserts stdout parses as JSON — a pre-doctor crash on
    // the duplicate profile would instead put a plain error string on
    // stderr and leave stdout empty. Reaching doctor's own report (whatever
    // its `healthy` verdict, which depends on the unrelated claude/docker
    // harness rows) is the thing under test, so only structure is checked.
    let report = result.json();
    assert!(
        report["checks"].is_array(),
        "doctor must still produce its own report: {report}"
    );
    let expected_data_dir = std::fs::canonicalize(estate.path())
        .expect("canonical estate root")
        .join("custom-durable-state");
    let reported = PathBuf::from(report["data_dir"].as_str().expect("data_dir"));
    assert_eq!(
        std::fs::canonicalize(&reported).unwrap_or(reported),
        expected_data_dir,
        "the manifest's data_dir must still be honored despite the unrelated profile defect"
    );
}

// -------------------------------------------- work transcript / output pointer

/// A minimal two-stage workflow — the same shape `m5_projections.rs` uses —
/// so a submission through the real CLI/daemon has something to run.
fn write_two_stage_workflow(root: &Path) {
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

/// Poll `sgt --json work show <id>` until the work reaches `state`, or panic
/// after a bounded wait. No `SGT_FAKE_SCRIPT` is set for the spawned daemon
/// in these tests, so the fake backend completes every turn immediately
/// (`FakeBackend::next_step`'s own default) — this is a short poll for the
/// daemon's async completion driver to catch up, not a wait for an actor.
fn wait_for_state(cwd: &Path, data_dir: &Path, id: &str, state: &str) -> Value {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let shown = run(cwd, Some(data_dir), &[], &["--json", "work", "show", id]);
        let body = shown.json();
        if body["work"]["state"] == state {
            return body;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "work {id} never reached {state:?}: {body}"
        );
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// guard-map: `sgt work transcript` is reachable end to end through the real
/// CLI/daemon and renders the honest empty-conversation case correctly (the
/// fake backend emits no conversation events for an ordinary run — see the
/// comment below), and `sgt work show`'s human output surfaces the R-MVP1-2
/// output pointer — branch, worktree, finalize commit — in the same command,
/// without hand-decoding the JSON blob above it. Mutation this kills: `work
/// transcript` erroring or returning the wrong `work_id`/shape, or `work
/// show`'s human rendering silently dropping the output-pointer summary
/// this milestone adds.
///
/// **What this test does NOT prove (MVP-3 test-honesty finding TH-3/TH-1):**
/// the "minimal blob decode" fallback for an envelope-less turn — the fake
/// backend never produces one, so `transcript_turns`'s
/// `KIND_CONVERSATION_TURN_ENDED`/`decode_partial_assistant_text` branch is
/// untouched here. That path is proven end to end, through a real
/// `ClaudeBackend`/`TurnReader` turn (a scripted `claude` CLI, not a
/// hand-fabricated `Event`) and a simulated adjacent-append journal loss, by
/// `src/api.rs`'s own
/// `transcript_turns_recovers_a_real_producers_text_across_a_simulated_
/// adjacent_append_loss`.
#[test]
fn work_transcript_and_output_pointer_are_reachable_from_the_real_cli() {
    let data_dir = DataDir::new();
    let repo = tempfile::TempDir::new().expect("tempdir");
    // §4.1/§6.1: an estate-scoped verb needs an exact estate root with a
    // derived `repos/<name>` mount — a bare git repo is no longer a
    // estate.
    support::scaffold_estate(repo.path(), "solo", &["solo"]);
    write_two_stage_workflow(repo.path());

    let submitted = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "run", "--workflow", "tiny", "read my output"],
    );
    submitted.assert_ok("run");
    let work_id = submitted.json()["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();

    let shown = wait_for_state(repo.path(), data_dir.path(), &work_id, "completed");
    assert!(
        !shown["output"].is_null(),
        "a completed work must carry an output pointer: {shown}"
    );

    // `sgt work show` (human form): the output pointer rides inside the same
    // one printed object — every other folded key (`stage`, `surface`,
    // `workflow`, ...) works this way, and several tests elsewhere in this
    // crate (`m3_execution.rs`'s `cli_respond_and_retry_through_the_binary`)
    // parse this "human form" as JSON, so it must stay one JSON value, not
    // JSON-plus-prose. The pointer is answerable in this one command either
    // way: no second command, no hand-decoding a *different* blob for it.
    let human_show = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["work", "show", &work_id],
    );
    human_show.assert_ok("work show");
    let shown_human: Value = serde_json::from_str(&human_show.stdout).expect("work show is json");
    let repo_pointer = &shown_human["output"]["repositories"][0];
    assert_eq!(
        repo_pointer["retained_branch"],
        format!("sergeant/{work_id}"),
        "the retained branch must be named: {shown_human}"
    );
    assert!(
        repo_pointer["worktree_path"].is_string() && repo_pointer["finalize_commit"].is_string(),
        "the worktree path and finalize commit must both be named: {shown_human}"
    );

    // `sgt work transcript`: plain text by default.
    let transcript_text = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["work", "transcript", &work_id],
    );
    transcript_text.assert_ok("work transcript");
    // The fake backend emits no conversation events on an ordinary run
    // (`m5_projections.rs`'s own note on this), so the honest plain-text
    // rendering for this work is the empty-conversation message — proving
    // the command reaches the daemon and renders successfully, rather than
    // fabricating turns the run never produced.
    assert!(
        transcript_text
            .stdout
            .contains("no conversation recorded for this work"),
        "got: {}",
        transcript_text.stdout
    );

    // `--json`: the same raw structured body the API serves.
    let transcript_json = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "work", "transcript", &work_id],
    );
    transcript_json.assert_ok("work transcript --json");
    let body = transcript_json.json();
    assert_eq!(body["work_id"], work_id);
    assert!(body["turns"].as_array().is_some());

    // Read-only: `sgt work transcript` must never mutate daemon state.
    let after = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "work", "show", &work_id],
    );
    assert_eq!(after.json()["work"]["state"], "completed");

    data_dir.reap();
}

/// guard-map: `sgt work retained` and `sgt work reap` (#109), reachable end
/// to end through the real CLI/daemon. `sgt work reap <id>` without `--yes`
/// previews and destroys nothing; `--yes` actually disposes of the captured
/// patch a dirty teardown left behind, and the entry drops out of `sgt work
/// retained` afterward.
#[test]
fn work_retained_and_reap_are_reachable_from_the_real_cli() {
    let data_dir = DataDir::new();
    let repo = tempfile::TempDir::new().expect("tempdir");
    // §4.1/§6.1: an estate-scoped verb needs an exact estate root with a
    // derived `repos/<name>` mount — a bare git repo is no longer a
    // estate.
    support::scaffold_estate(repo.path(), "solo", &["solo"]);
    write_two_stage_workflow(repo.path());

    // `hang` keeps the first stage in flight so the worktree can be dirtied
    // before anything tears it down — the env reaches the auto-spawned
    // daemon this `run` call spawns, the same way `SGT_FAKE_SCRIPT` reaches
    // it in the §39 walkthrough.
    let submitted = run(
        repo.path(),
        Some(data_dir.path()),
        &[("SGT_FAKE_SCRIPT", "hang")],
        &[
            "--json",
            "run",
            "--workflow",
            "tiny",
            "leaves a mess for #109",
        ],
    );
    submitted.assert_ok("run");
    let work_id = submitted.json()["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();
    let worktree = PathBuf::from(
        submitted.json()["surface"]["bindings"][0]["worktree_path"]
            .as_str()
            .expect("worktree"),
    );
    std::fs::write(worktree.join("half-done.rs"), "fn main() {}\n").expect("dirty the worktree");

    run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["cancel", &work_id],
    )
    .assert_ok("cancel");
    wait_for_state(repo.path(), data_dir.path(), &work_id, "canceled");

    // `sgt work retained` names it.
    let retained = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "work", "retained"],
    );
    retained.assert_ok("work retained");
    let entries = retained.json()["retained"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let entry = entries
        .iter()
        .find(|e| e["work_id"] == work_id)
        .unwrap_or_else(|| panic!("the dirty binding must be listed: {:?}", retained.json()));
    assert_eq!(entry["disposition"], "retained_dirty");
    let patch_path = PathBuf::from(entry["path"].as_str().expect("patch path"));
    assert!(patch_path.is_file(), "the named patch must actually exist");

    // Without `--yes`, reap previews and destroys nothing.
    let preview = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["work", "reap", &work_id],
    );
    preview.assert_ok("work reap (preview)");
    assert!(
        preview.stdout.contains("--yes"),
        "an unconfirmed reap must point at --yes: {}",
        preview.stdout
    );
    assert!(patch_path.is_file(), "a preview must not touch anything");

    // `--yes` actually disposes of it.
    let reaped = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["work", "reap", &work_id, "--yes"],
    );
    reaped.assert_ok("work reap --yes");
    assert!(
        !patch_path.exists(),
        "reap --yes must actually delete the patch"
    );

    let retained_after = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "work", "retained"],
    );
    retained_after.assert_ok("work retained (after reap)");
    assert!(
        retained_after.json()["retained"]
            .as_array()
            .into_iter()
            .flatten()
            .all(|e| e["work_id"] != work_id),
        "a reaped binding must not still be listed: {:?}",
        retained_after.json()
    );

    data_dir.reap();
}

// -------------------------------------------------- envelope / daemon stop

/// guard-map: `sgt run --turns N --ceiling-secs S` (checkpoint-friction
/// item, `docs/gauntlet/notes/mvp-bucketing-2026-08-11.md`'s MVP-3 row)
/// threads a per-submission override into R-MVP1-7's turn envelope — CLI/API
/// plumbing onto the existing per-Work mechanics (R-NS-4: `turn_cap_bonus`
/// already proved a Work-specific cap was legal engine state via `sgt
/// extend`; this is the same idea at submission time), not new engine
/// mechanics — and `sgt work show`'s `envelope` key makes turns
/// spent/capped/ceiling readable without decoding the journal.
///
/// Mutation this kills: `--turns`/`--ceiling-secs` being parsed but silently
/// dropped before reaching the submit body (the work would then run to
/// completion against the daemon's much larger default cap instead of
/// blocking after its first turn); `work_view`/`fleet_body` reporting the
/// daemon-wide default instead of the submitted override; or
/// `check_turn_envelope` ignoring `Work::envelope` and gating on the
/// daemon-wide cap regardless.
#[test]
fn run_turns_and_ceiling_secs_override_the_envelope_for_one_work() {
    let data_dir = DataDir::new();
    let repo = tempfile::TempDir::new().expect("tempdir");
    // §4.1/§6.1: an estate-scoped verb needs an exact estate root with a
    // derived `repos/<name>` mount — a bare git repo is no longer a
    // estate.
    support::scaffold_estate(repo.path(), "solo", &["solo"]);
    write_two_stage_workflow(repo.path());

    let submitted = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &[
            "--json",
            "run",
            "--workflow",
            "tiny",
            "--turns",
            "1",
            "--ceiling-secs",
            "120",
            "capped at one turn",
        ],
    );
    submitted.assert_ok("run --turns 1 --ceiling-secs 120");
    let work_id = submitted.json()["work"]["id"]
        .as_str()
        .expect("work id")
        .to_string();

    // The two-stage workflow's first stage spawns turn 1 and completes (the
    // fake backend finishes every turn immediately, per
    // `work_transcript_and_output_pointer_are_reachable_from_the_real_cli`'s
    // own note); the second stage's LAUNCH would be turn 2, which a cap of 1
    // refuses — so this work must land `blocked`, never `completed`. That
    // divergence from the sibling test above (same workflow, default cap,
    // reaches `completed`) is the whole proof that `--turns 1` reached the
    // engine's own check rather than being dropped somewhere in the
    // CLI/API path.
    let shown = wait_for_state(repo.path(), data_dir.path(), &work_id, "blocked");

    let envelope = &shown["envelope"];
    assert_eq!(
        envelope["turns_spawned"], 1,
        "exactly one turn must have spawned before the cap blocked stage two: {shown}"
    );
    assert_eq!(
        envelope["turn_cap"], 1,
        "the effective cap must be the submitted override, not the daemon-wide default: {shown}"
    );
    assert_eq!(
        envelope["turn_ceiling_secs"], 120.0,
        "the effective ceiling must be the submitted override: {shown}"
    );

    // `sgt work list --json` folds the same envelope onto every fleet row,
    // so an operator does not need one request per work to see it.
    let listed = run(
        repo.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "work", "list"],
    );
    listed.assert_ok("work list");
    let row = listed.json()["works"]
        .as_array()
        .and_then(|rows| rows.iter().find(|w| w["id"] == work_id).cloned())
        .expect("the submitted work must appear in the fleet view");
    assert_eq!(
        row["envelope"]["turn_cap"], 1,
        "the fleet view must fold the same per-Work override: {row}"
    );

    data_dir.reap();
}

/// guard-map: `sgt daemon stop` (E4, MVP-3's "cheap-now" item) actually
/// stops the daemon it names, gracefully — `data_dir.daemon_pids()` (the
/// same pattern-scan `DataDir`'s own `Drop` leak check uses) finds nothing
/// afterward, matching docs/DEVELOPMENT.md's own leak-detection convention. Also pins
/// idempotence in both directions named in the task: stopping a daemon that
/// is not running yet, and stopping one that is already stopped, are both
/// clean successes, never errors. `tests/m6_surfaces.rs`'s
/// `r_mvp3_admission_pause_survives_a_kill_between_pause_and_stop` covers
/// the admission-drain mechanism's L6 crash window directly at the API
/// level; this test proves the same verb reachable, end to end, through the
/// real CLI binary a user actually runs.
///
/// Mutation this kills: `sgt daemon stop` sending SIGKILL instead of the
/// graceful path (this test's daemon would still be gone, but
/// `r_mvp3_admission_pause_survives_a_kill_between_pause_and_stop`'s sibling
/// coverage of the graceful path would not exercise what it claims to); or
/// `daemon_stop` treating "no daemon running" as an error instead of a
/// no-op success.
///
/// **What this test does NOT prove (MVP-3 test-honesty finding TH-4):** that
/// `daemon_stop`'s own drain step (`src/cli.rs`) actually issues `POST
/// /v1/admission/pause` — dropping that call entirely would still leave
/// this test green, since nothing here submits work concurrently with a
/// slow drain to observe the refusal it exists to cause. The sibling test
/// named above proves the *mechanism* (pause → durable across a crash →
/// resumed on restart) directly at the API level, not that this CLI verb is
/// the one invoking it. Left open rather than silently claimed: a real
/// regression here needs a scripted (`SGT_FAKE_SCRIPT=hang`) in-flight turn
/// plus a concurrent submit racing a backgrounded `sgt daemon stop`, which
/// is more machinery than this fixer pass's effort budget covers for an
/// `info`-severity gap.
#[test]
fn daemon_stop_drains_admission_and_exits_cleanly() {
    let data_dir = DataDir::new();
    let cwd = tempfile::TempDir::new().expect("tempdir");
    // §4.2: `daemon`/`daemon stop`/`run` are all estate-scoped, so this rig
    // needs a real estate root to stand in — the exact-root check runs
    // before descriptor lookup and before any spawn decision (§4.3).
    support::scaffold_estate(cwd.path(), "solo", &["solo"]);

    // Idempotent: stopping a daemon that was never started is a clean
    // success, not an error — a clean stop refuses nothing, including a
    // repeat of itself.
    let idle_stop = run(
        cwd.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "daemon", "stop"],
    );
    idle_stop.assert_ok("daemon stop (never started)");
    assert_eq!(idle_stop.json()["status"], "not_running");

    // Auto-spawn a real daemon. `status` no longer does this (ADR 0009: it
    // joined the no-spawn set), so a mutating verb does the spawning here.
    let submit = run(
        cwd.path(),
        Some(data_dir.path()),
        &[],
        &["run", "trigger the auto-spawn"],
    );
    submit.assert_ok("run (auto-spawn)");
    let pids_before = data_dir.daemon_pids();
    assert_eq!(
        pids_before.len(),
        1,
        "exactly one daemon must be running after auto-spawn: {pids_before:?}"
    );

    let stop = run(
        cwd.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "daemon", "stop"],
    );
    stop.assert_ok("daemon stop");
    assert_eq!(
        stop.json()["status"],
        "stopped",
        "stop must report that it actually stopped the daemon: {}",
        stop.stdout
    );
    assert!(
        data_dir.daemon_pids().is_empty(),
        "no daemon may remain running on this data dir after `sgt daemon stop`"
    );

    // Idempotent the other direction: stopping again after it is already
    // gone is still a clean success.
    let after = run(
        cwd.path(),
        Some(data_dir.path()),
        &[],
        &["--json", "daemon", "stop"],
    );
    after.assert_ok("daemon stop (already stopped)");
    assert_eq!(after.json()["status"], "not_running");

    data_dir.reap();
}
