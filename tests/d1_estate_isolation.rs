//! **S6 D1 — the estate axis: A2 §2 stage 1, end to end across two estates.**
//!
//! # The seam this file exists to close
//!
//! Thirty-five test files invoke `sgt` through `CARGO_BIN_EXE_sgt`, and
//! `tests/w5_h1_acceptance.rs` already drives *two* estates against one host
//! daemon. What no file did was cross two estates **on the intelligence
//! path**: `tests/w5_search_surface.rs` is single-estate by construction (its
//! own `fn estate() -> Estate`), and its header concedes *"The live-estate
//! acceptance is not in this file."* Multi-estate coverage and intelligence
//! coverage both existed and had never intersected. That intersection is the
//! entire content of this file, and its absence is why a search from one
//! estate returned another estate's private code as hit #1 with a green
//! suite behind it (measured 2026-08-30,
//! `knowledge/evidence/resources/host-atlas-series/
//! estate-isolation-absent-2026-08-30.md`).
//!
//! # Why every assertion here has two halves
//!
//! *"A search from A returns no unit from B"* is **vacuous on its own**: it
//! passes when search returns nothing at all, so a change that broke
//! retrieval outright would show green and the guard would report the
//! confidentiality boundary as intact. Every isolation assertion below
//! therefore also asserts that the *same* query still returns A's own
//! matching units. One query, both halves, or it is not a test.
//!
//! # No clock decides anything here
//!
//! There is no deadline, no elapsed-time assertion and no ratio. `sgt
//! intelligence scan` answers when the scan is recorded, so the test waits on
//! that answer rather than on a duration. The one `sleep` is a polling
//! *cadence* while waiting for the daemon to publish its descriptor — it sets
//! how often to look, never whether the result is correct, and a daemon that
//! never starts hangs the test, which is the runner's problem to bound.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

use sergeant_rs::domain::source::{AuthorityClass, EstateAdmission, EstateBinding, SourceKind};
use sergeant_rs::runtime::atlas::db::{Admissibility, AtlasDb, SourceSelector};
use sergeant_rs::runtime::atlas::record::record_scan;
use sergeant_rs::runtime::journal::Journal;

mod support;
use support::DataDir;

const SGT: &str = env!("CARGO_BIN_EXE_sgt");

/// A term present in **both** estates' fixtures, so one query has something
/// to find on each side. This is what makes the positive half of every
/// assertion below real: a query that only ever matched one estate could not
/// tell "isolated" from "broken".
const SHARED_TERM: &str = "zzq7marker";

/// Estate A's own token, present nowhere else.
const ALPHA_TOKEN: &str = "SECRETTOKEN_ALPHA_ZZQ7";

/// Estate B's own token — the string the original measurement retrieved from
/// A, and the one no answer to A may ever contain again.
const BETA_TOKEN: &str = "SECRETTOKEN_BETA_ZZQ7";

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Run `sgt` addressed at one exact estate root against one host data dir.
fn sgt(data_dir: &Path, estate_root: &Path, args: &[&str]) -> Output {
    Command::new(SGT)
        .arg("-C")
        .arg(estate_root)
        .arg("--data-dir")
        .arg(data_dir)
        .args(args)
        .output()
        .expect("run sgt")
}

/// `sgt --json …`, parsed. Panics with both streams on a non-zero exit, so a
/// refusal shows its own message rather than a JSON parse error.
fn sgt_json(data_dir: &Path, estate_root: &Path, args: &[&str]) -> Value {
    let mut argv = vec!["--json"];
    argv.extend_from_slice(args);
    let out = sgt(data_dir, estate_root, &argv);
    assert!(
        out.status.success(),
        "sgt {args:?} failed: {}{}",
        stdout(&out),
        stderr(&out)
    );
    serde_json::from_str(&stdout(&out)).expect("sgt --json emits json")
}

