//! T4 (`reference/proposal-tui-t-series.md` §19.10/§19.11/§20.5): the
//! geometry matrix, the Taste pre-flight's mechanical half, and this repo's
//! real (not hand-typed) TUI screenshots.
//!
//! This lives as an internal `#[cfg(test)]` module of [`super`] — not an
//! external `tests/*.rs` binary — because most of §19.10's fixtures need an
//! *open* canonical Work surface, and [`super::work_view::WorkScreen`] is
//! reached only through `work_view::WorkScreen::from_parts`
//! (`pub(crate)`) or a live API round trip (`tests/m6_surfaces.rs`'s own
//! pattern, used where a live daemon is actually the point). Building each
//! of §19.10's ~30 state/size combinations against a live daemon would make
//! this suite the slowest in the tree for no correctness gain — every
//! fixture here is pure state plus [`super::draw`], exactly §19.1's
//! "pure state tests" philosophy the rest of this tree already follows.
//!
//! §19.10's own assertion list ("no collision", "focus is visible", …) is
//! honored the same way every other TUI test in this tree already does:
//! content assertions on a flattened `TestBackend` buffer, not pixel/style
//! introspection — Decision T2-63 rejects whole-frame snapshots for the same
//! reason. Where a §19.10 property has no textual signature at all (e.g. the
//! active-destination header label, which is color/bold-only), that is
//! named in the Taste pre-flight notes below rather than silently skipped.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use serde_json::{Value, json};

use super::connection::Live;
use super::estate::EstateTab;
use super::overlay::Overlay;
use super::*;

/// §19.10's own three sizes.
const SIZES: [(u16, u16); 3] = [(80, 24), (120, 36), (180, 48)];

fn draw_at(app: &App, width: u16, height: u16) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal
        .draw(|frame| draw(frame, app))
        .expect("draw the screen");
    terminal
}

fn lines_of(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer();
    let area = buffer.area;
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer.cell((x, y)).map_or(" ", |c| c.symbol()))
                .collect::<String>()
        })
        .collect()
}

fn text_of(terminal: &Terminal<TestBackend>) -> String {
    lines_of(terminal).join("\n")
}

// -------------------------------------------------------------- fixtures

fn home_empty() -> App {
    App::new()
}

fn home_full() -> App {
    let mut app = App::new();
    app.rows = fleet::fleet_rows(&json!({"works": [
        {"id": "01NEEDS", "state": "needs_input", "intent": "waiting on an answer",
         "stage": {"detail": "which path?"}},
        {"id": "01DONE", "state": "completed", "intent": "already done"},
        {"id": "01DIRTY", "state": "completed_dirty", "intent": "messy output"},
    ]}));
    app
}

fn home_admission_paused() -> App {
    let mut app = App::new();
    app.system = json!({"admission_paused": true});
    app
}

fn fleet_all_states() -> App {
    let mut app = App::new();
    app.destination = Destination::Fleet;
    app.drawer_open = false; // Narrow's own full-body drawer takeover is a separate fixture below.
    let works: Vec<Value> = fleet::REPORTED_STATES
        .iter()
        .enumerate()
        .map(|(i, state)| {
            json!({
                "id": format!("01STATE{i}"),
                "state": state,
                "intent": format!("work in state {state}"),
                "stage": {"stage_id": "10-implement", "index": 1, "of": 2, "status": "running", "detail": "a question or reason"},
                "resolved_backend": "fake",
                "workflow": "tiny",
            })
        })
        .collect();
    app.rows = fleet::fleet_rows(&json!({"works": works}));
    app
}

