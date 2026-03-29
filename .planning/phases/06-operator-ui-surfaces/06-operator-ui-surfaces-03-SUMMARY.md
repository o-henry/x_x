---
phase: 06-operator-ui-surfaces
plan: 03
subsystem: ui
tags: [rust, kaku-gui, tabbar, mouse, task-center]
requires:
  - phase: 06-01
    provides: scoped Task Center bootstrap helpers in TermWindow
provides:
  - compact actionable operator markers in the existing tabbar title path
  - dedicated tabbar hit regions for operator discoverability
  - marker click handoff into workspace-scoped Task Center views
affects: [06-04, operator-ui-surfaces, tabbar, task-center]
tech-stack:
  added: []
  patterns: [text-led tabbar operator markers, tabbar marker to Task Center handoff]
key-files:
  created: []
  modified:
    - kaku-gui/src/tabbar.rs
    - kaku-gui/src/termwindow/mouseevent.rs
    - kaku-gui/src/termwindow/render/fancy_tab_bar.rs
key-decisions:
  - "Used a compact ` · ops` suffix instead of adding badge-heavy tab chrome so the rail stays Kaku-native and text-first."
  - "Routed marker clicks into the existing workspace-scoped Task Center overlay rather than inventing a second operator surface."
patterns-established:
  - "Actionable tabbar affordances should reserve their own UI hit region so mouse routing can stay explicit without hijacking tab activation."
  - "Operator discoverability stays additive: passive state in the tabbar, workflow depth in Task Center."
requirements-completed: [UI-01, UI-02, UI-05]
duration: 5min
completed: 2026-03-27
---

# Phase 06 Plan 03: Tabbar Operator Discoverability Summary

**Compact ` · ops` tabbar markers with explicit hit regions and workspace-scoped Task Center handoff**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-27T22:46:13+09:00
- **Completed:** 2026-03-27T22:50:48+09:00
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added a compact, text-led operator marker to actionable tabs without turning the tabbar into a badge strip or workflow hub.
- Reserved a dedicated tabbar hit region for actionable operator markers so hover and click behavior can stay explicit.
- Routed actionable marker clicks through the existing Task Center bootstrap for the current workspace and kept the hover cursor informative-only.

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend the tabbar title path with compact operator markers that stay passive and Kaku-native** - `9df1e48` (test), `4a27ab1` (feat)
2. **Task 2: Route operator-marker clicks through existing tabbar mouse handling into scoped Task Center views** - `fc68f12` (test), `c711c58` (feat)

## Files Created/Modified

- `kaku-gui/src/tabbar.rs` - adds the actionable ` · ops` suffix logic, dedicated `OperatorMarker` hit regions, and tabbar coverage for passive vs actionable tabs.
- `kaku-gui/src/termwindow/mouseevent.rs` - routes operator-marker clicks into workspace-scoped Task Center opening and limits hand-cursor hover to actionable marker hits.
- `kaku-gui/src/termwindow/render/fancy_tab_bar.rs` - keeps the fancy tab bar renderer exhaustive and neutral for the new operator marker item.

## Decisions Made

- Used a compact ` · ops` suffix so the operator affordance reads like native tab metadata instead of a new UI control strip.
- Activated the clicked tab before handing off to Task Center when needed, but kept the actual workflow surface inside Task Center.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added minimal compile coverage for the new marker variant outside the tabbar path**
- **Found during:** Task 1 (Extend the tabbar title path with compact operator markers that stay passive and Kaku-native)
- **Issue:** Introducing `TabBarItem::OperatorMarker` made existing mouse and fancy-tab-bar match statements non-exhaustive.
- **Fix:** Added neutral compile-only handling in `mouseevent.rs` and `fancy_tab_bar.rs` so the new marker type could land without widening behavior early.
- **Files modified:** `kaku-gui/src/termwindow/mouseevent.rs`, `kaku-gui/src/termwindow/render/fancy_tab_bar.rs`
- **Verification:** `cargo check --locked -p kaku-gui`, `cargo fmt --all --check`
- **Committed in:** `4a27ab1` (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The deviation was required to keep the crate compiling once the new tabbar item existed. No product-scope creep.

## Issues Encountered

- The first `cargo test --locked -p kaku-gui tabbar -- --nocapture` run was blocked by missing Task Center test helper coverage that landed in Plan 06-02 shortly after this plan finished. The targeted tabbar suite was rerun after Wave 2 converged and is now green.

## Known Stubs

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The always-visible frame now advertises actionable operator state and can hand off into Task Center without adding a new surface.
- Wave 2 is fully verification-ready: the tabbar suite reran green after the Task Center helper work from Plan 06-02 landed, so Phase 06-04 can move straight into end-to-end verification and documentation closeout.

## Self-Check

PASSED

- Found `.planning/phases/06-operator-ui-surfaces/06-operator-ui-surfaces-03-SUMMARY.md`
- Found `.planning/phases/06-operator-ui-surfaces/deferred-items.md`
- Verified commits `9df1e48`, `4a27ab1`, `fc68f12`, and `c711c58` in `git log --oneline --all`

---
*Phase: 06-operator-ui-surfaces*
*Completed: 2026-03-27*