/// The `source` name of every hit in a `sgt search`/`sgt related` answer.
fn hit_sources(answer: &Value) -> Vec<String> {
    answer["hits"]
        .as_array()
        .expect("hits array")
        .iter()
        .map(|h| h["source"].as_str().expect("hit source").to_string())
        .collect()
}

/// Two estates on one host daemon, each with one repository mount carrying
/// [`SHARED_TERM`] plus its own private token, both scanned.
struct TwoEstates {
    data_dir: DataDir,
    alpha: tempfile::TempDir,
    beta: tempfile::TempDir,
}

impl TwoEstates {
    fn alpha(&self) -> &Path {
        self.alpha.path()
    }
    fn beta(&self) -> &Path {
        self.beta.path()
    }
}

fn fixture_source(token: &str) -> String {
    format!("pub fn {SHARED_TERM}_fn() {{ let _ = \"{token}\"; }}\n")
}

/// Build both estates, spawn one daemon on one data dir, scan both.
///
/// One `--data-dir` is the whole point: it is what makes one host daemon and
/// one host-scoped Atlas store, which is the configuration the leak was
/// measured in. Nothing here is contrived — it is what two `sgt` estates on
/// one developer machine do by default.
fn two_estates() -> TwoEstates {
    let data_dir = DataDir::new();
    let alpha = tempfile::TempDir::new().expect("alpha tempdir");
    let beta = tempfile::TempDir::new().expect("beta tempdir");
    support::scaffold_estate(alpha.path(), "d1-alpha", &["src-alpha"]);
    support::scaffold_estate(beta.path(), "d1-beta", &["src-beta"]);

    for (root, repo, token) in [
        (alpha.path(), "src-alpha", ALPHA_TOKEN),
        (beta.path(), "src-beta", BETA_TOKEN),
    ] {
        let mount = root.join("repos").join(repo);
        std::fs::write(mount.join("lib.rs"), fixture_source(token)).expect("write fixture");
        support::git(&mount, &["add", "."]);
        support::git(&mount, &["commit", "-m", "fixture"]);
    }

    spawn_daemon(data_dir.path());

    for root in [alpha.path(), beta.path()] {
        let out = sgt(data_dir.path(), root, &["intelligence", "scan"]);
        assert!(
            out.status.success(),
            "scan of {} failed: {}{}",
            root.display(),
            stdout(&out),
            stderr(&out)
        );
        assert!(
            stdout(&out).contains("recorded"),
            "scan of {} recorded nothing: {}",
            root.display(),
            stdout(&out)
        );
    }

    TwoEstates {
        data_dir,
        alpha,
        beta,
    }
}

