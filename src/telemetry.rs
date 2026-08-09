//! OpenTelemetry export (proposal §28) — **off by default**.
//!
//! §28's own framing is the design: "internal execution events remain Depot
//! events; OTel is an export/projection". So this module is a *fold over the
//! journal*, exactly like the DuckDB and graph projections, and not an
//! instrumentation layer sprinkled through the engine:
//!
//! - a `work` outlives every request, every stage and (via §25 recovery) every
//!   daemon process. Request-scoped `tracing` spans bridged through
//!   `tracing-opentelemetry` cannot represent that tree — they would export
//!   one sliver per HTTP handler. The journal already holds the nesting §28
//!   draws, with real start and end timestamps, so the spans are built from
//!   it;
//! - being a fold, the export is disposable and re-derivable in the same
//!   sense the other two projections are: nothing here is state anything
//!   else reads back.
//!
//! ```text
//! work
//!  └─ stage.<id>
//!      └─ execution.turn
//!          └─ tool.<name>
//! ```
//!
//! Spans carry the journal's own start and end timestamps, so an exported
//! duration is the duration the trajectory actually had — not the interval
//! between two moments this fold happened to run.
//!
//! **Off by default** is structural, not a runtime `if`: with telemetry
//! disabled [`Telemetry::from_config`] returns `None`, no provider is built,
//! no exporter task exists, and the daemon never subscribes anything to the
//! event stream.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use opentelemetry::trace::{
    Span as _, SpanBuilder, SpanContext, TraceContextExt, Tracer, TracerProvider,
};
use opentelemetry::{Context, KeyValue};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider, exporter::PushMetricExporter};
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanExporter};

use crate::domain::event::{Event, unix_millis};
use crate::domain::execution::{KIND_EXECUTION_STARTED, KIND_EXECUTION_STOPPED};
use crate::domain::work::{KIND_WORK_NEEDS_INPUT, KIND_WORK_SUBMITTED, WorkState};
use crate::domain::workflow::{
    KIND_STAGE_BLOCKED, KIND_STAGE_CANCELED, KIND_STAGE_COMPLETED, KIND_STAGE_ENTERED,
    KIND_STAGE_FAILED, KIND_STAGE_NEEDS_INPUT, KIND_STAGE_WAITING, KIND_WORKFLOW_BOUND,
};
use crate::runtime::graph::{
    KIND_CONVERSATION_TURN_ENDED, KIND_TOOL_COMPLETED, KIND_TOOL_REQUESTED, KIND_USAGE_UPDATED,
};

/// Service name reported on every exported span and metric.
pub const SERVICE_NAME: &str = "sergeant";

/// §28 export configuration. [`Default`] is **off**.
///
/// Every field here is a claim the daemon makes about its own behaviour, so
/// each one is read by [`TelemetryConfig::from_env_with`] and asserted in the
/// M5 suite: an advertised switch nobody measures is an unmeasured claim.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TelemetryConfig {
    /// Whether to export at all. Off unless explicitly switched on.
    pub enabled: bool,
    /// OTLP/HTTP collector base URL. Defaults to [`DEFAULT_OTLP_ENDPOINT`].
    pub otlp_endpoint: Option<String>,
}

/// Environment variable that switches §28 export on (`1`/`true`/`yes`/`on`).
pub const OTEL_ENABLE_VAR: &str = "SGT_OTEL";
/// Environment variable naming the OTLP/HTTP collector base URL.
pub const OTLP_ENDPOINT_VAR: &str = "SGT_OTLP_ENDPOINT";
/// Collector base URL used when export is on and none was configured.
pub const DEFAULT_OTLP_ENDPOINT: &str = "http://localhost:4318";

impl TelemetryConfig {
    /// Read the configuration from a variable lookup.
    ///
    /// Takes the lookup as an argument rather than reading the process
    /// environment directly so it is testable without mutating global state
    /// (which is racy under a threaded test runner, and would make the
    /// off-by-default assertion depend on test ordering).
    pub fn from_env_with(lookup: impl Fn(&str) -> Option<String>) -> Self {
        let enabled = lookup(OTEL_ENABLE_VAR).is_some_and(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        });
        Self {
            enabled,
            otlp_endpoint: lookup(OTLP_ENDPOINT_VAR).filter(|v| !v.trim().is_empty()),
        }
    }

    /// Read the configuration from this process's environment.
    pub fn from_env() -> Self {
        Self::from_env_with(|name| std::env::var(name).ok())
    }

    /// The endpoint export would use.
    pub fn endpoint(&self) -> &str {
        self.otlp_endpoint
            .as_deref()
            .unwrap_or(DEFAULT_OTLP_ENDPOINT)
    }
}

