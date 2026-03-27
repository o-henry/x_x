---
phase: 01-notification-routing
plan: 02
subsystem: transport
tags: [codec, rpc, mux-server, notifications, rust]
requires:
  - phase: 01-01
    provides: mux-owned notification APIs and unread routing helpers
provides:
  - typed notification PDUs for workspace/tab/pane notification flows
  - client RPC wrappers for notification commands
  - mux-server dispatch for create/list/mutate/jump/identify/capabilities
affects: [phase-01-cli, phase-01-tabbar]
tech-stack:
  added: []
  patterns:
    - typed request-response transport for notification flows
key-files:
  created: []
  modified:
    - crates/codec/src/lib.rs
    - crates/wezterm-client/src/client.rs
    - crates/wezterm-mux-server-impl/src/sessionhandler.rs
    - crates/wezterm-mux-server-impl/src/dispatch.rs
key-decisions:
  - "Keep notification transport payloads machine-readable with explicit structs rather than overloading existing commands."
  - "Reuse the existing focused-pane path for unread jumps instead of inventing a transport-only navigation flow."
patterns-established:
  - "Notification transport mirrors mux record fields with typed request/response PDUs."
  - "Server dispatch resolves identify and unread routing through existing mux identities."
requirements-completed: [NOTF-01, NOTF-02, NOTF-03, NOTF-04, NOTF-05]
duration: 58min
completed: 2026-03-26
---

# Phase 1: Notification Routing Summary

**Typed notification PDUs, client RPC wrappers, and mux-server dispatch now cover create/list/mutate/jump/identify/capabilities for Phase 1 notification flows**

## Performance

- **Duration:** 58 min
- **Started:** 2026-03-26T14:52:00Z
- **Completed:** 2026-03-26T15:49:49Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added stable typed notification payloads and PDU registrations in `codec`.
- Added client-side RPC wrappers for all Phase 1 notification transport calls.
- Added mux-server dispatch for notification CRUD, unread jump routing, identify, and capabilities responses.

## Task Commits

Plan completed in the current recovery checkpoint; commit not created yet.

## Files Created/Modified
- `crates/codec/src/lib.rs` - notification transport structs, PDU registrations, and codec tests
- `crates/wezterm-client/src/client.rs` - RPC wrappers for Phase 1 notification actions
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs` - server dispatch into mux notification APIs
- `crates/wezterm-mux-server-impl/src/dispatch.rs` - handled `NotificationsChanged` without breaking mux-server dispatch exhaustiveness

## Decisions Made
- Returned capabilities from a dedicated typed response so later CLI discovery can stay machine-readable.
- Kept list and clear filtering server-side by matching against mux-owned notification records rather than inventing a second identity table.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Recovered stalled transport execution inline**
- **Found during:** Plan 02 execution
- **Issue:** Delegated execution stalled after partial codec changes and never produced a complete plan result.
- **Fix:** Finished the transport implementation locally, repaired compile fallout in mux-server dispatch, and completed the codec verification target.
- **Files modified:** `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, `crates/wezterm-mux-server-impl/src/sessionhandler.rs`, `crates/wezterm-mux-server-impl/src/dispatch.rs`
- **Verification:** `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl`; `cargo test --locked -p codec -- --nocapture`
- **Committed in:** not committed yet

---

**Total deviations:** 1 auto-fixed (Rule 3 - Blocking)
**Impact on plan:** No scope increase beyond the transport/server path needed to make Wave 2 compile and verify.

## Issues Encountered
`MuxNotification::NotificationsChanged` required a matching server dispatch arm, and the timestamp string conversion path in mux-server dispatch needed a simpler representation to compile with the current chrono surface.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
`01-03` can now build the CLI commands against a verified typed transport, and `01-04` can consume mux unread state independently on the GUI side.

---
*Phase: 01-notification-routing*
*Completed: 2026-03-26*
