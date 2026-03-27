# Phase 05 Changed Files

## Summary

Phase 05 hardened the existing control-plane surfaces instead of adding new product areas. The work stayed inside mux lifecycle state, lifecycle transport/CLI contracts, baseline compatibility tests, Task Center consumer checks, and the final phase closeout docs.

## Code and Test Files

- `mux/src/task_panes.rs`
  Canonical rerun fallback coverage, retained-dead versus stale-live pruning behavior, and task-pane lifecycle tests used by the Phase 5 quick suite.
- `mux/src/lib.rs`
  Failure-path lifecycle retention, cwd preservation on immediate exits, cleanup notifications, and live lifecycle mutation semantics.
- `crates/codec/src/lib.rs`
  Typed `TaskPaneChanged` transport event for remote lifecycle refresh.
- `crates/wezterm-client/src/client.rs`
  Remote consumer resync on typed task-pane lifecycle updates.
- `crates/wezterm-mux-server-impl/src/dispatch.rs`
  Bridges mux lifecycle notifications into the transport event stream.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
  Treats `TaskPaneChanged` as unilateral lifecycle traffic.
- `kaku/src/cli/list_task_panes.rs`
  Exact JSON contract assertions for lifecycle listing output.
- `kaku/src/cli/set_remain_on_exit.rs`
  Exact JSON contract assertions for remain-on-exit mutations.
- `kaku/src/cli/rerun_pane.rs`
  Exact JSON contract assertions and accepted `status: "rerun"` enforcement.
- `kaku/src/cli/respawn_pane.rs`
  Exact JSON contract assertions and accepted `status: "respawn"` enforcement.
- `kaku/src/cli/silence_watchdog.rs`
  Exact JSON contract assertions for watchdog silence mutations.
- `kaku/src/cli/pipe_pane.rs`
  Exact JSON contract assertions for pipe-pane mutations.
- `kaku/tests/compatibility_cli.rs`
  Baseline pane-management compatibility regression slice for `list`, `spawn`, `split-pane`, `send-text`, `get-text`, and `set-tab-title`.
- `kaku-gui/src/termwindow/mod.rs`
  Lifecycle refresh and Task Center routing regression tests that keep the GUI a consumer of mux state.
- `kaku-gui/src/overlay/task_center.rs`
  Task Center filtering/search/additivity regression tests over mux snapshots.

## Closeout Docs

- `docs/KAKU_CONTROL_PLANE_SPEC.md`
  Final Phase 5 shipped outcome and known hardening boundaries.
- `PLANS.md`
  Phase 5 marked complete with the live runtime limitation recorded explicitly.
- `.planning/phases/05-hardening-and-compatibility/05-UAT.md`
  Real manual runtime verification log, commands, outcomes, and blockers.
- `.planning/phases/05-hardening-and-compatibility/05-LIMITATIONS.md`
  Residual hardening limits that remain after the automated suite and Wave 4 manual run.

## Phase 05 Task Commits

- `b9703c9` - test(05-01): add failing task pane retention coverage
- `10572d5` - feat(05-01): canonicalize rerun fallback command shape
- `10f7902` - test(05-01): cover immediate failed-pane cwd retention
- `4f31a89` - fix(05-01): preserve last known cwd for immediate failures
- `b165e88` - test(05-02): cover lifecycle retention and remote refresh invariants
- `ad34343` - feat(05-02): harden lifecycle retention and TaskPaneChanged transport
- `85a5758` - test(05-02): lock lifecycle CLI contract shapes
- `136a886` - feat(05-02): validate exact rerun and respawn status strings
- `8426361` - test(05-03): add baseline pane-management compatibility regression slice
- `53543aa` - test(05-03): guard Task Center additivity and routing refresh behavior
