use adw::prelude::*;
use gtk::gdk;
use kaku_runtime::bootstrap_gui_runtime;
use mux::notification_store::NotificationRecord;
use mux::task_panes::TaskPaneRecord;
use mux::workspace_state::{WorkspaceLogRecord, WorkspaceProgressRecord, WorkspaceStatusRecord};
use mux::Mux;
use std::collections::HashMap;
use wezterm_gui_subcommands::DEFAULT_WINDOW_CLASS;

const APP_ID: &str = "dev.tw93.kaku.NativeShell";

struct RuntimeBootState {
    title: String,
    status_chip: String,
    main_copy: String,
    workspaces: Vec<WorkspaceSummary>,
    active_workspace: String,
    inbox: Vec<String>,
    failures: Vec<String>,
    activity: Vec<String>,
}

#[derive(Clone)]
struct WorkspaceSummary {
    name: String,
    unread_count: usize,
    running_count: usize,
    failed_count: usize,
    status: Option<String>,
    progress: Option<u8>,
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| install_css());
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let runtime = bootstrap_runtime_state();
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Kaku Native Shell")
        .default_width(1480)
        .default_height(920)
        .build();

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("native-shell-root");
    root.append(&build_top_chrome(&runtime));
    root.append(&build_content(&runtime));
    window.set_content(Some(&root));
    window.present();
}

fn bootstrap_runtime_state() -> RuntimeBootState {
    match bootstrap_gui_runtime(DEFAULT_WINDOW_CLASS, None, Some("default"), true) {
        Ok(mux) => runtime_boot_state_from_mux(&mux),
        Err(err) => RuntimeBootState {
            title: "runtime bootstrap failed".to_string(),
            status_chip: "runtime error".to_string(),
            main_copy: format!(
                "The native shell window loaded, but Kaku runtime bootstrap failed:\n\n{err:#}"
            ),
            workspaces: vec![],
            active_workspace: "default".to_string(),
            inbox: vec![],
            failures: vec![],
            activity: vec![],
        },
    }
}

fn runtime_boot_state_from_mux(mux: &Mux) -> RuntimeBootState {
    let active_workspace = mux.active_workspace();
    let workspaces = mux.iter_workspaces();
    let statuses = mux
        .list_workspace_status()
        .into_iter()
        .map(|record| (record.workspace.clone(), record))
        .collect::<HashMap<_, _>>();
    let progresses = mux
        .list_workspace_progress()
        .into_iter()
        .map(|record| (record.workspace.clone(), record))
        .collect::<HashMap<_, _>>();
    let notifications = mux.list_notifications();
    let task_panes = mux.list_task_panes();
    let logs = mux.list_workspace_log(&active_workspace);

    let workspace_summaries = build_workspace_summaries(
        &workspaces,
        &statuses,
        &progresses,
        &notifications,
        &task_panes,
    );

    let unread_total = notifications.iter().filter(|record| record.unread).count();
    let running_total = task_panes.iter().filter(|record| !record.is_dead).count();
    let failed_total = task_panes.iter().filter(|record| record.is_failed).count();

    RuntimeBootState {
        title: format!("{active_workspace} workspace"),
        status_chip: format!(
            "{running_total} running  {failed_total} failed  {unread_total} inbox"
        ),
        main_copy: active_workspace_summary_copy(
            &active_workspace,
            statuses.get(&active_workspace),
            progresses.get(&active_workspace),
            &task_panes,
            &notifications,
        ),
        workspaces: workspace_summaries,
        active_workspace,
        inbox: notifications
            .iter()
            .filter(|record| record.unread)
            .take(4)
            .map(format_notification_row)
            .collect(),
        failures: task_panes
            .iter()
            .filter(|record| record.is_failed || !record.is_dead)
            .take(4)
            .map(format_task_row)
            .collect(),
        activity: logs.into_iter().rev().take(5).map(format_log_row).collect(),
    }
}

