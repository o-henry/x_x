---
phase: 05-hardening-and-compatibility
plan: 02
subsystem: testing
tags: [rust, mux, cli, rpc, lifecycle]
requires:
  - phase: 05-hardening-and-compatibility
    provides: "baseline failure-path task pane metadata retention from plan 01"
provides:
  - "Explicit task-pane retention and cleanup rules in mux-owned lifecycle state"
  - "Typed TaskPaneChanged transport refresh event for remote lifecycle consumers"
  - "Exact lifecycle CLI JSON contract coverage with validated rerun and respawn status semantics"
affects: [phase-05, task-pane-lifecycle, cli-contracts, transport]
tech-stack:
  added: []
  patterns:
    - "Mux lifecycle changes fan out through TaskPaneLifecycleChanged locally and TaskPaneChanged remotely"
    - "Lifecycle CLI commands validate exact machine-readable status values before rendering JSON"
key-files:
  created: []
  modified:
    - mux/src/lib.rs
    - mux/src/task_panes.rs
    - crates/codec/src/lib.rs
    - crates/wezterm-client/src/client.rs
    - crates/wezterm-mux-server-impl/src/sessionhandler.rs
    - crates/wezterm-mux-server-impl/src/dispatch.rs
    - kaku/src/cli/list_task_panes.rs
    - kaku/src/cli/set_remain_on_exit.rs
    - kaku/src/cli/rerun_pane.rs
    - kaku/src/cli/respawn_pane.rs
    - kaku/src/cli/silence_watchdog.rs
    - kaku/src/cli/pipe_pane.rs
key-decisions:
  - "Dead task-pane records are retained only for explicit lifecycle reasons: remain-on-exit, failure, or rerun metadata."
  - "Remote lifecycle consumers resync on a typed TaskPaneChanged transport event instead of inferring refresh timing."
  - "The stable respawn status string is `respawn`, and the CLI rejects unexpected rerun/respawn status values."
patterns-established:
  - "Treat stale live records and retained dead records differently: prune the former automatically, clear the latter explicitly."
  - "Lifecycle command tests assert exact top-level JSON fields so wrapper drift is caught in the command module that owns the contract."
requirements-completed: [COMP-03, COMP-04]
duration: 15min
completed: 2026-03-27
---

# Phase 05 Plan 02: Hardening and Compatibility Summary

**Task-pane lifecycle retention rules, typed refresh transport, and exact rerun/respawn CLI contracts for Phase 4 control-plane commands**

## Performance

- **Duration:** 15 min
- **Started:** 2026-03-27T10:42:00Z
- **Completed:** 2026-03-27T10:57:26Z
- **Tasks:** 2
- **Files modified:** 12

## Accomplishments
- Made mux retention behavior explicit so stale live task-pane records prune automatically while failed, rerunnable, or remain-on-exit dead panes stay queryable until explicit cleanup.
- Added a typed `TaskPaneChanged` transport event and wired remote client resync to the same lifecycle trigger local GUI consumers already use.
- Hardened all six Phase 4 lifecycle CLI modules with exact request/output contract tests and command-specific `rerun`/`respawn` status validation.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define lifecycle event, retention, and transport invariants across mux and RPC layers** - `b165e88`, `ad34343` (test, feat)
2. **Task 2: Stabilize every Phase 4 lifecycle CLI contract around the hardened semantics** - `85a5758`, `136a886` (test, feat)

## Files Created/Modified
- `mux/src/task_panes.rs` - codified retained-dead versus stale-live pruning behavior and extended lifecycle tests.
- `mux/src/lib.rs` - emitted lifecycle refreshes on clean record removal, aligned respawn status wording, and kept no-op lifecycle mutations from producing ambiguous semantics.
- `crates/codec/src/lib.rs` - added the typed `TaskPaneChanged` transport event to the lifecycle protocol surface.
- `crates/wezterm-client/src/client.rs` - resynced remote consumers on `TaskPaneChanged`.
- `crates/wezterm-mux-server-impl/src/dispatch.rs` - forwarded mux lifecycle notifications as typed transport refresh PDUs.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs` - treated `TaskPaneChanged` as unilateral transport traffic rather than a request.
- `kaku/src/cli/list_task_panes.rs` - asserted the exact stable list-task-panes JSON field set.
- `kaku/src/cli/set_remain_on_exit.rs` - asserted the exact stable remain-on-exit JSON field set.
- `kaku/src/cli/rerun_pane.rs` - enforced `status: "rerun"` before rendering machine-readable output.
- `kaku/src/cli/respawn_pane.rs` - enforced `status: "respawn"` before rendering machine-readable output.
- `kaku/src/cli/silence_watchdog.rs` - asserted the exact stable silence-watchdog JSON field set.
- `kaku/src/cli/pipe_pane.rs` - asserted the exact stable pipe-pane JSON field set.

## Decisions Made

- Retained dead task panes now survive pruning only when they still carry lifecycle value for users or downstream consumers; explicit cleanup owns their removal after that point.
- Remote refresh behavior stays event-driven and Kaku-native by using a typed lifecycle PDU instead of adding polling or broader multi-client infrastructure.
- `respawn` is the canonical machine-readable status string for respawn-pane so scripts can distinguish it from `rerun` without parsing prose.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The shared workspace had many unrelated tracked and untracked changes, including files this plan touched. The task was executed against the current workspace state and committed only plan-relevant files.
- Cargo commands briefly contended on package and artifact locks during verification because the workspace is shared; reruns completed successfully without changing scope.

## User Setup Required

None - no external service configuration required.

## Known Stubs

- `mux/src/lib.rs:2134` and `mux/src/lib.rs:2211` still contain pre-existing `TODO` notes about `TabId` disambiguation. They are unrelated to this lifecycle hardening plan.
- `mux/src/lib.rs:2259`, `mux/src/lib.rs:2266`, and `mux/src/lib.rs:2462` still contain pre-existing `FIXME` notes about clipboard and split-pane pixel dimensions. They do not block the task-pane lifecycle contract shipped here.
- `crates/wezterm-client/src/client.rs:201` still contains a pre-existing `FIXME` about unilateral traffic for non-real local domains. This plan narrowed lifecycle refresh behavior but did not widen client-domain architecture.

## Next Phase Readiness

- Phase 05-03 can build on stable lifecycle transport and CLI contracts without reopening rerun versus respawn wording or dead-record retention rules.
- Existing verification slices are now in place for mux lifecycle invariants, lifecycle CLI contracts, and transport compilation checks.

## Self-Check: PASSED

- Found `.planning/phases/05-hardening-and-compatibility/05-hardening-and-compatibility-02-SUMMARY.md`.
- Verified task commits `b165e88`, `ad34343`, `85a5758`, and `136a886` exist in git history.
