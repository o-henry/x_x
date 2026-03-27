# Phase 3: Task Center Overlay - Research

**Researched:** 2026-03-27
**Domain:** Kaku-native searchable overlay for mux-backed work aggregation and lightweight actions
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Overlay shape
- **D-01:** Task Center must reuse Kaku's existing overlay model rather than introducing a sidebar, separate screen, or shell redesign.
- **D-02:** The natural entry path is the existing overlay/launcher command surface and lifecycle, adapted to Task Center semantics.

### Data ownership
- **D-03:** Task Center is a read/act overlay over mux-owned truth only; it must not create a parallel store for notifications, metadata, or pane/task state.
- **D-04:** The overlay aggregates existing sources: workspaces, tabs, panes, unread notifications, and whatever failed/running signals are already available from the current control plane.

### Phase boundary vs Phase 4
- **D-05:** Phase 3 provides the overlay UX skeleton for failed/running work, but complete rerun/remain-on-exit lifecycle reliability belongs to Phase 4.
- **D-06:** Failed/running items may be surfaced in a limited mode if only partial signals exist today, as long as the overlay contract stays honest and Kaku-native.

### Action surface
- **D-07:** Primary Task Center actions are limited to: focus target, clear unread, and rerun failed pane when rerun metadata exists.
- **D-08:** Filtering must cover unread, failed, running, workspace, source, and kind per the spec and roadmap.

### the agent's Discretion
- Result ranking, grouping, and keyboard affordances inside the overlay.
- Exact row density and copy style, provided the result feels like an existing Kaku overlay.
- How much launcher helper logic to reuse directly versus lightly adapting the pattern in a dedicated Task Center module.

### Deferred Ideas (OUT OF SCOPE)
- Full lifecycle reliability for failed panes, remain-on-exit semantics, and rerun metadata production — Phase 4.
- New app shell, sidebar, dashboard, or browser-like management surface.
- Separate UI-spec phase for Phase 3; acceptable later if execution reveals genuine ambiguity.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TASK-01 | Open a searchable Task Center overlay that aggregates workspaces, tabs, panes, unread items, failed tasks, and running tasks. | Reuse overlay bootstrap and launcher-style filtering, but source data from mux-backed snapshots composed from notifications, workspace metadata, windows, tabs, and panes. |
| TASK-02 | Filter Task Center results by unread, failed, running, workspace, source, and kind. | Keep filters local to overlay state and operate on one normalized entry list with typed flags rather than multiple ad hoc lists. |
| TASK-03 | Focus a selected target directly from Task Center. | Reuse existing pane/tab/window identity paths and `TermWindow` action plumbing instead of inventing a new navigation mechanism. |
| TASK-04 | Clear unread state from Task Center. | Use existing notification store mutation APIs by resolving matching unread notification ids for the selected entry scope. |
| TASK-05 | Rerun a failed pane when rerun data is available. | Consumer-side support should exist in Phase 3, but metadata production and lifecycle guarantees remain Phase 4 scope. |
</phase_requirements>

## Summary

Phase 3 should be planned as an overlay-first aggregation feature that stitches together the sources added in Phases 1 and 2 instead of introducing a new control-plane store. The existing codebase already has the right shape for this: `kaku-gui/src/overlay/launcher.rs` demonstrates a searchable, keyboard-driven overlay pattern; `kaku-gui/src/overlay/mod.rs` provides the lifecycle bootstrap and cancellation path; `kaku-gui/src/commands.rs` is already the palette/command registry; `kaku-gui/src/termwindow/mod.rs` owns per-window cached mux snapshots and overlay assignment; `mux/src/lib.rs` already exposes notification unread counts and workspace metadata lookups.

The cleanest Phase 3 architecture is a normalized `TaskCenterEntry` model built from existing mux state at read time. That entry model can live in a focused `mux/src/task_center.rs` helper module or equivalent additive extension, but it should remain explicitly read-only: aggregate workspace entries from `workspace_state`, tab entries from existing window/tab traversal, pane entries from `PaneInformation`-style identity/title/progress data, and unread state from `notification_store`. For failed/running signals, the honest Phase 3 approach is to use currently available pane liveness/progress/user-var signals without claiming full task lifecycle semantics; Phase 4 can later improve precision without invalidating the overlay contract.

For rerun, the research-backed recommendation is to implement only the consumer side in Phase 3. The overlay should expose a rerun action when the selected failed pane already carries rerun metadata, most naturally through pane user vars or another explicit pane-scoped payload. The overlay should not invent that metadata source here, because doing so would pull lifecycle work forward from Phase 4.

**Primary recommendation:** Plan Phase 3 as five small additive slices: mux aggregation helpers, GUI snapshot plumbing, dedicated overlay UI, command/keybinding/action wiring, and a closeout slice that adds rerun-if-available handling plus docs/tests/limitations.

## Project Constraints (from AGENTS.md and spec)

