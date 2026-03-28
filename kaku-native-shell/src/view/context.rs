use crate::snapshot::RuntimeSnapshot;
use crate::view::{action_button, empty_state, info_row, pane_panel, scroller};
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct InboxPanelView {
    pub root: gtk::Box,
    pub mark_read_button: Option<gtk::Button>,
}

pub fn build_inbox_panel(snapshot: &RuntimeSnapshot) -> InboxPanelView {
    let mark_read = action_button("Mark Read");
    mark_read.add_css_class("subtle");
    mark_read.add_css_class("text-only");
    let frame = pane_panel("Inbox", None, Some(&mark_read));
    let selected = snapshot.active_workspace.as_str();
    let unread = snapshot
        .notifications
        .iter()
        .filter(|row| row.workspace == selected && row.unread)
        .cloned()
        .collect::<Vec<_>>();

    let mark_read_button = Some(mark_read);

    if unread.is_empty() {
        frame
            .body
            .append(&empty_state("No unread notifications in this workspace."));
        return InboxPanelView {
            root: frame.root,
            mark_read_button,
        };
    }

    let list = gtk::Box::new(Orientation::Vertical, 6);
    list.set_margin_start(12);
    list.set_margin_end(12);
    list.set_margin_top(14);
    list.set_margin_bottom(12);
    for row in unread.iter().take(8) {
        list.append(&notification_row(
            &row.title,
            &format!("{}  {}", row.kind, row.body.as_deref().unwrap_or("no body")),
        ));
    }
    frame.body.append(&scroller(&list));
    InboxPanelView {
        root: frame.root,
        mark_read_button,
    }
}

pub fn build_task_panel(snapshot: &RuntimeSnapshot) -> gtk::Box {
    let frame = pane_panel("Tasks", None, Option::<&gtk::Widget>::None);
    let selected = snapshot.active_workspace.as_str();
    let tasks = snapshot
        .task_panes
        .iter()
        .filter(|row| row.workspace.as_deref() == Some(selected))
        .cloned()
        .collect::<Vec<_>>();

    if tasks.is_empty() {
        frame
            .body
            .append(&empty_state("No task panes recorded for this workspace."));
        return frame.root;
    }

    let list = gtk::Box::new(Orientation::Vertical, 6);
    list.set_margin_start(12);
    list.set_margin_end(12);
    list.set_margin_top(14);
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
    frame.body.append(&scroller(&list));
    frame.root
}

pub fn build_metadata_panel(snapshot: &RuntimeSnapshot) -> gtk::Box {
    let frame = pane_panel("Metadata", None, Option::<&gtk::Widget>::None);
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
    body.add_css_class("pane-content");
    body.set_margin_start(12);
    body.set_margin_end(12);
    body.set_margin_top(12);
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
    frame.body.append(&body);
    frame.root
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
