---
phase: 07-operator-visual-polish
plan: 01
subsystem: ui
tags: [rust, kaku-gui, operator-rail, task-center, visual-polish]
requires:
  - phase: 06-operator-ui-surfaces
    provides: "Scoped Task Center workflows and persistent operator-nav activation paths"
provides:
  - "Slim full-height operator rail geometry with quieter active treatment"
  - "Regression coverage for operator-nav rendering and hit-target routing"
  - "Preserved scoped Task Center handoff from compact persistent navigation"
affects: [07-operator-visual-polish, task-center, operator-nav, runtime-verification]
tech-stack:
  added: []
  patterns:
    - "Source-level renderer regressions protect compact rail structure and explicit hit targets"
    - "Operator rail sizing is enforced through window-level width constants instead of overlay-only styling"
key-files:
  created: []
  modified:
    - kaku-gui/src/termwindow/mod.rs
    - kaku-gui/src/termwindow/render/paint.rs
    - kaku-gui/src/termwindow/mouseevent.rs
    - kaku-gui/src/tabbar.rs
key-decisions:
  - "The persistent rail stays edge-attached and icon-first, with geometry reduced in TermWindow constants rather than by adding another overlay layer."
  - "Compact rail regressions assert against the old Phase 06 header/panel structure so future polish cannot silently widen back into a branded side card."
  - "Runtime screenshot review remains a blocking expectation, but this execution records the macOS window-surfacing limitation honestly instead of claiming a visual sign-off."
patterns-established:
  - "Pattern: keep operator-nav hit regions explicit through UIItemType::OperatorNav even while chrome density changes."
  - "Pattern: use narrow, subdued active markers instead of rounded filled cards for persistent operator chrome."
requirements-completed: [POLISH-02, POLISH-03, POLISH-04]
duration: 10min
completed: 2026-03-28
---

# Phase 07 Plan 01: Operator Rail Polish Summary

**Slim persistent operator rail with compact hit-safe rows and preserved scoped Task Center activation**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-27T16:43:28Z
- **Completed:** 2026-03-27T16:53:49Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added operator-nav regression coverage that protects compact one-line rows, explicit hit targets, and the existing activation handoff.
- Reduced the always-visible rail width and removed the rounded card-like active treatment so the left edge reads closer to the reference structure.
- Preserved the scoped Task Center behavior Phase 06 shipped while tightening the rail’s geometry and click path safeguards.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add rail-specific render and hit-target regression coverage before changing the persistent navigation geometry** - `2fd3644` (test)
2. **Task 2: Rewrite the persistent rail into the reference-first slim navigation surface while preserving scoped Task Center behavior** - `72a31ac` (test), `89c0971` (feat)

_Note: Task 2 followed the TDD pattern with a failing rail-contract commit before the renderer/geometry changes landed._

## Files Created/Modified

- `kaku-gui/src/termwindow/render/paint.rs` - Added source-level operator-nav regressions and tightened the rail renderer, spacing, and active marker treatment.
- `kaku-gui/src/termwindow/mouseevent.rs` - Added operator-nav click-path regression coverage to preserve activation routing.
- `kaku-gui/src/termwindow/mod.rs` - Reduced persistent rail width/min-window gating and added the slim-rail contract test.
- `kaku-gui/src/tabbar.rs` - Restored the plain workspace metadata suffix helper needed for the current branch state to compile while rail tests were added.

## Decisions Made

- Kept the operator rail as persistent native window chrome instead of introducing any new overlay surface.
- Used source-based renderer assertions to guard against regressing back to the old `KAKU`/workspace header block.
- Tuned the active state toward a narrow, quieter highlight so the left rail reads as structure rather than a sidebar card.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Restored the plain workspace metadata suffix helper needed by the current branch**
- **Found during:** Task 1 (rail-specific render and hit-target regression coverage)
- **Issue:** The active branch state would not compile the new operator-nav slice without `apply_workspace_metadata_plain_suffix` being present in `tabbar.rs`.
- **Fix:** Reintroduced the helper alongside the regression additions so the targeted `operator_nav` test slice could compile and run.
- **Files modified:** `kaku-gui/src/tabbar.rs`
- **Verification:** `cargo test --locked -p kaku-gui operator_nav -- --nocapture`, `cargo check --locked -p kaku-gui`
- **Committed in:** `2fd3644`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The deviation was required to keep the current branch buildable and did not widen scope beyond enabling the planned rail work.

## Issues Encountered

- Parallel Phase 07 work landed on `main` during execution, so this summary records only the rail-specific commits and avoids touching unrelated visual-polish edits.
- Runtime screenshot verification was attempted twice; the first capture revealed a stale binary, and the rebuilt binary could not be surfaced reliably through macOS window automation for a trustworthy final screenshot.
- `cargo fmt --all --check` remains noisy because unrelated formatting differences already exist in the shared worktree, including files being touched by parallel agents.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The left rail now has executable regression coverage and slimmer geometry, so later Phase 07 work can polish top chrome and container hierarchy without re-opening the click-path contract.
- A clean runtime screenshot pass is still worth re-running once the Kaku window can be surfaced reliably, since the visual gate is intentionally stricter than the current automated tests.

## Self-Check

PASSED

---
*Phase: 07-operator-visual-polish*
*Completed: 2026-03-28*
