use crate::actions::{ShellAction, ShellActionContext, ShellActionOutcome};
use crate::runtime_bridge::{
    bootstrap_native_shell_runtime, NativeShellBootstrapPlan, NativeShellBootstrapResult,
};
use crate::snapshot::{
    derive_runtime_snapshot, refresh_scope_for_notification, RuntimeSnapshot, ShellLayoutContract,
    SnapshotRefreshScope, WorkspaceSummary,
};
use crate::terminal::{spawn_terminal_handle, TerminalHandle};
use crate::view::workspace::TwoPaneSplit;
use crate::view::{bind_header_swap, build_shell, PaneArrangement, ShellView};
use adw::prelude::*;
use chrono::{DateTime, TimeZone, Utc};
use gio::prelude::ActionMapExt;
use gio::SimpleAction;
use gio::SimpleActionGroup;
use glib::value::ToValue;
use gtk::gdk;
use gtk::gdk::prelude::ToplevelExt;
use gtk::prelude::{EventControllerExt, GestureSingleExt, NativeExt, WidgetExt};
use gtk::GestureClick;
use mux::notification_store::{NotificationRecord, NotificationUnreadMode};
use mux::Mux;
use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellTypographyContract {
    pub primary_mono_family: [&'static str; 3],
    pub operator_classes: [&'static str; 3],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellAffordanceContract {
    pub compact_count_labels: [&'static str; 3],
    pub header_badges: [&'static str; 3],
    pub action_labels: [&'static str; 8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellUiContract {
    pub layout_slots: [&'static str; 6],
    pub primary_surface: &'static str,
    pub persistent_context_slots: [&'static str; 3],
    pub typography: ShellTypographyContract,
    pub affordances: ShellAffordanceContract,
}

pub fn shell_ui_contract() -> ShellUiContract {
    ShellUiContract {
        layout_slots: [
            "chrome",
            "rail",
            "workspace",
            "activity",
            "inbox",
            "metadata",
        ],
        primary_surface: "workspace",
        persistent_context_slots: ["inbox", "tasks", "metadata"],
        typography: ShellTypographyContract {
            primary_mono_family: ["DMMono Nerd Font", "1984대화나눔_본문체_Regular", "monospace"],
            operator_classes: ["chrome-title", "rail-name", "pane-title"],
        },
        affordances: ShellAffordanceContract {
            compact_count_labels: ["1U 2R 0F", "70% 5L", "U1 T2"],
            header_badges: ["◉ unity-main", "3U", "2T"],
            action_labels: [
                "↻", "shell", "next", "prev", "close", "status", "progress", "reset",
            ],
        },
    }
}

pub struct AppController {
    pub window: adw::ApplicationWindow,
    content_host: gtk::Box,
    window_actions: SimpleActionGroup,
    selected_workspace: RefCell<Option<String>>,
    rail_collapsed: Cell<bool>,
    inbox_collapsed: Cell<bool>,
    metadata_collapsed: Cell<bool>,
    metadata_first: Cell<bool>,
    tasks_first: Cell<bool>,
    show_activity: Cell<bool>,
    show_tasks: Cell<bool>,
    show_metadata: Cell<bool>,
    show_terminal_sessions: Cell<bool>,
    two_pane_split: Cell<TwoPaneSplit>,
    zoomed_terminal_index: Cell<Option<usize>>,
    terminal_sessions: RefCell<Vec<Rc<TerminalHandle>>>,
    snapshot_cache: RefCell<RuntimeSnapshot>,
    snapshot_dirty: Cell<bool>,
    preferred_terminal_focus: Cell<Option<usize>>,
    focused_terminal_index: Cell<Option<usize>>,
    rerender_scheduled: Cell<bool>,
    smoke_text_injection_ok: Cell<bool>,
    smoke_text_rendered_ok: Cell<bool>,
    smoke_reset_ok: Cell<bool>,
    smoke_focus_cycle_ok: Cell<bool>,
    smoke_close_ok: Cell<bool>,
    smoke_peak_terminal_count: Cell<usize>,
    pending_refresh_scopes: Arc<Mutex<Vec<SnapshotRefreshScope>>>,
    refresh_flush_scheduled: Cell<bool>,
    pending_bootstrap: RefCell<Option<mpsc::Receiver<Result<NativeShellBootstrapResult, String>>>>,
    runtime_error: RefCell<Option<String>>,
    runtime_online: Cell<bool>,
    mounted_shell_root: RefCell<Option<gtk::Widget>>,
    live_terminal_targets: RefCell<Vec<gtk::DrawingArea>>,
    last_local_shell_signature: RefCell<String>,
    pending_startup_command: RefCell<Option<String>>,
}

#[derive(Clone, Debug)]
pub struct SmokeHarnessReport {
    pub startup_online: bool,
    pub terminal_count: usize,
    pub live_terminals: usize,
    pub pending_terminals: usize,
    pub errored_terminals: usize,
    pub peak_terminal_count: usize,
    pub text_injection_ok: bool,
    pub text_rendered_ok: bool,
    pub focus_cycle_ok: bool,
    pub close_ok: bool,
    pub reset_ok: bool,
    pub final_step: &'static str,
}

impl AppController {
    pub fn new(app: &adw::Application) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("shin-chan")
            .default_width(1180)
            .default_height(820)
            .build();
        let content_host = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content_host.set_hexpand(true);
        content_host.set_vexpand(true);
        content_host.set_size_request(0, 0);
        window.set_content(Some(&content_host));
        window.set_resizable(true);
        window.add_css_class("native-shell-window");
        let window_actions = SimpleActionGroup::new();
        window.insert_action_group("win", Some(&window_actions));

        let controller = Rc::new(Self {
            window,
            content_host,
            window_actions,
            selected_workspace: RefCell::new(None),
            rail_collapsed: Cell::new(false),
            inbox_collapsed: Cell::new(false),
            metadata_collapsed: Cell::new(false),
            metadata_first: Cell::new(false),
            tasks_first: Cell::new(false),
            show_activity: Cell::new(false),
            show_tasks: Cell::new(false),
            show_metadata: Cell::new(false),
            show_terminal_sessions: Cell::new(true),
            two_pane_split: Cell::new(TwoPaneSplit::SideBySide),
            zoomed_terminal_index: Cell::new(None),
            terminal_sessions: RefCell::new(Vec::new()),
            snapshot_cache: RefCell::new(RuntimeSnapshot::default()),
            snapshot_dirty: Cell::new(true),
            preferred_terminal_focus: Cell::new(None),
            focused_terminal_index: Cell::new(None),
            rerender_scheduled: Cell::new(false),
            smoke_text_injection_ok: Cell::new(false),
            smoke_text_rendered_ok: Cell::new(false),
            smoke_reset_ok: Cell::new(false),
            smoke_focus_cycle_ok: Cell::new(false),
            smoke_close_ok: Cell::new(false),
            smoke_peak_terminal_count: Cell::new(1),
            pending_refresh_scopes: Arc::new(Mutex::new(Vec::new())),
            refresh_flush_scheduled: Cell::new(false),
            pending_bootstrap: RefCell::new(None),
            runtime_error: RefCell::new(None),
            runtime_online: Cell::new(false),
            mounted_shell_root: RefCell::new(None),
            live_terminal_targets: RefCell::new(Vec::new()),
            last_local_shell_signature: RefCell::new(String::new()),
            pending_startup_command: RefCell::new(std::env::var("SHIN_CHAN_STARTUP_COMMAND").ok()),
        });
        controller.install_window_actions();
        controller.install_window_shortcuts(app);
        controller.start_local_shell_monitor();
        controller
    }

    pub fn bootstrap(self: &Rc<Self>) {
        if self.runtime_online.get() || self.pending_bootstrap.borrow().is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel();
        let plan = NativeShellBootstrapPlan::default();
        std::thread::spawn(move || {
            let result = bootstrap_native_shell_runtime(&plan).map_err(|err| format!("{err:#}"));
            let _ = tx.send(result);
        });

        self.pending_bootstrap.replace(Some(rx));
        self.reset_terminal_sessions();
        self.schedule_rerender();

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(80), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let result = {
                let pending = controller.pending_bootstrap.borrow();
                let Some(rx) = pending.as_ref() else {
                    return glib::ControlFlow::Break;
                };
                match rx.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Empty) => None,
                    Err(mpsc::TryRecvError::Disconnected) => Some(Err(
                        "runtime bootstrap worker disconnected before returning".to_string(),
                    )),
                }
            };

            let Some(result) = result else {
                return glib::ControlFlow::Continue;
            };
            controller.pending_bootstrap.borrow_mut().take();
            match result {
                Ok(result) => {
                    controller.runtime_online.set(true);
                    controller
                        .selected_workspace
                        .replace(Some(result.mux.active_workspace()));
                    controller.mark_snapshot_dirty();
                    controller.runtime_error.replace(None);
                    controller.bind_mux_subscriptions(&result.mux);
                }
                Err(err) => {
                    controller.runtime_online.set(false);
                    controller
                        .runtime_error
                        .replace(Some(format!("runtime bootstrap failed: {err}")));
                }
            }
            controller.schedule_rerender();
            glib::ControlFlow::Break
        });
    }

    pub fn maybe_run_smoke_harness(self: &Rc<Self>) {
        if std::env::var("KAKU_NATIVE_SHELL_SMOKE").ok().as_deref() != Some("1") {
            return;
        }

        let weak = Rc::downgrade(self);
        let attempts = Rc::new(Cell::new(0u8));
        glib::timeout_add_local(std::time::Duration::from_millis(900), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let next_attempt = attempts.get().saturating_add(1);
            attempts.set(next_attempt);
            if !controller.runtime_online.get() {
                if next_attempt >= 8 {
                    controller.emit_smoke_report("bootstrap_timeout");
                    controller.window.close();
                    glib::idle_add_local_once(|| std::process::exit(1));
                    return glib::ControlFlow::Break;
                }
                return glib::ControlFlow::Continue;
            }
            controller.run_smoke_sequence();
            glib::ControlFlow::Break
        });
    }

    pub fn rerender(self: &Rc<Self>) {
        self.rerender_scheduled.set(false);
        let snapshot = self.snapshot();
        let width = self.window.width();
        let compact = width > 0 && width < 1260;
        let shell = build_shell(
            &snapshot,
            &ShellLayoutContract::default(),
            self.rail_collapsed.get(),
            PaneArrangement {
                compact,
                metadata_first: self.metadata_first.get(),
                tasks_first: self.tasks_first.get(),
                show_activity: self.show_activity.get(),
                show_tasks: self.show_tasks.get(),
                show_metadata: self.show_metadata.get(),
                metadata_collapsed: self.metadata_collapsed.get(),
                show_terminal_sessions: self.show_terminal_sessions.get(),
                terminal_count: self.terminal_sessions.borrow().len().max(1),
                inbox_collapsed: self.inbox_collapsed.get(),
                two_pane_split: self.two_pane_split.get(),
                zoomed_terminal_index: self.zoomed_terminal_index.get(),
            },
            &self.terminal_sessions.borrow(),
            width,
        );
        self.bind_shell_view(shell);
    }

    fn defer(self: &Rc<Self>, f: impl FnOnce(Rc<Self>) + 'static) {
        let this = Rc::clone(self);
        glib::idle_add_local_once(move || f(this));
    }

    fn schedule_rerender(self: &Rc<Self>) {
        if self.rerender_scheduled.replace(true) {
            return;
        }
        let this = Rc::clone(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(16), move || {
            this.rerender();
        });
    }

    fn mark_snapshot_dirty(&self) {
        self.snapshot_dirty.set(true);
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        if self.snapshot_dirty.replace(false) {
            let mut snapshot = if self.runtime_online.get() {
                derive_runtime_snapshot(self.selected_workspace.borrow().as_deref())
            } else {
                self.local_shell_only_snapshot()
            };
            self.augment_snapshot_with_local_shell_state(&mut snapshot);
            self.snapshot_cache.replace(snapshot.clone());
            snapshot
        } else {
            self.snapshot_cache.borrow().clone()
        }
    }

    fn local_shell_only_snapshot(&self) -> RuntimeSnapshot {
        let workspace_name = self
            .selected_workspace
            .borrow()
            .clone()
            .unwrap_or_else(|| "workspace".to_string());
        RuntimeSnapshot {
            active_workspace: workspace_name.clone(),
            workspaces: vec![WorkspaceSummary {
                name: workspace_name,
                ..WorkspaceSummary::default()
            }],
            ..RuntimeSnapshot::default()
        }
    }

    fn augment_snapshot_with_local_shell_state(&self, snapshot: &mut RuntimeSnapshot) {
        let sessions = self.terminal_sessions.borrow();
        if sessions.is_empty() {
            return;
        }

        let workspace_name = snapshot.active_workspace.clone();
        let focused_index = self
            .focused_terminal_index
            .get()
            .unwrap_or(0)
            .min(sessions.len().saturating_sub(1));
        let focused_surface = sessions
            .get(focused_index)
            .map(|handle| handle.surface_state())
            .unwrap_or_default();
        let failed_count = sessions
            .iter()
            .filter(|handle| handle.surface_state().status_badge.is_some())
            .count();

        if let Some(summary) = snapshot
            .workspaces
            .iter_mut()
            .find(|summary| summary.name == workspace_name)
        {
            let shell_count = sessions.len();
            let location = if focused_surface.title.is_empty() {
                std::env::current_dir()
                    .ok()
                    .and_then(|path| path.to_str().map(|cwd| cwd.to_string()))
                    .unwrap_or_else(|| "~".to_string())
            } else {
                focused_surface.title.clone()
            };
            let shell_summary = if shell_count == 1 {
                "1 SHELL".to_string()
            } else {
                format!("{shell_count} SHELLS")
            };
            summary.detail = Some(format!("{location} • {shell_summary}"));
            summary.running_count = shell_count;
            summary.failed_count = failed_count.max(summary.failed_count);
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let now: DateTime<Utc> = Utc
            .timestamp_opt(now.as_secs() as i64, now.subsec_nanos())
            .single()
            .expect("valid UTC timestamp");
        let local_notifications = sessions
            .iter()
            .enumerate()
            .filter_map(|(index, handle)| {
                let surface = handle.surface_state();
                let badge = surface.status_badge?;
                Some(NotificationRecord {
                    notification_id: format!(
                        "local-shell-{workspace_name}-{index}-{}",
                        badge.to_lowercase().replace(' ', "-")
                    ),
                    workspace: workspace_name.clone(),
                    window_id: None,
                    tab_id: None,
                    pane_id: None,
                    kind: "shell".to_string(),
                    title: badge,
                    body: Some(surface.title),
                    unread: true,
                    unread_mode: NotificationUnreadMode::Sticky,
                    created_at: now,
                    updated_at: now,
                })
            })
            .collect::<Vec<_>>();

        if !local_notifications.is_empty() {
            snapshot.notifications.retain(|row| {
                !(row.workspace == workspace_name && row.kind == "shell")
            });
            snapshot.notifications.extend(local_notifications);
            if let Some(summary) = snapshot
                .workspaces
                .iter_mut()
                .find(|summary| summary.name == workspace_name)
            {
                summary.unread_count = snapshot
                    .notifications
                    .iter()
                    .filter(|row| row.workspace == workspace_name && row.unread)
                    .count();
            }
        }
    }

    fn local_shell_signature(&self) -> String {
        let focused = self.focused_terminal_index.get().unwrap_or(0);
        let sessions = self.terminal_sessions.borrow();
        let parts = sessions
            .iter()
            .map(|handle| {
                let state = handle.state();
                let surface = handle.surface_state();
                format!(
                    "{}|{}|{}|{}|{}",
                    surface.title,
                    surface.status_badge.unwrap_or_default(),
                    state.live,
                    state.pending_spawn,
                    state.has_error
                )
            })
            .collect::<Vec<_>>();
        format!("{focused}::{}", parts.join("||"))
    }

    fn start_local_shell_monitor(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(180), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let signature = controller.local_shell_signature();
            let mut last = controller.last_local_shell_signature.borrow_mut();
            if *last != signature {
                *last = signature;
                controller.mark_snapshot_dirty();
                controller.schedule_rerender();
            }
            glib::ControlFlow::Continue
        });
    }

    fn bind_mux_subscriptions(self: &Rc<Self>, mux: &Arc<Mux>) {
        let weak = Rc::downgrade(self);
        let refresh_action = SimpleAction::new("refresh-from-mux", None);
        refresh_action.connect_activate(move |_, _| {
            if let Some(controller) = weak.upgrade() {
                controller.schedule_flush_pending_refresh_scopes();
            }
        });
        self.window_actions.add_action(&refresh_action);

        let pending_refresh_scopes = Arc::clone(&self.pending_refresh_scopes);
        let main_context = glib::MainContext::default();
        let window_weak = glib::SendWeakRef::from(self.window.downgrade());
        mux.subscribe(move |notification| {
            let scope = refresh_scope_for_notification(&notification);
            if scope != SnapshotRefreshScope::Ignore {
                if let Ok(mut pending) = pending_refresh_scopes.lock() {
                    pending.push(scope);
                } else {
                    eprintln!("native-shell: refresh queue lock poisoned during mux subscribe");
                    return true;
                }
                let window_weak = window_weak.clone();
                main_context.invoke(move || {
                    if let Some(window) = window_weak.upgrade() {
                        let _ = gtk::prelude::WidgetExt::activate_action(
                            &window,
                            "win.refresh-from-mux",
                            None,
                        );
                    }
                });
            }
            true
        });
    }

    fn flush_pending_refresh_scopes(self: &Rc<Self>) {
        if !self.runtime_online.get() {
            return;
        }

        let scopes = {
            let Ok(mut pending) = self.pending_refresh_scopes.lock() else {
                eprintln!("native-shell: refresh queue lock poisoned during flush");
                return;
            };
            std::mem::take(&mut *pending)
        };
        if scopes.is_empty() {
            return;
        }

        if scopes
            .iter()
            .any(|scope| matches!(scope, SnapshotRefreshScope::WorkspaceList))
        {
            self.selected_workspace
                .replace(Some(Mux::get().active_workspace()));
        }
        self.mark_snapshot_dirty();
        self.schedule_rerender();
    }

    fn schedule_flush_pending_refresh_scopes(self: &Rc<Self>) {
        if self.refresh_flush_scheduled.replace(true) {
            return;
        }
        let this = Rc::clone(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(120), move || {
            this.refresh_flush_scheduled.set(false);
            this.flush_pending_refresh_scopes();
        });
    }

    fn bind_shell_view(self: &Rc<Self>, shell: ShellView) {
        let ShellView {
            root,
            chrome_drag_handle,
            refresh_button,
            terminal_button,
            terminal_panes,
            rail_buttons,
            rail_inbox_toggle_button,
            metadata_toggle_button,
            activity_root: _,
            metadata_root,
            activity_panel: _,
            metadata_panel,
            activity_header: _,
            metadata_header,
            tasks_root,
            tasks_panel,
            tasks_header,
        } = shell;

        {
            let window = self.window.clone();
            let drag = GestureClick::new();
            drag.set_button(1);
            drag.connect_pressed(move |gesture, _, x, y| {
                let Some(device) = gesture.current_event_device() else {
                    return;
                };
                let Some(surface) = window.surface() else {
                    return;
                };
                let Ok(toplevel) = surface.dynamic_cast::<gdk::Toplevel>() else {
                    return;
                };
                toplevel.begin_move(
                    &device,
                    gesture.current_button() as i32,
                    x,
                    y,
                    gesture.current_event_time(),
                );
            });
            chrome_drag_handle.add_controller(drag);
        }

        {
            let this = Rc::clone(self);
            refresh_button
                .connect_clicked(move |_| this.defer(|controller| controller.schedule_rerender()));
        }
        {
            let this = Rc::clone(self);
            terminal_button.connect_clicked(move |_| {
                this.defer(|controller| controller.split_focused_terminal())
            });
        }
        let focus_targets = terminal_panes
            .iter()
            .map(|pane| pane.focus_target.clone())
            .collect::<Vec<_>>();
        for pane in terminal_panes {
            if self.preferred_terminal_focus.get() == Some(pane.index) {
                pane.focus_target.grab_focus();
                self.focused_terminal_index.set(Some(pane.index));
                self.preferred_terminal_focus.set(None);
            }
            {
                let this = Rc::clone(self);
                let pane_index = pane.index;
                pane.focus_target
                    .connect_notify_local(Some("has-focus"), move |widget, _| {
                        if widget.has_focus() {
                            this.focused_terminal_index.set(Some(pane_index));
                        }
                    });
            }
            let this = Rc::clone(self);
            let source_index = pane.index;
            let drag_source = gtk::DragSource::builder()
                .actions(gdk::DragAction::MOVE)
                .build();
            let drag_handle_begin = pane.drag_handle.clone();
            drag_source.connect_drag_begin(move |_, _| {
                drag_handle_begin.add_css_class("drag-source");
            });
            let drag_handle_end = pane.drag_handle.clone();
            drag_source.connect_drag_end(move |_, _, _| {
                drag_handle_end.remove_css_class("drag-source");
            });
            drag_source.connect_prepare(move |_, _, _| {
                Some(gdk::ContentProvider::for_value(
                    &source_index.to_string().to_value(),
                ))
            });
            pane.drag_handle.add_controller(drag_source);

            let drop_target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
            let drop_target_enter = pane.drop_target.clone();
            drop_target.connect_enter(move |_, _, _| {
                drop_target_enter.add_css_class("drop-target");
                gdk::DragAction::MOVE
            });
            let drop_target_leave = pane.drop_target.clone();
            drop_target.connect_leave(move |_| {
                drop_target_leave.remove_css_class("drop-target");
            });
            let drop_target_drop = pane.drop_target.clone();
            let target_index = pane.index;
            drop_target.connect_drop(move |_, value, _, _| {
                drop_target_drop.remove_css_class("drop-target");
                let Ok(payload) = value.get::<String>() else {
                    return false;
                };
                let Ok(source_index) = payload.parse::<usize>() else {
                    return false;
                };
                if source_index == target_index {
                    return true;
                }
                this.defer(move |controller| {
                    controller.reorder_terminal_sessions(source_index, target_index)
                });
                true
            });
            pane.drop_target.add_controller(drop_target);
            {
                let this = Rc::clone(self);
                let pane_index = pane.index;
                pane.close_button.connect_clicked(move |_| {
                    this.defer(move |controller| controller.close_terminal_at(pane_index))
                });
            }
        }
        self.live_terminal_targets.replace(focus_targets);
        for (workspace, button) in rail_buttons {
            let this = Rc::clone(self);
            button.connect_clicked(move |_| {
                let workspace = workspace.clone();
                this.defer(move |controller| controller.select_workspace(&workspace));
            });
        }
        {
            let this = Rc::clone(self);
            rail_inbox_toggle_button
                .connect_clicked(move |_| this.defer(|controller| controller.toggle_rail_inbox()));
        }
        {
            let this = Rc::clone(self);
            metadata_toggle_button.connect_clicked(move |_| {
                this.defer(|controller| controller.toggle_metadata_body())
            });
        }
        {
            let this = Rc::clone(self);
            if this.show_metadata.get() {
                bind_header_swap(
                    &tasks_header,
                    &tasks_root,
                    &metadata_header,
                    &metadata_panel,
                    &metadata_root,
                    "side-swap",
                    move || this.defer(|controller| controller.toggle_side_panes()),
                );
            }
        }
        if self.show_metadata.get() {
            let this = Rc::clone(self);
            bind_header_swap(
                &metadata_header,
                &metadata_root,
                &tasks_header,
                &tasks_panel,
                &tasks_root,
                "side-swap",
                move || this.defer(|controller| controller.toggle_side_panes()),
            );
        }

        if let Some(previous_root) = self.mounted_shell_root.borrow_mut().take() {
            self.content_host.remove(&previous_root);
        }
        self.content_host.append(&root);
        self.mounted_shell_root.replace(Some(root.clone().upcast()));
        self.window.set_size_request(-1, -1);
        self.clamp_window_to_workarea();
    }

    fn clamp_window_to_workarea(&self) {
        let Some(surface) = self.window.surface() else {
            return;
        };
        let display = surface.display();
        let Some(monitor) = display.monitor_at_surface(&surface) else {
            return;
        };
        let geometry = monitor.geometry();
        let max_width = (geometry.width() - 80).max(960);
        let max_height = (geometry.height() - 80).max(720);
        let target_width = self.window.width().max(1180).min(max_width);
        let target_height = self.window.height().max(820).min(max_height);
        self.window.set_default_size(target_width, target_height);
    }

    fn select_workspace(self: &Rc<Self>, workspace: &str) {
        if !self.runtime_online.get() {
            return;
        }
        let mux = Mux::get();
        mux.set_active_workspace(workspace);
        self.selected_workspace.replace(Some(workspace.to_string()));
        self.mark_snapshot_dirty();
        self.schedule_rerender();
    }

    fn toggle_rail(self: &Rc<Self>) {
        self.rail_collapsed.set(!self.rail_collapsed.get());
        self.schedule_rerender();
    }

    fn toggle_rail_inbox(self: &Rc<Self>) {
        self.inbox_collapsed.set(!self.inbox_collapsed.get());
        self.schedule_rerender();
    }

    fn toggle_side_panes(self: &Rc<Self>) {
        self.tasks_first.set(!self.tasks_first.get());
        self.schedule_rerender();
    }

    fn toggle_metadata_body(self: &Rc<Self>) {
        self.metadata_collapsed.set(!self.metadata_collapsed.get());
        self.schedule_rerender();
    }

    fn install_window_actions(self: &Rc<Self>) {
        self.install_action("launch-terminal", {
            let this = Rc::clone(self);
            move || this.split_focused_terminal()
        });
        self.install_action("split-right", {
            let this = Rc::clone(self);
            move || this.split_terminal_with(TwoPaneSplit::SideBySide)
        });
        self.install_action("split-down", {
            let this = Rc::clone(self);
            move || this.split_terminal_with(TwoPaneSplit::Stacked)
        });
        self.install_action("toggle-split-direction", {
            let this = Rc::clone(self);
            move || this.toggle_two_pane_split_direction()
        });
        self.install_action("toggle-zoom-terminal", {
            let this = Rc::clone(self);
            move || this.toggle_zoom_focused_terminal()
        });
        self.install_action("launch-lazygit", {
            let this = Rc::clone(self);
            move || this.launch_tool_in_focused_terminal("lazygit")
        });
        self.install_action("launch-yazi", {
            let this = Rc::clone(self);
            move || this.launch_tool_in_focused_terminal("yazi")
        });
        self.install_action("run-doctor", {
            let this = Rc::clone(self);
            move || this.launch_tool_in_focused_terminal("kaku doctor")
        });
        self.install_action("open-config", {
            let this = Rc::clone(self);
            move || this.launch_tool_in_focused_terminal("kaku config")
        });
        self.install_action("focus-next-terminal", {
            let this = Rc::clone(self);
            move || this.focus_next_terminal()
        });
        self.install_action("focus-previous-terminal", {
            let this = Rc::clone(self);
            move || this.focus_previous_terminal()
        });
        self.install_action("close-terminal", {
            let this = Rc::clone(self);
            move || this.close_focused_terminal()
        });
        self.install_action("set-status", {
            let this = Rc::clone(self);
            move || this.set_status_for_selected()
        });
        self.install_action("clear-status", {
            let this = Rc::clone(self);
            move || this.clear_status_for_selected()
        });
        self.install_action("set-progress", {
            let this = Rc::clone(self);
            move || this.set_progress_for_selected()
        });
        self.install_action("clear-progress", {
            let this = Rc::clone(self);
            move || this.clear_progress_for_selected()
        });
        self.install_action("append-log", {
            let this = Rc::clone(self);
            move || this.append_log_for_selected()
        });
        self.install_action("reset-shell", {
            let this = Rc::clone(self);
            move || this.reset_shell_surface()
        });
        self.install_action("toggle-rail", {
            let this = Rc::clone(self);
            move || this.toggle_rail()
        });
        self.install_action("toggle-metadata", {
            let this = Rc::clone(self);
            move || this.toggle_metadata()
        });
        self.install_action("toggle-activity", {
            let this = Rc::clone(self);
            move || this.toggle_activity()
        });
        self.install_action("toggle-tasks", {
            let this = Rc::clone(self);
            move || this.toggle_tasks()
        });
    }

    fn run_smoke_sequence(self: &Rc<Self>) {
        self.smoke_text_injection_ok.set(false);
        self.smoke_text_rendered_ok.set(false);
        self.smoke_reset_ok.set(false);
        self.smoke_focus_cycle_ok.set(false);
        self.smoke_close_ok.set(false);
        self.smoke_peak_terminal_count
            .set(self.terminal_sessions.borrow().len().max(1));

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(150), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            controller.ensure_terminal_sessions();
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(450), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            controller.ensure_terminal_sessions();
            controller.ensure_terminal_sessions();
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(1150), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let before_focus = controller.focused_terminal_index.get();
            controller.focus_next_terminal();
            let after_next = controller.focused_terminal_index.get();
            controller.focus_previous_terminal();
            let after_previous = controller.focused_terminal_index.get();
            let focus_cycle_ok = before_focus.is_some()
                && after_next != before_focus
                && after_previous == before_focus;
            controller.smoke_focus_cycle_ok.set(focus_cycle_ok);
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(1400), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let before_len = controller.terminal_sessions.borrow().len();
            controller.close_focused_terminal();
            let after_len = controller.terminal_sessions.borrow().len();
            controller
                .smoke_close_ok
                .set(before_len > 1 && after_len + 1 == before_len);
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(1650), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let mut text_injection_ok = false;
            if let Some(last) = controller.terminal_sessions.borrow().last() {
                text_injection_ok = last.send_text("printf 'SMOKE_OK\\n'");
            }
            controller.smoke_text_injection_ok.set(text_injection_ok);
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(2350), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let rendered = controller
                .terminal_sessions
                .borrow()
                .last()
                .map(|terminal| terminal.viewport_contains("SMOKE_OK"))
                .unwrap_or(false);
            controller.smoke_text_rendered_ok.set(rendered);
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(2700), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            controller.reset_shell_surface();
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(3200), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let reset_ok = {
                let sessions = controller.terminal_sessions.borrow();
                sessions.len() == 1
                    && sessions
                        .first()
                        .map(|terminal| {
                            let state = terminal.state();
                            state.live || state.pending_spawn
                        })
                        .unwrap_or(false)
            };
            controller.smoke_reset_ok.set(reset_ok);
            glib::ControlFlow::Break
        });

        let weak = Rc::downgrade(self);
        glib::timeout_add_local(std::time::Duration::from_millis(3700), move || {
            let Some(controller) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            controller.emit_smoke_report("complete");
            controller.window.close();
            glib::idle_add_local_once(|| std::process::exit(0));
            glib::ControlFlow::Break
        });
    }

    fn emit_smoke_report(&self, final_step: &'static str) {
        let sessions = self.terminal_sessions.borrow();
        let mut live_terminals = 0usize;
        let mut pending_terminals = 0usize;
        let mut errored_terminals = 0usize;
        for terminal in sessions.iter() {
            let state = terminal.state();
            if state.live {
                live_terminals += 1;
            }
            if state.pending_spawn {
                pending_terminals += 1;
            }
            if state.has_error {
                errored_terminals += 1;
            }
        }

        let report = SmokeHarnessReport {
            startup_online: self.runtime_online.get(),
            terminal_count: sessions.len(),
            live_terminals,
            pending_terminals,
            errored_terminals,
            peak_terminal_count: self.smoke_peak_terminal_count.get(),
            text_injection_ok: self.smoke_text_injection_ok.get(),
            text_rendered_ok: self.smoke_text_rendered_ok.get(),
            focus_cycle_ok: self.smoke_focus_cycle_ok.get(),
            close_ok: self.smoke_close_ok.get(),
            reset_ok: self.smoke_reset_ok.get(),
            final_step,
        };
        eprintln!(
            "KAKU_NATIVE_SHELL_SMOKE startup_online={} terminal_count={} live_terminals={} pending_terminals={} errored_terminals={} peak_terminal_count={} text_injection_ok={} text_rendered_ok={} focus_cycle_ok={} close_ok={} reset_ok={} final_step={}",
            report.startup_online,
            report.terminal_count,
            report.live_terminals,
            report.pending_terminals,
            report.errored_terminals,
            report.peak_terminal_count,
            report.text_injection_ok,
            report.text_rendered_ok,
            report.focus_cycle_ok,
            report.close_ok,
            report.reset_ok,
            report.final_step
        );
    }

    fn install_window_shortcuts(self: &Rc<Self>, app: &adw::Application) {
        let this = Rc::clone(self);
        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(move |_, key, _, state| {
            let command = state.contains(gdk::ModifierType::META_MASK)
                || state.contains(gdk::ModifierType::SUPER_MASK)
                || state.contains(gdk::ModifierType::CONTROL_MASK);
            if !command {
                return glib::Propagation::Proceed;
            }

            let shifted = state.contains(gdk::ModifierType::SHIFT_MASK);
            let handled = match (key, shifted) {
                (gdk::Key::t, false) | (gdk::Key::T, false) => {
                    this.defer(|controller| controller.split_focused_terminal());
                    true
                }
                (gdk::Key::d, false) | (gdk::Key::D, false) => {
                    this.defer(|controller| {
                        controller.split_terminal_with(TwoPaneSplit::SideBySide)
                    });
                    true
                }
                (gdk::Key::d, true) | (gdk::Key::D, true) => {
                    this.defer(|controller| controller.split_terminal_with(TwoPaneSplit::Stacked));
                    true
                }
                (gdk::Key::s, true) | (gdk::Key::S, true) => {
                    this.defer(|controller| controller.toggle_two_pane_split_direction());
                    true
                }
                (gdk::Key::Return, true) => {
                    this.defer(|controller| controller.toggle_zoom_focused_terminal());
                    true
                }
                (gdk::Key::bracketright, false) => {
                    this.defer(|controller| controller.focus_next_terminal());
                    true
                }
                (gdk::Key::bracketleft, false) => {
                    this.defer(|controller| controller.focus_previous_terminal());
                    true
                }
                (gdk::Key::w, false) | (gdk::Key::W, false) => {
                    this.defer(|controller| controller.close_focused_terminal());
                    true
                }
                (gdk::Key::b, false) | (gdk::Key::B, false) => {
                    this.defer(|controller| controller.toggle_rail());
                    true
                }
                (gdk::Key::a, true) | (gdk::Key::A, true) => {
                    this.defer(|controller| controller.toggle_activity());
                    true
                }
                (gdk::Key::g, true) | (gdk::Key::G, true) => {
                    this.defer(|controller| controller.launch_tool_in_focused_terminal("lazygit"));
                    true
                }
                (gdk::Key::y, true) | (gdk::Key::Y, true) => {
                    this.defer(|controller| controller.launch_tool_in_focused_terminal("yazi"));
                    true
                }
                (gdk::Key::comma, false) => {
                    this.defer(|controller| {
                        controller.launch_tool_in_focused_terminal("kaku config")
                    });
                    true
                }
                (gdk::Key::t, true) | (gdk::Key::T, true) => {
                    this.defer(|controller| controller.toggle_tasks());
                    true
                }
                (gdk::Key::m, true) | (gdk::Key::M, true) => {
                    this.defer(|controller| controller.toggle_metadata());
                    true
                }
                (gdk::Key::x, true) | (gdk::Key::X, true) => {
                    this.defer(|controller| controller.clear_status_for_selected());
                    true
                }
                (gdk::Key::p, true) | (gdk::Key::P, true) => {
                    this.defer(|controller| controller.set_progress_for_selected());
                    true
                }
                (gdk::Key::l, true) | (gdk::Key::L, true) => {
                    this.defer(|controller| controller.append_log_for_selected());
                    true
                }
                (gdk::Key::o, true) | (gdk::Key::O, true) => {
                    this.defer(|controller| {
                        controller.launch_tool_in_focused_terminal("kaku doctor")
                    });
                    true
                }
                (gdk::Key::r, true) | (gdk::Key::R, true) => {
                    this.defer(|controller| controller.reset_shell_surface());
                    true
                }
                _ => false,
            };

            if handled {
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        self.window.add_controller(controller);

        app.set_accels_for_action("win.launch-terminal", &["<Meta>t", "<Primary>t"]);
        app.set_accels_for_action("win.split-right", &["<Meta>d", "<Primary>d"]);
        app.set_accels_for_action(
            "win.split-down",
            &["<Meta><Shift>d", "<Primary><Shift>d"],
        );
        app.set_accels_for_action(
            "win.toggle-split-direction",
            &["<Meta><Shift>s", "<Primary><Shift>s"],
        );
        app.set_accels_for_action(
            "win.toggle-zoom-terminal",
            &["<Meta><Shift>Return", "<Primary><Shift>Return"],
        );
        app.set_accels_for_action(
            "win.focus-next-terminal",
            &["<Meta>bracketright", "<Primary>bracketright"],
        );
        app.set_accels_for_action(
            "win.focus-previous-terminal",
            &["<Meta>bracketleft", "<Primary>bracketleft"],
        );
        app.set_accels_for_action("win.close-terminal", &["<Meta>w", "<Primary>w"]);
        app.set_accels_for_action("win.toggle-rail", &["<Meta>b", "<Primary>b"]);
        app.set_accels_for_action(
            "win.toggle-activity",
            &["<Meta><Shift>a", "<Primary><Shift>a"],
        );
        app.set_accels_for_action(
            "win.launch-lazygit",
            &["<Meta><Shift>g", "<Primary><Shift>g"],
        );
        app.set_accels_for_action(
            "win.launch-yazi",
            &["<Meta><Shift>y", "<Primary><Shift>y"],
        );
        app.set_accels_for_action("win.open-config", &["<Meta>comma", "<Primary>comma"]);
        app.set_accels_for_action(
            "win.toggle-tasks",
            &["<Meta><Shift>t", "<Primary><Shift>t"],
        );
        app.set_accels_for_action(
            "win.toggle-metadata",
            &["<Meta><Shift>m", "<Primary><Shift>m"],
        );
        app.set_accels_for_action(
            "win.clear-status",
            &["<Meta><Shift>x", "<Primary><Shift>x"],
        );
        app.set_accels_for_action(
            "win.set-progress",
            &["<Meta><Shift>p", "<Primary><Shift>p"],
        );
        app.set_accels_for_action(
            "win.append-log",
            &["<Meta><Shift>l", "<Primary><Shift>l"],
        );
        app.set_accels_for_action("win.run-doctor", &["<Meta><Shift>o", "<Primary><Shift>o"]);
        app.set_accels_for_action("win.reset-shell", &["<Meta><Shift>r", "<Primary><Shift>r"]);
    }

    fn install_action(self: &Rc<Self>, name: &str, handler: impl Fn() + 'static) {
        let action = SimpleAction::new(name, None);
        let handler = Rc::new(handler);
        action.connect_activate(move |_, _| {
            let handler = Rc::clone(&handler);
            glib::idle_add_local_once(move || {
                let _ = catch_unwind(AssertUnwindSafe(|| handler()));
            });
        });
        self.window_actions.add_action(&action);
    }

    fn set_status_for_selected(self: &Rc<Self>) {
        self.apply_action(ShellAction::SetStatus);
    }

    fn clear_status_for_selected(self: &Rc<Self>) {
        self.apply_action(ShellAction::ClearStatus);
    }

    fn set_progress_for_selected(self: &Rc<Self>) {
        self.apply_action(ShellAction::SetProgress);
    }

    fn clear_progress_for_selected(self: &Rc<Self>) {
        self.apply_action(ShellAction::ClearProgress);
    }

    fn append_log_for_selected(self: &Rc<Self>) {
        self.apply_action(ShellAction::AppendLog);
    }

    fn ensure_terminal_sessions(self: &Rc<Self>) {
        self.split_terminal_with(self.two_pane_split.get());
    }

    fn split_focused_terminal(self: &Rc<Self>) {
        self.split_terminal_with(self.two_pane_split.get());
    }

    fn split_terminal_with(self: &Rc<Self>, split: TwoPaneSplit) {
        self.show_terminal_sessions.set(true);
        self.two_pane_split.set(split);
        self.zoomed_terminal_index.set(None);
        let cwd = self.current_workspace_cwd();
        {
            let mut sessions = self.terminal_sessions.borrow_mut();
            if sessions.len() < 4 {
                let next_index = sessions.len();
                sessions.push(spawn_terminal_handle(cwd));
                self.preferred_terminal_focus.set(Some(next_index));
                self.focused_terminal_index.set(Some(next_index));
                self.smoke_peak_terminal_count
                    .set(self.smoke_peak_terminal_count.get().max(sessions.len()));
            }
        }
        self.runtime_error.replace(None);
        self.schedule_rerender();
    }

    fn toggle_two_pane_split_direction(self: &Rc<Self>) {
        let next = match self.two_pane_split.get() {
            TwoPaneSplit::SideBySide => TwoPaneSplit::Stacked,
            TwoPaneSplit::Stacked => TwoPaneSplit::SideBySide,
        };
        self.two_pane_split.set(next);
        self.schedule_rerender();
    }

    fn toggle_zoom_focused_terminal(self: &Rc<Self>) {
        let Some(index) = self.focused_terminal_index.get() else {
            return;
        };
        let next = match self.zoomed_terminal_index.get() {
            Some(current) if current == index => None,
            _ => Some(index),
        };
        self.zoomed_terminal_index.set(next);
        self.preferred_terminal_focus.set(Some(index));
        self.schedule_rerender();
    }

    fn launch_tool_in_focused_terminal(self: &Rc<Self>, command: &str) {
        let launch_in_new_pane = matches!(command, "lazygit" | "yazi");
        let index = if launch_in_new_pane {
            let cwd = self.current_workspace_cwd();
            let next_index = {
                let mut sessions = self.terminal_sessions.borrow_mut();
                if sessions.len() < 4 {
                    sessions.push(spawn_terminal_handle(cwd));
                    sessions.len() - 1
                } else {
                    self.focused_terminal_index.get().unwrap_or(0)
                }
            };
            self.preferred_terminal_focus.set(Some(next_index));
            self.focused_terminal_index.set(Some(next_index));
            self.runtime_error.replace(None);
            self.schedule_rerender();
            next_index
        } else {
            self.focused_terminal_index.get().unwrap_or(0)
        };
        let snippet = shell_tool_launch_snippet(command);
        let sent = self
            .terminal_sessions
            .borrow()
            .get(index)
            .map(|terminal| terminal.send_text(&format!("{snippet}\n")))
            .unwrap_or(false);
        if sent {
            self.preferred_terminal_focus.set(Some(index));
            self.focused_terminal_index.set(Some(index));
        }
    }

    fn reset_terminal_sessions(&self) {
        let cwd = self.current_workspace_cwd();
        let mut sessions = self.terminal_sessions.borrow_mut();
        sessions.clear();
        let handle = spawn_terminal_handle(cwd);
        if let Some(command) = self.pending_startup_command.borrow_mut().take() {
            let _ = handle.send_text(&format!("{command}\n"));
        }
        sessions.push(handle);
        self.preferred_terminal_focus.set(Some(0));
        self.focused_terminal_index.set(Some(0));
    }

    fn reorder_terminal_sessions(self: &Rc<Self>, source: usize, target: usize) {
        let mut sessions = self.terminal_sessions.borrow_mut();
        if source >= sessions.len() || target >= sessions.len() || source == target {
            return;
        }
        sessions.swap(source, target);
        if let Some(zoomed) = self.zoomed_terminal_index.get() {
            let next_zoomed = match zoomed {
                value if value == source => Some(target),
                value if value == target => Some(source),
                value => Some(value),
            };
            self.zoomed_terminal_index.set(next_zoomed);
        }
        let focused = self.focused_terminal_index.get();
        let next_focus = match focused {
            Some(index) if index == source => Some(target),
            Some(index) if index == target => Some(source),
            other => other,
        };
        self.focused_terminal_index.set(next_focus);
        drop(sessions);
        self.preferred_terminal_focus.set(next_focus);
        self.schedule_rerender();
    }

    fn focus_next_terminal(self: &Rc<Self>) {
        let len = self.terminal_sessions.borrow().len();
        if len == 0 {
            return;
        }
        let current = self.focused_terminal_index.get().unwrap_or(0);
        let next = (current + 1) % len;
        self.focus_terminal_index(next);
    }

    fn focus_previous_terminal(self: &Rc<Self>) {
        let len = self.terminal_sessions.borrow().len();
        if len == 0 {
            return;
        }
        let current = self.focused_terminal_index.get().unwrap_or(0);
        let previous = if current == 0 { len - 1 } else { current - 1 };
        self.focus_terminal_index(previous);
    }

    fn close_focused_terminal(self: &Rc<Self>) {
        let mut sessions = self.terminal_sessions.borrow_mut();
        if sessions.len() <= 1 {
            return;
        }
        let focused = self
            .focused_terminal_index
            .get()
            .unwrap_or_else(|| sessions.len().saturating_sub(1))
            .min(sessions.len() - 1);
        sessions.remove(focused);
        let next_focus = focused.min(sessions.len().saturating_sub(1));
        drop(sessions);
        self.remap_zoom_after_removal(focused);
        self.focused_terminal_index.set(Some(next_focus));
        self.preferred_terminal_focus.set(Some(next_focus));
        self.schedule_rerender();
    }

    fn close_terminal_at(self: &Rc<Self>, index: usize) {
        let mut sessions = self.terminal_sessions.borrow_mut();
        if sessions.len() <= 1 || index >= sessions.len() {
            return;
        }
        sessions.remove(index);
        let focused = self.focused_terminal_index.get();
        let next_focus = match focused {
            Some(current) if current == index => index.min(sessions.len().saturating_sub(1)),
            Some(current) if current > index => current.saturating_sub(1),
            Some(current) => current,
            None => index.min(sessions.len().saturating_sub(1)),
        };
        drop(sessions);
        self.remap_zoom_after_removal(index);
        self.focused_terminal_index.set(Some(next_focus));
        self.preferred_terminal_focus.set(Some(next_focus));
        self.schedule_rerender();
    }

    fn remap_zoom_after_removal(&self, removed_index: usize) {
        let next_zoomed = match self.zoomed_terminal_index.get() {
            Some(current) if current == removed_index => None,
            Some(current) if current > removed_index => Some(current - 1),
            other => other,
        };
        self.zoomed_terminal_index.set(next_zoomed);
    }

    fn reset_shell_surface(self: &Rc<Self>) {
        self.rail_collapsed.set(false);
        self.inbox_collapsed.set(false);
        self.metadata_collapsed.set(false);
        self.metadata_first.set(false);
        self.tasks_first.set(false);
        self.show_activity.set(false);
        self.show_tasks.set(false);
        self.show_metadata.set(false);
        self.show_terminal_sessions.set(true);
        self.two_pane_split.set(TwoPaneSplit::SideBySide);
        self.zoomed_terminal_index.set(None);
        self.reset_terminal_sessions();
        self.focused_terminal_index.set(Some(0));
        self.schedule_rerender();
    }

    fn toggle_activity(self: &Rc<Self>) {
        self.show_activity.set(!self.show_activity.get());
        self.schedule_rerender();
    }

    fn toggle_tasks(self: &Rc<Self>) {
        self.show_tasks.set(!self.show_tasks.get());
        self.schedule_rerender();
    }

    fn toggle_metadata(self: &Rc<Self>) {
        self.show_metadata.set(!self.show_metadata.get());
        if self.show_metadata.get() {
            self.metadata_collapsed.set(false);
        }
        self.schedule_rerender();
    }

    fn selected_workspace_name(&self) -> String {
        self.selected_workspace
            .borrow()
            .clone()
            .unwrap_or_else(|| "workspace".to_string())
    }

    fn current_workspace_cwd(&self) -> Option<String> {
        let snapshot = self.snapshot();
        snapshot
            .task_panes
            .iter()
            .find(|row| row.workspace.as_deref() == Some(snapshot.active_workspace.as_str()))
            .and_then(|row| row.current_working_dir.clone())
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|path| path.to_str().map(|cwd| cwd.to_string()))
            })
    }

    fn focus_terminal_index(&self, index: usize) {
        self.focused_terminal_index.set(Some(index));
        self.preferred_terminal_focus.set(Some(index));
        if let Some(target) = self.live_terminal_targets.borrow().get(index) {
            target.grab_focus();
            self.preferred_terminal_focus.set(None);
        }
    }

    fn apply_action(self: &Rc<Self>, action: ShellAction) {
        if !self.runtime_online.get() {
            return;
        }
        let context = {
            let mux = Mux::get();
            self.action_context(mux.as_ref())
        };
        let pending_refresh_scopes = Arc::clone(&self.pending_refresh_scopes);
        let main_context = glib::MainContext::default();
        let window_weak = glib::SendWeakRef::from(self.window.downgrade());

        std::thread::spawn(move || {
            let mux = Mux::get();
            let outcome = action.execute(mux.as_ref(), &context);
            let should_refresh = matches!(outcome, ShellActionOutcome::Mutated)
                || matches!(action, ShellAction::Refresh);
            if should_refresh {
                if let Ok(mut pending) = pending_refresh_scopes.lock() {
                    pending.push(SnapshotRefreshScope::WorkspaceContext);
                } else {
                    eprintln!("native-shell: refresh queue lock poisoned during action execute");
                    return;
                }
                main_context.invoke(move || {
                    if let Some(window) = window_weak.upgrade() {
                        let _ = gtk::prelude::WidgetExt::activate_action(
                            &window,
                            "win.refresh-from-mux",
                            None,
                        );
                    }
                });
            }
        });
    }

    fn action_context(&self, mux: &Mux) -> ShellActionContext {
        let workspace = self.selected_workspace_name();
        let current_status = mux
            .workspace_status_for_workspace(&workspace)
            .map(|record| record.status);
        let current_progress = mux
            .workspace_progress_for_workspace(&workspace)
            .map(|record| record.value);
        let visible_notification_ids = mux
            .list_notifications()
            .into_iter()
            .filter(|row| row.workspace == workspace && row.unread)
            .map(|row| row.notification_id)
            .collect::<Vec<_>>();
        let unix_timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        ShellActionContext {
            workspace,
            current_status,
            current_progress,
            visible_notification_ids,
            unix_timestamp_secs,
        }
    }
}

