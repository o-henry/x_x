# Changed files

- `mux/src/task_center.rs`
  Added the mux-owned Task Center snapshot model and limited-mode unread/failed/running/rerun derivation.
- `mux/src/lib.rs`
  Added `task_center_snapshot()` and notification-anchor glue so GUI consumers can read one normalized snapshot.
- `kaku-gui/src/termwindow/mod.rs`
  Added per-window Task Center cache refresh, focus/clear-unread/rerun helpers, and native overlay entrypoint wiring.
- `kaku-gui/src/overlay/mod.rs`
  Exported the dedicated Task Center overlay module.
- `kaku-gui/src/overlay/task_center.rs`
  Added the Task Center overlay, query-token filtering, empty-state handling, and focus/clear-unread/rerun action wiring.
- `config/src/keyassignment.rs`
  Added the first-class `ShowTaskCenter` key assignment.
- `kaku-gui/src/commands.rs`
  Registered Task Center in the command palette/default command set with native menu metadata and shortcut.
- `docs/KAKU_CONTROL_PLANE_SPEC.md`
  Documented the concrete Phase 3 overlay shape, filters, and guarded rerun semantics.
- `PLANS.md`
  Updated the Phase 3 scope notes to match the shipped overlay behavior.
