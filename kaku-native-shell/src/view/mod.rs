pub mod chrome;
pub mod context;
pub mod rail;
pub mod workspace;

pub use rail::workspace_badge_text;
pub use workspace::workspace_status_tokens;

use crate::snapshot::{RuntimeSnapshot, ShellLayoutContract};
use glib::value::ToValue;
use gtk::gdk;
use gtk::prelude::IsA;
use gtk::prelude::*;
use gtk::prelude::{EventControllerExt, GestureSingleExt, NativeExt, WidgetExt};
use gtk::{Align, Orientation, Paned, PolicyType};

#[derive(Clone, Copy, Debug, Default)]
pub struct PaneArrangement {
    pub compact: bool,
    pub metadata_first: bool,
    pub tasks_first: bool,
    pub show_terminal_sessions: bool,
}

pub struct PaneFrame {
    pub root: gtk::Box,
    pub header: gtk::Box,
    pub body: gtk::Box,
}

pub struct ShellView {
    pub root: gtk::Box,
    pub chrome_drag_handle: gtk::Box,
    pub refresh_button: gtk::Button,
    pub terminal_button: gtk::Button,
    pub rail_toggle_button: gtk::Button,
    pub rail_buttons: Vec<(String, gtk::Button)>,
    pub inbox_mark_read_button: Option<gtk::Button>,
    pub activity_header: gtk::Box,
    pub metadata_header: gtk::Box,
    pub inbox_header: gtk::Box,
    pub tasks_header: gtk::Box,
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

pub fn build_shell(
    snapshot: &RuntimeSnapshot,
    layout: &ShellLayoutContract,
    rail_collapsed: bool,
    arrangement: PaneArrangement,
) -> ShellView {
    let root = gtk::Box::new(Orientation::Vertical, 0);
    root.add_css_class("shell-root");

    let chrome_view = chrome::build_chrome(snapshot);
    chrome_view.root.set_height_request(layout.chrome_height);
    root.append(&chrome_view.root);

    let body = gtk::Box::new(Orientation::Horizontal, 0);
    body.set_vexpand(true);
    body.add_css_class("shell-body");

    let rail_view = rail::build_rail(snapshot, rail_collapsed);
    rail_view.root.set_size_request(
        if rail_collapsed {
            layout.collapsed_rail_width
        } else {
            224.max(layout.rail_width)
        },
        -1,
    );
    rail_view.root.set_hexpand(false);
    rail_view.root.set_halign(Align::Start);
    rail_view.root.set_vexpand(true);
    body.append(&rail_view.root);

    let center_column = gtk::Paned::new(Orientation::Vertical);
    center_column.add_css_class("shell-split");
    center_column.set_wide_handle(true);
    center_column.set_hexpand(true);
    center_column.set_vexpand(true);
    center_column.set_resize_start_child(true);
    center_column.set_resize_end_child(true);
    center_column.set_shrink_start_child(false);
    center_column.set_shrink_end_child(false);
    center_column.set_position(layout.workspace_split);

    let workspace_view =
        workspace::build_workspace_panel(snapshot, arrangement.show_terminal_sessions);
    workspace_view.root.set_hexpand(true);
    workspace_view.root.set_vexpand(true);
    workspace_view.root.set_size_request(540, 320);
    let activity_view = context::build_activity_panel(snapshot);
    let metadata_view = context::build_metadata_panel(snapshot);
    activity_view.root.set_hexpand(true);
    activity_view.root.set_vexpand(true);
    activity_view.root.set_size_request(420, 220);
    metadata_view.root.set_hexpand(false);
    metadata_view.root.set_vexpand(true);
    metadata_view.root.set_size_request(300, 220);

    let metadata_host = gtk::Box::new(Orientation::Horizontal, 0);
    metadata_host.set_hexpand(false);
    metadata_host.set_vexpand(true);
    metadata_host.set_size_request(layout.side_split, -1);
    metadata_host.append(&metadata_view.root);

    let lower_center = gtk::Paned::new(Orientation::Horizontal);
    lower_center.add_css_class("shell-split");
    lower_center.set_wide_handle(true);
    lower_center.set_resize_start_child(true);
    lower_center.set_resize_end_child(false);
    lower_center.set_shrink_start_child(false);
    lower_center.set_shrink_end_child(false);
    lower_center.set_position(layout.lower_split);
    if arrangement.metadata_first {
        lower_center.set_start_child(Some(&metadata_host));
        lower_center.set_end_child(Some(&activity_view.root));
    } else {
        lower_center.set_start_child(Some(&activity_view.root));
        lower_center.set_end_child(Some(&metadata_host));
    }

    center_column.set_start_child(Some(&workspace_view.root));
    center_column.set_end_child(Some(&lower_center));

    let side_column = gtk::Paned::new(Orientation::Vertical);
    side_column.add_css_class("shell-split");
    side_column.set_wide_handle(true);
    side_column.set_resize_start_child(true);
    side_column.set_resize_end_child(true);
    side_column.set_shrink_start_child(false);
    side_column.set_shrink_end_child(false);
    side_column.set_hexpand(false);
    side_column.set_halign(Align::End);
    side_column.set_vexpand(true);
    side_column.set_position(layout.side_split);
    let inbox_view = context::build_inbox_panel(snapshot);
    let task_view = context::build_task_panel(snapshot);
    inbox_view.root.set_vexpand(true);
    inbox_view.root.set_hexpand(true);
    inbox_view.root.set_size_request(layout.side_split, 240);
    task_view.root.set_vexpand(true);
    task_view.root.set_hexpand(true);
    task_view.root.set_size_request(layout.side_split, 220);
    if arrangement.tasks_first {
        side_column.set_start_child(Some(&task_view.root));
        side_column.set_end_child(Some(&inbox_view.root));
    } else {
        side_column.set_start_child(Some(&inbox_view.root));
        side_column.set_end_child(Some(&task_view.root));
    }

    let side_host = gtk::Box::new(Orientation::Horizontal, 0);
    side_host.set_hexpand(false);
    side_host.set_vexpand(true);
    side_host.set_size_request(layout.side_split, -1);
    side_host.append(&side_column);

    let body_split = Paned::new(Orientation::Horizontal);
    body_split.add_css_class("shell-split");
    body_split.set_wide_handle(true);
    body_split.set_hexpand(true);
    body_split.set_vexpand(true);
    body_split.set_resize_start_child(true);
    body_split.set_resize_end_child(false);
    body_split.set_shrink_start_child(false);
    body_split.set_shrink_end_child(false);
    body_split.set_position(layout.body_split);
    if arrangement.compact {
        let compact_stack = gtk::Box::new(Orientation::Vertical, 12);
        compact_stack.add_css_class("compact-stack");
        compact_stack.set_hexpand(true);
        compact_stack.set_vexpand(true);
        compact_stack.set_margin_start(0);
        compact_stack.set_margin_end(0);
        compact_stack.set_margin_top(0);
        compact_stack.set_margin_bottom(0);
        compact_stack.append(&workspace_view.root);
        compact_stack.append(&activity_view.root);
        compact_stack.append(&metadata_view.root);
        compact_stack.append(&inbox_view.root);
        compact_stack.append(&task_view.root);
        let compact_scroll = scroller(&compact_stack);
        compact_scroll.set_hexpand(true);
        compact_scroll.set_vexpand(true);
        body.append(&compact_scroll);
    } else {
        body_split.set_start_child(Some(&center_column));
        body_split.set_end_child(Some(&side_host));
        body.append(&body_split);
    }

    root.append(&body);

    ShellView {
        root,
        chrome_drag_handle: chrome_view.root.clone(),
        refresh_button: chrome_view.refresh_button,
        terminal_button: chrome_view.terminal_button,
        rail_toggle_button: rail_view.toggle_button,
        rail_buttons: rail_view.workspace_buttons,
        inbox_mark_read_button: inbox_view.mark_read_button,
        activity_header: activity_view.header,
        metadata_header: metadata_view.header,
        inbox_header: inbox_view.header,
        tasks_header: task_view.header,
    }
}

pub(crate) fn action_button(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("action-button");
    button
}

pub(crate) fn chrome_action_button(label: &str) -> gtk::Button {
    let button = action_button(label);
    button.add_css_class("chrome-button");
    button
}

pub(crate) fn pane_panel(
    title: &str,
    subtitle: Option<&str>,
    header_action: Option<&impl IsA<gtk::Widget>>,
) -> PaneFrame {
    let panel = gtk::Box::new(Orientation::Vertical, 0);
    panel.add_css_class("pane-panel");

    let header = gtk::Box::new(Orientation::Horizontal, 10);
    header.add_css_class("pane-titlebar");
    header.set_hexpand(true);

    let title_stack = gtk::Box::new(Orientation::Vertical, 2);
    title_stack.add_css_class("pane-header");
    title_stack.set_hexpand(true);
    title_stack.set_margin_start(12);
    title_stack.set_margin_end(12);
    title_stack.set_margin_top(8);
    title_stack.set_margin_bottom(8);

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("pane-title");
    title_stack.append(&title_label);

    if let Some(subtitle) = subtitle {
        let subtitle_label = gtk::Label::new(Some(subtitle));
        subtitle_label.set_halign(Align::Start);
        subtitle_label.add_css_class("pane-subtitle");
        title_stack.append(&subtitle_label);
    }

    header.append(&title_stack);

    if let Some(action) = header_action {
        let action_host = gtk::Box::new(Orientation::Horizontal, 0);
        action_host.add_css_class("pane-titlebar-action");
        action_host.set_margin_end(10);
        action_host.set_margin_top(8);
        action_host.set_margin_bottom(8);
        action_host.append(action);
        header.append(&action_host);
    }

    let drag = gtk::GestureClick::new();
    let drag_handle = header.clone();
    drag.set_button(1);
    drag.connect_pressed(move |gesture, _, x, y| {
        let Some(device) = gesture.current_event_device() else {
            return;
        };
        let Some(native) = drag_handle.native() else {
            return;
        };
        let Some(surface) = native.surface() else {
            return;
        };
        let Ok(toplevel) = surface.dynamic_cast::<gdk::Toplevel>() else {
            return;
        };
        toplevel.begin_move(
            &device,
            gesture.current_button() as i32,
            x,
            y,
            gesture.current_event_time(),
        );
    });
    title_stack.add_controller(drag);

    panel.append(&header);

    let body = gtk::Box::new(Orientation::Vertical, 0);
    body.add_css_class("pane-surface");
    body.set_hexpand(true);
    body.set_vexpand(true);
    panel.append(&body);

    PaneFrame {
        root: panel,
        header,
        body,
    }
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

pub(crate) fn bind_header_swap(
    source: &gtk::Box,
    target: &gtk::Box,
    tag: &'static str,
    on_drop: impl Fn() + 'static,
) {
    let drag_source = gtk::DragSource::builder()
        .actions(gdk::DragAction::MOVE)
        .build();
    drag_source
        .connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&tag.to_value())));
    source.add_controller(drag_source);

    let drop_target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
    drop_target.connect_drop(move |_, value, _, _| {
        let Ok(payload) = value.get::<String>() else {
            return false;
        };
        if payload == tag {
            on_drop();
            return true;
        }
        false
    });
    target.add_controller(drop_target);
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