- Read and follow `docs/KAKU_CONTROL_PLANE_SPEC.md` and `PLANS.md`.
- Rust-only, preserve Kaku feel, no Swift/Xcode/WebView/React/Electron/Tauri.
- Reuse preferred extension points first: `kaku-gui/src/overlay`, `kaku-gui/src/commands.rs`, `kaku-gui/src/termwindow`, `mux/src`.
- Keep changes additive and avoid large shell/UI refactors.

## Standard Stack

### Core
| Library / Module | Version | Purpose | Why Standard |
|------------------|---------|---------|--------------|
| `kaku-gui/src/overlay/launcher.rs` | repo-local | Searchable overlay behavior pattern | Already provides keyboard-centric entry filtering, row rendering, and selection behavior aligned with Kaku overlays. |
| `kaku-gui/src/overlay/mod.rs` | repo-local | Overlay lifecycle bootstrap/cancel path | Prevents inventing a second overlay manager for Task Center. |
| `kaku-gui/src/termwindow/mod.rs` | repo-local | GUI snapshot/action hub | Already handles mux notifications, overlay assignment, and tab/pane identity flow. |
| `mux/src/lib.rs` + `mux/src/task_center.rs` (new or equivalent helper) | repo-local | Read-only task center aggregation source | Keeps Task Center mux-backed and additive without creating a new daemon or store. |
| `config/src/keyassignment.rs` | repo-local | Native command/config entry point | Needed if Task Center becomes a first-class command/palette/keybinding action. |

### Supporting
| Library / Module | Version | Purpose | When to Use |
|------------------|---------|---------|-------------|
| `mux::Pane::is_dead`, `get_progress`, `copy_user_vars` | repo-local trait methods | Current failed/running/rerun signal surface | Use as the limited Phase 3 signal source without over-promising Phase 4 lifecycle guarantees. |
| `notification_store` helpers via `Mux` | repo-local | Unread counts and record ids | Use to build unread entries and implement clear-unread actions. |
| `workspace_state` helpers via `Mux` | repo-local | Workspace status/progress/log context | Use to enrich workspace and tab entries instead of adding GUI-local metadata models. |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Dedicated Task Center overlay | Reusing launcher itself with a new flag | Too coupled; Task Center needs custom rows/actions/entry types that go beyond launcher's current menu semantics. |
| Mux-backed aggregate helper | GUI-local aggregation only | Worse: duplicates unread/metadata logic and makes rename/update semantics harder to trust. |
| Limited rerun consumer in Phase 3 | Full rerun metadata production now | Worse: widens scope into Phase 4 lifecycle work. |
| Existing overlay/command surfaces | New persistent dashboard | Violates product constraints and risks changing Kaku's feel. |

## Architecture Patterns

### Pattern 1: Normalize Everything Into One Entry List
**What:** Build a single `TaskCenterEntry` list with typed source/kind/flags so filtering and actions operate on one model.
**When to use:** Immediately for TASK-01 and TASK-02.
**Example shape:**
```rust
pub enum TaskCenterSource {
    Workspace,
    Tab,
    Pane,
    Notification,
}

pub struct TaskCenterEntry {
    pub label: String,
    pub workspace: String,
    pub source: TaskCenterSource,
    pub pane_id: Option<PaneId>,
    pub tab_id: Option<TabId>,
    pub unread_count: usize,
    pub is_failed: bool,
    pub is_running: bool,
    pub kind: Option<String>,
}
```

### Pattern 2: Read-Only Mux Aggregation, GUI-Owned Presentation
**What:** `mux` builds authoritative snapshots; `kaku-gui` maps them into overlay rows and user actions.
**When to use:** For all entry gathering and refresh logic.
**Why:** Preserves the Phase 1/2 ownership rule without forcing overlay code into `mux`.

### Pattern 3: Action Helpers Live in `TermWindow`
**What:** The overlay should request `focus`, `clear unread`, and `rerun if available` through `TermWindow` helpers rather than calling deep mux internals directly from render code.
**When to use:** For TASK-03 through TASK-05.
**Why:** `TermWindow` already owns window-local overlay and action coordination.

### Pattern 4: Honest Limited-Mode Failed/Running Signals
**What:** Phase 3 may use existing pane liveness/progress/user-var signals as the current failed/running proxy.
**When to use:** Until Phase 4 introduces stronger lifecycle metadata.
**Why:** Satisfies the overlay UX goal without lying about lifecycle guarantees.

## Recommended Verification Strategy

- `cargo test --locked -p mux task_center -- --nocapture`
  Validate snapshot aggregation, unread filtering, and limited failed/running classification.
- `cargo test --locked -p kaku-gui task_center -- --nocapture`
  Validate filter logic, row rendering decisions, action availability, and overlay command wiring.
- `cargo check --locked -p kaku-gui`
  Validate the integrated GUI command/overlay path without widening to a full suite during execution loops.

## Planning Recommendation

Plan Phase 3 as:
1. mux aggregation snapshot foundation
2. termwindow cache/action plumbing
3. dedicated Task Center overlay UI
4. native command/keybinding + focus/clear actions
5. rerun-if-available support plus docs/tests/limitations closeout

This ordering keeps the phase additive, Kaku-native, and explicitly below the lifecycle boundary that belongs to Phase 4.
