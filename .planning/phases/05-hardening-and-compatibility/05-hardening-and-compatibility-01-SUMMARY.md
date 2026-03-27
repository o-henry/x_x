---
phase: 05-hardening-and-compatibility
plan: 01
subsystem: testing
tags: [rust, mux, task-panes, lifecycle, kaku-gui, compatibility]
requires:
  - phase: 04-task-pane-lifecycle
    provides: "mux-owned task-pane records, rerun metadata capture, and lifecycle CLI contracts"
provides:
  - "Canonical `/bin/sh -lc ...` rerun fallback coverage for mux task panes"
  - "Failure-path mux tests that keep failed pane records inspectable in the quick suite"
  - "Immediate-exit cwd retention so failed panes preserve inspectable lifecycle context"
affects: [phase-05-hardening-and-compatibility, mux, kaku-gui, lifecycle-cli]
tech-stack:
  added: []
  patterns: ["TDD coverage in the filtered `cargo test -p mux task_panes` slice", "Failure-path lifecycle metadata falls back to the last known live snapshot"]
key-files:
  created: []
  modified: [mux/src/task_panes.rs, mux/src/lib.rs]
key-decisions:
  - "Treat `/bin/sh -lc '…'` as the canonical fallback rerun command string because it matches `CommandBuilder` normalization and the rerun/respawn spawn path."
  - "Preserve the last known working directory on immediate task-pane failures so `list-task-panes` and later rerun validation keep usable lifecycle context."
patterns-established:
  - "Keep mux lifecycle truth authoritative and test failure retention inside the same filtered quick suite future phases depend on."
  - "Prefer last-known live metadata over dropping inspectability when fast failures cannot report fresh exit data."
requirements-completed: [COMP-02, COMP-04]
duration: 5min
completed: 2026-03-27
---

# Phase 05 Plan 01: Hardening Baseline Summary

**Mux lifecycle hardening with a canonical `/bin/sh -lc` rerun fallback and preserved cwd metadata for immediate failed task panes**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-27T10:35:00Z
- **Completed:** 2026-03-27T10:40:08Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Repaired the red `cargo test --locked -p mux task_panes -- --nocapture` slice and locked the fallback rerun command to one exact canonical string.
- Added mux-level failure-path coverage so failed task-pane records stay queryable in the quick suite until explicit cleanup.
- Hardened `record_task_pane_exit` to preserve the last known cwd when a pane fails before exit metadata can refresh.

## Task Commits

Each task was committed atomically:

1. **Task 1: Repair the red lifecycle quick suite and lock the canonical rerun fallback shape**
   `b9703c9` (test), `10572d5` (feat)
2. **Task 2: Harden the failing spawn path so GUI/runtime survives and lifecycle inspection stays available**
   `10f7902` (test), `4f31a89` (fix)

## Files Created/Modified
- `mux/src/task_panes.rs` - Defines the exact fallback rerun command contract used by the quick suite.
- `mux/src/lib.rs` - Adds mux-level failure retention tests and preserves prior cwd metadata on immediate failed exits.

## Decisions Made
- Canonicalized the fallback rerun string as `/bin/sh -lc 'echo PHASE4_FAILING_TASK; false'` because that is the stable `CommandBuilder` output already consumed cleanly by `rerun-pane` and `respawn-pane`.
- Hardened the mux exit path instead of introducing a parallel GUI cache so lifecycle inspection stays mux-owned and Kaku-native.
- Left `assets/macos/Kaku.app/Contents/Resources/kaku.lua` unchanged in this plan because the current guarded implementation already satisfied the narrow safety goal during verification; no broader Lua/UI behavior was introduced.

## Deviations from Plan

None - plan executed as written. The only adjustment was proving that no additional `kaku.lua` edit was required once the mux failure-path hardening and current guard were verified together.

## Issues Encountered

- The filtered quick suite only runs tests whose names include `task_panes`, so the new mux-level failure-retention coverage was renamed to stay inside the required verification slice.
- The worktree already contained unrelated tracked and untracked changes outside this plan. They were left untouched.

## User Setup Required

None - no external service configuration required.

## Known Stubs

None.

## Next Phase Readiness

- Phase 05 now has a trustworthy mux quick suite for later hardening waves.
- Failed task panes keep enough lifecycle context for later rerun/respawn and GUI verification work.
- Residual warnings in `wezterm-mux-server-impl` and `kaku-gui` remain pre-existing and out of scope for this plan.

## Self-Check: PASSED

- Found `.planning/phases/05-hardening-and-compatibility/05-hardening-and-compatibility-01-SUMMARY.md`
- Found commits `b9703c9`, `10572d5`, `10f7902`, and `4f31a89` in git history
