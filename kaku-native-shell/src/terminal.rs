use config::TermConfig;
use gtk::gdk;
use gtk::glib::translate::ToGlibPtr;
use gtk::pango;
use gtk::prelude::*;
use mux::renderable::{terminal_get_cursor_position, terminal_get_dimensions, terminal_get_lines};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::Duration;
use wezterm_term::color::{ColorAttribute, ColorPalette, SrgbaTuple};
use wezterm_term::{Intensity, Terminal, TerminalConfiguration, TerminalSize, Underline};

const MAX_CHUNKS_PER_TICK: usize = 24;
const MAX_BYTES_PER_TICK: usize = 128 * 1024;
const TERMINAL_ACTIVE_POLL_INTERVAL_MS: u64 = 8;
const TERMINAL_IDLE_POLL_INTERVAL_MS: u64 = 20;
const TERMINAL_HIDDEN_POLL_INTERVAL_MS: u64 = 420;
const TERMINAL_INSET_X: f64 = 12.0;
const TERMINAL_INSET_Y: f64 = 10.0;
const TERMINAL_FONT_DESC: &str = "DMMono Nerd Font 12";
const TERMINAL_STATUS_TITLE_PREFIX: &str = "kaku-status:";

#[link(name = "pangocairo-1.0")]
unsafe extern "C" {
    fn pango_cairo_show_layout(
        cr: *mut gtk::cairo::ffi::cairo_t,
        layout: *mut pango::ffi::PangoLayout,
    );
}

