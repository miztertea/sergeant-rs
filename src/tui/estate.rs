//! Estate (§12): Repositories, Groups, and Health, over the seven `/v1/
//! estate/*` and `/v1/doctor` routes T3 adds to [`crate::api`] — thin
//! wrappers over `crate::domain::manifest`/`crate::cli::doctor` that this
//! module never reaches for directly (§30's rule: the TUI talks to the
//! daemon and nothing else).
//!
//! §12.5 names what this screen deliberately does not build: `sgt init`,
//! daemon foreground/stop, harness passthrough, data-dir relocation, a raw
//! manifest editor, CPU/memory/process monitoring.

use std::time::Instant;

use ratatui::Frame;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, Padding, Paragraph, Wrap};
use serde_json::Value;

use crate::api::{ClientError, field_text};

use super::theme::Token;

/// §12's three sub-screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstateTab {
    Repositories,
    Groups,
    Health,
}

impl EstateTab {
    const ALL: [EstateTab; 3] = [
        EstateTab::Repositories,
        EstateTab::Groups,
        EstateTab::Health,
    ];

    fn label(self) -> &'static str {
        match self {
            EstateTab::Repositories => "Repositories",
            EstateTab::Groups => "Groups",
            EstateTab::Health => "Health",
        }
    }

    fn next(self) -> EstateTab {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    fn prev(self) -> EstateTab {
        let i = Self::ALL.iter().position(|t| *t == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// What a keystroke on the Estate screen itself (not one of its overlays)
/// asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstateOutcome {
    None,
    /// `a` on Repositories: open [`super::overlay::Overlay::RepoAddRemove`]
    /// in add mode.
    OpenAddRepo,
    /// `x`/`d` on Repositories, with a repo selected: open the same overlay
    /// in remove-confirmation mode (§12.1's exact confirmation text).
    OpenRemoveRepo(String),
    /// `a` on Groups: open [`super::overlay::Overlay::GroupEditRemove`] in
    /// edit mode (create a new group, or extend the selected one).
    OpenGroupEdit,
    /// `x`/`d` on Groups, with a group selected: open the same overlay in
    /// remove mode.
    OpenGroupRemove(String),
    /// `r` on Health, with the `disk_pressure` check selected: open
    /// [`super::overlay::Overlay::RetainedPreview`] (§12.4).
    OpenRetained,
}

/// A repo-add POST kicked off the render loop (§12.1: "the existing clone/
/// register operation runs off the render loop"). Polled non-blockingly
/// every tick from [`super::event_loop`] via [`EstateScreen::poll`] — never
/// awaited inline, so a slow `git clone` never stalls a keystroke or a
/// redraw.
pub struct PendingRepoAdd {
    pub name: String,
    pub started: Instant,
    rx: tokio::sync::oneshot::Receiver<Result<Value, ClientError>>,
}

impl PendingRepoAdd {
    pub fn new(
        name: String,
        rx: tokio::sync::oneshot::Receiver<Result<Value, ClientError>>,
    ) -> Self {
        Self {
            name,
            started: Instant::now(),
            rx,
        }
    }
}

/// One outcome of polling a [`PendingRepoAdd`].
pub enum PendingRepoAddOutcome {
    /// Still running — nothing to do but keep showing the spinner.
    InProgress,
    Done(Result<Value, ClientError>),
}

/// One field of the add-repo form, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepoField {
    Name,
    Origin,
    Instructions,
    Confirm,
}
const REPO_FIELDS: [RepoField; 4] = [
    RepoField::Name,
    RepoField::Origin,
    RepoField::Instructions,
    RepoField::Confirm,
];

/// `Overlay::RepoAddRemove`'s own mode (§12.1's two flows share one panel).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoOverlayMode {
    Add,
    ConfirmRemove { name: String },
}

/// §12.1's Add form: `name`, optional `origin`, `instructions: local|suppress`.
#[derive(Debug)]
pub struct RepoForm {
    pub mode: RepoOverlayMode,
    focus: usize,
    pub name: String,
    pub origin: String,
    /// Cycled by Enter on the field: `None` (unset, the manifest default),
    /// `Some("local")`, `Some("suppress")`.
    pub instructions: Option<&'static str>,
    pub last_error: Option<String>,
}

impl Default for RepoForm {
    fn default() -> Self {
        Self {
            mode: RepoOverlayMode::Add,
            focus: 0,
            name: String::new(),
            origin: String::new(),
            instructions: None,
            last_error: None,
        }
    }
}

/// What an [`RepoForm`]/[`GroupForm`] keystroke asked the overlay to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepoFormOutcome {
    None,
    Close,
    /// Add mode's Confirm: the `POST /v1/estate/repos` body.
    Submit {
        name: String,
        origin: Option<String>,
        instructions: Option<String>,
    },
    /// Remove mode's Confirm: `DELETE /v1/estate/repos/{name}`.
    ConfirmRemove(String),
}

impl RepoForm {
    pub fn reset_for_add(&mut self) {
        *self = RepoForm::default();
    }

    pub fn reset_for_remove(&mut self, name: String) {
        *self = RepoForm {
            mode: RepoOverlayMode::ConfirmRemove { name },
            ..RepoForm::default()
        };
    }

