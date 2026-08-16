//! Advisory-locking reliability (#85, ADR 0003 D6).
//!
//! `sgt init` and daemon start both rest on exactly one process holding
//! `daemon.lock` (`docs/DEVELOPMENT.md`'s "One owner" invariant;
//! `src/runtime/fsutil.rs::take_exclusive_lock`) — and that lock itself is
//! already cross-platform for free, because
//! [`std::fs::File::try_lock`] compiles to `flock` on Unix and `LockFileEx`
//! on Windows. What #85 asks for is not a new locking mechanism; it is a way
//! to tell, *before* trusting that lock, whether the filesystem underneath it
//! actually honors advisory locks at all. WSL2's `drvfs` (an estate cloned
//! under `/mnt/c/...`), NFS, and SMB all answer `try_lock` with success while
//! not actually excluding a second holder — the predicate this module
//! implements is deliberately "advisory locking unreliable on this
//! filesystem type", not "is this `/mnt/c`", so the same check covers all
//! three (ADR 0003 D6).
//!
//! Shaped like every other fact in this boundary (ADR 0002 D3): [`classify`]
//! and [`parse_proc_mounts`] are decision-logic functions exercised by the
//! tests below on any host, including a bare Linux box that has never seen a
//! `drvfs`/NFS/SMB mount — only the actual `/proc/mounts` read is
//! `#[cfg]`-gated. They are gated `cfg(any(test, target_os = "linux"))`
//! rather than left fully unconditional, the same fix `process.rs`'s
//! `parse_ps_output` applies and for the same reason: on Linux they have a
//! live production caller (`raw_detect` below) so they're never dead there,
//! but on macOS and every other non-Linux target they have no production
//! caller at all — only `cargo test` exercises them — and an unconditional
//! declaration trips `cargo clippy --all-targets -- -D warnings`'s
//! `dead_code` lint on that lib-only compilation (measured on this repo's
//! first macOS host, 2026-08-15; #85's arm was the one file in this module
//! family that hadn't yet applied `process.rs`'s precedent). macOS has no
//! `/proc` (D5's own liveness gap notes this), and what it should use
//! instead — `statfs`, `getmntinfo`, `diskutil` — is explicitly unmeasured
//! (ADR 0003's open question): rather than guess at an implementation
//! nobody has run, the macOS arm always reports [`Reliability::Unknown`],
//! which is the fail-closed-*safe* direction the asymmetry below calls for
//! (an inconclusive probe must never refuse). It closes only once someone
//! measures a real detection path on a macOS host.
//!
//! Ponytail rung **R2**: this is a new file, but it reuses the platform
//! boundary's already-established shape verbatim — the same `#[cfg]`-selected
//! module plus injected-probe pattern `data_dir.rs`, `disk.rs`, and
//! `process.rs` already use — rather than inventing a new mechanism, and it
//! adds no new dependency (`detect_for_path` is called by `daemon.rs` and
//! `cli.rs`'s doctor row; both are also R2 for the same reason).

use std::path::{Path, PathBuf};

/// The verdict [`detect_for_path`] reaches about one path's filesystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reliability {
    /// Advisory locking on this filesystem is expected to behave normally.
    Reliable,
    /// A specific, named filesystem type known to make advisory locking
    /// unreliable — `sgt init` and daemon start must refuse (ADR 0003 D6).
    Unreliable {
        /// The filesystem type as the mount table names it (e.g. `"drvfs"`,
        /// `"nfs4"`, `"cifs"`).
        filesystem: String,
    },
    /// Could not be determined on this platform or in this environment.
    /// **Never a refusal** — only a confirmed-bad filesystem refuses; an
    /// inconclusive probe must not brick the very platform whose detection
    /// is unmeasured (ADR 0003's stated open question).
    Unknown {
        /// What could not be determined, and why.
        reason: String,
    },
}

/// Filesystem types (as `/proc/mounts`' third column names them) known to
/// make advisory locking unreliable. Not an exhaustive survey of every
/// network or 9p-family filesystem in existence — it is the originating
/// `drvfs` case plus ADR 0003's explicit NFS/SMB generalization, extended by
/// name as new cases are measured (ADR 0003's second open question).
#[cfg(any(test, target_os = "linux"))]
const UNRELIABLE_FS_TYPES: &[&str] = &[
    "drvfs", "9p", "nfs", "nfs2", "nfs3", "nfs4", "cifs", "smb3", "smbfs",
];

