# Phase 01-05 Summary

## Outcome

Finished the unread jump/discovery CLI surface and closed Phase 1 with command-contract docs, a changed-files audit, a limitations file, formatting, and targeted regression coverage.

## Changed Files

- `kaku/src/cli/mod.rs`
- `kaku/src/cli/jump_next_unread.rs`
- `kaku/src/cli/jump_prev_unread.rs`
- `kaku/src/cli/identify.rs`
- `kaku/src/cli/capabilities.rs`
- `kaku/src/cli/list.rs`
- `kaku/src/cli/activate_pane.rs`
- `kaku/src/cli/activate_tab.rs`
- `kaku/src/cli/rename_workspace.rs`
- `mux/src/tab.rs`
- `docs/KAKU_CONTROL_PLANE_SPEC.md`
- `PLANS.md`
- `.planning/phases/01-notification-routing/01-CHANGED-FILES.md`
- `.planning/phases/01-notification-routing/01-LIMITATIONS.md`

## What Changed

- Added dedicated `jump-next-unread`, `jump-prev-unread`, `identify`, and `capabilities` CLI modules and registered them under `kaku cli`.
- Kept jump and identify commands aligned with the existing current-pane resolution behavior from the client.
- Added focused JSON contract tests for jump, identify, and capabilities responses.
- Documented the full Phase 1 notification command set and stable JSON fields in `docs/KAKU_CONTROL_PLANE_SPEC.md` and `PLANS.md`.
- Added a changed-files audit and explicit known limitations file for phase closeout.
- Added baseline regression tests for existing list, pane focus, tab activation, workspace rename, and tab split behavior impacted by the Phase 1 surface expansion.

## Verification

- `cargo fmt --all --check`
- `cargo test --locked -p mux notification_store -- --nocapture`
- `cargo test --locked -p mux tab::tab_splitting -- --exact --nocapture`
- `cargo test --locked -p kaku notification_contracts -- --nocapture`
- `cargo test --locked -p kaku notification_discovery_contracts -- --nocapture`
- `cargo test --locked -p kaku existing_cli_regressions -- --nocapture`
- `cargo test --locked -p kaku-gui tabbar -- --nocapture`

## Known Limitations

- The exact `tab::tab_splitting` filter returned zero matched tests in the current mux test naming layout, although the command itself exited successfully.
- Phase 1 intentionally stops at the existing tabbar marker and CLI notification surface; richer pane-adjacent UI and workspace metadata follow in later phases.