    pub fn on_key(&mut self, key: KeyEvent) -> RepoFormOutcome {
        match &self.mode {
            RepoOverlayMode::ConfirmRemove { name } => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => RepoFormOutcome::Close,
                KeyCode::Enter => RepoFormOutcome::ConfirmRemove(name.clone()),
                _ => RepoFormOutcome::None,
            },
            RepoOverlayMode::Add => {
                match key.code {
                    KeyCode::Esc => return RepoFormOutcome::Close,
                    KeyCode::Tab => {
                        self.focus = (self.focus + 1) % REPO_FIELDS.len();
                        return RepoFormOutcome::None;
                    }
                    KeyCode::BackTab => {
                        self.focus = (self.focus + REPO_FIELDS.len() - 1) % REPO_FIELDS.len();
                        return RepoFormOutcome::None;
                    }
                    KeyCode::Enter => {
                        return match REPO_FIELDS[self.focus] {
                            RepoField::Instructions => {
                                self.instructions = match self.instructions {
                                    None => Some("local"),
                                    Some("local") => Some("suppress"),
                                    _ => None,
                                };
                                RepoFormOutcome::None
                            }
                            RepoField::Confirm => self.try_submit(),
                            _ => {
                                self.focus = (self.focus + 1) % REPO_FIELDS.len();
                                RepoFormOutcome::None
                            }
                        };
                    }
                    KeyCode::Backspace => {
                        if let Some(field) = self.field_mut() {
                            field.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some(field) = self.field_mut() {
                            field.push(c);
                        }
                    }
                    _ => {}
                }
                RepoFormOutcome::None
            }
        }
    }

    fn field_mut(&mut self) -> Option<&mut String> {
        match REPO_FIELDS[self.focus] {
            RepoField::Name => Some(&mut self.name),
            RepoField::Origin => Some(&mut self.origin),
            RepoField::Instructions | RepoField::Confirm => None,
        }
    }

    fn try_submit(&mut self) -> RepoFormOutcome {
        let name = self.name.trim().to_string();
        if name.is_empty() {
            self.last_error = Some("name is required".to_string());
            return RepoFormOutcome::None;
        }
        let origin = (!self.origin.trim().is_empty()).then(|| self.origin.trim().to_string());
        RepoFormOutcome::Submit {
            name,
            origin,
            instructions: self.instructions.map(str::to_string),
        }
    }
}

/// One field of the group edit form, in tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GroupField {
    Name,
    Repos,
    Brief,
    Confirm,
}
const GROUP_FIELDS: [GroupField; 4] = [
    GroupField::Name,
    GroupField::Repos,
    GroupField::Brief,
    GroupField::Confirm,
];

/// `Overlay::GroupEditRemove`'s own mode (§12.2's full lifecycle in one
/// panel): create-or-extend, or remove (selected members, or the whole
/// group when none are named).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupOverlayMode {
    Edit,
    Remove { name: String },
}

/// §12.2's group form: create, extend membership, remove selected members,
/// or remove the group — `manifest::add_group`'s mkdir-p union semantics and
/// `manifest::remove_group`'s "empty repos removes the whole group" both
/// reached exactly as they already behave, never reimplemented here.
#[derive(Debug)]
pub struct GroupForm {
    pub mode: GroupOverlayMode,
    focus: usize,
    pub name: String,
    /// Edit mode: repos to add (union in). Remove mode: repos to remove
    /// (blank removes the whole group).
    pub repos: String,
    pub brief: String,
    pub last_error: Option<String>,
}