fn build_workspace_summaries(
    workspaces: &[String],
    statuses: &HashMap<String, WorkspaceStatusRecord>,
    progresses: &HashMap<String, WorkspaceProgressRecord>,
    notifications: &[NotificationRecord],
    task_panes: &[TaskPaneRecord],
) -> Vec<WorkspaceSummary> {
    let mut summaries = workspaces
        .iter()
        .map(|name| WorkspaceSummary {
            name: name.clone(),
            unread_count: notifications
                .iter()
                .filter(|record| record.workspace == *name && record.unread)
                .count(),
            running_count: task_panes
                .iter()
                .filter(|record| {
                    record.workspace.as_deref() == Some(name.as_str()) && !record.is_dead
                })
                .count(),
            failed_count: task_panes
                .iter()
                .filter(|record| {
                    record.workspace.as_deref() == Some(name.as_str()) && record.is_failed
                })
                .count(),
            status: statuses.get(name).map(|record| record.status.clone()),
            progress: progresses.get(name).map(|record| record.value),
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|left, right| left.name.cmp(&right.name));
    summaries
}

fn active_workspace_summary_copy(
    workspace: &str,
    status: Option<&WorkspaceStatusRecord>,
    progress: Option<&WorkspaceProgressRecord>,
    task_panes: &[TaskPaneRecord],
    notifications: &[NotificationRecord],
) -> String {
    let unread = notifications
        .iter()
        .filter(|record| record.workspace == workspace && record.unread)
        .count();
    let running = task_panes
        .iter()
        .filter(|record| record.workspace.as_deref() == Some(workspace) && !record.is_dead)
        .count();
    let failed = task_panes
        .iter()
        .filter(|record| record.workspace.as_deref() == Some(workspace) && record.is_failed)
        .count();

    let mut parts = vec![format!("workspace: {workspace}")];
    if let Some(status) = status {
        parts.push(format!("status: {}", status.status));
    }
    if let Some(progress) = progress {
        parts.push(format!("progress: {}%", progress.value));
    }
    parts.push(format!("running tasks: {running}"));
    parts.push(format!("failed tasks: {failed}"));
    parts.push(format!("unread notifications: {unread}"));
    parts.join("\n")
}

fn format_notification_row(record: &NotificationRecord) -> String {
    let body = record.body.clone().unwrap_or_default();
    if body.is_empty() {
        format!("{} · {}", record.workspace, record.title)
    } else {
        format!("{} · {} — {}", record.workspace, record.title, body)
    }
}

fn format_task_row(record: &TaskPaneRecord) -> String {
    let workspace = record
        .workspace
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let state = if record.is_failed {
        "failed"
    } else if record.is_dead {
        "idle"
    } else {
        "running"
    };
    format!("{workspace} · pane {} · {}", record.pane_id, state)
}

fn format_log_row(record: WorkspaceLogRecord) -> String {
    format!("#{} {}", record.seq, record.message)
}

fn build_top_chrome(runtime: &RuntimeBootState) -> gtk::Box {
    let chrome = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    chrome.add_css_class("native-shell-chrome");

    let title_stack = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let eyebrow = gtk::Label::new(Some("KAKU"));
    eyebrow.add_css_class("native-shell-eyebrow");
    eyebrow.set_xalign(0.0);

    let title = gtk::Label::new(Some(&runtime.title));
    title.add_css_class("native-shell-title");
    title.set_xalign(0.0);

    title_stack.append(&eyebrow);
    title_stack.append(&title);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let status_chip = gtk::Label::new(Some(&runtime.status_chip));
    status_chip.add_css_class("native-shell-status-chip");

    chrome.append(&title_stack);
    chrome.append(&spacer);
    chrome.append(&status_chip);
    chrome
}

fn build_content(runtime: &RuntimeBootState) -> gtk::Widget {
    let rail = build_rail(runtime);

    let workspace_shell = build_workspace_shell(runtime);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.add_css_class("native-shell-body");
    body.append(&rail);
    body.append(&workspace_shell);
    body.upcast()
}

fn build_rail(runtime: &RuntimeBootState) -> gtk::Box {
    let rail = gtk::Box::new(gtk::Orientation::Vertical, 10);
    rail.add_css_class("native-shell-rail");

    let workspace_header = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let title = gtk::Label::new(Some("KAKU"));
    title.add_css_class("native-shell-rail-title");
    title.set_xalign(0.0);

    let subtitle = gtk::Label::new(Some("WORKSPACES"));
    subtitle.add_css_class("native-shell-rail-subtitle");
    subtitle.set_xalign(0.0);

    workspace_header.append(&title);
    workspace_header.append(&subtitle);

    let nav = gtk::ListBox::new();
    nav.add_css_class("native-shell-nav");
    nav.set_selection_mode(gtk::SelectionMode::Single);
    for workspace in &runtime.workspaces {
        let mut badge_parts = vec![];
        if workspace.unread_count > 0 {
            badge_parts.push(format!("i{}", workspace.unread_count));
        }
        if workspace.running_count > 0 {
            badge_parts.push(format!("r{}", workspace.running_count));
        }
        if workspace.failed_count > 0 {
            badge_parts.push(format!("f{}", workspace.failed_count));
        }
        let badge = if badge_parts.is_empty() {
            None
        } else {
            Some(badge_parts.join(" "))
        };
        nav.append(&nav_row(
            &workspace.name,
            badge.as_deref(),
            workspace.name == runtime.active_workspace,
        ));
    }

    let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
    spacer.set_vexpand(true);

    rail.append(&workspace_header);
    rail.append(&nav);
    rail.append(&spacer);
    rail
}

fn nav_row(label: &str, count: Option<&str>, active: bool) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.add_css_class("native-shell-nav-row");
    if active {
        row.add_css_class("is-active");
    }

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    content.add_css_class("native-shell-nav-row-content");

    let dot = gtk::Label::new(Some(if active { "◆" } else { "•" }));
    dot.add_css_class("native-shell-nav-dot");

    let text = gtk::Label::new(Some(label));
    text.add_css_class("native-shell-nav-label");
    text.set_xalign(0.0);
    text.set_hexpand(true);

    content.append(&dot);
    content.append(&text);
    if let Some(count) = count {
        let badge = gtk::Label::new(Some(count));
        badge.add_css_class("native-shell-nav-badge");
        content.append(&badge);
    }

    row.set_child(Some(&content));
    row
}

