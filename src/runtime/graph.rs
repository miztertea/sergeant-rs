//! The derived temporal graph (proposal §23).
//!
//! §23 is explicit that sergeant's graph is *not* a graph database: it is a
//! projection folded out of the journal, exactly like every other read model
//! here. Delete it and nothing is lost — replay rebuilds it.
//!
//! This module owns only the derivation: `fn(&mut GraphContext, &Event) ->
//! GraphDelta`, a pure fold with no I/O. Where the rows are *stored* is the
//! analytical projection's business ([`crate::runtime::analytics`] materializes
//! them into §22's `graph_nodes` / `graph_edges` tables), so there is one
//! storage owner and one rebuild path rather than two.
//!
//! **Every edge records the sequence number of the journal event that
//! justifies it** — §23's inspectability rule, and the reason this is a
//! derivation rather than an opinion. An edge whose `source_seq` does not
//! resolve to an event whose content implies the edge is a defect, and the
//! M5 acceptance tests check exactly that for a completed work.
//!
//! Nodes and edges are instantiated only where the journal carries the facts.
//! §23's `Artifact`, `File`, `Commit` and `Finding` nodes (and the edges that
//! reach them) have no source in today's event set — no adapter reports file
//! changes or commits yet — so they are absent rather than declared empty.

use std::collections::BTreeMap;

use crate::domain::event::Event;
use crate::domain::execution::KIND_EXECUTION_STARTED;
use crate::domain::work::KIND_WORK_SUBMITTED;
use crate::domain::workflow::{KIND_STAGE_ENTERED, KIND_WORKFLOW_BOUND};
use crate::runtime::surface::KIND_SURFACE_MATERIALIZED;

/// §27 event kind: a prompt was sent to the actor.
pub const KIND_CONVERSATION_USER: &str = "conversation.user";
/// §27 event kind: the actor produced assistant text.
pub const KIND_CONVERSATION_ASSISTANT_COMPLETED: &str = "conversation.assistant.completed";
/// §27 event kind: the actor asked for a tool call.
pub const KIND_TOOL_REQUESTED: &str = "tool.requested";
/// §27 event kind: a tool call returned.
pub const KIND_TOOL_COMPLETED: &str = "tool.completed";
/// §27 event kind: a turn reported token usage and cost.
pub const KIND_USAGE_UPDATED: &str = "usage.updated";
/// §27 event kind: a turn finished, however it finished.
pub const KIND_CONVERSATION_TURN_ENDED: &str = "conversation.turn.ended";
/// §27 event kind: the actor authored a question and is waiting for a human
/// answer (GP-2's ask primitive). Its own kind rather than a flavour of
/// assistant text: what the actor *said* and what the actor is *waiting on*
/// are different facts, and only the second one parks a stage.
pub const KIND_CONVERSATION_ASK: &str = "conversation.ask";

/// One node of the derived graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    /// Stable `kind:identity` id.
    pub id: String,
    /// §23 node kind (`work`, `stage`, `execution`, ...).
    pub kind: &'static str,
    /// Human-readable label.
    pub label: String,
    /// Work this node belongs to, when it belongs to exactly one. Shared
    /// nodes — a backend, a model, a client — carry `None`: they are reached
    /// from a work's neighborhood through an edge, not owned by it.
    pub work_id: Option<String>,
    /// Journal seq of the event that first instantiated this node.
    pub source_seq: u64,
}

/// One edge of the derived graph, with the provenance §23 requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphEdge {
    /// Stable `relation|from|to` id.
    pub id: String,
    /// §23 relation name.
    pub relation: &'static str,
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Work whose neighborhood this edge belongs to, when it has one.
    pub work_id: Option<String>,
    /// **Journal seq of the event that justifies this edge** (§23).
    pub source_seq: u64,
}

/// Nodes and edges derived from one event.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct GraphDelta {
    /// Nodes first seen at this event.
    pub nodes: Vec<GraphNode>,
    /// Edges justified by this event.
    pub edges: Vec<GraphEdge>,
}

