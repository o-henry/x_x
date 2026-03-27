use crate::overlay::selector::{matcher_pattern, matcher_score};
use crate::termwindow::TermWindowNotif;
use mux::task_center::{TaskCenterEntry, TaskCenterKind, TaskCenterSource};
use mux::termwiztermtab::TermWizTerminal;
use mux::Mux;
use termwiz::cell::CellAttributes;
use termwiz::input::{InputEvent, KeyCode, KeyEvent, Modifiers};
use termwiz::surface::{Change, Position};
use termwiz::terminal::Terminal;
use termwiz_funcs::truncate_right;
use window::WindowOps;

const ROW_OVERHEAD: usize = 4;

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

    fn update_filter(&mut self) {
        let parsed = Self::parse_query(&self.filter_term);
        self.filters = parsed.filters;
        self.filtered_entries = self
            .entries
            .iter()
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
    }

    fn active_filter_summary(&self) -> String {
        let mut parts = Vec::new();
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

    fn format_row(entry: &TaskCenterEntry) -> String {
        let mut suffixes = Vec::new();
        suffixes.push(entry.workspace.clone());
        if entry.unread_count > 0 {
            suffixes.push(format!("u{}", entry.unread_count));
        }
        if entry.is_failed {
            suffixes.push("failed".to_string());
        }
        if entry.is_running {
            suffixes.push("running".to_string());
        }
        if entry.rerun_available {
            suffixes.push("rerun".to_string());
        }
        if let Some(status) = &entry.workspace_status {
            suffixes.push(format!("[{status}]"));
        }
        if let Some(progress) = entry.workspace_progress {
            suffixes.push(format!("{progress}%"));
        }
        format!("{} · {}", entry.label, suffixes.join(" "))
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
        self.entries = Mux::get().task_center_snapshot();
        self.update_filter();
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

    fn render(&mut self, term: &mut TermWizTerminal) -> termwiz::Result<()> {
        let size = term.get_screen_size()?;
        let max_width = size.cols.saturating_sub(2);
        self.max_items = size.rows.saturating_sub(ROW_OVERHEAD);

        let mut changes = vec![
            Change::ClearScreen(termwiz::color::ColorAttribute::Default),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(0),
            },
            Change::Text(format!(
                "{}\r\n",
                truncate_right("Task Center", max_width)
            )),
            Change::Text(format!(
                "{}\r\n",
                truncate_right(
                    &format!(
                        "filters: {} | query: {}",
                        self.active_filter_summary(),
                        if self.filter_term.is_empty() {
                            "(none)"
                        } else {
                            &self.filter_term
                        }
                    ),
                    max_width
                )
            )),
            Change::Text(
                truncate_right(
                    "Enter=focus C=clear unread R=rerun Esc=close | tokens: unread failed running workspace:<name> source:<kind> kind:<kind>",
                    max_width,
                )
                .to_string()
                .into(),
            ),
            Change::Text("\r\n".to_string()),
            Change::AllAttributes(CellAttributes::default()),
        ];

        if self.filtered_entries.is_empty() {
            changes.push(Change::Text(format!(
                "{}\r\n",
                truncate_right(self.empty_state_message(), max_width)
            )));
            return term.render(&changes);
        }

        for (row_num, (entry_idx, entry)) in self
            .filtered_entries
            .iter()
            .enumerate()
            .skip(self.top_row)
            .enumerate()
        {
            if row_num > self.max_items {
                break;
            }

            if entry_idx == self.active_idx {
                changes.push(termwiz::cell::AttributeChange::Reverse(true).into());
            }
            changes.push(Change::Text(format!(
                "{}\r\n",
                truncate_right(&Self::format_row(entry), max_width)
            )));
            if entry_idx == self.active_idx {
                changes.push(termwiz::cell::AttributeChange::Reverse(false).into());
            }
        }

        term.render(&changes)
    }

    fn run_loop(&mut self, term: &mut TermWizTerminal) -> anyhow::Result<()> {
        while let Ok(Some(event)) = term.poll_input(None) {
            match event {
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Enter,
                    ..
                }) => {
                    self.trigger_focus();
                    break;
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Escape,
                    ..
                }) => break,
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    modifiers: Modifiers::NONE,
                }) if (c == 'c' || c == 'C') && self.can_clear_unread() => {
                    self.trigger_clear_unread();
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    modifiers: Modifiers::NONE,
                }) if (c == 'r' || c == 'R') && self.can_rerun() => {
                    self.trigger_rerun();
                    break;
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    modifiers: Modifiers::NONE,
                }) => {
                    self.filter_term.push(c);
                    self.update_filter();
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Backspace,
                    ..
                }) => {
                    self.filter_term.pop();
                    self.update_filter();
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::UpArrow,
                    ..
                }) => {
                    self.active_idx = self.active_idx.saturating_sub(1);
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::DownArrow,
                    ..
                }) => {
                    if !self.filtered_entries.is_empty() {
                        self.active_idx =
                            (self.active_idx + 1).min(self.filtered_entries.len() - 1);
                    }
                }
                _ => {}
            }
            self.render(term)?;
        }
        Ok(())
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
    use super::{task_center, TaskCenterOverlay};
    use mux::task_center::{TaskCenterEntry, TaskCenterKind, TaskCenterSource};

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
        assert!(row.contains("u2"));
        assert!(row.contains("failed"));
        assert!(row.contains("running"));
        assert!(row.contains("rerun"));
        assert!(row.contains("[blocked]"));
        assert!(row.contains("37%"));
    }
}
