# Phase 01-04 Summary

## Outcome

Surfaced mux-backed unread notification state in the existing Kaku tabbar without adding a new shell surface.

## Changed Files

- `kaku-gui/src/termwindow/mod.rs`
- `kaku-gui/src/tabbar.rs`
- `kaku-gui/src/frontend.rs`

## What Changed

- Extended `TabInformation` with `has_unread_notifications` and `unread_notification_count`.
- Populated unread tab state from `Mux::notification_unread_count_for_tab(...)`.
- Exposed unread fields to Lua tab formatters through `UserData`.
- Triggered GUI refresh handling for `MuxNotification::NotificationsChanged`.
- Prefixed default tab titles with `! ` when unread notifications are present.
- Added focused tests covering Lua field exposure, notification-triggered refresh, and tabbar prefix behavior.

## Verification

- `cargo test --locked -p kaku-gui tabbar -- --nocapture`
- `cargo check --locked -p kaku-gui`

## Known Limitations

- The unread marker is text-only and limited to the existing default tab title path in this phase.
- `cargo check` reports a dead-code warning for the tabbar refresh helper, but the plan acceptance criteria pass.