/// The little bit of carried state the derivation needs: relations that link
/// an event to an *earlier* one (`Stage → preceded → Stage`, `Message →
/// caused → ToolCall`) cannot be read off a single event.
///
/// Rebuilt from scratch on every rebuild, so it never becomes a second source
/// of truth; two folds of the same journal produce the same graph.
#[derive(Debug, Default)]
pub struct GraphContext {
    /// Most recent stage node per work.
    last_stage: BTreeMap<String, String>,
    /// Most recent message per execution, as `(journal event id, node id)`.
    /// The event id is what makes `caused` provable: a tool call is linked to
    /// the message the *journal itself* recorded as its cause, never to
    /// "whatever came before".
    last_message: BTreeMap<String, (String, String)>,
}

/// Node id for a work. Internal: node ids are a derivation detail, and
/// callers read them off the emitted nodes and edges.
fn work_node(work_id: &str) -> String {
    format!("work:{work_id}")
}

/// Node id for one attempt of one stage of one work.
fn stage_node(work_id: &str, stage_id: &str, attempt: u64) -> String {
    format!("stage:{work_id}/{stage_id}#{attempt}")
}

/// Node id for an execution.
fn execution_node(execution_id: &str) -> String {
    format!("execution:{execution_id}")
}

impl GraphContext {
    /// Derive the nodes and edges one event justifies.
    ///
    /// Unknown kinds and unexpected payload shapes yield an empty delta: a
    /// newer writer's events must never brick an older reader's rebuild
    /// (§20's forward-compatibility stance, the same rule the registry
    /// reducer follows).
    pub fn derive(&mut self, event: &Event) -> GraphDelta {
        let mut delta = GraphDelta::default();
        let seq = event.seq;
        match event.kind.as_str() {
            KIND_WORK_SUBMITTED => {
                let work = &event.payload["work"];
                let Some(work_id) = work["id"].as_str() else {
                    return delta;
                };
                let work_node_id = work_node(work_id);
                delta.node(
                    &work_node_id,
                    "work",
                    work["intent"].as_str().unwrap_or(work_id),
                    Some(work_id),
                    seq,
                );
                // §23's `Client` node: who asked. Origin client (§13) when the
                // request declared one, else the recorded creator.
                if let Some(client) = work["origin_client"]
                    .as_str()
                    .or_else(|| work["created_by"].as_str())
                {
                    let node = format!("client:{client}");
                    delta.node(&node, "client", client, None, seq);
                    delta.edge("submitted", &node, &work_node_id, Some(work_id), seq);
                }
                // estate-root Phase C, §7.4: `work["workspace"]` is a
                // pre-Phase-C submission field, never written by a new
                // `work.submitted` — so this branch only ever fires for a
                // legacy journal replay now. The `workflow.bound` branch
                // below is the label's live source for every Work
                // journaled from Phase C onward (plan-time estate name,
                // not a client-supplied one).
                if let Some(workspace) = work["workspace"].as_str() {
                    let node = format!("workspace:{workspace}");
                    delta.node(&node, "workspace", workspace, None, seq);
                    delta.edge("scoped_to", &work_node_id, &node, Some(work_id), seq);
                }
            }
            KIND_WORKFLOW_BOUND => {
                let Some(work_id) = event.work_id.as_deref() else {
                    return delta;
                };
                let work_node_id = work_node(work_id);
                let workflow = &event.payload["workflow"];
                if let Some(name) = workflow["name"].as_str() {
                    let version = workflow["version"].as_str().unwrap_or("?");
                    let node = format!("workflow:{name}@{version}");
                    delta.node(&node, "workflow", &format!("{name} v{version}"), None, seq);
                    delta.edge("follows", &work_node_id, &node, Some(work_id), seq);
                }
                if let Some(profile) = event.payload["profile"]["name"].as_str() {
                    let node = format!("profile:{profile}");
                    delta.node(&node, "profile", profile, None, seq);
                    delta.edge("uses_profile", &work_node_id, &node, Some(work_id), seq);
                }
                // estate-root Phase C, §7.4: the discovered estate name at
                // plan time (`StartPlan.workspace.name`, journaled here
                // since before Phase C existed) is the label's live source
                // now that `Work.workspace` is never written for a new
                // submission — analytics.rs's `WorkRow` already prefers
                // this same field over the submission-time one; this is
                // that same "tolerate absence, prefer plan-time name"
                // reading for the graph's `workspace` node/edge.
                if let Some(workspace) = event.payload["workspace"].as_str() {
                    let node = format!("workspace:{workspace}");
                    delta.node(&node, "workspace", workspace, None, seq);
                    delta.edge("scoped_to", &work_node_id, &node, Some(work_id), seq);
                }
            }
            KIND_SURFACE_MATERIALIZED => {
                let Some(work_id) = event.work_id.as_deref() else {
                    return delta;
                };
                let work_node_id = work_node(work_id);
                let bindings = event.payload["surface"]["bindings"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                for binding in &bindings {
                    let Some(repository) = binding["repository"].as_str() else {
                        continue;
                    };
                    let node = format!("repository:{repository}");
                    let label = binding["source_path"].as_str().unwrap_or(repository);
                    delta.node(&node, "repository", label, None, seq);
                    delta.edge("targets", &work_node_id, &node, Some(work_id), seq);
                }
            }
            KIND_STAGE_ENTERED => {
                let (Some(work_id), Some(stage_id)) =
                    (event.work_id.as_deref(), event.payload["stage_id"].as_str())
                else {
                    return delta;
                };
                let attempt = event.payload["attempt"].as_u64().unwrap_or(1);
                let node = stage_node(work_id, stage_id, attempt);
                delta.node(
                    &node,
                    "stage",
                    &format!("{stage_id} (attempt {attempt})"),
                    Some(work_id),
                    seq,
                );
                if let Some(previous) = self.last_stage.get(work_id) {
                    // §23's `Stage → preceded → Stage`. Justified by *this*
                    // event: the journal says this stage was entered after
                    // that one, and the seq is where a reader can check.
                    let previous = previous.clone();
                    delta.edge("preceded", &previous, &node, Some(work_id), seq);
                }
                self.last_stage.insert(work_id.to_string(), node);
            }
            KIND_EXECUTION_STARTED => {
                let Some(work_id) = event.work_id.as_deref() else {
                    return delta;
                };
                let execution = &event.payload["execution"];
                let Some(execution_id) = execution["execution_id"].as_str() else {
                    return delta;
                };
                let node = execution_node(execution_id);
                delta.node(&node, "execution", execution_id, Some(work_id), seq);
                delta.edge("executes", &node, &work_node(work_id), Some(work_id), seq);
                if let Some(backend) = execution["backend"].as_str() {
                    let backend_node = format!("backend:{backend}");
                    delta.node(&backend_node, "backend", backend, None, seq);
                    delta.edge("uses", &node, &backend_node, Some(work_id), seq);
                }
                if let Some(native_id) = execution["native_id"].as_str() {
                    let session = format!("native_session:{native_id}");
                    delta.node(&session, "native_session", native_id, None, seq);
                    delta.edge("bound_to", &node, &session, Some(work_id), seq);
                }
                if let Some(stage_id) = execution["stage_id"].as_str() {
                    let attempt = execution["attempt"].as_u64().unwrap_or(1);
                    let stage = stage_node(work_id, stage_id, attempt);
                    delta.edge("ran_in", &node, &stage, Some(work_id), seq);
                }
            }
            KIND_CONVERSATION_USER | KIND_CONVERSATION_ASSISTANT_COMPLETED => {
                let Some(execution_id) = event.execution_id.as_deref() else {
                    return delta;
                };
                let role = if event.kind == KIND_CONVERSATION_USER {
                    "user"
                } else {
                    "assistant"
                };
                let node = format!("message:{seq}");
                let text = event.payload["text"].as_str().unwrap_or_default();
                delta.node(
                    &node,
                    "message",
                    &format!("{role}: {}", truncate(text, 80)),
                    event.work_id.as_deref(),
                    seq,
                );
                delta.edge(
                    "produced",
                    &execution_node(execution_id),
                    &node,
                    event.work_id.as_deref(),
                    seq,
                );
                self.last_message
                    .insert(execution_id.to_string(), (event.id.clone(), node));
            }
            KIND_TOOL_REQUESTED => {
                let Some(execution_id) = event.execution_id.as_deref() else {
                    return delta;
                };
                let tool_use_id = event.payload["id"]
                    .as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| format!("seq{seq}"));
                let name = event.payload["name"].as_str().unwrap_or("tool");
                let node = format!("tool_call:{execution_id}/{tool_use_id}");
                delta.node(&node, "tool_call", name, event.work_id.as_deref(), seq);
                delta.edge(
                    "requested",
                    &execution_node(execution_id),
                    &node,
                    event.work_id.as_deref(),
                    seq,
                );
                // §23's `Message → caused → ToolCall`, emitted only when the
                // journal's own causation link names the message. Ordering
                // alone ("the message just before it") would be a guess; the
                // causation id is a recorded fact.
                if let (Some(cause), Some((message_event, message_node))) = (
                    event.causation_id.as_deref(),
                    self.last_message.get(execution_id),
                ) && cause == message_event
                {
                    delta.edge("caused", message_node, &node, event.work_id.as_deref(), seq);
                }
            }
            KIND_USAGE_UPDATED => {
                let Some(execution_id) = event.execution_id.as_deref() else {
                    return delta;
                };
                let models = event.payload["model_usage"]
                    .as_object()
                    .cloned()
                    .unwrap_or_default();
                for model in models.keys() {
                    let node = format!("model:{model}");
                    delta.node(&node, "model", model, None, seq);
                    delta.edge(
                        "used_model",
                        &execution_node(execution_id),
                        &node,
                        event.work_id.as_deref(),
                        seq,
                    );
                }
            }
            _ => {}
        }
        delta
    }
}