/// One `/proc/mounts` row this module cares about.
#[cfg(any(test, target_os = "linux"))]
struct MountEntry {
    mount_point: PathBuf,
    fs_type: String,
}

/// `/proc/mounts` escapes space, tab, newline and backslash in the mount
/// point field as three-digit octal (`\040` for a space); everything else
/// passes through unchanged. Necessary because a WSL2 drive letter mount or
/// an operator-chosen mount point can legitimately contain a space.
#[cfg(any(test, target_os = "linux"))]
fn unescape_octal(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\'
            && i + 3 < bytes.len()
            && bytes[i + 1..i + 4].iter().all(u8::is_ascii_digit)
            && let Ok(text) = std::str::from_utf8(&bytes[i + 1..i + 4])
            && let Ok(value) = u8::from_str_radix(text, 8)
        {
            out.push(value);
            i += 4;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Parse `/proc/mounts`' text shape: whitespace-separated columns, one mount
/// per line (`device mount_point fs_type options dump pass`). A line with
/// fewer than three columns is dropped rather than turned into a bogus
/// entry — the same "never guess" posture [`crate::platform::process`]'s
/// per-line parsing takes.
#[cfg(any(test, target_os = "linux"))]
fn parse_proc_mounts(content: &str) -> Vec<MountEntry> {
    content
        .lines()
        .filter_map(|line| {
            let mut columns = line.split_whitespace();
            let _device = columns.next()?;
            let mount_point = columns.next()?;
            let fs_type = columns.next()?;
            Some(MountEntry {
                mount_point: PathBuf::from(unescape_octal(mount_point)),
                fs_type: fs_type.to_string(),
            })
        })
        .collect()
}

/// The decision logic ADR 0002 D3 asks for: given an already-canonicalized
/// path and the mount table, find the mount that actually governs it — the
/// entry with the *longest* matching mount-point prefix, since a deeper
/// mount (e.g. `/mnt/c`) always shadows a shallower one (`/`) — and classify
/// by its filesystem type. No covering mount at all is [`Reliability::Unknown`],
/// never [`Reliability::Unreliable`]: absence of evidence must not read as
/// evidence of a bad filesystem.
///
/// Gating on `cfg(any(test, target_os = "linux"))` (rather than leaving this
/// and the parsing helpers above fully unconditional, as the module doc
/// comment's "plain, unconditionally-compiled" phrasing might suggest) is
/// the same fix `process.rs`'s `parse_ps_output` applies for the same
/// reason: on Linux this has a live production caller (`raw_detect` below),
/// so it's never dead there; on macOS and other non-Linux targets it has no
/// production caller at all, only the tests below — and `cargo clippy
/// --all-targets -D warnings` (`docs/DEVELOPMENT.md`'s gate) trips
/// `dead_code` on that lib-only compilation without this gate. Measured on
/// this host 2026-08-15 (macOS, Apple M3 Pro): un-gated, `cargo clippy
/// --all-targets -- -D warnings` fails; gated, it's clean.
#[cfg(any(test, target_os = "linux"))]
fn classify(path: &Path, mounts: &[MountEntry]) -> Reliability {
    let governing = mounts
        .iter()
        .filter(|entry| path.starts_with(&entry.mount_point))
        .max_by_key(|entry| entry.mount_point.as_os_str().len());
    match governing {
        Some(entry) if UNRELIABLE_FS_TYPES.contains(&entry.fs_type.as_str()) => {
            Reliability::Unreliable {
                filesystem: entry.fs_type.clone(),
            }
        }
        Some(_) => Reliability::Reliable,
        None => Reliability::Unknown {
            reason: format!(
                "no mount entry covers {} — the mount table could not be matched against it",
                path.display()
            ),
        },
    }
}

/// [`classify`] plus [`parse_proc_mounts`], with the mount-table text
/// supplied by `mounts_source` instead of a hardcoded `/proc/mounts` read —
/// the injected-probe seam (ADR 0002 D2) that lets the whole wiring from a
/// path down to a verdict be exercised in tests without a real
/// `drvfs`/NFS/SMB mount anywhere in the test sandbox. `raw_detect` below is
/// the only caller that supplies the real thing.
#[cfg(any(test, target_os = "linux"))]
fn raw_detect_from(path: &Path, mounts_source: impl FnOnce() -> Option<String>) -> Reliability {
    match mounts_source() {
        Some(content) => classify(path, &parse_proc_mounts(&content)),
        None => Reliability::Unknown {
            reason: "cannot read /proc/mounts".to_string(),
        },
    }
}

#[cfg(target_os = "linux")]
fn raw_detect(path: &Path) -> Reliability {
    raw_detect_from(path, || std::fs::read_to_string("/proc/mounts").ok())
}

/// **UNVERIFIED — unmeasured, not merely untested.** There is no macOS host
/// in this estate (`docs/environments/`) to measure `statfs`, `getmntinfo`,
/// or `diskutil info`'s output shape against, and ADR 0003 names this
/// exactly as an open question rather than assumed. Reporting
/// [`Reliability::Unknown`] unconditionally is the fail-closed-*safe*
/// direction: it produces a `sgt doctor` warn row (never a refusal) until
/// someone measures a real detection path on a real Mac and replaces this.
#[cfg(target_os = "macos")]
fn raw_detect(_path: &Path) -> Reliability {
    Reliability::Unknown {
        reason: "macOS advisory-locking-reliability detection is unmeasured (#85) — /proc/mounts \
                 is Linux-only, and what macOS should use instead (statfs, getmntinfo, or \
                 `diskutil info`) needs its own measurement pass on a real host before this can \
                 classify anything there"
            .to_string(),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn raw_detect(_path: &Path) -> Reliability {
    Reliability::Unknown {
        reason: "advisory-locking-reliability detection is not implemented on this platform"
            .to_string(),
    }
}

/// Walk up from `path` to the nearest ancestor that already exists — the
/// same technique `cli.rs`'s `data_dir_check` uses for #67, needed here
/// because `sgt init` calls this before the data dir itself has been
/// created.
fn nearest_existing_ancestor(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

/// Is `path`'s filesystem one where advisory locking (`flock`) is reliable?
/// Walks up to the nearest existing ancestor first (`path` itself may not
/// exist yet — `sgt init` calls this before scaffolding anything), then
/// canonicalizes it so a symlinked estate resolves to the mount it actually
/// lives on. Both `sgt init`'s refusal, `daemon::start_with`'s refusal, and
/// `sgt doctor`'s `filesystem` row all call this same entry point, so the
/// three surfaces can never disagree about what "unreliable" means.
pub fn detect_for_path(path: &Path) -> Reliability {
    let Some(existing) = nearest_existing_ancestor(path) else {
        return Reliability::Unknown {
            reason: format!("no existing ancestor found above {}", path.display()),
        };
    };
    match existing.canonicalize() {
        Ok(canonical) => raw_detect(&canonical),
        Err(e) => Reliability::Unknown {
            reason: format!("cannot canonicalize {}: {e}", existing.display()),
        },
    }
}

/// The shared remedy text for a confirmed-unreliable `filesystem` — used
/// verbatim by both `daemon::DaemonError::UnreliableFilesystem` and `sgt
/// doctor`'s `filesystem` row, so an operator sees the same instruction
/// however they hit this.
pub fn remedy(filesystem: &str) -> String {
    format!(
        "advisory locking (flock) is unreliable on {filesystem} filesystems — move the estate \
         (or point --data-dir / SGT_DATA_DIR) to a filesystem where it is reliable: a native \
         Linux filesystem rather than a WSL2 /mnt/c drvfs mount, and not an NFS or SMB share"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octal_escaped_spaces_in_mount_points_are_unescaped() {
        let mounts = parse_proc_mounts("dev /mnt/My\\040Drive ext4 rw 0 0\n");
        assert_eq!(mounts.len(), 1);
        assert_eq!(mounts[0].mount_point, PathBuf::from("/mnt/My Drive"));
    }

    #[test]
    fn a_line_with_too_few_columns_is_dropped_not_a_bogus_entry() {
        assert!(parse_proc_mounts("only-one-column\n").is_empty());
        assert!(parse_proc_mounts("dev /mnt\n").is_empty());
    }

    /// The originating WSL2 case: a deeper `drvfs` mount must win over the
    /// shallower `/` entry that would otherwise also match by prefix.
    #[test]
    fn the_longest_matching_mount_wins_over_the_root() {
        let mounts = parse_proc_mounts(
            "/dev/sda1 / ext4 rw 0 0\n\
             C: /mnt/c drvfs rw 0 0\n",
        );
        let verdict = classify(Path::new("/mnt/c/estate/data"), &mounts);
        assert_eq!(
            verdict,
            Reliability::Unreliable {
                filesystem: "drvfs".to_string()
            }
        );
    }

    #[test]
    fn nfs_and_cifs_are_unreliable_the_same_way_drvfs_is() {
        for fs_type in ["nfs", "nfs4", "cifs", "smbfs"] {
            let mounts = parse_proc_mounts(&format!("srv:/x /mnt/share {fs_type} rw 0 0\n"));
            let verdict = classify(Path::new("/mnt/share/estate"), &mounts);
            assert_eq!(
                verdict,
                Reliability::Unreliable {
                    filesystem: fs_type.to_string()
                },
                "fs_type {fs_type} must be classified unreliable"
            );
        }
    }

    #[test]
    fn an_ordinary_filesystem_is_reliable() {
        let mounts = parse_proc_mounts("/dev/sda1 / ext4 rw 0 0\n");
        assert_eq!(
            classify(Path::new("/home/user/estate"), &mounts),
            Reliability::Reliable
        );
    }

    #[test]
    fn no_covering_mount_is_unknown_not_unreliable() {
        let verdict = classify(Path::new("/nowhere/at/all"), &[]);
        assert!(
            matches!(verdict, Reliability::Unknown { .. }),
            "absence of a matching mount must not be read as evidence of a bad filesystem: {verdict:?}"
        );
    }

    /// The fail-closed asymmetry itself, pinned at the seam every caller
    /// actually goes through: an unreadable mount table is inconclusive, and
    /// inconclusive must never collapse into "unreliable, refuse".
    #[test]
    fn an_unreadable_mount_table_is_unknown_not_a_refusal() {
        let verdict = raw_detect_from(Path::new("/"), || None);
        assert!(
            matches!(verdict, Reliability::Unknown { .. }),
            "a probe that could not run must not be treated as a confirmed-bad filesystem: {verdict:?}"
        );
    }

    #[test]
    fn raw_detect_from_wires_a_real_unreliable_verdict_end_to_end() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let canonical = dir.path().canonicalize().expect("canonicalize");
        let mounts = format!("none {} 9p rw 0 0\n", canonical.display());
        let verdict = raw_detect_from(&canonical, || Some(mounts.clone()));
        assert_eq!(
            verdict,
            Reliability::Unreliable {
                filesystem: "9p".to_string()
            }
        );
    }

    #[test]
    fn detect_for_path_walks_up_to_an_existing_ancestor() {
        // `child/data` does not exist; only `dir` does. Reverting the
        // ancestor walk to require `path` itself to exist would make this
        // return Unknown instead of actually classifying `dir`'s real
        // filesystem.
        let dir = tempfile::TempDir::new().expect("tempdir");
        let not_yet_created = dir.path().join("child").join("data");
        let verdict = detect_for_path(&not_yet_created);
        // This host's real filesystem is whatever it is — the property this
        // test pins is that the walk reached *some* real, matched mount
        // rather than giving up as Unknown for the trivial reason that the
        // leaf path does not exist yet.
        assert!(
            !matches!(&verdict, Reliability::Unknown { reason } if reason.contains("no existing ancestor")),
            "must not give up before walking up to {}: {verdict:?}",
            dir.path().display()
        );
    }

    #[test]
    fn remedy_names_the_filesystem_and_where_to_move_the_estate() {
        let text = remedy("drvfs");
        assert!(text.contains("drvfs"));
        assert!(text.contains("--data-dir") || text.contains("SGT_DATA_DIR"));
    }
}