impl Default for GroupForm {
    fn default() -> Self {
        Self {
            mode: GroupOverlayMode::Edit,
            focus: 0,
            name: String::new(),
            repos: String::new(),
            brief: String::new(),
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupFormOutcome {
    None,
    Close,
    /// Edit mode's Confirm: the `POST /v1/estate/groups` body.
    Submit {
        name: String,
        repos: Vec<String>,
        brief: Option<String>,
    },
    /// Remove mode's Confirm: the `DELETE /v1/estate/groups/{name}` body —
    /// empty `repos` removes the whole group.
    ConfirmRemove {
        name: String,
        repos: Vec<String>,
    },
}

impl GroupForm {
    /// Edit mode, pre-filled from an existing group (extend) or blank
    /// (create).
    pub fn reset_for_edit(&mut self, existing: Option<&Value>) {
        *self = GroupForm {
            mode: GroupOverlayMode::Edit,
            name: existing
                .and_then(|g| g["name"].as_str())
                .unwrap_or_default()
                .to_string(),
            ..GroupForm::default()
        };
    }

    pub fn reset_for_remove(&mut self, name: String) {
        *self = GroupForm {
            mode: GroupOverlayMode::Remove { name },
            ..GroupForm::default()
        };
    }

    pub fn on_key(&mut self, key: KeyEvent) -> GroupFormOutcome {
        match key.code {
            KeyCode::Esc => return GroupFormOutcome::Close,
            KeyCode::Tab => {
                self.focus = (self.focus + 1) % GROUP_FIELDS.len();
                return GroupFormOutcome::None;
            }
            KeyCode::BackTab => {
                self.focus = (self.focus + GROUP_FIELDS.len() - 1) % GROUP_FIELDS.len();
                return GroupFormOutcome::None;
            }
            KeyCode::Enter if GROUP_FIELDS[self.focus] == GroupField::Confirm => {
                return self.try_submit();
            }
            KeyCode::Enter => {
                self.focus = (self.focus + 1) % GROUP_FIELDS.len();
            }
            KeyCode::Backspace => {
                if let Some(field) = self.field_mut() {
                    field.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(field) = self.field_mut() {
                    field.push(c);
                }
            }
            _ => {}
        }
        GroupFormOutcome::None
    }

    fn field_mut(&mut self) -> Option<&mut String> {
        match GROUP_FIELDS[self.focus] {
            GroupField::Name => match &self.mode {
                GroupOverlayMode::Edit => Some(&mut self.name),
                GroupOverlayMode::Remove { .. } => None,
            },
            GroupField::Repos => Some(&mut self.repos),
            GroupField::Brief => match &self.mode {
                GroupOverlayMode::Edit => Some(&mut self.brief),
                GroupOverlayMode::Remove { .. } => None,
            },
            GroupField::Confirm => None,
        }
    }

    fn split_repos(&self) -> Vec<String> {
        self.repos
            .split([',', ' '])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }

    fn try_submit(&mut self) -> GroupFormOutcome {
        match &self.mode {
            GroupOverlayMode::Edit => {
                let name = self.name.trim().to_string();
                if name.is_empty() {
                    self.last_error = Some("name is required".to_string());
                    return GroupFormOutcome::None;
                }
                let brief = (!self.brief.trim().is_empty()).then(|| self.brief.trim().to_string());
                GroupFormOutcome::Submit {
                    name,
                    repos: self.split_repos(),
                    brief,
                }
            }
            GroupOverlayMode::Remove { name } => GroupFormOutcome::ConfirmRemove {
                name: name.clone(),
                repos: self.split_repos(),
            },
        }
    }
}

/// W4d deliverable 3: the estate this screen currently addresses, once
/// picked. `root` is the canonical estate root — the key
/// `repos_for`/`groups_for`/`doctor_for` and `GET /v1/estates` itself both
/// address estates by (D1) — `name` is display-only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedEstate {
    pub name: String,
    pub root: std::path::PathBuf,
}

/// The Estate destination's whole state: three sub-screens' data and
/// selection, plus the two edit forms and the retained-state preview
/// [`super::overlay::Overlay::RepoAddRemove`]/[`super::overlay::Overlay::
/// GroupEditRemove`]/[`super::overlay::Overlay::RetainedPreview`] need.
pub struct EstateScreen {
    pub tab: EstateTab,
    /// `GET /v1/estates`' entries (H1 §4) — every estate this daemon has
    /// admitted, fed to the picker (W4d deliverable 3). Populated whenever
    /// this screen is in front, the same discipline as `repos`/`groups`/
    /// `doctor` below.
    pub estates: Vec<Value>,
    /// Which entry in `estates` the picker is currently highlighting,
    /// before anything is picked.
    pub picker_selected: usize,
    /// The estate `repos`/`groups`/`doctor` below address, once chosen.
    /// `None` shows only the picker: this screen's existing sub-tabs are
    /// inherently single-estate-at-a-time (their own `/v1/estate/*` routes
    /// address exactly one), so there is nothing of theirs to show before
    /// one is addressed. A client that already addresses one fixed estate
    /// (`ApiClient::estate_root` — the pre-H1, still-supported single-
    /// estate-scoped shape) is auto-picked the moment this screen is
    /// entered, so that shape's behavior is unchanged byte for byte: no
    /// picker ever appears for it.
    pub picked: Option<PickedEstate>,
    /// `GET /v1/estate/repos`'s `repos` array.
    pub repos: Vec<Value>,
    /// `GET /v1/estate/groups`'s `groups` array.
    pub groups: Vec<Value>,
    /// `GET /v1/doctor`'s whole report body.
    pub doctor: Value,
    pub repo_selected: usize,
    pub group_selected: usize,
    pub check_selected: usize,
    pub repo_form: RepoForm,
    pub group_form: GroupForm,
    pub pending_repo_add: Option<PendingRepoAdd>,
    /// `GET /v1/retained`'s entries — populated when
    /// [`super::overlay::Overlay::RetainedPreview`] opens (§12.4: reuses the
    /// existing estate-wide read, not a new route).
    pub retained: Vec<Value>,
    pub retained_selected: usize,
    /// Set on a failed mutation, so it stays visible next to the action that
    /// caused it (§17.5) rather than only flashing on the status line.
    pub last_error: Option<String>,
}

impl Default for EstateScreen {
    fn default() -> Self {
        Self {
            tab: EstateTab::Repositories,
            estates: Vec::new(),
            picker_selected: 0,
            picked: None,
            repos: Vec::new(),
            groups: Vec::new(),
            doctor: Value::Null,
            repo_selected: 0,
            group_selected: 0,
            check_selected: 0,
            repo_form: RepoForm::default(),
            group_form: GroupForm::default(),
            pending_repo_add: None,
            retained: Vec::new(),
            retained_selected: 0,
            last_error: None,
        }
    }
}

impl EstateScreen {
    /// Clamp every selection index to the data currently loaded, the same
    /// discipline [`super::fleet::FleetScreen::clamp`] applies — a refresh
    /// that shrinks a list must never leave a selection pointing past its end.
    pub fn clamp(&mut self) {
        self.picker_selected = clamp_index(self.picker_selected, self.estates.len());
        self.repo_selected = clamp_index(self.repo_selected, self.repos.len());
        self.group_selected = clamp_index(self.group_selected, self.groups.len());
        let checks = self.doctor["checks"].as_array().map_or(0, Vec::len);
        self.check_selected = clamp_index(self.check_selected, checks);
        self.retained_selected = clamp_index(self.retained_selected, self.retained.len());
    }

    /// Auto-pick for a client that already addresses one fixed estate
    /// (`ApiClient::estate_root` — the pre-H1 single-estate-scoped shape) —
    /// called by `App::refresh` every time this screen is in front, so it
    /// is idempotent rather than a one-time seed: a `root` that has not
    /// changed leaves `picked` alone (no selection reset mid-browse), and a
    /// changed one (a different `-C`, in practice never mid-session) is
    /// re-picked with a fresh name.
    pub fn auto_pick(&mut self, root: &std::path::Path, name: &str) {
        if self.picked.as_ref().is_some_and(|p| p.root == root) {
            return;
        }
        self.picked = Some(PickedEstate {
            name: name.to_string(),
            root: root.to_path_buf(),
        });
    }

    /// Whether the picker is actually in front of the sub-tabs right now.
    ///
    /// Deliberately **not** just `self.picked.is_none()`: before the first
    /// `App::refresh` has run at all (this screen just entered, nothing
    /// fetched yet — the exact moment the pre-existing T3 estate tests
    /// press Tab/`a`/`x` against), `picked` is `None` and `estates` is
    /// still empty too. Gating on `estates` being non-empty as well means
    /// that ordinary "nothing loaded yet" moment renders and keys exactly
    /// as it always has — exactly like every other lazily-loaded pane here
    /// (`repos`/`groups`/`doctor` themselves start empty and are simply
    /// blank until the first successful read). A client that never fetches
    /// `estates` at all (the fixed-single-estate shape, `auto_pick`'s own
    /// case) never shows a picker for exactly this reason: the condition
    /// this method checks never both hold for it.
    fn showing_picker(&self) -> bool {
        self.picked.is_none() && !self.estates.is_empty()
    }

    /// The currently highlighted picker entry's name and root, if the list
    /// is non-empty and its entry has both fields — the same tolerant
    /// `field_text`-adjacent reading every other screen here already does
    /// for a body it does not control the shape of.
    fn picker_choice(&self) -> Option<PickedEstate> {
        let entry = self.estates.get(self.picker_selected)?;
        let name = entry["name"].as_str()?.to_string();
        let root = entry["root"].as_str()?;
        Some(PickedEstate {
            name,
            root: std::path::PathBuf::from(root),
        })
    }

    pub fn selected_repo(&self) -> Option<&Value> {
        self.repos.get(self.repo_selected)
    }

    pub fn selected_group(&self) -> Option<&Value> {
        self.groups.get(self.group_selected)
    }

    pub fn selected_check(&self) -> Option<&Value> {
        self.doctor["checks"]
            .as_array()
            .and_then(|c| c.get(self.check_selected))
    }

    pub fn selected_retained(&self) -> Option<&Value> {
        self.retained.get(self.retained_selected)
    }

    /// Non-blocking poll of an in-flight repo add (§12.1). Called every loop
    /// tick from [`super::event_loop`] — `Ok(Empty)` means still running.
    pub fn poll(&mut self) -> Option<PendingRepoAddOutcome> {
        let pending = self.pending_repo_add.as_mut()?;
        match pending.rx.try_recv() {
            Ok(result) => Some(PendingRepoAddOutcome::Done(result)),
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {
                Some(PendingRepoAddOutcome::InProgress)
            }
            Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                Some(PendingRepoAddOutcome::Done(Err(ClientError::Transport(
                    "the add-repo task ended without answering".to_string(),
                ))))
            }
        }
    }

    /// Keys the Estate screen itself handles, outside any overlay — tab
    /// switching and list navigation are captured here, ahead of the global
    /// destination-nav keys, the same way [`super::work_view::WorkScreen`]
    /// captures its own `Tab`.
    ///
    /// W4d deliverable 3: while [`Self::showing_picker`] holds, every key
    /// is the picker's own (Up/Down to browse, Enter to choose) — none of
    /// the existing sub-tab keys below fire. See that method's own doc
    /// comment for why this is not simply "nothing is picked yet".
    pub fn on_key(&mut self, key: KeyCode) -> EstateOutcome {
        if self.showing_picker() {
            match key {
                KeyCode::Down | KeyCode::Char('j') => {
                    self.picker_selected = (self.picker_selected + 1) % self.estates.len();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.picker_selected =
                        (self.picker_selected + self.estates.len() - 1) % self.estates.len();
                }
                KeyCode::Enter => {
                    if let Some(choice) = self.picker_choice() {
                        self.picked = Some(choice);
                        self.repo_selected = 0;
                        self.group_selected = 0;
                        self.check_selected = 0;
                    }
                }
                _ => {}
            }
            return EstateOutcome::None;
        }
        // `p` returns to the picker — a no-op unless `estates` actually has
        // more than one candidate to offer (`showing_picker` needs it
        // non-empty too), the same "harmless outside its own tier" shape
        // Fleet's own `v` uses.
        if key == KeyCode::Char('p') {
            self.picked = None;
            self.picker_selected = 0;
            return EstateOutcome::None;
        }
        match key {
            KeyCode::Tab => self.tab = self.tab.next(),
            KeyCode::BackTab => self.tab = self.tab.prev(),
            KeyCode::Char('h') | KeyCode::Left => self.tab = self.tab.prev(),
            KeyCode::Char('l') | KeyCode::Right => self.tab = self.tab.next(),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Char('a') => {
                return match self.tab {
                    EstateTab::Repositories => EstateOutcome::OpenAddRepo,
                    EstateTab::Groups => EstateOutcome::OpenGroupEdit,
                    EstateTab::Health => EstateOutcome::None,
                };
            }
            KeyCode::Char('x') | KeyCode::Char('d') => {
                return match self.tab {
                    EstateTab::Repositories => self
                        .selected_repo()
                        .and_then(|r| r["name"].as_str())
                        .map(|n| EstateOutcome::OpenRemoveRepo(n.to_string()))
                        .unwrap_or(EstateOutcome::None),
                    EstateTab::Groups => self
                        .selected_group()
                        .and_then(|g| g["name"].as_str())
                        .map(|n| EstateOutcome::OpenGroupRemove(n.to_string()))
                        .unwrap_or(EstateOutcome::None),
                    EstateTab::Health => EstateOutcome::None,
                };
            }
            KeyCode::Char('r')
                if self.tab == EstateTab::Health
                    && self
                        .selected_check()
                        .and_then(|c| c["name"].as_str())
                        .map(|n| n == "disk_pressure")
                        .unwrap_or(false) =>
            {
                return EstateOutcome::OpenRetained;
            }
            _ => {}
        }
        EstateOutcome::None
    }

    fn move_selection(&mut self, delta: i64) {
        let (selected, len) = match self.tab {
            EstateTab::Repositories => (&mut self.repo_selected, self.repos.len()),
            EstateTab::Groups => (&mut self.group_selected, self.groups.len()),
            EstateTab::Health => (
                &mut self.check_selected,
                self.doctor["checks"].as_array().map_or(0, Vec::len),
            ),
        };
        if len == 0 {
            return;
        }
        *selected = ((*selected as i64 + delta).rem_euclid(len as i64)) as usize;
    }
}

fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { index.min(len - 1) }
}

// -------------------------------------------------------------- rendering

pub fn render(frame: &mut Frame, area: Rect, estate: &EstateScreen) {
    if estate.showing_picker() {
        render_picker(frame, area, estate);
        return;
    }
    let [tabs_area, body_area] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).areas(area);
    let picked_name = estate.picked.as_ref().map_or("-", |p| p.name.as_str());
    render_tabs(frame, tabs_area, estate.tab, picked_name);
    match estate.tab {
        EstateTab::Repositories => render_repos(frame, body_area, estate),
        EstateTab::Groups => render_groups(frame, body_area, estate),
        EstateTab::Health => render_health(frame, body_area, estate),
    }
}

