use crate::snapshot::RuntimeSnapshot;
use crate::view::workspace::workspace_status_tokens;
use gtk::prelude::*;
use gtk::{Align, Orientation};
pub struct RailView {
    pub root: gtk::Box,
    pub workspace_buttons: Vec<(String, gtk::Button)>,
    pub inbox_toggle_button: gtk::Button,
}

pub fn build_rail(snapshot: &RuntimeSnapshot, collapsed: bool, inbox_collapsed: bool) -> RailView {
    let rail = gtk::Box::new(Orientation::Vertical, 4);
    rail.add_css_class("workspace-rail");
    rail.set_margin_start(0);
    rail.set_margin_end(0);
    rail.set_margin_top(0);
    rail.set_margin_bottom(0);

    let header = gtk::Box::new(Orientation::Horizontal, 10);
    header.add_css_class("pane-titlebar");
    header.add_css_class("rail-titlebar");
    header.set_halign(Align::Fill);
    header.set_hexpand(true);

    let eyebrow = gtk::Label::new(Some("WORKSPACES"));
    eyebrow.set_halign(Align::Start);
    eyebrow.set_hexpand(true);
    eyebrow.add_css_class("pane-title");
    eyebrow.set_margin_start(12);
    eyebrow.set_margin_end(12);
    eyebrow.set_margin_top(8);
    eyebrow.set_margin_bottom(8);
    if collapsed {
        eyebrow.set_visible(false);
    }

    header.append(&eyebrow);
    rail.append(&header);

    let top_region = gtk::Box::new(Orientation::Vertical, 0);
    top_region.set_vexpand(true);
    rail.append(&top_region);

    let mut workspace_buttons = Vec::new();
    for summary in &snapshot.workspaces {
        let row = gtk::Button::new();
        row.set_halign(Align::Fill);
        row.set_hexpand(true);
        row.add_css_class("rail-row");
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
            if collapsed { 2 } else { 8 },
        );
        outer.set_hexpand(true);
        outer.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Fill
        });

        let name_box = gtk::Box::new(Orientation::Vertical, 2);
        name_box.set_hexpand(!collapsed);
        name_box.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Fill
        });

        let collapsed_name = summary
            .name
            .chars()
            .next()
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "•".to_string());
        let name = gtk::Label::new(Some(if collapsed {
            &collapsed_name
        } else {
            &summary.name
        }));
        name.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Start
        });
        name.add_css_class("rail-name");

        let meta = gtk::Label::new(Some(&workspace_badge_text(
            summary.unread_count,
            summary.running_count,
            summary.failed_count,
        )));
        meta.set_halign(if collapsed {
            Align::Center
        } else {
            Align::Start
        });
        meta.add_css_class("rail-meta");
        if collapsed {
            meta.set_label(&collapsed_badge_text(
                summary.unread_count,
                summary.running_count,
                summary.failed_count,
                summary.log_count,
            ));
        }
        name_box.append(&name);
        name_box.append(&meta);

        let trailing_text = if collapsed {
            String::new()
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
            trailing.set_visible(false);
        }

        outer.append(&name_box);
        if !collapsed {
            outer.append(&trailing);
        }
        row.set_child(Some(&outer));

        workspace_buttons.push((summary.name.clone(), row.clone()));
        top_region.append(&row);
    }

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
    inbox_header.set_halign(Align::Fill);
    inbox_header.set_hexpand(true);

    let inbox_label = gtk::Label::new(Some("INBOX"));
    inbox_label.set_halign(Align::Start);
    inbox_label.set_hexpand(true);
    inbox_label.add_css_class("pane-title");
    inbox_label.set_margin_start(12);
    inbox_label.set_margin_end(12);
    inbox_label.set_margin_top(8);
    inbox_label.set_margin_bottom(8);
    inbox_header.append(&inbox_label);
    inbox_toggle_button.set_valign(Align::Center);
    inbox_toggle_button.set_halign(Align::Center);
    inbox_toggle_button.set_margin_end(10);
    inbox_header.append(&inbox_toggle_button);
    rail.append(&inbox_header);

    let inbox_region = gtk::Box::new(Orientation::Vertical, 0);
    inbox_region.add_css_class("rail-inbox-region");
    inbox_region.set_vexpand(true);
    inbox_region.set_visible(!inbox_collapsed);
    inbox_region.set_margin_bottom(8);

    let selected = snapshot.active_workspace.as_str();
    let unread = snapshot
        .notifications
        .iter()
        .filter(|row| row.workspace == selected && row.unread)
        .take(6)
        .cloned()
        .collect::<Vec<_>>();

    if unread.is_empty() {
        let empty = gtk::Label::new(Some("No unread notifications"));
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
        list.set_margin_bottom(6);
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
    rail.append(&inbox_region);

    RailView {
        root: rail,
        workspace_buttons,
        inbox_toggle_button,
    }
}

pub fn workspace_badge_text(
    unread_count: usize,
    running_count: usize,
    failed_count: usize,
) -> String {
    format!("{unread_count}U {running_count}R {failed_count}F")
}

fn collapsed_badge_text(
    unread_count: usize,
    running_count: usize,
    failed_count: usize,
    log_count: usize,
) -> String {
    format!("{unread_count}U {running_count}R {failed_count}F {log_count}L")
}
