use gtk::gdk;

pub fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .native-shell-window, .shell-root, window, box {
          font-family: 'DM Mono', 'SF Mono', monospace;
        }

        .shell-root {
          background: #101116;
          color: #d8d8dc;
        }

        .shell-body {
          background: #101116;
        }

        .shell-chrome {
          background: #14151a;
          border-bottom: 1px solid rgba(255,255,255,0.05);
        }

        .chrome-title {
          font-size: 12px;
          font-weight: 600;
          color: #f2f2f5;
        }

        .chrome-subtitle, .pane-subtitle, .rail-eyebrow, .rail-meta, .row-detail, .workspace-summary, .workspace-hint, .info-label, .empty-state {
          font-size: 10px;
          color: #8d8e99;
        }

        .chrome-pill {
          padding: 0;
          background: transparent;
          border: none;
          border-radius: 0;
          color: #b9bcc8;
        }

        .workspace-rail {
          background: #121318;
          border-right: 1px solid rgba(255,255,255,0.05);
        }

        .rail-row {
          padding: 10px 14px;
          border-radius: 0;
          background: transparent;
          border: none;
          border-left: 1px solid transparent;
        }

        .rail-row.active {
          background: rgba(255,255,255,0.03);
          border-left-color: rgba(122, 161, 255, 0.7);
        }

        .rail-name, .pane-title, .row-title, .workspace-title, .info-value {
          font-size: 12px;
          font-weight: 600;
          color: #f1f1f5;
        }

        .rail-trailing {
          font-size: 10px;
          color: #94a8db;
        }

        .pane-panel {
          background: transparent;
          border: none;
          border-radius: 0;
        }

        .pane-header {
          border-bottom: 1px solid rgba(255,255,255,0.05);
          background: transparent;
        }

        .pane-body {
          background: transparent;
        }

        .workspace-title {
          font-size: 17px;
        }

        .action-button {
          background: rgba(255,255,255,0.02);
          color: #cfd3df;
          border-radius: 0;
          border: 1px solid rgba(255,255,255,0.05);
          min-height: 30px;
          min-width: 88px;
          padding: 4px 10px;
          box-shadow: none;
        }

        .action-button.compact {
          min-width: 72px;
        }

        .action-button.chrome-button {
          min-width: 104px;
        }

        .action-button.text-only {
          min-width: 0;
        }

        .list-row, .info-row {
          padding: 6px 0;
          border-bottom: 1px solid rgba(255,255,255,0.04);
        }

        .pane-content {
          margin-top: 14px;
        }

        scrolledwindow scrollbar,
        scrolledwindow scrollbar slider {
          min-width: 0;
          min-height: 0;
          background: transparent;
          border: none;
          box-shadow: none;
        }

        scrolledwindow scrollbar slider {
          min-width: 3px;
          min-height: 3px;
          background: rgba(255,255,255,0.08);
          border-radius: 0;
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