/// A single row with every field near-worst-case long, so §19.10's "long
/// values remain contained" has something real to check: the Fleet table's
/// own `Fill`/`Length` constraints (not hand-padded strings, §10.1's
/// Decision T2-37) must keep every other column on screen regardless.
fn fleet_long_values() -> App {
    let mut app = App::new();
    app.destination = Destination::Fleet;
    app.drawer_open = false;
    let long = "x".repeat(400);
    app.rows = fleet::fleet_rows(&json!({"works": [{
        "id": long,
        "state": "needs_input",
        "intent": long,
        "workflow": long,
        "workspace": long,
        "stage": {"stage_id": "10-implement", "index": 1, "of": 2, "status": "running", "detail": long},
        "resolved_backend": "fake",
    }]}));
    app
}

/// An app with `id` open at `state`, the pinned `detail` naming its
/// question/reason (§9.3) — the shape every Work-state fixture below shares.
fn app_with_open_work(id: &str, state: &str, detail: &str) -> App {
    let mut app = App::new();
    app.rows = fleet::fleet_rows(&json!({
        "works": [{"id": id, "state": state, "intent": "fix the flaky thing",
                    "stage": {"stage_id": "10-implement", "detail": detail}}]
    }));
    app.destination = Destination::Fleet;
    app.open_work = Some(OpenWork {
        id: id.to_string(),
        from: Destination::Fleet,
    });
    app.work_screen = Some(work_view::WorkScreen::from_parts(
        id.to_string(),
        json!({
            "work": {"id": id, "intent": "fix the flaky thing", "state": state},
            "stage": {"stage_id": "10-implement", "attempt": 2, "detail": detail},
            "envelope": {"turn_cap": 12, "turns_spawned": 3},
            "workflow": {"name": "tiny", "version": "1"},
        }),
        Vec::new(),
        Vec::new(),
        None,
    ));
    app
}

fn workflows_catalog() -> App {
    let mut app = App::new();
    app.destination = Destination::Workflows;
    app.drawer_open = false;
    app.workflows.set_entries(vec![
        json!({"name": "implement", "version": "1", "source": "/repo/.sergeant/workflows/implement",
               "content_hash": "a".repeat(64), "status": "published", "stages": []}),
        json!({"name": "diagnose-bug", "version": "2", "source": "/repo/.sergeant/workflows/diagnose-bug",
               "content_hash": "b".repeat(64), "status": "published", "stages": []}),
    ]);
    app
}

fn workflows_error() -> App {
    let mut app = App::new();
    app.destination = Destination::Workflows;
    app.drawer_open = false;
    app.workflows.last_error = Some("catalog read failed: transport error".to_string());
    app
}

fn estate_repos() -> App {
    let mut app = App::new();
    app.destination = Destination::Estate;
    app.drawer_open = false;
    app.estate.tab = EstateTab::Repositories;
    app.estate.repos = vec![
        json!({"name": "svc-a", "path": "repos/svc-a", "origin": "git@x:svc-a", "instructions": "suppress"}),
        json!({"name": "svc-b", "path": "repos/svc-b", "origin": null, "instructions": "local"}),
    ];
    app
}

fn estate_groups() -> App {
    let mut app = App::new();
    app.destination = Destination::Estate;
    app.drawer_open = false;
    app.estate.tab = EstateTab::Groups;
    app.estate.groups = vec![json!({"name": "core", "repos": ["svc-a"], "brief": "core services"})];
    app
}

fn estate_health() -> App {
    let mut app = App::new();
    app.destination = Destination::Estate;
    app.drawer_open = false;
    app.estate.tab = EstateTab::Health;
    app.estate.doctor = json!({"checks": [
        {"name": "git", "status": "ok", "detail": "found 2.43"},
        {"name": "disk_pressure", "status": "warn", "detail": "82% full", "remedy": "free up space"},
    ]});
    // The remedy only shows on the Detail pane for the *selected* check
    // (§12.3) — select disk_pressure so its remedy is actually on screen.
    app.estate.check_selected = 1;
    app
}

// ------------------------------------------------------------------ tests