pub struct TerminalHandle {
    title: String,
    live: RefCell<Option<TerminalLiveHandle>>,
    error: RefCell<Option<String>>,
    pending_spawn: RefCell<Option<mpsc::Receiver<Result<SpawnedTerminalSession, String>>>>,
    queued_input: RefCell<Vec<String>>,
    strip_state: RefCell<TerminalStripState>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TerminalHandleState {
    pub live: bool,
    pub pending_spawn: bool,
    pub has_error: bool,
}

pub struct TerminalWidgetView {
    pub root: gtk::Box,
    pub drag_handle: gtk::Box,
    pub drop_target: gtk::Box,
    pub focus_target: gtk::DrawingArea,
    pub close_button: gtk::Button,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TerminalStripState {
    title: String,
    status_badge: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TerminalSurfaceState {
    pub title: String,
    pub status_badge: Option<String>,
}

struct TerminalLiveHandle {
    terminal: RefCell<Terminal>,
    session: TerminalSession,
    rx: RefCell<mpsc::Receiver<Vec<u8>>>,
    has_pending_output: Arc<AtomicBool>,
    backlog: RefCell<VecDeque<Vec<u8>>>,
}

struct SpawnedTerminalSession {
    session: TerminalSession,
    rx: mpsc::Receiver<Vec<u8>>,
    has_pending_output: Arc<AtomicBool>,
}

static ZSH_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static BASH_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static FISH_CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn spawn_terminal_handle(cwd: Option<String>) -> Rc<TerminalHandle> {
    Rc::new(spawn_terminal_handle_inner(cwd))
}

pub fn build_terminal_widget(handle: Rc<TerminalHandle>) -> TerminalWidgetView {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("terminal-host");
    root.set_hexpand(true);
    root.set_vexpand(true);

    let chrome = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    chrome.add_css_class("terminal-toolbar");
    chrome.add_css_class("pane-titlebar");
    chrome.set_hexpand(true);

    let path_label = gtk::Label::new(Some(&handle.title));
    path_label.add_css_class("terminal-path");
    path_label.add_css_class("pane-title");
    path_label.set_halign(gtk::Align::Start);
    path_label.set_hexpand(true);
    path_label.set_ellipsize(pango::EllipsizeMode::Middle);
    path_label.set_single_line_mode(true);
    path_label.set_margin_start(10);
    path_label.set_margin_end(10);
    path_label.set_margin_top(8);
    path_label.set_margin_bottom(8);

    chrome.append(&path_label);
    let status_badge = gtk::Label::new(None);
    status_badge.add_css_class("terminal-status-badge");
    status_badge.set_halign(gtk::Align::End);
    status_badge.set_valign(gtk::Align::Center);
    status_badge.set_margin_end(4);
    status_badge.set_visible(false);
    chrome.append(&status_badge);
    let close_button = gtk::Button::new();
    close_button.add_css_class("terminal-close-button");
    close_button.add_css_class("subtle-control-button");
    let close_icon = gtk::Image::from_file(format!(
        "{}/assets/icons/xmark.svg",
        env!("CARGO_MANIFEST_DIR")
    ));
    close_icon.set_pixel_size(11);
    close_button.set_child(Some(&close_icon));
    close_button.set_valign(gtk::Align::Center);
    close_button.set_margin_end(10);
    chrome.append(&close_button);
    root.append(&chrome);

    let view = gtk::DrawingArea::new();
    view.add_css_class("terminal-view");
    view.set_hexpand(true);
    view.set_vexpand(true);
    view.set_focusable(true);
    root.append(&view);

    let view_for_output = view.clone();
    let handle_for_output = Rc::clone(&handle);
    let initial_snapshot = {
        let live_ref = handle.live.borrow();
        live_ref.as_ref().and_then(|live| {
            live.terminal
                .try_borrow_mut()
                .ok()
                .map(|mut terminal| render_terminal_snapshot(&mut terminal))
        })
    };
    if let Some(snapshot) = initial_snapshot.as_ref() {
        handle.strip_state.replace(snapshot.strip_state.clone());
    }
    apply_terminal_strip_state(
        &path_label,
        &status_badge,
        initial_snapshot.as_ref().map(|snapshot| &snapshot.strip_state),
        &handle.title,
    );
    let snapshot_state = Rc::new(RefCell::new(initial_snapshot));
    let snapshot_state_for_draw = Rc::clone(&snapshot_state);
    let sync_size: Rc<dyn Fn()> = {
        let handle = Rc::clone(&handle);
        let view = view.clone();
        Rc::new(move || {
            let live_ref = handle.live.borrow();
            if let Some(live) = live_ref.as_ref() {
                sync_pty_size(&live.session, &live.terminal, &view);
            }
        })
    };
    view.set_draw_func(move |area, cr, width, height| {
        draw_terminal_surface(
            area,
            cr,
            width,
            height,
            snapshot_state_for_draw.borrow().as_ref(),
        );
    });

    {
        let sync_size = Rc::clone(&sync_size);
        view.connect_map(move |_| sync_size());
    }
    {
        let view_for_focus = view.clone();
        view.connect_map(move |_| {
            view_for_focus.grab_focus();
        });
    }
    {
        let sync_size = Rc::clone(&sync_size);
        view.connect_notify_local(Some("width"), move |_, _| sync_size());
    }
    {
        let sync_size = Rc::clone(&sync_size);
        view.connect_notify_local(Some("height"), move |_, _| sync_size());
    }

    schedule_terminal_pump(
        view_for_output,
        handle_for_output,
        snapshot_state,
        sync_size,
        path_label.clone(),
        status_badge.clone(),
        TERMINAL_ACTIVE_POLL_INTERVAL_MS,
    );

    let handle_for_keys = Rc::clone(&handle);
    let view_for_actions = view.clone();
    let key_controller = gtk::EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, state| {
        if state.contains(gdk::ModifierType::META_MASK)
            || state.contains(gdk::ModifierType::SUPER_MASK)
            || state.contains(gdk::ModifierType::CONTROL_MASK)
        {
            let shifted = state.contains(gdk::ModifierType::SHIFT_MASK);
            let action = match (key, shifted) {
                (gdk::Key::t, false) | (gdk::Key::T, false) => Some("win.launch-terminal"),
                (gdk::Key::bracketright, false) => Some("win.focus-next-terminal"),
                (gdk::Key::bracketleft, false) => Some("win.focus-previous-terminal"),
                (gdk::Key::w, false) | (gdk::Key::W, false) => Some("win.close-terminal"),
                (gdk::Key::b, false) | (gdk::Key::B, false) => Some("win.toggle-rail"),
                (gdk::Key::a, true) | (gdk::Key::A, true) => Some("win.toggle-activity"),
                (gdk::Key::g, true) | (gdk::Key::G, true) => Some("win.launch-lazygit"),
                (gdk::Key::y, true) | (gdk::Key::Y, true) => Some("win.launch-yazi"),
                (gdk::Key::d, true) | (gdk::Key::D, true) => Some("win.run-doctor"),
                (gdk::Key::comma, false) => Some("win.open-config"),
                (gdk::Key::t, true) | (gdk::Key::T, true) => Some("win.toggle-tasks"),
                (gdk::Key::m, true) | (gdk::Key::M, true) => Some("win.toggle-metadata"),
                (gdk::Key::s, true) | (gdk::Key::S, true) => Some("win.set-status"),
                (gdk::Key::x, true) | (gdk::Key::X, true) => Some("win.clear-status"),
                (gdk::Key::p, true) | (gdk::Key::P, true) => Some("win.set-progress"),
                (gdk::Key::l, true) | (gdk::Key::L, true) => Some("win.append-log"),
                (gdk::Key::r, true) | (gdk::Key::R, true) => Some("win.reset-shell"),
                _ => None,
            };
            if let Some(action) = action {
                return if view_for_actions.activate_action(action, None).is_ok() {
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                };
            }
        }
        if let Some(sequence) = key_to_bytes(key, state) {
            let live_ref = handle_for_keys.live.borrow();
            if let Some(live) = live_ref.as_ref() {
                let _ = live.session.write_all(&sequence);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    view.add_controller(key_controller);

    install_focus_click(&view, &view);
    install_focus_click(&chrome, &view);
    install_focus_click(&root, &view);
    view.grab_focus();

    TerminalWidgetView {
        root: root.clone(),
        drag_handle: chrome,
        drop_target: root,
        focus_target: view,
        close_button,
    }
}

fn spawn_terminal_handle_inner(cwd: Option<String>) -> TerminalHandle {
    let title = terminal_title(cwd.as_deref());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(spawn_terminal_session(cwd));
    });

    TerminalHandle {
        strip_state: RefCell::new(TerminalStripState {
            title: title.clone(),
            status_badge: None,
        }),
        title,
        live: RefCell::new(None),
        error: RefCell::new(None),
        pending_spawn: RefCell::new(Some(rx)),
        queued_input: RefCell::new(Vec::new()),
    }
}

fn spawn_terminal_session(cwd: Option<String>) -> Result<SpawnedTerminalSession, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 30,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|err| format!("Failed to open PTY: {err}"))?;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
    let mut cmd = CommandBuilder::new(&shell);
    let shell_support = configure_shell_command(&shell, &mut cmd)?;
    cmd.env("TERM", "xterm-256color");
    cmd.env("CLICOLOR", "1");
    cmd.env("CLICOLOR_FORCE", "1");
    cmd.env("LSCOLORS", "GxFxCxDxBxegedabagaced");
    cmd.env("STARSHIP_CONFIG", "/dev/null");
    cmd.env("STARSHIP_SHELL", "disabled");
    configure_shell_prompt_env(&shell, &mut cmd, shell_support.config_dir.as_deref());
    if let Some(dir) = cwd.and_then(normalize_cwd) {
        cmd.cwd(dir);
    }

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|err| format!("Failed to spawn shell: {err}"))?;
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|err| format!("Failed to clone PTY reader: {err}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|err| format!("Failed to acquire PTY writer: {err}"))?;
    let writer = Arc::new(Mutex::new(writer));
    let session = TerminalSession {
        writer,
        master: RefCell::new(pair.master),
        child: RefCell::new(Some(child)),
        last_size: Cell::new((0, 0)),
        last_widget_extent: Cell::new((0, 0)),
    };
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let has_pending_output = Arc::new(AtomicBool::new(false));
    let has_pending_output_for_reader = Arc::clone(&has_pending_output);
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut chunk = [0u8; 4096];
        loop {
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => {
                    has_pending_output_for_reader.store(true, Ordering::SeqCst);
                    if tx.send(chunk[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(SpawnedTerminalSession {
        session,
        rx,
        has_pending_output,
    })
}

struct TerminalSession {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: RefCell<Box<dyn MasterPty + Send>>,
    child: RefCell<Option<Box<dyn Child + Send>>>,
    last_size: Cell<(u16, u16)>,
    last_widget_extent: Cell<(i32, i32)>,
}

impl TerminalSession {
    fn write_all(&self, bytes: &[u8]) -> std::io::Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| std::io::Error::other("terminal writer poisoned"))?;
        writer.write_all(bytes)?;
        writer.flush()
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.borrow_mut().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl TerminalHandle {
    fn poll_spawn_completion(&self) -> bool {
        let result = {
            let pending = self.pending_spawn.borrow();
            let Some(rx) = pending.as_ref() else {
                return false;
            };
            match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => {
                    Some(Err("Failed to receive terminal startup result".to_string()))
                }
            }
        };

        let Some(result) = result else {
            return false;
        };
        self.pending_spawn.borrow_mut().take();
        match result {
            Ok(spawned) => {
                self.error.borrow_mut().take();
                let live =
                    make_live_handle(spawned.session, spawned.rx, spawned.has_pending_output);
                {
                    let mut queued = self.queued_input.borrow_mut();
                    for command in queued.drain(..) {
                        let _ = live.session.write_all(command.as_bytes());
                    }
                }
                self.live.borrow_mut().replace(live);
                self.strip_state.replace(TerminalStripState {
                    title: self.title.clone(),
                    status_badge: None,
                });
            }
            Err(err) => {
                self.error.borrow_mut().replace(err);
            }
        }
        true
    }

    pub fn state(&self) -> TerminalHandleState {
        TerminalHandleState {
            live: self.live.borrow().is_some(),
            pending_spawn: self.pending_spawn.borrow().is_some(),
            has_error: self.error.borrow().is_some(),
        }
    }

    pub fn send_text(&self, text: &str) -> bool {
        let live_ref = self.live.borrow();
        if let Some(live) = live_ref.as_ref() {
            return live.session.write_all(text.as_bytes()).is_ok();
        }
        drop(live_ref);
        if self.pending_spawn.borrow().is_some() {
            self.queued_input.borrow_mut().push(text.to_string());
            return true;
        }
        false
    }

    pub fn viewport_contains(&self, needle: &str) -> bool {
        let live_ref = self.live.borrow();
        let Some(live) = live_ref.as_ref() else {
            return false;
        };
        let Ok(mut terminal) = live.terminal.try_borrow_mut() else {
            return false;
        };
        render_terminal_snapshot(&mut terminal)
            .text
            .contains(needle)
    }

    fn needs_active_pump(&self) -> bool {
        if self.pending_spawn.borrow().is_some() {
            return true;
        }
        let live_ref = self.live.borrow();
        let Some(live) = live_ref.as_ref() else {
            return false;
        };
        if live.has_pending_output.load(Ordering::SeqCst) {
            return true;
        }
        live.backlog
            .try_borrow()
            .map(|backlog| !backlog.is_empty())
            .unwrap_or(false)
    }

    pub fn surface_state(&self) -> TerminalSurfaceState {
        let strip = self.strip_state.borrow().clone();
        TerminalSurfaceState {
            title: if strip.title.is_empty() {
                self.title.clone()
            } else {
                strip.title
            },
            status_badge: strip.status_badge,
        }
    }
}

fn schedule_terminal_pump(
    view: gtk::DrawingArea,
    handle: Rc<TerminalHandle>,
    snapshot_state: Rc<RefCell<Option<TerminalRenderSnapshot>>>,
    sync_size: Rc<dyn Fn()>,
    path_label: gtk::Label,
    status_badge: gtk::Label,
    delay_ms: u64,
) {
    glib::timeout_add_local_once(Duration::from_millis(delay_ms), move || {
        if view.root().is_none() {
            return;
        }

        let visible = view.is_mapped() && view.is_visible();
        if visible {
            sync_size();
        }
        let mut changed = handle.poll_spawn_completion();
        if changed && visible {
            sync_size();
        }
        if visible {
            {
                let live_ref = handle.live.borrow();
                if let Some(live) = live_ref.as_ref() {
                    if live.has_pending_output.swap(false, Ordering::SeqCst) {
                        if let (Ok(rx), Ok(mut terminal)) =
                            (live.rx.try_borrow_mut(), live.terminal.try_borrow_mut())
                        {
                            let mut chunks = 0usize;
                            let mut bytes_processed = 0usize;
                            loop {
                                let next_chunk = if let Ok(mut backlog) = live.backlog.try_borrow_mut()
                                {
                                    backlog.pop_front()
                                } else {
                                    None
                                }
                                .or_else(|| rx.try_recv().ok());

                                let Some(bytes) = next_chunk else {
                                    break;
                                };

                                if chunks >= MAX_CHUNKS_PER_TICK
                                    || bytes_processed + bytes.len() > MAX_BYTES_PER_TICK
                                {
                                    if let Ok(mut backlog) = live.backlog.try_borrow_mut() {
                                        backlog.push_front(bytes);
                                    }
                                    live.has_pending_output.store(true, Ordering::SeqCst);
                                    break;
                                }
                                bytes_processed += bytes.len();
                                chunks += 1;
                                terminal.advance_bytes(&bytes);
                                changed = true;
                            }
                            let backlog_pending = live
                                .backlog
                                .try_borrow()
                                .map(|backlog| !backlog.is_empty())
                                .unwrap_or(false);
                            if backlog_pending {
                                live.has_pending_output.store(true, Ordering::SeqCst);
                            } else if let Ok(bytes) = rx.try_recv() {
                                if let Ok(mut backlog) = live.backlog.try_borrow_mut() {
                                    backlog.push_back(bytes);
                                }
                                live.has_pending_output.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
            if changed {
                let live_ref = handle.live.borrow();
                if let Some(live) = live_ref.as_ref() {
                    if let Ok(mut terminal) = live.terminal.try_borrow_mut() {
                        let snapshot = render_terminal_snapshot(&mut terminal);
                        handle.strip_state.replace(snapshot.strip_state.clone());
                        apply_terminal_strip_state(
                            &path_label,
                            &status_badge,
                            Some(&snapshot.strip_state),
                            &handle.title,
                        );
                        snapshot_state.replace(Some(snapshot));
                        view.queue_draw();
                    }
                }
            }
        }

        let state = handle.state();
        if !state.pending_spawn && state.has_error {
            return;
        }
        let next_delay = if !visible {
            TERMINAL_HIDDEN_POLL_INTERVAL_MS
        } else if handle.needs_active_pump() {
            TERMINAL_ACTIVE_POLL_INTERVAL_MS
        } else {
            TERMINAL_IDLE_POLL_INTERVAL_MS
        };
        schedule_terminal_pump(
            view,
            handle,
            snapshot_state,
            sync_size,
            path_label,
            status_badge,
            next_delay,
        );
    });
}

fn make_live_handle(
    session: TerminalSession,
    rx: mpsc::Receiver<Vec<u8>>,
    has_pending_output: Arc<AtomicBool>,
) -> TerminalLiveHandle {
    let term_config = Arc::new(TermConfig::new());
    let mut palette = term_config.color_palette();
    apply_native_shell_palette(&mut palette);
    term_config.set_client_palette(palette);
    let writer = Arc::clone(&session.writer);
    let terminal = Terminal::new(
        terminal_size_for_widget(120, 30, 0, 0),
        term_config as Arc<dyn TerminalConfiguration + Send + Sync>,
        "shin-chan",
        config::wezterm_version(),
        Box::new(SharedPtyWriter::new(writer)),
    );
    TerminalLiveHandle {
        terminal: RefCell::new(terminal),
        session,
        rx: RefCell::new(rx),
        has_pending_output,
        backlog: RefCell::new(VecDeque::new()),
    }
}

fn key_to_bytes(key: gdk::Key, state: gdk::ModifierType) -> Option<Vec<u8>> {
    if state.contains(gdk::ModifierType::CONTROL_MASK) {
        return control_key_to_bytes(key);
    }

    let mut bytes = match key {
        gdk::Key::Return => Some(b"\r".to_vec()),
        gdk::Key::BackSpace => Some(vec![0x7f]),
        gdk::Key::Tab => Some(b"\t".to_vec()),
        gdk::Key::Escape => Some(vec![0x1b]),
        gdk::Key::Up => Some(b"\x1b[A".to_vec()),
        gdk::Key::Down => Some(b"\x1b[B".to_vec()),
        gdk::Key::Right => Some(b"\x1b[C".to_vec()),
        gdk::Key::Left => Some(b"\x1b[D".to_vec()),
        gdk::Key::Home => Some(b"\x1b[H".to_vec()),
        gdk::Key::End => Some(b"\x1b[F".to_vec()),
        gdk::Key::Page_Up => Some(b"\x1b[5~".to_vec()),
        gdk::Key::Page_Down => Some(b"\x1b[6~".to_vec()),
        gdk::Key::Delete => Some(b"\x1b[3~".to_vec()),
        _ => key.to_unicode().map(|ch| ch.to_string().into_bytes()),
    }?;

    if state.contains(gdk::ModifierType::ALT_MASK) {
        let mut prefixed = vec![0x1b];
        prefixed.append(&mut bytes);
        return Some(prefixed);
    }

    Some(bytes)
}

fn control_key_to_bytes(key: gdk::Key) -> Option<Vec<u8>> {
    match key {
        gdk::Key::BackSpace => Some(vec![0x7f]),
        gdk::Key::Return => Some(b"\n".to_vec()),
        gdk::Key::space => Some(vec![0]),
        gdk::Key::Up => Some(b"\x1b[1;5A".to_vec()),
        gdk::Key::Down => Some(b"\x1b[1;5B".to_vec()),
        gdk::Key::Right => Some(b"\x1b[1;5C".to_vec()),
        gdk::Key::Left => Some(b"\x1b[1;5D".to_vec()),
        _ => key.to_unicode().and_then(control_char_to_byte),
    }
}

fn control_char_to_byte(ch: char) -> Option<Vec<u8>> {
    match ch {
        '@' | '`' | ' ' => Some(vec![0]),
        'a'..='z' => Some(vec![(ch as u8) - b'a' + 1]),
        'A'..='Z' => Some(vec![(ch as u8) - b'A' + 1]),
        '[' => Some(vec![27]),
        '\\' => Some(vec![28]),
        ']' => Some(vec![29]),
        '^' => Some(vec![30]),
        '_' => Some(vec![31]),
        _ => None,
    }
}

fn install_focus_click(widget: &impl IsA<gtk::Widget>, view: &gtk::DrawingArea) {
    let click = gtk::GestureClick::new();
    let view_for_focus = view.clone();
    click.connect_pressed(move |_, _, _, _| {
        view_for_focus.grab_focus();
    });
    widget.add_controller(click);
}

struct ShellLaunchSupport {
    config_dir: Option<PathBuf>,
}

fn configure_shell_command(
    shell: &str,
    cmd: &mut CommandBuilder,
) -> Result<ShellLaunchSupport, String> {
    let shell_name = Path::new(shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    match shell_name {
        "zsh" => {
            let config_dir = prepare_minimal_shell_config(shell_name, &ZSH_CONFIG_DIR)?;
            cmd.env("ZDOTDIR", &config_dir);
            cmd.arg("-i");
            Ok(ShellLaunchSupport {
                config_dir: Some(config_dir),
            })
        }
        "bash" => {
            let config_dir = prepare_minimal_shell_config(shell_name, &BASH_CONFIG_DIR)?;
            let rcfile = config_dir.join(".bashrc");
            cmd.arg("--noprofile");
            cmd.arg("--rcfile");
            cmd.arg(rcfile);
            cmd.arg("-i");
            Ok(ShellLaunchSupport {
                config_dir: Some(config_dir),
            })
        }
        "fish" => {
            let config_dir = prepare_minimal_shell_config(shell_name, &FISH_CONFIG_DIR)?;
            cmd.env("XDG_CONFIG_HOME", &config_dir);
            cmd.arg("-i");
            Ok(ShellLaunchSupport {
                config_dir: Some(config_dir),
            })
        }
        _ => Ok(ShellLaunchSupport { config_dir: None }),
    }
}

fn configure_shell_prompt_env(shell: &str, cmd: &mut CommandBuilder, config_dir: Option<&Path>) {
    let shell_name = Path::new(shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    match shell_name {
        "zsh" => {
            if config_dir.is_none() {
                cmd.env("PROMPT", "%F{81}%2~%f %# ");
            }
            cmd.env("RPROMPT", "");
            cmd.env_remove("PS1");
        }
        "bash" | "sh" => {
            if config_dir.is_none() {
                cmd.env("PS1", "\\[\\e[38;5;81m\\]\\w\\[\\e[0m\\] \\$ ");
            }
            cmd.env_remove("PROMPT");
            cmd.env_remove("RPROMPT");
        }
        _ => {
            cmd.env("PROMPT", "%2~ %# ");
            cmd.env("RPROMPT", "");
        }
    }
}

fn prepare_minimal_shell_config(
    shell_name: &str,
    cache: &OnceLock<PathBuf>,
) -> Result<PathBuf, String> {
    let config_dir = cache
        .get()
        .cloned()
        .unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "kaku-native-shell-shared-{}-{}",
                shell_name,
                std::process::id()
            ))
        });
    fs::create_dir_all(&config_dir)
        .map_err(|err| format!("Failed to create shell config dir: {err}"))?;

    match shell_name {
        "zsh" => {
            fs::write(
                config_dir.join(".zshrc"),
                "unsetopt beep\nsetopt interactive_comments\nPROMPT_EOL_MARK=''\nexport CLICOLOR=1\nexport CLICOLOR_FORCE=1\nexport LSCOLORS='GxFxCxDxBxegedabagaced'\nexport LESS='-R'\nalias ll='ls -lah'\nalias la='ls -A'\nalias l='ls -CF'\nalias gs='git status --short --branch'\nalias gd='git diff --minimal'\nalias gl='git log --oneline --decorate -12'\nalias gco='git checkout'\nalias vim='nvim'\nalias vi='nvim'\nfunction yy() {\n  if command -v yazi >/dev/null 2>&1; then\n    yazi \"$@\"\n  else\n    print -P '%F{203}yazi is not installed%f'\n    return 127\n  fi\n}\nfunction lg() {\n  if command -v lazygit >/dev/null 2>&1; then\n    lazygit \"$@\"\n  else\n    print -P '%F{203}lazygit is not installed%f'\n    return 127\n  fi\n}\nfunction _kaku_precmd() {\n  local exit_code=$?\n  printf '\\033]2;kaku-status:%s:%s\\007' \"$exit_code\" \"$PWD\"\n}\nfunction _kaku_chpwd() {\n  printf '\\033]2;kaku-status:0:%s\\007' \"$PWD\"\n}\nfunction command_not_found_handler() {\n  print -P \"%F{203}command not found:%f %F{210}${1}%f\"\n  return 127\n}\nautoload -Uz add-zsh-hook\nadd-zsh-hook precmd _kaku_precmd\nadd-zsh-hook chpwd _kaku_chpwd\nPROMPT='%F{81}%2~%f $ '\nRPROMPT=''\n",
            )
            .map_err(|err| format!("Failed to write .zshrc: {err}"))?;
        }
        "bash" => {
            fs::write(
                config_dir.join(".bashrc"),
                "export CLICOLOR=1\nexport CLICOLOR_FORCE=1\nexport LSCOLORS='GxFxCxDxBxegedabagaced'\nexport LESS='-R'\nshopt -s checkwinsize\nalias ll='ls -lah'\nalias la='ls -A'\nalias l='ls -CF'\nalias gs='git status --short --branch'\nalias gd='git diff --minimal'\nalias gl='git log --oneline --decorate -12'\nalias gco='git checkout'\nalias vim='nvim'\nalias vi='nvim'\nyy() {\n  if command -v yazi >/dev/null 2>&1; then\n    yazi \"$@\"\n  else\n    printf '\\e[38;5;203myazi is not installed\\e[0m\\n'\n    return 127\n  fi\n}\nlg() {\n  if command -v lazygit >/dev/null 2>&1; then\n    lazygit \"$@\"\n  else\n    printf '\\e[38;5;203mlazygit is not installed\\e[0m\\n'\n    return 127\n  fi\n}\nkaku_prompt_command() {\n  local exit_code=$?\n  printf '\\033]2;kaku-status:%s:%s\\007' \"$exit_code\" \"$PWD\"\n}\ncommand_not_found_handle() {\n  printf '\\e[38;5;203mcommand not found:\\e[0m \\e[38;5;210m%s\\e[0m\\n' \"$1\"\n  return 127\n}\nPROMPT_COMMAND=kaku_prompt_command\nPS1='\\[\\e[38;5;81m\\]\\w\\[\\e[0m\\] \\$ '\n",
            )
                .map_err(|err| format!("Failed to write .bashrc: {err}"))?;
        }
        "fish" => {
            let fish_dir = config_dir.join("fish");
            fs::create_dir_all(&fish_dir)
                .map_err(|err| format!("Failed to create fish config dir: {err}"))?;
            fs::write(
                fish_dir.join("config.fish"),
                "set -gx CLICOLOR 1\nset -gx CLICOLOR_FORCE 1\nset -gx LESS '-R'\nalias ll 'ls -lah'\nalias la 'ls -A'\nalias l 'ls -CF'\nalias gs 'git status --short --branch'\nalias gd 'git diff --minimal'\nalias gl 'git log --oneline --decorate -12'\nalias gco 'git checkout'\nalias vim 'nvim'\nalias vi 'nvim'\nfunction yy\n  if command -q yazi\n    yazi $argv\n  else\n    set_color 203\n    printf 'yazi is not installed\\n'\n    set_color normal\n    return 127\n  end\nend\nfunction lg\n  if command -q lazygit\n    lazygit $argv\n  else\n    set_color 203\n    printf 'lazygit is not installed\\n'\n    set_color normal\n    return 127\n  end\nend\nfunction fish_prompt\n  set -l exit_code $status\n  printf '\\e]2;kaku-status:%s:%s\\a' $exit_code $PWD\n  set_color 81\n  printf '%s' (prompt_pwd)\n  set_color normal\n  printf ' > '\nend\nfunction fish_command_not_found\n  set_color 203\n  printf 'command not found:'\n  set_color 210\n  printf ' %s\\n' $argv[1]\n  set_color normal\nend\n",
            )
            .map_err(|err| format!("Failed to write fish config: {err}"))?;
        }
        _ => {}
    }

    let _ = cache.set(config_dir.clone());
    Ok(config_dir)
}

fn normalize_cwd(cwd: String) -> Option<PathBuf> {
    let without_scheme = cwd.strip_prefix("file://").unwrap_or(&cwd);
    if without_scheme.is_empty() {
        None
    } else {
        Some(PathBuf::from(without_scheme))
    }
}

fn terminal_title(cwd: Option<&str>) -> String {
    friendly_display_path(cwd.unwrap_or("workspace shell"))
}

fn friendly_display_path(path: &str) -> String {
    let without_scheme = path.strip_prefix("file://").unwrap_or(path);
    let candidate = PathBuf::from(without_scheme);
    let home = std::env::var("HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(dirs_next::home_dir);
    if let Some(home) = home {
        if candidate == home {
            return "~".to_string();
        }
        if let Ok(stripped) = candidate.strip_prefix(&home) {
            let stripped = stripped.to_string_lossy();
            if stripped.is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", stripped.trim_start_matches('/'));
        }
    }
    without_scheme.to_string()
}

fn sync_pty_size(
    session: &TerminalSession,
    terminal: &RefCell<Terminal>,
    widget: &impl IsA<gtk::Widget>,
) {
    let width = widget.width().max(1);
    let height = widget.height().max(1);
    if width <= 1 || height <= 1 {
        return;
    }
    let widget_extent = (width, height);
    if session.last_widget_extent.get() == widget_extent {
        return;
    }
    session.last_widget_extent.set(widget_extent);

    let (cell_width, cell_height) = measure_cell_size(widget);
    let horizontal_padding = (TERMINAL_INSET_X * 2.0) as i32;
    let vertical_padding = (TERMINAL_INSET_Y * 2.0) as i32;
    let cols = (((width - horizontal_padding).max(40) as f64) / cell_width)
        .floor()
        .max(20.0) as u16;
    let rows = (((height - vertical_padding).max(36) as f64) / cell_height)
        .floor()
        .max(6.0) as u16;
    let next = (cols, rows);
    if session.last_size.get() == next {
        return;
    }

    if session
        .master
        .borrow_mut()
        .resize(PtySize {
            rows,
            cols,
            pixel_width: width as u16,
            pixel_height: height as u16,
        })
        .is_ok()
    {
        terminal
            .borrow_mut()
            .resize(terminal_size_for_widget(cols, rows, width, height));
        session.last_size.set(next);
    }
}

fn measure_cell_size(widget: &impl IsA<gtk::Widget>) -> (f64, f64) {
    let context = widget.pango_context();
    let font_desc = terminal_font_description();
    let metrics = context.metrics(Some(&font_desc), None);
    let cell_width = (metrics.approximate_digit_width() as f64 / pango::SCALE as f64).max(8.0);
    let cell_height =
        ((metrics.ascent() + metrics.descent()) as f64 / pango::SCALE as f64).max(1.0);
    (cell_width, cell_height)
}

fn terminal_size_for_widget(cols: u16, rows: u16, width: i32, height: i32) -> TerminalSize {
    TerminalSize {
        rows: rows as usize,
        cols: cols as usize,
        pixel_width: width.max(0) as usize,
        pixel_height: height.max(0) as usize,
        dpi: 0,
    }
}

struct SharedPtyWriter {
    inner: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl SharedPtyWriter {
    fn new(inner: Arc<Mutex<Box<dyn Write + Send>>>) -> Self {
        Self { inner }
    }
}

impl Write for SharedPtyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("terminal writer poisoned"))?;
        inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::other("terminal writer poisoned"))?;
        inner.flush()
    }
}

struct TerminalRenderSnapshot {
    #[allow(dead_code)]
    text: String,
    #[allow(dead_code)]
    cursor_offset: Option<i32>,
    dominant_bg: [u8; 4],
    lines: Vec<TerminalRenderLine>,
    strip_state: TerminalStripState,
}

#[derive(Clone)]
struct TerminalRenderLine {
    cells: Vec<TerminalRenderCell>,
}

#[derive(Clone)]
struct TerminalRenderCell {
    text: String,
    width: usize,
    style: TerminalTextStyle,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct TerminalTextStyle {
    fg: [u8; 4],
    bg: [u8; 4],
    has_background: bool,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

#[derive(Clone, Copy)]
struct CursorPlacement {
    col: usize,
    fg: [u8; 4],
    bg: [u8; 4],
}

fn render_terminal_snapshot(terminal: &mut Terminal) -> TerminalRenderSnapshot {
    let dims = terminal_get_dimensions(terminal);
    let top = dims.physical_top;
    let bottom = top + dims.viewport_rows as isize;
    let (_, lines) = terminal_get_lines(terminal, top..bottom);
    let cursor = terminal_get_cursor_position(terminal);
    let palette = terminal.palette();
    let strip_state = terminal_strip_state_from_title(terminal.get_title()).unwrap_or_default();

    let mut styled_lines = Vec::with_capacity(dims.viewport_rows);
    let mut rendered_lines = Vec::with_capacity(dims.viewport_rows);
    let mut bg_counts: HashMap<[u8; 4], usize> = HashMap::new();
    let mut cursor_offset = None;
    let mut offset = 0i32;

    for row in 0..dims.viewport_rows {
        let stable_row = top + row as isize;
        let cursor = if cursor.y == stable_row {
            Some(CursorPlacement {
                col: cursor.x,
                fg: rgba_bytes(palette.cursor_fg),
                bg: rgba_bytes(palette.cursor_bg),
            })
        } else {
            None
        };
        let (text, mut cells) = if let Some(line) = lines.get(row) {
            render_terminal_line(line, &palette, dims.cols, dims.reverse_video, cursor)
        } else {
            render_empty_terminal_line(&palette, dims.cols, cursor)
        };

        if text.to_ascii_lowercase().contains("command not found") {
            accent_command_not_found_line(&mut cells);
        }

        if let Some(cursor) = cursor {
            cursor_offset = Some(offset + cursor.col as i32);
        }

        styled_lines.push(TerminalRenderLine { cells });
        if let Some(line) = styled_lines.last() {
            for cell in &line.cells {
                *bg_counts.entry(cell.style.bg).or_insert(0) += cell.width;
            }
        }
        offset += text.chars().count() as i32;
        rendered_lines.push(text);

        if row + 1 < dims.viewport_rows {
            offset += 1;
        }
    }

    let dominant_bg = bg_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(bg, _)| bg)
        .unwrap_or(rgba_bytes(palette.background));

    TerminalRenderSnapshot {
        text: rendered_lines.join("\n"),
        cursor_offset,
        dominant_bg,
        lines: styled_lines,
        strip_state,
    }
}

fn accent_command_not_found_line(cells: &mut [TerminalRenderCell]) {
    let accent = [0xff, 0x84, 0x7a, 0xff];
    for cell in cells.iter_mut() {
        if cell.text.trim().is_empty() {
            continue;
        }
        cell.style.fg = accent;
    }
}

fn apply_terminal_strip_state(
    path_label: &gtk::Label,
    status_badge: &gtk::Label,
    strip_state: Option<&TerminalStripState>,
    fallback_title: &str,
) {
    let title = strip_state
        .map(|state| state.title.as_str())
        .filter(|title| !title.is_empty())
        .unwrap_or(fallback_title);
    path_label.set_label(title);
    if let Some(badge) = strip_state.and_then(|state| state.status_badge.as_deref()) {
        status_badge.set_label(badge);
        status_badge.set_visible(true);
    } else {
        status_badge.set_visible(false);
        status_badge.set_label("");
    }
}

fn terminal_strip_state_from_title(raw_title: &str) -> Option<TerminalStripState> {
    let payload = raw_title.strip_prefix(TERMINAL_STATUS_TITLE_PREFIX)?;
    let mut parts = payload.splitn(3, ':');
    let exit_code = parts.next()?.parse::<i32>().ok()?;
    let cwd = parts.next().unwrap_or_default();
    let title = if cwd.is_empty() {
        String::new()
    } else {
        friendly_display_path(cwd)
    };
    let status_badge = match exit_code {
        0 => None,
        127 => Some("NOT FOUND".to_string()),
        code => Some(format!("EXIT {code}")),
    };
    Some(TerminalStripState { title, status_badge })
}

fn draw_terminal_surface(
    area: &gtk::DrawingArea,
    cr: &gtk::cairo::Context,
    width: i32,
    height: i32,
    snapshot: Option<&TerminalRenderSnapshot>,
) {
    let Some(snapshot) = snapshot else {
        cr.set_source_rgb(0.05, 0.06, 0.08);
        let _ = cr.paint();
        return;
    };

    let bg = rgba_from_bytes(snapshot.dominant_bg);
    cr.set_source_rgba(
        bg.red() as f64,
        bg.green() as f64,
        bg.blue() as f64,
        bg.alpha() as f64,
    );
    let _ = cr.paint();

    let (cell_width, cell_height) = measure_cell_size(area);
    cr.set_antialias(gtk::cairo::Antialias::Best);
    let layout = area.create_pango_layout(None);
    let font_desc = terminal_font_description();
    layout.set_font_description(Some(&font_desc));
    layout.set_single_paragraph_mode(true);
    let mut attrs_cache: HashMap<TerminalTextStyle, pango::AttrList> = HashMap::new();

    let mut y = TERMINAL_INSET_Y;
    for line in &snapshot.lines {
        let mut x = TERMINAL_INSET_X;
        let mut idx = 0usize;
        while idx < line.cells.len() {
            let start_x = x;
            let style = line.cells[idx].style.clone();
            let mut run_text = String::new();
            let mut run_cols = 0usize;
            while idx < line.cells.len() && line.cells[idx].style == style {
                run_text.push_str(&line.cells[idx].text);
                run_cols += line.cells[idx].width;
                idx += 1;
            }

            let slot_width = cell_width * run_cols as f64;
            if style.has_background {
                let bg = rgba_from_bytes(style.bg);
                cr.set_source_rgba(
                    bg.red() as f64,
                    bg.green() as f64,
                    bg.blue() as f64,
                    bg.alpha() as f64,
                );
                cr.rectangle(start_x, y, slot_width, cell_height);
                let _ = cr.fill();
            }

            let fg = rgba_from_bytes(style.fg);
            cr.set_source_rgba(
                fg.red() as f64,
                fg.green() as f64,
                fg.blue() as f64,
                fg.alpha() as f64,
            );
            let attrs = attrs_cache
                .entry(style.clone())
                .or_insert_with(|| terminal_pango_attrs(&style))
                .clone();
            layout.set_attributes(Some(&attrs));
            layout.set_text(&run_text);
            let (_text_width, text_height) = layout.pixel_size();
            let baseline_y = y + ((cell_height - text_height.max(1) as f64) * 0.5).max(0.0);
            cr.move_to(start_x, baseline_y);
            unsafe {
                pango_cairo_show_layout(cr.to_raw_none(), layout.to_glib_none().0);
            }

            x += slot_width;
            if x > (width as f64 - TERMINAL_INSET_X) {
                break;
            }
        }

        y += cell_height;
        if y > (height as f64 - TERMINAL_INSET_Y) {
            break;
        }
    }
}

fn terminal_font_description() -> pango::FontDescription {
    pango::FontDescription::from_string(TERMINAL_FONT_DESC)
}

fn apply_native_shell_palette(palette: &mut ColorPalette) {
    palette.background = (0x3e, 0x41, 0x49).into();
    palette.foreground = (0xea, 0xec, 0xf4).into();
    palette.cursor_bg = (0x6f, 0xd6, 0xa7).into();
    palette.cursor_fg = (0x10, 0x11, 0x16).into();
}

fn terminal_pango_attrs(style: &TerminalTextStyle) -> pango::AttrList {
    let attrs = pango::AttrList::new();
    attrs.insert(pango::AttrInt::new_fallback(true));
    if style.bold {
        attrs.insert(pango::AttrInt::new_weight(pango::Weight::Bold));
    }
    if style.italic {
        attrs.insert(pango::AttrInt::new_style(pango::Style::Italic));
    }
    if style.underline {
        attrs.insert(pango::AttrInt::new_underline(pango::Underline::Single));
    }
    if style.strikethrough {
        attrs.insert(pango::AttrInt::new_strikethrough(true));
    }
    attrs
}

fn rgba_from_bytes(rgba: [u8; 4]) -> gdk::RGBA {
    gdk::RGBA::new(
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
        rgba[3] as f32 / 255.0,
    )
}

fn render_terminal_line(
    line: &wezterm_term::Line,
    palette: &wezterm_term::color::ColorPalette,
    cols: usize,
    reverse_video: bool,
    cursor: Option<CursorPlacement>,
) -> (String, Vec<TerminalRenderCell>) {
    let mut text = String::new();
    let mut cells = Vec::new();
    let mut cell_col = 0usize;

    for cell in line.visible_cells() {
        let base_style = style_for_cell(cell.attrs(), palette, reverse_video);
        let width = cell.width().max(1);
        let style = if let Some(cursor) = cursor.filter(|cursor| {
            cursor.col >= cell_col && cursor.col < cell_col + width
        }) {
            cursor_style_for_cell(&base_style, cursor)
        } else {
            base_style
        };
        let grapheme = if cell.attrs().invisible() {
            " ".repeat(width)
        } else {
            cell.str().to_string()
        };
        text.push_str(&grapheme);
        cells.push(TerminalRenderCell {
            text: grapheme,
            width,
            style,
        });
        cell_col += width;
    }

    if cell_col < cols {
        let trailing_cols = cols - cell_col;
        let default_style = default_terminal_style(palette);
        if let Some(cursor) = cursor {
            if cursor.col >= cell_col && cursor.col < cols {
                let padding = cursor.col.saturating_sub(cell_col);
                if padding > 0 {
                    let pad_text = " ".repeat(padding);
                    text.push_str(&pad_text);
                    cells.push(TerminalRenderCell {
                        text: pad_text,
                        width: padding,
                        style: default_style.clone(),
                    });
                }

                text.push(' ');
                cells.push(TerminalRenderCell {
                    text: " ".to_string(),
                    width: 1,
                    style: cursor_style_for_cell(&default_style, cursor),
                });

                let remaining = cols.saturating_sub(cursor.col + 1);
                if remaining > 0 {
                    let trailing = " ".repeat(remaining);
                    text.push_str(&trailing);
                    cells.push(TerminalRenderCell {
                        text: trailing,
                        width: remaining,
                        style: default_style,
                    });
                }
            } else {
                let trailing = " ".repeat(trailing_cols);
                text.push_str(&trailing);
                cells.push(TerminalRenderCell {
                    text: trailing,
                    width: trailing_cols,
                    style: default_style,
                });
            }
        } else {
            let trailing = " ".repeat(trailing_cols);
            text.push_str(&trailing);
            cells.push(TerminalRenderCell {
                text: trailing,
                width: trailing_cols,
                style: default_style,
            });
        }
    }

    (text, cells)
}

fn render_empty_terminal_line(
    palette: &wezterm_term::color::ColorPalette,
    cols: usize,
    cursor: Option<CursorPlacement>,
) -> (String, Vec<TerminalRenderCell>) {
    let default_style = default_terminal_style(palette);
    let total_cols = cols.max(1);
    match cursor {
        Some(cursor) if cursor.col < total_cols => {
            let mut cells = Vec::new();
            let mut text = String::new();
            if cursor.col > 0 {
                let left = " ".repeat(cursor.col);
                text.push_str(&left);
                cells.push(TerminalRenderCell {
                    text: left,
                    width: cursor.col,
                    style: default_style.clone(),
                });
            }
            text.push(' ');
            cells.push(TerminalRenderCell {
                text: " ".to_string(),
                width: 1,
                style: cursor_style_for_cell(&default_style, cursor),
            });
            let remaining = total_cols.saturating_sub(cursor.col + 1);
            if remaining > 0 {
                let right = " ".repeat(remaining);
                text.push_str(&right);
                cells.push(TerminalRenderCell {
                    text: right,
                    width: remaining,
                    style: default_style,
                });
            }
            (text, cells)
        }
        _ => (
            " ".repeat(total_cols),
            vec![TerminalRenderCell {
                text: " ".repeat(total_cols),
                width: total_cols,
                style: default_style,
            }],
        ),
    }
}

fn style_for_cell(
    attrs: &wezterm_term::CellAttributes,
    palette: &wezterm_term::color::ColorPalette,
    reverse_video: bool,
) -> TerminalTextStyle {
    let mut foreground = palette.resolve_fg(brightened_foreground(attrs));
    let mut background = palette.resolve_bg(attrs.background());

    if attrs.reverse() ^ reverse_video {
        std::mem::swap(&mut foreground, &mut background);
    }

    TerminalTextStyle {
        fg: rgba_bytes(foreground),
        bg: rgba_bytes(background),
        has_background: rgba_bytes(background) != rgba_bytes(palette.background),
        bold: attrs.intensity() == Intensity::Bold,
        italic: attrs.italic(),
        underline: attrs.underline() != Underline::None,
        strikethrough: attrs.strikethrough(),
    }
}

fn default_terminal_style(palette: &wezterm_term::color::ColorPalette) -> TerminalTextStyle {
    TerminalTextStyle {
        fg: rgba_bytes(palette.foreground),
        bg: rgba_bytes(palette.background),
        has_background: false,
        bold: false,
        italic: false,
        underline: false,
        strikethrough: false,
    }
}

fn cursor_style_for_cell(style: &TerminalTextStyle, cursor: CursorPlacement) -> TerminalTextStyle {
    let mut cursor_style = style.clone();
    cursor_style.fg = cursor.fg;
    cursor_style.bg = cursor.bg;
    cursor_style.has_background = true;
    cursor_style
}

fn brightened_foreground(attrs: &wezterm_term::CellAttributes) -> ColorAttribute {
    match attrs.foreground() {
        ColorAttribute::PaletteIndex(idx) if idx < 8 && attrs.intensity() == Intensity::Bold => {
            ColorAttribute::PaletteIndex(idx + 8)
        }
        color => color,
    }
}

fn rgba_bytes(color: SrgbaTuple) -> [u8; 4] {
    [
        (color.0.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.1.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.2.clamp(0.0, 1.0) * 255.0).round() as u8,
        (color.3.clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::{render_terminal_snapshot, terminal_strip_state_from_title, TerminalHandle};
    use config::TermConfig;
    use std::cell::RefCell;
    use std::sync::mpsc;
    use std::sync::Arc;
    use wezterm_term::{Terminal, TerminalConfiguration, TerminalSize};

    fn make_terminal(rows: usize, cols: usize) -> Terminal {
        Terminal::new(
            TerminalSize {
                rows,
                cols,
                pixel_width: cols * 8,
                pixel_height: rows * 16,
                dpi: 0,
            },
            Arc::new(TermConfig::new()) as Arc<dyn TerminalConfiguration + Send + Sync>,
            "shin-chan",
            config::wezterm_version(),
            Box::new(Vec::new()),
        )
    }

    #[test]
    fn terminal_snapshot_uses_real_terminal_state_for_basic_edits() {
        let mut terminal = make_terminal(4, 16);
        terminal.advance_bytes(b"hello");
        terminal.advance_bytes(b"\x08a");

        let snapshot = render_terminal_snapshot(&mut terminal);
        assert!(
            snapshot
                .text
                .lines()
                .next()
                .unwrap_or_default()
                .starts_with("hella"),
            "expected rendered line to start with edited text, got {:?}",
            snapshot.text.lines().next().unwrap_or_default()
        );
        assert_eq!(snapshot.cursor_offset, Some(5));
    }

    #[test]
    fn terminal_snapshot_tracks_cursor_after_csi_movement() {
        let mut terminal = make_terminal(4, 16);
        terminal.advance_bytes(b"alpha\nbeta");
        terminal.advance_bytes(b"\x1b[1A\x1b[3C!");
        terminal.advance_bytes(b"\x1b[2Ktop");

        let snapshot = render_terminal_snapshot(&mut terminal);
        let lines: Vec<_> = snapshot.text.lines().collect();
        assert!(
            lines
                .first()
                .copied()
                .unwrap_or_default()
                .trim_end()
                .ends_with("top"),
            "expected rendered line to preserve the real cursor column before `top`, got {:?}",
            lines.first().copied().unwrap_or_default()
        );
        assert!(
            lines
                .get(1)
                .copied()
                .unwrap_or_default()
                .trim_end()
                .ends_with("beta"),
            "expected rendered line to preserve the real cursor column before `beta`, got {:?}",
            lines.get(1).copied().unwrap_or_default()
        );
        assert!(snapshot.cursor_offset.is_some());
    }

    #[test]
    fn pending_terminal_send_text_is_queued_until_spawn_completes() {
        let (_tx, rx) = mpsc::channel();
        let handle = TerminalHandle {
            title: "test".to_string(),
            live: RefCell::new(None),
            error: RefCell::new(None),
            pending_spawn: RefCell::new(Some(rx)),
            queued_input: RefCell::new(Vec::new()),
            strip_state: RefCell::new(Default::default()),
        };

        assert!(handle.send_text("printf 'HELLO'\n"));
        assert_eq!(
            handle.queued_input.borrow().as_slice(),
            &["printf 'HELLO'\n".to_string()]
        );
    }

    #[test]
    fn parses_terminal_title_payload_for_missing_command() {
        let parsed = terminal_strip_state_from_title("kaku-status:127:/Users/henry/Documents/code")
            .expect("title payload should parse");
        assert_eq!(parsed.title, "~/Documents/code");
        assert_eq!(parsed.status_badge.as_deref(), Some("NOT FOUND"));
    }

    #[test]
    fn parses_terminal_title_payload_for_generic_failure() {
        let parsed =
            terminal_strip_state_from_title("kaku-status:2:/tmp/work").expect("payload should parse");
        assert_eq!(parsed.title, "/tmp/work");
        assert_eq!(parsed.status_badge.as_deref(), Some("EXIT 2"));
    }
}