/// Failure building the export pipeline.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// The OTLP exporter could not be constructed (bad endpoint, no client).
    #[error("cannot build the OTLP exporter: {0}")]
    Exporter(String),
}

/// A span this fold has opened and not yet ended.
///
/// The SDK span itself is held (not just its ids): OpenTelemetry 0.32's
/// `SpanBuilder` no longer accepts caller-chosen trace and span ids, so a
/// child's parent is established by handing the SDK the parent's live
/// [`SpanContext`]. Holding the span is also what lets it be ended later with
/// the journal's timestamp rather than the wall clock.
struct OpenSpan {
    span: opentelemetry_sdk::trace::Span,
    context: SpanContext,
    start_ms: i64,
    attributes: Vec<KeyValue>,
}

/// Everything the fold carries between events.
#[derive(Default)]
struct FoldState {
    /// Open `work` span per work id.
    works: BTreeMap<String, OpenSpan>,
    /// Open `stage.*` span per work id (a work has at most one live stage).
    stages: BTreeMap<String, OpenSpan>,
    /// Open `execution.turn` span per execution id.
    executions: BTreeMap<String, OpenSpan>,
    /// Open `tool.*` span per `execution_id/tool_use_id`.
    tools: BTreeMap<String, OpenSpan>,
    /// Backend each work routed to, for failure attribution.
    backends: BTreeMap<String, String>,
    /// Last known §10 state per work, and when it was entered — the input to
    /// the active gauges and the wait histogram.
    states: BTreeMap<String, (WorkState, i64)>,
}

/// The §28 instruments this build exports.
///
/// Only metrics whose inputs exist in today's journal are here. §28 also
/// lists `backend_restart_total` (no adapter reports a restart as such) and
/// `event_ingest_lag` (the journal records one timestamp per event, not an
/// observation time to subtract it from) — absent rather than exported as a
/// constant zero.
struct Instruments {
    work_active: opentelemetry::metrics::UpDownCounter<i64>,
    execution_active: opentelemetry::metrics::UpDownCounter<i64>,
    execution_duration: opentelemetry::metrics::Histogram<f64>,
    stage_duration: opentelemetry::metrics::Histogram<f64>,
    wait_duration: opentelemetry::metrics::Histogram<f64>,
    tool_duration: opentelemetry::metrics::Histogram<f64>,
    journal_append: opentelemetry::metrics::Histogram<f64>,
    needs_input_total: opentelemetry::metrics::Counter<u64>,
    backend_failure_total: opentelemetry::metrics::Counter<u64>,
    token_input_total: opentelemetry::metrics::Counter<u64>,
    token_output_total: opentelemetry::metrics::Counter<u64>,
}

/// The §28 export projection: fold journal events in, spans and metrics out.
pub struct Telemetry {
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
    tracer: opentelemetry_sdk::trace::SdkTracer,
    instruments: Instruments,
    state: Mutex<FoldState>,
}

impl std::fmt::Debug for Telemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Telemetry")
    }
}

impl Telemetry {
    /// Build the export pipeline for `config`, or `None` when export is off.
    ///
    /// `None` is the whole of "off by default": there is no disabled
    /// [`Telemetry`] instance to accidentally leave wired up.
    pub fn from_config(config: &TelemetryConfig) -> Result<Option<Self>, TelemetryError> {
        if !config.enabled {
            return Ok(None);
        }
        Ok(Some(Self::otlp(config.endpoint())?))
    }