#[test]
fn geometry_home_empty_full_and_admission_paused() {
    // The drawer's own Narrow full-body takeover is a separate, dedicated
    // fixture below (`geometry_home_full_narrow_drawer_replaces_the_body_not_alongside_it`)
    // — closed here so this test is purely about the form itself.
    for (name, mut app) in [
        ("empty", home_empty()),
        ("full", home_full()),
        ("admission-paused", home_admission_paused()),
    ] {
        app.drawer_open = false;
        for (w, h) in SIZES {
            let terminal = draw_at(&app, w, h);
            let lines = lines_of(&terminal);
            assert_eq!(
                lines.len(),
                h as usize,
                "{name} at {w}x{h}: the buffer must cover the whole frame"
            );
            let text = lines.join("\n");
            assert!(text.contains("Home"), "{name} at {w}x{h}: {text}");
            assert!(
                text.contains("INTENT"),
                "{name} at {w}x{h}: the composer must always be visible: {text}"
            );
            let footer = lines.last().expect("at least one row");
            assert!(
                !footer.trim().is_empty(),
                "{name} at {w}x{h}: the footer row must never render blank: {text}"
            );
        }
    }

    let paused = home_admission_paused();
    let text = text_of(&draw_at(&paused, 120, 36));
    assert!(
        text.contains("admission is paused"),
        "admission-paused must be named in the form, not merely implied: {text}"
    );
}

#[test]
fn geometry_home_full_wide_shows_drawer_form_and_rail_together() {
    let app = home_full();
    let text = text_of(&draw_at(&app, 180, 48));
    assert!(text.contains("Attention"), "Wide's drawer: {text}");
    assert!(
        text.contains("waiting on an answer"),
        "drawer content: {text}"
    );
    assert!(text.contains("Home / New Work"), "Wide's form: {text}");
    assert!(text.contains("Recent Outputs"), "Wide's rail: {text}");
    assert!(text.contains("already done"), "rail content: {text}");
}

#[test]
fn geometry_home_full_narrow_drawer_replaces_the_body_not_alongside_it() {
    let app = home_full();
    assert!(app.drawer_open, "drawer opens by default (§7.3)");
    let text = text_of(&draw_at(&app, 80, 24));
    assert!(text.contains("Attention"), "Narrow-open drawer: {text}");
    assert!(
        !text.contains("Home / New Work"),
        "§18.3: the drawer is a full-body overlay at Narrow — it must not \
         share the frame with the primary body: {text}"
    );
}

#[test]
fn geometry_fleet_every_reported_state() {
    let app = fleet_all_states();
    for (w, h) in SIZES {
        let terminal = draw_at(&app, w, h);
        let text = text_of(&terminal);
        assert!(text.contains("Fleet —"), "{w}x{h}: {text}");
        // Every glyph must actually land on screen — the codepoint the
        // Fleet table cells, the Attention drawer, and the header all draw
        // from `theme::state_glyph` — proving §19.10's "no one-cell glyph
        // violation" past `theme.rs`'s own unit-only check
        // (`every_glyph_is_exactly_one_cell_wide`, char-count only) with a
        // real render: a glyph silently dropped or double-width-truncated
        // by the backend would fail this even though the unit test passes.
        for state in fleet::REPORTED_STATES {
            assert!(
                text.contains(theme::state_glyph(state)),
                "{w}x{h}: the {state} glyph must render, not vanish: {text}"
            );
        }
    }
    // Only Wide (150+ cols) is guaranteed room for every one of the 9
    // states' full row text at once — Narrow/Medium may legitimately show
    // fewer rows, which the loop above already tolerates.
    let text = text_of(&draw_at(&app, 180, 48));
    for state in fleet::REPORTED_STATES {
        assert!(
            text.contains(&format!("work in state {state}")),
            "Wide must fit every state's row: {text}"
        );
    }
}

