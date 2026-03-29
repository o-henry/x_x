---
phase: 02-workspace-metadata-plane
plan: 02
subsystem: transport
tags: [codec, wezterm-client, mux-server, workspace-metadata, rust]
requires:
  - phase: 02-01
    provides: mux-owned workspace metadata APIs and WorkspaceMetadataChanged event
provides:
  - typed workspace metadata PDUs for status, progress, and logs
  - client RPC wrappers for all Phase 2 metadata commands
  - mux-server dispatch into workspace metadata APIs with stable timestamp strings
affects: [phase-02-cli, phase-02-closeout]
tech-stack:
  added: []
  patterns:
    - typed request/response contracts for workspace-scoped metadata
    - explicit server-side conversion from mux records into transport state payloads
key-files:
  created: []
  modified:
    - crates/codec/src/lib.rs
    - crates/wezterm-client/src/client.rs
    - crates/wezterm-mux-server-impl/src/sessionhandler.rs
    - crates/wezterm-mux-server-impl/src/dispatch.rs
key-decisions:
  - "Keep Phase 2 transport additive by extending the existing Pdu enum and rpc! pattern instead of introducing a generic metadata multiplexer."
  - "Serialize timestamps into RFC 3339 strings manually from UTC components because this repo's chrono configuration disables the usual formatting helpers."
patterns-established:
  - "Workspace metadata transport contracts return deterministic mutation summaries with cleared_count/workspaces and stable record field names."
  - "WorkspaceMetadataChanged must be handled exhaustively as a no-op in dispatcher loops until a streamed metadata contract exists."
requirements-completed: [META-01, META-02, META-03, META-04]
duration: 34min
completed: 2026-03-27
---

# Phase 2: Workspace Metadata Plane Summary

**Status, progress, and log metadata now travel end-to-end through typed codec, client, and mux-server contracts**

## Performance

- **Duration:** 34 min
- **Started:** 2026-03-27T13:20:00Z
- **Completed:** 2026-03-27T13:54:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added typed Phase 2 PDUs and stable payload structs for workspace status, progress, and bounded logs.
- Added `rpc!` client wrappers for all dedicated workspace metadata commands.
- Wired mux-server request handling into the new mux-owned workspace metadata APIs and kept dispatcher exhaustiveness with `WorkspaceMetadataChanged`.

## Task Commits

Combined recovery path after delegated executor stalled during RED checkpoint:

1. **Task 1 + Task 2: workspace metadata transport, client wrappers, and server dispatch** - pending commit (recovery path)

## Files Created/Modified
- `crates/codec/src/lib.rs` - Added workspace metadata state structs, request/response PDUs, stable JSON field coverage, and round-trip tests.
- `crates/wezterm-client/src/client.rs` - Added typed `rpc!` wrappers for status, progress, and log metadata commands.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs` - Added workspace metadata request handling and UTC timestamp-to-RFC3339 string conversion.
- `crates/wezterm-mux-server-impl/src/dispatch.rs` - Added exhaustive no-op handling for `WorkspaceMetadataChanged`.

## Decisions Made
- Kept `Clear*` responses deterministic with `cleared_count` plus `workspaces`, matching the stable machine-readable contract expected by later CLI work.
- Trimmed `ListWorkspaceLog.limit` from the newest end after filtering so transport stays aligned with append order semantics from the mux store.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Recovered stalled delegated execution after RED-only checkpoint**
- **Found during:** Plan 02-02 execution
- **Issue:** The delegated executor stopped after confirming the RED test failure and never advanced through implementation, verification, or summary creation.
- **Fix:** Completed the transport/server recovery inline in the orchestrator worktree, resolved the repo-specific chrono formatting constraints, and reran the focused verification commands.
- **Files modified:** `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, `crates/wezterm-mux-server-impl/src/sessionhandler.rs`, `crates/wezterm-mux-server-impl/src/dispatch.rs`
- **Verification:** `cargo check --locked -p wezterm-client -p wezterm-mux-server-impl`; `cargo test --locked -p codec -- --nocapture`
- **Committed in:** pending commit (recovery path)

---

**Total deviations:** 1 auto-fixed (Rule 3 - Blocking)
**Impact on plan:** No scope change. The recovery stayed inside the exact transport/server file set defined by the plan.

## Issues Encountered
The main transport blocker was timestamp formatting: this repo disables chrono default features, so common RFC 3339 helper methods were unavailable. Building the string from UTC date/time components resolved that without widening dependencies or changing the contract shape.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
The dedicated workspace metadata transport contract is now ready for the CLI plan to expose stable `kaku cli` commands without inventing ad hoc parsing or direct mux calls.

---
*Phase: 02-workspace-metadata-plane*
*Completed: 2026-03-27*
