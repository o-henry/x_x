---
phase: 06-operator-ui-surfaces
plan: 01
subsystem: ui
tags: [rust, kaku-gui, termwindow, codec, wezterm-client, mux-server]
requires:
  - phase: 02-workspace-metadata-plane
    provides: workspace status/progress state and CLI-domain mutation contracts
  - phase: 03-task-center-overlay
    provides: mux-owned Task Center snapshot rows and TermWindow row actions
  - phase: 05-hardening-and-compatibility
    provides: durable task-pane lifecycle records and verified rerun/remain-on-exit paths
provides:
  - typed workspace progress list transport for GUI refreshes
  - TermWindow helpers for scoped Task Center opening
  - TermWindow helpers for workspace status/progress refresh and mutation
  - TermWindow helper for remain-on-exit row actions through task-pane lifecycle state
affects: [06-02-PLAN.md, 06-03-PLAN.md, operator-ui-surfaces, task-center, workspace-metadata]
tech-stack:
  added: []
  patterns:
    - client-domain workspace metadata refreshes status and progress together before repainting UI state
    - Task Center overlays stay view-only while TermWindow owns scope filtering and row action dispatch
    - remain-on-exit toggles reuse mux task-pane lifecycle records rather than a new UI-owned state path
key-files:
  created: []
  modified:
    - crates/codec/src/lib.rs
    - crates/wezterm-client/src/client.rs
    - crates/wezterm-mux-server-impl/src/sessionhandler.rs
    - kaku-gui/src/termwindow/mod.rs
key-decisions:
  - "Added ListWorkspaceProgress as a new additive codec PDU pair instead of renumbering existing workspace metadata messages."
  - "TermWindow refreshes status and progress caches together so tab/title consumers see one coherent metadata snapshot."
  - "Scoped Task Center entry filtering and remain-on-exit row toggles live in TermWindow so overlays stay transient views, not new state owners."
patterns-established:
  - "Transport parity pattern: whenever the GUI needs a metadata read path, mirror the existing status/progress mutation contract style through codec, client, and mux-server."
  - "Controller seam pattern: later Phase 06 surfaces should call TermWindow helpers for scoped Task Center views and workspace metadata changes instead of inventing new action paths."
requirements-completed: [UI-03, UI-04, UI-05]
duration: 19min
completed: 2026-03-27
---

# Phase 06 Plan 01: Operator UI Surfaces Summary

**Typed workspace progress refresh transport plus a TermWindow-owned controller seam for scoped Task Center views, metadata edits, and remain-on-exit toggles**

## Performance

- **Duration:** 19 min
- **Started:** 2026-03-27T13:20:31Z
- **Completed:** 2026-03-27T13:39:31Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added a typed `list_workspace_progress` request/response path so GUI metadata refreshes can read progress without touching mux state directly from overlays.
- Upgraded `TermWindow` to refresh workspace status and progress caches together and expose common-path helpers for workspace metadata mutations.
- Added a reusable controller seam for scoped Task Center opening and remain-on-exit row actions so later Phase 06 surfaces can stay additive and Kaku-native.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend the typed workspace-metadata transport so the GUI can read and refresh progress as well as status** - `68f2a3f` (test), `68555d2` (feat)
2. **Task 2: Add TermWindow operator helpers for scoped Task Center opening and common-path metadata edits** - `bb3f401` (feat)

## Files Created/Modified

- `crates/codec/src/lib.rs` - Added additive `ListWorkspaceProgress` / `ListWorkspaceProgressResponse` PDUs and round-trip coverage.
- `crates/wezterm-client/src/client.rs` - Exposed the typed `list_workspace_progress` RPC wrapper alongside the existing workspace metadata calls.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs` - Served the new workspace progress list request from mux-owned progress records.
- `kaku-gui/src/termwindow/mod.rs` - Added scoped Task Center helpers, symmetric workspace status/progress cache refreshes, metadata mutation helpers, remain-on-exit row toggles, and `workspace_metadata`-filtered tests.

## Decisions Made

- Appended new progress-list PDUs at fresh IDs instead of shifting existing identifiers so the transport change stays additive and stable.
- Refreshed status and progress caches together in `TermWindow` so tabbar/title surfaces don't repaint from a half-updated metadata snapshot.
- Kept Task Center scope filtering in `TermWindow` instead of the overlay so future tabbar/menu affordances can request scoped operator views through one controller seam.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cargo fmt --check` required formatting updates in `kaku-gui/src/termwindow/mod.rs` before the Task 2 verification pass.
- The new `workspace_metadata` tests initially missed imports for `WorkspaceStatusState`, `WorkspaceProgressState`, and `TaskCenterScope`; adding those imports resolved the only test compile failure.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `06-02-PLAN.md` can now focus on Task Center inline actions and keyboard/mouse parity without inventing another controller or metadata transport path.
- `06-03-PLAN.md` can open pre-scoped Task Center views from tabbar affordances using the new TermWindow scope helpers.
- The remaining warnings in verification output are pre-existing dead-code/static-lib warnings, not blockers for Phase 06 UI work.

## Self-Check

PASSED

- Found summary artifact at `.planning/phases/06-operator-ui-surfaces/06-operator-ui-surfaces-01-SUMMARY.md`
- Verified task commits `68f2a3f`, `68555d2`, and `bb3f401` exist in git history

---
*Phase: 06-operator-ui-surfaces*
*Completed: 2026-03-27*
