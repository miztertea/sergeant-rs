//! sergeant-rs library surface.
//!
//! The binary (`sgt`) is a thin shell over this crate; keeping the modules in
//! a library target lets integration tests exercise the event core directly.

pub mod api;
pub mod backend;
pub mod cli;
pub mod daemon;
pub mod domain;
pub mod platform;
pub mod runtime;
pub mod telemetry;
pub mod tui;
pub mod watch;

/// Fixtures shared across this crate's own unit tests (`cargo test --lib`),
/// as opposed to `tests/support`, which serves the separately-compiled
/// integration test binaries under `tests/`.
#[cfg(test)]
pub(crate) mod test_support {
    /// #83: a freshly written, freshly `chmod +x`'d stand-in script can
    /// transiently fail `execve(2)` with `ETXTBSY` ("text file busy", `os
    /// error 26`) while another handle on the same inode is still open for
    /// writing — under `cargo test`'s default thread parallelism, a sibling
    /// test's fork-to-exec window can overlap this one's write. Retry until
    /// the exec stops being refused, or surface any other failure
    /// immediately.
    pub(crate) fn wait_until_executable(path: &std::path::Path) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while let Err(e) = std::process::Command::new(path).arg("--version").output() {
            assert!(
                e.raw_os_error() == Some(26) && std::time::Instant::now() < deadline,
                "the stand-in at {path:?} is not runnable: {e}"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    }
}