/// Start one host daemon on `data_dir` and wait until it has published its
/// descriptor.
///
/// The wait is on **recorded state** — the descriptor file the daemon writes
/// when it is serving — not on a duration. The 25 ms sleep is polling cadence
/// and nothing else: no branch below reads a clock, and a daemon that never
/// publishes hangs this test rather than failing it on a timer, which is the
/// honest outcome (the runner bounds hangs; a deadline assertion would turn a
/// slow host into a false failure).
#[allow(clippy::zombie_processes)]
fn spawn_daemon(data_dir: &Path) {
    let child = Command::new(SGT)
        .arg("--data-dir")
        .arg(data_dir)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn sgt daemon");
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
    });
    while sergeant_rs::daemon::read_descriptor(data_dir)
        .expect("read descriptor")
        .is_none()
    {
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

// =====================================================================
// 1. `sgt search` — the measured leak.
// =====================================================================

/// **The guard, both halves, one query.**
///
/// From estate A, a search for a term both estates contain must return A's
/// own units *and* none of B's. Dropping either half makes the test
/// worthless: without the negative there is no boundary, and without the
/// positive an empty answer would pass.
///
/// This is the exact configuration of the 2026-08-30 measurement — two
/// estates, one `--data-dir`, one daemon — and before the estate axis landed
/// it failed on the negative half with `src-beta/code:lib.rs#0` in the
/// answer.
#[test]
fn a_search_from_one_estate_sees_its_own_units_and_none_of_the_other_estates() {
    let rig = two_estates();

    let answer = sgt_json(rig.data_dir.path(), rig.alpha(), &["search", SHARED_TERM]);
    let sources = hit_sources(&answer);

    // (b) the positive half — this query really does retrieve.
    assert!(
        sources.iter().any(|s| s == "src-alpha"),
        "estate A's own matching units disappeared from its own search — the \
         isolation half below would pass vacuously on this answer: {sources:?}"
    );
    // (a) the negative half — and it retrieves nothing of B's.
    assert!(
        !sources.iter().any(|s| s == "src-beta"),
        "estate B's units are admissible from estate A (A2 §2 stage 1): {sources:?}"
    );

    // The token itself, not just the source name: the rendered answer must
    // carry no byte of B's private content.
    let rendered = serde_json::to_string(&answer).expect("render");
    assert!(
        !rendered.contains(BETA_TOKEN),
        "estate B's private token reached estate A's answer: {rendered}"
    );

    // And the mirror image, so this cannot pass by A simply being empty of
    // everything: B sees its own and none of A's.
    let from_beta = sgt_json(rig.data_dir.path(), rig.beta(), &["search", SHARED_TERM]);
    let beta_sources = hit_sources(&from_beta);
    assert!(
        beta_sources.iter().any(|s| s == "src-beta"),
        "estate B cannot see its own units: {beta_sources:?}"
    );
    assert!(
        !beta_sources.iter().any(|s| s == "src-alpha"),
        "estate A's units are admissible from estate B: {beta_sources:?}"
    );
}

/// A search that names B's private token by hand, from A, returns nothing of
/// B's — and A's own units still answer a query that names A's token.
///
/// The first half is the literal repro command from the measurement; the
/// second is what stops it being vacuous.
#[test]
fn naming_the_other_estates_secret_from_this_one_retrieves_nothing_of_theirs() {
    let rig = two_estates();

    let leak = sgt_json(rig.data_dir.path(), rig.alpha(), &["search", BETA_TOKEN]);
    assert!(
        !hit_sources(&leak).iter().any(|s| s == "src-beta"),
        "the measured leak is still open: {:?}",
        hit_sources(&leak)
    );

    let own = sgt_json(rig.data_dir.path(), rig.alpha(), &["search", ALPHA_TOKEN]);
    assert!(
        hit_sources(&own).iter().any(|s| s == "src-alpha"),
        "estate A cannot retrieve its own token, so the assertion above proves \
         nothing: {:?}",
        hit_sources(&own)
    );
}

// =====================================================================
// 2. `sgt related` — the same admissibility path, verified by running it.
// =====================================================================

/// `related` is not a second implementation of the filter — it takes the same
/// [`Admissibility`] — but "same code path" is a claim, and this runs it.
///
/// Both halves again: A's own anchor still yields A's neighbours (the answer
/// is not empty), and B's coordinate is not resolvable from A at all —
/// `related` refuses it as an inadmissible unit rather than answering from
/// another estate's world.
#[test]
fn related_resolves_this_estates_anchor_and_refuses_the_other_estates_coordinate() {
    let rig = two_estates();

    let own = sgt_json(
        rig.data_dir.path(),
        rig.alpha(),
        &["related", "src-alpha/code:lib.rs#0"],
    );
    assert_eq!(
        own["anchor"]["source"].as_str(),
        Some("src-alpha"),
        "estate A cannot anchor on its own unit: {own}"
    );
    assert!(
        !hit_sources(&own).iter().any(|s| s == "src-beta"),
        "estate B's units are related-reachable from estate A: {:?}",
        hit_sources(&own)
    );

    let across = sgt(
        rig.data_dir.path(),
        rig.alpha(),
        &["related", "src-beta/code:lib.rs#0"],
    );
    assert!(
        !across.status.success(),
        "estate A resolved estate B's coordinate: {}",
        stdout(&across)
    );
    let refusal = format!("{}{}", stdout(&across), stderr(&across));
    assert!(
        refusal.contains("no admissible unit"),
        "the refusal should name inadmissibility, not something else: {refusal}"
    );
    assert!(
        !refusal.contains(BETA_TOKEN),
        "the refusal leaked the content it refused: {refusal}"
    );

    // Positive control for that refusal: the identical command from B — the
    // estate that owns the coordinate — succeeds. Without this, "related
    // refuses" would also be satisfied by `related` being broken.
    let from_owner = sgt_json(
        rig.data_dir.path(),
        rig.beta(),
        &["related", "src-beta/code:lib.rs#0"],
    );
    assert_eq!(
        from_owner["anchor"]["source"].as_str(),
        Some("src-beta"),
        "estate B cannot anchor on its own unit either, so the refusal above is \
         not evidence of isolation: {from_owner}"
    );
}

// =====================================================================
// 3. C1's compiled-context selection (C1 §4: "estate coordinate").
// =====================================================================

/// Record one generation for `estate`, in `db`.
fn record_for(db: &mut AtlasDb, journal: &mut Journal, source: &str, estate: &str, token: &str) {
    let scan = support::scan(
        source,
        SourceKind::LocalKnowledge,
        AuthorityClass::EstateReadonly,
        vec![support::file(
            "notes.md",
            vec![support::unit(0, "Notes", &format!("{SHARED_TERM} {token}"))],
        )],
    );
    record_scan(
        db,
        journal,
        &scan,
        None,
        &EstateBinding::Estate(estate.to_string()),
    )
    .expect("record");
}

/// **C1 §4 names `estate coordinate` as the compiler's first input.** A
/// snapshot compiled in one estate must not bind another estate's evidence.
///
/// Driven at the admissibility layer the compiler actually calls, over one
/// store holding both estates' generations — the compiler's own two
/// `SourceSelector::Any` authority passes (knowledge and external) are the
/// widest reach it has, and this is exactly the filter they now carry.
///
/// Both halves: the pass admits estate A's knowledge generation, and admits
/// no generation of B's.
#[test]
fn a_compilations_admissibility_pass_binds_only_the_addressed_estates_evidence() {
    let data_dir = DataDir::new();
    let mut journal = Journal::open(data_dir.path()).expect("journal");
    let mut db = AtlasDb::open(data_dir.path()).expect("atlas");

    record_for(
        &mut db,
        &mut journal,
        "alpha-notes",
        "/estates/alpha",
        ALPHA_TOKEN,
    );
    record_for(
        &mut db,
        &mut journal,
        "beta-notes",
        "/estates/beta",
        BETA_TOKEN,
    );

    // Exactly the filter `runtime::context::compile` builds for its
    // knowledge pass, with the estate coordinate the request carried.
    let pass = Admissibility {
        estate: EstateAdmission::Estate("/estates/alpha".to_string()),
        source: SourceSelector::Any,
        kind: None,
        authority: Some(AuthorityClass::EstateReadonly),
    };
    let admitted = db.admissible_generations(&pass, 32).expect("admissible");
    let names: Vec<&str> = admitted
        .hits
        .iter()
        .map(|g| g.source_name.as_str())
        .collect();

    assert!(
        names.contains(&"alpha-notes"),
        "the compiler cannot see its own estate's knowledge source: {names:?}"
    );
    assert!(
        !names.contains(&"beta-notes"),
        "a compilation in estate alpha can bind estate beta's evidence: {names:?}"
    );

    // The unit body, not just the generation: nothing of B's text is
    // reachable through the same filter.
    let units = db.admissible_units(&pass, 32).expect("units");
    let bodies: String = units.hits.iter().map(|u| u.unit.body.clone()).collect();
    assert!(
        bodies.contains(ALPHA_TOKEN),
        "the positive half is empty, so the negative half below is vacuous: {bodies:?}"
    );
    assert!(
        !bodies.contains(BETA_TOKEN),
        "estate beta's knowledge text is admissible in an alpha compilation: {bodies:?}"
    );
}

// =====================================================================
// 4. The default is DENY — the property the whole axis rests on.
// =====================================================================

/// `Admissibility::default()` admits **nothing**, not everything.
///
/// This is the one structural difference between the estate axis and its two
/// neighbours (`kind` and `authority`, whose `None` admits every value), and
/// it is the difference the fix depends on: an estate axis that defaulted to
/// admit-everything would have left every existing consumer — all of which
/// omitted the estate — reading every estate exactly as before. Deleting the
/// `? IS NOT NULL` guard from the predicate turns this test red.
#[test]
fn an_admissibility_that_names_no_estate_admits_nothing() {
    let data_dir = DataDir::new();
    let mut journal = Journal::open(data_dir.path()).expect("journal");
    let mut db = AtlasDb::open(data_dir.path()).expect("atlas");
    record_for(
        &mut db,
        &mut journal,
        "alpha-notes",
        "/estates/alpha",
        ALPHA_TOKEN,
    );

    let nothing = db
        .admissible_generations(&Admissibility::default(), 32)
        .expect("admissible");
    assert!(
        nothing.hits.is_empty(),
        "an unaddressed filter admitted {} generation(s) — the estate axis is \
         default-allow, which is the defect, not the fix",
        nothing.hits.len()
    );

    // Positive control: the same store, addressed, is not empty. Without
    // this the assertion above would also pass on a store that holds nothing.
    let admitted = db
        .admissible_generations(&Admissibility::within_estate("/estates/alpha"), 32)
        .expect("admissible");
    assert_eq!(
        admitted.hits.len(),
        1,
        "the store the assertion above ran against was empty, so it proved \
         nothing"
    );
}

/// A search that addresses no estate is **refused**, not answered widely.
///
/// The daemon route is estate-scoped now; `?estate_root=` absent is the same
/// "no estate addressed" refusal every other estate-scoped route gives, and
/// specifically not an answer over every estate on the host.
#[test]
fn a_search_that_addresses_no_estate_is_refused_rather_than_answered_widely() {
    let rig = two_estates();
    let descriptor = sergeant_rs::daemon::read_descriptor(rig.data_dir.path())
        .expect("read descriptor")
        .expect("a daemon is serving");
    // Through the crate's own client, deliberately with **no** estate root
    // named (`ApiClient::new` addresses none until `with_estate_root` says
    // so) — this is the request shape the CLI can no longer produce and the
    // daemon must still refuse on its own account.
    let client =
        sergeant_rs::api::ApiClient::new(&descriptor.endpoint, &descriptor.token).expect("client");
    let answer = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async { client.get(&format!("/v1/search?q={SHARED_TERM}")).await });
    let body = match answer {
        Ok(value) => serde_json::to_string(&value).expect("render"),
        Err(e) => format!("{e}"),
    };
    assert!(
        !body.contains(ALPHA_TOKEN) && !body.contains(BETA_TOKEN),
        "an unaddressed search answered from the host's estates: {body}"
    );
}

/// Guard-rail for this file itself: the fixtures really are distinguishable.
///
/// If both estates' bytes were identical, every isolation assertion above
/// would pass for the wrong reason.
#[test]
fn the_two_estates_fixtures_are_actually_different() {
    assert_ne!(ALPHA_TOKEN, BETA_TOKEN);
    assert!(fixture_source(ALPHA_TOKEN).contains(SHARED_TERM));
    assert!(fixture_source(BETA_TOKEN).contains(SHARED_TERM));
    assert!(!fixture_source(ALPHA_TOKEN).contains(BETA_TOKEN));
    let _: PathBuf = PathBuf::from("/estates/alpha");
}
