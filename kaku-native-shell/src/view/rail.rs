use crate::snapshot::RuntimeSnapshot;
use crate::view::workspace::workspace_status_tokens;
use gtk::prelude::*;
use gtk::{Align, Orientation};
use std::path::PathBuf;

pub struct RailView {
    pub root: gtk::Box,
    pub workspace_buttons: Vec<(String, gtk::Button)>,
    pub toggle_button: gtk::Button,
}

pub fn build_rail(snapshot: &RuntimeSnapshot, collapsed: bool) -> RailView {
    let rail = gtk::Box::new(Orientation::Vertical, 4);
    rail.add_css_class("workspace-rail");
    rail.set_margin_start(0);
    rail.set_margin_end(0);
    rail.set_margin_top(8);
    rail.set_margin_bottom(8);

    let header = gtk::Box::new(Orientation::Horizontal, 8);
    header.set_margin_start(14);
    header.set_margin_end(14);
    header.set_margin_bottom(6);
    header.set_halign(Align::Fill);

    let eyebrow = gtk::Label::new(Some("WORKSPACES"));
    eyebrow.set_halign(Align::Start);
    eyebrow.set_hexpand(true);
    eyebrow.add_css_class("rail-eyebrow");
    if collapsed {
        eyebrow.set_visible(false);
    }

    let toggle_button = gtk::Button::new();
    toggle_button.add_css_class("rail-toggle");
    let icon = gtk::Image::from_file(rail_toggle_icon_path(collapsed));
    icon.set_pixel_size(14);
    toggle_button.set_child(Some(&icon));

    header.append(&eyebrow);
    header.append(&toggle_button);
    rail.append(&header);

    let mut workspace_buttons = Vec::new();
    for summary in &snapshot.workspaces {
        let row = gtk::Button::new();
        row.set_halign(Align::Fill);
        row.set_hexpand(true);
        row.add_css_class("rail-row");
        if summary.name == snapshot.active_workspace {
            row.add_css_class("active");
        }

        let outer = gtk::Box::new(Orientation::Horizontal, 8);
        outer.set_hexpand(true);

        let name_box = gtk::Box::new(Orientation::Vertical, 2);
        name_box.set_hexpand(true);

        let name = gtk::Label::new(Some(&summary.name));
        name.set_halign(Align::Start);
        name.add_css_class("rail-name");
        if collapsed {
            name.set_visible(false);
        }

        let meta = gtk::Label::new(Some(&workspace_badge_text(
            summary.unread_count,
            summary.running_count,
            summary.failed_count,
        )));
        meta.set_halign(Align::Start);
        meta.add_css_class("rail-meta");
        if collapsed {
            meta.set_visible(false);
        }
        name_box.append(&name);
        name_box.append(&meta);

        let trailing_text = if collapsed {
            workspace_badge_text(
                summary.unread_count,
                summary.running_count,
                summary.failed_count,
            )
        } else {
            workspace_status_tokens(
                summary.status.as_deref(),
                summary.progress,
                summary.log_count,
            )
            .join(" ")
        };
        let trailing = gtk::Label::new(Some(&trailing_text));
        trailing.set_halign(Align::End);
        trailing.add_css_class("rail-trailing");
        if collapsed {
            trailing.set_halign(Align::Center);
        }

        outer.append(&name_box);
        outer.append(&trailing);
        row.set_child(Some(&outer));

        workspace_buttons.push((summary.name.clone(), row.clone()));
        rail.append(&row);
    }

    RailView {
        root: rail,
        workspace_buttons,
        toggle_button,
    }
}

fn rail_toggle_icon_path(collapsed: bool) -> PathBuf {
    let icon_name = if collapsed { "opend.svg" } else { "closed.svg" };
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("icons")
        .join(icon_name)
}

pub fn workspace_badge_text(
    unread_count: usize,
    running_count: usize,
    failed_count: usize,
) -> String {
    format!("{unread_count}U {running_count}R {failed_count}F")
}
