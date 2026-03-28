use crate::snapshot::RuntimeSnapshot;
use crate::view::workspace::workspace_status_tokens;
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct RailView {
    pub root: gtk::Box,
    pub workspace_buttons: Vec<(String, gtk::Button)>,
}

pub fn build_rail(snapshot: &RuntimeSnapshot) -> RailView {
    let rail = gtk::Box::new(Orientation::Vertical, 4);
    rail.add_css_class("workspace-rail");
    rail.set_margin_start(0);
    rail.set_margin_end(0);
    rail.set_margin_top(10);
    rail.set_margin_bottom(10);

    let eyebrow = gtk::Label::new(Some("WORKSPACES"));
    eyebrow.set_halign(Align::Start);
    eyebrow.add_css_class("rail-eyebrow");
    eyebrow.set_margin_start(10);
    eyebrow.set_margin_end(10);
    eyebrow.set_margin_bottom(6);
    rail.append(&eyebrow);

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

        let meta = gtk::Label::new(Some(&workspace_badge_text(
            summary.unread_count,
            summary.running_count,
            summary.failed_count,
        )));
        meta.set_halign(Align::Start);
        meta.add_css_class("rail-meta");
        name_box.append(&name);
        name_box.append(&meta);

        let trailing = gtk::Label::new(Some(
            &workspace_status_tokens(
                summary.status.as_deref(),
                summary.progress,
                summary.log_count,
            )
            .join(" "),
        ));
        trailing.set_halign(Align::End);
        trailing.add_css_class("rail-trailing");

        outer.append(&name_box);
        outer.append(&trailing);
        row.set_child(Some(&outer));

        workspace_buttons.push((summary.name.clone(), row.clone()));
        rail.append(&row);
    }

    RailView {
        root: rail,
        workspace_buttons,
    }
}

pub fn workspace_badge_text(
    unread_count: usize,
    running_count: usize,
    failed_count: usize,
) -> String {
    format!("{unread_count}U {running_count}R {failed_count}F")
}
