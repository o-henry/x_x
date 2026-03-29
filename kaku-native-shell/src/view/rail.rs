use crate::snapshot::{RuntimeSnapshot, WorkspaceSummary};
use crate::view::display_workspace_name;
use gtk::pango::EllipsizeMode;
use gtk::prelude::*;
use gtk::{Align, ListBox, ListBoxRow, Orientation, SelectionMode};

pub struct RailView {
    pub root: gtk::Box,
    pub workspace_buttons: Vec<(String, gtk::Button)>,
    pub inbox_toggle_button: gtk::Button,
}

pub fn build_rail(snapshot: &RuntimeSnapshot, collapsed: bool, _inbox_collapsed: bool) -> RailView {
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

    let inbox_toggle_button = gtk::Button::new();
    inbox_toggle_button.set_visible(false);

    RailView {
        root: rail,
        workspace_buttons,
        inbox_toggle_button,
    }
}

fn workspace_items(_snapshot: &RuntimeSnapshot, summary: &WorkspaceSummary) -> Option<gtk::Widget> {
    let items = ListBox::new();
    items.add_css_class("rail-work-items");
    items.add_css_class("rail-work-items-list");
    items.set_selection_mode(SelectionMode::None);
    items.set_activate_on_single_click(false);
    items.set_show_separators(false);
    items.set_vexpand(false);
    items.set_valign(Align::Start);

    if let Some(detail) = summary
        .detail
        .as_deref()
        .filter(|detail| !detail.trim().is_empty() && detail.trim() != "IDLE")
    {
        let (primary, trailing) = split_workspace_detail(detail);
        items.append(&workspace_item_row(
            "WORKTREE",
            &primary,
            trailing.as_deref(),
            Some("running"),
        ));
    }

    if items.first_child().is_some() {
        Some(items.upcast())
    } else {
        None
    }
}

fn workspace_item_row(
    title: &str,
    primary: &str,
    trailing: Option<&str>,
    state: Option<&str>,
) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.add_css_class("rail-work-item");
    row.add_css_class("rail-work-item-row");
    row.set_activatable(false);
    row.set_selectable(false);
    row.set_height_request(40);
    if let Some(state) = state {
        row.add_css_class(&format!("rail-work-item-{state}"));
    }

    let content = gtk::Box::new(Orientation::Horizontal, 10);
    content.add_css_class("rail-work-item-content");
    content.set_hexpand(true);
    content.set_halign(Align::Fill);
    content.set_valign(Align::Center);
    content.set_vexpand(false);
    row.set_child(Some(&content));

    let indicator = gtk::Box::new(Orientation::Horizontal, 0);
    indicator.add_css_class("rail-work-item-dot");
    indicator.set_size_request(8, 8);
    indicator.set_valign(Align::Center);
    indicator.set_halign(Align::Center);
    indicator.set_margin_start(4);
    content.append(&indicator);

    let title = gtk::Label::new(Some(&title.to_uppercase()));
    title.set_halign(Align::Start);
    title.set_valign(Align::Center);
    title.set_wrap(false);
    title.set_ellipsize(EllipsizeMode::End);
    title.add_css_class("rail-work-item-title");
    title.set_xalign(0.0);
    content.append(&title);

    let normalized_primary = if primary.trim().is_empty() {
        "~".to_string()
    } else {
        primary.to_uppercase()
    };
    let primary = gtk::Label::new(Some(&normalized_primary));
    primary.set_halign(Align::Start);
    primary.set_hexpand(true);
    primary.set_wrap(false);
    primary.set_ellipsize(EllipsizeMode::End);
    primary.set_valign(Align::Center);
    primary.set_xalign(0.0);
    primary.add_css_class("rail-work-item-primary");
    content.append(&primary);

    if let Some(trailing) = trailing.filter(|value| !value.trim().is_empty()) {
        let trailing = gtk::Label::new(Some(&trailing.to_uppercase()));
        trailing.set_halign(Align::End);
        trailing.set_wrap(false);
        trailing.set_valign(Align::Center);
        trailing.set_xalign(1.0);
        trailing.add_css_class("rail-work-item-trailing");
        content.append(&trailing);
    }

    row
}

fn split_workspace_detail(detail: &str) -> (String, Option<String>) {
    let mut parts = detail
        .split(" • ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return ("~".to_string(), None);
    }

    if parts.len() == 1 {
        return (parts.remove(0).to_string(), None);
    }

    let trailing = parts.pop().map(str::to_string);
    let primary = parts.join(" / ");
    (primary, trailing)
}
