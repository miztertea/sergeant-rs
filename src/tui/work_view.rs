//! The canonical Work surface (§13, Decision T2-22/T2-48): five read-only
//! tabs — Thread (§13.3), Workflow (§13.4), Evidence (§13.5), Graph (§13.6),
//! Details (§13.7, including §13.8's Output and §13.9's Envelope cards) —
//! over four already-shipped API reads: `GET /v1/work/{id}`, `GET /v1/work/
//! {id}/transcript`, `GET /v1/events?work_id=…`, and `GET /v1/graph/work/
//! {id}`.
//!
//! There is one canonical representation (Decision T2-48): [`render`] is
//! only ever shown for the Work `App::open_work` names, and never becomes a
//! second Work screen with its own action vocabulary. Fleet's own selected-
//! row preview (§10.1) is a distinct, deliberately smaller thing —
//! [`render_preview`] — built from the row Fleet already has loaded, with no
//! per-Work fetch of its own.
//!
//! §13.10's action matrix is display-only here: T1c wires respond/retry/
//! extend/cancel to the API. This Work only shows, per the current reported
//! state, which actions the daemon would accept.

use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph, Wrap};
use serde_json::Value;

use crate::api::{ApiClient, ClientError, field_text, stage_label};

use super::connection::Live;
use super::fleet::WorkRow;
use super::theme::{self, Token};

/// §13.2's five views, in their displayed order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Thread,
    Workflow,
    Evidence,
    Graph,
    Details,
}

impl Tab {
    const ALL: [Tab; 5] = [
        Tab::Thread,
        Tab::Workflow,
        Tab::Evidence,
        Tab::Graph,
        Tab::Details,
    ];

    fn label(self) -> &'static str {
        match self {
            Tab::Thread => "Thread",
            Tab::Workflow => "Workflow",
            Tab::Evidence => "Evidence",
            Tab::Graph => "Graph",
            Tab::Details => "Details",
        }
    }

    fn next(self) -> Tab {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    fn prev(self) -> Tab {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }

    fn from_digit(c: char) -> Option<Tab> {
        match c {
            '1' => Some(Tab::Thread),
            '2' => Some(Tab::Workflow),
            '3' => Some(Tab::Evidence),
            '4' => Some(Tab::Graph),
            '5' => Some(Tab::Details),
            _ => None,
        }
    }
}

/// §13.5's default Evidence window, and the bounded "load older" step/cap —
/// growing the window and re-fetching is the whole mechanism (`work_events`
/// only ever answers "the newest N", so "older" means "more").
pub const DEFAULT_EVIDENCE_LIMIT: usize = 200;
const EVIDENCE_LIMIT_STEP: usize = 200;
const EVIDENCE_LIMIT_MAX: usize = 2000;

/// What a keystroke inside an open Work surface asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkScreenOutcome {
    None,
    /// Return to wherever this Work was opened from.
    Close,
    /// §13.5's bounded "load older": grow the Evidence window and re-fetch.
    LoadOlder,
}

/// The canonical Work surface's own state: the four fetched reads, which tab
/// is showing, and each tab's local navigation (selection, Evidence's filter
/// text and window).
#[derive(Debug, Clone)]
pub struct WorkScreen {
    pub id: String,
    work: Value,
    transcript: Vec<Value>,
    events: Vec<Value>,
    graph: Option<Value>,
    tab: Tab,
    evidence_limit: usize,
    evidence_filter: String,
    evidence_filter_focus: bool,
    thread_selected: usize,
    evidence_selected: usize,
    graph_selected: usize,
    /// Set when a background refresh (not the initial open) fails — shown
    /// inline rather than losing the screen the last successful read built.
    pub last_error: Option<String>,
}

impl WorkScreen {
    pub(crate) fn from_parts(
        id: String,
        work: Value,
        transcript: Vec<Value>,
        events: Vec<Value>,
        graph: Option<Value>,
    ) -> Self {
        Self {
            id,
            work,
            transcript,
            events,
            graph,
            tab: Tab::Thread,
            evidence_limit: DEFAULT_EVIDENCE_LIMIT,
            evidence_filter: String::new(),
            evidence_filter_focus: false,
            thread_selected: 0,
            evidence_selected: 0,
            graph_selected: 0,
            last_error: None,
        }
    }

    /// Open a Work: the four reads §13.3-§13.6 are built from. The graph
    /// read is best-effort (`None` on failure) — the analytical projection
    /// can be transiently unavailable (503) without that taking down a
    /// screen whose other three reads succeeded.
    pub async fn load(client: &ApiClient, id: &str) -> Result<Self, ClientError> {
        let work = client.work(id).await?;
        let transcript = client.work_transcript(id).await?;
        let events = client.work_events(id, DEFAULT_EVIDENCE_LIMIT).await?;
        let graph = client.get(&format!("/v1/graph/work/{id}")).await.ok();
        Ok(Self::from_parts(
            id.to_string(),
            work,
            turns_of(transcript),
            events_of(events),
            graph,
        ))
    }

    /// Re-read this Work's four sources at the current Evidence bound,
    /// leaving tab/scroll/filter UI state untouched — the same "state enters
    /// only through a read" rule `App::refresh` follows for everything else.
    /// Errors are recorded on [`WorkScreen::last_error`] rather than
    /// discarding the last successful read.
    pub async fn refresh(&mut self, client: &ApiClient) -> Result<(), ClientError> {
        let id = self.id.clone();
        let work = client.work(&id).await?;
        let transcript = client.work_transcript(&id).await?;
        let events = client.work_events(&id, self.evidence_limit).await?;
        let graph = client.get(&format!("/v1/graph/work/{id}")).await.ok();
        self.work = work;
        self.transcript = turns_of(transcript);
        self.events = events_of(events);
        self.graph = graph;
        self.last_error = None;
        Ok(())
    }

    /// §13.5: grow the Evidence window (bounded) and re-fetch.
    pub async fn load_older(&mut self, client: &ApiClient) -> Result<(), ClientError> {
        self.evidence_limit = (self.evidence_limit + EVIDENCE_LIMIT_STEP).min(EVIDENCE_LIMIT_MAX);
        self.refresh(client).await
    }

    fn state(&self) -> String {
        field_text(&self.work["work"]["state"])
    }

    // ---------------------------------------------------------------- keys

