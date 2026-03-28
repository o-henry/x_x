use crate::snapshot::RuntimeSnapshot;
use crate::view::chrome_action_button;
use gtk::prelude::*;
use gtk::Orientation;

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

    let spacer = gtk::Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let active = gtk::Label::new(Some(&snapshot.active_workspace));
    active.add_css_class("chrome-pill");

    let refresh_button = chrome_action_button("Refresh");
    let terminal_button = chrome_action_button("Open Terminal");

    chrome.append(&spacer);
    chrome.append(&active);

    ChromeView {
        root: chrome,
        refresh_button,
        terminal_button,
    }
}