    /// An OTLP/HTTP pipeline against `endpoint` (batched, as a collector
    /// expects).
    ///
    /// Building the exporter does not require a reachable collector: OTLP
    /// export is asynchronous and best-effort, which is correct for a
    /// projection — a collector that is down must never affect execution.
    pub fn otlp(endpoint: &str) -> Result<Self, TelemetryError> {
        use opentelemetry_otlp::WithExportConfig;
        let spans = opentelemetry_otlp::SpanExporter::builder()
            .with_http()
            .with_endpoint(format!("{}/v1/traces", endpoint.trim_end_matches('/')))
            .build()
            .map_err(|e| TelemetryError::Exporter(e.to_string()))?;
        let metrics = opentelemetry_otlp::MetricExporter::builder()
            .with_http()
            .with_endpoint(format!("{}/v1/metrics", endpoint.trim_end_matches('/')))
            .build()
            .map_err(|e| TelemetryError::Exporter(e.to_string()))?;
        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(resource())
            .with_batch_exporter(spans)
            .build();
        Ok(Self::from_providers(tracer_provider, metrics))
    }

    /// A pipeline over caller-supplied exporters, exporting each span the
    /// moment it ends.
    ///
    /// This is how §28 is verified without a collector: hand it an in-memory
    /// exporter and the span tree is an assertion rather than a hope.
    pub fn with_exporters(
        spans: impl SpanExporter + 'static,
        metrics: impl PushMetricExporter + 'static,
    ) -> Self {
        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(resource())
            .with_simple_exporter(spans)
            .build();
        Self::from_providers(tracer_provider, metrics)
    }

    fn from_providers(
        tracer_provider: SdkTracerProvider,
        metrics: impl PushMetricExporter + 'static,
    ) -> Self {
        let meter_provider = SdkMeterProvider::builder()
            .with_resource(resource())
            .with_reader(PeriodicReader::builder(metrics).build())
            .build();
        let tracer = tracer_provider.tracer(SERVICE_NAME);
        let meter = opentelemetry::metrics::MeterProvider::meter(&meter_provider, SERVICE_NAME);
        let instruments = Instruments {
            work_active: meter.i64_up_down_counter("sergeant_work_active").build(),
            execution_active: meter
                .i64_up_down_counter("sergeant_execution_active")
                .build(),
            execution_duration: meter
                .f64_histogram("sergeant_execution_duration_seconds")
                .build(),
            stage_duration: meter
                .f64_histogram("sergeant_stage_duration_seconds")
                .build(),
            wait_duration: meter
                .f64_histogram("sergeant_wait_duration_seconds")
                .build(),
            tool_duration: meter
                .f64_histogram("sergeant_tool_duration_seconds")
                .build(),
            journal_append: meter
                .f64_histogram("sergeant_journal_append_seconds")
                .build(),
            needs_input_total: meter.u64_counter("sergeant_needs_input_total").build(),
            backend_failure_total: meter.u64_counter("sergeant_backend_failure_total").build(),
            token_input_total: meter.u64_counter("sergeant_token_input_total").build(),
            token_output_total: meter.u64_counter("sergeant_token_output_total").build(),
        };
        Self {
            tracer_provider,
            meter_provider,
            tracer,
            instruments,
            state: Mutex::new(FoldState::default()),
        }
    }

    /// Record how long one journal append took (§28's
    /// `journal_append_seconds`). Called from the journal's append observer,
    /// which is the only place that timing exists.
    pub fn record_journal_append(&self, elapsed: Duration) {
        self.instruments
            .journal_append
            .record(elapsed.as_secs_f64(), &[]);
    }

    /// Flush both pipelines. Tests call this before asserting; the daemon
    /// calls it on shutdown.
    pub fn force_flush(&self) {
        let _ = self.tracer_provider.force_flush();
        let _ = self.meter_provider.force_flush();
    }

    /// Flush and stop. Spans still open are *not* exported: an unfinished
    /// span has no end time, and inventing one would export a duration the
    /// journal never recorded.
    pub fn shutdown(&self) {
        self.force_flush();
        let _ = self.tracer_provider.shutdown();
        let _ = self.meter_provider.shutdown();
    }

    /// Fold one journal event into spans and metrics.
    pub fn record(&self, event: &Event) {
        let Some(ts) = unix_millis(&event.timestamp) else {
            return;
        };
        let mut state = self.state.lock().expect("telemetry fold state lock");
        self.record_locked(&mut state, event, ts);
    }

