use crate::actions::{ShellAction, ShellActionContext, ShellActionOutcome};
use crate::runtime_bridge::{bootstrap_native_shell_runtime, NativeShellBootstrapPlan};
use crate::snapshot::{
    derive_runtime_snapshot, refresh_scope_for_notification, RuntimeSnapshot, ShellLayoutContract,
    SnapshotRefreshScope, WorkspaceSummary,
};
use adw::prelude::*;
use gio::SimpleAction;
use gtk::gdk;
use gtk::prelude::IsA;
use gtk::{Align, Orientation, PolicyType};
use mux::notification_store::NotificationRecord;
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::WorkspaceLogRecord;
use mux::Mux;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AppController {
    pub window: adw::ApplicationWindow,
    selected_workspace: RefCell<Option<String>>,
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

        Rc::new(Self {
            window,
            selected_workspace: RefCell::new(None),
            pending_refresh_scopes: Arc::new(Mutex::new(Vec::new())),
            runtime_error: RefCell::new(None),
            runtime_online: Cell::new(false),
        })
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
        let root = self.build_shell(snapshot);
        self.window.set_content(Some(&root));
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
                pending_refresh_scopes.lock().expect("refresh queue").push(scope);
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

    fn build_shell(self: &Rc<Self>, snapshot: RuntimeSnapshot) -> gtk::Box {
        let layout = ShellLayoutContract::default();
        let root = gtk::Box::new(Orientation::Vertical, 0);
        root.add_css_class("shell-root");

        let chrome = self.build_chrome(&snapshot);
        chrome.set_height_request(layout.chrome_height);
        root.append(&chrome);

        let body = gtk::Box::new(Orientation::Horizontal, 0);
        body.set_vexpand(true);
        body.add_css_class("shell-body");

        let rail = self.build_rail(&snapshot);
        rail.set_width_request(layout.rail_width);
        body.append(&rail);

        let main_area = self.build_main_area(&snapshot);
        body.append(&main_area);

        root.append(&body);
        root
    }

    fn build_chrome(self: &Rc<Self>, snapshot: &RuntimeSnapshot) -> gtk::Box {
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
            "{} workspaces  {} unread  {} tasks",
            snapshot.workspaces.len(),
            snapshot.notifications.iter().filter(|row| row.unread).count(),
            snapshot.task_panes.len()
        )));
        subtitle.set_halign(Align::Start);
        subtitle.add_css_class("chrome-subtitle");
        title_box.append(&title);
        title_box.append(&subtitle);

        let spacer = gtk::Box::new(Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        let active = gtk::Label::new(Some(&format!("active: {}", snapshot.active_workspace)));
        active.add_css_class("chrome-pill");
        let refresh = action_button("Refresh");
        {
            let this = Rc::clone(self);
            refresh.connect_clicked(move |_| this.rerender());
        }
        let terminal = action_button("Launch Terminal");
        {
            let this = Rc::clone(self);
            terminal.connect_clicked(move |_| this.launch_terminal_window());
        }

        chrome.append(&title_box);
        chrome.append(&spacer);
        chrome.append(&active);
        chrome.append(&refresh);
        chrome.append(&terminal);
        chrome
    }

    fn build_rail(self: &Rc<Self>, snapshot: &RuntimeSnapshot) -> gtk::ScrolledWindow {
        let rail = gtk::Box::new(Orientation::Vertical, 6);
        rail.add_css_class("workspace-rail");
        rail.set_margin_start(12);
        rail.set_margin_end(10);
        rail.set_margin_top(16);
        rail.set_margin_bottom(16);

        let eyebrow = gtk::Label::new(Some("WORKSPACES"));
        eyebrow.set_halign(Align::Start);
        eyebrow.add_css_class("rail-eyebrow");
        rail.append(&eyebrow);

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
            let meta = gtk::Label::new(Some(&format!(
                "{} unread  {} run  {} fail",
                summary.unread_count, summary.running_count, summary.failed_count
            )));
            meta.set_halign(Align::Start);
            meta.add_css_class("rail-meta");
            name_box.append(&name);
            name_box.append(&meta);

            let mut status_bits = Vec::new();
            if let Some(status) = &summary.status {
                status_bits.push(status.clone());
            }
            if let Some(progress) = summary.progress {
                status_bits.push(format!("{progress}%"));
            }
            if summary.log_count > 0 {
                status_bits.push(format!("{} logs", summary.log_count));
            }

            let trailing = gtk::Label::new(Some(&status_bits.join(" · ")));
            trailing.set_halign(Align::End);
            trailing.add_css_class("rail-trailing");

            outer.append(&name_box);
            outer.append(&trailing);
            row.set_child(Some(&outer));

            let this = Rc::clone(self);
            let workspace = summary.name.clone();
            row.connect_clicked(move |_| this.select_workspace(&workspace));
            rail.append(&row);
        }

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
            .child(&rail)
            .build();
        scroller.add_css_class("rail-scroll");
        scroller
    }

    fn build_main_area(self: &Rc<Self>, snapshot: &RuntimeSnapshot) -> gtk::Paned {
        let center_column = gtk::Paned::new(Orientation::Vertical);
        center_column.set_position(430);

        let side_column = gtk::Paned::new(Orientation::Vertical);
        side_column.set_position(320);

        let outer = gtk::Paned::new(Orientation::Horizontal);
        outer.set_position(910);

        let top_center = self.build_primary_workspace_panel(snapshot);
        let lower_center = gtk::Paned::new(Orientation::Horizontal);
        lower_center.set_position(560);
        lower_center.set_start_child(Some(&self.build_activity_panel(snapshot)));
        lower_center.set_end_child(Some(&self.build_metadata_panel(snapshot)));

        center_column.set_start_child(Some(&top_center));
        center_column.set_end_child(Some(&lower_center));

        side_column.set_start_child(Some(&self.build_inbox_panel(snapshot)));
        side_column.set_end_child(Some(&self.build_task_panel(snapshot)));

        outer.set_start_child(Some(&center_column));
        outer.set_end_child(Some(&side_column));
        outer
    }

    fn build_primary_workspace_panel(self: &Rc<Self>, snapshot: &RuntimeSnapshot) -> gtk::Box {
        let workspace = self.current_workspace_summary(snapshot);
        let panel = pane_panel("Workspace", Some("Active Kaku workspace state"));
        let body = gtk::Box::new(Orientation::Vertical, 14);
        body.add_css_class("pane-body");
        body.set_margin_start(18);
        body.set_margin_end(18);
        body.set_margin_top(18);
        body.set_margin_bottom(18);

        let title = gtk::Label::new(Some(&workspace.name));
        title.set_halign(Align::Start);
        title.add_css_class("workspace-title");

        let summary = gtk::Label::new(Some(&format!(
            "{} unread  {} running  {} failed  {} logs",
            workspace.unread_count, workspace.running_count, workspace.failed_count, workspace.log_count
        )));
        summary.set_halign(Align::Start);
        summary.add_css_class("workspace-summary");

        let state_row = gtk::Box::new(Orientation::Horizontal, 10);
        state_row.set_halign(Align::Start);
        if let Some(status) = &workspace.status {
            state_row.append(&pill(&format!("status {}", status)));
        } else {
            state_row.append(&pill("status unset"));
        }
        if let Some(progress) = workspace.progress {
            state_row.append(&pill(&format!("progress {}%", progress)));
        } else {
            state_row.append(&pill("progress unset"));
        }

        let actions = gtk::Box::new(Orientation::Horizontal, 8);
        actions.set_halign(Align::Start);
        let launch = action_button("Launch Terminal");
        {
            let this = Rc::clone(self);
            launch.connect_clicked(move |_| this.launch_terminal_window());
        }
        let set_status = action_button("Set Status");
        {
            let this = Rc::clone(self);
            set_status.connect_clicked(move |_| this.set_status_for_selected());
        }
        let clear_status = action_button("Clear Status");
        {
            let this = Rc::clone(self);
            clear_status.connect_clicked(move |_| this.clear_status_for_selected());
        }
        let set_progress = action_button("Set Progress");
        {
            let this = Rc::clone(self);
            set_progress.connect_clicked(move |_| this.set_progress_for_selected());
        }
        let clear_progress = action_button("Clear Progress");
        {
            let this = Rc::clone(self);
            clear_progress.connect_clicked(move |_| this.clear_progress_for_selected());
        }
        let append_log = action_button("Append Log");
        {
            let this = Rc::clone(self);
            append_log.connect_clicked(move |_| this.append_log_for_selected());
        }
        actions.append(&launch);
        actions.append(&set_status);
        actions.append(&clear_status);
        actions.append(&set_progress);
        actions.append(&clear_progress);
        actions.append(&append_log);

        let hint = gtk::Label::new(Some(
            "Use the left rail to switch workspaces. Buttons above mutate real mux-backed state and refresh the surrounding panes immediately.",
        ));
        hint.set_wrap(true);
        hint.set_halign(Align::Start);
        hint.add_css_class("workspace-hint");

        body.append(&title);
        body.append(&summary);
        body.append(&state_row);
        body.append(&actions);
        body.append(&hint);
        panel.append(&body);
        panel
    }

    fn build_inbox_panel(self: &Rc<Self>, snapshot: &RuntimeSnapshot) -> gtk::Box {
        let panel = pane_panel("Inbox", Some("Unread notifications"));
        let selected = snapshot.active_workspace.as_str();
        let unread = snapshot
            .notifications
            .iter()
            .filter(|row| row.workspace == selected && row.unread)
            .cloned()
            .collect::<Vec<_>>();

        let header_actions = gtk::Box::new(Orientation::Horizontal, 8);
        header_actions.set_margin_start(14);
        header_actions.set_margin_end(14);
        header_actions.set_margin_top(12);
        let mark_read = action_button("Mark Visible Read");
        {
            let this = Rc::clone(self);
            mark_read.connect_clicked(move |_| this.mark_notifications_read_for_selected());
        }
        header_actions.append(&mark_read);
        panel.append(&header_actions);

        if unread.is_empty() {
            panel.append(&empty_state("No unread notifications in this workspace."));
            return panel;
        }

        let list = gtk::Box::new(Orientation::Vertical, 8);
        list.set_margin_start(14);
        list.set_margin_end(14);
        list.set_margin_top(6);
        list.set_margin_bottom(14);
        for row in unread.iter().take(8) {
            list.append(&notification_row(row));
        }
        panel.append(&scroller(&list));
        panel
    }

    fn build_task_panel(&self, snapshot: &RuntimeSnapshot) -> gtk::Box {
        let panel = pane_panel("Tasks", Some("Running and failed task panes"));
        let selected = snapshot.active_workspace.as_str();
        let tasks = snapshot
            .task_panes
            .iter()
            .filter(|row| row.workspace.as_deref() == Some(selected))
            .cloned()
            .collect::<Vec<_>>();

        if tasks.is_empty() {
            panel.append(&empty_state("No task panes recorded for this workspace."));
            return panel;
        }

        let list = gtk::Box::new(Orientation::Vertical, 8);
        list.set_margin_start(14);
        list.set_margin_end(14);
        list.set_margin_top(12);
        list.set_margin_bottom(14);
        for row in tasks.iter().take(10) {
            list.append(&task_row(row));
        }
        panel.append(&scroller(&list));
        panel
    }

    fn build_activity_panel(&self, snapshot: &RuntimeSnapshot) -> gtk::Box {
        let panel = pane_panel("Activity", Some("Recent workspace log"));
        if snapshot.logs.is_empty() {
            panel.append(&empty_state("No workspace log entries yet."));
            return panel;
        }

        let list = gtk::Box::new(Orientation::Vertical, 8);
        list.set_margin_start(14);
        list.set_margin_end(14);
        list.set_margin_top(12);
        list.set_margin_bottom(14);
        for row in snapshot.logs.iter().rev().take(12) {
            list.append(&log_row(row));
        }
        panel.append(&scroller(&list));
        panel
    }

    fn build_metadata_panel(&self, snapshot: &RuntimeSnapshot) -> gtk::Box {
        let panel = pane_panel("Metadata", Some("Selected workspace control-plane state"));
        let selected = snapshot.active_workspace.as_str();
        let status = snapshot
            .statuses
            .iter()
            .find(|row| row.workspace == selected)
            .map(|row| row.status.clone());
        let progress = snapshot
            .progresses
            .iter()
            .find(|row| row.workspace == selected)
            .map(|row| row.value);

        let body = gtk::Box::new(Orientation::Vertical, 10);
        body.set_margin_start(14);
        body.set_margin_end(14);
        body.set_margin_top(12);
        body.set_margin_bottom(14);

        body.append(&info_row("Workspace", selected));
        body.append(&info_row("Status", status.as_deref().unwrap_or("unset")));
        body.append(&info_row(
            "Progress",
            &progress
                .map(|value| format!("{value}%"))
                .unwrap_or_else(|| "unset".to_string()),
        ));
        body.append(&info_row(
            "Unread",
            &snapshot
                .notifications
                .iter()
                .filter(|row| row.workspace == selected && row.unread)
                .count()
                .to_string(),
        ));
        body.append(&info_row(
            "Tasks",
            &snapshot
                .task_panes
                .iter()
                .filter(|row| row.workspace.as_deref() == Some(selected))
                .count()
                .to_string(),
        ));
        panel.append(&body);
        panel
    }

    fn current_workspace_summary<'a>(&self, snapshot: &'a RuntimeSnapshot) -> &'a WorkspaceSummary {
        snapshot
            .workspaces
            .iter()
            .find(|summary| summary.name == snapshot.active_workspace)
            .or_else(|| snapshot.workspaces.first())
            .expect("at least one workspace summary")
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

    fn mark_notifications_read_for_selected(self: &Rc<Self>) {
        if !self.runtime_online.get() {
            return;
        }
        self.apply_action(ShellAction::MarkVisibleRead);
    }

    fn launch_terminal_window(self: &Rc<Self>) {
        let Some(path) = sibling_binary("kaku-gui") else {
            self.runtime_error
                .replace(Some("unable to locate sibling kaku-gui binary".to_string()));
            self.rerender();
            return;
        };

        match Command::new(path)
            .arg("start")
            .arg("--always-new-process")
            .spawn()
        {
            Ok(_) => {
                self.runtime_error.replace(None);
            }
            Err(err) => {
                self.runtime_error
                    .replace(Some(format!("failed to launch terminal window: {err}")));
            }
        }
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
        let mux = Mux::get();
        let context = self.action_context(mux.as_ref());
        if matches!(action.execute(mux.as_ref(), &context), ShellActionOutcome::NoMutation)
            && matches!(action, ShellAction::Refresh)
        {
            self.rerender();
        }
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

fn action_button(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("action-button");
    button
}

fn pane_panel(title: &str, subtitle: Option<&str>) -> gtk::Box {
    let panel = gtk::Box::new(Orientation::Vertical, 0);
    panel.add_css_class("pane-panel");

    let header = gtk::Box::new(Orientation::Vertical, 3);
    header.add_css_class("pane-header");
    header.set_margin_start(14);
    header.set_margin_end(14);
    header.set_margin_top(12);
    header.set_margin_bottom(10);
    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("pane-title");
    header.append(&title_label);
    if let Some(subtitle) = subtitle {
        let subtitle_label = gtk::Label::new(Some(subtitle));
        subtitle_label.set_halign(Align::Start);
        subtitle_label.add_css_class("pane-subtitle");
        header.append(&subtitle_label);
    }
    panel.append(&header);
    panel
}

fn empty_state(message: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(message));
    label.set_margin_start(14);
    label.set_margin_end(14);
    label.set_margin_top(12);
    label.set_margin_bottom(14);
    label.set_wrap(true);
    label.set_halign(Align::Start);
    label.add_css_class("empty-state");
    label
}