impl GraphDelta {
    fn node(
        &mut self,
        id: &str,
        kind: &'static str,
        label: &str,
        work_id: Option<&str>,
        source_seq: u64,
    ) {
        self.nodes.push(GraphNode {
            id: id.to_string(),
            kind,
            label: label.to_string(),
            work_id: work_id.map(str::to_string),
            source_seq,
        });
    }

    fn edge(
        &mut self,
        relation: &'static str,
        from: &str,
        to: &str,
        work_id: Option<&str>,
        source_seq: u64,
    ) {
        self.edges.push(GraphEdge {
            id: format!("{relation}|{from}|{to}"),
            relation,
            from: from.to_string(),
            to: to.to_string(),
            work_id: work_id.map(str::to_string),
            source_seq,
        });
    }
}

/// Shorten a label to `limit` characters, marking that it was shortened.
fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let head: String = text.chars().take(limit).collect();
    format!("{head}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::event::{EventDraft, EventSource};
    use serde_json::json;

    fn event(seq: u64, work_id: &str, kind: &str, payload: serde_json::Value) -> Event {
        EventDraft::new(EventSource::new("daemon", "test"), kind, payload)
            .with_work_id(work_id)
            .into_event(seq)
    }

    #[test]
    fn stage_precedence_links_consecutive_attempts_and_never_the_first() {
        let mut context = GraphContext::default();
        let first = context.derive(&event(
            1,
            "w",
            KIND_STAGE_ENTERED,
            json!({"stage_id": "00-first", "index": 0, "attempt": 1}),
        ));
        assert!(
            first.edges.is_empty(),
            "the first stage of a work is preceded by nothing: {first:?}"
        );
        let second = context.derive(&event(
            2,
            "w",
            KIND_STAGE_ENTERED,
            json!({"stage_id": "10-second", "index": 1, "attempt": 1}),
        ));
        let edge = second.edges.first().expect("a preceded edge");
        assert_eq!(edge.relation, "preceded");
        assert_eq!(edge.from, stage_node("w", "00-first", 1));
        assert_eq!(edge.to, stage_node("w", "10-second", 1));
        assert_eq!(
            edge.source_seq, 2,
            "the justifying event is the *later* stage entry"
        );
    }

    #[test]
    fn a_tool_call_is_linked_to_its_cause_only_when_the_journal_recorded_one() {
        let mut context = GraphContext::default();
        let mut message = event(
            1,
            "w",
            KIND_CONVERSATION_USER,
            json!({"text": "do the thing"}),
        );
        message.execution_id = Some("e1".to_string());
        context.derive(&message);

        // A tool call whose causation id names that message: the §23 edge.
        let mut caused = event(
            2,
            "w",
            KIND_TOOL_REQUESTED,
            json!({"id": "t1", "name": "Bash"}),
        );
        caused.execution_id = Some("e1".to_string());
        caused.causation_id = Some(message.id.clone());
        let delta = context.derive(&caused);
        let edge = delta
            .edges
            .iter()
            .find(|e| e.relation == "caused")
            .expect("a caused edge");
        assert_eq!(edge.from, "message:1");
        assert_eq!(edge.source_seq, 2);

        // One with no recorded cause is *not* linked to the nearest message:
        // proximity is not causation, and inventing the link would make
        // `source_seq` point at an event that does not justify it.
        let mut uncaused = event(
            3,
            "w",
            KIND_TOOL_REQUESTED,
            json!({"id": "t2", "name": "Read"}),
        );
        uncaused.execution_id = Some("e1".to_string());
        let delta = context.derive(&uncaused);
        assert!(
            delta.edges.iter().all(|e| e.relation != "caused"),
            "an unrecorded cause must not be guessed: {delta:?}"
        );

        // And one whose causation names some *other* event is not linked to
        // this execution's last message either. This is the case that
        // separates "the journal recorded *this* link" from "there was a
        // causation id and a message lying around": without it the identity
        // check could be weakened to a presence check and nothing would fail.
        let mut miscaused = event(
            4,
            "w",
            KIND_TOOL_REQUESTED,
            json!({"id": "t3", "name": "Edit"}),
        );
        miscaused.execution_id = Some("e1".to_string());
        miscaused.causation_id = Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string());
        let delta = context.derive(&miscaused);
        assert!(
            delta.edges.iter().all(|e| e.relation != "caused"),
            "a causation id naming another event must not link this message: {delta:?}"
        );
    }

    #[test]
    fn unknown_kinds_and_malformed_payloads_derive_nothing() {
        let mut context = GraphContext::default();
        assert_eq!(
            context.derive(&event(1, "w", "something.newer", json!({"x": 1}))),
            GraphDelta::default()
        );
        assert_eq!(
            context.derive(&event(
                2,
                "w",
                KIND_EXECUTION_STARTED,
                json!({"execution": 7})
            )),
            GraphDelta::default()
        );
    }

    /// An event with no work scope at all (as opposed to `event()`, which
    /// always sets one) — the shape several of `derive`'s guards fire on.
    fn event_no_work(seq: u64, kind: &str, payload: serde_json::Value) -> Event {
        EventDraft::new(EventSource::new("daemon", "test"), kind, payload).into_event(seq)
    }

    /// Issue #33 item 2: `GraphContext::derive`'s guard family across event
    /// kinds beyond the one case already pinned above (`execution.started`
    /// with a malformed `execution` object) — each event below is missing
    /// exactly the scoping field its handler reads, and must derive an empty
    /// delta rather than panic or invent a node from absent data.
    #[test]
    fn derive_guards_across_kinds_skip_malformed_events_without_panicking() {
        let mut context = GraphContext::default();

        assert_eq!(
            context.derive(&event_no_work(
                1,
                KIND_WORK_SUBMITTED,
                json!({"work": {"intent": "orphan"}}),
            )),
            GraphDelta::default(),
            "work.submitted with no work.id must derive nothing"
        );
        assert_eq!(
            context.derive(&event_no_work(
                2,
                KIND_WORKFLOW_BOUND,
                json!({"workflow": {"name": "tiny", "version": "1"}}),
            )),
            GraphDelta::default(),
            "workflow.bound with no work_id must derive nothing"
        );
        assert_eq!(
            // Non-empty bindings on purpose: this arm only emits from
            // inside the loop over them, so `"bindings": []` would derive
            // nothing whether or not the work_id guard is there.
            context.derive(&event_no_work(
                3,
                KIND_SURFACE_MATERIALIZED,
                json!({"surface": {"bindings": [
                    {"repository": "repo", "source_path": "/src/repo"},
                ]}}),
            )),
            GraphDelta::default(),
            "surface.materialized with no work_id must derive nothing"
        );
        assert_eq!(
            context.derive(&event(4, "w", KIND_STAGE_ENTERED, json!({"index": 0}))),
            GraphDelta::default(),
            "stage.entered with no stage_id must derive nothing"
        );
        assert_eq!(
            context.derive(&event_no_work(
                5,
                KIND_EXECUTION_STARTED,
                json!({"execution": {"execution_id": "e1"}}),
            )),
            GraphDelta::default(),
            "execution.started with no work_id must derive nothing"
        );
        assert_eq!(
            context.derive(&event(
                6,
                "w",
                KIND_CONVERSATION_USER,
                json!({"text": "hi"})
            )),
            GraphDelta::default(),
            "a conversation event with no execution scope must derive nothing"
        );
        assert_eq!(
            context.derive(&event(
                7,
                "w",
                KIND_TOOL_REQUESTED,
                json!({"id": "t1", "name": "Bash"}),
            )),
            GraphDelta::default(),
            "tool.requested with no execution scope must derive nothing"
        );
        assert_eq!(
            context.derive(&event(
                8,
                "w",
                KIND_USAGE_UPDATED,
                json!({"model_usage": {"claude": {}}}),
            )),
            GraphDelta::default(),
            "usage.updated with no execution scope must derive nothing"
        );
    }

    /// Issue #33 item 2: `truncate()`'s shortening branch (never exercised —
    /// no baseline fixture used a message long enough) alongside the
    /// unchanged and boundary cases.
    #[test]
    fn truncate_marks_shortened_text_and_leaves_text_at_the_limit_untouched() {
        assert_eq!(truncate("short", 80), "short");
        let exactly_at_limit = "x".repeat(80);
        assert_eq!(truncate(&exactly_at_limit, 80), exactly_at_limit);

        let long = "x".repeat(100);
        let shortened = truncate(&long, 80);
        assert_eq!(shortened.chars().count(), 81, "{shortened}");
        assert!(shortened.ends_with('…'), "{shortened}");
        assert!(shortened.starts_with(&"x".repeat(80)), "{shortened}");
    }

    /// The same shortening branch, exercised end-to-end through `derive` so
    /// the graph label a real long conversation turn produces is pinned too.
    #[test]
    fn a_long_conversation_message_is_truncated_in_its_graph_label() {
        let mut context = GraphContext::default();
        let mut message = event(
            1,
            "w",
            KIND_CONVERSATION_USER,
            json!({"text": "y".repeat(200)}),
        );
        message.execution_id = Some("e1".to_string());
        let delta = context.derive(&message);
        let node = delta
            .nodes
            .iter()
            .find(|n| n.kind == "message")
            .expect("a message node");
        assert!(
            node.label.ends_with('…'),
            "a long message must be marked truncated: {}",
            node.label
        );
        assert!(
            node.label.chars().count() < 200,
            "the label must be shorter than the untruncated text: {}",
            node.label
        );
    }
}
