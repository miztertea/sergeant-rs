//! The embedded HTML dashboard (proposal §29).
//!
//! The daemon serves its own dashboard: server-rendered HTML, assets compiled
//! into the binary with `include_str!`, a few dozen lines of vanilla
//! JavaScript, and `EventSource` against the API's existing SSE endpoint. No
//! Node, no CDN, no template engine — the pages are built by string
//! formatting, which is what §34's "embedded server-rendered assets" asks for
//! and what a projection of already-shaped JSON actually needs.
//!
//! **This module has no access to daemon internals, and that is enforced by
//! the compiler, not by discipline.** Its only state is [`ApiViews`], whose
//! inner `ApiState` is private: every value rendered below is exactly a `/v1`
//! response body, produced by the same function the corresponding endpoint
//! uses. §30's rule — "if the TUI needs a private shortcut, the API is
//! incomplete" — is the dashboard's rule too, and the way to add a field to a
//! screen here is to add it to the API first.
//!
//! Auth is the same bearer token as everything else, presented either as a
//! header or — because a browser navigating to a URL can set no headers — as
//! `?token=`. The tradeoff and its bound (safe methods only) are documented on
//! [`crate::api::TOKEN_QUERY_PARAM`]'s reader, and **this module decides no
//! part of it**: the daemon layers [`crate::api`]'s own bearer gate over these
//! routes, exactly as it does over `/v1`, so an unauthorized request never
//! reaches a handler here. What the pages still need is the token's *value*,
//! to put it back on the links and asset URLs a browser will follow — and
//! they read it with the API's extractor, not a second copy of the rule.

use std::fmt::Write as _;

use axum::Router;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use serde_json::Value;

// The view helpers below are the API's, not this module's: `text` is the
// shared `-`-for-missing rule and `stage_label` the shared stage coordinate,
// so the dashboard and the TUI cannot tell different stories about the same
// field, `urlencode` is the client's own escaping rule rather than a second
// copy of it, and `presented_token` is the one bearer-extraction rule this
// listener has.
use crate::api::{
    ApiViews, SSE_EVENT_KINDS, TOKEN_QUERY_PARAM, field_text as text, presented_token, stage_label,
    urlencode,
};

/// Dashboard stylesheet, compiled into the binary (§29: no CDN).
pub const DASHBOARD_CSS: &str = include_str!("../web/dashboard.css");
/// Dashboard script, compiled into the binary (§29: no Node).
pub const DASHBOARD_JS: &str = include_str!("../web/dashboard.js");

/// Path prefix the dashboard is mounted at.
pub const DASHBOARD_ROOT: &str = "/ui";

/// How many of a work's most recent events the detail page renders.
pub const DETAIL_EVENT_LIMIT: usize = 200;

/// The dashboard's routes, ready to be merged into the daemon's router.
pub fn routes(views: ApiViews) -> Router {
    Router::new()
        .route(DASHBOARD_ROOT, get(fleet_page))
        .route("/ui/work/{id}", get(work_page))
        .route("/ui/assets/dashboard.css", get(stylesheet))
        .route("/ui/assets/dashboard.js", get(script))
        .with_state(views)
}

/// The token this request presented — the *value*, not a verdict.
///
/// Authorization is already settled by the time a handler here runs: the
/// daemon layers [`crate::api`]'s bearer gate over these routes. What the
/// pages need is the token itself, because every link, asset URL and
/// `EventSource` the browser follows next has to carry it. Reading it with
/// [`presented_token`] rather than a local copy is the point: the copy this
/// module used to keep accepted a query token on any method, so the listener
/// had two extraction rules that disagreed about the safe-method bound.
///
/// The rejection is unreachable behind the gate — it exists because an
/// extractor must return something, and it returns the one 401 this daemon
/// has rather than inventing a second.
struct PageToken(String);

impl<S: Send + Sync> FromRequestParts<S> for PageToken {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        presented_token(&parts.method, &parts.headers, parts.uri.query())
            .map(PageToken)
            .ok_or_else(crate::api::unauthorized_response)
    }
}

