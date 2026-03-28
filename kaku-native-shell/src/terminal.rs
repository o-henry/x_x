use gtk::gdk;
use gtk::prelude::*;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::cell::RefCell;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

pub fn build_terminal_widget(cwd: Option<String>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("terminal-host");
    root.set_hexpand(true);
    root.set_vexpand(true);

    let scroller = gtk::ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .has_frame(false)
        .build();

    let view = gtk::TextView::new();
    view.add_css_class("terminal-view");
    view.set_editable(false);
    view.set_cursor_visible(false);
    view.set_monospace(true);
    view.set_wrap_mode(gtk::WrapMode::WordChar);
    view.set_top_margin(14);
    view.set_bottom_margin(14);
    view.set_left_margin(14);
    view.set_right_margin(14);
    scroller.set_child(Some(&view));
    root.append(&scroller);

    let buffer = view.buffer();

    let pty_system = native_pty_system();
    let pair = match pty_system.openpty(PtySize {
        rows: 30,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    }) {
        Ok(pair) => pair,
        Err(err) => {
            root.append(&error_label(&format!("Failed to open PTY: {err}")));
            return root;
        }
    };

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut cmd = CommandBuilder::new(shell);
    cmd.env("TERM", "dumb");
    cmd.env("NO_COLOR", "1");
    if let Some(dir) = cwd.and_then(normalize_cwd) {
        cmd.cwd(dir);
    }

    let _child = match pair.slave.spawn_command(cmd) {
        Ok(child) => child,
        Err(err) => {
            root.append(&error_label(&format!("Failed to spawn shell: {err}")));
            return root;
        }
    };

    let reader = match pair.master.try_clone_reader() {
        Ok(reader) => reader,
        Err(err) => {
            root.append(&error_label(&format!("Failed to clone PTY reader: {err}")));
            return root;
        }
    };
    let writer = match pair.master.take_writer() {
        Ok(writer) => writer,
        Err(err) => {
            root.append(&error_label(&format!(
                "Failed to acquire PTY writer: {err}"
            )));
            return root;
        }
    };

    let writer = Rc::new(RefCell::new(writer));
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut chunk = [0u8; 4096];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(chunk[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let buffer_for_output = buffer.clone();
    let view_for_output = view.clone();
    glib::timeout_add_local(Duration::from_millis(16), move || {
        let mut changed = false;
        while let Ok(bytes) = rx.try_recv() {
            append_output(&buffer_for_output, &String::from_utf8_lossy(&bytes));
            changed = true;
        }
        if changed {
            let mut end = buffer_for_output.end_iter();
            view_for_output.scroll_to_iter(&mut end, 0.0, false, 0.0, 1.0);
        }
        glib::ControlFlow::Continue
    });

    let writer_for_keys = Rc::clone(&writer);
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, state| {
        if state.contains(gdk::ModifierType::META_MASK)
            || state.contains(gdk::ModifierType::CONTROL_MASK)
        {
            return glib::Propagation::Proceed;
        }
        if let Some(sequence) = key_to_bytes(key) {
            let _ = writer_for_keys.borrow_mut().write_all(&sequence);
            let _ = writer_for_keys.borrow_mut().flush();
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    view.add_controller(key_controller);

    let click = gtk::GestureClick::new();
    let view_for_focus = view.clone();
    click.connect_pressed(move |_, _, _, _| {
        view_for_focus.grab_focus();
    });
    view.add_controller(click);

    root
}

fn append_output(buffer: &gtk::TextBuffer, text: &str) {
    let mut end = buffer.end_iter();
    buffer.insert(&mut end, text);
}

fn key_to_bytes(key: gdk::Key) -> Option<Vec<u8>> {
    match key {
        gdk::Key::Return => Some(b"\r".to_vec()),
        gdk::Key::BackSpace => Some(vec![0x7f]),
        gdk::Key::Tab => Some(b"\t".to_vec()),
        gdk::Key::Up => Some(b"\x1b[A".to_vec()),
        gdk::Key::Down => Some(b"\x1b[B".to_vec()),
        gdk::Key::Right => Some(b"\x1b[C".to_vec()),
        gdk::Key::Left => Some(b"\x1b[D".to_vec()),
        _ => key.to_unicode().map(|ch| ch.to_string().into_bytes()),
    }
}

fn error_label(message: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(message));
    label.set_halign(gtk::Align::Start);
    label.set_margin_start(14);
    label.set_margin_end(14);
    label.set_margin_top(14);
    label.add_css_class("empty-state");
    label
}

fn normalize_cwd(cwd: String) -> Option<PathBuf> {
    let without_scheme = cwd.strip_prefix("file://").unwrap_or(&cwd);
    if without_scheme.is_empty() {
        None
    } else {
        Some(PathBuf::from(without_scheme))
    }
}
