use crate::actions::{ShellAction, ShellActionContext, ShellActionOutcome};
use crate::runtime_bridge::{bootstrap_native_shell_runtime, NativeShellBootstrapPlan};
use crate::snapshot::{
    derive_runtime_snapshot, refresh_scope_for_notification, RuntimeSnapshot, ShellLayoutContract,
    SnapshotRefreshScope,
};
use crate::view::{bind_header_swap, build_shell, PaneArrangement, ShellView};
use adw::prelude::*;
use gio::SimpleAction;
use gtk::gdk;
use gtk::gdk::prelude::ToplevelExt;
use gtk::prelude::{EventControllerExt, GestureSingleExt, NativeExt, WidgetExt};
use gtk::GestureClick;
use mux::Mux;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
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
    pub action_labels: [&'static str; 7],
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
            primary_mono_family: [
                "Basically A Mono",
                "1984대화나눔_본문체_Regular",
                "monospace",
            ],
            operator_classes: ["chrome-title", "rail-name", "pane-title"],
        },
        affordances: ShellAffordanceContract {
            compact_count_labels: ["1U 2R 0F", "70% 5L", "U1 T2"],
            header_badges: ["◉ unity-main", "3U", "2T"],
            action_labels: [
                "↻", "terminal", "status", "clear", "progress", "clear", "log",
            ],
        },
    }
}

pub struct AppController {
    pub window: adw::ApplicationWindow,
    selected_workspace: RefCell<Option<String>>,
    rail_collapsed: Cell<bool>,
    metadata_first: Cell<bool>,
    tasks_first: Cell<bool>,
    show_terminal_sessions: Cell<bool>,
    pending_refresh_scopes: Arc<Mutex<Vec<SnapshotRefreshScope>>>,
    runtime_error: RefCell<Option<String>>,
    runtime_online: Cell<bool>,
}

impl AppController {
    pub fn new(app: &adw::Application) -> Rc<Self> {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Kaku Native Shell")
            .default_width(1480)
            .default_height(920)
            .build();
        window.add_css_class("native-shell-window");

        let controller = Rc::new(Self {
            window,
            selected_workspace: RefCell::new(None),
            rail_collapsed: Cell::new(false),
            metadata_first: Cell::new(false),
            tasks_first: Cell::new(false),
            show_terminal_sessions: Cell::new(true),
            pending_refresh_scopes: Arc::new(Mutex::new(Vec::new())),
            runtime_error: RefCell::new(None),
            runtime_online: Cell::new(false),
        });
        controller.install_window_actions(app);
        controller
    }

    pub fn bootstrap(self: &Rc<Self>) {
        let plan = NativeShellBootstrapPlan::default();
        match bootstrap_native_shell_runtime(&plan) {
            Ok(result) => {
                self.runtime_online.set(true);
                self.selected_workspace
                    .replace(Some(result.mux.active_workspace()));
                self.runtime_error.replace(None);
                self.bind_mux_subscriptions(&result.mux);
            }
            Err(err) => {
                self.runtime_online.set(false);
                self.runtime_error
                    .replace(Some(format!("runtime bootstrap failed: {err:#}")));
            }
        }

        self.rerender();
    }

