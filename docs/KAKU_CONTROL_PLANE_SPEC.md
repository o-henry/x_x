# Kaku Agent Control Plane Spec

## 1. Goal

This repository is a local-only fork of Kaku.

The goal is to preserve vanilla Kaku's feel while turning it into a Rust-only multi-agent terminal for solo Unity game development by adding:

1. cmux-like notification and workspace metadata ergonomics
2. tmux-like task-pane lifecycle and control semantics

This is not an upstream PR workflow.
This repo is for local use only.

## 2. Hard Constraints

- Rust-first only.
- Do not introduce Swift, Xcode projects, AppKit sidecar apps, React, Electron, Tauri, or WebView shells.
- Do not copy, port, translate, or reuse source code from cmux or tmux.
- Reimplement behavior from scratch in a Kaku-native way.
- Preserve Kaku's existing architecture and feel.
- Prefer extending these existing Kaku paths:
  - `kaku/src/cli`
  - `kaku-gui/src/tabbar.rs`
  - `kaku-gui/src/commands.rs`
  - `kaku-gui/src/overlay`
  - `kaku-gui/src/termwindow`
  - `mux/src`

## 3. Existing Kaku Capabilities To Reuse

Before adding anything new, reuse existing Kaku capabilities where possible:

- `kaku cli`
- pane/tab/workspace listing
- send-text / get-text style pane I/O
- split-pane / spawn
- set-tab-title
- existing tabbar rendering
- existing overlays and command surfaces
- existing bell / background attention signaling
- existing mux notifications

## 4. Product Shape

### Phase 1 — Notification subsystem
Add:

- in-memory notification store
- unread indexes by workspace / tab / pane
- CLI commands:
  - `notify`
  - `list-notifications`
  - `clear-notifications`
  - `mark-read`
  - `mark-unread`
  - `jump-next-unread`
  - `jump-prev-unread`
  - `identify`
  - `capabilities`
- pane/tab visual unread markers
- sticky vs clear-on-focus semantics
- stable JSON output

### Phase 1 CLI contract

Phase 1 machine-oriented commands:

- `notify`
- `list-notifications`
- `clear-notifications`
- `mark-read`
- `mark-unread`
- `jump-next-unread`
- `jump-prev-unread`
- `identify`
- `capabilities`

Notification record JSON fields:

- `notification_id`
- `workspace`
- `window_id`
- `tab_id`
- `pane_id`
- `kind`
- `title`
- `body`
- `unread`
- `unread_mode`
- `created_at`
- `updated_at`

Mutation response JSON fields:

- `cleared_count`
- `updated_count`
- `notification_ids`

Capabilities JSON fields:

- `notification_commands`
- `unread_modes`
- `supports_tabbar_markers`

### Phase 2 — Workspace metadata plane
Add:

- workspace-scoped status store
- workspace-scoped progress store
- workspace-scoped log store
- CLI commands:
  - `set-status`
  - `clear-status`
  - `list-status`
  - `set-progress`
  - `clear-progress`
  - `log`
  - `clear-log`
  - `list-log`
- UI surfacing using existing Kaku surfaces:
  - tabbar
  - right-status or equivalent
  - overlay

### Phase 2 CLI contract

Phase 2 machine-oriented commands:

- `set-status`
- `clear-status`
- `list-status`
- `set-progress`
- `clear-progress`
- `log`
- `clear-log`
- `list-log`

Workspace status record JSON fields:

- `workspace`
- `status`
- `updated_at`

Workspace progress record JSON fields:

- `workspace`
- `value`
- `updated_at`

Workspace log record JSON fields:

- `workspace`
- `seq`
- `message`
- `created_at`

Workspace metadata clear-mutation JSON fields:

- `cleared_count`
- `workspaces`

Default UI surfacing for Phase 2:

- compact workspace metadata suffix in the existing tab title path
- mux-driven title/status refresh on `WorkspaceMetadataChanged`
- no new sidebar or Task Center surface in this phase

### Phase 3 — Task Center
Add a searchable Task Center overlay.

Sources:
- workspaces
- tabs
- panes
- unread notifications
- failed tasks
- running tasks

Actions:
- focus target
- clear unread
- rerun failed pane if available through existing rerun metadata only

Filters:
- unread
- failed
- running
- workspace
- source
- kind

Concrete shipped shape:
- native `ShowTaskCenter` command in existing Kaku command surfaces
- dedicated overlay module, not a launcher flag overload
- one normalized row list over mux-owned snapshot data
- query-token filtering (`unread`, `failed`, `running`, `workspace:<name>`, `source:<kind>`, `kind:<kind>`) plus free-text search
- focus on `Enter`
- clear unread on selected rows with unread notification ids
- rerun only when the selected failed row still has usable rerun metadata at runtime

### Phase 4 — Task-pane lifecycle
Add:

- remain-on-exit semantics
- rerun / respawn pane
- silence watchdog for selected task panes
- pipe-pane / tee output to file first
- pane lifecycle events
- failed pane persistence

### Phase 4 concrete shipped shape

Phase 4 keeps lifecycle truth mux-owned and extends existing Kaku pane semantics rather than creating a detached tmux-like server model.

Shipped lifecycle behavior:

- durable task-pane registry in `mux`
- remain-on-exit intent persisted per pane and applied through existing `ExitBehavior::Hold`-style behavior
- failed-pane metadata persists long enough for Task Center and CLI consumers to inspect dead panes after exit
- rerun / respawn commands consume durable rerun metadata captured from pane user vars and cwd snapshots
- Task Center failed rows can keep rerun affordances after pane exit when durable metadata exists
- pipe-pane duplicates decoded pane output to a file without changing normal pane rendering or interactivity
- silence-watchdog persists pane-scoped silence intent for lifecycle consumers

Phase 4 machine-oriented commands:

- `list-task-panes`
- `set-remain-on-exit`
- `rerun-pane`
- `respawn-pane`
- `silence-watchdog`
- `pipe-pane`

Task-pane list JSON fields:

- `pane_id`
- `workspace`
- `window_id`
- `tab_id`
- `remain_on_exit`
- `silenced`
- `is_dead`
- `is_failed`
- `rerun_available`
- `tee_path`
- `current_working_dir`
- `updated_at`

Remain-on-exit mutation JSON fields:

- `pane_id`
- `remain_on_exit`
- `is_dead`
- `is_failed`
- `updated_at`

Rerun / respawn mutation JSON fields:

- `pane_id`
- `spawned_pane_id`
- `status`

Watchdog silence mutation JSON fields:

- `pane_id`
- `silenced`

Pipe-pane mutation JSON fields:

- `pane_id`
- `tee_path`

Known Phase 4 boundaries:

- `silence-watchdog` currently persists intent and Task Center-facing state, but Phase 5 still owns deeper watchdog policy hardening
- rerun / respawn rely on durable rerun metadata captured from pane user vars; if upstream task launchers never seed rerun metadata, the action stays unavailable
- tee output is additive append-only duplication and is not a general pipe-redirection subsystem

### Phase 5 — Hardening
Shipped hardening outcomes:

- targeted automated coverage for mux task-pane retention, lifecycle CLI contracts, baseline pane-management CLI compatibility, and Task Center consumer behavior
- typed lifecycle refresh fan-out for remote consumers through `TaskPaneChanged`
- explicit dead-record retention and cleanup rules for mux-owned task panes
- Phase 5 closeout docs that record changed files, manual UAT evidence, and known limitations instead of leaving hidden follow-up work

Wave 4 manual runtime verification on 2026-03-27 established:

- baseline `list`, `spawn`, `split-pane`, `send-text`, `get-text`, `set-tab-title`, `set-remain-on-exit`, `silence-watchdog`, `pipe-pane`, and `list-task-panes` commands remained usable against a live `kaku-gui` session
- the prior failing-pane fixture path (`/bin/sh -lc 'echo PHASE4_FAILING_TASK; false'`) no longer reproduced the earlier `kaku.lua` recursion -> GUI socket EOF failure chain; the failed pane stayed inspectable through `list-task-panes`
- `pipe-pane` duplicated pane output to a file in the live session, though the tee remains raw terminal output rather than a sanitized log format
- Task Center remains an additive Kaku-native overlay in shipped code and tests; the live GUI session stayed a standard Kaku window rather than a new dashboard shell

Known Phase 5 boundaries:

- the rebuilt live `respawn-pane` path now returns the documented machine-readable `respawn` status; remaining Phase 5 limits are about surrounding manual coverage rather than the respawn contract itself
- lifecycle records intentionally favor inspectability over aggressive cleanup; long-lived sessions still depend on explicit cleanup/pruning boundaries
- manual Wave 4 verification focused on the default workspace compatibility path and failing-pane survivability; broader multi-workspace/window interaction still relies on the preserved baseline command surface plus existing automated coverage

## 5. Non-Goals

These are explicitly out of scope for v1:

- embedded browser
- PR / GitHub UI
- port scanner
- new socket daemon
- full detached tmux server semantics
- large UI redesign
- cmux visual clone
- any Swift/Xcode-based side app

## 6. UI Strategy

Do not start with a sidebar.

Prefer this order:
1. tabbar markers
2. right-status / status surfaces
3. overlay
4. Task Center

Only consider a sidebar after the core control plane is complete.

## 7. Unity-Oriented Use Cases

The product must work well for a solo Unity game developer running multiple panes such as:

- planner agent
- worker agent
- reviewer agent
- Unity build pane
- Unity test pane
- logs pane
- git pane

It should make it easy to:
- know which pane needs attention
- keep failed build/test panes visible
- rerun failed panes
- store and view status/progress/log metadata per workspace
- move quickly across active agent/build/test panes

## 8. Acceptance Criteria

The work is done only when:

1. background panes can raise unread attention state correctly
2. next/previous unread navigation works
3. status/progress/log metadata is scoped and visible correctly
4. failed task panes remain visible
5. failed task panes can rerun or respawn
6. output teeing works to file
7. existing Kaku behavior is not broken:
   - split-pane
   - spawn
   - send-text
   - get-text
   - list
   - set-tab-title
   - workspace switching
   - tab navigation

## 9. Implementation Rules

- No giant refactor.
- No god file.
- No speculative architecture rewrite.
- Prefer additive modules.
- Every new CLI surface must have a stable machine-readable contract.
- Event-driven updates are preferred over polling.
- Every phase must compile independently.
- Every phase must include at least targeted verification.
