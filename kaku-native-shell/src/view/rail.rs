use crate::snapshot::{RuntimeSnapshot, WorkspaceSummary};
use crate::view::display_workspace_name;
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct RailView {
    pub root: gtk::Box,
    pub workspace_buttons: Vec<(String, gtk::Button)>,
    pub inbox_toggle_button: gtk::Button,
}

pub fn build_rail(snapshot: &RuntimeSnapshot, collapsed: bool, inbox_collapsed: bool) -> RailView {
    let rail = gtk::Box::new(Orientation::Vertical, 0);
    rail.add_css_class("workspace-rail");
    rail.set_margin_start(0);
    rail.set_margin_end(0);
    rail.set_margin_top(0);
    rail.set_margin_bottom(0);

    let top_region = gtk::Box::new(Orientation::Vertical, 0);
    top_region.set_vexpand(true);
    rail.append(&top_region);

    let mut workspace_buttons = Vec::new();
    for summary in &snapshot.workspaces {
        let workspace_group = gtk::Box::new(Orientation::Vertical, 0);
        workspace_group.add_css_class("rail-workspace-group");

        let row = gtk::Button::new();
        row.set_halign(Align::Fill);
        row.set_hexpand(true);
        row.set_height_request(40);
        row.add_css_class("rail-row");
        row.add_css_class("rail-workspace-row");
        row.add_css_class("pane-titlebar");
        if collapsed {
            row.add_css_class("collapsed");
        }
        if summary.name == snapshot.active_workspace {
            row.add_css_class("active");
        }

        let outer = gtk::Box::new(
            if collapsed {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            },
            if collapsed { 2 } else { 10 },
        );
        outer.set_hexpand(true);
        outer.set_valign(Align::Center);
        outer.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Fill
        });
        outer.set_margin_start(0);
        outer.set_margin_end(0);
        outer.set_margin_top(0);
        outer.set_margin_bottom(0);

        let display_name = display_workspace_name(&summary.name);
        let collapsed_name = display_name
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "•".to_string());

        let name_box = gtk::Box::new(Orientation::Vertical, 1);
        name_box.set_hexpand(true);
        name_box.set_valign(Align::Center);
        name_box.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Start
        });

        let display_label = if collapsed {
            collapsed_name.clone()
        } else {
            display_name.to_uppercase()
        };
        let name = gtk::Label::new(Some(&display_label));
        name.set_valign(Align::Center);
        name.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Start
        });
        name.set_hexpand(!collapsed);
        name.add_css_class("rail-name");
        name_box.append(&name);
        if !collapsed {
            name_box.set_margin_start(12);
            name_box.set_margin_end(12);
        }
        outer.append(&name_box);
        row.set_child(Some(&outer));

        workspace_buttons.push((summary.name.clone(), row.clone()));
        workspace_group.append(&row);
        if !collapsed {
            if let Some(items) = workspace_items(snapshot, summary) {
                workspace_group.append(&items);
            }
        }
        top_region.append(&workspace_group);
    }

    let bottom_section = gtk::Box::new(Orientation::Vertical, 0);
    bottom_section.set_vexpand(false);
    rail.append(&bottom_section);

    let inbox_toggle_button = gtk::Button::new();
    inbox_toggle_button.add_css_class("rail-toggle-button");
    let icon_path = if inbox_collapsed {
        format!(
            "{}/assets/icons/inbox-toggle-up.svg",
            env!("CARGO_MANIFEST_DIR")
        )
    } else {
        format!(
            "{}/assets/icons/inbox-toggle-down.svg",
            env!("CARGO_MANIFEST_DIR")
        )
    };
    let toggle_icon = gtk::Image::from_file(icon_path);
    toggle_icon.set_pixel_size(11);
    inbox_toggle_button.set_child(Some(&toggle_icon));

    let inbox_header = gtk::Box::new(Orientation::Horizontal, 10);
    inbox_header.add_css_class("pane-titlebar");
    inbox_header.add_css_class("rail-titlebar");
    inbox_header.add_css_class("rail-inbox-header");
    if inbox_collapsed {
        inbox_header.add_css_class("rail-inbox-collapsed");
    }
    inbox_header.set_halign(Align::Fill);
    inbox_header.set_hexpand(true);
    inbox_header.set_height_request(40);
    let inbox_label = gtk::Label::new(Some("INBOX"));
    inbox_label.set_halign(Align::Start);
    inbox_label.set_hexpand(true);
    inbox_label.set_valign(Align::Center);
    inbox_label.add_css_class("pane-title");
    inbox_label.add_css_class("rail-section-title");
    inbox_label.set_margin_start(12);
    inbox_label.set_margin_end(12);
    inbox_label.set_margin_top(0);
    inbox_label.set_margin_bottom(0);
    inbox_header.append(&inbox_label);
    inbox_toggle_button.set_valign(Align::Center);
    inbox_toggle_button.set_halign(Align::Center);
    inbox_toggle_button.set_margin_end(10);
    inbox_header.append(&inbox_toggle_button);
    bottom_section.append(&inbox_header);

    let inbox_region = gtk::Box::new(Orientation::Vertical, 0);
    inbox_region.add_css_class("rail-inbox-region");
    inbox_region.set_vexpand(true);
    inbox_region.set_visible(!inbox_collapsed);
    inbox_region.set_margin_bottom(0);

    let selected = snapshot.active_workspace.as_str();
    let unread = snapshot
        .notifications
        .iter()
        .filter(|row| row.workspace == selected && row.unread)
        .take(6)
        .cloned()
        .collect::<Vec<_>>();

    if unread.is_empty() {
        let empty = gtk::Label::new(Some("NO UNREAD NOTIFICATIONS"));
        empty.set_halign(Align::Start);
        empty.set_wrap(true);
        empty.set_margin_start(14);
        empty.set_margin_end(14);
        empty.set_margin_top(8);
        empty.add_css_class("empty-state");
        inbox_region.append(&empty);
    } else {
        let list = gtk::Box::new(Orientation::Vertical, 6);
        list.set_margin_start(14);
        list.set_margin_end(14);
        list.set_margin_top(8);
        list.set_margin_bottom(14);
        for row in unread {
            let outer = gtk::Box::new(Orientation::Vertical, 4);
            outer.add_css_class("list-row");
            let title = gtk::Label::new(Some(&row.title));
            title.set_halign(Align::Start);
            title.add_css_class("row-title");
            outer.append(&title);
            if let Some(body) = row.body {
                let detail = gtk::Label::new(Some(&body));
                detail.set_halign(Align::Start);
                detail.set_wrap(true);
                detail.add_css_class("row-detail");
                outer.append(&detail);
            }
            list.append(&outer);
        }
        inbox_region.append(&list);
    }
    bottom_section.append(&inbox_region);

    RailView {
        root: rail,
        workspace_buttons,
        inbox_toggle_button,
    }
}

