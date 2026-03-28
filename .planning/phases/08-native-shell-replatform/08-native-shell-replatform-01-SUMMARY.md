---
phase: 08-native-shell-replatform
plan: 01
subsystem: ui
tags: [rust, gtk4, libadwaita, mux, runtime-bootstrap, native-shell]
requires:
  - phase: 06-operator-ui-surfaces
    provides: mux-owned notification, metadata, and task-pane control-plane state
provides:
  - lib-backed native shell modules for controller, snapshot, and runtime seams
  - explicit shared runtime bootstrap path for the native shell
  - Wave 0 shell contract tests for actions, layout, and bootstrap ownership
affects: [08-02-PLAN.md, 08-03-PLAN.md, native-shell-parity, runtime-ownership]
tech-stack:
  added: []
  patterns: [library-backed shell entry, shared runtime bootstrap result, test-first shell seams]
key-files:
  created:
    - kaku-native-shell/src/lib.rs
    - kaku-native-shell/src/app_controller.rs
    - kaku-native-shell/src/runtime_bridge.rs
    - kaku-native-shell/src/snapshot.rs
    - kaku-native-shell/tests/shell_actions.rs
    - kaku-native-shell/tests/shell_snapshot.rs
    - kaku-native-shell/tests/runtime_bootstrap.rs
  modified:
    - kaku-native-shell/src/main.rs
    - crates/kaku-runtime/src/lib.rs
    - kaku-gui/src/main.rs
key-decisions:
  - "The native shell now boots through a first-class shared runtime seam in `kaku-runtime`, while `kaku-gui` stays on the same publish path as a fallback companion."
  - "Wave 0 contracts live in a library-backed shell crate so controller, snapshot, and bootstrap behavior can be tested without treating GTK startup as the only seam."
patterns-established:
  - "Thin binary entry: `main.rs` delegates to `kaku_native_shell::run()`."
  - "Shared runtime authority: bootstrap functions return both mux ownership and published socket context."
requirements-completed: [UI-01, UI-05, POLISH-03]
duration: 8 min
completed: 2026-03-28
---

# Phase 08 Plan 01: Native Shell Runtime Authority Summary

**Library-backed native shell seams with shared runtime bootstrap ownership and Wave 0 contract tests for actions, layout, and socket authority**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-28T03:14:30Z
- **Completed:** 2026-03-28T03:22:30Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments

- Split `kaku-native-shell` out of its monolithic `main.rs` into exported controller, snapshot, and runtime modules that can be exercised without launching GTK.
- Added the Phase 08 Wave 0 contract tests named in validation: `shell_actions_visible`, `shell_layout_contract`, and `runtime_bootstrap_ownership`.
- Moved native-shell bootstrap ownership onto an explicit shared runtime path in `crates/kaku-runtime` while keeping `kaku-gui` compiling against the same publish seam as a fallback companion.

## Task Commits

Each task was committed atomically:

1. **Task 1: Create the Wave 0 shell contracts and test harness before moving startup authority** - `0893324` (test), `1f9c8b5` (feat)
2. **Task 2: Move shared runtime/bootstrap authority into the native shell path while keeping `kaku-gui` as a fallback companion** - `c6ede11` (feat)

_Note: Task 1 followed TDD with a failing harness commit before the module split implementation._

## Files Created/Modified

- `kaku-native-shell/src/main.rs` - Thin binary entry that delegates to the library shell runner.
- `kaku-native-shell/src/lib.rs` - Public shell module surface and application entrypoint.
- `kaku-native-shell/src/app_controller.rs` - Extracted GTK controller, action inventory, and shell rendering logic.
- `kaku-native-shell/src/runtime_bridge.rs` - Native-shell bootstrap plan/result types and shared-runtime delegation.
- `kaku-native-shell/src/snapshot.rs` - Snapshot derivation and layout contract types for Wave 0 tests.
- `kaku-native-shell/tests/shell_actions.rs` - Operator action contract test harness.
- `kaku-native-shell/tests/shell_snapshot.rs` - Layout/snapshot contract harness.
- `kaku-native-shell/tests/runtime_bootstrap.rs` - Runtime ownership proof harness.
- `crates/kaku-runtime/src/lib.rs` - Shared bootstrap result/publish helpers plus explicit native-shell bootstrap path.
- `kaku-gui/src/main.rs` - Fallback companion path updated to reuse the shared runtime publish helper.

## Decisions Made

- Used `kaku-native-shell` as a real library target instead of keeping test-only helpers in `main.rs`, because Phase 08 validation requires non-GTK seams to exist on disk and compile.
- Kept `kaku-gui` on the same runtime publish helper instead of forking the startup path, so fallback compatibility remains coupled to the shared runtime contract rather than drifting immediately.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- Cargo package-cache locking showed up during parallel verification because this executor is running alongside other agents. Verification was rerun sequentially where needed and all planned checks completed cleanly.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 08 now has the Wave 0 seams required for parity work: controller actions, layout contracts, and runtime authority proof all compile as dedicated tests.
- The native shell still uses transitional timer-driven rerendering; later plans can now replace that scaffolding with mux-driven refresh without touching bootstrap authority again.

## Self-Check: PASSED

- Found summary file on disk.
- Verified task commit hashes `0893324`, `1f9c8b5`, and `c6ede11` in git history.

---
*Phase: 08-native-shell-replatform*
*Completed: 2026-03-28*
