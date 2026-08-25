# Runtime, environment, and telemetry

The normal estate data directory is `.sergeant/data`; precedence is explicit global `--data-dir`, `SGT_DATA_DIR`, estate configuration, then platform defaults where the command permits. It contains the journal, blobs, projections, runtime descriptor/token, locks, and default Work surfaces. Treat it as private local state.

Relevant environment includes `SGT_DATA_DIR`, `SGT_CLIENT_TIMEOUT_SECS`, `SGT_OTEL`, and `SGT_OTLP_ENDPOINT`, plus native harness variables owned by those harnesses. Command-line values take precedence where the corresponding flag exists.

Set `SGT_OTEL=1` to enable OTLP/HTTP export. The default endpoint is `http://localhost:4318`. Sergeant records Work/stage/execution activity and operational metrics, but telemetry is lossy and disposable. The journal remains the reconstruction authority.

Analytics queries projections derived from durable evidence. If a projection disagrees with the journal-backed Work view, rebuild or diagnose the projection rather than rewriting history.
