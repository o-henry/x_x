use crate::snapshot::RuntimeSnapshot;
use crate::terminal::{build_terminal_widget, TerminalHandle};
use gtk::gdk;
use gtk::prelude::*;
use gtk::{Align, Orientation};
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TwoPaneSplit {
    #[default]
    SideBySide,
    Stacked,
}

pub struct WorkspacePanelView {
    pub root: gtk::Widget,
    pub terminal_panes: Vec<TerminalPaneHandle>,
}

pub struct TerminalPaneHandle {
    pub index: usize,
    pub drag_handle: gtk::Box,
    pub drop_target: gtk::Box,
    pub focus_target: gtk::DrawingArea,
    pub close_button: gtk::Button,
}

pub fn build_workspace_panel(
    snapshot: &RuntimeSnapshot,
    show_terminal_sessions: bool,
    terminals: &[Rc<TerminalHandle>],
    split: TwoPaneSplit,
    zoomed_terminal_index: Option<usize>,
) -> WorkspacePanelView {
    if show_terminal_sessions {
        let terminal_area = build_terminal_area(terminals, split, zoomed_terminal_index);
        let host = gtk::Box::new(Orientation::Vertical, 0);
        host.add_css_class("pane-surface");
        host.add_css_class("live-terminal-host");
        host.set_hexpand(true);
        host.set_vexpand(true);
        terminal_area.root.set_hexpand(true);
        terminal_area.root.set_vexpand(true);
        host.append(&terminal_area.root);
        WorkspacePanelView {
            root: host.upcast(),
            terminal_panes: terminal_area.panes,
        }
    } else {
        let root = gtk::Box::new(Orientation::Vertical, 0);
        root.add_css_class("pane-panel");
        root.add_css_class("pane-surface");
        let content = gtk::Box::new(Orientation::Vertical, 12);
        content.add_css_class("pane-body");
        content.add_css_class("pane-content");
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(0);
        content.set_margin_bottom(0);
        content.append(&workspace_snapshot_block(snapshot));
        root.append(&content);
        WorkspacePanelView {
            root: root.upcast(),
            terminal_panes: Vec::new(),
        }
    }
}

pub fn build_shortcut_bar() -> gtk::ScrolledWindow {
    shortcut_bar()
}

pub fn shortcut_entries() -> [(&'static str, &'static str); 19] {
    [
        ("NEW SHELL", "CMD+T"),
        ("SPLIT RIGHT", "CMD+D"),
        ("SPLIT DOWN", "CMD+SHIFT+D"),
        ("TOGGLE SPLIT", "CMD+SHIFT+S"),
        ("ZOOM", "CMD+SHIFT+ENTER"),
        ("CLOSE PANE", "CMD+W"),
        ("LAZYGIT", "CMD+SHIFT+G"),
        ("YAZI", "CMD+SHIFT+Y"),
        ("DOCTOR", "CMD+SHIFT+O"),
        ("CONFIG", "CMD+,"),
        ("WORKSPACES", "CMD+B"),
        ("ACTIVITY", "CMD+SHIFT+A"),
        ("TASKS", "CMD+SHIFT+T"),
        ("METADATA", "CMD+SHIFT+M"),
        ("STATUS", "CMD+SHIFT+S"),
        ("CLEAR", "CMD+SHIFT+X"),
        ("PROGRESS", "CMD+SHIFT+P"),
        ("RESET", "CMD+SHIFT+R"),
        ("LOG", "CMD+SHIFT+L"),
    ]
}

fn workspace_snapshot_block(snapshot: &RuntimeSnapshot) -> gtk::Box {
    let block = gtk::Box::new(Orientation::Vertical, 8);
    block.add_css_class("workspace-block");

    block.append(&section_label("WORKTREE"));
    block.append(&detail_line(
        &current_workspace_cwd(snapshot).unwrap_or_else(|| "No active cwd".to_string()),
    ));

    block.append(&section_label("TASK SNAPSHOT"));
    let task_summary = snapshot
        .task_panes
        .iter()
        .filter(|row| row.workspace.as_deref() == Some(snapshot.active_workspace.as_str()))
        .take(3)
        .map(|pane| {
            let state = if pane.is_failed {
                "failed"
            } else if pane.is_dead {
                "dead"
            } else {
                "running"
            };
            format!("pane {} {}", pane.pane_id, state)
        })
        .collect::<Vec<_>>();
    if task_summary.is_empty() {
        block.append(&detail_line("No tracked task panes"));
    } else {
        for line in task_summary {
            block.append(&detail_line(&line));
        }
    }

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

    block
}

struct TerminalAreaView {
    root: gtk::Widget,
    panes: Vec<TerminalPaneHandle>,
}

