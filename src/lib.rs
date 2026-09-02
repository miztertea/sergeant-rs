//! sergeant-rs library surface.
//!
//! The binary (`sgt`) is a thin shell over this crate; keeping the modules in
//! a library target lets integration tests exercise the event core directly.

pub mod api;
pub mod backend;
pub mod cli;
pub mod daemon;
pub mod domain;
pub mod harness;
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

    /// A bare-bones HTTP/1.1 responder for a fixed script of `(status,
    /// body)` pairs, answered once each, in order (R2: one shared site for
    /// `cli`'s scan-follow tests and `api`'s request-retry tests — both
    /// need "accept, read, then hang up with no response" as the real shape
    /// of a lost connection, not a mocking dependency, S6 client-request-
    /// retry). Returns the endpoint and a shared counter of accepted
    /// connections, so a test can assert exactly how many attempts were
    /// made rather than inferring it from timing.
    ///
    /// `status: 0` is the one sentinel: accept the connection, read
    /// whatever request arrives, then drop it with no bytes written at all
    /// — a connection reset, not a status code — for a script entry that
    /// must simulate a hangup rather than answer.
    pub(crate) fn spawn_scripted_http_server(
        script: Vec<(u16, &'static str)>,
    ) -> (String, std::sync::Arc<std::sync::atomic::AtomicUsize>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind fake daemon");
        let addr = listener.local_addr().expect("local addr");
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempts_clone = attempts.clone();
        std::thread::spawn(move || {
            use std::io::{Read, Write};
            for (status, body) in script {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                attempts_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf); // discard the request itself
                if status == 0 {
                    drop(stream); // hangup: no response written at all
                    continue;
                }
                let reason = match status {
                    200 => "OK",
                    202 => "Accepted",
                    404 => "Not Found",
                    _ => "Error",
                };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        (format!("http://{addr}"), attempts)
    }
}
