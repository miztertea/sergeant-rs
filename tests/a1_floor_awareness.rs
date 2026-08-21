//! W3 §11.3: the standing enforcement for A1 (a pruned journal must never
//! be misreported as corruption).
//!
//! `Journal::replay()`/`Journal::replay_after(seq)` are the *strict*
//! primitives: they expect the replay to start at seq 1 (or exactly
//! `seq + 1`), and report anything else — including the ordinary, healthy
//! shape a real prune leaves behind (a floor above 1) — as
//! `JournalError::SeqDiscontinuity`. `Journal::replay_from_floor()` /
//! `Journal::replay_data_dir_from_floor()` are the floor-aware siblings
//! W3 added specifically so a pruned journal replays as the ordinary,
//! healthy outcome it is (I-W3-10).
//!
//! A manual audit today finds exactly two production call sites of the
//! strict primitives, both legitimately bounded rather than naively
//! strict from seq 1:
//! - `runtime::startup::Plan::replay` — the shared pass's own dispatcher,
//!   which calls `replay_after` only on the cache-hit (`Windowed`) arm and
//!   `replay_from_floor` on the cache-miss (`Full`) arm, so the strict
//!   primitive there is never reached with a floor above 1 underneath it;
//! - `api::Core::events_after` — always bounded to a seq the caller
//!   already knows is at or above the floor (the prune engine's own
//!   guard-held top-up, `/v1/events`, the SSE resume).
//!
//! That audit holds only until the next PR. This is the standing scan that
//! holds it after that: any *other* production call site of the strict
//! primitives — a new handler added later, a helper that reaches for the
//! familiar `.replay()` instead of the floor-aware sibling — fails here
//! with the call site printed, rather than silently reintroducing "pruned
//! journal reported as corruption" the next time this build actually
//! prunes something. Modeled on the same structural-scan idiom
//! `tests/m9_watch.rs` and `tests/a4_blob_ref_pinning.rs` already use.

use std::path::{Path, PathBuf};

/// The only sanctioned production call sites of the strict replay
/// primitives, named by the function that legitimately calls them —
/// `(file relative to src/, enclosing fn signature fragment)`.
const SANCTIONED_STRICT_REPLAY_SITES: &[(&str, &str)] = &[
    (
        "runtime/startup.rs",
        "pub fn replay(&self, journal: &Journal)",
    ),
    ("api.rs", "pub fn events_after(&self, after: u64)"),
];

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

/// Every byte range covered by a `#[cfg(test)]`-attributed item (a
/// function or a module) — computed per-occurrence, not "everything after
/// the first marker", since a file can freely mix a small `#[cfg(test)]`
/// helper with real production code that follows it before the real test
/// module begins (this codebase's `src/api.rs` does exactly that).
fn test_only_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    for (attr_start, _) in source.match_indices("#[cfg(test)]") {
        // The block belonging to the very next `fn` or `mod` item after the
        // attribute (skipping any other attributes / doc comments / blank
        // lines in between).
        let after = &source[attr_start..];
        let Some(rel_open) = after.find('{') else {
            continue;
        };
        let block_start = attr_start + rel_open;
        ranges.push((attr_start, block_end(source, block_start)));
    }
    ranges
}

fn in_any_range(index: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(start, end)| index >= *start && index < *end)
}

/// A `#[cfg(test)]` item can itself be *inside* another `#[cfg(test)]`
/// item's range (a `mod tests { ... #[test] fn foo() ... }`) — dedupe by
/// merging any range fully nested inside another before use, purely so the
/// reported "sanctioned site" search below does not get confused; it does
/// not change which bytes count as test-only either way.
fn merged(mut ranges: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    ranges.sort_unstable();
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        if let Some(last) = out.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        out.push((start, end));
    }
    out
}