#[test]
fn geometry_fleet_long_values_stay_contained() {
    let app = fleet_long_values();
    for (w, h) in SIZES {
        let terminal = draw_at(&app, w, h);
        let lines = lines_of(&terminal);
        for line in &lines {
            assert!(
                line.chars().count() as u16 <= w,
                "{w}x{h}: a row overflowed the frame width: {line:?}"
            );
        }
        let text = lines.join("\n");
        // The state column is `Length`, not `Fill` — a 400-char intent must
        // not crowd it off screen.
        assert!(
            text.contains("needs_input"),
            "{w}x{h}: the state column must survive a huge intent: {text}"
        );
    }
}

#[test]
fn geometry_work_surface_across_every_reported_state() {
    let states = [
        ("active", "n/a"),
        ("needs_input", "which retry budget?"),
        ("blocked", "waiting on a human decision"),
        ("failed", "the stage exited nonzero"),
        ("completed", "n/a"),
        ("completed_dirty", "n/a"),
    ];
    for (state, detail) in states {
        let app = app_with_open_work("01WORKSTATE", state, detail);
        for (w, h) in SIZES {
            let terminal = draw_at(&app, w, h);
            let lines = lines_of(&terminal);
            let text = lines.join("\n");
            assert!(
                text.contains(state),
                "{state} at {w}x{h}: the reported state must stay visible: {text}"
            );
            assert!(
                text.contains("Thread")
                    && text.contains("Workflow")
                    && text.contains("Evidence")
                    && text.contains("Graph")
                    && text.contains("Details"),
                "{state} at {w}x{h}: the full tab bar must stay visible: {text}"
            );
            assert!(
                text.contains("actions ("),
                "{state} at {w}x{h}: the action row must stay visible: {text}"
            );
            if state == "needs_input" {
                assert!(
                    text.contains(detail),
                    "{state} at {w}x{h}: the pinned question must stay visible: {text}"
                );
                assert!(text.contains("ANSWER"), "{state} at {w}x{h}: {text}");
            } else {
                assert!(
                    text.contains("COMMAND"),
                    "{state} at {w}x{h}: a disabled composer must say so, not vanish: {text}"
                );
            }
            // §19.10's "composer/footer never overlap": the footer occupies
            // exactly the last row (`mod.rs`'s own fixed vertical layout),
            // so the composer's own border title must never appear there.
            let footer = lines.last().expect("at least one row");
            assert!(
                !footer.contains("ANSWER") && !footer.contains("COMMAND"),
                "{state} at {w}x{h}: the composer bled into the footer row: {footer:?}"
            );
        }
    }
}

#[test]
fn geometry_work_surface_focus_is_visible_on_the_answer_composer() {
    // §19.10's "focus is visible": the ANSWER composer's own border title
    // text differs before/after focus (unlike the tab bar's active-tab
    // marking, which is color/bold only — named in this Work's Taste
    // pre-flight notes rather than silently assumed testable here).
    let mut app = app_with_open_work("01FOCUS", "needs_input", "which path?");
    let unfocused = text_of(&draw_at(&app, 120, 36));
    assert!(unfocused.contains("press 'r' to respond"), "{unfocused}");
    app.on_key(KeyCode::Char('r'));
    let focused = text_of(&draw_at(&app, 120, 36));
    assert!(
        focused.contains("Ctrl+Enter send"),
        "focusing the composer must change its own title text: {focused}"
    );
}

#[test]
fn geometry_workflow_tab_never_shows_a_percent_gauge() {
    let mut app = app_with_open_work("01WF", "active", "n/a");
    app.on_key(KeyCode::Char('2')); // Workflow tab (§13.2)
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&app, w, h));
        assert!(
            !text.contains('%'),
            "{w}x{h}: Decision T2-50 — order, never a percent bar: {text}"
        );
    }
}