fn build_workspace_shell(runtime: &RuntimeBootState) -> gtk::Widget {
    let shell = gtk::Paned::new(gtk::Orientation::Horizontal);
    shell.set_wide_handle(false);
    shell.set_position(930);
    shell.set_start_child(Some(&build_primary_column(runtime)));
    shell.set_end_child(Some(&build_secondary_column(runtime)));
    shell.upcast()
}

fn build_primary_column(runtime: &RuntimeBootState) -> gtk::Widget {
    let column = gtk::Paned::new(gtk::Orientation::Vertical);
    column.set_wide_handle(false);
    column.set_position(470);
    column.set_start_child(Some(&build_terminal_pane(runtime)));
    column.set_end_child(Some(&build_lower_primary_row(runtime)));
    column.upcast()
}

fn build_lower_primary_row(runtime: &RuntimeBootState) -> gtk::Widget {
    let row = gtk::Paned::new(gtk::Orientation::Horizontal);
    row.set_wide_handle(false);
    row.set_position(520);
    row.set_start_child(Some(&build_panel(
        "activity",
        Some("workspace log"),
        &runtime.activity,
        "No workspace log entries yet",
        "native-shell-subpane",
    )));
    row.set_end_child(Some(&build_panel(
        "task center",
        Some("unread + failed"),
        &runtime.failures,
        "No running or failed task panes",
        "native-shell-subpane",
    )));
    row.upcast()
}

fn build_secondary_column(runtime: &RuntimeBootState) -> gtk::Widget {
    let column = gtk::Paned::new(gtk::Orientation::Vertical);
    column.set_wide_handle(false);
    column.set_position(420);
    column.set_start_child(Some(&build_panel(
        "context",
        Some("workspace + inbox"),
        &build_context_rows(runtime),
        "No unread notifications",
        "native-shell-browserpane",
    )));
    column.set_end_child(Some(&build_panel(
        "selection",
        Some("operator detail"),
        &build_selection_rows(runtime),
        "No selection details yet",
        "native-shell-subpane",
    )));
    column.upcast()
}

fn build_terminal_pane(runtime: &RuntimeBootState) -> gtk::Widget {
    let lines = runtime
        .main_copy
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    build_panel(
        "workspace",
        Some(&runtime.active_workspace),
        &lines,
        "No workspace summary available",
        "native-shell-mainpane",
    )
}

fn build_context_rows(runtime: &RuntimeBootState) -> Vec<String> {
    let mut rows = vec![];
    rows.extend(runtime.inbox.iter().cloned());
    let active = runtime
        .workspaces
        .iter()
        .find(|workspace| workspace.name == runtime.active_workspace);
    if let Some(active) = active {
        if let Some(status) = &active.status {
            rows.push(format!("status  {status}"));
        }
        if let Some(progress) = active.progress {
            rows.push(format!("progress  {progress}%"));
        }
    }
    rows
}

fn build_selection_rows(runtime: &RuntimeBootState) -> Vec<String> {
    let mut rows = vec![format!("active workspace  {}", runtime.active_workspace)];
    rows.extend(
        runtime
            .workspaces
            .iter()
            .map(|workspace| {
                format!(
                    "{}  unread:{} run:{} fail:{}",
                    workspace.name,
                    workspace.unread_count,
                    workspace.running_count,
                    workspace.failed_count
                )
            })
            .take(6),
    );
    rows
}