fn build_terminal_area(
    terminals: &[Rc<TerminalHandle>],
    split: TwoPaneSplit,
    zoomed_terminal_index: Option<usize>,
) -> TerminalAreaView {
    if terminals.is_empty() {
        let empty = gtk::Box::new(Orientation::Vertical, 0);
        empty.add_css_class("pane-surface");
        empty.add_css_class("live-terminal-host");
        empty.set_hexpand(true);
        empty.set_vexpand(true);
        return TerminalAreaView {
            root: empty.upcast(),
            panes: Vec::new(),
        };
    }

    if let Some(index) = zoomed_terminal_index.filter(|index| *index < terminals.len()) {
        let pane = terminal_surface(index, terminals[index].clone());
        return TerminalAreaView {
            root: pane.root.clone().upcast(),
            panes: vec![pane.handle],
        };
    }

    match terminals.len().max(1) {
        1 => {
            let pane = terminal_surface(0, terminals[0].clone());
            TerminalAreaView {
                root: pane.root.clone().upcast(),
                panes: vec![pane.handle],
            }
        }
        2 => {
            let start = terminal_surface(0, terminals[0].clone());
            let end = terminal_surface(1, terminals[1].clone());
            let split = match split {
                TwoPaneSplit::SideBySide => split_horizontal(start.root.clone(), end.root.clone()),
                TwoPaneSplit::Stacked => split_vertical(start.root.clone(), end.root.clone()),
            };
            TerminalAreaView {
                root: split.upcast(),
                panes: vec![start.handle, end.handle],
            }
        }
        3 => {
            let left = terminal_surface(0, terminals[0].clone());
            let top_right = terminal_surface(1, terminals[1].clone());
            let bottom_right = terminal_surface(2, terminals[2].clone());
            let right = split_vertical(top_right.root.clone(), bottom_right.root.clone());
            let split = split_horizontal(left.root.clone(), right);
            TerminalAreaView {
                root: split.upcast(),
                panes: vec![left.handle, top_right.handle, bottom_right.handle],
            }
        }
        _ => {
            let top_left = terminal_surface(0, terminals[0].clone());
            let bottom_left = terminal_surface(1, terminals[1].clone());
            let top_right = terminal_surface(2, terminals[2].clone());
            let bottom_right = terminal_surface(3, terminals[3].clone());
            let left = split_vertical(top_left.root.clone(), bottom_left.root.clone());
            let right = split_vertical(top_right.root.clone(), bottom_right.root.clone());
            let split = split_horizontal(left, right);
            TerminalAreaView {
                root: split.upcast(),
                panes: vec![
                    top_left.handle,
                    bottom_left.handle,
                    top_right.handle,
                    bottom_right.handle,
                ],
            }
        }
    }
}

struct TerminalPaneView {
    root: gtk::Box,
    handle: TerminalPaneHandle,
}

fn terminal_surface(index: usize, handle: Rc<TerminalHandle>) -> TerminalPaneView {
    let terminal_view = build_terminal_widget(handle);
    terminal_view.root.set_hexpand(true);
    terminal_view.root.set_vexpand(true);
    terminal_view
        .root
        .add_css_class("workspace-terminal-surface");
    TerminalPaneView {
        root: terminal_view.root.clone(),
        handle: TerminalPaneHandle {
            index,
            drag_handle: terminal_view.drag_handle,
            drop_target: terminal_view.drop_target,
            focus_target: terminal_view.focus_target,
            close_button: terminal_view.close_button,
        },
    }
}

fn split_horizontal(start: impl IsA<gtk::Widget>, end: impl IsA<gtk::Widget>) -> gtk::Paned {
    let split = gtk::Paned::new(Orientation::Horizontal);
    split.add_css_class("shell-split");
    split.set_wide_handle(true);
    split.set_hexpand(true);
    split.set_vexpand(true);
    split.set_resize_start_child(true);
    split.set_resize_end_child(true);
    split.set_shrink_start_child(false);
    split.set_shrink_end_child(false);
    split.set_start_child(Some(&start));
    split.set_end_child(Some(&end));
    bind_split_ratio(&split, Orientation::Horizontal, 0.5);
    split
}

fn split_vertical(start: impl IsA<gtk::Widget>, end: impl IsA<gtk::Widget>) -> gtk::Paned {
    let split = gtk::Paned::new(Orientation::Vertical);
    split.add_css_class("shell-split");
    split.set_wide_handle(true);
    split.set_hexpand(true);
    split.set_vexpand(true);
    split.set_resize_start_child(true);
    split.set_resize_end_child(true);
    split.set_shrink_start_child(false);
    split.set_shrink_end_child(false);
    split.set_start_child(Some(&start));
    split.set_end_child(Some(&end));
    bind_split_ratio(&split, Orientation::Vertical, 0.5);
    split
}