#[test]
fn geometry_workflows_catalog_detail_and_error() {
    for (name, app) in [
        ("catalog", workflows_catalog()),
        ("error", workflows_error()),
    ] {
        for (w, h) in SIZES {
            let text = text_of(&draw_at(&app, w, h));
            assert!(text.contains("Workflows"), "{name} at {w}x{h}: {text}");
            match name {
                "catalog" => {
                    assert!(text.contains("implement"), "{name} at {w}x{h}: {text}");
                    assert!(text.contains("Detail"), "{name} at {w}x{h}: {text}");
                }
                "error" => assert!(
                    text.contains("could not load workflows"),
                    "{name} at {w}x{h}: {text}"
                ),
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn geometry_estate_repos_groups_and_health() {
    for (name, app) in [
        ("repos", estate_repos()),
        ("groups", estate_groups()),
        ("health", estate_health()),
    ] {
        for (w, h) in SIZES {
            let text = text_of(&draw_at(&app, w, h));
            assert!(text.contains("Estate"), "{name} at {w}x{h}: {text}");
            match name {
                "repos" => assert!(text.contains("svc-a"), "{name} at {w}x{h}: {text}"),
                "groups" => assert!(text.contains("core"), "{name} at {w}x{h}: {text}"),
                "health" => {
                    assert!(text.contains("disk_pressure"), "{name} at {w}x{h}: {text}");
                    assert!(text.contains("free up space"), "{name} at {w}x{h}: {text}");
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn geometry_attention_open_closed_and_over_an_overlay() {
    // Short enough to fit even the Medium tier's 24-column drawer
    // (`theme::DRAWER_WIDTH_MEDIUM`) unclipped — a long-value drawer row is
    // `geometry_fleet_long_values_stay_contained`'s job, not this one's.
    let rows = || {
        fleet::fleet_rows(&json!({"works": [
            {"id": "01A", "state": "needs_input", "intent": "needs a look",
             "stage": {"detail": "which path?"}},
            {"id": "01B", "state": "blocked", "intent": "stuck"},
        ]}))
    };
    for (w, h) in SIZES {
        let mut open = App::new();
        open.rows = rows();
        open.drawer_open = true;
        let text = text_of(&draw_at(&open, w, h));
        assert!(text.contains("needs a look"), "{w}x{h}: {text}");

        let mut closed = App::new();
        closed.rows = rows();
        closed.drawer_open = false;
        let text = text_of(&draw_at(&closed, w, h));
        assert!(
            !text.contains("needs a look"),
            "{w}x{h}: a closed drawer must not still show its rows: {text}"
        );
        assert!(text.contains("Home / New Work"), "{w}x{h}: {text}");

        let mut overlay = App::new();
        overlay.rows = rows();
        overlay.overlay = Some(Overlay::Help);
        let text = text_of(&draw_at(&overlay, w, h));
        assert!(
            text.contains("Navigation"),
            "{w}x{h}: the overlay must render over whatever tier the drawer would take: {text}"
        );
    }
    // Medium/Wide: drawer and primary body both inline, at once.
    for (w, h) in [(120, 36), (180, 48)] {
        let mut open = App::new();
        open.rows = rows();
        open.drawer_open = true;
        let text = text_of(&draw_at(&open, w, h));
        assert!(text.contains("needs a look"), "{w}x{h}: {text}");
        assert!(
            text.contains("Home / New Work"),
            "{w}x{h}: Medium/Wide show the drawer alongside the primary body, not instead of it: {text}"
        );
    }
}

#[test]
fn geometry_palette_chooser_and_confirmations() {
    for overlay in [
        Overlay::CancelConfirmation,
        Overlay::RetryConfirmation,
        Overlay::ExtendEnvelope,
        Overlay::ReapConfirmation,
    ] {
        let mut app = app_with_open_work("01CONF", "blocked", "n/a");
        app.overlay = Some(overlay);
        for (w, h) in SIZES {
            let text = text_of(&draw_at(&app, w, h));
            assert!(
                text.contains("01CONF"),
                "{overlay:?} at {w}x{h}: every confirmation must name the Work it acts on (§15.5): {text}"
            );
            assert!(
                text.contains("Esc") && text.contains("Enter"),
                "{overlay:?} at {w}x{h}: {text}"
            );
        }
    }

    let mut chooser = App::new();
    chooser
        .workflows
        .set_entries(vec![
            json!({"name": "implement", "version": "1", "source": "/x", "content_hash": "a", "stages": []}),
        ]);
    chooser.overlay = Some(Overlay::WorkflowChooser);
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&chooser, w, h));
        assert!(text.contains("implement"), "chooser at {w}x{h}: {text}");
    }

    let mut repo_add = App::new();
    repo_add.destination = Destination::Estate;
    repo_add.overlay = Some(Overlay::RepoAddRemove);
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&repo_add, w, h));
        assert!(
            text.contains("Add a repository"),
            "repo add at {w}x{h}: {text}"
        );
    }

    let mut group_edit = App::new();
    group_edit.destination = Destination::Estate;
    group_edit.estate.tab = EstateTab::Groups;
    group_edit.overlay = Some(Overlay::GroupEditRemove);
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&group_edit, w, h));
        assert!(
            text.contains("Create or extend a group"),
            "group edit at {w}x{h}: {text}"
        );
    }

    // Issue #154: `Overlay::ConnectionDetail` had no render body and no key
    // binding anywhere — this follow-up Work built both. Proven at every
    // size, for every `Live` state, that the real body renders rather than
    // the fixed "not built in this Work" placeholder every unbuilt overlay
    // still falls back to (still proven above for the slash palette below).
    for live in [Live::Attached, Live::Reconnecting, Live::AuthFailed] {
        let mut detail = App::new();
        detail.live = live;
        detail.overlay = Some(Overlay::ConnectionDetail);
        for (w, h) in SIZES {
            let text = text_of(&draw_at(&detail, w, h));
            assert!(
                !text.contains("not built in this Work"),
                "{live:?} connection detail at {w}x{h} must render its real body: {text}"
            );
            assert!(text.contains("Esc"), "{live:?} at {w}x{h}: {text}");
        }
    }

    // §15.3's slash palette (Decision T2-55, issue #152): the fixed
    // vocabulary renders for real, the same shape as the chooser/repo/group
    // panels above.
    let mut palette = App::new();
    palette.overlay = Some(Overlay::SlashPalette);
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&palette, w, h));
        assert!(text.contains("/home"), "palette at {w}x{h}: {text}");
    }
}

