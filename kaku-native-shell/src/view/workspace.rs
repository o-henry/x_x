use crate::snapshot::{RuntimeSnapshot, WorkspaceSummary};
use crate::view::{pane_panel, pill};
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct WorkspacePanelView {
    pub root: gtk::Box,
}

pub fn build_workspace_panel(snapshot: &RuntimeSnapshot) -> WorkspacePanelView {
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
        ("TERMINAL", "CMD+T"),
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