/// W4d deliverable 3: the picker above Repositories/Groups/Health,
/// GET-/v1/estates-fed, shown in place of the sub-tabs until one is chosen.
fn render_picker(frame: &mut Frame, area: Rect, estate: &EstateScreen) {
    let block = bordered("Estate — pick one (↑/↓, Enter)");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if estate.estates.is_empty() {
        frame.render_widget(
            Paragraph::new("no estates admitted to this daemon yet").wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }
    let items: Vec<ListItem> = estate
        .estates
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let style = if i == estate.picker_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let line = format!("{:<24} {}", field_text(&e["name"]), field_text(&e["root"]),);
            ListItem::new(line).style(style)
        })
        .collect();
    frame.render_widget(List::new(items), inner);
}

fn render_tabs(frame: &mut Frame, area: Rect, active: EstateTab, picked_name: &str) {
    let mut spans = vec![Span::styled(
        format!("Estate [{picked_name}]  "),
        Style::default().fg(Token::Muted.rgb()),
    )];
    for tab in EstateTab::ALL {
        let style = if tab == active {
            Style::default()
                .fg(Token::Focus.rgb())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Token::Muted.rgb())
        };
        spans.push(Span::styled(format!("{}  ", tab.label()), style));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn bordered(title: &str) -> Block<'_> {
    Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Token::Border.rgb()))
        .padding(Padding::horizontal(1))
}

