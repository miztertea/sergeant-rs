# Runtime, environment, and telemetry

The host runtime root — precedence is explicit global `--data-dir`, `SGT_DATA_DIR`, then the platform default (XDG on Linux, Application Support on macOS) — holds the journal, blobs, the DuckDB projection, the daemon's own descriptor/token/lock, and default Work surfaces for every estate the daemon has admitted. It is shared across estates, not per-estate; treat it as private local state. `[estate] data_dir` is deprecated (`sgt doctor` warns if a manifest still declares it — the journal and projection it used to place have no per-estate home to place anymore). `[estate] surfaces_dir`/`SGT_SURFACES_DIR` still narrow Work surfaces back to being estate-local; without one, a Work's surface materializes under the shared host root, not under the estate.

Relevant environment includes `SGT_DATA_DIR`, `SGT_CLIENT_TIMEOUT_SECS`, `SGT_OTEL`, and `SGT_OTLP_ENDPOINT`, plus native harness variables owned by those harnesses. Command-line values take precedence where the corresponding flag exists.

Set `SGT_OTEL=1` to enable OTLP/HTTP export. The default endpoint is `http://localhost:4318`. Sergeant records Work/stage/execution activity and operational metrics, but telemetry is lossy and disposable. The journal remains the reconstruction authority.

Analytics queries projections derived from durable evidence. If a projection disagrees with the journal-backed Work view, rebuild or diagnose the projection rather than rewriting history.