#[test]
fn a_new_strict_replay_site_outside_the_sanctioned_list_fails_this_scan() {
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let journal_rs = src_root.join("runtime").join("journal.rs");

    // Resolve each sanctioned site to a concrete byte range, once, so the
    // scan below can ask "is this occurrence inside a sanctioned site" in
    // addition to "is this occurrence in test-only code".
    let sanctioned_ranges: Vec<(PathBuf, usize, usize)> = SANCTIONED_STRICT_REPLAY_SITES
        .iter()
        .map(|(rel, signature)| {
            let path = src_root.join(rel);
            let source =
                std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
            let start = source.find(signature).unwrap_or_else(|| {
                panic!(
                    "sanctioned site signature {signature:?} not found in {path:?} — it moved \
                     or was renamed; update SANCTIONED_STRICT_REPLAY_SITES to match, after \
                     confirming the call there is still legitimately bounded"
                )
            });
            (path, start, block_end(&source, start))
        })
        .collect();

    let mut checked_a_file = false;
    for path in all_src_files() {
        if path == journal_rs {
            // The primitives' own definition site — exempt by construction,
            // the same way A4's blob-ref scan exempts `blob.rs` itself.
            continue;
        }
        checked_a_file = true;
        let source =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        let excluded = merged(test_only_ranges(&source));

        for pattern in ["journal.replay(", "journal.replay_after("] {
            for (index, _) in source.match_indices(pattern) {
                if in_any_range(index, &excluded) {
                    continue; // test-only code — not production
                }
                let sanctioned = sanctioned_ranges
                    .iter()
                    .any(|(sanctioned_path, start, end)| {
                        sanctioned_path == &path && index >= *start && index < *end
                    });
                assert!(
                    sanctioned,
                    "{}:{} calls the strict replay primitive `{}` outside every sanctioned \
                     site — a pruned journal (a floor above 1) will be misreported as \
                     `SeqDiscontinuity` here instead of the healthy outcome it is (I-W3-10). \
                     Either route this through `replay_from_floor`/`replay_data_dir_from_floor`, \
                     or — if this call site is genuinely bounded to a seq already known to be \
                     at or above the floor — add it to SANCTIONED_STRICT_REPLAY_SITES with the \
                     reasoning for why. Context: {:?}",
                    path.display(),
                    index,
                    pattern.trim_end_matches('('),
                    &source[index.saturating_sub(160)..(index + 40).min(source.len())]
                );
            }
        }
    }
    assert!(checked_a_file, "the src/ walk must actually visit files");
}

/// The scan above is only meaningful if it actually walks the files where
/// the sanctioned sites live — this pins that down directly, independent
/// of the main scan's own bookkeeping.
#[test]
fn the_scan_actually_covers_both_sanctioned_files() {
    let files = all_src_files();
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    for (rel, _) in SANCTIONED_STRICT_REPLAY_SITES {
        let expected: PathBuf = src_root.join(rel);
        assert!(
            files.contains(&expected),
            "expected {expected:?} to be found by all_src_files()"
        );
    }
}

/// Guards the guard: `test_only_ranges` must not accidentally swallow real
/// production code that follows a small `#[cfg(test)]`-attributed helper —
/// exactly the shape `src/api.rs` has (`decode_partial_assistant_text`
/// above its real `mod tests`). A naive "everything after the first
/// `#[cfg(test)]` marker" scan would wrongly treat the whole rest of the
/// file as test-only and silently stop checking it.
#[test]
fn test_only_ranges_does_not_swallow_production_code_between_two_cfg_test_items() {
    let source = r#"
#[cfg(test)]
fn helper() {
    1
}

pub fn production_call_site() {
    thing.replay_after(1);
}

#[cfg(test)]
mod tests {
    fn another() {
        thing.replay();
    }
}
"#;
    let ranges = merged(test_only_ranges(source));
    let production_index = source.find("production_call_site").expect("marker");
    assert!(
        !in_any_range(production_index, &ranges),
        "production code between two #[cfg(test)] items must not be treated as test-only"
    );
    let first_helper_index = source.find("fn helper").expect("marker");
    assert!(
        in_any_range(first_helper_index, &ranges),
        "the first #[cfg(test)] fn's own body must be treated as test-only"
    );
    let mod_tests_index = source.rfind("fn another").expect("marker");
    assert!(
        in_any_range(mod_tests_index, &ranges),
        "the real test module's body must be treated as test-only"
    );
}

/// Sanity check on the block-matching helper itself, since both scans
/// above depend on it agreeing with the real language's brace nesting
/// (strings/comments aside — this codebase's other structural scans share
/// the same, documented limitation).
#[test]
fn block_end_finds_the_matching_close_brace() {
    let source = "fn f() { if true { 1 } else { 2 } }\nfn g() {}";
    let start = source.find("fn f").unwrap();
    let end = block_end(source, start);
    assert_eq!(&source[start..=end], "fn f() { if true { 1 } else { 2 } }");
}

/// Not a scan — a reachability sanity check, so this file cannot silently
/// stop meaning anything if someone renames or removes the module it is
/// about.
#[test]
fn journal_rs_still_exists_at_the_expected_path() {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("runtime")
        .join("journal.rs");
    assert!(path.exists(), "expected {path:?} to exist");
}
