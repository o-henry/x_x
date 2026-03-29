# Phase 1: Notification Routing - Discussion Log (Assumptions Mode)

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md — this log preserves the analysis.

**Date:** 2026-03-26
**Phase:** 01-Notification Routing
**Mode:** assumptions
**Areas analyzed:** State ownership, CLI shape, Unread semantics, UI surfacing, Automation helpers

## Assumptions Presented

### State ownership
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Notifications and unread indexes should become mux-owned state keyed by existing workspace/tab/pane identities, not a new GUI-owned cache. | Likely | `docs/KAKU_CONTROL_PLANE_SPEC.md`, `.planning/ROADMAP.md`, `mux/src/lib.rs`, `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/frontend.rs` |

### CLI shape
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Phase 1 should add dedicated `kaku cli` subcommands for notification actions, backed by typed codec/client/server requests, with stable machine-readable output for list/query paths. | Likely | `docs/KAKU_CONTROL_PLANE_SPEC.md`, `kaku/src/cli/mod.rs`, `kaku/src/cli/list.rs`, `crates/codec/src/lib.rs`, `crates/wezterm-mux-server-impl/src/sessionhandler.rs` |

### Unread semantics
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Unread should be pane-targeted attention that persists until explicitly cleared or marked read, while navigation uses existing pane/tab/window order for next/previous unread. | Unclear | `docs/KAKU_CONTROL_PLANE_SPEC.md`, `mux/src/tab.rs`, `.planning/ROADMAP.md` |

### UI surfacing
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Visual surfacing should stay minimal and Kaku-native: tabbar markers first, then only small pane-adjacent/state hints if needed, with no sidebar or redesign. | Confident | `docs/KAKU_CONTROL_PLANE_SPEC.md`, `.planning/ROADMAP.md`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/termwindow/mod.rs` |

### Automation helpers
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| `identify` and `capabilities` belong in phase 1 as part of the machine-oriented contract, so external tooling can resolve targets and detect supported notification features safely. | Likely | `docs/KAKU_CONTROL_PLANE_SPEC.md`, `kaku/src/cli/mod.rs` |

## Corrections Made

### Unread semantics
- **Original assumption:** Unread should persist until explicitly cleared or marked read.
- **User correction:** Support two notification modes: `clear-on-focus` auto-reads when the target pane is focused, and `sticky` remains unread until explicitly cleared or marked read.
- **Reason:** Unread state should stay pane-targeted with tab/workspace aggregation, and next/previous unread navigation must follow existing pane/tab/window ordering without changing current Kaku navigation behavior.

## External Research

No external research was required. Existing repo docs, codebase maps, and source seams were sufficient to capture phase-1 decisions.