    pub fn on_key(&mut self, key: KeyCode) -> WorkScreenOutcome {
        if self.evidence_filter_focus {
            match key {
                KeyCode::Esc | KeyCode::Enter => self.evidence_filter_focus = false,
                KeyCode::Backspace => {
                    self.evidence_filter.pop();
                }
                KeyCode::Char(c) => self.evidence_filter.push(c),
                _ => {}
            }
            return WorkScreenOutcome::None;
        }
        match key {
            KeyCode::Esc | KeyCode::Char('q') => return WorkScreenOutcome::Close,
            KeyCode::Tab => self.tab = self.tab.next(),
            KeyCode::BackTab => self.tab = self.tab.prev(),
            KeyCode::Char(c) if Tab::from_digit(c).is_some() => {
                self.tab = Tab::from_digit(c).expect("checked by the guard above");
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Char('/') if self.tab == Tab::Evidence => self.evidence_filter_focus = true,
            KeyCode::Char('x') if self.tab == Tab::Evidence && !self.evidence_filter.is_empty() => {
                self.evidence_filter.clear();
            }
            KeyCode::Char('o') if self.tab == Tab::Evidence => return WorkScreenOutcome::LoadOlder,
            _ => {}
        }
        WorkScreenOutcome::None
    }

    fn move_selection(&mut self, delta: i32) {
        let count = match self.tab {
            Tab::Thread => self.thread_rows().len(),
            Tab::Evidence => self.visible_events().len(),
            Tab::Graph => self.graph_lines().len(),
            Tab::Workflow | Tab::Details => 0,
        };
        if count == 0 {
            return;
        }
        if let Some(selected) = self.selected_mut() {
            let next = (*selected as i32 + delta).clamp(0, count as i32 - 1);
            *selected = next as usize;
        }
    }

    fn selected_mut(&mut self) -> Option<&mut usize> {
        match self.tab {
            Tab::Thread => Some(&mut self.thread_selected),
            Tab::Evidence => Some(&mut self.evidence_selected),
            Tab::Graph => Some(&mut self.graph_selected),
            Tab::Workflow | Tab::Details => None,
        }
    }

    // ------------------------------------------------------------- Thread

    /// §13.3: the transcript and the event history, merged by journal
    /// sequence and classified per the default-rendering table. Every row
    /// traces back to a journaled fact (Decision T2-49) — nothing here is
    /// inferred progress or chain-of-thought.
    fn thread_rows(&self) -> Vec<ThreadRow> {
        let mut rows = vec![ThreadRow {
            seq: 0,
            ts: field_text(&self.work["work"]["created_at"]),
            fact: ThreadFact::Intent(field_text(&self.work["work"]["intent"])),
        }];
        for turn in &self.transcript {
            let seq = turn["seq"].as_u64().unwrap_or(0);
            let ts = field_text(&turn["ts"]);
            let text = field_text(&turn["text"]);
            let fact = match turn["role"].as_str().unwrap_or("") {
                "ask" => ThreadFact::Ask(text),
                "assistant" => ThreadFact::Assistant(text),
                "user" => ThreadFact::Human(text),
                other => ThreadFact::Generic(format!("transcript.{other}")),
            };
            rows.push(ThreadRow { seq, ts, fact });
        }
        for event in &self.events {
            let kind = event["kind"].as_str().unwrap_or("");
            // Already covered by the transcript above (§13.3: "merged by
            // journal sequence" — the transcript IS the conversation.* slice
            // of the same journal, decoded).
            if kind.starts_with("conversation.") {
                continue;
            }
            rows.push(ThreadRow {
                seq: event["seq"].as_u64().unwrap_or(0),
                ts: field_text(&event["timestamp"]),
                fact: classify_event(kind, &event["payload"]),
            });
        }
        if !self.work["output"].is_null() {
            rows.push(ThreadRow {
                seq: u64::MAX,
                ts: String::new(),
                fact: ThreadFact::Output(output_summary(&self.work["output"])),
            });
        }
        rows.sort_by_key(|r| r.seq);
        if self.state() == "completed_dirty" {
            for row in &mut rows {
                if matches!(row.fact, ThreadFact::Completed) {
                    row.fact = ThreadFact::CompletedDirty;
                }
            }
        }
        rows
    }

    // ----------------------------------------------------------- Workflow

    /// §13.4: the pinned ordered stage list as a rail. Decision T2-50 —
    /// order, not duration; never a percent bar. Only the *current* stage's
    /// executor is available from `GET /v1/work/{id}` (`stage.executor`);
    /// earlier/later stages' pinned bindings are not part of that response,
    /// so this shows harness/profile only where the read actually supplies
    /// it, per stage.
    fn workflow_lines(&self) -> Vec<Line<'static>> {
        let stage_ids: Vec<String> = self.work["workflow"]["stages"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        if stage_ids.is_empty() {
            return vec![Line::from("No pinned workflow yet.")];
        }
        let current_index = self.work["stage"]["index"].as_u64();
        let attempt = self.work["stage"]["attempt"].as_u64().unwrap_or(1);
        let executor = &self.work["stage"]["executor"];
        stage_ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let i = i as u64;
                let (glyph, style) = match current_index {
                    Some(cur) if i < cur => ("✓", Style::default().fg(Token::Success.rgb())),
                    Some(cur) if i == cur => ("⠹", Style::default().fg(Token::Focus.rgb())),
                    _ => ("·", Style::default().fg(Token::Muted.rgb())),
                };
                let mut spans = vec![
                    Span::styled(format!("{glyph} "), style),
                    Span::styled(id.clone(), style),
                ];
                if current_index == Some(i) {
                    let mut suffix = format!("  attempt {attempt}");
                    if let Some(harness) = executor["harness"].as_str() {
                        suffix.push_str(&format!("  {harness}"));
                        if let Some(profile) = executor["profile"]["name"].as_str() {
                            suffix.push_str(&format!(" ({profile})"));
                        }
                    }
                    spans.push(Span::styled(
                        suffix,
                        Style::default().fg(Token::Muted.rgb()),
                    ));
                }
                Line::from(spans)
            })
            .collect()
    }

    // ----------------------------------------------------------- Evidence

    /// §13.5's local filter over already-loaded rows: substring match on
    /// kind, source, or payload — never a new search backend.
    fn visible_events(&self) -> Vec<&Value> {
        let needle = self.evidence_filter.to_lowercase();
        self.events
            .iter()
            .filter(|event| {
                if needle.is_empty() {
                    return true;
                }
                let kind = event["kind"].as_str().unwrap_or("").to_lowercase();
                let source = format!(
                    "{} {}",
                    field_text(&event["source"]["type"]),
                    field_text(&event["source"]["name"])
                )
                .to_lowercase();
                let payload = event["payload"].to_string().to_lowercase();
                kind.contains(&needle) || source.contains(&needle) || payload.contains(&needle)
            })
            .collect()
    }

    // -------------------------------------------------------------- Graph

    /// §13.6: the one-Work graph neighborhood, walked from the Work node
    /// outward — forward edges (`from` is this node) and reverse edges
    /// (`to` is this node) both shown, since real relations point either way
    /// (`executes` runs execution → work, not work → execution). Two levels
    /// deep: the neighborhood the API returns is already bounded to this
    /// Work's own edges plus whatever they reach, not a transitive walk of
    /// the whole fleet. Decision T2-51: nothing here is a node or edge the
    /// response did not actually supply.
    fn graph_lines(&self) -> Vec<String> {
        let Some(graph) = &self.graph else {
            return vec![
                "Graph unavailable (the analytics projection did not answer).".to_string(),
            ];
        };
        let nodes = graph["nodes"].as_array().cloned().unwrap_or_default();
        let edges = graph["edges"].as_array().cloned().unwrap_or_default();
        if nodes.is_empty() {
            return vec!["No graph data yet for this Work.".to_string()];
        }
        let label_of = |id: &str| -> String {
            nodes
                .iter()
                .find(|n| n["node_id"].as_str() == Some(id))
                .map(|n| field_text(&n["label"]))
                .unwrap_or_else(|| id.to_string())
        };
        let root = format!("work:{}", self.id);
        let mut lines = vec![format!("Work  {}", label_of(&root))];
        for edge in &edges {
            let from = edge["from_node"].as_str().unwrap_or("");
            let to = edge["to_node"].as_str().unwrap_or("");
            let relation = edge["relation"].as_str().unwrap_or("");
            if from == root {
                lines.push(format!("  ├─ {relation} -> {}", label_of(to)));
                for inner in &edges {
                    if inner["from_node"].as_str() == Some(to) {
                        let inner_to = inner["to_node"].as_str().unwrap_or("");
                        let inner_relation = inner["relation"].as_str().unwrap_or("");
                        lines.push(format!(
                            "  │    ├─ {inner_relation} -> {}",
                            label_of(inner_to)
                        ));
                    }
                }
            } else if to == root {
                lines.push(format!("  ├─ {relation} <- {}", label_of(from)));
            }
        }
        lines
    }

    // ------------------------------------------------------------ Details

    /// §13.7's progressive-disclosure facts, plus §13.8's Output and §13.9's
    /// Envelope cards rendered with exactly the fields those sections list.
    /// Sections the current API read cannot fill (persisted-vs-reported
    /// state, workflow content identity, retained state) say so rather than
    /// fabricating an answer.
    fn details_lines(&self) -> Vec<Line<'static>> {
        let w = &self.work["work"];
        let mut lines = Vec::new();

        heading("Work", &mut lines);
        lines.push(kv("id", &self.id));
        lines.push(kv("origin", &field_text(&w["origin_client"])));
        lines.push(kv("created by", &field_text(&w["created_by"])));
        lines.push(kv("created at", &field_text(&w["created_at"])));
        blank(&mut lines);

        heading("Workspace", &mut lines);
        lines.push(kv("workspace", &field_text(&w["workspace"])));
        lines.push(kv("repositories", &join_str_array(&w["repositories"])));
        blank(&mut lines);

        heading("State", &mut lines);
        let state = field_text(&w["state"]);
        lines.push(kv("reported state", &state));
        if state == "completed_dirty" {
            lines.push(Line::from(Span::styled(
                "  persisted state is completed (ADR 0007(b)); this read does not expose \
                 persisted and reported state as two distinct fields"
                    .to_string(),
                Style::default().fg(Token::Muted.rgb()),
            )));
        }
        blank(&mut lines);

        heading("Workflow", &mut lines);
        lines.push(kv("name", &field_text(&self.work["workflow"]["name"])));
        lines.push(kv(
            "version",
            &field_text(&self.work["workflow"]["version"]),
        ));
        lines.push(kv("source", &field_text(&self.work["workflow"]["source"])));
        lines.push(kv("route source", &field_text(&self.work["route_source"])));
        blank(&mut lines);

        heading("Reservation", &mut lines);
        if self.work["reservation"].is_null() {
            lines.push(kv("reservation", "-"));
        } else {
            let r = &self.work["reservation"];
            lines.push(kv("execution id", &field_text(&r["execution_id"])));
            lines.push(kv("backend", &field_text(&r["backend"])));
            lines.push(kv("native id", &field_text(&r["native_id"])));
            lines.push(kv(
                "stage",
                &format!(
                    "{} attempt {}",
                    field_text(&r["stage_id"]),
                    field_text(&r["attempt"])
                ),
            ));
        }
        blank(&mut lines);

        heading("Execution / native session", &mut lines);
        if self.work["execution"].is_null() {
            lines.push(kv("execution", "-"));
        } else {
            let e = &self.work["execution"];
            lines.push(kv("execution id", &field_text(&e["execution_id"])));
            lines.push(kv("backend", &field_text(&e["backend"])));
            lines.push(kv("native session", &field_text(&e["native_id"])));
            lines.push(kv("stop requested", &field_text(&e["stop_requested"])));
        }
        blank(&mut lines);

        heading("Surface bindings", &mut lines);
        match self.work["surface"]["bindings"].as_array() {
            Some(bindings) if !bindings.is_empty() => {
                for binding in bindings {
                    lines.push(kv(
                        &field_text(&binding["repository"]),
                        &format!(
                            "worktree {}  branch {}  base {}@{}",
                            field_text(&binding["worktree_path"]),
                            field_text(&binding["work_branch"]),
                            field_text(&binding["base_branch"]),
                            field_text(&binding["base_sha"]),
                        ),
                    ));
                }
            }
            _ => lines.push(kv("surface", "-")),
        }
        blank(&mut lines);

        heading("Teardown", &mut lines);
        match self.work["teardown"]["bindings"].as_array() {
            Some(bindings) if !bindings.is_empty() => {
                lines.push(kv("clean", &field_text(&self.work["teardown"]["clean"])));
                for binding in bindings {
                    lines.push(kv(
                        &field_text(&binding["repository"]),
                        &format!(
                            "{}  branch {}  final {}",
                            field_text(&binding["disposition"]),
                            field_text(&binding["work_branch"]),
                            field_text(&binding["final_sha"]),
                        ),
                    ));
                }
            }
            _ => lines.push(kv("teardown", "-")),
        }
        blank(&mut lines);

        heading("Output (§13.8)", &mut lines);
        match self.work["output"]["repositories"].as_array() {
            Some(repos) if !repos.is_empty() => {
                for repo in repos {
                    lines.push(kv(
                        &field_text(&repo["repository"]),
                        &format!(
                            "source {}  branch {}  commit {}  worktree {}  {}",
                            field_text(&repo["source_repo"]),
                            field_text(&repo["retained_branch"]),
                            field_text(&repo["finalize_commit"]),
                            field_text(&repo["worktree_path"]),
                            field_text(&repo["disposition"]),
                        ),
                    ));
                }
            }
            _ => lines.push(kv("output", "-")),
        }
        blank(&mut lines);

        heading("Envelope (§13.9)", &mut lines);
        let env = &self.work["envelope"];
        lines.push(kv(
            "turns spawned / effective cap",
            &format!(
                "{} / {}",
                field_text(&env["turns_spawned"]),
                field_text(&env["turn_cap"])
            ),
        ));
        lines.push(kv("bonus turns", &field_text(&env["turn_cap_bonus"])));
        lines.push(kv(
            "per-turn ceiling (secs)",
            &field_text(&env["turn_ceiling_secs"]),
        ));
        let latest_extension = self
            .events
            .iter()
            .rev()
            .find(|e| e["kind"] == "execution.turn_envelope_extended");
        lines.push(kv(
            "latest extension",
            &match latest_extension {
                Some(e) => format!(
                    "+{} turns at {}",
                    field_text(&e["payload"]["additional_turns"]),
                    field_text(&e["timestamp"])
                ),
                None => "-".to_string(),
            },
        ));
        blank(&mut lines);

        heading("Retained state", &mut lines);
        lines.push(Line::from(Span::styled(
            "  not read here — /v1/retained is an estate-wide inspect verb outside this \
             Work's four scoped reads (T3's Estate surface, §20.4)"
                .to_string(),
            Style::default().fg(Token::Muted.rgb()),
        )));

        lines
    }
}

