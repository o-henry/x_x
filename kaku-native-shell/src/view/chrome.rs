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

    let active = gtk::Label::new(Some(&active_context_label(snapshot)));
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

fn active_context_label(snapshot: &RuntimeSnapshot) -> String {
    snapshot
        .task_panes
        .iter()
        .find(|row| row.workspace.as_deref() == Some(snapshot.active_workspace.as_str()))
        .and_then(|row| row.current_working_dir.as_ref())
        .map(|cwd| shorten_cwd(cwd))
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .and_then(|path| path.to_str().map(|s| shorten_cwd(s)))
        })
        .unwrap_or_else(|| snapshot.active_workspace.clone())
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