/// `GET /ui` — the Fleet screen.
async fn fleet_page(State(views): State<ApiViews>, PageToken(token): PageToken) -> Response {
    let system = views.system().await;
    let fleet = views.fleet().await;
    Html(render_fleet(&system, &fleet, &token)).into_response()
}

/// `GET /ui/work/{id}` — the Work detail screen.
async fn work_page(
    State(views): State<ApiViews>,
    Path(id): Path<String>,
    PageToken(token): PageToken,
) -> Response {
    let system = views.system().await;
    let Some(detail) = views.work(&id).await else {
        return (
            StatusCode::NOT_FOUND,
            Html(render_missing_work(&system, &id, &token)),
        )
            .into_response();
    };
    // A journal the daemon cannot read is not a work with no history. The
    // `/v1/events` endpoint answers 500 here, so this page does too — a
    // dashboard that quietly rendered "no transitions recorded" over a read
    // failure would be telling a different truth than the HTTP client looking
    // at the same daemon (§7: clients are equal).
    let events = match views.work_events(&id, DETAIL_EVENT_LIMIT).await {
        Ok(events) => events,
        Err(error) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(render_read_failure(&system, &id, &error, &token)),
            )
                .into_response();
        }
    };
    Html(render_work(&system, &id, &detail, &events, &token)).into_response()
}

/// `GET /ui/assets/dashboard.css`.
///
/// Assets are authenticated like the pages that reference them: the token
/// travels on the `<link>`/`<script>` URL, so a stylesheet is not a hole in
/// the same listener's auth.
async fn stylesheet() -> Response {
    asset("text/css; charset=utf-8", DASHBOARD_CSS)
}

/// `GET /ui/assets/dashboard.js`.
async fn script() -> Response {
    asset("text/javascript; charset=utf-8", DASHBOARD_JS)
}

