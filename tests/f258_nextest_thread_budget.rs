//! #258 / F-IN-01 — the m7 thread-budget isolation contract is structural,
//! not prose.
//!
//! `.config/nextest.toml` schedules
//! `m7_docker_executor::large_captured_output_does_not_grow_this_process_
//! proportionally` alone via `threads-required = "num-test-threads"`,
//! deliberately instead of a `[test-groups]` entry (see that file's own
//! comment for why). Before this test, that contract was asserted only in a
//! TOML comment: nothing failed if the override were weakened, retargeted by
//! a typo, dropped, or converted to the `test-groups` form the comment warns
//! against — CI's bare `cargo nextest run --locked` only validates that the
//! file parses, not what it says. This test parses the checked-in config and
//! pins the override's filter and its `threads-required` value, so a drift
//! in either fails here instead of silently reintroducing the starvation
//! class #258 fixed.

use std::path::Path;

#[derive(serde::Deserialize)]
struct NextestConfig {
    profile: Profiles,
}

#[derive(serde::Deserialize)]
struct Profiles {
    default: DefaultProfile,
}

#[derive(serde::Deserialize)]
struct DefaultProfile {
    #[serde(default)]
    overrides: Vec<Override>,
}

#[derive(serde::Deserialize)]
struct Override {
    filter: String,
    #[serde(rename = "threads-required")]
    threads_required: Option<toml::Value>,
}

const EXPECTED_FILTER: &str = "binary_id(=sergeant-rs::m7_docker_executor) and test(=large_captured_output_does_not_grow_this_process_proportionally)";

#[test]
fn m7_heavy_test_is_still_scheduled_alone_via_threads_required() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join(".config/nextest.toml");
    let raw = std::fs::read_to_string(&config_path).unwrap_or_else(|e| {
        panic!(
            "expected {} to exist and be readable: {e}",
            config_path.display()
        )
    });

    let config: NextestConfig =
        toml::from_str(&raw).expect(".config/nextest.toml must parse as valid nextest config");

    let overrides = &config.profile.default.overrides;
    let heavy = overrides
        .iter()
        .find(|o| o.filter == EXPECTED_FILTER)
        .unwrap_or_else(|| {
            panic!(
                "expected a [[profile.default.overrides]] entry with filter = {EXPECTED_FILTER:?}; \
                 found filters: {:?}. This is the #258 isolation contract for \
                 m7_docker_executor::large_captured_output_does_not_grow_this_process_proportionally \
                 — it must keep targeting exactly this binary and test.",
                overrides.iter().map(|o| &o.filter).collect::<Vec<_>>()
            )
        });

    assert_eq!(
        heavy
            .threads_required
            .as_ref()
            .and_then(toml::Value::as_str),
        Some("num-test-threads"),
        "the #258 override must keep `threads-required = \"num-test-threads\"` \
         (reserving the whole thread budget so the heavy test runs with no \
         siblings) — not a numeric value, not absent, and deliberately not a \
         `[test-groups]` entry (a group's max-threads only caps concurrency \
         *within* the group, which does not address contention with \
         everything outside it)."
    );

    // Deliberately not a test-groups conversion: assert no [test-groups]
    // table exists in the raw document at all, since that would be exactly
    // the "simplification" the checked-in comment forbids.
    let doc: toml::Value =
        toml::from_str(&raw).expect("re-parsing as a generic Value must also succeed");
    assert!(
        doc.get("test-groups").is_none(),
        "found a [test-groups] table in .config/nextest.toml — the #258 \
         override must stay a profile override (threads-required), not be \
         converted into a test group, which cannot express contention with \
         tests outside the group."
    );
}

/// The other half of "structural, not prose": what this config does **not**
/// configure may not be cited elsewhere as if it did.
///
/// `tests/support/mod.rs::scan_to_completion` waits on a scan's own reported
/// completion and deliberately carries no time bound of its own (elapsed
/// time is not evidence about a product whose input size is the estate).
/// That is right, and it makes the question "then what ends a hang?" load
/// bearing. The helper answered it by naming this file. This file answers
/// nothing of the sort: it holds one `[[profile.default.overrides]]` entry
/// about thread scheduling, and no `slow-timeout`/`terminate-after`
/// anywhere, so a hanging test is warned about — `SLOW [>60.000s]`, again at
/// 120 s, at 180 s, forever — and never killed. `.github/workflows/ci.yml`'s
/// `test` job declares no `timeout-minutes` either (unlike `coverage.yml`
/// and `matrix.yml`), so the real bound is GitHub's 6-hour default
/// cancellation, which names no test.
///
/// Deliberately blunt: it trips on *any* mention of this config in the
/// shared support module, not on one exact sentence, because the failure
/// class is a reader trusting a bound that is not there — and a citation
/// reworded around an exact-phrase guard would be the same defect wearing
/// different words. Either configure the termination and the citation
/// becomes true, or do not point at this file.
#[test]
fn the_shared_test_support_module_may_not_cite_this_config_as_a_hang_bound_it_does_not_configure() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config_path = repo_root.join(".config/nextest.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read .config/nextest.toml");
    let doc: toml::Value = toml::from_str(&raw).expect(".config/nextest.toml must parse");

    fn configures_termination(value: &toml::Value) -> bool {
        match value {
            toml::Value::Table(table) => {
                table.contains_key("terminate-after") || table.values().any(configures_termination)
            }
            toml::Value::Array(items) => items.iter().any(configures_termination),
            _ => false,
        }
    }
    let terminates = configures_termination(&doc);

    let support_path = repo_root.join("tests/support/mod.rs");
    let support = std::fs::read_to_string(&support_path).expect("read tests/support/mod.rs");
    let cites = support.contains(".config/nextest.toml");

    assert!(
        !cites || terminates,
        "tests/support/mod.rs points at .config/nextest.toml, but that config sets no \
         `terminate-after` anywhere, so nothing there kills a hanging test — nextest only warns \
         (`SLOW [>60.000s]`, and again every 60 s, forever) and ci.yml's `test` job sets no \
         `timeout-minutes`. Either add the termination this citation claims, or stop citing this \
         file for it: an unbounded wait whose stated safety net does not exist is worse than one \
         that says plainly that nothing below the CI job ends it."
    );
}
