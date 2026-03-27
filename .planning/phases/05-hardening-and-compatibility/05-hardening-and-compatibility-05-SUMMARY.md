---
phase: 05-hardening-and-compatibility
plan: 05
subsystem: testing
tags: [rust, cli, mux, uat, compatibility, respawn]
requires:
  - phase: 05-04
    provides: "Phase 5 hardening docs, lifecycle contract tests, and the initial live UAT artifact that exposed the respawn mismatch"
provides:
  - "Fresh live respawn-pane evidence from rebuilt binaries and a newly published gui socket"
  - "Phase 5 closeout docs aligned to the verified respawn contract"
  - "Removal of the stale respawned sample that contradicted the shipped contract"
affects: [phase-05-closeout, lifecycle-contracts, respawn-pane]
tech-stack:
  added: []
  patterns: ["Use a freshly published gui socket for live Kaku UAT when the default published socket is stale", "Keep lifecycle contract strings normalized across codec samples, CLI validation, and closeout docs"]
key-files:
  created: [.planning/phases/05-hardening-and-compatibility/05-hardening-and-compatibility-05-SUMMARY.md]
  modified:
    - crates/codec/src/lib.rs
    - .planning/phases/05-hardening-and-compatibility/05-UAT.md
    - .planning/phases/05-hardening-and-compatibility/05-LIMITATIONS.md
    - docs/KAKU_CONTROL_PLANE_SPEC.md
    - PLANS.md
key-decisions:
  - "Treated the freshly rebuilt live gui socket as the source of truth for the respawn check instead of trusting the stale published socket path"
  - "Removed the respawn limitation from Phase 5 docs only after the rebuilt runtime returned status=respawn end-to-end"
patterns-established:
  - "Live contract mismatches must be rechecked against a fresh kaku-gui start before changing runtime code"
  - "Phase closeout docs should preserve only still-open boundaries, not resolved historical failures"
requirements-completed: [COMP-03, COMP-04]
duration: 18min
completed: 2026-03-27
---

# Phase 05 Plan 05: Hardening And Compatibility Summary

**Fresh rebuilt `respawn-pane` UAT now returns `status: "respawn"` end-to-end, and Phase 5 closeout docs no longer carry the stale mismatch**

## Performance

- **Duration:** 18 min
- **Started:** 2026-03-27T11:40:00Z
- **Completed:** 2026-03-27T11:57:58Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Rebuilt `kaku`, `kaku-gui`, and `wezterm-mux-server-impl`, launched a fresh live GUI session, and reran the retained-pane respawn flow against the newly published `gui-sock-2572` socket.
- Confirmed the live runtime returns the documented machine-readable respawn contract, then rewrote `05-UAT.md` with the exact commands, socket details, and passing output.
- Removed the stale `respawned` limitation language from Phase 5 closeout artifacts and normalized the contradictory codec round-trip sample to `respawn`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Rebuild the live respawn path, prove the runtime status, and normalize the remaining emitter only if the mismatch still reproduces** - `ead2096` (fix)
2. **Task 2: Update the closeout docs and limitations to match the verified respawn result** - `6f188aa` (docs)

## Files Created/Modified
- `.planning/phases/05-hardening-and-compatibility/05-hardening-and-compatibility-05-SUMMARY.md` - Plan summary and execution record
- `crates/codec/src/lib.rs` - Normalized the respawn round-trip sample to the shipped `respawn` status
- `.planning/phases/05-hardening-and-compatibility/05-UAT.md` - Replaced the stale failing respawn artifact with the rebuilt live rerun evidence
- `.planning/phases/05-hardening-and-compatibility/05-LIMITATIONS.md` - Removed the resolved respawn mismatch limitation
- `docs/KAKU_CONTROL_PLANE_SPEC.md` - Updated Phase 5 boundaries to reflect the passing live respawn contract
- `PLANS.md` - Replaced the stale residual limitation with the remaining manual-coverage boundary

## Decisions Made

- Used the newly published GUI socket from the fresh `kaku-gui start --always-new-process` session for live verification because the previously published default socket was stale and no longer connectable.
- Kept runtime code untouched after the rebuilt live run returned `status: "respawn"`; only contradictory evidence and closeout docs were updated.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Normalized stale respawn sample data in codec round-trip coverage**
- **Found during:** Task 1 (live respawn rerun)
- **Issue:** `crates/codec/src/lib.rs` still used `status: "respawned"` in the respawn round-trip sample, which contradicted the shipped CLI contract and the fresh live runtime result.
- **Fix:** Changed the sample response to `status: "respawn"` so source examples, tests, and live evidence agree.
- **Files modified:** `crates/codec/src/lib.rs`
- **Verification:** `cargo test --locked -p kaku lifecycle_contracts -- --nocapture`, `cargo test --locked -p mux task_panes -- --nocapture`, and `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui`
- **Committed in:** `ead2096`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** The auto-fix was narrowly scoped to the verified respawn mismatch story. No runtime behavior or unrelated Phase 5 surfaces were widened.

## Issues Encountered

- The previously published default GUI socket pointed at a stale `gui-sock-88268`, so direct `kaku cli` calls failed to connect until the live UAT was rerun against the fresh socket published by the rebuilt `kaku-gui` session.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 5’s respawn gap is closed: runtime evidence, contract enforcement, and closeout docs now agree on `status: "respawn"`.
- Remaining live boundaries are limited to broader desktop/manual coverage such as visible Task Center interaction and wider multi-window/workspace behavior.

## Self-Check: PASSED

- Found summary file: `.planning/phases/05-hardening-and-compatibility/05-hardening-and-compatibility-05-SUMMARY.md`
- Found task commits: `ead2096`, `6f188aa`