fn render_repos(frame: &mut Frame, area: Rect, estate: &EstateScreen) {
    let block = bordered("Repositories — a add · x remove");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if estate.repos.is_empty() {
        frame.render_widget(
            Paragraph::new("no repositories declared").wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }
    let groups_by_repo = |name: &str| -> String {
        let members: Vec<&str> = estate
            .groups
            .iter()
            .filter(|g| {
                g["repos"]
                    .as_array()
                    .map(|repos| repos.iter().any(|r| r.as_str() == Some(name)))
                    .unwrap_or(false)
            })
            .filter_map(|g| g["name"].as_str())
            .collect();
        if members.is_empty() {
            "-".to_string()
        } else {
            members.join(", ")
        }
    };
    let items: Vec<ListItem> = estate
        .repos
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let name = field_text(&r["name"]);
            let style = if i == estate.repo_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let line = format!(
                "{:<20} {:<40} origin={:<28} instructions={:<10} groups={}",
                name,
                field_text(&r["path"]),
                field_text(&r["origin"]),
                field_text(&r["instructions"]),
                groups_by_repo(&name),
            );
            ListItem::new(line).style(style)
        })
        .collect();
    frame.render_widget(List::new(items), inner);
}

fn render_groups(frame: &mut Frame, area: Rect, estate: &EstateScreen) {
    let block = bordered("Groups — a create/extend · x remove");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if estate.groups.is_empty() {
        frame.render_widget(
            Paragraph::new("no groups declared").wrap(Wrap { trim: false }),
            inner,
        );
        return;
    }
    let items: Vec<ListItem> = estate
        .groups
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let style = if i == estate.group_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let repos: Vec<String> = g["repos"]
                .as_array()
                .map(|r| {
                    r.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let line = format!(
                "{:<20} [{}]{}",
                field_text(&g["name"]),
                repos.join(", "),
                g["brief"]
                    .as_str()
                    .map(|b| format!("  {b}"))
                    .unwrap_or_default(),
            );
            ListItem::new(line).style(style)
        })
        .collect();
    frame.render_widget(List::new(items), inner);
}

