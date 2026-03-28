use crate::snapshot::RuntimeSnapshot;
use crate::view::action_button;
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct ChromeView {
    pub root: gtk::Box,
    pub refresh_button: gtk::Button,
    pub terminal_button: gtk::Button,
}

pub fn build_chrome(snapshot: &RuntimeSnapshot) -> ChromeView {
    let chrome = gtk::Box::new(Orientation::Horizontal, 12);
    chrome.add_css_class("shell-chrome");
    chrome.set_margin_start(14);
    chrome.set_margin_end(16);
    chrome.set_margin_top(10);
    chrome.set_margin_bottom(8);

    let title_box = gtk::Box::new(Orientation::Vertical, 2);
    let title = gtk::Label::new(Some("Kaku Native Shell"));
    title.set_halign(Align::Start);
    title.add_css_class("chrome-title");

    let subtitle = gtk::Label::new(Some(&format!(
        "{} ws  {}U  {}T",
        snapshot.workspaces.len(),
        snapshot
            .notifications
            .iter()
            .filter(|row| row.unread)
            .count(),
        snapshot.task_panes.len()
    )));
    subtitle.set_halign(Align::Start);
    subtitle.add_css_class("chrome-subtitle");
    title_box.append(&title);
    title_box.append(&subtitle);

    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let active = gtk::Label::new(Some(&format!("◉ {}", snapshot.active_workspace)));
    active.add_css_class("chrome-pill");

    let refresh_button = action_button("↻");
    let terminal_button = action_button("⌂ open terminal");

    chrome.append(&title_box);
    chrome.append(&spacer);
    chrome.append(&active);
    chrome.append(&refresh_button);
    chrome.append(&terminal_button);

    ChromeView {
        root: chrome,
        refresh_button,
        terminal_button,
    }
}