fn build_panel(
    title: &str,
    subtitle: Option<&str>,
    rows: &[String],
    empty: &str,
    panel_class: &str,
) -> gtk::Widget {
    let frame = gtk::Frame::new(None);
    frame.add_css_class("native-shell-pane");
    frame.add_css_class(panel_class);

    let shell = gtk::Box::new(gtk::Orientation::Vertical, 0);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header.add_css_class("native-shell-pane-header");

    let title_label = gtk::Label::new(Some(title));
    title_label.add_css_class("native-shell-pane-title");
    title_label.set_xalign(0.0);

    let subtitle_label = gtk::Label::new(subtitle);
    subtitle_label.add_css_class("native-shell-pane-subtitle");
    subtitle_label.set_xalign(0.0);

    let title_stack = gtk::Box::new(gtk::Orientation::Vertical, 1);
    title_stack.append(&title_label);
    if subtitle.is_some() {
        title_stack.append(&subtitle_label);
    }

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let controls = gtk::Label::new(Some("◦ ◦ ◦"));
    controls.add_css_class("native-shell-pane-controls");

    header.append(&title_stack);
    header.append(&spacer);
    header.append(&controls);

    let body = gtk::Box::new(gtk::Orientation::Vertical, 6);
    body.add_css_class("native-shell-pane-body");
    if rows.is_empty() {
        let line = gtk::Label::new(Some(empty));
        line.add_css_class("native-shell-panel-copy");
        line.set_xalign(0.0);
        line.set_wrap(true);
        body.append(&line);
    } else {
        for row in rows {
            let line = gtk::Label::new(Some(row));
            line.add_css_class("native-shell-panel-copy");
            line.set_xalign(0.0);
            line.set_wrap(true);
            body.append(&line);
        }
    }

    shell.append(&header);
    shell.append(&body);
    frame.set_child(Some(&shell));
    frame.upcast()
}

fn install_css() {
    let Some(display) = gdk::Display::default() else {
        return;
    };

    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        r#"
        window {
          background: #121116;
          color: #f2f2f0;
          font-family: "DM Mono", "SF Mono", monospace;
        }

        .native-shell-root {
          background: #121116;
        }

        .native-shell-chrome {
          min-height: 34px;
          padding: 6px 12px 5px 12px;
          border-bottom: 1px solid rgba(255,255,255,0.05);
          background: #18181d;
        }

        .native-shell-eyebrow,
        .native-shell-rail-subtitle {
          color: rgba(255,255,255,0.30);
          font-size: 9px;
          letter-spacing: 0.10em;
        }

        .native-shell-title,
        .native-shell-rail-title {
          color: #f2f2f0;
          font-size: 11px;
          font-weight: 500;
        }

        .native-shell-status-chip {
          padding: 2px 7px;
          border-radius: 4px;
          color: #b7b6b2;
          background: rgba(255,255,255,0.03);
          font-size: 10px;
        }

        .native-shell-body {
          background: #121116;
        }

        .native-shell-rail {
          min-width: 176px;
          padding: 10px 10px 8px 10px;
          border-right: 1px solid rgba(255,255,255,0.04);
          background: #17181a;
        }

        .native-shell-nav {
          background: transparent;
        }

        .native-shell-nav row {
          background: transparent;
          min-height: 26px;
          border-radius: 0;
        }

        .native-shell-nav row.is-active {
          background: rgba(255,255,255,0.06);
        }

        .native-shell-nav-row-content {
          padding: 2px 6px;
        }

        .native-shell-nav-label,
        .native-shell-nav-dot,
        .native-shell-nav-badge {
          color: #c8c7c3;
          font-size: 11px;
        }

        .native-shell-pane {
          margin: 0;
          background: #111214;
          border-radius: 0;
          border: 1px solid rgba(255,255,255,0.04);
        }

        .native-shell-mainpane {
          background: #0d0e10;
        }

        .native-shell-browserpane,
        .native-shell-subpane {
          background: #111214;
        }

        .native-shell-pane-header {
          min-height: 28px;
          padding: 4px 8px;
          border-bottom: 1px solid rgba(255,255,255,0.04);
          background: #191a1d;
        }

        .native-shell-pane-title {
          color: #e8e7e3;
          font-size: 10px;
          font-weight: 500;
        }

        .native-shell-pane-subtitle,
        .native-shell-pane-controls {
          color: rgba(255,255,255,0.30);
          font-size: 9px;
        }

        .native-shell-pane-body {
          padding: 10px 10px 12px 10px;
          background: inherit;
        }

        .native-shell-panel-copy {
          color: rgba(255,255,255,0.70);
          font-size: 11px;
        }

        separator {
          background: rgba(255,255,255,0.05);
          min-width: 1px;
          min-height: 1px;
        }
        "#,
    );

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
