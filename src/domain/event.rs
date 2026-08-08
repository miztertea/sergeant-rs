//! Event envelope (proposal §19, deviation D1: `sergeant.event/v1` schema).
//!
//! Every state change in sergeant is recorded as an immutable event in this
//! envelope. Unknown envelope fields found in stored events are preserved on
//! read (`extra`), so a future schema revision can reinterpret history without
//! this version destroying fidelity (§20).

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Envelope schema identifier for events written by this version.
pub const EVENT_SCHEMA: &str = "sergeant.event/v1";

/// Who emitted an event (e.g. `{"type": "backend", "name": "claude"}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventSource {
    /// Source category: `daemon`, `backend`, `cli`, ...
    #[serde(rename = "type")]
    pub source_type: String,
    /// Concrete source name within the category.
    pub name: String,
    /// Source fields this version does not understand, preserved verbatim so
    /// re-serialization is lossless (forward compatibility, §20). Unknown-field
    /// preservation must cover the whole envelope, including sub-objects.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl EventSource {
    /// Convenience constructor.
    pub fn new(source_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            source_type: source_type.into(),
            name: name.into(),
            extra: BTreeMap::new(),
        }
    }
}

/// The stable event envelope. One journal line is one serialized `Event`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Envelope schema identifier (`sergeant.event/v1`).
    pub schema: String,
    /// Daemon-local, monotonically increasing sequence number (assigned by
    /// the journal on append; starts at 1).
    pub seq: u64,
    /// ULID identifying this event.
    pub id: String,
    /// RFC3339 UTC timestamp with millisecond precision.
    pub timestamp: String,
    /// Emitter of the event.
    pub source: EventSource,
    /// Workspace this event belongs to, when applicable. Absent keys and
    /// explicit `null`s in stored lines both read as `None`; this version's
    /// writer never emits the key when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Work item this event belongs to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,
    /// Execution this event belongs to, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<String>,
    /// Groups one logical operation across events.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    /// The event id that caused this event, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    /// Dotted event kind, e.g. `tool.completed`.
    pub kind: String,
    /// Kind-specific payload.
    pub payload: Value,
    /// Envelope fields this version does not understand, preserved verbatim
    /// so re-serialization is lossless (forward compatibility, §20).
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Everything the caller supplies for a new event; the journal assigns
/// `schema`, `seq`, `id`, and `timestamp` at append time.
#[derive(Debug, Clone, PartialEq)]
pub struct EventDraft {
    /// Emitter of the event.
    pub source: EventSource,
    /// Optional workspace scope.
    pub workspace_id: Option<String>,
    /// Optional work scope.
    pub work_id: Option<String>,
    /// Optional execution scope.
    pub execution_id: Option<String>,
    /// Optional logical-operation grouping id.
    pub correlation_id: Option<String>,
    /// Optional causing event id.
    pub causation_id: Option<String>,
    /// Dotted event kind.
    pub kind: String,
    /// Kind-specific payload.
    pub payload: Value,
}

impl EventDraft {
    /// A draft with only the required fields set.
    pub fn new(source: EventSource, kind: impl Into<String>, payload: Value) -> Self {
        Self {
            source,
            workspace_id: None,
            work_id: None,
            execution_id: None,
            correlation_id: None,
            causation_id: None,
            kind: kind.into(),
            payload,
        }
    }

    /// Set the work id scope.
    pub fn with_work_id(mut self, work_id: impl Into<String>) -> Self {
        self.work_id = Some(work_id.into());
        self
    }

    /// Finalize into a full envelope at the given sequence number, stamping a
    /// fresh ULID id and the current UTC time.
    pub fn into_event(self, seq: u64) -> Event {
        Event {
            schema: EVENT_SCHEMA.to_string(),
            seq,
            id: ulid::Ulid::generate().to_string(),
            timestamp: rfc3339_utc_now(),
            source: self.source,
            workspace_id: self.workspace_id,
            work_id: self.work_id,
            execution_id: self.execution_id,
            correlation_id: self.correlation_id,
            causation_id: self.causation_id,
            kind: self.kind,
            payload: self.payload,
            extra: BTreeMap::new(),
        }
    }
}

/// Current UTC time as RFC3339 with millisecond precision
/// (e.g. `2026-08-08T01:42:12.437Z`).
pub fn rfc3339_utc_now() -> String {
    rfc3339_utc(SystemTime::now())
}

