# Phase 01-03 Summary

## Outcome

Added the core Phase 1 notification CLI subcommands for create, list, clear, mark read, and mark unread with stable JSON contracts and a table fallback for listing.

## Changed Files

- `kaku/src/cli/mod.rs`
- `kaku/src/cli/notify.rs`
- `kaku/src/cli/list_notifications.rs`
- `kaku/src/cli/clear_notifications.rs`
- `kaku/src/cli/mark_read.rs`
- `kaku/src/cli/mark_unread.rs`

## What Changed

- Registered `notify`, `list-notifications`, `clear-notifications`, `mark-read`, and `mark-unread` under `kaku cli`.
- Added typed request builders that map CLI flags directly onto the notification RPC requests from plan 02.
- Printed created notifications and mutation responses as stable pretty JSON.
- Added `table|json` output support for `list-notifications` with the required notification columns.
- Added focused contract tests for workspace-only, tab-only, and pane-scoped notification JSON shapes plus mutation response shapes.

## Verification

- `cargo test --locked -p kaku notification_contracts -- --nocapture`

## Known Limitations

- This plan intentionally stops at create/list/clear/mark read/mark unread and does not include jump/discovery commands.
- The CLI tests validate request/response contract shape locally rather than exercising a live mux server.
