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

    let vertical = gtk::Paned::new(gtk::Orientation::Vertical);
    vertical.set_wide_handle(false);
    vertical.set_position(620);
    vertical.set_start_child(Some(&build_main_row(runtime)));
    vertical.set_end_child(Some(&build_bottom_panel(runtime)));

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.add_css_class("native-shell-body");
    body.append(&rail);
    body.append(&vertical);
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

fn build_main_row(runtime: &RuntimeBootState) -> gtk::Paned {
    let horizontal = gtk::Paned::new(gtk::Orientation::Horizontal);
    horizontal.set_wide_handle(false);
    horizontal.set_position(980);
    horizontal.set_start_child(Some(&build_main_surface(runtime)));
    horizontal.set_end_child(Some(&build_context_panel(runtime)));
    horizontal
}

fn build_main_surface(runtime: &RuntimeBootState) -> gtk::Widget {
    let shell = gtk::Box::new(gtk::Orientation::Vertical, 18);
    shell.add_css_class("native-shell-surface");
    shell.set_hexpand(true);
    shell.set_vexpand(true);

    let heading = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let title = gtk::Label::new(Some("Main work surface"));
    title.add_css_class("native-shell-surface-title");
    title.set_xalign(0.0);

    let subtitle = gtk::Label::new(Some(
        "This center pane becomes the new Kaku workspace shell.",
    ));
    subtitle.add_css_class("native-shell-surface-copy");
    subtitle.set_xalign(0.0);

    let heading_stack = gtk::Box::new(gtk::Orientation::Vertical, 4);
    heading_stack.append(&title);
    heading_stack.append(&subtitle);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let open_terminal = gtk::Button::with_label("Open terminal");
    open_terminal.add_css_class("suggested-action");

    heading.append(&heading_stack);
    heading.append(&spacer);
    heading.append(&open_terminal);

    let canvas = gtk::Frame::new(None);
    canvas.add_css_class("native-shell-canvas");
    canvas.set_hexpand(true);
    canvas.set_vexpand(true);

    let canvas_copy = gtk::Label::new(Some(&runtime.main_copy));
    canvas_copy.add_css_class("native-shell-canvas-copy");
    canvas_copy.set_wrap(true);
    canvas_copy.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    canvas_copy.set_justify(gtk::Justification::Center);
    canvas.set_child(Some(&canvas_copy));

    shell.append(&heading);
    shell.append(&canvas);
    shell.upcast()
}

fn build_context_panel(runtime: &RuntimeBootState) -> gtk::Widget {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
    panel.add_css_class("native-shell-panel");
    panel.append(&panel_card(
        "Inbox",
        &runtime.inbox,
        "No unread notifications",
    ));
    panel.append(&panel_card(
        "Tasks",
        &runtime.failures,
        "No running or failed task panes",
    ));
    let metadata_lines = runtime
        .workspaces
        .iter()
        .find(|workspace| workspace.name == runtime.active_workspace)
        .map(|workspace| {
            let mut rows = vec![];
            if let Some(status) = &workspace.status {
                rows.push(format!("status · {status}"));
            }
            if let Some(progress) = workspace.progress {
                rows.push(format!("progress · {progress}%"));
            }
            if rows.is_empty() {
                rows.push("No workspace metadata yet".to_string());
            }
            rows
        })
        .unwrap_or_else(|| vec!["No workspace metadata yet".to_string()]);
    panel.append(&panel_card(
        "Metadata",
        &metadata_lines,
        "No workspace metadata yet",
    ));
    panel.upcast()
}

fn build_bottom_panel(runtime: &RuntimeBootState) -> gtk::Widget {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 10);
    panel.add_css_class("native-shell-bottom");

    let title = gtk::Label::new(Some("Recent activity"));
    title.add_css_class("native-shell-panel-title");
    title.set_xalign(0.0);

    let frame = gtk::Frame::new(None);
    frame.add_css_class("native-shell-bottom-frame");

    let rows = gtk::Box::new(gtk::Orientation::Vertical, 6);
    if runtime.activity.is_empty() {
        let copy = gtk::Label::new(Some("No workspace log entries yet"));
        copy.add_css_class("native-shell-panel-copy");
        copy.set_xalign(0.0);
        rows.append(&copy);
    } else {
        for line in &runtime.activity {
            let copy = gtk::Label::new(Some(line));
            copy.add_css_class("native-shell-panel-copy");
            copy.set_wrap(true);
            copy.set_xalign(0.0);
            rows.append(&copy);
        }
    }
    frame.set_child(Some(&rows));

    panel.append(&title);
    panel.append(&frame);
    panel.upcast()
}

fn panel_card(title: &str, rows: &[String], empty: &str) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.add_css_class("native-shell-card");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("native-shell-panel-title");
    heading.set_xalign(0.0);

    content.append(&heading);
    if rows.is_empty() {
        let body = gtk::Label::new(Some(empty));
        body.add_css_class("native-shell-panel-copy");
        body.set_wrap(true);
        body.set_xalign(0.0);
        content.append(&body);
    } else {
        for row in rows {
            let body = gtk::Label::new(Some(row));
            body.add_css_class("native-shell-panel-copy");
            body.set_wrap(true);
            body.set_xalign(0.0);
            content.append(&body);
        }
    }
    frame.set_child(Some(&content));
    frame
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
          min-height: 56px;
          padding: 14px 18px 10px 18px;
          border-bottom: 1px solid rgba(255,255,255,0.06);
          background: #15141a;
        }

        .native-shell-eyebrow,
        .native-shell-rail-subtitle {
          color: rgba(255,255,255,0.42);
          font-size: 11px;
          letter-spacing: 0.12em;
        }

        .native-shell-title,
        .native-shell-rail-title {
          color: #f2f2f0;
          font-size: 14px;
          font-weight: 600;
        }

        .native-shell-status-chip {
          padding: 6px 10px;
          border-radius: 999px;
          color: #d9d8d4;
          background: rgba(255,255,255,0.05);
        }

        .native-shell-body {
          background: #121116;
        }

        .native-shell-rail {
          min-width: 118px;
          padding: 16px 10px 14px 12px;
          border-right: 1px solid rgba(255,255,255,0.04);
          background: #181820;
        }

        .native-shell-nav {
          background: transparent;
        }

        .native-shell-nav row {
          background: transparent;
          min-height: 30px;
          border-radius: 10px;
        }

        .native-shell-nav row.is-active {
          background: rgba(255,255,255,0.08);
        }

        .native-shell-nav-row-content {
          padding: 4px 8px;
        }

        .native-shell-nav-label,
        .native-shell-nav-dot,
        .native-shell-nav-badge {
          color: #dddcd9;
          font-size: 12px;
        }

        .native-shell-surface,
        .native-shell-panel,
        .native-shell-bottom {
          padding: 16px;
          background: #121116;
        }

        .native-shell-surface-title,
        .native-shell-panel-title {
          color: #f4f4f1;
          font-size: 13px;
          font-weight: 600;
        }

        .native-shell-surface-copy,
        .native-shell-panel-copy,
        .native-shell-canvas-copy {
          color: rgba(255,255,255,0.66);
          font-size: 12px;
        }

        .native-shell-canvas,
        .native-shell-card,
        .native-shell-bottom-frame {
          background: #181820;
          border-radius: 16px;
          border: 1px solid rgba(255,255,255,0.06);
          padding: 16px;
        }
        "#,
    );

    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