fn workspace_items(snapshot: &RuntimeSnapshot, summary: &WorkspaceSummary) -> Option<gtk::Box> {
    let items = gtk::Box::new(Orientation::Vertical, 0);
    items.add_css_class("rail-work-items");

    if let Some(detail) = summary
        .detail
        .as_deref()
        .filter(|detail| !detail.trim().is_empty() && detail.trim() != "IDLE")
    {
        items.append(&workspace_item_row("WORKTREE", detail, Some("running")));
    }

    for row in snapshot
        .notifications
        .iter()
        .filter(|row| row.workspace == summary.name && row.unread)
        .take(4)
    {
        let detail = row.body.as_deref().unwrap_or("");
        items.append(&workspace_item_row(&row.title, detail, Some("unread")));
    }

    for pane in snapshot
        .task_panes
        .iter()
        .filter(|pane| pane.workspace.as_deref() == Some(summary.name.as_str()))
        .take(4)
    {
        let title = if pane.is_failed {
            format!("PANE {} FAILED", pane.pane_id)
        } else if pane.is_dead {
            format!("PANE {} DONE", pane.pane_id)
        } else {
            format!("PANE {} RUNNING", pane.pane_id)
        };
        let detail = pane
            .current_working_dir
            .as_deref()
            .map(shorten_work_item_path)
            .unwrap_or_default();
        let state = if pane.is_failed {
            Some("failed")
        } else if pane.is_dead {
            None
        } else {
            Some("running")
        };
        items.append(&workspace_item_row(&title, &detail, state));
    }

    if items.first_child().is_some() {
        Some(items)
    } else {
        None
    }
}

fn workspace_item_row(title: &str, detail: &str, state: Option<&str>) -> gtk::Box {
    let row = gtk::Box::new(Orientation::Horizontal, 8);
    row.add_css_class("rail-work-item");
    row.set_height_request(40);
    row.set_halign(Align::Fill);
    row.set_hexpand(true);
    row.set_valign(Align::Center);
    if let Some(state) = state {
        row.add_css_class(&format!("rail-work-item-{state}"));
    }

    let indicator = gtk::Box::new(Orientation::Horizontal, 0);
    indicator.add_css_class("rail-work-item-dot");
    indicator.set_size_request(8, 8);
    indicator.set_valign(Align::Center);
    row.append(&indicator);

    let copy = gtk::Box::new(Orientation::Vertical, 2);
    copy.set_hexpand(true);
    copy.set_halign(Align::Fill);
    copy.set_valign(Align::Center);

    let title = gtk::Label::new(Some(&title.to_uppercase()));
    title.set_halign(Align::Start);
    title.set_hexpand(true);
    title.set_valign(Align::Center);
    title.add_css_class("rail-work-item-title");
    copy.append(&title);

    if !detail.trim().is_empty() {
        let detail = gtk::Label::new(Some(&detail.to_uppercase()));
        detail.set_halign(Align::Start);
        detail.set_hexpand(true);
        detail.set_wrap(true);
        detail.set_valign(Align::Center);
        detail.add_css_class("rail-work-item-detail");
        copy.append(&detail);
    }

    row.append(&copy);
    row
}

fn shorten_work_item_path(value: &str) -> String {
    let without_scheme = value.strip_prefix("file://").unwrap_or(value);
    let trimmed_home = std::env::var("HOME")
        .ok()
        .and_then(|home| without_scheme.strip_prefix(&home).map(|rest| format!("~{rest}")))
        .unwrap_or_else(|| without_scheme.to_string());
    let parts = trimmed_home
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.len() <= 3 {
        return trimmed_home;
    }
    let tail = &parts[parts.len() - 3..];
    format!("…/{}/{}/{}", tail[0], tail[1], tail[2])
}