fn turns_of(transcript: Value) -> Vec<Value> {
    transcript["turns"].as_array().cloned().unwrap_or_default()
}

fn events_of(events: Value) -> Vec<Value> {
    events["events"].as_array().cloned().unwrap_or_default()
}

fn join_str_array(value: &Value) -> String {
    value
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string())
}

fn output_summary(output: &Value) -> String {
    match output["repositories"].as_array() {
        Some(repos) if !repos.is_empty() => repos
            .iter()
            .map(|r| {
                format!(
                    "{}: branch {} @ {} ({})",
                    field_text(&r["repository"]),
                    field_text(&r["retained_branch"]),
                    field_text(&r["finalize_commit"]),
                    field_text(&r["disposition"]),
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        _ => "output pointer recorded, no repositories".to_string(),
    }
}

fn heading(title: &str, lines: &mut Vec<Line<'static>>) {
    lines.push(Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Token::Muted.rgb()),
    )));
}

fn blank(lines: &mut Vec<Line<'static>>) {
    lines.push(Line::default());
}

fn kv(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {label:<28} "),
            Style::default().fg(Token::Muted.rgb()),
        ),
        Span::raw(value.to_string()),
    ])
}

/// One merged Thread row: its journal sequence (the merge key), the
/// timestamp it carries, and what it renders as.
struct ThreadRow {
    seq: u64,
    ts: String,
    fact: ThreadFact,
}

