use crate::snapshot::{RuntimeSnapshot, WorkspaceSummary};
use crate::view::{action_button, empty_state, pane_panel, pill, scroller, WorkspaceActionButtons};
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct WorkspacePanelView {
    pub root: gtk::Box,
    pub actions: WorkspaceActionButtons,
}

pub fn build_workspace_panel(snapshot: &RuntimeSnapshot) -> WorkspacePanelView {
    let workspace = current_workspace_summary(snapshot);
    let panel = pane_panel("Workspace", None);
    let body = gtk::Box::new(Orientation::Vertical, 12);
    body.add_css_class("pane-body");
    body.set_margin_start(14);
    body.set_margin_end(14);
    body.set_margin_top(14);
    body.set_margin_bottom(14);

    let title = gtk::Label::new(Some(&workspace.name));
    title.set_halign(Align::Start);
    title.add_css_class("workspace-title");

    let summary = gtk::Label::new(Some(&format!(
        "{}U  {}R  {}F  {}L",
        workspace.unread_count,
        workspace.running_count,
        workspace.failed_count,
        workspace.log_count
    )));
    summary.set_halign(Align::Start);
    summary.add_css_class("workspace-summary");

    let state_row = gtk::Box::new(Orientation::Horizontal, 8);
    state_row.set_halign(Align::Start);
    if let Some(status) = &workspace.status {
        state_row.append(&pill(status));
    } else {
        state_row.append(&pill("unset"));
    }
    if let Some(progress) = workspace.progress {
        state_row.append(&pill(&format!("{progress}%")));
    } else {
        state_row.append(&pill("unset"));
    }

    let actions_row = gtk::Box::new(Orientation::Horizontal, 8);
    actions_row.set_halign(Align::Start);
    let launch_terminal = action_button("open");
    let set_status = action_button("status");
    let clear_status = action_button("clear");
    let set_progress = action_button("progress");
    let clear_progress = action_button("clear");
    let append_log = action_button("log");
    for button in [
        &launch_terminal,
        &set_status,
        &clear_status,
        &set_progress,
        &clear_progress,
        &append_log,
    ] {
        actions_row.append(button);
    }

    body.append(&title);
    body.append(&summary);
    body.append(&state_row);
    body.append(&actions_row);
    panel.append(&body);

    WorkspacePanelView {
        root: panel,
        actions: WorkspaceActionButtons {
            launch_terminal,
            set_status,
            clear_status,
            set_progress,
            clear_progress,
            append_log,
        },
    }
}

pub fn build_activity_panel(snapshot: &RuntimeSnapshot) -> gtk::Box {
    let panel = pane_panel("Activity", None);
    if snapshot.logs.is_empty() {
        panel.append(&empty_state("No workspace log entries yet."));
        return panel;
    }

    let list = gtk::Box::new(Orientation::Vertical, 8);
    list.set_margin_start(14);
    list.set_margin_end(14);
    list.set_margin_top(12);
    list.set_margin_bottom(14);
    for row in snapshot.logs.iter().rev().take(12) {
        list.append(&log_row(row.seq, &row.message));
    }
    panel.append(&scroller(&list));
    panel
}

pub fn workspace_status_tokens(
    status: Option<&str>,
    progress: Option<u8>,
    log_count: usize,
) -> [String; 3] {
    [
        status.unwrap_or("unset").to_string(),
        format!("{}%", progress.unwrap_or(0)),
        format!("{}L", log_count),
    ]
}

fn current_workspace_summary(snapshot: &RuntimeSnapshot) -> &WorkspaceSummary {
    snapshot
        .workspaces
        .iter()
        .find(|summary| summary.name == snapshot.active_workspace)
        .or_else(|| snapshot.workspaces.first())
        .expect("at least one workspace summary")
}

fn log_row(seq: u64, message: &str) -> gtk::Box {
    let outer = gtk::Box::new(Orientation::Vertical, 4);
    outer.add_css_class("list-row");

    let title = gtk::Label::new(Some(&format!("log #{seq}")));
    title.set_halign(Align::Start);
    title.add_css_class("row-title");

    let detail = gtk::Label::new(Some(message));
    detail.set_halign(Align::Start);
    detail.set_wrap(true);
    detail.add_css_class("row-detail");

    outer.append(&title);
    outer.append(&detail);
    outer
}
