---
phase: 05-hardening-and-compatibility
plan: 03
subsystem: testing
tags: [rust, cli, kaku-gui, task-center, compatibility, regression]
requires:
  - phase: 05-02
    provides: lifecycle CLI contracts and stable task-pane mutation payloads
provides:
  - legacy pane-management CLI compatibility regression coverage
  - Task Center consumer-overlay compatibility guardrails
  - GUI refresh and workspace-routing checks for lifecycle notifications
affects: [phase-05-hardening-and-compatibility, cli, task-center, termwindow]
tech-stack:
  added: []
  patterns: [path-included CLI compatibility harness for binary crate tests, mux-snapshot consumer assertions for Task Center]
key-files:
  created: [kaku/tests/compatibility_cli.rs]
  modified: [kaku-gui/src/termwindow/mod.rs, kaku-gui/src/overlay/task_center.rs]
key-decisions:
  - "Used a dedicated integration-style compatibility slice in kaku/tests instead of widening the kaku binary into a library crate."
  - "Kept Task Center hardening in tests around mux snapshot consumption and routing helpers rather than adding new GUI controller logic."
patterns-established:
  - "Compatibility tests for legacy CLI commands can include real command source behind a narrow local harness when the crate is binary-only."
  - "Task Center hardening should prove refresh and routing behavior with unit tests while preserving mux-owned state and Kaku-native overlays."
requirements-completed: [COMP-01, COMP-02, COMP-05]
duration: 11min
completed: 2026-03-27
---

# Phase 05 Plan 03: Hardening and Compatibility Summary

**Legacy pane-management CLI compatibility coverage plus Task Center refresh and routing guardrails that keep the control plane additive to Kaku**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-27T11:00:00Z
- **Completed:** 2026-03-27T11:11:04Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added a dedicated `kaku/tests/compatibility_cli.rs` slice that covers `list`, `spawn`, `split-pane`, `send-text`, `get-text`, `set-tab-title`, and coexistence with `list-task-panes`.
- Verified behavior-level legacy command expectations without introducing a new CLI harness or changing Kaku’s binary-crate shape.
- Added GUI regression checks in Task Center and `TermWindow` so lifecycle refreshes remain additive and workspace/window routing stays on existing Kaku paths.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add a focused compatibility regression slice for legacy pane-management CLI commands** - `8426361` (test)
2. **Task 2: Guard GUI refresh behavior so Task Center stays additive and baseline navigation remains intact** - `53543aa` (test)

## Files Created/Modified
- `kaku/tests/compatibility_cli.rs` - Dedicated compatibility slice for baseline pane-management CLI contracts and dispatch-shape preservation.
- `kaku-gui/src/termwindow/mod.rs` - Added lifecycle refresh and task-center routing regression tests.
- `kaku-gui/src/overlay/task_center.rs` - Added snapshot-consumer regression tests for Task Center search and row formatting.

## Decisions Made
- Used a local test harness that includes the real CLI command source files so the new compatibility slice could verify command behavior without refactoring `kaku` into a library crate.
- Kept the GUI hardening focused on unit tests around existing mux-backed helpers, which preserves Task Center as a consumer overlay instead of expanding it into a controller surface.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- `cargo test --locked -p kaku compatibility_cli -- --nocapture` initially matched zero tests because the dedicated slice did not exist yet; this was the intended TDD red step.
- The first version of the CLI harness hit Clap debug assertions for trailing command arguments, so the behavior-level spawn and split checks were moved to module-local test constructors that still exercise the real command fields.
- Cargo briefly waited on the shared artifact lock in this parallel workspace; verification completed successfully after the lock cleared.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 05 now has explicit preservation checks for baseline CLI behavior and Task Center additivity, which supports the remaining hardening/docs closeout work in `05-04`.
- The CLI compatibility slice emits only dead-code warnings from included command modules during tests; behavior is verified, but a future library extraction would be cleaner if broader reuse is ever needed.

## Self-Check

PASSED

- Found summary file: `.planning/phases/05-hardening-and-compatibility/05-hardening-and-compatibility-03-SUMMARY.md`
- Found task commits: `8426361`, `53543aa`

---
*Phase: 05-hardening-and-compatibility*
*Completed: 2026-03-27*
