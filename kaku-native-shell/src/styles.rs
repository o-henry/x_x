use gtk::gdk;

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