fn scroller(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(child)
        .build()
}

fn pill(label: &str) -> gtk::Label {
    let pill = gtk::Label::new(Some(label));
    pill.add_css_class("chrome-pill");
    pill
}

fn notification_row(row: &NotificationRecord) -> gtk::Box {
    let outer = gtk::Box::new(Orientation::Vertical, 4);
    outer.add_css_class("list-row");

    let title = gtk::Label::new(Some(&row.title));
    title.set_halign(Align::Start);
    title.add_css_class("row-title");
    let detail = gtk::Label::new(Some(&format!(
        "{} · {}",
        row.kind,
        row.body.as_deref().unwrap_or("no body")
    )));
    detail.set_halign(Align::Start);
    detail.set_wrap(true);
    detail.add_css_class("row-detail");
    outer.append(&title);
    outer.append(&detail);
    outer
}

fn task_row(row: &TaskPaneRecord) -> gtk::Box {
    let outer = gtk::Box::new(Orientation::Vertical, 4);
    outer.add_css_class("list-row");
    let title = gtk::Label::new(Some(&format!("pane {}", row.pane_id)));
    title.set_halign(Align::Start);
    title.add_css_class("row-title");
    let status = if row.is_failed {
        "failed"
    } else if row.is_dead {
        "dead"
    } else {
        "running"
    };
    let detail = gtk::Label::new(Some(&format!(
        "{} · remain:{} · silent:{}",
        status, row.remain_on_exit, row.silenced
    )));
    detail.set_halign(Align::Start);
    detail.add_css_class("row-detail");
    outer.append(&title);
    outer.append(&detail);
    if let Some(cwd) = &row.current_working_dir {
        let cwd_label = gtk::Label::new(Some(cwd));
        cwd_label.set_halign(Align::Start);
        cwd_label.add_css_class("row-detail");
        outer.append(&cwd_label);
    }
    outer
}

