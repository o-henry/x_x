---
phase: 05-hardening-and-compatibility
verified: 2026-03-27T12:12:18Z
status: passed
score: 6/6 must-haves verified
re_verification:
  previous_status: gaps_found
  previous_score: 4/6
  gaps_closed:
    - "The control-plane lifecycle contract is regression-safe end-to-end, including live `respawn-pane` behavior."
  gaps_remaining: []
  regressions: []
human_verification: []
---

# Phase 05: Hardening And Compatibility Verification Report

**Phase Goal:** The control plane is regression-safe, documented, and still feels like Kaku while preserving the existing pane/tab/workspace baseline.
**Verified:** 2026-03-27T12:12:18Z
**Status:** passed
**Re-verification:** Yes - after gap closure

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Existing Kaku CLI pane-management commands and core pane or tab behavior still work without regressions. | ✓ VERIFIED | `cargo test --locked -p kaku compatibility_cli -- --nocapture` passed with 6 compatibility tests, and `05-UAT.md` records successful live `spawn`, `split-pane`, `send-text`, `get-text`, and `set-tab-title` runs. |
| 2 | Every new CLI surface added by the control plane has a stable machine-readable contract and documented behavior. | ✓ VERIFIED | `cargo test --locked -p kaku lifecycle_contracts -- --nocapture` passed with 14 tests, `kaku/src/cli/respawn_pane.rs` still enforces `status == "respawn"`, and the refreshed live UAT records `respawn-pane` returning `status: "respawn"`. |
| 3 | Each completed phase has targeted tests or checks plus documented limitations before it is considered done. | ✓ VERIFIED | `05-VALIDATION.md`, `05-UAT.md`, `05-LIMITATIONS.md`, and `05-CHANGED-FILES.md` all exist, and the full targeted Phase 5 suite is green in this re-verification. |
| 4 | The old failing-pane path no longer kills the GUI session and lifecycle inspection stays available. | ✓ VERIFIED | `05-UAT.md` records the `/bin/sh -lc 'echo PHASE4_FAILING_TASK; false'` path surviving without the old recursion and EOF chain while `list-task-panes` remained usable. |
| 5 | The end-to-end `respawn-pane` contract is now aligned across source, tests, and live runtime evidence. | ✓ VERIFIED | `05-UAT.md` now records a fresh rebuilt runtime returning `status: "respawn"`, `crates/codec/src/lib.rs` no longer carries the stale `respawned` round-trip sample, and the stale limitation was removed from `05-LIMITATIONS.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, and `PLANS.md`. |
| 6 | The fork still feels like Kaku in live use, with Task Center remaining an additive overlay rather than a new shell. | ✓ VERIFIED | `05-UAT.md` now records a live desktop `Shell -> Task Center` interaction that opened the overlay with real entries and returned cleanly to the normal Kaku terminal shell after accepting the selected row. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `mux/src/task_panes.rs` | Explicit dead-record retention and canonical rerun fallback | ✓ VERIFIED | The file still contains exact fallback coverage plus explicit pruning and retention tests. |
| `mux/src/lib.rs` | Mux-owned lifecycle bookkeeping and stable rerun or respawn emitters | ✓ VERIFIED | `rerun_task_pane()` and `respawn_task_pane()` both delegate through `spawn_task_pane_from_record(..., "rerun" | "respawn")`, and lifecycle mutations still notify through `TaskPaneLifecycleChanged`. |
| `crates/codec/src/lib.rs` | Typed lifecycle transport schema aligned with shipped status strings | ✓ VERIFIED | `TaskPaneChanged` remains present and the respawn sample now uses `status: "respawn"`. |
| `kaku/src/cli/respawn_pane.rs` | Stable respawn request and output contract | ✓ VERIFIED | The command still rejects unexpected statuses and prints only `pane_id`, `spawned_pane_id`, and `status`. |
| `kaku/tests/compatibility_cli.rs` | Compatibility regression slice for baseline commands | ✓ VERIFIED | The file covers `list`, `spawn`, `split-pane`, `send-text`, `get-text`, and `set-tab-title`, including behavior-level dispatch checks. |
| `kaku-gui/src/overlay/task_center.rs` | Task Center remains a consumer overlay over mux snapshots | ✓ VERIFIED | The overlay filters, formatting, clear-unread gating, and rerun gating are snapshot-driven and remain additive. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `mux/src/lib.rs` | `crates/wezterm-mux-server-impl/src/dispatch.rs` | `TaskPaneLifecycleChanged` bridged into `TaskPaneChanged` | ✓ WIRED | `dispatch.rs` still translates lifecycle notifications into the typed transport event. |
| `crates/wezterm-client/src/client.rs` | remote lifecycle PDUs | client resync on `TaskPaneChanged` | ✓ WIRED | The client still treats `TaskPaneChanged` as a refresh trigger instead of a polling fallback. |
| `kaku/src/cli/list_task_panes.rs` | typed lifecycle schema | JSON rendering of documented fields only | ✓ WIRED | The CLI renders directly from `TaskPaneState` without wrapper drift. |
| `mux/src/lib.rs` | `kaku/src/cli/respawn_pane.rs` | respawn status propagation | ✓ WIRED | Source emits `respawn`, CLI enforces `respawn`, and refreshed live UAT now matches the same value. |
| `kaku-gui/src/termwindow/mod.rs` | `kaku-gui/src/overlay/task_center.rs` | refresh and additive overlay behavior | ✓ WIRED | `TermWindow` still refreshes Task Center state on lifecycle notifications and uses overlay-specific focus routing helpers. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `mux/src/task_panes.rs` | task-pane records | real pane exits, live metadata refreshes, and rerun metadata capture | Yes | ✓ FLOWING |
| `kaku/src/cli/list_task_panes.rs` | `task_panes` | `client.list_task_panes()` over typed RPC | Yes | ✓ FLOWING |
| `kaku-gui/src/overlay/task_center.rs` | `entries` and `filtered_entries` | `Mux::task_center_snapshot()` via `TermWindow` cache refresh | Yes | ✓ FLOWING |
| `kaku/src/cli/respawn_pane.rs` | `response.status` | live mux and server response | Yes | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Mux hardening baseline | `cargo test --locked -p mux task_panes -- --nocapture` | 10 passed, 0 failed | ✓ PASS |
| Lifecycle CLI contract slice | `cargo test --locked -p kaku lifecycle_contracts -- --nocapture` | 14 passed, 0 failed | ✓ PASS |
| Legacy compatibility slice | `cargo test --locked -p kaku compatibility_cli -- --nocapture` | 6 passed, 0 failed | ✓ PASS |
| Task Center regression slice | `cargo test --locked -p kaku-gui task_center -- --nocapture` | 12 passed, 0 failed | ✓ PASS |
| Transport and GUI compile coverage | `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` | finished successfully | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `COMP-01` | 05-03 | Existing Kaku CLI pane-management commands stay usable | ✓ SATISFIED | Compatibility slice passed and live UAT exercised the baseline command family successfully. |
| `COMP-02` | 05-01, 05-03 | Existing pane, tab, focus, and workspace behavior stays usable | ✓ SATISFIED | Manual UAT proved the default workspace compatibility path and the failing-pane recovery path; the remaining broader coverage note is a scope limitation, not a contradictory failure. |
| `COMP-03` | 05-02, 05-04, 05-05 | New CLI surfaces have stable machine-readable contracts | ✓ SATISFIED | The stale respawn contradiction is closed in source, tests, and fresh live UAT evidence. |
| `COMP-04` | 05-01, 05-02, 05-04, 05-05 | Each phase has targeted verification before being done | ✓ SATISFIED | Validation, UAT, limitations, and changed-files artifacts exist and the targeted suite is green in this re-verification. |
| `COMP-05` | 05-03, 05-04 | Fork still feels like Kaku rather than a different shell | ✓ SATISFIED | The live desktop UAT now includes a visible Task Center overlay interaction and return to the standard Kaku terminal shell. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| `mux/src/lib.rs` | various pre-existing locations | legacy `TODO` and `FIXME` comments outside the Phase 5 contract path | ℹ️ Info | Existing cleanup debt, but not blocking the Phase 5 hardening goal. |
| `crates/wezterm-mux-server-impl/src/sessionhandler.rs` | 60 | unused helper warning during `cargo check` | ℹ️ Info | Harmless warning only; it does not affect the verified transport path. |

### Gaps Summary

The prior blocking gap is closed. The refreshed repository evidence agrees on the `respawn-pane` contract end-to-end: the CLI enforces `status: "respawn"`, the codec sample matches it, the docs no longer carry the stale mismatch, and the updated live UAT records a rebuilt runtime returning the same status.

The final qualitative check is also now complete. The live desktop UAT opened Task Center from the real `kaku-gui` menu, rendered real running and failed task entries, and returned to the ordinary terminal shell after accepting the selected row. That closes the last "still feels like Kaku" uncertainty, so the honest final status is **`passed`**.

---

_Verified: 2026-03-27T12:12:18Z_  
_Verifier: Claude (gsd-verifier)_