fn bind_split_ratio(split: &gtk::Paned, orientation: Orientation, ratio: f64) {
    let applied = Rc::new(std::cell::Cell::new(false));
    let updater: Rc<dyn Fn()> = {
        let split = split.clone();
        let applied = Rc::clone(&applied);
        Rc::new(move || {
            if applied.get() {
                return;
            }
            let extent = split.max_position().max(match orientation {
                Orientation::Horizontal => split.width(),
                Orientation::Vertical => split.height(),
                _ => 0,
            });
            if extent > 0 {
                split.set_position(((extent as f64) * ratio).round() as i32);
                applied.set(true);
            }
        })
    };

    {
        let updater = Rc::clone(&updater);
        glib::idle_add_local_once(move || updater());
    }

    {
        let updater = Rc::clone(&updater);
        split.connect_map(move |_| {
            let updater = Rc::clone(&updater);
            glib::idle_add_local_once(move || updater());
        });
    }

    let property = match orientation {
        Orientation::Horizontal => "width",
        Orientation::Vertical => "height",
        _ => return,
    };
    let property_updater = Rc::clone(&updater);
    split.connect_notify_local(Some(property), move |_, _| {
        let updater = Rc::clone(&property_updater);
        glib::idle_add_local_once(move || updater());
    });
    let max_position_updater = Rc::clone(&updater);
    split.connect_notify_local(Some("max-position"), move |_, _| {
        let updater = Rc::clone(&max_position_updater);
        glib::idle_add_local_once(move || updater());
    });
}

fn shortcut_bar() -> gtk::ScrolledWindow {
    let shortcuts = gtk::Box::new(Orientation::Horizontal, 8);
    shortcuts.add_css_class("shortcut-strip");
    shortcuts.set_hexpand(false);
    shortcuts.set_vexpand(false);
    shortcuts.set_halign(Align::Start);
    shortcuts.set_valign(Align::Center);
    shortcuts.set_baseline_position(gtk::BaselinePosition::Center);
    shortcuts.set_size_request(0, -1);

    let entries = shortcut_entries();
    for (index, (label, accel)) in entries.iter().enumerate() {
        shortcuts.append(&shortcut_chip(label, accel));
        if index + 1 < entries.len() {
            shortcuts.append(&shortcut_separator());
        }
    }

    let scroller = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(false)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .has_frame(false)
        .child(&shortcuts)
        .build();
    scroller.add_css_class("shortcut-bar");
    scroller.set_overlay_scrolling(false);
    scroller.set_kinetic_scrolling(false);
    scroller.set_min_content_width(0);
    scroller.set_size_request(0, -1);
    scroller.set_propagate_natural_width(false);
    scroller.set_propagate_natural_height(false);
    scroller.set_halign(Align::Fill);
    scroller.set_valign(Align::Center);
    scroller.set_can_focus(false);

    let drag_origin = Rc::new(Cell::new(0.0));
    let scroller_begin = scroller.clone();
    let drag_origin_begin = Rc::clone(&drag_origin);
    let drag = gtk::GestureDrag::new();
    drag.set_button(gdk::BUTTON_PRIMARY);
    drag.connect_drag_begin(move |_, _, _| {
        drag_origin_begin.set(scroller_begin.hadjustment().value());
    });
    let scroller_update = scroller.clone();
    let drag_origin_update = Rc::clone(&drag_origin);
    drag.connect_drag_update(move |_, offset_x, _| {
        let adjustment = scroller_update.hadjustment();
        let max_value = (adjustment.upper() - adjustment.page_size()).max(adjustment.lower());
        let next = (drag_origin_update.get() - offset_x).clamp(adjustment.lower(), max_value);
        if (adjustment.value() - next).abs() >= 0.5 {
            adjustment.set_value(next);
        }
    });
    scroller.add_controller(drag);
    scroller
}

fn shortcut_chip(label: &str, accel: &str) -> gtk::Box {
    let chip = gtk::Box::new(Orientation::Horizontal, 8);
    chip.add_css_class("shortcut-chip");
    chip.set_valign(Align::Center);

    let action = gtk::Label::new(Some(label));
    action.set_halign(Align::Start);
    action.set_valign(Align::Center);
    action.add_css_class("shortcut-action");

    let combo = gtk::Label::new(Some(accel));
    combo.set_halign(Align::End);
    combo.set_valign(Align::Center);
    combo.add_css_class("shortcut-combo");

    chip.append(&action);
    chip.append(&combo);
    chip
}

fn shortcut_separator() -> gtk::Label {
    let separator = gtk::Label::new(Some("|"));
    separator.add_css_class("shortcut-separator");
    separator.set_valign(Align::Center);
    separator
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
    current_workspace_raw_cwd(snapshot).map(|cwd| shorten_cwd(&cwd))
}

fn current_workspace_raw_cwd(snapshot: &RuntimeSnapshot) -> Option<String> {
    snapshot
        .task_panes
        .iter()
        .find(|row| row.workspace.as_deref() == Some(snapshot.active_workspace.as_str()))
        .and_then(|row| row.current_working_dir.as_ref())
        .cloned()
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|path| path.to_str().map(|cwd| cwd.to_string()))
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