    fn record_locked(&self, state: &mut FoldState, event: &Event, ts: i64) {
        let work_id = event.work_id.clone();
        match event.kind.as_str() {
            KIND_WORK_SUBMITTED => {
                let work = &event.payload["work"];
                let Some(id) = work["id"].as_str() else {
                    return;
                };
                let span = self.open_span(
                    "work",
                    ts,
                    vec![
                        KeyValue::new("sergeant.work.id", id.to_string()),
                        KeyValue::new(
                            "sergeant.work.intent",
                            work["intent"].as_str().unwrap_or_default().to_string(),
                        ),
                    ],
                    None,
                );
                state.works.insert(id.to_string(), span);
                state
                    .states
                    .insert(id.to_string(), (WorkState::Pending, ts));
            }
            KIND_WORKFLOW_BOUND => {
                if let (Some(id), Some(backend)) =
                    (work_id.as_deref(), event.payload["backend"].as_str())
                {
                    state.backends.insert(id.to_string(), backend.to_string());
                }
            }
            KIND_STAGE_ENTERED => {
                let (Some(id), Some(stage_id)) =
                    (work_id.as_deref(), event.payload["stage_id"].as_str())
                else {
                    return;
                };
                // A retry re-enters a stage whose span may still be open
                // (the previous attempt ended by being superseded, not by a
                // stage event). Close it here rather than leak it.
                if let Some(open) = state.stages.remove(id) {
                    self.end_span(open, ts, "superseded");
                }
                let attempt = event.payload["attempt"].as_u64().unwrap_or(1);
                let Some(parent) = state.works.get(id).map(|s| s.context.clone()) else {
                    return;
                };
                let span = self.open_span(
                    format!("stage.{stage_id}"),
                    ts,
                    vec![
                        KeyValue::new("sergeant.work.id", id.to_string()),
                        KeyValue::new("sergeant.stage.id", stage_id.to_string()),
                        KeyValue::new("sergeant.stage.attempt", attempt as i64),
                    ],
                    Some(&parent),
                );
                state.stages.insert(id.to_string(), span);
            }
            KIND_STAGE_COMPLETED
            | KIND_STAGE_WAITING
            | KIND_STAGE_NEEDS_INPUT
            | KIND_STAGE_BLOCKED
            | KIND_STAGE_FAILED
            | KIND_STAGE_CANCELED => {
                let Some(id) = work_id.as_deref() else { return };
                let Some(open) = state.stages.remove(id) else {
                    return;
                };
                let status = event.kind.strip_prefix("stage.").unwrap_or(&event.kind);
                let attributes = open.attributes.clone();
                let seconds = seconds_between(open.start_ms, ts);
                self.end_span(open, ts, status);
                let mut labels = attributes;
                labels.push(KeyValue::new("sergeant.stage.status", status.to_string()));
                self.instruments.stage_duration.record(seconds, &labels);
            }
            KIND_EXECUTION_STARTED => {
                let execution = &event.payload["execution"];
                let (Some(id), Some(execution_id)) =
                    (work_id.as_deref(), execution["execution_id"].as_str())
                else {
                    return;
                };
                let parent = match state.stages.get(id).or_else(|| state.works.get(id)) {
                    Some(open) => open.context.clone(),
                    None => return,
                };
                let backend = execution["backend"].as_str().unwrap_or("unknown");
                let span = self.open_span(
                    "execution.turn",
                    ts,
                    vec![
                        KeyValue::new("sergeant.work.id", id.to_string()),
                        KeyValue::new("sergeant.execution.id", execution_id.to_string()),
                        KeyValue::new("sergeant.backend", backend.to_string()),
                    ],
                    Some(&parent),
                );
                state.executions.insert(execution_id.to_string(), span);
                self.instruments
                    .execution_active
                    .add(1, &[KeyValue::new("sergeant.backend", backend.to_string())]);
            }
            KIND_EXECUTION_STOPPED | KIND_CONVERSATION_TURN_ENDED => {
                let execution_id = event.payload["execution_id"]
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| event.execution_id.clone());
                let Some(execution_id) = execution_id else {
                    return;
                };
                let Some(open) = state.executions.remove(&execution_id) else {
                    return;
                };
                let attributes = open.attributes.clone();
                let seconds = seconds_between(open.start_ms, ts);
                self.end_span(open, ts, "ended");
                self.instruments
                    .execution_duration
                    .record(seconds, &attributes);
                let backend = attributes
                    .iter()
                    .find(|kv| kv.key.as_str() == "sergeant.backend")
                    .cloned();
                self.instruments
                    .execution_active
                    .add(-1, backend.as_slice());
            }
            KIND_TOOL_REQUESTED => {
                let Some(execution_id) = event.execution_id.as_deref() else {
                    return;
                };
                let Some(parent) = state
                    .executions
                    .get(execution_id)
                    .map(|s| s.context.clone())
                else {
                    return;
                };
                let tool_use_id = event.payload["id"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("seq{}", event.seq));
                let name = event.payload["name"].as_str().unwrap_or("tool").to_string();
                let span = self.open_span(
                    format!("tool.{}", name.to_ascii_lowercase()),
                    ts,
                    vec![
                        KeyValue::new("sergeant.tool.name", name),
                        KeyValue::new("sergeant.execution.id", execution_id.to_string()),
                    ],
                    Some(&parent),
                );
                state
                    .tools
                    .insert(format!("{execution_id}/{tool_use_id}"), span);
            }
            KIND_TOOL_COMPLETED => {
                let (Some(execution_id), Some(tool_use_id)) = (
                    event.execution_id.as_deref(),
                    event.payload["tool_use_id"].as_str(),
                ) else {
                    return;
                };
                let Some(open) = state.tools.remove(&format!("{execution_id}/{tool_use_id}"))
                else {
                    return;
                };
                let attributes = open.attributes.clone();
                let seconds = seconds_between(open.start_ms, ts);
                let status = if event.payload["is_error"].as_bool() == Some(true) {
                    "error"
                } else {
                    "ok"
                };
                self.end_span(open, ts, status);
                self.instruments.tool_duration.record(seconds, &attributes);
            }
            KIND_USAGE_UPDATED => {
                let usage = &event.payload["usage"];
                let labels = [KeyValue::new(
                    "sergeant.work.id",
                    work_id.clone().unwrap_or_default(),
                )];
                if let Some(input) = usage["input_tokens"].as_u64() {
                    self.instruments.token_input_total.add(input, &labels);
                }
                if let Some(output) = usage["output_tokens"].as_u64() {
                    self.instruments.token_output_total.add(output, &labels);
                }
            }
            _ => {}
        }