fn render_health(frame: &mut Frame, area: Rect, estate: &EstateScreen) {
    let [list_area, detail_area] =
        Layout::vertical([Constraint::Percentage(60), Constraint::Min(3)]).areas(area);
    let block = bordered("Health — GET /v1/doctor");
    let inner = block.inner(list_area);
    frame.render_widget(block, list_area);
    let checks = estate.doctor["checks"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if checks.is_empty() {
        frame.render_widget(
            Paragraph::new("no report yet").wrap(Wrap { trim: false }),
            inner,
        );
    } else {
        let items: Vec<ListItem> = checks
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let status = field_text(&c["status"]);
                let color = match status.as_str() {
                    "ok" => Token::Success,
                    "warn" => Token::Warning,
                    "fail" => Token::Danger,
                    _ => Token::Muted,
                };
                let style = if i == estate.check_selected {
                    Style::default()
                        .fg(color.rgb())
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().fg(color.rgb())
                };
                // §12.3: one row per check. A handful of checks (e.g.
                // `git_surfaces`) carry a multi-line `detail` meant for the
                // Detail pane below, not for the list — embedding it here
                // verbatim would let one check's row swallow the vertical
                // space several others need, silently pushing later checks
                // off screen even though every one of them is still `ok`.
                // The list gets the first line as a summary; the full text
                // stays one selection away.
                let detail = field_text(&c["detail"]);
                let summary = detail.lines().next().unwrap_or("");
                let line = format!("[{:<4}] {:<20} {}", status, field_text(&c["name"]), summary);
                ListItem::new(line).style(style)
            })
            .collect();
        frame.render_widget(List::new(items), inner);
    }

    let detail_block = bordered("Detail");
    let detail_inner = detail_block.inner(detail_area);
    frame.render_widget(detail_block, detail_area);
    let detail = match estate.selected_check() {
        Some(check) => {
            let mut text = format!(
                "{}  [{}]\n\n{}",
                field_text(&check["name"]),
                field_text(&check["status"]),
                field_text(&check["detail"]),
            );
            if let Some(remedy) = check["remedy"].as_str() {
                text.push_str(&format!("\n\nremedy: {remedy}"));
            }
            if check["name"].as_str() == Some("disk_pressure") {
                text.push_str("\n\nr  view retained estate-wide state");
            }
            text
        }
        None => "healthy — no check to detail".to_string(),
    };
    frame.render_widget(
        Paragraph::new(detail).wrap(Wrap { trim: false }),
        detail_inner,
    );
}

/// §12.1's exact confirmation text — reused verbatim by
/// [`super::overlay::render`].
pub fn remove_repo_body(name: &str, last_error: Option<&str>) -> String {
    let mut body = format!(
        "Remove {name} from the estate?\n\n\
         This removes the estate declaration.\n\
         It does not delete repos/{name} from disk.\n\n\
         Enter confirms · Esc/q aborts, nothing is sent."
    );
    if let Some(error) = last_error {
        body.push_str(&format!("\n\nlast attempt failed: {error}"));
    }
    body
}

/// The add-repo form body, including the spinner/elapsed line while a clone
/// or verify is in flight (§12.1: "no fabricated clone percentage").
pub fn add_repo_body(form: &RepoForm, pending: Option<&PendingRepoAdd>) -> String {
    if let Some(pending) = pending {
        let mut body = format!(
            "Adding {}…\n\n  * {}s elapsed\n\nThis runs off the render loop; the screen stays live.",
            pending.name,
            pending.started.elapsed().as_secs(),
        );
        if let Some(error) = &form.last_error {
            body.push_str(&format!("\n\nlast attempt failed: {error}"));
        }
        return body;
    }
    let focus_marker = |focused: bool| if focused { " <" } else { "" };
    let field_marker = |focused: bool| if focused { "" } else { "_" };
    let name_focused = form_focused(form, RepoField::Name);
    let origin_focused = form_focused(form, RepoField::Origin);
    let instructions_focused = form_focused(form, RepoField::Instructions);
    let confirm_focused = form_focused(form, RepoField::Confirm);
    let mut body = format!(
        "Add a repository\n\n  \
         name          {}{}\n  \
         origin        {}{}\n  \
         instructions  {}{}\n\n  \
         [ confirm ]{}\n\n\
         Tab moves between fields · Enter cycles instructions/confirms · Esc/q aborts.",
        form.name,
        field_marker(name_focused),
        form.origin,
        field_marker(origin_focused),
        form.instructions.unwrap_or("(default)"),
        field_marker(instructions_focused),
        focus_marker(confirm_focused),
    );
    if let Some(error) = &form.last_error {
        body.push_str(&format!("\n\n{error}"));
    }
    body
}

fn form_focused(form: &RepoForm, field: RepoField) -> bool {
    REPO_FIELDS[form.focus_index()] == field
}

impl RepoForm {
    fn focus_index(&self) -> usize {
        self.focus
    }
}

/// The group edit/remove form body.
pub fn group_form_body(form: &GroupForm) -> String {
    match &form.mode {
        GroupOverlayMode::Edit => {
            let field_marker = |focused: bool| if focused { "" } else { "_" };
            let focus_marker = |focused: bool| if focused { " <" } else { "" };
            let name_focused = group_focused(form, GroupField::Name);
            let repos_focused = group_focused(form, GroupField::Repos);
            let brief_focused = group_focused(form, GroupField::Brief);
            let confirm_focused = group_focused(form, GroupField::Confirm);
            let mut body = format!(
                "Create or extend a group\n\n  \
                 name    {}{}\n  \
                 repos   {}{}  (comma/space-separated; adds to existing membership)\n  \
                 brief   {}{}\n\n  \
                 [ confirm ]{}\n\n\
                 Tab moves between fields · Enter on Confirm sends it · Esc/q aborts.",
                form.name,
                field_marker(name_focused),
                form.repos,
                field_marker(repos_focused),
                form.brief,
                field_marker(brief_focused),
                focus_marker(confirm_focused),
            );
            if let Some(error) = &form.last_error {
                body.push_str(&format!("\n\n{error}"));
            }
            body
        }
        GroupOverlayMode::Remove { name } => {
            let field_marker = |focused: bool| if focused { "" } else { "_" };
            let focus_marker = |focused: bool| if focused { " <" } else { "" };
            let repos_focused = group_focused(form, GroupField::Repos);
            let confirm_focused = group_focused(form, GroupField::Confirm);
            let mut body = format!(
                "Remove from group {name}?\n\n  \
                 members to remove   {}{}  (blank removes the whole group)\n\n  \
                 [ confirm ]{}\n\n\
                 Tab moves between fields · Enter on Confirm sends it · Esc/q aborts.",
                form.repos,
                field_marker(repos_focused),
                focus_marker(confirm_focused),
            );
            if let Some(error) = &form.last_error {
                body.push_str(&format!("\n\n{error}"));
            }
            body
        }
    }
}