fn log_row(row: &WorkspaceLogRecord) -> gtk::Box {
    let outer = gtk::Box::new(Orientation::Vertical, 4);
    outer.add_css_class("list-row");
    let title = gtk::Label::new(Some(&format!("log #{}", row.seq)));
    title.set_halign(Align::Start);
    title.add_css_class("row-title");
    let detail = gtk::Label::new(Some(&row.message));
    detail.set_halign(Align::Start);
    detail.set_wrap(true);
    detail.add_css_class("row-detail");
    outer.append(&title);
    outer.append(&detail);
    outer
}

fn info_row(label: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::new(Orientation::Vertical, 3);
    row.add_css_class("info-row");
    let label_widget = gtk::Label::new(Some(label));
    label_widget.set_halign(Align::Start);
    label_widget.add_css_class("info-label");
    let value_widget = gtk::Label::new(Some(value));
    value_widget.set_halign(Align::Start);
    value_widget.add_css_class("info-value");
    row.append(&label_widget);
    row.append(&value_widget);
    row
}

fn sibling_binary(name: &str) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    Some(exe.with_file_name(name))
}

pub fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .native-shell-window, .shell-root, window, box {
          font-family: 'DM Mono', 'SF Mono', monospace;
        }

        .shell-root {
          background: #111116;
          color: #d8d8dc;
        }

        .shell-chrome {
          background: #17171d;
          border-bottom: 1px solid rgba(255,255,255,0.06);
        }

        .chrome-title {
          font-size: 14px;
          font-weight: 700;
          color: #f2f2f5;
        }

        .chrome-subtitle, .pane-subtitle, .rail-eyebrow, .rail-meta, .row-detail, .workspace-summary, .workspace-hint, .info-label, .empty-state {
          font-size: 11px;
          color: #8d8e99;
        }

        .chrome-pill {
          padding: 4px 8px;
          background: #202029;
          border: 1px solid rgba(255,255,255,0.08);
          border-radius: 999px;
          color: #d8d8dc;
        }

        .workspace-rail {
          background: #15161c;
          border-right: 1px solid rgba(255,255,255,0.06);
        }

        .rail-row {
          padding: 10px 12px;
          border-radius: 10px;
          background: transparent;
          border: 1px solid transparent;
        }

        .rail-row.active {
          background: #222530;
          border-color: rgba(122, 161, 255, 0.18);
        }

        .rail-name, .pane-title, .row-title, .workspace-title, .info-value {
          font-size: 13px;
          font-weight: 600;
          color: #f1f1f5;
        }

        .rail-trailing {
          font-size: 10px;
          color: #7aa1ff;
        }

        .pane-panel {
          background: #15161c;
          border: 1px solid rgba(255,255,255,0.06);
          border-radius: 0;
        }

        .pane-header {
          border-bottom: 1px solid rgba(255,255,255,0.05);
          background: #171920;
        }

        .pane-body {
          background: #13141a;
        }

        .workspace-title {
          font-size: 18px;
        }

        .action-button {
          background: #202532;
          color: #e8e9ef;
          border-radius: 10px;
          border: 1px solid rgba(255,255,255,0.08);
          padding: 6px 10px;
        }

        .list-row, .info-row {
          padding: 8px 0;
          border-bottom: 1px solid rgba(255,255,255,0.04);
        }
        ",
    );

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
