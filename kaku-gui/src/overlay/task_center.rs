use crate::overlay::selector::{matcher_pattern, matcher_score};
use crate::termwindow::TermWindowNotif;
use mux::task_center::{TaskCenterEntry, TaskCenterKind, TaskCenterSource};
use mux::termwiztermtab::TermWizTerminal;
use mux::Mux;
use termwiz::cell::{unicode_column_width, AttributeChange, CellAttributes};
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers, MouseButtons, MouseEvent};
use termwiz::surface::{Change, Position};
use termwiz::terminal::Terminal;
use termwiz_funcs::truncate_right;
use window::WindowOps;

const ROW_OVERHEAD: usize = 4;
const ROW_START_Y: usize = ROW_OVERHEAD - 1;
const RAIL_WIDTH: usize = 18;
const CONTENT_START_X: usize = RAIL_WIDTH + 2;
const ICON_FOCUS: &str = "⏎";
const ICON_CLEAR: &str = "●";
const ICON_RERUN: &str = "↺";
const ICON_HOLD: &str = "◎";
const ICON_STATUS: &str = "◇";
const ICON_PROGRESS: &str = "◔";
const ICON_DELETE: &str = "⌫";
const ICON_WORKSPACE: &str = "⌂";
const ICON_SOURCE: &str = "◦";
const ICON_UNREAD: &str = "●";
const ICON_FAILED: &str = "✕";
const ICON_RUNNING: &str = "◌";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowAction {
    Focus,
    ClearUnread,
    Rerun,
    ToggleRemainOnExit,
    EditStatus,
    EditProgress,
    ClearMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActionTarget {
    action: RowAction,
    start: usize,
    end: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RowLayout {
    text: String,
    action_targets: Vec<ActionTarget>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PromptKind {
    Status,
    Progress,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PromptState {
    kind: PromptKind,
    workspace: String,
    input: String,
    error: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ConfirmState {
    workspace: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum OverlayMode {
    List,
    Prompt(PromptState),
    Confirm(ConfirmState),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TaskCenterSection {
    All,
    Inbox,
    Failed,
    Running,
    Metadata,
    Tasks,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClickState {
    row: usize,
    streak: usize,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct TaskCenterFilters {
    unread: bool,
    failed: bool,
    running: bool,
    workspace: Option<String>,
    source: Option<TaskCenterSource>,
    kind: Option<TaskCenterKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedTaskCenterQuery {
    filters: TaskCenterFilters,
    free_text: String,
}

pub struct TaskCenterOverlay {
    entries: Vec<TaskCenterEntry>,
    filtered_entries: Vec<TaskCenterEntry>,
    filter_term: String,
    filters: TaskCenterFilters,
    active_idx: usize,
    top_row: usize,
    max_items: usize,
    window: Option<::window::Window>,
    mode: OverlayMode,
    click_state: Option<ClickState>,
    selected_section: TaskCenterSection,
}

impl TaskCenterOverlay {
    pub fn new(entries: Vec<TaskCenterEntry>) -> Self {
        let mut overlay = Self {
            entries,
            filtered_entries: Vec::new(),
            filter_term: String::new(),
            filters: TaskCenterFilters::default(),
            active_idx: 0,
            top_row: 0,
            max_items: 0,
            window: None,
            mode: OverlayMode::List,
            click_state: None,
            selected_section: TaskCenterSection::All,
        };
        overlay.update_filter();
        overlay
    }

    fn parse_query(query: &str) -> ParsedTaskCenterQuery {
        let mut filters = TaskCenterFilters::default();
        let mut free_text = Vec::new();

        for token in query.split_whitespace() {
            if token.eq_ignore_ascii_case("unread") {
                filters.unread = true;
            } else if token.eq_ignore_ascii_case("failed") {
                filters.failed = true;
            } else if token.eq_ignore_ascii_case("running") {
                filters.running = true;
            } else if let Some(workspace) = token.strip_prefix("workspace:") {
                if !workspace.is_empty() {
                    filters.workspace = Some(workspace.to_string());
                }
            } else if let Some(source) = token.strip_prefix("source:") {
                filters.source = parse_source(source);
            } else if let Some(kind) = token.strip_prefix("kind:") {
                filters.kind = parse_kind(kind);
            } else {
                free_text.push(token.to_string());
            }
        }

        ParsedTaskCenterQuery {
            filters,
            free_text: free_text.join(" "),
        }
    }

    fn searchable_text(entry: &TaskCenterEntry) -> String {
        let mut parts = vec![
            entry.label.clone(),
            entry.workspace.clone(),
            format!("{:?}", entry.source).to_lowercase(),
            format!("{:?}", entry.kind).to_lowercase(),
        ];
        if let Some(source_label) = &entry.source_label {
            parts.push(source_label.clone());
        }
        if let Some(kind_label) = &entry.kind_label {
            parts.push(kind_label.clone());
        }
        if let Some(status) = &entry.workspace_status {
            parts.push(status.clone());
        }
        if let Some(progress) = entry.workspace_progress {
            parts.push(progress.to_string());
        }
        parts.join(" ")
    }

    fn matches_filters(entry: &TaskCenterEntry, filters: &TaskCenterFilters) -> bool {
        if filters.unread && entry.unread_count == 0 {
            return false;
        }
        if filters.failed && !entry.is_failed {
            return false;
        }
        if filters.running && !entry.is_running {
            return false;
        }
        if let Some(workspace) = &filters.workspace {
            if &entry.workspace != workspace {
                return false;
            }
        }
        if let Some(source) = &filters.source {
            if &entry.source != source {
                return false;
            }
        }
        if let Some(kind) = &filters.kind {
            if &entry.kind != kind {
                return false;
            }
        }
        true
    }

    fn matches_selected_section(&self, entry: &TaskCenterEntry) -> bool {
        match self.selected_section {
            TaskCenterSection::All => true,
            TaskCenterSection::Inbox => entry.unread_count > 0,
            TaskCenterSection::Failed => entry.is_failed,
            TaskCenterSection::Running => entry.is_running,
            TaskCenterSection::Metadata => {
                entry.workspace_status.is_some() || entry.workspace_progress.is_some()
            }
            TaskCenterSection::Tasks => matches!(entry.kind, TaskCenterKind::Pane),
        }
    }

    fn update_filter(&mut self) {
        let parsed = Self::parse_query(&self.filter_term);
        self.filters = parsed.filters;
        self.filtered_entries = self
            .entries
            .iter()
            .filter(|entry| self.matches_selected_section(entry))
            .filter(|entry| Self::matches_filters(entry, &self.filters))
            .cloned()
            .collect();

        if !parsed.free_text.is_empty() {
            let pattern = matcher_pattern(&parsed.free_text);
            let mut scored_entries = self
                .filtered_entries
                .drain(..)
                .filter_map(|entry| {
                    let score = matcher_score(&pattern, &Self::searchable_text(&entry))?;
                    Some((score, entry))
                })
                .collect::<Vec<_>>();
            scored_entries.sort_by_key(|(score, _)| *score);
            scored_entries.reverse();
            self.filtered_entries = scored_entries.into_iter().map(|(_, entry)| entry).collect();
        }

        self.active_idx = 0;
        self.top_row = 0;
        self.click_state = None;
    }

    fn active_filter_summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(self.selected_section.label().to_ascii_lowercase());
        if self.filters.unread {
            parts.push("unread".to_string());
        }
        if self.filters.failed {
            parts.push("failed".to_string());
        }
        if self.filters.running {
            parts.push("running".to_string());
        }
        if let Some(workspace) = &self.filters.workspace {
            parts.push(format!("workspace:{workspace}"));
        }
        if let Some(source) = &self.filters.source {
            parts.push(format!("source:{}", format!("{source:?}").to_lowercase()));
        }
        if let Some(kind) = &self.filters.kind {
            parts.push(format!("kind:{}", format!("{kind:?}").to_lowercase()));
        }
        if parts.is_empty() {
            "all".to_string()
        } else {
            parts.join(" ")
        }
    }

    fn base_row_text(entry: &TaskCenterEntry) -> String {
        let mut suffixes = Vec::new();
        suffixes.push(format!("{ICON_WORKSPACE} {}", entry.workspace));
        let source = entry
            .source_label
            .clone()
            .unwrap_or_else(|| format!("{:?}", entry.source).to_lowercase());
        suffixes.push(format!("{ICON_SOURCE} {source}"));
        if let Some(kind) = &entry.kind_label {
            suffixes.push(kind.clone());
        }
        if entry.unread_count > 0 {
            suffixes.push(format!("{ICON_UNREAD}{}", entry.unread_count));
        }
        if entry.is_failed {
            suffixes.push(ICON_FAILED.to_string());
        }
        if entry.is_running {
            suffixes.push(ICON_RUNNING.to_string());
        }
        if entry.rerun_available {
            suffixes.push(ICON_RERUN.to_string());
        }
        if let Some(status) = &entry.workspace_status {
            suffixes.push(format!("{ICON_STATUS} {status}"));
        }
        if let Some(progress) = entry.workspace_progress {
            suffixes.push(format!("{ICON_PROGRESS} {progress}%"));
        }
        format!("{}  {}", entry.label, suffixes.join("  "))
    }

    fn selected_entry_remain_on_exit(&self) -> Option<bool> {
        self.selected_entry()
            .as_ref()
            .and_then(Self::remain_on_exit_state_for_entry)
    }

    fn remain_on_exit_state_for_entry(entry: &TaskCenterEntry) -> Option<bool> {
        if let Some(pane_id) = entry.pane_id {
            if let Some(mux) = Mux::try_get() {
                if let Some(record) = mux.task_pane_record(pane_id) {
                    return Some(record.remain_on_exit);
                }
            }
        }

        entry
            .kind_label
            .as_deref()
            .filter(|label| *label == "task-pane")
            .map(|_| false)
    }

    fn can_toggle_remain_on_exit(&self) -> bool {
        self.selected_entry_remain_on_exit().is_some()
    }

    fn can_edit_metadata(&self) -> bool {
        self.selected_entry().is_some()
    }

    fn can_clear_metadata(&self) -> bool {
        self.selected_entry()
            .map(|entry| entry.workspace_status.is_some() || entry.workspace_progress.is_some())
            .unwrap_or(false)
    }

    fn action_targets_for_entry(entry: &TaskCenterEntry) -> Vec<(RowAction, String)> {
        let mut actions = vec![(RowAction::Focus, format!("[{ICON_FOCUS} open]"))];

        if entry.unread_count > 0 && !entry.notification_ids.is_empty() {
            actions.push((RowAction::ClearUnread, format!("[{ICON_CLEAR} clear]")));
        }
        if entry.is_failed && entry.rerun_available {
            actions.push((RowAction::Rerun, format!("[{ICON_RERUN} rerun]")));
        }
        if let Some(remain_on_exit) = Self::remain_on_exit_state_for_entry(entry) {
            actions.push((
                RowAction::ToggleRemainOnExit,
                format!(
                    "[{ICON_HOLD} hold:{}]",
                    if remain_on_exit { "on" } else { "off" }
                ),
            ));
        }
        actions.push((RowAction::EditStatus, format!("[{ICON_STATUS} status]")));
        actions.push((
            RowAction::EditProgress,
            format!("[{ICON_PROGRESS} progress]"),
        ));
        if entry.workspace_status.is_some() || entry.workspace_progress.is_some() {
            actions.push((RowAction::ClearMetadata, format!("[{ICON_DELETE} clear]")));
        }

        actions
    }

    fn row_layout(entry: &TaskCenterEntry, active: bool) -> RowLayout {
        let mut text = Self::base_row_text(entry);
        let mut action_targets = Vec::new();

        if active {
            let mut cursor = unicode_column_width(&text, None);
            for (action, label) in Self::action_targets_for_entry(entry) {
                text.push(' ');
                cursor += 1;
                let start = cursor;
                text.push_str(&label);
                cursor += unicode_column_width(&label, None);
                action_targets.push(ActionTarget {
                    action,
                    start,
                    end: cursor,
                });
            }
        }

        RowLayout {
            text,
            action_targets,
        }
    }

    fn format_row(entry: &TaskCenterEntry) -> String {
        Self::base_row_text(entry)
    }

    fn empty_state_message(&self) -> &'static str {
        if self.entries.is_empty() {
            "No task-center entries yet"
        } else {
            "No entries match the current task-center query"
        }
    }

    fn selected_entry(&self) -> Option<TaskCenterEntry> {
        self.filtered_entries.get(self.active_idx).cloned()
    }

    fn can_clear_unread(&self) -> bool {
        self.selected_entry()
            .map(|entry| entry.unread_count > 0 && !entry.notification_ids.is_empty())
            .unwrap_or(false)
    }

    fn can_rerun(&self) -> bool {
        self.selected_entry()
            .map(|entry| entry.is_failed && entry.rerun_available)
            .unwrap_or(false)
    }

    fn header_lines(&self) -> [String; 3] {
        [
            "Task Center".to_string(),
            format!(
                "scope {}  / {}",
                self.active_filter_summary(),
                if self.filter_term.is_empty() {
                    "type to filter"
                } else {
                    &self.filter_term
                }
            ),
            format!(
                "←/→ rail  {ICON_FOCUS} focus  click select  dbl-click focus  {ICON_RERUN} rerun  {ICON_HOLD} hold  {ICON_STATUS} status  {ICON_PROGRESS} progress  esc close"
            ),
        ]
    }

    fn section_entries(&self) -> Vec<(TaskCenterSection, String)> {
        TaskCenterSection::ordered()
            .iter()
            .copied()
            .map(|section| {
                let count = self
                    .entries
                    .iter()
                    .filter(|entry| self.section_matches(section, entry))
                    .count();
                (
                    section,
                    format!("{} {} {}", section.icon(), section.label(), count),
                )
            })
            .collect()
    }

    fn section_matches(&self, section: TaskCenterSection, entry: &TaskCenterEntry) -> bool {
        match section {
            TaskCenterSection::All => true,
            TaskCenterSection::Inbox => entry.unread_count > 0,
            TaskCenterSection::Failed => entry.is_failed,
            TaskCenterSection::Running => entry.is_running,
            TaskCenterSection::Metadata => {
                entry.workspace_status.is_some() || entry.workspace_progress.is_some()
            }
            TaskCenterSection::Tasks => matches!(entry.kind, TaskCenterKind::Pane),
        }
    }

    fn set_selected_section(&mut self, section: TaskCenterSection) {
        if self.selected_section != section {
            self.selected_section = section;
            self.update_filter();
        }
    }

    fn move_section_left(&mut self) {
        let sections = TaskCenterSection::ordered();
        let idx = sections
            .iter()
            .position(|section| *section == self.selected_section)
            .unwrap_or(0);
        if idx > 0 {
            self.set_selected_section(sections[idx - 1]);
        }
    }

    fn move_section_right(&mut self) {
        let sections = TaskCenterSection::ordered();
        let idx = sections
            .iter()
            .position(|section| *section == self.selected_section)
            .unwrap_or(0);
        if idx + 1 < sections.len() {
            self.set_selected_section(sections[idx + 1]);
        }
    }

    fn visible_row_capacity(&self) -> usize {
        self.max_items.max(1)
    }

    fn ensure_active_row_visible(&mut self) {
        let capacity = self.visible_row_capacity();
        if self.active_idx < self.top_row {
            self.top_row = self.active_idx;
        } else if self.active_idx >= self.top_row + capacity {
            self.top_row = self.active_idx + 1 - capacity;
        }
    }

    fn move_selection_up(&mut self) {
        self.active_idx = self.active_idx.saturating_sub(1);
        self.ensure_active_row_visible();
        self.click_state = None;
    }

    fn move_selection_down(&mut self) {
        if !self.filtered_entries.is_empty() {
            self.active_idx = (self.active_idx + 1).min(self.filtered_entries.len() - 1);
            self.ensure_active_row_visible();
        }
        self.click_state = None;
    }

    fn sync_entries_from_mux(&mut self) {
        self.entries = Mux::get().task_center_snapshot();
        self.update_filter();
    }

    fn trigger_focus(&self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(entry) = self.selected_entry() else {
            return;
        };
        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
            let _ = term_window.focus_task_center_entry(&entry);
        })));
        window.focus();
    }

    fn trigger_clear_unread(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(entry) = self.selected_entry() else {
            return;
        };
        if entry.unread_count == 0 || entry.notification_ids.is_empty() {
            return;
        }

        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
            term_window.clear_task_center_entry_unread(&entry);
        })));
        self.sync_entries_from_mux();
    }

    fn trigger_rerun(&self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(entry) = self.selected_entry() else {
            return;
        };
        if !entry.is_failed || !entry.rerun_available {
            return;
        }

        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
            let _ = term_window.rerun_task_center_entry(&entry);
        })));
        window.focus();
    }

    fn trigger_toggle_remain_on_exit(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let Some(entry) = self.selected_entry() else {
            return;
        };
        let Some(remain_on_exit) = Self::remain_on_exit_state_for_entry(&entry) else {
            return;
        };

        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
            term_window.set_task_center_entry_remain_on_exit(&entry, !remain_on_exit);
        })));
        self.sync_entries_from_mux();
    }

    fn open_status_prompt(&mut self) {
        if let Some(entry) = self.selected_entry() {
            self.mode = OverlayMode::Prompt(PromptState {
                kind: PromptKind::Status,
                workspace: entry.workspace,
                input: entry.workspace_status.unwrap_or_default(),
                error: None,
            });
        }
    }

    fn open_progress_prompt(&mut self) {
        if let Some(entry) = self.selected_entry() {
            self.mode = OverlayMode::Prompt(PromptState {
                kind: PromptKind::Progress,
                workspace: entry.workspace,
                input: entry
                    .workspace_progress
                    .map(|progress| progress.to_string())
                    .unwrap_or_default(),
                error: None,
            });
        }
    }

    fn open_clear_metadata_confirm(&mut self) {
        if let Some(entry) = self.selected_entry() {
            self.mode = OverlayMode::Confirm(ConfirmState {
                workspace: entry.workspace,
            });
        }
    }

    fn submit_prompt(&mut self, prompt: PromptState) {
        let Some(window) = self.window.clone() else {
            self.mode = OverlayMode::List;
            return;
        };
        let workspace = prompt.workspace.clone();
        match prompt.kind {
            PromptKind::Status => {
                let status = prompt.input.trim().to_string();
                window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                    if status.is_empty() {
                        term_window.clear_workspace_status_via_client(Some(workspace));
                    } else {
                        term_window.set_workspace_status_via_client(workspace, status);
                    }
                })));
                self.mode = OverlayMode::List;
                self.sync_entries_from_mux();
            }
            PromptKind::Progress => {
                let input = prompt.input.trim();
                if input.is_empty() {
                    window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                        term_window.clear_workspace_progress_via_client(Some(workspace));
                    })));
                    self.mode = OverlayMode::List;
                    self.sync_entries_from_mux();
                    return;
                }

                match input.parse::<u8>() {
                    Ok(value) if value <= 100 => {
                        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
                            term_window.set_workspace_progress_via_client(workspace, value);
                        })));
                        self.mode = OverlayMode::List;
                        self.sync_entries_from_mux();
                    }
                    _ => {
                        self.mode = OverlayMode::Prompt(PromptState {
                            error: Some("Progress must be a number from 0 to 100".to_string()),
                            ..prompt
                        });
                    }
                }
            }
        }
    }

    fn confirm_clear_metadata(&mut self) {
        let Some(window) = self.window.clone() else {
            self.mode = OverlayMode::List;
            return;
        };
        let OverlayMode::Confirm(confirm) = &self.mode else {
            return;
        };
        let workspace = confirm.workspace.clone();
        window.notify(TermWindowNotif::Apply(Box::new(move |term_window| {
            term_window.clear_workspace_status_via_client(Some(workspace.clone()));
            term_window.clear_workspace_progress_via_client(Some(workspace));
        })));
        self.mode = OverlayMode::List;
        self.sync_entries_from_mux();
    }

    fn render_list(&mut self, term: &mut TermWizTerminal) -> termwiz::Result<()> {
        let size = term.get_screen_size()?;
        let rail_width = RAIL_WIDTH.min(size.cols.saturating_sub(4));
        let content_start_x = (rail_width + 2).min(size.cols.saturating_sub(1));
        let max_width = size.cols.saturating_sub(content_start_x + 1);
        self.max_items = size.rows.saturating_sub(ROW_OVERHEAD);

        let header = self.header_lines();
        let mut changes = vec![
            Change::ClearScreen(termwiz::color::ColorAttribute::Default),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(0),
            },
            Change::Text(format!("{}\r\n", truncate_right(&header[0], max_width))),
            Change::Text(format!("{}\r\n", truncate_right(&header[1], max_width))),
            Change::Text(truncate_right(&header[2], max_width).to_string().into()),
            Change::Text("\r\n".to_string()),
            Change::AllAttributes(CellAttributes::default()),
        ];

        self.ensure_active_row_visible();

        let section_entries = self.section_entries();
        let visible_rows = self.visible_row_capacity();
        for row_num in 0..visible_rows {
            let row_y = ROW_START_Y + row_num;

            if let Some((section, label)) = section_entries.get(row_num) {
                changes.push(Change::CursorPosition {
                    x: Position::Absolute(0),
                    y: Position::Absolute(row_y),
                });
                if *section == self.selected_section {
                    changes.push(AttributeChange::Reverse(true).into());
                }
                changes.push(Change::Text(
                    truncate_right(label, rail_width).to_string().into(),
                ));
                if *section == self.selected_section {
                    changes.push(AttributeChange::Reverse(false).into());
                }
            }

            changes.push(Change::CursorPosition {
                x: Position::Absolute(content_start_x),
                y: Position::Absolute(row_y),
            });

            if let Some((entry_idx, entry)) = self
                .filtered_entries
                .iter()
                .enumerate()
                .skip(self.top_row)
                .nth(row_num)
            {
                if entry_idx == self.active_idx {
                    changes.push(AttributeChange::Reverse(true).into());
                }
                let row = Self::row_layout(entry, entry_idx == self.active_idx);
                changes.push(Change::Text(
                    truncate_right(&row.text, max_width).to_string().into(),
                ));
                if entry_idx == self.active_idx {
                    changes.push(AttributeChange::Reverse(false).into());
                }
            } else if row_num == 0 && self.filtered_entries.is_empty() {
                changes.push(Change::Text(
                    truncate_right(self.empty_state_message(), max_width)
                        .to_string()
                        .into(),
                ));
            }
        }

        term.render(&changes)
    }

    fn render_prompt(
        &mut self,
        term: &mut TermWizTerminal,
        prompt: &PromptState,
    ) -> termwiz::Result<()> {
        let size = term.get_screen_size()?;
        let max_width = size.cols.saturating_sub(2);
        let title = match prompt.kind {
            PromptKind::Status => format!("{ICON_STATUS} status  {}", prompt.workspace),
            PromptKind::Progress => format!("{ICON_PROGRESS} progress  {}", prompt.workspace),
        };
        let current = if prompt.input.is_empty() {
            "(empty)".to_string()
        } else {
            prompt.input.clone()
        };
        let help = match prompt.kind {
            PromptKind::Status => "Enter save  Esc cancel  blank clears",
            PromptKind::Progress => "Enter save  Esc cancel  blank clears  0-100",
        };
        let prompt_label = match prompt.kind {
            PromptKind::Status => "status> ",
            PromptKind::Progress => "progress> ",
        };
        let mut changes = vec![
            Change::ClearScreen(termwiz::color::ColorAttribute::Default),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(0),
            },
            Change::Text(format!("{}\r\n", truncate_right(&title, max_width))),
            Change::Text(format!(
                "{}\r\n",
                truncate_right(&format!("current: {current}"), max_width)
            )),
            Change::Text(format!("{}\r\n", truncate_right(help, max_width))),
            Change::Text(format!(
                "{}\r\n",
                truncate_right(&format!("{prompt_label}{}", prompt.input), max_width)
            )),
        ];
        if let Some(error) = &prompt.error {
            changes.push(Change::Text(format!(
                "{}\r\n",
                truncate_right(error, max_width)
            )));
        }
        term.render(&changes)
    }

    fn render_confirm(
        &mut self,
        term: &mut TermWizTerminal,
        confirm: &ConfirmState,
    ) -> termwiz::Result<()> {
        let size = term.get_screen_size()?;
        let max_width = size.cols.saturating_sub(2);
        let changes = vec![
            Change::ClearScreen(termwiz::color::ColorAttribute::Default),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(0),
            },
            Change::Text(format!("{ICON_DELETE} clear metadata\r\n")),
            Change::Text(format!(
                "{}\r\n",
                truncate_right(
                    &format!(
                        "Clear the visible status/progress summary for workspace {}?",
                        confirm.workspace
                    ),
                    max_width
                )
            )),
            Change::Text("Enter/Y=clear Esc/N=cancel\r\n".to_string()),
        ];
        term.render(&changes)
    }

    fn render(&mut self, term: &mut TermWizTerminal) -> termwiz::Result<()> {
        match self.mode.clone() {
            OverlayMode::List => self.render_list(term),
            OverlayMode::Prompt(prompt) => self.render_prompt(term, &prompt),
            OverlayMode::Confirm(confirm) => self.render_confirm(term, &confirm),
        }
    }

    fn row_action_at(&self, row_idx: usize, x: usize) -> Option<RowAction> {
        let entry = self.filtered_entries.get(row_idx)?;
        let layout = Self::row_layout(entry, true);
        layout
            .action_targets
            .into_iter()
            .find(|target| x >= target.start && x < target.end)
            .map(|target| target.action)
    }

    fn section_at_row(&self, row: usize) -> Option<TaskCenterSection> {
        self.section_entries().get(row).map(|(section, _)| *section)
    }

    fn trigger_row_action(&mut self, action: RowAction) -> bool {
        match action {
            RowAction::Focus => {
                self.trigger_focus();
                true
            }
            RowAction::ClearUnread => {
                self.trigger_clear_unread();
                false
            }
            RowAction::Rerun => {
                self.trigger_rerun();
                true
            }
            RowAction::ToggleRemainOnExit => {
                self.trigger_toggle_remain_on_exit();
                false
            }
            RowAction::EditStatus => {
                self.open_status_prompt();
                false
            }
            RowAction::EditProgress => {
                self.open_progress_prompt();
                false
            }
            RowAction::ClearMetadata => {
                self.open_clear_metadata_confirm();
                false
            }
        }
    }

    fn handle_list_mouse_event(&mut self, event: MouseEvent) -> bool {
        let MouseEvent {
            x,
            y,
            mouse_buttons,
            ..
        } = event;

        if mouse_buttons.contains(MouseButtons::VERT_WHEEL) {
            if mouse_buttons.contains(MouseButtons::WHEEL_POSITIVE) {
                self.top_row = self.top_row.saturating_sub(1);
            } else {
                self.top_row += 1;
                self.top_row = self.top_row.min(
                    self.filtered_entries
                        .len()
                        .saturating_sub(self.visible_row_capacity()),
                );
            }
            if y as usize >= ROW_START_Y {
                let row = self.top_row + y as usize - ROW_START_Y;
                if row < self.filtered_entries.len() {
                    self.active_idx = row;
                }
            }
            self.ensure_active_row_visible();
            return false;
        }

        if (y as usize) < ROW_START_Y {
            return false;
        }

        let row = y as usize - ROW_START_Y;
        if (x as usize) < RAIL_WIDTH {
            if mouse_buttons == MouseButtons::LEFT {
                if let Some(section) = self.section_at_row(row) {
                    self.set_selected_section(section);
                }
            }
            return false;
        }

        let content_x = x as usize;
        let row = self.top_row + row;
        if row >= self.filtered_entries.len() {
            return false;
        }

        self.active_idx = row;
        self.ensure_active_row_visible();

        if mouse_buttons == MouseButtons::LEFT {
            if let Some(action) = self.row_action_at(row, content_x.saturating_sub(CONTENT_START_X))
            {
                self.click_state = None;
                return self.trigger_row_action(action);
            }

            let streak = match self.click_state {
                Some(click) if click.row == row => click.streak + 1,
                _ => 1,
            };
            self.click_state = Some(ClickState { row, streak });
            if streak >= 2 {
                self.trigger_focus();
                return true;
            }
        }

        false
    }

    fn handle_prompt_key(&mut self, key: KeyEvent) {
        let OverlayMode::Prompt(mut prompt) = self.mode.clone() else {
            return;
        };

        match key {
            KeyEvent {
                key: KeyCode::Enter,
                ..
            } => self.submit_prompt(prompt),
            KeyEvent {
                key: KeyCode::Escape,
                ..
            } => self.mode = OverlayMode::List,
            KeyEvent {
                key: KeyCode::Backspace,
                ..
            } => {
                prompt.input.pop();
                prompt.error = None;
                self.mode = OverlayMode::Prompt(prompt);
            }
            KeyEvent {
                key: KeyCode::Char(c),
                modifiers,
            } if !modifiers.contains(Modifiers::CTRL)
                && !modifiers.contains(Modifiers::SUPER)
                && !modifiers.contains(Modifiers::ALT) =>
            {
                prompt.input.push(c);
                prompt.error = None;
                self.mode = OverlayMode::Prompt(prompt);
            }
            _ => {}
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> bool {
        match key {
            KeyEvent {
                key: KeyCode::Enter,
                ..
            }
            | KeyEvent {
                key: KeyCode::Char('y'),
                ..
            }
            | KeyEvent {
                key: KeyCode::Char('Y'),
                ..
            } => {
                self.confirm_clear_metadata();
                false
            }
            KeyEvent {
                key: KeyCode::Escape,
                ..
            }
            | KeyEvent {
                key: KeyCode::Char('n'),
                ..
            }
            | KeyEvent {
                key: KeyCode::Char('N'),
                ..
            } => {
                self.mode = OverlayMode::List;
                false
            }
            _ => false,
        }
    }

    fn run_loop(&mut self, term: &mut TermWizTerminal) -> anyhow::Result<()> {
        while let Ok(Some(event)) = term.poll_input(None) {
            let should_close = match self.mode.clone() {
                OverlayMode::List => match event {
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Enter,
                        ..
                    }) => {
                        self.trigger_focus();
                        true
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Escape,
                        ..
                    }) => true,
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) if (c == 'c' || c == 'C') && self.can_clear_unread() => {
                        self.trigger_clear_unread();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) if (c == 'r' || c == 'R') && self.can_rerun() => {
                        self.trigger_rerun();
                        true
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) if (c == 'h' || c == 'H') && self.can_toggle_remain_on_exit() => {
                        self.trigger_toggle_remain_on_exit();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) if (c == 's' || c == 'S') && self.can_edit_metadata() => {
                        self.open_status_prompt();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) if (c == 'p' || c == 'P') && self.can_edit_metadata() => {
                        self.open_progress_prompt();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) if (c == 'm' || c == 'M') && self.can_clear_metadata() => {
                        self.open_clear_metadata_confirm();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Char(c),
                        modifiers: Modifiers::NONE,
                    }) => {
                        self.filter_term.push(c);
                        self.update_filter();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::Backspace,
                        ..
                    }) => {
                        self.filter_term.pop();
                        self.update_filter();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::UpArrow,
                        ..
                    }) => {
                        self.move_selection_up();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::DownArrow,
                        ..
                    }) => {
                        self.move_selection_down();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::LeftArrow,
                        ..
                    }) => {
                        self.move_section_left();
                        false
                    }
                    InputEvent::Key(KeyEvent {
                        key: KeyCode::RightArrow,
                        ..
                    }) => {
                        self.move_section_right();
                        false
                    }
                    InputEvent::Mouse(mouse) => self.handle_list_mouse_event(mouse),
                    _ => false,
                },
                OverlayMode::Prompt(_) => {
                    if let InputEvent::Key(key) = event {
                        self.handle_prompt_key(key);
                    }
                    false
                }
                OverlayMode::Confirm(_) => match event {
                    InputEvent::Key(key) => self.handle_confirm_key(key),
                    _ => false,
                },
            };

            if should_close {
                break;
            }
            self.render(term)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn row_text_for_test(entry: &TaskCenterEntry, active: bool) -> String {
        Self::row_layout(entry, active).text
    }

    #[cfg(test)]
    fn header_lines_for_test(&self) -> Vec<String> {
        self.header_lines().to_vec()
    }

    #[cfg(test)]
    fn set_visible_rows_for_test(&mut self, visible_rows: usize) {
        self.max_items = visible_rows.max(1);
    }

    #[cfg(test)]
    fn handle_mouse_for_test(&mut self, x: usize, y: usize, mouse_buttons: MouseButtons) -> bool {
        self.handle_list_mouse_event(MouseEvent {
            x: x as u16,
            y: y as u16,
            mouse_buttons,
            modifiers: Modifiers::NONE,
        })
    }

    #[cfg(test)]
    fn selected_label_for_test(&self) -> Option<&str> {
        self.filtered_entries
            .get(self.active_idx)
            .map(|entry| entry.label.as_str())
    }

    #[cfg(test)]
    fn top_row_for_test(&self) -> usize {
        self.top_row
    }

    #[cfg(test)]
    fn selected_section_for_test(&self) -> TaskCenterSection {
        self.selected_section
    }

    #[cfg(test)]
    fn action_x_for_test(&self, action: RowAction) -> Option<usize> {
        let entry = self.filtered_entries.get(self.active_idx)?;
        let layout = Self::row_layout(entry, true);
        layout
            .action_targets
            .into_iter()
            .find(|target| target.action == action)
            .map(|target| CONTENT_START_X + target.start)
    }

    #[cfg(test)]
    fn mode_name_for_test(&self) -> &'static str {
        match self.mode {
            OverlayMode::List => "list",
            OverlayMode::Prompt(PromptState {
                kind: PromptKind::Status,
                ..
            }) => "prompt:status",
            OverlayMode::Prompt(PromptState {
                kind: PromptKind::Progress,
                ..
            }) => "prompt:progress",
            OverlayMode::Confirm(_) => "confirm",
        }
    }
}

