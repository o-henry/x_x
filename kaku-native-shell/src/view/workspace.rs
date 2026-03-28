use crate::snapshot::{RuntimeSnapshot, WorkspaceSummary};
use crate::view::{pane_panel, pill};
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct WorkspacePanelView {
    pub root: gtk::Box,
}

pub fn build_workspace_panel(
    snapshot: &RuntimeSnapshot,
    show_terminal_sessions: bool,
) -> WorkspacePanelView {
    let workspace = current_workspace_summary(snapshot);
    let body = gtk::Box::new(Orientation::Vertical, 14);
    body.add_css_class("pane-body");
    body.add_css_class("pane-content");
    body.set_margin_start(14);
    body.set_margin_end(14);
    body.set_margin_top(14);
    body.set_margin_bottom(14);

    let title = gtk::Label::new(Some(&workspace.name));
    title.set_halign(Align::Start);
    title.add_css_class("workspace-title");

    let summary = gtk::Label::new(Some(&format!(
        "{} unread  {} running  {} failed  {} logs",
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

    let frame = pane_panel("Workspace", None, Option::<&gtk::Widget>::None);

    let shortcut_legend = gtk::Box::new(Orientation::Vertical, 4);
    shortcut_legend.add_css_class("shortcut-legend");
    for (label, accel) in [
        ("TERMINAL SESSIONS", "CMD+T"),
        ("STATUS", "CMD+SHIFT+S"),
        ("CLEAR STATUS", "CMD+SHIFT+X"),
        ("PROGRESS", "CMD+SHIFT+P"),
        ("RESET PROGRESS", "CMD+SHIFT+R"),
        ("APPEND LOG", "CMD+SHIFT+L"),
    ] {
        shortcut_legend.append(&shortcut_line(label, accel));
    }

    body.append(&title);
    body.append(&summary);
    body.append(&state_row);
    body.append(&shortcut_legend);
    body.append(&workspace_snapshot_block(snapshot, show_terminal_sessions));
    frame.body.append(&body);

    WorkspacePanelView { root: frame.root }
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

fn shortcut_line(action: &str, combo: &str) -> gtk::Box {
    let row = gtk::Box::new(Orientation::Horizontal, 8);
    row.add_css_class("shortcut-line");

    let action_label = gtk::Label::new(Some(action));
    action_label.set_halign(Align::Start);
    action_label.set_hexpand(true);
    action_label.add_css_class("shortcut-action");

    let combo_label = gtk::Label::new(Some(combo));
    combo_label.set_halign(Align::End);
    combo_label.add_css_class("shortcut-combo");

    row.append(&action_label);
    row.append(&combo_label);
    row
}

fn workspace_snapshot_block(snapshot: &RuntimeSnapshot, show_terminal_sessions: bool) -> gtk::Box {
    let block = gtk::Box::new(Orientation::Vertical, 10);
    block.add_css_class("workspace-block");

    block.append(&section_label("WORKTREE"));
    block.append(&detail_line(
        &current_workspace_cwd(snapshot).unwrap_or_else(|| "No active cwd".to_string()),
    ));

    block.append(&section_label("INBOX PREVIEW"));
    let unread = snapshot
        .notifications
        .iter()
        .filter(|row| row.workspace == snapshot.active_workspace && row.unread)
        .take(3)
        .cloned()
        .collect::<Vec<_>>();
    if unread.is_empty() {
        block.append(&detail_line("No unread notifications"));
    } else {
        for item in unread {
            let text = match item.body.as_deref() {
                Some(body) if !body.is_empty() => format!("{}  {}", item.title, body),
                _ => item.title,
            };
            block.append(&detail_line(&text));
        }
    }

    if show_terminal_sessions {
        block.append(&section_label("TERMINAL SESSIONS"));
        let panes = snapshot
            .task_panes
            .iter()
            .filter(|row| row.workspace.as_deref() == Some(snapshot.active_workspace.as_str()))
            .take(4)
            .collect::<Vec<_>>();
        if panes.is_empty() {
            block.append(&detail_line("No live task panes in this workspace"));
        } else {
            for pane in panes {
                let state = if pane.is_failed {
                    "failed"
                } else if pane.is_dead {
                    "dead"
                } else {
                    "running"
                };
                let cwd = pane
                    .current_working_dir
                    .as_deref()
                    .map(shorten_cwd)
                    .unwrap_or_else(|| "no cwd".to_string());
                block.append(&detail_line(&format!(
                    "pane {}  {}  {}",
                    pane.pane_id, state, cwd
                )));
            }
        }
    }

    block
}

fn section_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_halign(Align::Start);
    label.add_css_class("workspace-section-label");
    label
}

fn detail_line(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.set_halign(Align::Start);
    label.set_wrap(true);
    label.add_css_class("workspace-detail-line");
    label
}

fn current_workspace_cwd(snapshot: &RuntimeSnapshot) -> Option<String> {
    snapshot
        .task_panes
        .iter()
        .find(|row| row.workspace.as_deref() == Some(snapshot.active_workspace.as_str()))
        .and_then(|row| row.current_working_dir.as_ref())
        .map(|cwd| shorten_cwd(cwd))
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|path| path.to_str().map(shorten_cwd))
        })
}

fn shorten_cwd(cwd: &str) -> String {
    let without_scheme = cwd.strip_prefix("file://").unwrap_or(cwd);
    let mut parts = without_scheme
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() <= 2 {
        return without_scheme.to_string();
    }
    let tail = parts.split_off(parts.len() - 2);
    format!("…/{}/{}", tail[0], tail[1])
}