fn asset(content_type: &'static str, body: &'static str) -> Response {
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

// ---------------------------------------------------------------- rendering
//
// Everything below is a pure function of `/v1` response bodies. Keeping it
// pure is what makes the screens testable without a browser and what keeps
// this module honest: there is nothing here to read state *from*.

/// Escape text for HTML text nodes and quoted attribute values.
fn esc(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// The shared page chrome: head, top bar, live indicator, ticker, script.
///
/// `data-kinds` carries the SSE stream's own vocabulary
/// ([`crate::api::SSE_EVENT_KINDS`]) to the page, because `EventSource` can
/// only subscribe to frame names it can enumerate and a list kept by hand in
/// the JavaScript drifts from the daemon that writes the frames. The server
/// knows the vocabulary; it says so.
fn page(system: &Value, title: &str, token: &str, work_id: Option<&str>, body: &str) -> String {
    let query = format!("{TOKEN_QUERY_PARAM}={}", urlencode(token));
    let head = system["journal_head"].as_u64().unwrap_or(0);
    let kinds = SSE_EVENT_KINDS.join(" ");
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head>\
<meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
<meta name=\"referrer\" content=\"no-referrer\">\
<title>{title} — sergeant</title>\
<link rel=\"stylesheet\" href=\"/ui/assets/dashboard.css?{query}\">\
</head><body id=\"dashboard\" data-token=\"{token_attr}\" data-from=\"{head}\" data-work=\"{work_attr}\" data-kinds=\"{kinds}\">\
<header class=\"topbar\">\
<h1><a href=\"/ui?{query}\">sergeant</a></h1>\
<span class=\"meta\">v{version} · api {api} · {data_dir}</span>\
<span class=\"live\" id=\"live\" data-state=\"connecting\">connecting…</span>\
</header>\
<main>{body}</main>\
<h2 style=\"padding:0 1.25rem\">live journal</h2>\
<ul class=\"ticker\" id=\"ticker\" style=\"padding:0 1.25rem\"></ul>\
<script src=\"/ui/assets/dashboard.js?{query}\"></script>\
</body></html>\n",
        title = esc(title),
        token_attr = esc(token),
        work_attr = esc(work_id.unwrap_or("")),
        version = esc(&text(&system["version"])),
        api = esc(&text(&system["api_revision"])),
        data_dir = esc(&text(&system["data_dir"])),
    )
}

/// The Fleet screen: every work, its §10 state, its stage coordinate.
pub fn render_fleet(system: &Value, fleet: &Value, token: &str) -> String {
    let empty = Vec::new();
    let works = fleet["works"].as_array().unwrap_or(&empty);
    let mut body = String::new();
    let _ = write!(body, "<h2>fleet — {} work items</h2>", works.len());
    if works.is_empty() {
        body.push_str("<p class=\"empty\">no work yet — <code>sgt run \"…\"</code></p>");
        return page(system, "Fleet", token, None, &body);
    }
    body.push_str(
        "<table><thead><tr>\
<th>work</th><th>state</th><th>stage</th><th>backend</th><th>workspace</th>\
<th>intent</th><th>created</th></tr></thead><tbody>",
    );
    for work in works {
        let id = text(&work["id"]);
        let state = text(&work["state"]);
        let _ = write!(
            body,
            "<tr>\
<td><a href=\"/ui/work/{href}?{TOKEN_QUERY_PARAM}={qtoken}\">{id}</a></td>\
<td class=\"state state-{state_class}\">{state}</td>\
<td>{stage}</td><td>{backend}</td><td>{workspace}</td>\
<td class=\"wrap\">{intent}</td><td>{created}</td>\
</tr>",
            href = urlencode(&id),
            qtoken = urlencode(token),
            id = esc(&id),
            state_class = esc(&state),
            state = esc(&state),
            stage = esc(&stage_label(&work["stage"])),
            backend = esc(&text(&work["resolved_backend"])),
            workspace = esc(&text(&work["workspace"])),
            intent = esc(&text(&work["intent"])),
            created = esc(&text(&work["created_at"])),
        );
    }
    body.push_str("</tbody></table>");
    page(system, "Fleet", token, None, &body)
}

/// A 500 that still looks like the dashboard, and names the fault instead of
/// pretending the work has no history.
pub fn render_read_failure(system: &Value, id: &str, error: &Value, token: &str) -> String {
    let body = format!(
        "<h2>work {id}</h2>\
<p class=\"empty\">this work's events could not be read: {message} \
(<code>{code}</code>). The journal is the durable record — run \
<code>sgt doctor</code>; nothing on this page is missing because it did not \
happen.</p>",
        id = esc(id),
        message = esc(&text(&error["error"]["message"])),
        code = esc(&text(&error["error"]["code"])),
    );
    page(system, "Work", token, None, &body)
}

/// A 404 that still looks like the dashboard.
pub fn render_missing_work(system: &Value, id: &str, token: &str) -> String {
    let body = format!(
        "<h2>work</h2><p class=\"empty\">no work with id {}</p>",
        esc(id)
    );
    page(system, "Work", token, None, &body)
}

/// The Work detail screen (§29's work page, for the fields this P0 has).
pub fn render_work(
    system: &Value,
    id: &str,
    detail: &Value,
    events: &Value,
    token: &str,
) -> String {
    let work = &detail["work"];
    let mut body = String::new();

    let state = text(&work["state"]);
    let _ = write!(
        body,
        "<h2>work {id}</h2>\
<p class=\"state state-{class}\">{state}</p>\
<h2>intent</h2><p class=\"wrap\">{intent}</p>",
        id = esc(id),
        class = esc(&state),
        state = esc(&state),
        intent = esc(&text(&work["intent"])),
    );

    // Workflow + stage.
    body.push_str("<h2>workflow / stage</h2>");
    let workflow = &detail["workflow"];
    body.push_str(&fields(&[
        ("workflow", text(&workflow["name"])),
        ("version", text(&workflow["version"])),
        ("source", text(&workflow["source"])),
        ("stages", list_text(&workflow["stages"])),
        ("current stage", stage_label(&detail["stage"])),
        ("attempt", text(&detail["stage"]["attempt"])),
        ("detail", text(&detail["stage"]["detail"])),
    ]));

    // Surface (§11 bindings).
    body.push_str("<h2>surface</h2>");
    match detail["surface"]["bindings"].as_array() {
        Some(bindings) if !bindings.is_empty() => {
            body.push_str(
                "<table><thead><tr><th>repository</th><th>worktree</th>\
<th>work branch</th><th>base</th><th>head</th></tr></thead><tbody>",
            );
            for binding in bindings {
                let _ = write!(
                    body,
                    "<tr><td>{repo}</td><td class=\"wrap\">{path}</td><td>{branch}</td>\
<td>{base}@{base_sha}</td><td>{head}</td></tr>",
                    repo = esc(&text(&binding["repository"])),
                    path = esc(&text(&binding["worktree_path"])),
                    branch = esc(&text(&binding["work_branch"])),
                    base = esc(&text(&binding["base_branch"])),
                    base_sha = esc(&short_sha(&text(&binding["base_sha"]))),
                    head = esc(&short_sha(&text(&binding["head_sha"]))),
                );
            }
            body.push_str("</tbody></table>");
        }
        _ => body.push_str("<p class=\"empty\">no repository surface bound</p>"),
    }
    if !detail["teardown"].is_null() {
        let _ = write!(
            body,
            "<p class=\"pointer\">teardown: {}</p>",
            esc(&text(&detail["teardown"]["reason"]))
        );
    }

    // Execution + native session identity (§25's separation, on screen).
    body.push_str("<h2>execution</h2>");
    let execution = &detail["execution"];
    if execution.is_null() {
        body.push_str("<p class=\"empty\">no live execution</p>");
    } else {
        body.push_str(&fields(&[
            ("execution id", text(&execution["execution_id"])),
            ("backend", text(&execution["backend"])),
            ("native session", text(&execution["native_id"])),
            ("stage", text(&execution["stage_id"])),
            ("attempt", text(&execution["attempt"])),
            ("stop requested", text(&execution["stop_requested"])),
            ("route source", text(&detail["route_source"])),
        ]));
    }

    let empty = Vec::new();
    let events = events["events"].as_array().unwrap_or(&empty);

    // Conversation and tool activity (§27's normalized events).
    body.push_str("<h2>conversation and tool activity</h2>");
    let activity: Vec<&Value> = events
        .iter()
        .filter(|e| is_activity(e["kind"].as_str().unwrap_or("")))
        .collect();
    if activity.is_empty() {
        body.push_str("<p class=\"empty\">no normalized conversation events yet</p>");
    } else {
        body.push_str(
            "<table><thead><tr><th>seq</th><th>kind</th><th>detail</th></tr></thead><tbody>",
        );
        for event in activity {
            let _ = write!(
                body,
                "<tr><td>{seq}</td><td>{kind}</td><td class=\"wrap\">{detail}</td></tr>",
                seq = esc(&text(&event["seq"])),
                kind = esc(&text(&event["kind"])),
                detail = esc(&activity_detail(event)),
            );
        }
        body.push_str("</tbody></table>");
    }

    // State transitions.
    body.push_str("<h2>state transitions</h2>");
    let transitions: Vec<&Value> = events
        .iter()
        .filter(|e| {
            let kind = e["kind"].as_str().unwrap_or("");
            kind.starts_with("work.") || kind.starts_with("stage.")
        })
        .collect();
    if transitions.is_empty() {
        body.push_str("<p class=\"empty\">no transitions recorded</p>");
    } else {
        body.push_str(
            "<table><thead><tr><th>seq</th><th>when</th><th>event</th><th>detail</th>\
</tr></thead><tbody>",
        );
        for event in transitions {
            let _ = write!(
                body,
                "<tr><td>{seq}</td><td>{when}</td><td>{kind}</td><td class=\"wrap\">{detail}</td></tr>",
                seq = esc(&text(&event["seq"])),
                when = esc(&text(&event["timestamp"])),
                kind = esc(&text(&event["kind"])),
                detail = esc(&transition_detail(event)),
            );
        }
        body.push_str("</tbody></table>");
    }

    // Usage.
    body.push_str("<h2>usage</h2>");
    let usage: Vec<&Value> = events
        .iter()
        .filter(|e| e["kind"] == "usage.updated")
        .collect();
    if usage.is_empty() {
        body.push_str("<p class=\"empty\">no usage reported by this backend</p>");
    } else {
        body.push_str(
            "<table><thead><tr><th>seq</th><th>input</th><th>output</th><th>cost usd</th>\
<th>model pin</th></tr></thead><tbody>",
        );
        for event in usage {
            let payload = &event["payload"];
            let _ = write!(
                body,
                "<tr><td>{seq}</td><td>{input}</td><td>{output}</td><td>{cost}</td><td>{pin}</td></tr>",
                seq = esc(&text(&event["seq"])),
                input = esc(&text(&payload["usage"]["input_tokens"])),
                output = esc(&text(&payload["usage"]["output_tokens"])),
                cost = esc(&text(&payload["total_cost_usd"])),
                pin = esc(&text(&payload["model_pin"]["verdict"])),
            );
        }
        body.push_str("</tbody></table>");
    }

    let _ = write!(
        body,
        "<p class=\"pointer\">graph: <code>GET /v1/graph/work/{id}</code> · \
events: <code>GET /v1/events?work_id={id}</code></p>",
        id = esc(id)
    );

    page(system, "Work", token, Some(id), &body)
}

/// Whether an event kind is §27 conversation/tool/usage activity.
fn is_activity(kind: &str) -> bool {
    kind.starts_with("conversation.") || kind.starts_with("tool.") || kind == "usage.updated"
}

/// One line of human detail for a normalized activity event.
fn activity_detail(event: &Value) -> String {
    let payload = &event["payload"];
    match event["kind"].as_str().unwrap_or("") {
        "conversation.user" | "conversation.assistant.completed" => text(&payload["text"]),
        "tool.requested" => format!(
            "{} {}",
            text(&payload["name"]),
            compact(&payload["input"], 160)
        ),
        "tool.completed" => format!(
            "{} {}",
            text(&payload["tool_use_id"]),
            compact(&payload["content"], 160)
        ),
        _ => compact(payload, 200),
    }
}

/// One line of human detail for a state-transition event.
fn transition_detail(event: &Value) -> String {
    let payload = &event["payload"];
    for key in ["reason", "prompt", "detail", "summary", "stage_id", "from"] {
        if !payload[key].is_null() {
            return text(&payload[key]);
        }
    }
    compact(payload, 160)
}

/// JSON compacted to a bounded single line.
fn compact(value: &Value, max: usize) -> String {
    if value.is_null() {
        return String::new();
    }
    let mut rendered = value.to_string();
    if rendered.chars().count() > max {
        rendered = rendered.chars().take(max).collect::<String>() + "…";
    }
    rendered
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}

fn list_text(value: &Value) -> String {
    match value.as_array() {
        Some(items) if !items.is_empty() => items.iter().map(text).collect::<Vec<_>>().join(" → "),
        _ => "-".to_string(),
    }
}

/// A definition list of label/value pairs.
fn fields(pairs: &[(&str, String)]) -> String {
    let mut out = String::from("<dl class=\"fields\">");
    for (label, value) in pairs {
        let _ = write!(
            out,
            "<dt>{}</dt><dd>{}</dd>",
            esc(label),
            esc(value.as_str())
        );
    }
    out.push_str("</dl>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn text_nodes_and_attributes_are_escaped() {
        let system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        let fleet = json!({"works": [{
            "id": "01ABC",
            "state": "pending",
            "intent": "<script>alert('x')</script>",
            "workspace": "w",
            "created_at": "2026-08-09T00:00:00.000Z",
            "stage": null,
            "resolved_backend": null,
        }]});
        let html = render_fleet(&system, &fleet, "tok\"en");
        assert!(
            !html.contains("<script>alert"),
            "an intent must never reach the page as markup: {html}"
        );
        assert!(html.contains("&lt;script&gt;alert"), "escaped form missing");
        assert!(
            !html.contains("data-token=\"tok\"en\""),
            "an attribute value must not be able to close its own quote"
        );
    }

    /// A journal that cannot be read must not look like a work with nothing
    /// in it. The endpoint answers 500 here; so does this page, and it says
    /// which fault it hit instead of showing three green empty sections.
    #[test]
    fn an_unreadable_journal_is_named_not_rendered_as_emptiness() {
        let system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        let error = json!({"error": {"code": "internal", "message": "segment 3 is torn"}});
        let html = render_read_failure(&system, "01ABC", &error, "tok");
        assert!(
            html.contains("segment 3 is torn"),
            "the fault must be named"
        );
        assert!(html.contains("01ABC"), "…on the work it happened for");
        for green in [
            "no transitions recorded",
            "no normalized conversation events yet",
            "no usage reported by this backend",
        ] {
            assert!(
                !html.contains(green),
                "a read failure must never be dressed as an empty section: {green:?}"
            );
        }
    }

    /// The `-`-for-missing rule, where it is defined for both clients.
    ///
    /// It lives in `api.rs` because the TUI and the dashboard must not tell
    /// different stories about the same absent field; it is tested here,
    /// beside the renderers that lean on it hardest.
    #[test]
    fn an_absent_field_reads_as_a_dash_and_a_present_one_as_itself() {
        assert_eq!(text(&Value::Null), "-");
        assert_eq!(text(&json!("")), "-", "an empty string is absence too");
        assert_eq!(text(&json!("fake")), "fake");
        assert_eq!(text(&json!(3)), "3", "a number reads as its JSON form");
        assert_eq!(text(&json!(false)), "false");
    }

    #[test]
    fn a_stage_reads_as_position_and_status() {
        let stage = json!({"stage_id": "10-implement", "index": 1, "of": 4, "status": "running"});
        assert_eq!(stage_label(&stage), "10-implement 2/4 · running");
        assert_eq!(stage_label(&Value::Null), "-");
    }

    /// A second, sparse fixture: every `render_work` section's "nothing
    /// here" branch, all at once. The only fixture this page is exercised
    /// against elsewhere always binds a repository surface, so the unbound,
    /// no-execution, no-events shape never rendered before this.
    #[test]
    fn render_work_renders_every_sections_empty_branch() {
        let system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        let detail = json!({
            "work": {"state": "pending", "intent": "do a thing"},
            "workflow": {"name": "wf", "version": 1, "source": "inline", "stages": []},
            "stage": null,
            "surface": {"bindings": []},
            "teardown": null,
            "execution": null,
            "route_source": null,
        });
        let events = json!({"events": []});

        let html = render_work(&system, "01ABC", &detail, &events, "tok");

        assert!(
            html.contains("no repository surface bound"),
            "an unbound work must say so rather than showing an empty table: {html}"
        );
        assert!(
            !html.contains("<th>repository</th>"),
            "the surface table must not render at all when there are no bindings: {html}"
        );
        assert!(
            !html.contains("teardown:"),
            "a work that was never torn down must not show a teardown line: {html}"
        );
        assert!(
            html.contains("no live execution"),
            "a work with no live execution must say so: {html}"
        );
        assert!(
            html.contains("no normalized conversation events yet"),
            "an empty event list must not render an empty activity table: {html}"
        );
        assert!(
            html.contains("no transitions recorded"),
            "an empty event list must not render an empty transitions table: {html}"
        );
        assert!(
            html.contains("no usage reported by this backend"),
            "an empty event list must not render an empty usage table: {html}"
        );
    }

    /// Its opposite: the fixture with something in every section.
    ///
    /// The sparse fixture above pinned the "nothing here" branches, and the
    /// live-daemon render in the m6 suite reaches this page with a torn-down
    /// surface and no §27 activity — so the activity and usage tables, and
    /// every arm of the per-kind `activity_detail` line, rendered nowhere.
    /// They are where the dashboard puts a backend's own words in front of a
    /// human, which is exactly the content that must not reach the page as
    /// markup.
    #[test]
    fn render_work_renders_activity_usage_and_teardown_when_there_is_something_to_show() {
        let system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        let detail = json!({
            "work": {"state": "completed", "intent": "do a thing"},
            "workflow": {"name": "wf", "version": 1, "source": "inline", "stages": ["00-only"]},
            "stage": {"stage_id": "00-only", "index": 0, "of": 1, "status": "completed"},
            "surface": {"bindings": []},
            "teardown": {"reason": "work completed"},
            "execution": null,
            "route_source": "global_default",
        });
        let events = json!({"events": [
            {"seq": 11, "kind": "conversation.user", "payload": {"text": "hello <there>"}},
            {"seq": 12, "kind": "conversation.assistant.completed",
             "payload": {"text": "done"}},
            {"seq": 13, "kind": "tool.requested",
             "payload": {"name": "Bash", "input": {"command": "ls"}}},
            {"seq": 14, "kind": "tool.completed",
             "payload": {"tool_use_id": "tu_1", "content": "README.md"}},
            // A kind the renderer has no special case for, and one with no
            // payload at all: both fall through to the compacted form.
            {"seq": 15, "kind": "conversation.assistant.thinking", "payload": {"tokens": 12}},
            {"seq": 16, "kind": "tool.requested", "payload": {"name": "Read"}},
            {"seq": 17, "kind": "usage.updated", "payload": {
                "usage": {"input_tokens": 120, "output_tokens": 34},
                "total_cost_usd": 0.0042,
                "model_pin": {"verdict": "honored"},
            }},
        ]});

        let html = render_work(&system, "01ABC", &detail, &events, "tok");

        assert!(
            html.contains("teardown: work completed"),
            "a torn-down surface says why: {html}"
        );
        for (needle, what) in [
            ("<th>seq</th><th>kind</th><th>detail</th>", "activity table"),
            ("conversation.user", "an activity row's kind"),
            (
                "hello &lt;there&gt;",
                "an assistant/user line's text, escaped",
            ),
            (
                "Bash {&quot;command&quot;:&quot;ls&quot;}",
                "a tool request's name and input",
            ),
            (
                "tu_1 &quot;README.md&quot;",
                "a tool result's id and content",
            ),
            (
                "{&quot;tokens&quot;:12}",
                "an unrecognised kind's compacted payload",
            ),
            ("<th>cost usd</th>", "usage table"),
            (
                "<td>120</td><td>34</td><td>0.0042</td><td>honored</td>",
                "a usage row",
            ),
        ] {
            assert!(
                html.contains(needle),
                "the {what} must render ({needle:?} not found in): {html}"
            );
        }
        assert!(
            !html.contains("<there>"),
            "a backend's words must never reach the page as markup: {html}"
        );
        assert!(
            html.contains("<td class=\"wrap\">Read </td>"),
            "a tool request with no input renders the name and nothing else, \
             not the word null: {html}"
        );
        for green in [
            "no normalized conversation events yet",
            "no usage reported by this backend",
        ] {
            assert!(
                !html.contains(green),
                "a populated section must not also claim to be empty: {green:?}"
            );
        }
    }

    /// A work id that is not there is still a dashboard page — chrome, live
    /// ticker and all — and the id it echoes back is escaped like every other
    /// text node. The 404 body is the one page rendered from a value that
    /// came straight off the URL.
    #[test]
    fn a_missing_work_is_a_dashboard_page_with_its_id_escaped() {
        let system = json!({"version": "0.1.0", "api_revision": "v1", "data_dir": "/tmp/d"});
        let html = render_missing_work(&system, "01A&B<script>", "tok");
        assert!(
            html.contains("no work with id 01A&amp;B&lt;script&gt;"),
            "the id is echoed, escaped — ampersand included: {html}"
        );
        assert!(
            !html.contains("<script>"),
            "an id off the URL must never reach the page as markup: {html}"
        );
        assert!(
            html.contains("id=\"ticker\"") && html.contains("dashboard.js"),
            "a 404 keeps the chrome, so the page is still live: {html}"
        );
    }
}