impl TaskCenterSection {
    fn ordered() -> [TaskCenterSection; 6] {
        [
            TaskCenterSection::All,
            TaskCenterSection::Inbox,
            TaskCenterSection::Failed,
            TaskCenterSection::Running,
            TaskCenterSection::Metadata,
            TaskCenterSection::Tasks,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            TaskCenterSection::All => "All",
            TaskCenterSection::Inbox => "Inbox",
            TaskCenterSection::Failed => "Failed",
            TaskCenterSection::Running => "Running",
            TaskCenterSection::Metadata => "Metadata",
            TaskCenterSection::Tasks => "Tasks",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            TaskCenterSection::All => "◈",
            TaskCenterSection::Inbox => ICON_UNREAD,
            TaskCenterSection::Failed => ICON_FAILED,
            TaskCenterSection::Running => ICON_RUNNING,
            TaskCenterSection::Metadata => ICON_STATUS,
            TaskCenterSection::Tasks => ICON_SOURCE,
        }
    }
}

pub fn task_center(
    entries: Vec<TaskCenterEntry>,
    mut term: TermWizTerminal,
    window: ::window::Window,
) -> anyhow::Result<()> {
    let mut overlay = TaskCenterOverlay::new(entries);
    overlay.window = Some(window);
    term.set_raw_mode()?;
    term.render(&[Change::Title("Task Center".to_string())])?;
    overlay.render(&mut term)?;
    overlay.run_loop(&mut term)
}