    pub fn rerender(self: &Rc<Self>) {
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
                show_terminal_sessions: self.show_terminal_sessions.get(),
            },
        );
        self.bind_shell_view(shell);
    }

    fn defer(self: &Rc<Self>, f: impl FnOnce(Rc<Self>) + 'static) {
        let this = Rc::clone(self);
        glib::idle_add_local_once(move || f(this));
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        if !self.runtime_online.get() {
            return RuntimeSnapshot::default();
        }

        derive_runtime_snapshot(self.selected_workspace.borrow().as_deref())
    }

    fn bind_mux_subscriptions(self: &Rc<Self>, mux: &Arc<Mux>) {
        let weak = Rc::downgrade(self);
        let refresh_action = SimpleAction::new("refresh-from-mux", None);
        refresh_action.connect_activate(move |_, _| {
            if let Some(controller) = weak.upgrade() {
                controller.flush_pending_refresh_scopes();
            }
        });
        self.window.add_action(&refresh_action);

        let pending_refresh_scopes = Arc::clone(&self.pending_refresh_scopes);
        let main_context = glib::MainContext::default();
        let window_weak = glib::SendWeakRef::from(self.window.downgrade());
        mux.subscribe(move |notification| {
            let scope = refresh_scope_for_notification(&notification);
            if scope != SnapshotRefreshScope::Ignore {
                pending_refresh_scopes
                    .lock()
                    .expect("refresh queue")
                    .push(scope);
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
            let mut pending = self.pending_refresh_scopes.lock().expect("refresh queue");
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

        self.rerender();
    }

    fn bind_shell_view(self: &Rc<Self>, shell: ShellView) {
        let ShellView {
            root,
            chrome_drag_handle,
            refresh_button,
            terminal_button,
            rail_toggle_button,
            rail_buttons,
            activity_header,
            metadata_header,
            inbox_header,
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
            refresh_button.connect_clicked(move |_| this.defer(|controller| controller.rerender()));
        }
        {
            let this = Rc::clone(self);
            terminal_button.connect_clicked(move |_| {
                this.defer(|controller| controller.toggle_terminal_sessions())
            });
        }
        {
            let this = Rc::clone(self);
            rail_toggle_button
                .connect_clicked(move |_| this.defer(|controller| controller.toggle_rail()));
        }
        for (workspace, button) in rail_buttons {
            let this = Rc::clone(self);
            button.connect_clicked(move |_| {
                let workspace = workspace.clone();
                this.defer(move |controller| controller.select_workspace(&workspace));
            });
        }
        {
            let this = Rc::clone(self);
            bind_header_swap(
                &activity_header,
                &metadata_header,
                "lower-swap",
                move || this.defer(|controller| controller.toggle_lower_panes()),
            );
        }
        {
            let this = Rc::clone(self);
            bind_header_swap(
                &metadata_header,
                &activity_header,
                "lower-swap",
                move || this.defer(|controller| controller.toggle_lower_panes()),
            );
        }
        {
            let this = Rc::clone(self);
            bind_header_swap(&inbox_header, &tasks_header, "side-swap", move || {
                this.defer(|controller| controller.toggle_side_panes())
            });
        }
        {
            let this = Rc::clone(self);
            bind_header_swap(&tasks_header, &inbox_header, "side-swap", move || {
                this.defer(|controller| controller.toggle_side_panes())
            });
        }

        self.window.set_content(Some(&root));
    }

    fn select_workspace(self: &Rc<Self>, workspace: &str) {
        if !self.runtime_online.get() {
            return;
        }
        let mux = Mux::get();
        mux.set_active_workspace(workspace);
        self.selected_workspace.replace(Some(workspace.to_string()));
        self.rerender();
    }

    fn toggle_rail(self: &Rc<Self>) {
        self.rail_collapsed.set(!self.rail_collapsed.get());
        self.rerender();
    }

    fn toggle_lower_panes(self: &Rc<Self>) {
        self.metadata_first.set(!self.metadata_first.get());
        self.rerender();
    }

    fn toggle_side_panes(self: &Rc<Self>) {
        self.tasks_first.set(!self.tasks_first.get());
        self.rerender();
    }

    fn install_window_actions(self: &Rc<Self>, app: &adw::Application) {
        self.install_action("launch-terminal", {
            let this = Rc::clone(self);
            move || this.toggle_terminal_sessions()
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

        app.set_accels_for_action("win.launch-terminal", &["<Meta>t"]);
        app.set_accels_for_action("win.set-status", &["<Meta><Shift>s"]);
        app.set_accels_for_action("win.clear-status", &["<Meta><Shift>x"]);
        app.set_accels_for_action("win.set-progress", &["<Meta><Shift>p"]);
        app.set_accels_for_action("win.clear-progress", &["<Meta><Shift>r"]);
        app.set_accels_for_action("win.append-log", &["<Meta><Shift>l"]);
    }

    fn install_action(self: &Rc<Self>, name: &str, handler: impl Fn() + 'static) {
        let action = SimpleAction::new(name, None);
        action.connect_activate(move |_, _| handler());
        self.window.add_action(&action);
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

    fn toggle_terminal_sessions(self: &Rc<Self>) {
        self.show_terminal_sessions
            .set(!self.show_terminal_sessions.get());
        self.runtime_error.replace(None);
        self.rerender();
    }

    fn selected_workspace_name(&self) -> String {
        self.selected_workspace
            .borrow()
            .clone()
            .unwrap_or_else(|| "default".to_string())
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
                pending_refresh_scopes
                    .lock()
                    .expect("refresh queue")
                    .push(SnapshotRefreshScope::WorkspaceContext);
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
