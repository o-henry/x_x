use crate::snapshot::RuntimeSnapshot;
use crate::view::chrome_action_button;
use gtk::prelude::*;
use gtk::{Align, Orientation};

pub struct ChromeView {
    pub root: gtk::Box,
    pub refresh_button: gtk::Button,
    pub terminal_button: gtk::Button,
}

pub fn build_chrome(snapshot: &RuntimeSnapshot) -> ChromeView {
    let chrome = gtk::Box::new(Orientation::Horizontal, 8);
    chrome.add_css_class("shell-chrome");
    chrome.set_margin_start(10);
    chrome.set_margin_end(10);
    chrome.set_margin_top(2);
    chrome.set_margin_bottom(0);

    let title_box = gtk::Box::new(Orientation::Vertical, 1);
    title_box.set_margin_top(5);
    let title = gtk::Label::new(Some("Kaku"));
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

    let active = gtk::Label::new(Some(&snapshot.active_workspace));
    active.add_css_class("chrome-pill");

    let refresh_button = chrome_action_button("Refresh");
    let terminal_button = chrome_action_button("Open Terminal");

    chrome.append(&title_box);
    chrome.append(&spacer);
    chrome.append(&active);

    ChromeView {
        root: chrome,
        refresh_button,
        terminal_button,
    }
}
