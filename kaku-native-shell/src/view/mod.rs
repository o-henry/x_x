pub mod chrome;
pub mod context;
pub mod rail;
pub mod workspace;

pub use rail::workspace_badge_text;
pub use workspace::workspace_status_tokens;

use crate::snapshot::{RuntimeSnapshot, ShellLayoutContract};
use gtk::prelude::IsA;
use gtk::prelude::*;
use gtk::{Align, Orientation, PolicyType};

pub struct WorkspaceActionButtons {
    pub launch_terminal: gtk::Button,
    pub set_status: gtk::Button,
    pub clear_status: gtk::Button,
    pub set_progress: gtk::Button,
    pub clear_progress: gtk::Button,
    pub append_log: gtk::Button,
}

pub struct ShellView {
    pub root: gtk::Box,
    pub refresh_button: gtk::Button,
    pub terminal_button: gtk::Button,
    pub rail_buttons: Vec<(String, gtk::Button)>,
    pub workspace_actions: WorkspaceActionButtons,
    pub inbox_mark_read_button: Option<gtk::Button>,
}

pub fn shell_slot_order() -> [&'static str; 7] {
    [
        "chrome",
        "rail",
        "workspace",
        "activity",
        "metadata",
        "inbox",
        "tasks",
    ]
}

pub fn context_panel_titles() -> [&'static str; 4] {
    ["Inbox", "Tasks", "Activity", "Metadata"]
}

pub fn build_shell(snapshot: &RuntimeSnapshot, layout: &ShellLayoutContract) -> ShellView {
    let root = gtk::Box::new(Orientation::Vertical, 0);
    root.add_css_class("shell-root");

    let chrome_view = chrome::build_chrome(snapshot);
    chrome_view.root.set_height_request(layout.chrome_height);
    root.append(&chrome_view.root);

    let body = gtk::Box::new(Orientation::Horizontal, 0);
    body.set_vexpand(true);
    body.add_css_class("shell-body");

    let rail_view = rail::build_rail(snapshot);
    rail_view
        .root
        .set_size_request(156.min(layout.rail_width), -1);
    rail_view.root.set_hexpand(false);
    rail_view.root.set_halign(Align::Start);
    rail_view.root.set_vexpand(true);
    body.append(&rail_view.root);

    let center_column = gtk::Box::new(Orientation::Vertical, 0);
    center_column.set_hexpand(true);
    center_column.set_vexpand(true);

    let workspace_view = workspace::build_workspace_panel(snapshot);
    workspace_view.root.set_hexpand(true);
    workspace_view.root.set_vexpand(true);
    let activity_panel = workspace::build_activity_panel(snapshot);
    let metadata_panel = context::build_metadata_panel(snapshot);
    activity_panel.set_hexpand(true);
    activity_panel.set_vexpand(true);
    metadata_panel.set_width_request(248);
    metadata_panel.set_hexpand(false);
    metadata_panel.set_vexpand(true);

    let lower_center = gtk::Box::new(Orientation::Horizontal, 0);
    lower_center.set_height_request(252);
    lower_center.append(&activity_panel);
    lower_center.append(&metadata_panel);

    center_column.append(&workspace_view.root);
    center_column.append(&lower_center);

    let side_column = gtk::Box::new(Orientation::Vertical, 0);
    side_column.set_size_request(224, -1);
    side_column.set_hexpand(false);
    side_column.set_halign(Align::End);
    side_column.set_vexpand(true);
    let inbox_view = context::build_inbox_panel(snapshot);
    let task_panel = context::build_task_panel(snapshot);
    inbox_view.root.set_vexpand(true);
    task_panel.set_vexpand(true);
    side_column.append(&inbox_view.root);
    side_column.append(&task_panel);

    body.append(&center_column);
    body.append(&side_column);

    root.append(&body);

    ShellView {
        root,
        refresh_button: chrome_view.refresh_button,
        terminal_button: chrome_view.terminal_button,
        rail_buttons: rail_view.workspace_buttons,
        workspace_actions: workspace_view.actions,
        inbox_mark_read_button: inbox_view.mark_read_button,
    }
}

pub(crate) fn action_button(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("action-button");
    button
}

pub(crate) fn compact_action_button(label: &str) -> gtk::Button {
    let button = action_button(label);
    button.add_css_class("compact");
    button
}

pub(crate) fn chrome_action_button(label: &str) -> gtk::Button {
    let button = action_button(label);
    button.add_css_class("chrome-button");
    button
}

pub(crate) fn pane_panel(title: &str, subtitle: Option<&str>) -> gtk::Box {
    let panel = gtk::Box::new(Orientation::Vertical, 0);
    panel.add_css_class("pane-panel");

    let header = gtk::Box::new(Orientation::Vertical, 3);
    header.add_css_class("pane-header");
    header.set_margin_start(12);
    header.set_margin_end(12);
    header.set_margin_top(10);
    header.set_margin_bottom(10);

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("pane-title");
    header.append(&title_label);

    if let Some(subtitle) = subtitle {
        let subtitle_label = gtk::Label::new(Some(subtitle));
        subtitle_label.set_halign(Align::Start);
        subtitle_label.add_css_class("pane-subtitle");
        header.append(&subtitle_label);
    }

    panel.append(&header);
    panel
}

pub(crate) fn empty_state(message: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(message));
    label.set_margin_start(14);
    label.set_margin_end(14);
    label.set_margin_top(12);
    label.set_margin_bottom(14);
    label.set_wrap(true);
    label.set_halign(Align::Start);
    label.add_css_class("empty-state");
    label
}

pub(crate) fn scroller(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::External)
        .has_frame(false)
        .child(child)
        .build()
}

pub(crate) fn pill(label: &str) -> gtk::Label {
    let pill = gtk::Label::new(Some(label));
    pill.add_css_class("chrome-pill");
    pill
}

pub(crate) fn info_row(label: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::new(Orientation::Vertical, 3);
    row.add_css_class("info-row");

    let label_widget = gtk::Label::new(Some(label));
    label_widget.set_halign(Align::Start);
    label_widget.add_css_class("info-label");

    let value_widget = gtk::Label::new(Some(value));
    value_widget.set_halign(Align::Start);
    value_widget.add_css_class("info-value");

    row.append(&label_widget);
    row.append(&value_widget);
    row
}