fn parse_source(input: &str) -> Option<TaskCenterSource> {
    match input.to_ascii_lowercase().as_str() {
        "workspace" => Some(TaskCenterSource::Workspace),
        "tab" => Some(TaskCenterSource::Tab),
        "pane" => Some(TaskCenterSource::Pane),
        "notification" => Some(TaskCenterSource::Notification),
        _ => None,
    }
}

fn parse_kind(input: &str) -> Option<TaskCenterKind> {
    match input.to_ascii_lowercase().as_str() {
        "workspace" => Some(TaskCenterKind::Workspace),
        "tab" => Some(TaskCenterKind::Tab),
        "pane" => Some(TaskCenterKind::Pane),
        "notification" => Some(TaskCenterKind::Notification),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        task_center, RowAction, TaskCenterOverlay, TaskCenterSection, CONTENT_START_X, ROW_START_Y,
    };
    use mux::task_center::{TaskCenterEntry, TaskCenterKind, TaskCenterSource};
    use termwiz::input::MouseButtons;

    fn entry(
        label: &str,
        workspace: &str,
        source: TaskCenterSource,
        kind: TaskCenterKind,
    ) -> TaskCenterEntry {
        TaskCenterEntry {
            label: label.to_string(),
            workspace: workspace.to_string(),
            source,
            kind,
            source_label: None,
            kind_label: None,
            window_id: Some(1),
            tab_id: Some(mux::tab::TabId::from(1usize)),
            pane_id: Some(mux::pane::PaneId::from(1usize)),
            unread_count: 0,
            notification_ids: vec![],
            is_failed: false,
            is_running: false,
            rerun_available: false,
            workspace_status: None,
            workspace_progress: None,
        }
    }

    #[test]
    fn task_center_search_narrows_normalized_entry_text() {
        let mut build_failed = entry(
            "Build failed",
            "unity-main",
            TaskCenterSource::Notification,
            TaskCenterKind::Notification,
        );
        build_failed.kind_label = Some("build.failed".to_string());

        let mut overlay = TaskCenterOverlay::new(vec![
            build_failed,
            entry(
                "Workspace status",
                "default",
                TaskCenterSource::Workspace,
                TaskCenterKind::Workspace,
            ),
        ]);
        overlay.filter_term = "build.failed".to_string();
        overlay.update_filter();

        assert_eq!(overlay.filtered_entries.len(), 1);
        assert_eq!(overlay.filtered_entries[0].label, "Build failed");
    }

    #[test]
    fn task_center_filters_cover_unread_failed_running_workspace_source_and_kind() {
        let mut notification = entry(
            "Unread build",
            "unity-main",
            TaskCenterSource::Notification,
            TaskCenterKind::Notification,
        );
        notification.unread_count = 2;
        notification.is_failed = true;
        notification.is_running = true;

        let mut overlay = TaskCenterOverlay::new(vec![
            notification,
            entry(
                "Pane row",
                "default",
                TaskCenterSource::Pane,
                TaskCenterKind::Pane,
            ),
        ]);
        overlay.filter_term =
            "unread failed running workspace:unity-main source:notification kind:notification"
                .to_string();
        overlay.update_filter();

        assert_eq!(overlay.filtered_entries.len(), 1);
        assert_eq!(overlay.filtered_entries[0].label, "Unread build");
    }

    #[test]
    fn task_center_search_consumes_workspace_status_and_progress_snapshot_fields() {
        let mut workspace = entry(
            "Workspace health",
            "unity-main",
            TaskCenterSource::Workspace,
            TaskCenterKind::Workspace,
        );
        workspace.workspace_status = Some("blocked".to_string());
        workspace.workspace_progress = Some(72);

        let mut overlay = TaskCenterOverlay::new(vec![
            workspace,
            entry(
                "Pane row",
                "default",
                TaskCenterSource::Pane,
                TaskCenterKind::Pane,
            ),
        ]);
        overlay.filter_term = "blocked 72".to_string();
        overlay.update_filter();

        assert_eq!(overlay.filtered_entries.len(), 1);
        assert_eq!(overlay.filtered_entries[0].label, "Workspace health");
    }

    #[test]
    fn task_center_empty_state_is_stable() {
        let overlay = TaskCenterOverlay::new(Vec::new());
        assert_eq!(overlay.empty_state_message(), "No task-center entries yet");
        let _ = task_center;
    }

    #[test]
    fn task_center_clear_unread_action_is_conditional_on_unread_state() {
        let mut unread = entry(
            "Unread build",
            "default",
            TaskCenterSource::Notification,
            TaskCenterKind::Notification,
        );
        unread.unread_count = 1;
        unread.notification_ids = vec!["notification-1".to_string()];

        let overlay = TaskCenterOverlay::new(vec![unread]);
        assert!(overlay.can_clear_unread());

        let overlay = TaskCenterOverlay::new(vec![entry(
            "Pane row",
            "default",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        )]);
        assert!(!overlay.can_clear_unread());
    }

    #[test]
    fn task_center_rerun_action_is_conditional_on_failed_rows_with_metadata() {
        let mut failed = entry(
            "Failed build",
            "default",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        );
        failed.is_failed = true;
        failed.rerun_available = true;

        let overlay = TaskCenterOverlay::new(vec![failed]);
        assert!(overlay.can_rerun());

        let overlay = TaskCenterOverlay::new(vec![entry(
            "Healthy pane",
            "default",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        )]);
        assert!(!overlay.can_rerun());
    }

    #[test]
    fn task_center_format_row_stays_a_snapshot_of_mux_entry_state() {
        let mut failed = entry(
            "Failed build",
            "unity-main",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        );
        failed.unread_count = 2;
        failed.is_failed = true;
        failed.is_running = true;
        failed.rerun_available = true;
        failed.workspace_status = Some("blocked".to_string());
        failed.workspace_progress = Some(37);

        let row = TaskCenterOverlay::format_row(&failed);
        assert!(row.starts_with("Failed build"));
        assert!(row.contains("unity-main"));
        assert!(row.contains("●2"));
        assert!(row.contains("✕"));
        assert!(row.contains("◌"));
        assert!(row.contains("↺"));
        assert!(row.contains("◇ blocked"));
        assert!(row.contains("37%"));
    }

    #[test]
    fn task_center_active_row_exposes_inline_operator_actions_without_multiline_bodies() {
        let mut task_pane = entry(
            "Worker loop",
            "unity-main",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        );
        task_pane.kind_label = Some("task-pane".to_string());
        task_pane.unread_count = 2;
        task_pane.notification_ids = vec!["notif-1".to_string()];
        task_pane.is_failed = true;
        task_pane.rerun_available = true;
        task_pane.workspace_status = Some("blocked".to_string());
        task_pane.workspace_progress = Some(42);

        let row = TaskCenterOverlay::row_text_for_test(&task_pane, true);
        assert!(!row.contains('\n'));
        assert!(row.contains("[⏎ open]"));
        assert!(row.contains("[● clear]"));
        assert!(row.contains("[↺ rerun]"));
        assert!(row.contains("[◎ hold:"));
        assert!(row.contains("[◇ status]"));
        assert!(row.contains("[◔ progress]"));
        assert!(row.contains("[⌫ clear]"));
    }

    #[test]
    fn task_center_header_copy_stays_within_three_lines_and_advertises_mouse_parity() {
        let overlay = TaskCenterOverlay::new(vec![entry(
            "Worker loop",
            "unity-main",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        )]);
        let header = overlay.header_lines_for_test();

        assert_eq!(header.len(), 3);
        assert!(header[0].contains("Task Center"));
        assert!(header[2].contains("click select"));
        assert!(header[2].contains("dbl-click focus"));
        assert!(header[2].contains("←/→ rail"));
        assert!(header[2].contains("hold"));
        assert!(header[2].contains("status"));
        assert!(header[2].contains("progress"));
    }

    #[test]
    fn task_center_left_rail_mouse_click_switches_section() {
        let mut unread = entry(
            "Unread build",
            "unity-main",
            TaskCenterSource::Notification,
            TaskCenterKind::Notification,
        );
        unread.unread_count = 2;

        let mut overlay = TaskCenterOverlay::new(vec![
            unread,
            entry(
                "Worker",
                "unity-main",
                TaskCenterSource::Pane,
                TaskCenterKind::Pane,
            ),
        ]);

        let should_close = overlay.handle_mouse_for_test(1, ROW_START_Y + 1, MouseButtons::LEFT);
        assert!(!should_close);
        assert_eq!(
            overlay.selected_section_for_test(),
            TaskCenterSection::Inbox
        );
        assert_eq!(overlay.filtered_entries.len(), 1);
        assert_eq!(overlay.filtered_entries[0].label, "Unread build");
    }

    #[test]
    fn task_center_arrow_keys_can_switch_sections() {
        let mut with_metadata = entry(
            "Workspace health",
            "unity-main",
            TaskCenterSource::Workspace,
            TaskCenterKind::Workspace,
        );
        with_metadata.workspace_status = Some("blocked".to_string());

        let mut overlay = TaskCenterOverlay::new(vec![
            with_metadata,
            entry(
                "Worker",
                "unity-main",
                TaskCenterSource::Pane,
                TaskCenterKind::Pane,
            ),
        ]);

        overlay.move_section_right();
        overlay.move_section_right();
        overlay.move_section_right();
        overlay.move_section_right();

        assert_eq!(
            overlay.selected_section_for_test(),
            TaskCenterSection::Metadata
        );
        assert_eq!(overlay.filtered_entries.len(), 1);
        assert_eq!(overlay.filtered_entries[0].label, "Workspace health");
    }

    #[test]
    fn task_center_single_click_selects_and_double_click_requests_primary_action() {
        let mut overlay = TaskCenterOverlay::new(vec![
            entry(
                "Planner",
                "default",
                TaskCenterSource::Pane,
                TaskCenterKind::Pane,
            ),
            entry(
                "Worker",
                "unity-main",
                TaskCenterSource::Pane,
                TaskCenterKind::Pane,
            ),
        ]);
        overlay.set_visible_rows_for_test(3);

        let should_close =
            overlay.handle_mouse_for_test(CONTENT_START_X, ROW_START_Y + 1, MouseButtons::LEFT);
        assert!(!should_close);
        assert_eq!(overlay.selected_label_for_test(), Some("Worker"));

        let should_close =
            overlay.handle_mouse_for_test(CONTENT_START_X, ROW_START_Y + 1, MouseButtons::LEFT);
        assert!(should_close);
    }

    #[test]
    fn task_center_wheel_scroll_keeps_selection_and_top_row_in_sync() {
        let mut overlay = TaskCenterOverlay::new(
            (0..6)
                .map(|idx| {
                    entry(
                        &format!("Pane {idx}"),
                        "unity-main",
                        TaskCenterSource::Pane,
                        TaskCenterKind::Pane,
                    )
                })
                .collect(),
        );
        overlay.set_visible_rows_for_test(2);

        overlay.handle_mouse_for_test(0, ROW_START_Y + 1, MouseButtons::VERT_WHEEL);
        assert_eq!(overlay.top_row_for_test(), 1);
        assert_eq!(overlay.selected_label_for_test(), Some("Pane 2"));
    }

    #[test]
    fn task_center_inline_action_hit_testing_enters_metadata_prompt_mode() {
        let mut entry = entry(
            "Worker loop",
            "unity-main",
            TaskCenterSource::Pane,
            TaskCenterKind::Pane,
        );
        entry.workspace_status = Some("blocked".to_string());
        entry.workspace_progress = Some(42);

        let mut overlay = TaskCenterOverlay::new(vec![entry]);
        overlay.set_visible_rows_for_test(3);

        let x = overlay.action_x_for_test(RowAction::EditStatus).unwrap();
        let should_close = overlay.handle_mouse_for_test(x, ROW_START_Y, MouseButtons::LEFT);
        assert!(!should_close);
        assert_eq!(overlay.mode_name_for_test(), "prompt:status");
    }
}
