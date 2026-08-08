//! Small filesystem helpers shared by the event core.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Create `dir` and any missing ancestors, then fsync the parent of every
/// directory actually created, so the new dirents survive a crash — the same
/// crash-consistency recipe [`write_atomic`] applies to files, carried up the
/// ancestor chain. Without this, the first fsync-acknowledged write inside a
/// freshly created directory could vanish with the directory itself. When the
/// whole path already exists this issues no syncs at all.
pub(crate) fn create_dir_all_durable(dir: &Path) -> std::io::Result<()> {
    let mut created = Vec::new();
    let mut cur = dir;
    while !cur.exists() {
        created.push(cur.to_path_buf());
        match cur.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => cur = parent,
            _ => break,
        }
    }
    fs::create_dir_all(dir)?;
    for new_dir in &created {
        if let Some(parent) = new_dir.parent()
            && !parent.as_os_str().is_empty()
        {
            File::open(parent)?.sync_all()?;
        }
    }
    Ok(())
}

/// Write `bytes` to `path` atomically and durably: a fresh uniquely-named
/// tmp file next to `path` is written and fsynced, renamed over `path`, and
/// the containing directory is fsynced. A reader never observes a partial
/// file, and a crash leaves either the old content or the new — never a torn
/// mix. Replacement arrives as a new inode (rename), never as an in-place
/// truncate of the destination.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("tmp-{}", ulid::Ulid::generate()));
    let mut file = OpenOptions::new().create_new(true).write(true).open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_data()?;
    drop(file);
    fs::rename(&tmp, path)?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

/// [`write_atomic`], but the file is born owner-only (mode `0600` on Unix):
/// the tmp file is created with the restrictive mode *before* any bytes are
/// written, so no reader ever observes the content under wider permissions —
/// not even between write and a later chmod. Used for the runtime descriptor,
/// which carries the bearer token.
pub(crate) fn write_atomic_secret(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension(format!("tmp-{}", ulid::Ulid::generate()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp)?;
    file.write_all(bytes)?;
    file.sync_data()?;
    drop(file);
    fs::rename(&tmp, path)?;
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_dir_all_durable_creates_nested_dirs_and_is_idempotent() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let nested = dir.path().join("a").join("b").join("c");
        create_dir_all_durable(&nested).expect("create nested");
        assert!(nested.is_dir());
        // Idempotent on an existing path (the no-sync fast path).
        create_dir_all_durable(&nested).expect("recreate");
        assert!(nested.is_dir());
    }

    #[test]
    fn write_atomic_replaces_via_rename_and_leaves_no_tmp() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("state.json");
        write_atomic(&path, b"one").expect("first write");
        assert_eq!(fs::read(&path).expect("read"), b"one");

        #[cfg(unix)]
        let first_inode = {
            use std::os::unix::fs::MetadataExt;
            fs::metadata(&path).expect("metadata").ino()
        };

        write_atomic(&path, b"two").expect("second write");
        assert_eq!(fs::read(&path).expect("read"), b"two");

        // The replacement arrived by renaming a fresh file over the path,
        // not by opening and truncating the destination in place: a plain
        // truncating write would keep the inode.
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            assert_ne!(
                fs::metadata(&path).expect("metadata").ino(),
                first_inode,
                "atomic replace must produce a new inode"
            );
        }

        // No tmp litter remains.
        let names: Vec<String> = fs::read_dir(dir.path())
            .expect("read dir")
            .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["state.json"]);
    }
}