/// §13.3's default-rendering table, as data.
enum ThreadFact {
    Intent(String),
    Human(String),
    Assistant(String),
    Ask(String),
    Divider(String),
    Lifecycle(String),
    Tool(String),
    Waiting(String),
    Blocked(String),
    Failed(String),
    EnvelopeExtended(String),
    CeilingInterrupted,
    Output(String),
    Completed,
    CompletedDirty,
    Canceled,
    /// "unknown event -> Evidence-only generic line" (§13.3's own last row).
    Generic(String),
}

/// Classify one non-`conversation.*` event kind per §13.3's table.
fn classify_event(kind: &str, payload: &Value) -> ThreadFact {
    if kind.starts_with("stage.") || kind == "workflow.bound" {
        return ThreadFact::Divider(kind.to_string());
    }
    match kind {
        "execution.reserved"
        | "execution.started"
        | "execution.stopped"
        | "execution.abandoned"
        | "execution.reconciled" => ThreadFact::Lifecycle(kind.to_string()),
        "tool.requested" => ThreadFact::Tool(format!("requested {}", field_text(&payload["name"]))),
        "tool.completed" => {
            let failed = payload["is_error"].as_bool().unwrap_or(false);
            ThreadFact::Tool(format!(
                "completed {}",
                if failed { "(error)" } else { "(ok)" }
            ))
        }
        "work.waiting" => ThreadFact::Waiting(field_text(&payload["reason"])),
        "work.blocked" => ThreadFact::Blocked(field_text(&payload["reason"])),
        "work.failed" => ThreadFact::Failed(field_text(&payload["reason"])),
        "execution.turn_envelope_extended" => ThreadFact::EnvelopeExtended(format!(
            "+{} turns",
            field_text(&payload["additional_turns"])
        )),
        "execution.turn_ceiling_interrupted" => ThreadFact::CeilingInterrupted,
        "work.completed" => ThreadFact::Completed,
        "work.canceled" => ThreadFact::Canceled,
        other => ThreadFact::Generic(other.to_string()),
    }
}

fn thread_item(row: &ThreadRow) -> ListItem<'static> {
    let ts = Span::styled(
        format!("{:<20} ", row.ts),
        Style::default().fg(Token::Muted.rgb()),
    );
    let label_style = Style::default().fg(Token::Muted.rgb());
    let (label, text, style) = match &row.fact {
        ThreadFact::Intent(text) => (
            "intent",
            text.clone(),
            Style::default()
                .fg(Token::Text.rgb())
                .add_modifier(Modifier::BOLD),
        ),
        ThreadFact::Human(text) => (
            "human",
            text.clone(),
            Style::default().fg(Token::Text.rgb()),
        ),
        ThreadFact::Assistant(text) => (
            "assistant",
            text.clone(),
            Style::default().fg(Token::Text.rgb()),
        ),
        ThreadFact::Ask(text) => (
            "ask",
            text.clone(),
            Style::default()
                .fg(Token::Attention.rgb())
                .add_modifier(Modifier::BOLD),
        ),
        ThreadFact::Divider(kind) => ("·", kind.clone(), Style::default().fg(Token::Muted.rgb())),
        ThreadFact::Lifecycle(kind) => (
            "exec",
            kind.clone(),
            Style::default().fg(Token::Muted.rgb()),
        ),
        ThreadFact::Tool(text) => ("tool", text.clone(), Style::default().fg(Token::Text.rgb())),
        ThreadFact::Waiting(reason) => (
            "waiting",
            reason.clone(),
            Style::default().fg(Token::Attention.rgb()),
        ),
        ThreadFact::Blocked(reason) => (
            "blocked",
            reason.clone(),
            Style::default().fg(Token::Warning.rgb()),
        ),
        ThreadFact::Failed(reason) => (
            "failed",
            reason.clone(),
            Style::default().fg(Token::Danger.rgb()),
        ),
        ThreadFact::EnvelopeExtended(text) => (
            "envelope",
            text.clone(),
            Style::default().fg(Token::Warning.rgb()),
        ),
        ThreadFact::CeilingInterrupted => (
            "ceiling",
            "turn ceiling interrupted".to_string(),
            Style::default().fg(Token::Warning.rgb()),
        ),
        ThreadFact::Output(text) => (
            "output",
            text.clone(),
            Style::default().fg(Token::Success.rgb()),
        ),
        ThreadFact::Completed => (
            "completed",
            "work completed".to_string(),
            Style::default().fg(Token::Success.rgb()),
        ),
        ThreadFact::CompletedDirty => (
            "completed",
            "output needs review".to_string(),
            Style::default().fg(Token::Attention.rgb()),
        ),
        ThreadFact::Canceled => (
            "canceled",
            "work canceled".to_string(),
            Style::default().fg(Token::Muted.rgb()),
        ),
        ThreadFact::Generic(kind) => (
            "evidence",
            format!("{kind} — see Evidence"),
            Style::default().fg(Token::Muted.rgb()),
        ),
    };
    ListItem::new(Line::from(vec![
        ts,
        Span::styled(format!("{label:<10} "), label_style),
        Span::styled(text, style),
    ]))
}

