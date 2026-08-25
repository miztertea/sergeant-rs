# Automation and observability

Use global `--json` and documented machine interfaces; never scrape human text. `sgt watch [<id>] --follow` emits attention and terminal notices without a polling loop. Treat each watch event as invalidation and use its fresh snapshot as Work truth.

The loopback v1 API exposes JSON mutation/query routes and SSE events. Authenticate `/v1/*` with the bearer token from the estate runtime descriptor. Supply a unique caller command ID for mutation idempotency and safely reuse that ID only when retrying the same logical command.

OpenTelemetry is optional and lossy:

```sh
SGT_OTEL=1 SGT_OTLP_ENDPOINT=http://localhost:4318 sgt daemon
```

Use telemetry and analytics for observation, not reconstruction. The journal is durable truth; projections can be rebuilt.

Pin integrations to a Sergeant release and test JSON/watch/API fixtures during upgrades. See [machine interfaces](../reference/machine-interfaces.md) and [runtime reference](../reference/runtime.md).
