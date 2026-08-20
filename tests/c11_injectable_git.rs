//! C11 (estate-root Phase 0): [`GIT_BIN_ENV`] actually redirects every
//! invocation `runtime::git` makes, not merely a hardcoded `"git"` literal
//! somewhere upstream of it — the one fact §18's scripted-git admission tests
//! (no network, no branch switch, no fetch/pull/reset on any admission path)
//! all rest on.
//!
//! **This must stay the only test in this file.** `std::env::set_var` is
//! process-global, not thread-scoped, and `runtime::git::command` reads
//! `SGT_GIT_BIN` fresh on every invocation (deliberately — see its doc
//! comment). Set inside the lib test binary, the override is visible to every
//! other test thread for as long as it is set, and the ~60 call sites that
//! reach real Git through this module then run the stub instead: measured at
//! HEAD, that intermittently failed `failure_carries_gits_own_diagnostic`
//! three distinct ways — the stub's stdout arriving where Git's "not a git
//! repository" diagnostic was expected, `rev-parse` "succeeding" in a
//! non-repository because the stub always exits zero, and the override
//! outliving the `TempDir` holding the script.
//!
//! Cargo gives every integration-test file its own process, so a file holding
//! exactly one test has no sibling thread to leak into. A second test added
//! here reintroduces the race; give it its own file instead.

use sergeant_rs::runtime::git::{GIT_BIN_ENV, git};

#[test]
fn the_git_binary_is_overridable_via_env() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let script = dir.path().join("scripted-git");
    std::fs::write(&script, "#!/bin/sh\necho ran: \"$@\"\n").expect("write script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&script).expect("stat").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).expect("chmod");
    }

    // SAFETY (test-only): this binary runs exactly one test, so there is no
    // other thread in this process to observe the mutation — which is the
    // whole reason the test lives here rather than beside the module. Set
    // immediately before the one invocation that reads it and removed
    // immediately after, with no assertion — and so no possible panic — in
    // between, so the override never outlives the `TempDir` holding the
    // script it points at.
    unsafe { std::env::set_var(GIT_BIN_ENV, &script) };
    let out = git(dir.path(), &["status"]);
    unsafe { std::env::remove_var(GIT_BIN_ENV) };

    let out = out.expect("the scripted binary must run in place of git and exit zero");
    assert!(
        out.contains("--no-pager") && out.contains("status"),
        "expected the scripted binary to echo the real invocation, got {out:?}"
    );
}
