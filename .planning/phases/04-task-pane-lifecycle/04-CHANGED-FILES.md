# Changed files

- `mux/src/task_panes.rs`
  Added the mux-owned durable task-pane registry, rerun metadata extraction, and lifecycle persistence tests.
- `mux/src/lib.rs`
  Added task-pane registry ownership, remain-on-exit / rerun / respawn / silence / tee APIs, exit recording, and additive tee output duplication.
- `mux/src/localpane.rs`
  Extended dead-pane handling to honor remain-on-exit intent and record durable exit metadata without replacing existing Kaku exit behavior.
- `mux/src/task_center.rs`
  Upgraded Task Center snapshot assembly to include durable task-pane rows and rerun availability beyond live pane-only heuristics.
- `crates/codec/src/lib.rs`
  Added Phase 4 transport PDUs and JSON contract coverage for task-pane lifecycle commands.
- `crates/wezterm-client/src/client.rs`
  Added typed client RPC helpers for task-pane lifecycle commands.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
  Added server-side handlers and scheduling helpers for task-pane lifecycle operations.
- `crates/wezterm-mux-server-impl/src/dispatch.rs`
  Taught the mux server dispatch path about lifecycle notifications so consumers can refresh safely.
- `kaku/src/cli/mod.rs`
  Registered the Phase 4 lifecycle subcommands.
- `kaku/src/cli/list_task_panes.rs`
  Added the machine-readable task-pane lifecycle list command.
- `kaku/src/cli/set_remain_on_exit.rs`
  Added the remain-on-exit mutation command and JSON contract tests.
- `kaku/src/cli/rerun_pane.rs`
  Added the durable rerun mutation command and JSON contract tests.
- `kaku/src/cli/respawn_pane.rs`
  Added the durable respawn mutation command and JSON contract tests.
- `kaku/src/cli/silence_watchdog.rs`
  Added the watchdog silence mutation command and JSON contract tests.
- `kaku/src/cli/pipe_pane.rs`
  Added the pipe-pane / tee mutation command and JSON contract tests.
- `kaku-gui/src/termwindow/mod.rs`
  Upgraded GUI lifecycle consumers to refresh from durable task-pane state and rerun from dead-pane metadata.
- `kaku-gui/src/frontend.rs`
  Added lifecycle notification awareness to keep GUI-side notification handling exhaustive.
- `docs/KAKU_CONTROL_PLANE_SPEC.md`
  Documented the shipped Phase 4 lifecycle contract, command set, and boundaries honestly.
- `PLANS.md`
  Updated milestone progress and recorded the shipped Phase 4 contract.