        if event.kind == KIND_WORK_NEEDS_INPUT {
            self.instruments.needs_input_total.add(
                1,
                &[KeyValue::new(
                    "sergeant.work.id",
                    work_id.clone().unwrap_or_default(),
                )],
            );
        }
        if let (Some(id), Some(new_state)) =
            (work_id.as_deref(), WorkState::for_event_kind(&event.kind))
        {
            self.transition(state, id, new_state, ts);
        }
    }

    /// Apply a §10 state transition to the gauges, the wait histogram, the
    /// failure counter, and the `work` span's lifetime.
    fn transition(&self, state: &mut FoldState, work_id: &str, new: WorkState, ts: i64) {
        let previous = state.states.insert(work_id.to_string(), (new, ts));
        let labels = [KeyValue::new("sergeant.work.id", work_id.to_string())];
        if let Some((old, since)) = previous {
            if is_waiting(old) {
                let mut wait_labels = labels.to_vec();
                wait_labels.push(KeyValue::new("sergeant.work.state", old.as_str()));
                self.instruments
                    .wait_duration
                    .record(seconds_between(since, ts), &wait_labels);
            }
            if old == WorkState::Active && new != WorkState::Active {
                self.instruments.work_active.add(-1, &labels);
            }
        }
        if new == WorkState::Active {
            self.instruments.work_active.add(1, &labels);
        }
        if matches!(new, WorkState::Failed | WorkState::Blocked) {
            let backend = state
                .backends
                .get(work_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            self.instruments.backend_failure_total.add(
                1,
                &[
                    KeyValue::new("sergeant.backend", backend),
                    KeyValue::new("sergeant.work.state", new.as_str()),
                ],
            );
        }
        if is_terminal(new) {
            // The work span closes with the work, and with it the whole
            // trace: a terminal work emits nothing more.
            if let Some(open) = state.stages.remove(work_id) {
                self.end_span(open, ts, new.as_str());
            }
            if let Some(open) = state.works.remove(work_id) {
                self.end_span(open, ts, new.as_str());
            }
        }
    }

    /// Open a span with the journal's start timestamp, nested under `parent`.
    fn open_span(
        &self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        start_ms: i64,
        attributes: Vec<KeyValue>,
        parent: Option<&SpanContext>,
    ) -> OpenSpan {
        let context = match parent {
            Some(parent) => Context::new().with_remote_span_context(parent.clone()),
            None => Context::new(),
        };
        let span = self.tracer.build_with_context(
            SpanBuilder::from_name(name)
                .with_start_time(system_time(start_ms))
                .with_attributes(attributes.clone()),
            &context,
        );
        let span_context = span.span_context().clone();
        OpenSpan {
            span,
            context: span_context,
            start_ms,
            attributes,
        }
    }

    /// End one span at the instant the journal recorded, not at the instant
    /// the fold happened to reach the event.
    fn end_span(&self, mut open: OpenSpan, end_ms: i64, status: &str) {
        open.span
            .set_attribute(KeyValue::new("sergeant.status", status.to_string()));
        open.span.end_with_timestamp(system_time(end_ms));
    }
}