fn group_focused(form: &GroupForm, field: GroupField) -> bool {
    GROUP_FIELDS[form.focus] == field
}

/// The retained-state preview body (§12.4): estate-wide, from the existing
/// `GET /v1/retained` read — never a new route.
pub fn retained_body(estate: &EstateScreen) -> String {
    if estate.retained.is_empty() {
        return "nothing retained across the estate.\n\nEsc/q closes this panel.".to_string();
    }
    let mut lines = vec!["Retained state (estate-wide)\n".to_string()];
    for (i, entry) in estate.retained.iter().enumerate() {
        let marker = if i == estate.retained_selected {
            ">"
        } else {
            " "
        };
        lines.push(format!(
            "{marker} {:<12} {:<20} {:<10} {}b  {}",
            field_text(&entry["work_id"]),
            field_text(&entry["repository"]),
            field_text(&entry["disposition"]),
            field_text(&entry["bytes"]),
            field_text(&entry["path"]),
        ));
    }
    lines.push(String::new());
    lines.push("j/k move · p reap the selected Work's retained state · Esc/q closes".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// A picked estate for tests that exercise the existing sub-tab
    /// behavior (W4d deliverable 3 gates every one of them on `picked` —
    /// the picker itself is exercised separately below).
    fn some_picked() -> Option<PickedEstate> {
        Some(PickedEstate {
            name: "acme".to_string(),
            root: std::path::PathBuf::from("/estates/acme"),
        })
    }

    fn repos_estate() -> EstateScreen {
        EstateScreen {
            picked: some_picked(),
            repos: vec![
                json!({"name": "svc-a", "path": "repos/svc-a", "origin": "git@x:svc-a", "instructions": "suppress"}),
                json!({"name": "svc-b", "path": "repos/svc-b", "origin": null, "instructions": "local"}),
            ],
            groups: vec![json!({"name": "core", "repos": ["svc-a"], "brief": "core services"})],
            ..EstateScreen::default()
        }
    }

    #[test]
    fn tab_key_cycles_repositories_groups_health() {
        let mut estate = EstateScreen {
            picked: some_picked(),
            ..EstateScreen::default()
        };
        assert_eq!(estate.tab, EstateTab::Repositories);
        estate.on_key(KeyCode::Tab);
        assert_eq!(estate.tab, EstateTab::Groups);
        estate.on_key(KeyCode::Tab);
        assert_eq!(estate.tab, EstateTab::Health);
        estate.on_key(KeyCode::BackTab);
        assert_eq!(estate.tab, EstateTab::Groups);
    }

    #[test]
    fn j_k_move_the_selection_on_the_active_tab() {
        let mut estate = repos_estate();
        assert_eq!(estate.repo_selected, 0);
        estate.on_key(KeyCode::Char('j'));
        assert_eq!(estate.repo_selected, 1);
        estate.on_key(KeyCode::Char('k'));
        assert_eq!(estate.repo_selected, 0);
    }

    #[test]
    fn a_opens_add_repo_on_repositories_and_group_edit_on_groups() {
        let mut estate = repos_estate();
        assert_eq!(
            estate.on_key(KeyCode::Char('a')),
            EstateOutcome::OpenAddRepo
        );
        estate.tab = EstateTab::Groups;
        assert_eq!(
            estate.on_key(KeyCode::Char('a')),
            EstateOutcome::OpenGroupEdit
        );
    }

    #[test]
    fn x_opens_remove_for_the_selected_repo_or_group() {
        let mut estate = repos_estate();
        assert_eq!(
            estate.on_key(KeyCode::Char('x')),
            EstateOutcome::OpenRemoveRepo("svc-a".to_string())
        );
        estate.tab = EstateTab::Groups;
        assert_eq!(
            estate.on_key(KeyCode::Char('x')),
            EstateOutcome::OpenGroupRemove("core".to_string())
        );
    }

    #[test]
    fn r_opens_retained_only_for_the_disk_pressure_check() {
        let mut estate = EstateScreen {
            tab: EstateTab::Health,
            picked: some_picked(),
            doctor: json!({"checks": [
                {"name": "git", "status": "ok", "detail": "found"},
                {"name": "disk_pressure", "status": "warn", "detail": "82% full"},
            ]}),
            ..EstateScreen::default()
        };
        assert_eq!(estate.on_key(KeyCode::Char('r')), EstateOutcome::None);
        estate.on_key(KeyCode::Char('j'));
        assert_eq!(
            estate.on_key(KeyCode::Char('r')),
            EstateOutcome::OpenRetained
        );
    }

    #[test]
    fn clamp_pulls_a_selection_back_when_a_refresh_shrinks_the_list() {
        let mut estate = repos_estate();
        estate.repo_selected = 5;
        estate.clamp();
        assert_eq!(estate.repo_selected, 1);
    }

    fn two_estates() -> EstateScreen {
        EstateScreen {
            estates: vec![
                json!({"name": "payments", "root": "/estates/payments"}),
                json!({"name": "billing", "root": "/estates/billing"}),
            ],
            ..EstateScreen::default()
        }
    }

    #[test]
    fn nothing_picked_shows_only_the_picker_and_ignores_sub_tab_keys() {
        let mut estate = two_estates();
        // Tab is Repositories/Groups/Health's own key once picked; with
        // nothing picked yet it must be a no-op, not a tab switch.
        estate.on_key(KeyCode::Tab);
        assert_eq!(estate.tab, EstateTab::Repositories);
        assert!(estate.picked.is_none());
    }

    #[test]
    fn up_down_browse_the_picker_and_enter_picks_the_highlighted_estate() {
        let mut estate = two_estates();
        assert_eq!(estate.picker_selected, 0);
        estate.on_key(KeyCode::Down);
        assert_eq!(estate.picker_selected, 1);
        estate.on_key(KeyCode::Up);
        assert_eq!(estate.picker_selected, 0);

        estate.on_key(KeyCode::Down);
        estate.on_key(KeyCode::Enter);
        assert_eq!(
            estate.picked,
            Some(PickedEstate {
                name: "billing".to_string(),
                root: std::path::PathBuf::from("/estates/billing"),
            })
        );
    }

    #[test]
    fn p_returns_a_picked_screen_to_the_picker() {
        let mut estate = repos_estate();
        assert!(estate.picked.is_some());
        estate.on_key(KeyCode::Char('p'));
        assert!(estate.picked.is_none());
    }

    #[test]
    fn auto_pick_is_idempotent_over_an_unchanged_root() {
        let mut estate = EstateScreen::default();
        let root = std::path::Path::new("/estates/acme");
        estate.auto_pick(root, "acme");
        estate.auto_pick(root, "renamed"); // same root, e.g. a later manifest edit
        assert_eq!(
            estate.picked.as_ref().map(|p| p.name.as_str()),
            Some("acme"),
            "an unchanged root is not re-picked, even if the display name changed upstream"
        );
    }

    #[test]
    fn auto_pick_repicks_on_a_changed_root() {
        let mut estate = EstateScreen::default();
        estate.auto_pick(std::path::Path::new("/estates/a"), "a");
        estate.auto_pick(std::path::Path::new("/estates/b"), "b");
        assert_eq!(
            estate.picked,
            Some(PickedEstate {
                name: "b".to_string(),
                root: std::path::PathBuf::from("/estates/b"),
            })
        );
    }

    #[test]
    fn remove_repo_body_states_the_declaration_is_removed_not_the_disk_copy() {
        let body = remove_repo_body("svc-a", None);
        assert!(body.contains("This removes the estate declaration."));
        assert!(body.contains("It does not delete repos/svc-a from disk."));
    }

    #[test]
    fn add_repo_form_tab_cycles_through_fields_and_confirm_submits() {
        let mut form = RepoForm::default();
        form.on_key(KeyCode::Char('a').into());
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Char('b').into());
        assert_eq!(form.name, "a");
        assert_eq!(form.origin, "b");
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Enter.into()); // cycles instructions to "local"
        assert_eq!(form.instructions, Some("local"));
        form.on_key(KeyCode::Tab.into());
        let outcome = form.on_key(KeyCode::Enter.into());
        assert_eq!(
            outcome,
            RepoFormOutcome::Submit {
                name: "a".to_string(),
                origin: Some("b".to_string()),
                instructions: Some("local".to_string()),
            }
        );
    }

    #[test]
    fn add_repo_form_refuses_an_empty_name() {
        let mut form = RepoForm::default();
        form.reset_for_add();
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Tab.into());
        let outcome = form.on_key(KeyCode::Enter.into());
        assert_eq!(outcome, RepoFormOutcome::None);
        assert!(form.last_error.is_some());
    }

    #[test]
    fn remove_repo_form_confirms_with_the_prefilled_name() {
        let mut form = RepoForm::default();
        form.reset_for_remove("svc-a".to_string());
        let outcome = form.on_key(KeyCode::Enter.into());
        assert_eq!(outcome, RepoFormOutcome::ConfirmRemove("svc-a".to_string()));
    }

    #[test]
    fn group_edit_form_submits_split_repos() {
        let mut form = GroupForm::default();
        form.reset_for_edit(None);
        form.name = "core".to_string();
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Char('a').into());
        form.on_key(KeyCode::Char(',').into());
        form.on_key(KeyCode::Char('b').into());
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Tab.into());
        let outcome = form.on_key(KeyCode::Enter.into());
        assert_eq!(
            outcome,
            GroupFormOutcome::Submit {
                name: "core".to_string(),
                repos: vec!["a".to_string(), "b".to_string()],
                brief: None,
            }
        );
    }

    #[test]
    fn group_remove_form_defaults_to_whole_group_when_repos_left_blank() {
        let mut form = GroupForm::default();
        form.reset_for_remove("core".to_string());
        // Name(0, disabled in Remove mode) -> Repos(1) -> Brief(2, also
        // disabled) -> Confirm(3): three Tabs from the default focus reach
        // Confirm without ever typing into the (blank) repos field.
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Tab.into());
        form.on_key(KeyCode::Tab.into());
        let outcome = form.on_key(KeyCode::Enter.into());
        assert_eq!(
            outcome,
            GroupFormOutcome::ConfirmRemove {
                name: "core".to_string(),
                repos: Vec::new(),
            }
        );
    }

    #[test]
    fn retained_body_lists_estate_wide_entries_not_just_one_work() {
        let estate = EstateScreen {
            retained: vec![
                json!({"work_id": "01A", "repository": "svc-a", "disposition": "retained_dirty", "bytes": 120, "path": "/data/a"}),
                json!({"work_id": "01B", "repository": "svc-b", "disposition": "retained_dirty", "bytes": 40, "path": "/data/b"}),
            ],
            ..EstateScreen::default()
        };
        let body = retained_body(&estate);
        assert!(body.contains("01A"));
        assert!(body.contains("01B"));
    }
}
