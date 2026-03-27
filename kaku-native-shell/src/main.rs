use adw::prelude::*;
use gtk::gdk;

const APP_ID: &str = "dev.tw93.kaku.NativeShell";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| install_css());
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Kaku Native Shell")
        .default_width(1480)
        .default_height(920)
        .build();

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.add_css_class("native-shell-root");
    root.append(&build_top_chrome());
    root.append(&build_content());
    window.set_content(Some(&root));
    window.present();
}

fn build_top_chrome() -> gtk::Box {
    let chrome = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    chrome.add_css_class("native-shell-chrome");

    let title_stack = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let eyebrow = gtk::Label::new(Some("KAKU"));
    eyebrow.add_css_class("native-shell-eyebrow");
    eyebrow.set_xalign(0.0);

    let title = gtk::Label::new(Some("default workspace"));
    title.add_css_class("native-shell-title");
    title.set_xalign(0.0);

    title_stack.append(&eyebrow);
    title_stack.append(&title);

    let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let status_chip = gtk::Label::new(Some("3 running  2 failed  4 inbox"));
    status_chip.add_css_class("native-shell-status-chip");

    chrome.append(&title_stack);
    chrome.append(&spacer);
    chrome.append(&status_chip);
    chrome
}

fn build_content() -> gtk::Widget {
    let rail = build_rail();

    let vertical = gtk::Paned::new(gtk::Orientation::Vertical);
    vertical.set_wide_handle(false);
    vertical.set_position(620);
    vertical.set_start_child(Some(&build_main_row()));
    vertical.set_end_child(Some(&build_bottom_panel()));

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.add_css_class("native-shell-body");
    body.append(&rail);
    body.append(&vertical);
    body.upcast()
}

fn build_rail() -> gtk::Box {
    let rail = gtk::Box::new(gtk::Orientation::Vertical, 10);
    rail.add_css_class("native-shell-rail");

    let workspace_header = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let title = gtk::Label::new(Some("KAKU"));
    title.add_css_class("native-shell-rail-title");
    title.set_xalign(0.0);

    let subtitle = gtk::Label::new(Some("DEFAULT"));
    subtitle.add_css_class("native-shell-rail-subtitle");
    subtitle.set_xalign(0.0);

    workspace_header.append(&title);
    workspace_header.append(&subtitle);

    let nav = gtk::ListBox::new();
    nav.add_css_class("native-shell-nav");
    nav.set_selection_mode(gtk::SelectionMode::Single);
    nav.append(&nav_row("Home", Some("4"), true));
    nav.append(&nav_row("Inbox", None, false));
    nav.append(&nav_row("Run", Some("3"), false));
    nav.append(&nav_row("Fail", Some("2"), false));
    nav.append(&nav_row("Meta", None, false));
    nav.append(&nav_row("Tasks", Some("2"), false));

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

fn build_main_row() -> gtk::Paned {
    let horizontal = gtk::Paned::new(gtk::Orientation::Horizontal);
    horizontal.set_wide_handle(false);
    horizontal.set_position(980);
    horizontal.set_start_child(Some(&build_main_surface()));
    horizontal.set_end_child(Some(&build_context_panel()));
    horizontal
}

fn build_main_surface() -> gtk::Widget {
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

    let canvas_copy = gtk::Label::new(Some(
        "Phase 8 starts here: this area will host the primary Kaku workspace surface and terminal bridge.",
    ));
    canvas_copy.add_css_class("native-shell-canvas-copy");
    canvas_copy.set_wrap(true);
    canvas_copy.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    canvas_copy.set_justify(gtk::Justification::Center);
    canvas.set_child(Some(&canvas_copy));

    shell.append(&heading);
    shell.append(&canvas);
    shell.upcast()
}

fn build_context_panel() -> gtk::Widget {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
    panel.add_css_class("native-shell-panel");
    panel.append(&panel_card(
        "Inbox",
        "Unread notifications and focus actions will live here.",
    ));
    panel.append(&panel_card(
        "Failures",
        "Failed/running task snapshots bridge in from mux state next.",
    ));
    panel.append(&panel_card(
        "Metadata",
        "Workspace status and progress editing will move into this panel.",
    ));
    panel.upcast()
}

fn build_bottom_panel() -> gtk::Widget {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 10);
    panel.add_css_class("native-shell-bottom");

    let title = gtk::Label::new(Some("Recent activity"));
    title.add_css_class("native-shell-panel-title");
    title.set_xalign(0.0);

    let frame = gtk::Frame::new(None);
    frame.add_css_class("native-shell-bottom-frame");

    let copy = gtk::Label::new(Some(
        "Task history, logs, and operator actions will move here once the control-plane bridge is wired.",
    ));
    copy.add_css_class("native-shell-panel-copy");
    copy.set_wrap(true);
    copy.set_xalign(0.0);
    frame.set_child(Some(&copy));

    panel.append(&title);
    panel.append(&frame);
    panel.upcast()
}

fn panel_card(title: &str, copy: &str) -> gtk::Frame {
    let frame = gtk::Frame::new(None);
    frame.add_css_class("native-shell-card");

    let content = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("native-shell-panel-title");
    heading.set_xalign(0.0);

    let body = gtk::Label::new(Some(copy));
    body.add_css_class("native-shell-panel-copy");
    body.set_wrap(true);
    body.set_xalign(0.0);

    content.append(&heading);
    content.append(&body);
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
