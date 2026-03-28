use gtk::gdk;

pub fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .native-shell-window, .shell-root, window, box {
          font-family: 'Basically A Mono', '1984대화나눔_본문체_Regular', '1984대화나눔_본문체_Light', '1984대화나눔_본문체_Bold', monospace;
        }

        .shell-root {
          background: #101116;
          color: #d8d8dc;
        }

        .shell-body {
          background: #101116;
        }

        .shell-split separator {
          background: rgba(255,255,255,0.014);
          min-width: 1px;
          min-height: 1px;
        }

        .shell-split separator:hover {
          background: rgba(122,161,255,0.08);
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
          padding: 12px 16px;
          border-radius: 0;
          background: transparent;
          border: none;
          border-left: 1px solid transparent;
        }

        .rail-row.active {
          background: rgba(255,255,255,0.045);
          border-left-color: transparent;
        }

        .rail-row.collapsed {
          padding: 10px 6px;
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

        .rail-toggle {
          background: transparent;
          border: none;
          border-radius: 0;
          min-width: 18px;
          min-height: 18px;
          padding: 0;
          box-shadow: none;
        }

        .rail-toggle:hover {
          background: rgba(255,255,255,0.04);
        }

        .rail-toggle image {
          color: #f3f4f8;
        }

        .pane-panel {
          background: #111318;
          border: 1px solid rgba(255,255,255,0.06);
          border-radius: 0;
          min-width: 0;
          min-height: 0;
        }

        .pane-titlebar {
          background: #171920;
          border-bottom: 1px solid rgba(255,255,255,0.06);
          min-height: 32px;
        }

        .pane-titlebar:hover {
          background: #191c23;
        }

        .pane-header {
          background: transparent;
        }

        .pane-body {
          background: transparent;
        }

        .pane-surface {
          background: #111318;
          min-width: 0;
          min-height: 0;
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

        .action-button.subtle {
          background: transparent;
          border: none;
          min-height: 24px;
          min-width: 0;
          padding: 4px 8px;
          color: #b2b6c5;
        }

        .action-button.subtle:hover {
          background: rgba(255,255,255,0.04);
          color: #e4e7f2;
        }

        .subtle-control-button {
          background: transparent;
          border: none;
          border-radius: 0;
          min-height: 22px;
          min-width: 0;
          padding: 0 2px;
          color: #aeb3c3;
          box-shadow: none;
        }

        .subtle-control-button:hover {
          background: transparent;
          color: #eceff8;
        }

        .subtle-control-label {
          font-size: 10px;
          font-weight: 600;
          color: #aeb3c3;
          padding: 2px 0;
        }

        .action-popover {
          background: #17191f;
          border: 1px solid rgba(255,255,255,0.06);
          border-radius: 0;
          box-shadow: none;
        }

        .action-popover-list {
          background: transparent;
        }

        .shortcut-legend {
          margin-top: 4px;
        }

        .shortcut-line {
          padding: 2px 0;
        }

        .workspace-block {
          margin-top: 4px;
          padding-top: 6px;
          border-top: 1px solid rgba(255,255,255,0.05);
        }

        .workspace-section-label {
          font-size: 10px;
          color: #8d8e99;
          margin-top: 4px;
        }

        .workspace-detail-line {
          font-size: 11px;
          color: #c3c6d3;
        }

        .shortcut-action, .shortcut-combo {
          font-family: 'DepartureMono Nerd Font', monospace;
          font-size: 11px;
          font-weight: 400;
          color: #aeb3c3;
        }

        .shortcut-combo {
          color: #d7dbea;
        }

        .list-row, .info-row {
          padding: 6px 0;
          border-bottom: 1px solid rgba(255,255,255,0.04);
        }

        .pane-content {
          margin-top: 12px;
        }

        .compact-stack > box {
          margin-bottom: 0;
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
