---
phase: 08-native-shell-replatform
plan: 02
subsystem: ui
tags: [rust, gtk4, libadwaita, mux, native-shell]
requires:
  - phase: 08-01
    provides: library-backed native shell scaffold, runtime bootstrap seam, and shell layout contracts
provides:
  - mux-derived runtime snapshots that can be tested without `Mux::get()`
  - classified mux notification refresh routing for the native shell controller
  - reusable shell action seam for unread and workspace metadata mutations
affects: [08-03, 08-04, native-shell-controller, shell-parity]
tech-stack:
  added: [chrono]
  patterns: [source-derived shell snapshots, queued GTK refresh dispatch, mux-backed shell action targets]
key-files:
  created:
    - kaku-native-shell/src/actions.rs
  modified:
    - kaku-native-shell/Cargo.toml
    - Cargo.lock
    - kaku-native-shell/src/app_controller.rs
    - kaku-native-shell/src/snapshot.rs
    - kaku-native-shell/tests/shell_snapshot.rs
    - kaku-native-shell/tests/shell_actions.rs
key-decisions:
  - "Native shell refreshes now flow through classified mux notifications and a GTK-side queue instead of a correctness timer."
  - "Unread and workspace metadata mutations live behind `ShellActionTarget` so later codec/client migration stays visible."
patterns-established:
  - "RuntimeSnapshot::from_source keeps snapshot derivation testable and prevents GTK-local truth from creeping into shell state."
  - "A hidden window refresh action can safely drain mux-driven refresh scopes on the GTK main loop without capturing the controller across threads."
  - "ShellActionContext centralizes visible workspace status, progress, unread IDs, and log message derivation before mutation."
requirements-completed: [UI-02, UI-03, UI-04, UI-05]
duration: 7min
completed: 2026-03-28
---

# Phase 08 Plan 02: Event-Driven Native Shell Controller Summary

**Mux-derived shell snapshots, notification-driven refresh dispatch, and a reusable action seam for metadata and unread mutations**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-28T03:32:23Z
- **Completed:** 2026-03-28T03:39:42Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Added native-shell tests that lock snapshot derivation to mux-shaped source data and define targeted refresh classification.
- Removed the old 2-second authoritative refresh loop from the controller and replaced it with mux subscription dispatch onto GTK.
- Routed workspace metadata and visible-unread mutations through `actions.rs` instead of direct inline `Mux` calls in each controller handler.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add failing snapshot/action tests that lock the shell to mux truth and event-driven refresh** - `a44d895` (`test`)
2. **Task 2: Remove correctness-critical polling and replace it with mux subscription driven refresh** - `9326a7b` (`feat`)
3. **Task 3: Route unread and metadata mutations through a reusable shell action layer** - `6f24ee8` (`feat`)

Additional cleanup:

1. **Post-task formatting:** `2a07093` (`refactor`)

## Files Created/Modified

- `kaku-native-shell/src/actions.rs` - Shared `ShellAction`, `ShellActionContext`, and `ShellActionTarget` seam with a concrete `Mux` implementation.
- `kaku-native-shell/src/app_controller.rs` - Subscription-driven refresh queue, GTK-side refresh action, and controller delegation into the action seam.
- `kaku-native-shell/src/snapshot.rs` - Source-backed snapshot derivation and `MuxNotification` refresh classification.
- `kaku-native-shell/tests/shell_snapshot.rs` - Snapshot truth and refresh-classification coverage for the native shell.
- `kaku-native-shell/tests/shell_actions.rs` - Action seam coverage, including real mux-backed metadata mutation assertions.
- `kaku-native-shell/Cargo.toml` - Direct `chrono` access for the native-shell integration tests.

## Decisions Made

- Used `RuntimeSnapshot::from_source` as the main shell snapshot API so tests can prove view-model behavior without relying on global mux state.
- Batched mux refresh scopes through a queue plus a hidden window action because the installed `glib` version lacks the `MainContext::channel` path used in newer examples.
- Kept selection coherence tied to `WorkspaceList` refresh scopes so external workspace changes reset the shell onto the mux active workspace.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added direct `chrono` access for native-shell integration tests**
- **Found during:** Task 1 (snapshot/action test harness)
- **Issue:** The new integration tests needed `DateTime<Utc>` constructors for mux record fixtures, but `kaku-native-shell` did not expose `chrono` directly.
- **Fix:** Added `chrono.workspace = true` to `kaku-native-shell/Cargo.toml` and refreshed `Cargo.lock` offline before rerunning locked test commands.
- **Files modified:** `kaku-native-shell/Cargo.toml`, `Cargo.lock`
- **Verification:** All named `shell_uses_mux_truth`, `shell_snapshot_context_updates`, `shell_action_mutations`, and `shell_metadata_actions` tests pass with `--locked`.
- **Committed in:** `a44d895`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The auto-fix was required to let the new native-shell tests compile. No scope creep beyond the test harness.

## Issues Encountered

- `glib 0.22.3` does not provide `MainContext::channel`, so the first GTK handoff approach failed at compile time. The controller switched to `MainContext::invoke` plus a queued hidden window action, preserving event-driven refresh without adding polling back.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The native shell now has the controller seams needed for deeper parity work without reintroducing shell-owned state.
- Phase 08-03 can build on `refresh_scope_for_notification`, `RuntimeSnapshot::from_source`, and `ShellActionTarget` instead of coupling new UI work directly to `Mux`.
- Notification read mutations are seam-tested through `ShellActionTarget`, but full end-to-end notification-record mutation still depends on richer live notification fixtures than this wave needed.

## Known Stubs

None.

## Self-Check: PASSED

- Found summary artifact at `.planning/phases/08-native-shell-replatform/08-native-shell-replatform-02-SUMMARY.md`
- Verified task/refactor commits `a44d895`, `9326a7b`, `6f24ee8`, and `2a07093` exist in git history