fn shell_tool_launch_snippet(command: &str) -> String {
    match command {
        "lazygit" => tool_presence_wrapper("lazygit", "lazygit"),
        "yazi" => tool_presence_wrapper("yazi", "yazi"),
        "kaku doctor" => tool_presence_wrapper("kaku", "kaku doctor"),
        "kaku config" => tool_presence_wrapper("kaku", "kaku config"),
        other => other.to_string(),
    }
}

fn tool_presence_wrapper(binary: &str, launch: &str) -> String {
    let binary_label = binary.to_ascii_uppercase();
    format!(
        "if command -v {binary} >/dev/null 2>&1; then {launch}; else printf '{binary_label} IS NOT INSTALLED\\n'; fi"
    )
}

#[cfg(test)]
mod tests {
    use super::shell_tool_launch_snippet;

    #[test]
    fn lazygit_launch_snippet_is_user_friendly() {
        let snippet = shell_tool_launch_snippet("lazygit");
        assert!(snippet.contains("command -v lazygit"));
        assert!(snippet.contains("lazygit;"));
        assert!(snippet.contains("LAZYGIT IS NOT INSTALLED"));
    }

    #[test]
    fn kaku_config_launch_snippet_checks_for_kaku_binary() {
        let snippet = shell_tool_launch_snippet("kaku config");
        assert!(snippet.contains("command -v kaku"));
        assert!(snippet.contains("kaku config"));
        assert!(snippet.contains("KAKU IS NOT INSTALLED"));
    }
}
