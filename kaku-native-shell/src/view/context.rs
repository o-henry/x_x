use crate::snapshot::RuntimeSnapshot;
use crate::view::{action_button, empty_state, info_row, pane_panel, scroller};
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct InboxPanelView {
    pub root: gtk::Box,
    pub mark_read_button: Option<gtk::Button>,
}

pub fn build_inbox_panel(snapshot: &RuntimeSnapshot) -> InboxPanelView {
    let panel = pane_panel("Inbox", None);
    let selected = snapshot.active_workspace.as_str();
    let unread = snapshot
        .notifications
        .iter()
        .filter(|row| row.workspace == selected && row.unread)
        .cloned()
        .collect::<Vec<_>>();

    let header_actions = gtk::Box::new(Orientation::Horizontal, 8);
    header_actions.set_margin_start(12);
    header_actions.set_margin_end(12);
    header_actions.set_margin_top(8);
    let mark_read = action_button("mark");
    header_actions.append(&mark_read);
    panel.append(&header_actions);
    let mark_read_button = Some(mark_read);

    if unread.is_empty() {
        panel.append(&empty_state("No unread notifications in this workspace."));
        return InboxPanelView {
            root: panel,
            mark_read_button,
        };
    }

    let list = gtk::Box::new(Orientation::Vertical, 6);
    list.set_margin_start(12);
    list.set_margin_end(12);
    list.set_margin_top(6);
    list.set_margin_bottom(12);
    for row in unread.iter().take(8) {
        list.append(&notification_row(
            &row.title,
            &format!("{}  {}", row.kind, row.body.as_deref().unwrap_or("no body")),
        ));
    }
    panel.append(&scroller(&list));
    InboxPanelView {
        root: panel,
        mark_read_button,
    }
}

pub fn build_task_panel(snapshot: &RuntimeSnapshot) -> gtk::Box {
    let panel = pane_panel("Tasks", None);
    let selected = snapshot.active_workspace.as_str();
    let tasks = snapshot
        .task_panes
        .iter()
        .filter(|row| row.workspace.as_deref() == Some(selected))
        .cloned()
        .collect::<Vec<_>>();

    if tasks.is_empty() {
        panel.append(&empty_state("No task panes recorded for this workspace."));
        return panel;
    }

    let list = gtk::Box::new(Orientation::Vertical, 6);
    list.set_margin_start(12);
    list.set_margin_end(12);
    list.set_margin_top(10);
    list.set_margin_bottom(12);
    for row in tasks.iter().take(10) {
        let status = if row.is_failed {
            "failed"
        } else if row.is_dead {
            "dead"
        } else {
            "running"
        };
        let detail = format!(
            "{status}  remain:{}  silent:{}",
            row.remain_on_exit, row.silenced
        );
        list.append(&notification_row(&format!("pane {}", row.pane_id), &detail));
        if let Some(cwd) = &row.current_working_dir {
            list.append(&notification_row("cwd", cwd));
        }
    }
    panel.append(&scroller(&list));
    panel
}

pub fn build_metadata_panel(snapshot: &RuntimeSnapshot) -> gtk::Box {
    let panel = pane_panel("Metadata", None);
    let selected = snapshot.active_workspace.as_str();
    let status = snapshot
        .statuses
        .iter()
        .find(|row| row.workspace == selected)
        .map(|row| row.status.clone());
    let progress = snapshot
        .progresses
        .iter()
        .find(|row| row.workspace == selected)
        .map(|row| row.value);

    let body = gtk::Box::new(Orientation::Vertical, 10);
    body.set_margin_start(12);
    body.set_margin_end(12);
    body.set_margin_top(10);
    body.set_margin_bottom(12);

    body.append(&info_row("Workspace", selected));
    body.append(&info_row("Status", status.as_deref().unwrap_or("unset")));
    body.append(&info_row(
        "Progress",
        &progress
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "unset".to_string()),
    ));
    body.append(&info_row(
        "Unread",
        &snapshot
            .notifications
            .iter()
            .filter(|row| row.workspace == selected && row.unread)
            .count()
            .to_string(),
    ));
    body.append(&info_row(
        "Tasks",
        &snapshot
            .task_panes
            .iter()
            .filter(|row| row.workspace.as_deref() == Some(selected))
            .count()
            .to_string(),
    ));
    panel.append(&body);
    panel
}

fn notification_row(title_text: &str, detail_text: &str) -> gtk::Box {
    let outer = gtk::Box::new(Orientation::Vertical, 4);
    outer.add_css_class("list-row");

    let title = gtk::Label::new(Some(title_text));
    title.set_halign(Align::Start);
    title.add_css_class("row-title");

    let detail = gtk::Label::new(Some(detail_text));
    detail.set_halign(Align::Start);
    detail.set_wrap(true);
    detail.add_css_class("row-detail");

    outer.append(&title);
    outer.append(&detail);
    outer
}
