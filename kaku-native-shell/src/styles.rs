use gtk::gdk;

pub fn install_css() {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .native-shell-window, .shell-root, window, box {
          font-family: 'DMMono Nerd Font', '1984대화나눔_본문체_Regular', '1984대화나눔_본문체_Light', '1984대화나눔_본문체_Bold', monospace;
        }

        .native-shell-window,
        window {
          background: #101116;
        }

        .shell-root {
          background: #101116;
          color: #d8d8dc;
          border-radius: 12px;
          border: 1px solid rgba(255,255,255,0.07);
          min-width: 0;
          min-height: 0;
        }

        .shell-body {
          background: #101116;
          margin: 0;
          padding: 0;
          min-width: 0;
          min-height: 0;
        }

        button,
        button:hover,
        button:focus,
        button:focus-visible,
        button:active {
          outline: none;
          box-shadow: none;
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
          min-height: 28px;
          padding: 4px 8px 4px 0;
        }

        .chrome-title {
          font-size: 11px;
          font-weight: 600;
          color: #c9ccd6;
        }

        .chrome-subtitle, .pane-subtitle, .rail-eyebrow, .rail-meta, .workspace-summary, .workspace-hint, .info-label {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 9px;
          color: #8d8e99;
        }

        .chrome-pill {
          padding: 0;
          background: transparent;
          border: none;
          border-radius: 0;
          color: #b9bcc8;
          font-size: 10px;
        }

        .workspace-rail {
          background: #121318;
          border-right: 1px solid rgba(255,255,255,0.05);
          border-top-left-radius: 12px;
        }

        .rail-titlebar {
          min-height: 40px;
          border-bottom: 1px solid rgba(255,255,255,0.06);
        }

        .rail-inbox-header {
          border-top: none;
        }

        .rail-inbox-collapsed {
          min-height: 40px;
        }

        .rail-toggle-button,
        .rail-toggle-button:hover,
        .rail-toggle-button:focus,
        .rail-toggle-button:active {
          background: transparent;
          border: none;
          box-shadow: none;
          outline: none;
          min-width: 16px;
          min-height: 40px;
          padding: 0;
        }

        .rail-inbox-region {
          border-top: 1px solid rgba(255,255,255,0.03);
          padding-bottom: 12px;
        }

        .side-column > box {
          min-height: 0;
        }

        .rail-row {
          padding: 0;
          border-radius: 0;
          background: transparent;
          border: none;
          border-left: 1px solid transparent;
          min-height: 40px;
        }

        .rail-row.active {
          background: rgba(255,255,255,0.045);
          border-left-color: transparent;
        }

        .rail-row.collapsed {
          padding: 10px 6px;
        }

        .rail-workspace-row {
          min-height: 40px;
          border-bottom: 1px solid rgba(255,255,255,0.06);
        }

        .rail-workspace-row.detailed {
          min-height: 40px;
        }

        .pane-title, .workspace-title, .info-value {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 11px;
          font-weight: 600;
          color: #f1f1f5;
        }

        .workspace-rail .rail-name,
        .workspace-rail .rail-section-title,
        .workspace-rail .row-title {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 12px;
          font-weight: 600;
          color: #f1f1f5;
        }

        .workspace-rail .rail-detail,
        .workspace-rail .row-detail,
        .workspace-rail .empty-state,
        .workspace-rail .rail-meta,
        .workspace-rail .rail-eyebrow,
        .workspace-rail .rail-work-item-detail {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 10px;
          color: #8d8e99;
        }

        .rail-status-indicator {
          min-width: 16px;
          min-height: 16px;
        }

        .rail-status-dot {
          opacity: 0.95;
        }

        .rail-status-unread .rail-status-dot {
          color: #7aa1ff;
          -gtk-icon-palette: success #7aa1ff, warning #7aa1ff, error #7aa1ff;
        }

        .rail-status-failed .rail-status-dot {
          color: #ff8f7a;
          -gtk-icon-palette: success #ff8f7a, warning #ff8f7a, error #ff8f7a;
        }

        .rail-status-running .rail-status-dot {
          color: #72d5b4;
          -gtk-icon-palette: success #72d5b4, warning #72d5b4, error #72d5b4;
        }

        .rail-status-idle .rail-status-dot {
          color: rgba(255,255,255,0.14);
          -gtk-icon-palette: success rgba(255,255,255,0.14), warning rgba(255,255,255,0.14), error rgba(255,255,255,0.14);
        }

        .rail-trailing {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 9px;
          color: #94a8db;
        }

        .rail-workspace-group {
          border-bottom: 1px solid rgba(255,255,255,0.06);
        }

        .rail-work-items {
          padding: 6px 0 10px 0;
          background: rgba(255,255,255,0.02);
        }

        .rail-work-item {
          min-height: 40px;
          padding: 0 12px;
          border-bottom: 1px solid rgba(255,255,255,0.06);
        }

        .rail-work-items > :last-child.rail-work-item {
          border-bottom: none;
        }

        .rail-work-item-dot {
          min-width: 8px;
          min-height: 8px;
          border-radius: 999px;
          background: #ff9f43;
          margin: 0 2px 0 0;
        }

        .rail-work-item-unread .rail-work-item-dot {
          background: #ff9f43;
        }

        .rail-work-item-failed .rail-work-item-dot {
          background: #ff9f43;
        }

        .rail-work-item-running .rail-work-item-dot {
          background: #ff9f43;
        }

        .rail-work-item-title {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 11px;
          font-weight: 600;
          color: #d7d9e4;
          margin: 0;
          padding: 0;
          line-height: 1.0;
        }

        .rail-work-item-detail {
          margin: 0;
          padding: 0;
          line-height: 1.0;
        }

        .rail-row:focus,
        .rail-row:focus-visible,
        .pane-titlebar:focus,
        .pane-titlebar:focus-visible {
          outline: none;
          box-shadow: none;
          border: none;
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
          padding: 0;
        }

        .workspace-rail .pane-titlebar.rail-workspace-row,
        .workspace-rail .pane-titlebar.rail-titlebar,
        .workspace-rail .rail-toggle-button {
          min-height: 40px;
        }

        .pane-titlebar:hover {
          background: #191c23;
        }

        .pane-panel.drag-source {
          opacity: 0.78;
        }

        .pane-panel.drop-target {
          border-color: rgba(122,161,255,0.38);
          box-shadow: inset 0 0 0 1px rgba(122,161,255,0.16);
        }

        .pane-titlebar.drop-target,
        .pane-surface.drop-target,
        .pane-panel.drop-target .pane-titlebar,
        .pane-panel.drop-target .pane-surface {
          background: #202733;
          border-color: rgba(122,161,255,0.28);
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

        .live-terminal-host {
          background: #0d0f14;
        }

        .workspace-title {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 14px;
        }

        .workspace-summary,
        .rail-meta {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 9px;
          letter-spacing: 0;
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

        .workspace-meta-row {
          border-bottom: 1px solid rgba(255,255,255,0.05);
          padding-bottom: 2px;
        }

        .shortcut-strip {
          margin-top: 0;
          margin-bottom: 0;
          padding: 0 14px;
          min-height: 40px;
          min-width: 0;
        }

        .shortcut-chip {
          background: transparent;
          border: none;
          padding: 0;
          min-height: 0;
        }

        .shortcut-action,
        .shortcut-combo {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 11px;
          font-weight: 500;
          text-transform: uppercase;
        }

        .shortcut-action {
          color: #aeb3c3;
        }

        .shortcut-combo {
          color: #e7e9ef;
        }

        .shortcut-separator {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 11px;
          font-weight: 500;
          color: #5f6472;
          margin: 0 2px;
        }

        .workspace-block {
          margin-top: 2px;
          padding-top: 6px;
          border-top: 1px solid rgba(255,255,255,0.05);
        }

        .shortcut-bar {
          margin-top: 0;
          border-top: 1px solid rgba(255,255,255,0.05);
          background: #101116;
          min-height: 40px;
          min-width: 0;
          border-bottom-left-radius: 12px;
          border-bottom-right-radius: 12px;
          margin-left: 0;
          margin-right: 0;
          padding-left: 0;
          padding-right: 0;
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

        .terminal-host {
          background: #0d0f14;
          border: none;
          min-height: 0;
        }

        .terminal-toolbar {
          background: #171920;
          border-bottom: 1px solid rgba(255,255,255,0.06);
          min-height: 40px;
        }

        .terminal-toolbar.drag-source {
          opacity: 0.78;
        }

        .terminal-path {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 11px;
          font-weight: 600;
          color: #edf0f8;
        }

        .terminal-status-badge {
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 10px;
          font-weight: 700;
          letter-spacing: 0.08em;
          color: #ffb38a;
          margin-top: 1px;
        }

        .terminal-close-button,
        .terminal-close-button:hover,
        .terminal-close-button:focus,
        .terminal-close-button:active {
          background: transparent;
          border: none;
          box-shadow: none;
          min-width: 20px;
          min-height: 20px;
          padding: 0;
        }

        .terminal-close-button image {
          opacity: 0.84;
        }

        .terminal-close-button:hover image {
          opacity: 1;
        }

        .terminal-host.drop-target {
          border-color: rgba(122,161,255,0.38);
          box-shadow: inset 0 0 0 1px rgba(122,161,255,0.16);
        }

        .terminal-host.drop-target .terminal-toolbar,
        .terminal-host.drop-target .terminal-view {
          background: #202733;
        }

        .terminal-view {
          background: #3e4149;
          color: #edf0f8;
          caret-color: #f4f6fb;
          font-family: 'DMMono Nerd Font', monospace;
          font-size: 12px;
        }

        .terminal-view text {
          background: transparent;
        }

        .workspace-terminal-surface {
          min-height: 240px;
        }

        .list-row, .info-row {
          padding: 6px 0;
          border-bottom: 1px solid rgba(255,255,255,0.04);
        }

        .pane-content {
          margin-top: 6px;
        }

        .compact-stack > box {
          margin-bottom: 0;
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