#[test]
fn geometry_reconnecting_and_auth_failed_render_identically_across_ticks() {
    for live in [Live::Reconnecting, Live::AuthFailed] {
        let mut app = App::new();
        app.live = live;
        app.rows = fleet::fleet_rows(&json!({"works": [
            {"id": "01ACTIVE", "state": "active", "intent": "still running"},
        ]}));
        app.destination = Destination::Fleet;
        app.drawer_open = false;
        // The header is a single unwrapped line (`mod.rs`'s own
        // `header_line`) — at 80 columns the full parenthetical hint past
        // the core word can legitimately clip, so this checks the durable
        // word itself, not `Live::label()`'s whole string.
        let marker = match live {
            Live::Reconnecting => "RECONNECTING",
            Live::AuthFailed => "AUTH FAILED",
            Live::Attached => unreachable!("not in this fixture's set"),
        };
        for (w, h) in SIZES {
            let first = text_of(&draw_at(&app, w, h));
            let second = text_of(&draw_at(&app, w, h));
            assert_eq!(
                first, second,
                "{live:?} at {w}x{h}: identical state must render identically — \
                 §19.10's 'no active animation while stale' means nothing here \
                 may depend on wall-clock ticks"
            );
            assert!(
                first.contains(marker),
                "{live:?} at {w}x{h}: the durable indicator must stay visible: {first}"
            );
        }
    }
}

