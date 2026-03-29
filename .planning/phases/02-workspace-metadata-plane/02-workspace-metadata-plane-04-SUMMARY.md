---
phase: 02-workspace-metadata-plane
plan: 04
subsystem: ui
tags: [rust, kaku-gui, mux, tabbar, workspace-metadata]
requires:
  - phase: 02-01
    provides: "Mux-owned workspace metadata store and WorkspaceMetadataChanged notifications"
provides:
  - "TabInformation snapshots now carry mux-owned workspace status and progress"
  - "Workspace metadata changes refresh existing title and status surfaces through mux notifications"
  - "Default tab titles append compact workspace status/progress suffixes without changing Kaku layout"
affects: [phase-02-plan-05, task-center, status-surfaces]
tech-stack:
  added: []
  patterns: [event-driven gui refresh, mux-owned workspace metadata snapshots, compact tab title suffix composition]
key-files:
  created: []
  modified: [kaku-gui/src/termwindow/mod.rs, kaku-gui/src/tabbar.rs, kaku-gui/src/frontend.rs, crates/wezterm-mux-server-impl/src/dispatch.rs]
key-decisions:
  - "Workspace status and progress are read from the mux window workspace when building TabInformation snapshots."
  - "Workspace metadata stays in the existing default tab title path as plain text instead of introducing new tabbar chrome."
patterns-established:
  - "GUI metadata refreshes should follow dedicated MuxNotification variants and reuse existing title/status invalidation paths."
  - "Workspace metadata rendering composes a suffix before unread prefixing so default title semantics stay stable."
requirements-completed: [META-05]
duration: 16min
completed: 2026-03-27
---

# Phase 02 Plan 04: Workspace Metadata Plane Summary

**Mux-backed workspace status and progress now flow into tab snapshots and render as compact default tab title suffixes**

## Performance

- **Duration:** 16 min
- **Started:** 2026-03-27T04:23:30Z
- **Completed:** 2026-03-27T04:39:40Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added `workspace_status` and `workspace_progress` to `TabInformation` and exposed both fields to Lua formatters.
- Wired `WorkspaceMetadataChanged` into the existing title/status invalidation path so metadata refreshes stay mux-driven.
- Rendered compact ` · [status]`, ` · NN%`, and ` · [status] NN%` suffixes in the default tab title path while preserving unread prefixes.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend termwindow metadata snapshots and refresh triggers** - `9bb9567` (fix)
2. **Task 2: Compose a compact default tab title for workspace status and progress** - `c5b99b1` (feat)

## Files Created/Modified
- `kaku-gui/src/termwindow/mod.rs` - Added workspace metadata fields to `TabInformation`, Lua exposure, mux lookups, and metadata-triggered refresh handling.
- `kaku-gui/src/tabbar.rs` - Added workspace metadata suffix helpers and focused tab title composition tests.
- `kaku-gui/src/frontend.rs` - Added a no-op `WorkspaceMetadataChanged` match arm so the GUI frontend still compiles against the expanded mux notification enum.
- `crates/wezterm-mux-server-impl/src/dispatch.rs` - Added a no-op `WorkspaceMetadataChanged` match arm so the mux server dispatcher remains exhaustive during GUI test builds.

## Decisions Made
- Used the window workspace from `Mux::get_window(self.mux_window_id)` as the lookup key for `workspace_status_for_workspace` and `workspace_progress_for_workspace`, keeping workspace metadata mux-owned instead of pane-local.
- Kept metadata rendering text-only in the existing default title path to preserve Kaku’s current tabbar layout and unread marker behavior.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added exhaustive handling for WorkspaceMetadataChanged in mux server dispatch**
- **Found during:** Task 1 (Extend termwindow metadata snapshots and refresh triggers)
- **Issue:** `crates/wezterm-mux-server-impl/src/dispatch.rs` did not match the existing `MuxNotification::WorkspaceMetadataChanged` variant, so the focused GUI test target failed to compile.
- **Fix:** Added a no-op match arm for `WorkspaceMetadataChanged` in the server dispatch loop.
- **Files modified:** `crates/wezterm-mux-server-impl/src/dispatch.rs`
- **Verification:** `cargo test --locked -p kaku-gui tabbar -- --nocapture`
- **Committed in:** `9bb9567`

**2. [Rule 3 - Blocking] Added exhaustive handling for WorkspaceMetadataChanged in GUI frontend**
- **Found during:** Task 1 (Extend termwindow metadata snapshots and refresh triggers)
- **Issue:** `kaku-gui/src/frontend.rs` also lacked a `WorkspaceMetadataChanged` match arm, which prevented the focused GUI test target from compiling.
- **Fix:** Added a no-op `WorkspaceMetadataChanged` branch in the frontend mux subscription handler.
- **Files modified:** `kaku-gui/src/frontend.rs`
- **Verification:** `cargo test --locked -p kaku-gui tabbar -- --nocapture`
- **Committed in:** `9bb9567`

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both fixes were compile blockers needed to verify the planned GUI changes. No scope creep beyond exhaustive enum handling.

## Issues Encountered
- The first verification run surfaced pre-existing exhaustive-match gaps for `WorkspaceMetadataChanged` outside the two planned files. Both were resolved inline so the focused plan target could compile and run.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- The GUI now has mux-backed workspace metadata on the existing title path, which gives Phase 02-05 a stable base for regression checks and documentation.
- Remaining Phase 2 work should preserve the current event-driven refresh pattern and avoid moving workspace metadata into pane-local state or new UI chrome.

## Self-Check: PASSED

- Found `.planning/phases/02-workspace-metadata-plane/02-workspace-metadata-plane-04-SUMMARY.md`
- Found task commits `9bb9567` and `c5b99b1` in git history

---
*Phase: 02-workspace-metadata-plane*
*Completed: 2026-03-27*