/// Whether a §10 state is one the work is *waiting in* (§28's wait duration).
fn is_waiting(state: WorkState) -> bool {
    matches!(
        state,
        WorkState::Waiting | WorkState::NeedsInput | WorkState::Blocked
    )
}

/// Whether a §10 state ends the work for export purposes.
fn is_terminal(state: WorkState) -> bool {
    matches!(
        state,
        WorkState::Completed | WorkState::Failed | WorkState::Canceled
    )
}

fn seconds_between(start_ms: i64, end_ms: i64) -> f64 {
    (end_ms - start_ms).max(0) as f64 / 1000.0
}

fn system_time(ms: i64) -> SystemTime {
    UNIX_EPOCH + Duration::from_millis(ms.max(0) as u64)
}

fn resource() -> Resource {
    Resource::builder()
        .with_service_name(SERVICE_NAME)
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_is_off_unless_switched_on_and_the_switch_is_explicit() {
        // The default, and an environment with nothing set: off.
        assert!(!TelemetryConfig::default().enabled);
        let off = TelemetryConfig::from_env_with(|_| None);
        assert_eq!(off, TelemetryConfig::default());
        assert!(Telemetry::from_config(&off).expect("config").is_none());

        // Set-but-not-truthy is still off: an operator who wrote SGT_OTEL=0
        // asked for it to be off, and a mere-presence check would betray that.
        for value in ["0", "false", "no", "off", ""] {
            let config = TelemetryConfig::from_env_with(|name| {
                (name == OTEL_ENABLE_VAR).then(|| value.to_string())
            });
            assert!(!config.enabled, "SGT_OTEL={value:?} must not enable export");
            assert!(Telemetry::from_config(&config).expect("config").is_none());
        }
        for value in ["1", "true", "YES", " on "] {
            let config = TelemetryConfig::from_env_with(|name| {
                (name == OTEL_ENABLE_VAR).then(|| value.to_string())
            });
            assert!(config.enabled, "SGT_OTEL={value:?} must enable export");
        }
    }

    #[test]
    fn the_endpoint_is_configurable_and_defaults() {
        let config = TelemetryConfig::from_env_with(|name| match name {
            OTEL_ENABLE_VAR => Some("1".to_string()),
            OTLP_ENDPOINT_VAR => Some("http://collector.invalid:4318".to_string()),
            _ => None,
        });
        assert_eq!(config.endpoint(), "http://collector.invalid:4318");
        let defaulted =
            TelemetryConfig::from_env_with(|name| (name == OTEL_ENABLE_VAR).then(|| "1".into()));
        assert_eq!(defaulted.endpoint(), DEFAULT_OTLP_ENDPOINT);
        // A blank value is not an endpoint; it falls back rather than
        // producing "/v1/traces" against nothing.
        let blank = TelemetryConfig::from_env_with(|name| match name {
            OTEL_ENABLE_VAR => Some("1".to_string()),
            OTLP_ENDPOINT_VAR => Some("   ".to_string()),
            _ => None,
        });
        assert_eq!(blank.endpoint(), DEFAULT_OTLP_ENDPOINT);
    }
}