/// Format a `SystemTime` as RFC3339 UTC with millisecond precision.
///
/// Implemented from the days-to-civil algorithm to avoid pulling a date-time
/// crate into the pinned dependency set. Times before the Unix epoch clamp to
/// the epoch (sergeant never legitimately produces them).
pub fn rfc3339_utc(t: SystemTime) -> String {
    let d = t.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs() as i64;
    let millis = d.subsec_millis();

    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (hour, minute, second) = (
        secs_of_day / 3600,
        (secs_of_day / 60) % 60,
        secs_of_day % 60,
    );

    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year_of_era = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 {
        year_of_era + 1
    } else {
        year_of_era
    };

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rfc3339_known_instants() {
        let t = UNIX_EPOCH + Duration::from_millis(0);
        assert_eq!(rfc3339_utc(t), "1970-01-01T00:00:00.000Z");
        // 2026-08-08T01:42:12.437Z == 1786153332.437 seconds after the epoch.
        let t = UNIX_EPOCH + Duration::from_millis(1_786_153_332_437);
        assert_eq!(rfc3339_utc(t), "2026-08-08T01:42:12.437Z");
        // Leap-year day.
        let t = UNIX_EPOCH + Duration::from_secs(1_709_164_800);
        assert_eq!(rfc3339_utc(t), "2024-02-29T00:00:00.000Z");
    }

    #[test]
    fn draft_into_event_stamps_envelope() {
        let before = rfc3339_utc_now();
        let event = EventDraft::new(
            EventSource::new("daemon", "sergeant"),
            "work.created",
            serde_json::json!({"title": "x"}),
        )
        .with_work_id("work_1")
        .into_event(7);
        let after = rfc3339_utc_now();
        assert_eq!(event.schema, EVENT_SCHEMA);
        assert_eq!(event.seq, 7);
        assert_eq!(event.work_id.as_deref(), Some("work_1"));
        assert_eq!(event.id.len(), 26); // ULID canonical text form
        // The stamped timestamp is the real current UTC time in the pinned
        // fixed-width RFC3339 shape (`rfc3339_utc` itself is pinned to known
        // instants above; fixed-width strings compare chronologically), not
        // an arbitrary constant that merely ends in 'Z'.
        assert_eq!(event.timestamp.len(), 24);
        assert!(event.timestamp.ends_with('Z'));
        assert!(
            before <= event.timestamp && event.timestamp <= after,
            "timestamp {} outside [{before}, {after}]",
            event.timestamp
        );
        assert!(event.extra.is_empty());
        assert!(event.source.extra.is_empty());
    }

    #[test]
    fn explicit_null_known_field_reads_as_unset_and_unknown_nulls_survive() {
        // A line from another writer may spell a *known* optional field as an
        // explicit `null`: it reads as unset (same as an absent key). An
        // *unknown* field — even a null-valued one — is preserved verbatim
        // through `extra`, per the contract's forward-compatibility clause;
        // known-field null normalization is fine because stored lines are
        // immutable (re-serialization never rewrites a journal line).
        let line = concat!(
            r#"{"schema":"sergeant.event/v1","seq":1,"id":"01ARZ3NDEKTSV4RRFFQ69G5FAV","#,
            r#""timestamp":"2026-08-08T00:00:00.000Z","#,
            r#""source":{"type":"daemon","name":"sergeant"},"#,
            r#""work_id":null,"kind":"work.created","payload":{},"unknown_null":null}"#
        );
        let event: Event = serde_json::from_str(line).expect("parse");
        assert_eq!(event.work_id, None); // explicit null reads as unset
        assert_eq!(event.workspace_id, None); // key genuinely absent
        assert_eq!(event.extra["unknown_null"], Value::Null);
        let reserialized = serde_json::to_string(&event).expect("serialize");
        // The unknown null-valued field survives re-serialization; the
        // null-valued *known* field is normalized to an absent key.
        assert!(reserialized.contains(r#""unknown_null":null"#));
        assert!(!reserialized.contains("work_id"));
        // And the re-serialized form round-trips to the same parsed event.
        let back: Event = serde_json::from_str(&reserialized).expect("reparse");
        assert_eq!(back, event);
    }
}