fn evidence_item(event: &Value) -> ListItem<'static> {
    let seq = event["seq"].as_u64().unwrap_or(0);
    let ts = field_text(&event["timestamp"]);
    let kind = field_text(&event["kind"]);
    let source = format!(
        "{}:{}",
        field_text(&event["source"]["type"]),
        field_text(&event["source"]["name"])
    );
    let mut meta = Vec::new();
    if let Some(x) = event["execution_id"].as_str() {
        meta.push(format!("exec={x}"));
    }
    if let Some(x) = event["correlation_id"].as_str() {
        meta.push(format!("corr={x}"));
    }
    if let Some(x) = event["causation_id"].as_str() {
        meta.push(format!("cause={x}"));
    }
    let header = format!(
        "#{seq:<6} {ts:<24} {kind:<32} {source}{}",
        if meta.is_empty() {
            String::new()
        } else {
            format!("  {}", meta.join(" "))
        }
    );
    let payload = truncate(&event["payload"].to_string(), 160);
    ListItem::new(vec![
        Line::from(Span::styled(header, Style::default().fg(Token::Text.rgb()))),
        Line::from(Span::styled(
            format!("        {payload}"),
            Style::default().fg(Token::Muted.rgb()),
        )),
    ])
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let head: String = text.chars().take(limit).collect();
    format!("{head}…")
}

/// §13.10's display-only action matrix: the ordinary-text treatment and the
/// action set the daemon would accept for the current reported state.
/// Nothing here calls the API — T1c wires these verbs up.
fn actions_for_state(state: &str) -> (&'static str, &'static [&'static str]) {
    match state {
        "pending" | "active" => ("disabled", &["cancel", "inspect"]),
        "waiting" => ("disabled", &["retry", "cancel", "inspect"]),
        "needs_input" => ("answer", &["respond", "cancel", "inspect"]),
        "blocked" => ("disabled", &["retry", "extend", "cancel", "inspect"]),
        "failed" => ("disabled", &["retry", "cancel", "inspect"]),
        "completed" => ("disabled", &["inspect"]),
        "completed_dirty" => ("disabled", &["inspect", "reap"]),
        "canceled" => ("disabled", &["inspect"]),
        _ => ("disabled", &["inspect"]),
    }
}

// -------------------------------------------------------------- rendering

/// §13.1's header, ending with the display-only action matrix (§13.10) and
/// (when a background refresh has failed) the last error.
fn header_lines(screen: &WorkScreen, live: Live) -> Vec<Line<'static>> {
    let w = &screen.work["work"];
    let state = field_text(&w["state"]);
    let intent = field_text(&w["intent"]);
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{} ", theme::state_glyph(&state)),
            Style::default().fg(theme::state_token(&state).rgb()),
        ),
        Span::styled(
            state.clone(),
            Style::default()
                .fg(theme::state_token(&state).rgb())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::raw(intent),
    ])];

    let stage = &screen.work["stage"];
    let mut meta = Vec::new();
    if let Some(name) = screen.work["workflow"]["name"].as_str() {
        let version = field_text(&screen.work["workflow"]["version"]);
        meta.push(Span::styled(
            format!("workflow {name} v{version}"),
            Style::default().fg(Token::Info.rgb()),
        ));
    }
    if !stage.is_null() {
        meta.push(Span::raw(format!("  stage {}", stage_label(stage))));
        if let Some(attempt) = stage["attempt"].as_u64() {
            meta.push(Span::raw(format!("  attempt {attempt}")));
        }
    }
    if !meta.is_empty() {
        lines.push(Line::from(meta));
    }

    if let Some(harness) = stage["executor"]["harness"].as_str() {
        let mut executor_line = format!("executor {harness}");
        if let Some(profile) = stage["executor"]["profile"]["name"].as_str() {
            executor_line.push_str(&format!(" ({profile})"));
        }
        lines.push(Line::from(Span::styled(
            executor_line,
            Style::default().fg(Token::Muted.rgb()),
        )));
    }

    let target = match w["workspace"].as_str() {
        Some(workspace) => workspace.to_string(),
        None => join_str_array(&w["repositories"]),
    };
    lines.push(Line::from(Span::styled(
        format!("target {target}"),
        Style::default().fg(Token::Muted.rgb()),
    )));

    let env = &screen.work["envelope"];
    let bonus = match env["turn_cap_bonus"].as_u64() {
        Some(bonus) if bonus > 0 => format!(" (+{bonus} bonus)"),
        _ => String::new(),
    };
    lines.push(Line::from(Span::styled(
        format!(
            "turns {}/{}{bonus}  ceiling {}s",
            field_text(&env["turns_spawned"]),
            field_text(&env["turn_cap"]),
            field_text(&env["turn_ceiling_secs"]),
        ),
        Style::default().fg(Token::Muted.rgb()),
    )));

    lines.push(Line::from(Span::styled(
        format!("connection {}", live.label()),
        Style::default().fg(Token::Muted.rgb()),
    )));

    if state == "needs_input" {
        lines.push(Line::from(Span::styled(
            format!("? {}", field_text(&stage["detail"])),
            Style::default()
                .fg(Token::Attention.rgb())
                .add_modifier(Modifier::BOLD),
        )));
    }

    let (ordinary, actions) = actions_for_state(&state);
    lines.push(Line::from(Span::styled(
        format!(
            "actions ({ordinary}, T1c not wired): {}",
            actions.join(" · ")
        ),
        Style::default().fg(Token::Muted.rgb()),
    )));

    if let Some(error) = &screen.last_error {
        lines.push(Line::from(Span::styled(
            format!("last refresh failed: {error}"),
            Style::default().fg(Token::Danger.rgb()),
        )));
    }

    lines
}

fn tabs_line(active: Tab) -> Paragraph<'static> {
    let spans: Vec<Span> = Tab::ALL
        .iter()
        .map(|tab| {
            let style = if *tab == active {
                Style::default()
                    .fg(Token::Focus.rgb())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Token::Muted.rgb())
            };
            Span::styled(format!("{}  ", tab.label()), style)
        })
        .collect();
    Paragraph::new(Line::from(spans))
}

