# Phase 04 — Research

## Goal

Plan Phase 4 so failed or long-running panes remain useful after exit and can be rerun, silenced, or tee'd without rebuilding context manually, while preserving Kaku's existing pane/tab/workspace behavior.

## Findings

### 1. Existing hold/exit behavior is already the right remain-on-exit foundation

- [`mux/src/localpane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/localpane.rs) already models:
  - `ExitBehavior::Close`
  - `ExitBehavior::CloseOnCleanExit`
  - `ExitBehavior::Hold`
  - `ProcessState::DeadPendingClose`
- `is_dead()` is the transition point that converts running processes into dead, held, or closed panes and emits the exit trailer text.
- `can_close_without_prompting()` already treats held/dead panes specially.

**Implication:** Phase 4 should extend `ExitBehavior`-adjacent behavior and pane death transitions, not create a detached task runtime or parallel dead-pane state machine.

### 2. The current rerun path is consumer-side only and loses durability when pane metadata disappears

- [`kaku-gui/src/termwindow/mod.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku-gui/src/termwindow/mod.rs) currently implements `rerun_task_center_entry(...)` by reading live pane user vars like:
  - `KAKU_RERUN_COMMAND`
  - `kaku.rerun.command`
- [`mux/src/task_center.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/task_center.rs) computes `rerun_available` from live pane user vars as a limited-mode heuristic.
- [`03-LIMITATIONS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/.planning/phases/03-task-center-overlay/03-LIMITATIONS.md) explicitly records that rerun still depends on runtime pane user vars being present.

**Implication:** Phase 4 must add mux-owned lifecycle persistence for rerun metadata so failed-pane affordances survive beyond the transient live-pane snapshot.

### 3. Mux already has the seams needed for lifecycle ownership and snapshot composition

- [`mux/src/pane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/pane.rs) exposes pane-scoped lifecycle inputs:
  - `is_dead()`
  - `copy_user_vars()`
  - `exit_behavior()`
  - cwd and process info accessors
- [`mux/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs) already owns notification and workspace metadata stores and builds Task Center snapshots from pane/window traversal.
- [`PLANS.md`](/Users/henry/Documents/code/vibe/hybrid/x_x/PLANS.md) already calls out `mux/src/task_panes.rs` as the preferred Phase 4 focused module.

**Implication:** A dedicated mux-owned `task_panes` store is consistent with the existing control-plane pattern and gives both CLI and GUI one source of truth.

### 4. Pipe-pane / tee should hook into existing output fan-out, not replace pane I/O

- [`mux/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/lib.rs) parses pane output in `parse_buffered_data(...)` and forwards action batches through `send_actions_to_mux(...)`.
- [`mux/src/localpane.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/mux/src/localpane.rs) has `emit_output_for_pane(...)` for synthetic output into an existing pane.

**Implication:** Tee is best implemented as additive duplication on the mux output path, preserving normal pane rendering and interactivity. It should not become a second PTY path or replace pane output handling.

### 5. CLI-first lifecycle contracts fit the repo and reduce GUI-only semantics

- Prior phases established the one-command-per-file pattern in [`kaku/src/cli`](/Users/henry/Documents/code/vibe/hybrid/x_x/kaku/src/cli).
- Transport work already extends cleanly through:
  - [`crates/codec/src/lib.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/crates/codec/src/lib.rs)
  - [`crates/wezterm-client/src/client.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/crates/wezterm-client/src/client.rs)
  - [`crates/wezterm-mux-server-impl/src/sessionhandler.rs`](/Users/henry/Documents/code/vibe/hybrid/x_x/crates/wezterm-mux-server-impl/src/sessionhandler.rs)
- Task Center in Phase 3 is already a consumer overlay and should remain so.

**Implication:** Phase 4 should expose explicit lifecycle commands first, then update Task Center and other GUI surfaces to consume the durable lifecycle state.

## Recommended Plan Shape

1. Build a mux-owned `task_panes` lifecycle store and integrate it with localpane death/hold transitions.
2. Add typed transport and CLI contracts for remain-on-exit, rerun/respawn, silence, and lifecycle inspection.
3. Switch Task Center / GUI consumer paths from live-pane heuristics to durable lifecycle snapshots.
4. Add tee-to-file behavior as additive output duplication in mux.
5. Close with docs, limitations, targeted tests, and UAT support.

## Risks

- Extending `localpane` death transitions incorrectly could regress existing close/hold semantics.
- Lifecycle metadata that outlives pane identity too long could produce stale rerun actions or wrong Task Center rows.
- Tee hooks inserted too early in the parsing path could change pane rendering order or duplicate decoded output incorrectly.
- GUI should not become the owner of lifecycle semantics; it must consume mux state.

## Validation Architecture

Phase 4 can use existing Rust test infrastructure without Wave 0 bootstrap.

- **Mux unit/slice tests**
  - `cargo test --locked -p mux task_panes -- --nocapture`
  - Covers lifecycle store transitions, metadata persistence, remain-on-exit logic, and tee bookkeeping.
- **Transport / contract verification**
  - `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl`
  - `cargo test --locked -p codec -- --nocapture`
- **CLI contract tests**
  - `cargo test --locked -p kaku lifecycle_contracts -- --nocapture`
- **GUI / Task Center consumer verification**
  - `cargo test --locked -p kaku-gui task_center -- --nocapture`
  - `cargo check --locked -p kaku-gui`

The plan should keep fast feedback under a couple of minutes by using mux/task-center targeted slices after each wave and reserve broader `cargo check` calls for transport and GUI integration waves.

## Open Questions Resolved By Planning

- Exact CLI names are still discretionary, but they should remain explicit and machine-readable.
- Whether silence is implemented as a boolean flag or small enum can be decided during planning, provided it remains pane-scoped.
- Whether tee metadata is exposed in `list-task-panes` or only implied through CLI action results can be decided during planning, provided verification remains deterministic.
