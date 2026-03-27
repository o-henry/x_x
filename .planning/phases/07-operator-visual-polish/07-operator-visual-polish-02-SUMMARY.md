---
phase: 07-operator-visual-polish
plan: 02
subsystem: ui
tags: [rust, kaku-gui, tabbar, fancy-tab-bar, chrome]
requires:
  - phase: 06-03
    provides: compact actionable operator markers and Task Center handoff in the existing tabbar path
provides:
  - icon-first operator marker clusters that keep runtime tab chrome quieter than Phase 06
  - thinner fancy tab bar sizing and flatter tab silhouettes for restrained top chrome
  - tabbar-focused regression coverage for the quieter chrome metrics and height policy
affects: [07-03, operator-visual-polish, tabbar, top-chrome]
tech-stack:
  added: []
  patterns: [readable plain titles with compact runtime chrome markers, explicit chrome density constants]
key-files:
  created: []
  modified:
    - kaku-gui/src/tabbar.rs
    - kaku-gui/src/termwindow/render/fancy_tab_bar.rs
    - kaku-gui/src/termwindow/render/tab_bar.rs
key-decisions:
  - "Kept `compute_tab_plain_title` readable for rename/fallback paths while moving the rendered tabbar chrome to compact operator marker clusters."
  - "Lowered fancy-tab height and padding through named density constants so the top strip can be tested and tuned as restrained application chrome."
patterns-established:
  - "Operator state can stay discoverable through a separate marker hit region without forcing status text into every rendered tab title."
  - "Top-chrome density changes should be expressed as shared constants plus targeted tests instead of scattered magic numbers."
requirements-completed: [POLISH-01, POLISH-02, POLISH-03, POLISH-04]
duration: 53min
completed: 2026-03-28
---

# Phase 07 Plan 02: Operator Chrome Summary

**Compact operator marker clusters and a thinner fancy tab strip that push Kaku’s top chrome closer to the reference hierarchy without changing the control-plane workflow**

## Performance

- **Duration:** 53 min
- **Started:** 2026-03-28T00:54:00+0900
- **Completed:** 2026-03-28T01:47:39+0900
- **Tasks:** 2
- **Files modified:** 3 key files

## Accomplishments

- Split readable plain-title fallbacks from the rendered tabbar path so workspace metadata remains understandable outside the chrome while the runtime strip stays icon-first and quieter.
- Replaced the generic operator suffix with compact unread/status/progress marker clusters that render in a dedicated operator hit region instead of taking over the tab label.
- Reduced fancy-tab bar height, padding, and capsule weight through explicit chrome-density constants plus targeted tests for the new thinner top strip.

## Task Commits

Each task was committed atomically:

1. **Task 1: Replace text-heavy tabbar suffixes with compact icon-first operator and metadata markers** - `3f4ab58` (feat)
2. **Task 2: Tighten the fancy tab bar into restrained top chrome with lower height and less capsule treatment** - `ec7f37f` (feat)

Additional cleanup:

3. **Post-task formatting pass for phase files** - `7a11d21` (refactor)

## Files Created/Modified

- `kaku-gui/src/tabbar.rs` - keeps `compute_tab_plain_title` readable while moving the runtime tabbar surface to compact operator marker clusters and adds regression coverage for the new marker behavior.
- `kaku-gui/src/termwindow/render/fancy_tab_bar.rs` - introduces shared chrome-density constants and flatter tab/operator marker styling for the quieter top strip.
- `kaku-gui/src/termwindow/render/tab_bar.rs` - lowers the fancy tab-bar height multiplier and adds a regression test that locks the thinner height policy.

## Decisions Made

- Kept the readable workspace metadata fallback in `compute_tab_plain_title` so rename/title paths still expose text, while the rendered tabbar path now suppresses text-heavy metadata and uses icon-first markers.
- Treated chrome density as explicit policy by naming the line-height, padding, and corner constants instead of tuning the top strip only through ad hoc pixel changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added a follow-up formatting commit after the first chrome commit**
- **Found during:** Task 2 (Tighten the fancy tab bar into restrained top chrome with lower height and less capsule treatment)
- **Issue:** `cargo fmt --all --check` still reported formatting drift in the three phase files after the feature commits.
- **Fix:** Ran `rustfmt` directly on the phase files and committed the formatting-only diff as a small follow-up refactor commit.
- **Files modified:** `kaku-gui/src/tabbar.rs`, `kaku-gui/src/termwindow/render/fancy_tab_bar.rs`, `kaku-gui/src/termwindow/render/tab_bar.rs`
- **Verification:** `cargo fmt --all --check`, `cargo test --locked -p kaku-gui tabbar -- --nocapture`, `cargo check --locked -p kaku-gui`
- **Committed in:** `7a11d21`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** The follow-up commit was mechanical and kept the planned scope intact.

## Issues Encountered

- The worktree was already dirty before execution, including the target chrome files. I treated the current working tree as the source of truth and staged only the phase files for this plan’s follow-up commits.
- The Task 2 feature commit picked up adjacent `termwindow` files that were already moving with the broader Phase 07 rail work. I left that commit in place and constrained the later formatting cleanup back to the three plan files rather than rewriting shared history mid-phase.
- I did not launch a fresh runtime for screenshot capture in this turn, so the plan’s reference-frame comparison remains a truthful manual follow-up rather than a claimed completion artifact.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The tabbar and top chrome are materially quieter and now expose operator state through compact marker clusters plus a thinner fancy tab strip, which gives Phase 07-03 a better shell hierarchy to build on.
- A fresh runtime screenshot comparison against `/tmp/kaku_ref_003.png` and `/tmp/kaku_ref_030.png` is still needed before claiming the chrome fully matches the reference density.

## Self-Check

PASSED

- Found `.planning/phases/07-operator-visual-polish/07-operator-visual-polish-02-SUMMARY.md`
- Verified commits `3f4ab58`, `ec7f37f`, and `7a11d21` in `git log --oneline --all`

---
*Phase: 07-operator-visual-polish*
*Completed: 2026-03-28*