/// Draw the canonical Work surface (§13). `None` means the fetch that opens
/// a Work is still in flight.
pub fn render(frame: &mut Frame, area: Rect, screen: Option<&WorkScreen>, live: Live) {
    let Some(screen) = screen else {
        render_loading(frame, area);
        return;
    };
    let block = Block::bordered()
        .title(format!("Work — {}", screen.id))
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let header = header_lines(screen, live);
    let header_height = header.len() as u16;
    let [header_area, tabs_area, body_area] = Layout::vertical([
        Constraint::Length(header_height),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas(inner);
    frame.render_widget(
        Paragraph::new(header).wrap(Wrap { trim: false }),
        header_area,
    );
    frame.render_widget(tabs_line(screen.tab), tabs_area);

    match screen.tab {
        Tab::Thread => render_thread(frame, body_area, screen),
        Tab::Workflow => render_workflow(frame, body_area, screen),
        Tab::Evidence => render_evidence(frame, body_area, screen),
        Tab::Graph => render_graph(frame, body_area, screen),
        Tab::Details => render_details(frame, body_area, screen),
    }
}

fn render_loading(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("Work")
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new("Loading…"), inner);
}

fn render_thread(frame: &mut Frame, area: Rect, screen: &WorkScreen) {
    let rows = screen.thread_rows();
    if rows.is_empty() {
        frame.render_widget(Paragraph::new("Nothing journaled yet."), area);
        return;
    }
    let items: Vec<ListItem> = rows.iter().map(thread_item).collect();
    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(screen.thread_selected.min(rows.len() - 1)));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_workflow(frame: &mut Frame, area: Rect, screen: &WorkScreen) {
    frame.render_widget(
        Paragraph::new(screen.workflow_lines()).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_evidence(frame: &mut Frame, area: Rect, screen: &WorkScreen) {
    let [filter_area, list_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(area);
    let filter_text = if screen.evidence_filter_focus {
        format!("filter> {}_", screen.evidence_filter)
    } else {
        format!(
            "filter: {}  window {} events  ('/' filter · 'x' clear · 'o' load older)",
            if screen.evidence_filter.is_empty() {
                "-"
            } else {
                &screen.evidence_filter
            },
            screen.evidence_limit,
        )
    };
    frame.render_widget(
        Paragraph::new(filter_text).style(Style::default().fg(Token::Muted.rgb())),
        filter_area,
    );

    let visible = screen.visible_events();
    if visible.is_empty() {
        let msg = if screen.events.is_empty() {
            "No events yet."
        } else {
            "No events match the current filter — 'x' clears it."
        };
        frame.render_widget(Paragraph::new(msg), list_area);
        return;
    }
    let items: Vec<ListItem> = visible.iter().map(|event| evidence_item(event)).collect();
    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(screen.evidence_selected.min(visible.len() - 1)));
    frame.render_stateful_widget(list, list_area, &mut state);
}

fn render_graph(frame: &mut Frame, area: Rect, screen: &WorkScreen) {
    let lines = screen.graph_lines();
    let selected = screen.graph_selected.min(lines.len().saturating_sub(1));
    let items: Vec<ListItem> = lines.into_iter().map(ListItem::new).collect();
    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    let mut state = ListState::default();
    state.select(Some(selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn render_details(frame: &mut Frame, area: Rect, screen: &WorkScreen) {
    frame.render_widget(
        Paragraph::new(screen.details_lines()).wrap(Wrap { trim: false }),
        area,
    );
}

/// Fleet's Wide contextual rail preview (§10.1): a lightweight summary of
/// the selected row, built entirely from what Fleet already has loaded — no
/// per-Work fetch of its own, and never the canonical Work surface itself
/// (Decision T2-48). Enter opens the real thing.
pub fn render_preview(frame: &mut Frame, area: Rect, row: Option<&WorkRow>) {
    let title = match row {
        Some(row) => format!("Work — {}", row.id),
        None => "Work".to_string(),
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Token::Border.rgb()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let body = match row {
        Some(row) => {
            let mut body = format!(
                "{}\n\n{}\n\nstate      {} {}\nstage      {}\nworkflow   {}\n",
                row.id,
                row.intent,
                theme::state_glyph(&row.state),
                row.state,
                row.stage,
                row.workflow,
            );
            if row.question != "-" {
                body.push_str(&format!("question   {}\n", row.question));
            }
            body.push_str(
                "\nEnter opens the full Work surface (Thread/Workflow/Evidence/Graph/Details).",
            );
            body
        }
        None => "No Work selected.".to_string(),
    };
    frame.render_widget(Paragraph::new(body).wrap(Wrap { trim: false }), inner);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn work_body(state: &str) -> Value {
        json!({
            "work": {
                "id": "01WORK",
                "intent": "fix the thing",
                "state": state,
                "repositories": ["svc-a"],
                "workspace": Value::Null,
                "created_by": "cli",
                "created_at": "2026-08-16T00:00:00Z",
                "origin_client": "cli",
            },
            "stage": {
                "stage_id": "10-implement",
                "index": 1,
                "attempt": 2,
                "status": "active",
                "detail": "which retry budget?",
                "of": 3,
                "executor": {"stage_id": "10-implement", "index": 1, "kind": "actor", "harness": "claude", "route_source": "stage_harness", "profile": {"name": "default"}},
            },
            "workflow": {"name": "software-change", "version": "1", "source": "embedded", "stages": ["00-plan", "10-implement", "20-verify"]},
            "route_source": "stage_harness",
            "reservation": Value::Null,
            "execution": {"execution_id": "exec-1", "backend": "claude", "native_id": "sess-1", "stage_id": "10-implement", "attempt": 2, "stop_requested": false},
            "surface": {"work_id": "01WORK", "root": "/data/surfaces/01WORK", "bindings": [
                {"repository": "svc-a", "source_path": "/repos/svc-a", "base_branch": "main", "base_sha": "abc123", "worktree_path": "/data/surfaces/01WORK/svc-a", "work_branch": "work/01WORK", "head_sha": "def456", "origin": "cut"},
            ]},
            "teardown": Value::Null,
            "output": Value::Null,
            "envelope": {"turns_spawned": 3, "turn_cap": 10, "turn_cap_bonus": 0, "turn_ceiling_secs": 300.0},
        })
    }

    fn screen_with(state: &str, events: Vec<Value>) -> WorkScreen {
        WorkScreen::from_parts(
            "01WORK".to_string(),
            work_body(state),
            vec![
                json!({"seq": 2, "ts": "2026-08-16T00:00:01Z", "role": "user", "text": "please fix it", "source": "event"}),
                json!({"seq": 4, "ts": "2026-08-16T00:00:03Z", "role": "assistant", "text": "on it", "source": "event"}),
                json!({"seq": 5, "ts": "2026-08-16T00:00:04Z", "role": "ask", "text": "which retry budget?", "source": "event"}),
            ],
            events,
            None,
        )
    }

    #[test]
    fn header_shows_the_13_1_facts_and_pins_the_needs_input_question() {
        let screen = screen_with("needs_input", Vec::new());
        let lines: Vec<String> = header_lines(&screen, Live::Attached)
            .iter()
            .map(|l| l.to_string())
            .collect();
        let text = lines.join("\n");
        assert!(text.contains("needs_input"), "state: {text}");
        assert!(text.contains("fix the thing"), "intent: {text}");
        assert!(
            text.contains("software-change v1"),
            "pinned workflow: {text}"
        );
        assert!(text.contains("10-implement"), "current stage: {text}");
        assert!(text.contains("attempt"), "attempt: missing from {text}");
        assert!(text.contains("claude"), "stage executor: {text}");
        assert!(text.contains("svc-a"), "target repos: {text}");
        assert!(text.contains("turns 3/10"), "turn envelope: {text}");
        assert!(text.contains("connection live"), "connection truth: {text}");
        assert!(
            text.contains("? which retry budget?"),
            "needs_input pins its question above the fold: {text}"
        );
        assert!(
            text.contains("respond"),
            "action matrix names respond: {text}"
        );
    }

    #[test]
    fn full_ids_stay_out_of_the_header() {
        let screen = screen_with("active", Vec::new());
        let text: String = header_lines(&screen, Live::Attached)
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            !text.contains("exec-1") && !text.contains("sess-1"),
            "full execution/native ids belong to Details, not the header: {text}"
        );
    }

    #[test]
    fn thread_merges_transcript_and_events_by_seq_intent_first() {
        let screen = screen_with(
            "active",
            vec![
                json!({"seq": 1, "timestamp": "2026-08-16T00:00:00Z", "kind": "stage.entered", "source": {"type": "daemon", "name": "engine"}, "payload": {"stage_id": "00-plan"}}),
                json!({"seq": 3, "timestamp": "2026-08-16T00:00:02Z", "kind": "tool.requested", "source": {"type": "backend", "name": "claude"}, "payload": {"name": "Bash"}}),
            ],
        );
        let rows = screen.thread_rows();
        assert!(
            matches!(rows[0].fact, ThreadFact::Intent(_)),
            "intent leads"
        );
        let seqs: Vec<u64> = rows.iter().map(|r| r.seq).collect();
        let mut sorted = seqs.clone();
        sorted.sort();
        assert_eq!(seqs, sorted, "rows are in journal-sequence order");
        assert!(
            rows.iter()
                .any(|r| matches!(&r.fact, ThreadFact::Ask(t) if t == "which retry budget?")),
            "an actor-authored ask becomes a gold card"
        );
        assert!(
            rows.iter()
                .any(|r| matches!(&r.fact, ThreadFact::Divider(k) if k == "stage.entered")),
            "a stage transition becomes a compact divider"
        );
        assert!(
            rows.iter()
                .any(|r| matches!(&r.fact, ThreadFact::Tool(t) if t.contains("Bash"))),
            "a tool request becomes a tool row"
        );
    }

    #[test]
    fn an_unknown_event_falls_back_to_an_evidence_only_line() {
        let screen = screen_with(
            "active",
            vec![
                json!({"seq": 9, "timestamp": "t", "kind": "usage.updated", "source": {"type": "backend", "name": "claude"}, "payload": {}}),
            ],
        );
        let rows = screen.thread_rows();
        assert!(
            rows.iter()
                .any(|r| matches!(&r.fact, ThreadFact::Generic(k) if k == "usage.updated")),
            "an unrecognized kind must not be guessed at, only marked Evidence-only"
        );
    }

    #[test]
    fn completed_is_relabeled_output_needs_review_when_the_work_is_completed_dirty() {
        let screen = screen_with(
            "completed_dirty",
            vec![
                json!({"seq": 9, "timestamp": "t", "kind": "work.completed", "source": {"type": "daemon", "name": "engine"}, "payload": {}}),
            ],
        );
        let rows = screen.thread_rows();
        assert!(
            rows.iter()
                .any(|r| matches!(r.fact, ThreadFact::CompletedDirty)),
            "completed_dirty must never read as a plain success"
        );
        assert!(!rows.iter().any(|r| matches!(r.fact, ThreadFact::Completed)));
    }

    #[test]
    fn thread_never_shows_chain_of_thought_or_guessed_progress() {
        // Decision T2-49: every row traces to a journal-backed fact. This
        // pins the negative half — nothing about "thinking" or a percent
        // complete is ever synthesized by the classifier.
        let screen = screen_with("active", Vec::new());
        let text: String = screen
            .thread_rows()
            .iter()
            .map(|r| match &r.fact {
                ThreadFact::Intent(t)
                | ThreadFact::Human(t)
                | ThreadFact::Assistant(t)
                | ThreadFact::Ask(t) => t.clone(),
                _ => String::new(),
            })
            .collect();
        assert!(!text.to_lowercase().contains("thinking"));
        assert!(!text.contains('%'));
    }

    #[test]
    fn workflow_rail_marks_done_current_and_not_entered() {
        let screen = screen_with("active", Vec::new());
        let lines: Vec<String> = screen
            .workflow_lines()
            .iter()
            .map(|l| l.to_string())
            .collect();
        assert_eq!(lines.len(), 3);
        assert!(
            lines[0].starts_with('✓'),
            "earlier stage is done: {lines:?}"
        );
        assert!(
            lines[1].starts_with('⠹'),
            "current stage is in progress: {lines:?}"
        );
        assert!(
            lines[1].contains("attempt 2"),
            "attempt number shown: {lines:?}"
        );
        assert!(
            lines[2].starts_with('·'),
            "future stage is not entered: {lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.contains('%')),
            "Decision T2-50: never a percent bar: {lines:?}"
        );
    }

    #[test]
    fn evidence_filter_matches_kind_source_or_payload_case_insensitively() {
        let screen = screen_with(
            "active",
            vec![
                json!({"seq": 1, "timestamp": "t", "kind": "tool.requested", "source": {"type": "backend", "name": "claude"}, "payload": {"name": "Bash"}}),
                json!({"seq": 2, "timestamp": "t", "kind": "stage.entered", "source": {"type": "daemon", "name": "engine"}, "payload": {"stage_id": "00-plan"}}),
            ],
        );
        let mut filtered = screen.clone();
        filtered.evidence_filter = "BASH".to_string();
        assert_eq!(filtered.visible_events().len(), 1);
        assert_eq!(filtered.visible_events()[0]["kind"], "tool.requested");
    }

    #[test]
    fn load_older_grows_the_window_and_is_bounded() {
        let mut screen = screen_with("active", Vec::new());
        assert_eq!(screen.evidence_limit, DEFAULT_EVIDENCE_LIMIT);
        for _ in 0..20 {
            screen.evidence_limit =
                (screen.evidence_limit + EVIDENCE_LIMIT_STEP).min(EVIDENCE_LIMIT_MAX);
        }
        assert_eq!(
            screen.evidence_limit, EVIDENCE_LIMIT_MAX,
            "bounded, never unbounded growth"
        );
    }

    #[test]
    fn graph_lines_are_honest_about_missing_data() {
        let screen = screen_with("active", Vec::new());
        let lines = screen.graph_lines();
        assert!(lines[0].to_lowercase().contains("unavailable"));
        assert!(
            !lines
                .iter()
                .any(|l| l.contains("File") || l.contains("Artifact") || l.contains("Commit")),
            "Decision T2-51: no node kinds the response never supplied"
        );
    }

    #[test]
    fn graph_walks_a_real_neighborhood_forward_and_reverse() {
        let mut screen = screen_with("active", Vec::new());
        screen.graph = Some(json!({
            "nodes": [
                {"node_id": "work:01WORK", "kind": "work", "label": "fix the thing", "work_id": "01WORK", "source_seq": 1},
                {"node_id": "repository:svc-a", "kind": "repository", "label": "svc-a", "work_id": Value::Null, "source_seq": 2},
                {"node_id": "execution:e1", "kind": "execution", "label": "e1", "work_id": "01WORK", "source_seq": 3},
                {"node_id": "backend:claude", "kind": "backend", "label": "claude", "work_id": Value::Null, "source_seq": 3},
            ],
            "edges": [
                {"edge_id": "targets|work:01WORK|repository:svc-a", "relation": "targets", "from_node": "work:01WORK", "to_node": "repository:svc-a", "source_seq": 2},
                {"edge_id": "executes|execution:e1|work:01WORK", "relation": "executes", "from_node": "execution:e1", "to_node": "work:01WORK", "source_seq": 3},
                {"edge_id": "uses|execution:e1|backend:claude", "relation": "uses", "from_node": "execution:e1", "to_node": "backend:claude", "source_seq": 3},
            ],
        }));
        let lines = screen.graph_lines();
        let text = lines.join("\n");
        assert!(text.contains("targets -> svc-a"));
        assert!(
            text.contains("executes <- e1"),
            "a reverse-pointing edge is still shown: {text}"
        );
    }

    #[test]
    fn details_names_the_gap_when_a_field_the_proposal_lists_is_not_in_the_response() {
        let screen = screen_with("active", Vec::new());
        let text: String = screen
            .details_lines()
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            text.contains("not read here"),
            "retained state names the gap: {text}"
        );
    }

    #[test]
    fn details_output_section_has_exactly_13_8s_fields() {
        let mut work = work_body("completed");
        work["output"] = json!({
            "work_id": "01WORK",
            "clean": true,
            "repositories": [{"repository": "svc-a", "source_repo": "/repos/svc-a", "retained_branch": "work/01WORK", "finalize_commit": "abc123", "worktree_path": "/data/surfaces/01WORK/svc-a", "disposition": "removed"}],
        });
        let screen =
            WorkScreen::from_parts("01WORK".to_string(), work, Vec::new(), Vec::new(), None);
        let text: String = screen
            .details_lines()
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        for expected in ["svc-a", "/repos/svc-a", "work/01WORK", "abc123", "removed"] {
            assert!(
                text.contains(expected),
                "{expected} missing from output section: {text}"
            );
        }
        assert!(!text.contains("diff"), "§13.8: no file list or diff");
    }

    #[test]
    fn action_matrix_matches_13_10_exactly() {
        assert_eq!(
            actions_for_state("pending"),
            ("disabled", &["cancel", "inspect"][..])
        );
        assert_eq!(
            actions_for_state("waiting"),
            ("disabled", &["retry", "cancel", "inspect"][..])
        );
        assert_eq!(
            actions_for_state("needs_input"),
            ("answer", &["respond", "cancel", "inspect"][..])
        );
        assert_eq!(
            actions_for_state("blocked"),
            ("disabled", &["retry", "extend", "cancel", "inspect"][..])
        );
        assert_eq!(
            actions_for_state("completed"),
            ("disabled", &["inspect"][..])
        );
        assert_eq!(
            actions_for_state("completed_dirty"),
            ("disabled", &["inspect", "reap"][..])
        );
        assert_eq!(
            actions_for_state("canceled"),
            ("disabled", &["inspect"][..])
        );
    }

    #[test]
    fn tab_and_back_tab_cycle_through_all_five_views() {
        let mut screen = screen_with("active", Vec::new());
        assert_eq!(screen.tab, Tab::Thread);
        screen.on_key(KeyCode::Tab);
        assert_eq!(screen.tab, Tab::Workflow);
        screen.on_key(KeyCode::Tab);
        assert_eq!(screen.tab, Tab::Evidence);
        screen.on_key(KeyCode::BackTab);
        assert_eq!(screen.tab, Tab::Workflow);
    }

    #[test]
    fn digit_keys_jump_directly_to_a_tab() {
        let mut screen = screen_with("active", Vec::new());
        screen.on_key(KeyCode::Char('4'));
        assert_eq!(screen.tab, Tab::Graph);
        screen.on_key(KeyCode::Char('1'));
        assert_eq!(screen.tab, Tab::Thread);
    }

    #[test]
    fn esc_or_q_closes_the_surface() {
        let mut screen = screen_with("active", Vec::new());
        assert_eq!(screen.on_key(KeyCode::Esc), WorkScreenOutcome::Close);
        assert_eq!(screen.on_key(KeyCode::Char('q')), WorkScreenOutcome::Close);
    }

    #[test]
    fn evidence_filter_focus_intercepts_every_key_including_esc_and_q() {
        let mut screen = screen_with("active", Vec::new());
        screen.tab = Tab::Evidence;
        assert_eq!(screen.on_key(KeyCode::Char('/')), WorkScreenOutcome::None);
        assert!(screen.evidence_filter_focus);
        for c in "bash".chars() {
            screen.on_key(KeyCode::Char(c));
        }
        assert_eq!(screen.evidence_filter, "bash");
        assert_eq!(
            screen.on_key(KeyCode::Char('q')),
            WorkScreenOutcome::None,
            "typing 'q' into the filter must not close the surface"
        );
        assert_eq!(screen.evidence_filter, "bashq");
        screen.on_key(KeyCode::Esc);
        assert!(
            !screen.evidence_filter_focus,
            "Esc leaves the filter, not the surface"
        );
    }

    #[test]
    fn o_on_evidence_asks_to_load_older() {
        let mut screen = screen_with("active", Vec::new());
        screen.tab = Tab::Evidence;
        assert_eq!(
            screen.on_key(KeyCode::Char('o')),
            WorkScreenOutcome::LoadOlder
        );
    }
}
