//! S4 Y5 (G6) doctrine amendment — the FULL S2 shape, part (c): a
//! red-then-green test pinning the scoped meaning of AGENTS.md's "`sgt`
//! never fetches" sentence, ADR-first (ADR 0023).
//!
//! `AGENTS.md`'s CAN section states, as enforceable fact: "Admission
//! requires each selected mount's clean, attached HEAD and pins its exact
//! SHA; `sgt` never fetches, pulls, switches branches, or infers a remote
//! default to get there". That sentence is scoped to *admission* by its own
//! words, but a reader could still take "`sgt` never fetches" as an
//! absolute claim about the whole binary — which S4 Y5's external-Git
//! acquisition (`sgt intelligence add`) now falsifies on purpose, in a
//! different subsystem with no Work authority. ADR 0023 records the scoped
//! meaning; this file pins both halves of it:
//!
//! * the doctrine text itself names the ADR and the scope (a documentation
//!   check — this is what makes it "red" against the pre-amendment text and
//!   "green" against the amended one);
//! * the code boundary the doctrine describes actually holds: admission
//!   ([`sergeant_rs::runtime::surface`]) never references the external-git
//!   fetch surface, structurally, the same style
//!   `tests/x5_a1a_acceptance.rs`'s item 12 check already uses for a
//!   different boundary.
//!
//! **How this was actually proven red-then-green**, since a single committed
//! file cannot show its own history: before landing the AGENTS.md edit
//! alongside this test, the amended sentence and ADR citation were removed
//! (`git stash` on `AGENTS.md` alone) and this test's first assertion
//! failed exactly as expected — the pre-amendment text has no "ADR 0023" to
//! find. Restoring the edit turned it green. The commit this test lands in
//! carries both, never the test alone.

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The doctrine text: the admission bullet still says `sgt` never fetches
/// (unweakened — admission genuinely still never does), and now cites the
/// ADR that scopes what "never" ranges over.
#[test]
fn agents_md_scopes_the_never_fetches_sentence_to_admission_and_cites_the_adr() {
    let agents = read("AGENTS.md");
    let bullet_start = agents
        .find("Admission requires each selected mount's clean, attached HEAD")
        .expect("the admission bullet must still exist, unweakened");
    let window = &agents[bullet_start..(bullet_start + 900).min(agents.len())];
    assert!(
        window.contains("sgt` never fetches, pulls, switches branches"),
        "the original claim must survive verbatim — this is a scoping amendment, not a \
         retraction: {window}"
    );
    assert!(
        window.contains("ADR 0023"),
        "the scoped meaning must cite its ADR (S2's own shape: ADR first) — none found in: \
         {window}"
    );
    assert!(
        window.to_lowercase().contains("scoped to admission")
            || window.to_lowercase().contains("different subjects"),
        "the amendment must actually say what is scoped and why the external-git fetch is not \
         an exception to it: {window}"
    );
}

/// The code boundary the doctrine describes: admission/materialization
/// never names the external-git acquisition surface. Structural, matching
/// `tests/x5_a1a_acceptance.rs`'s own item-12 style — the claim worth
/// checking is that the call does not exist to be made, not that some
/// hypothetical caller behaves.
#[test]
fn admission_never_references_the_external_git_fetch_surface() {
    let surface = read("src/runtime/surface.rs");
    for forbidden in [
        "git_fetch_restricted",
        "external_git::",
        "acquire_and_scan",
        "acquire_external_git_on_lane",
    ] {
        assert!(
            !surface.contains(forbidden),
            "src/runtime/surface.rs (admission/materialization) must never reference `{forbidden}` \
             — external-git acquisition is a different subsystem with no Work authority"
        );
    }
}

/// The reverse check, so this file cannot pass by accident because the
/// symbols it forbids happen not to exist anywhere in the crate: the
/// external-git fetch surface really is defined, elsewhere, doing real work.
#[test]
fn the_external_git_fetch_surface_genuinely_exists_elsewhere() {
    let git_module = read("src/runtime/git.rs");
    assert!(
        git_module.contains("pub fn git_fetch_restricted"),
        "the function this test forbids admission from calling must be real"
    );
    let external_git = read("src/runtime/atlas/external_git.rs");
    assert!(
        external_git.contains("pub fn acquire_and_scan"),
        "the acquisition entry point must be real"
    );
}