#[test]
fn geometry_terminal_too_small_shows_the_safe_notice_never_panics() {
    // §17.7/§19.10: below the 80x24 floor is definitionally outside the
    // three canonical sizes, so this checks representative below-floor
    // points instead of forcing the fixture into that grid.
    let app = App::new();
    for (w, h) in [(79u16, 24u16), (80, 23), (40, 10), (10, 5)] {
        let terminal = draw_at(&app, w, h);
        let text = text_of(&terminal);
        assert!(text.contains("too small"), "{w}x{h}: {text}");
        assert!(text.contains(&format!("{w}x{h}")), "{w}x{h}: {text}");
    }
}

#[test]
fn geometry_retained_preview_and_reap_confirmation() {
    let mut app = App::new();
    app.destination = Destination::Estate;
    app.estate.tab = EstateTab::Health;
    app.estate.retained = vec![json!({
        "work_id": "01RET", "repository": "svc-a", "disposition": "retained_dirty",
        "bytes": 4096, "path": "/data/svc-a",
    })];
    app.overlay = Some(Overlay::RetainedPreview);
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&app, w, h));
        assert!(text.contains("01RET"), "{w}x{h}: {text}");
    }

    // `p` opens the reap confirmation for the selected retained entry —
    // driven through the real keymap (`App::on_key_overlay`'s
    // `Overlay::RetainedPreview` arm), not by hand-setting
    // `retained_reap_id`.
    app.on_key(KeyCode::Char('p'));
    assert_eq!(app.overlay, Some(Overlay::ReapConfirmation));
    for (w, h) in SIZES {
        let text = text_of(&draw_at(&app, w, h));
        assert!(
            text.contains("01RET"),
            "{w}x{h}: the reap confirmation must still name the Work: {text}"
        );
        assert!(text.contains("Reap"), "{w}x{h}: {text}");
    }
}

// ------------------------------------------------------- real screenshots

/// T4 (§20.5's "real Ratatui screenshots"): this repo has no terminal-image
/// capture tool available (the `docs/img/*.png` pair is a hand-captured
/// artifact of the pre-T-series two-screen TUI, unrelated to this
/// mechanism and left untouched). `TestBackend` is already this tree's own
/// established rendering contract (Decision T2-63); this test asks
/// nothing new of it — it flattens a real `draw()` call's buffer to plain
/// text and writes it to disk, so the file on disk is exactly what
/// [`tui_screenshot`]'s many other assertions already prove correct, not a
/// hand-typed mockup.
///
/// `#[ignore]` by default: writing to the working tree is not something an
/// ordinary `cargo test` should do as a side effect. Regenerate with:
/// `cargo test --lib -- --ignored generate_t4_wide_screenshots`.
#[test]
#[ignore = "writes docs/tui-screenshots/*.txt; run explicitly to regenerate"]
fn generate_t4_wide_screenshots() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/tui-screenshots");
    std::fs::create_dir_all(&dir).expect("create docs/tui-screenshots");

    let fixtures: Vec<(&str, App)> = vec![
        ("home", home_full()),
        ("fleet", fleet_all_states()),
        (
            "work-thread",
            app_with_open_work("01SCREEN", "needs_input", "which retry budget?"),
        ),
        ("workflows", workflows_catalog()),
        ("estate", estate_repos()),
    ];
    for (name, app) in fixtures {
        let terminal = draw_at(&app, 180, 48);
        let text = text_of(&terminal);
        let header = format!(
            "# Real TestBackend render (180x48, Wide) of sergeant's TUI — {name}.\n\
             # Generated by src/tui/geometry_matrix_tests.rs's \
             generate_t4_wide_screenshots, not hand-typed: this is `tui::draw`'s\n\
             # own output through ratatui's `TestBackend`, flattened to plain text.\n\
             # Regenerate with: cargo test --lib -- --ignored generate_t4_wide_screenshots\n\n"
        );
        let path = dir.join(format!("{name}-wide.txt"));
        std::fs::write(&path, format!("{header}{text}\n"))
            .unwrap_or_else(|e| panic!("write {path:?}: {e}"));
    }
}
