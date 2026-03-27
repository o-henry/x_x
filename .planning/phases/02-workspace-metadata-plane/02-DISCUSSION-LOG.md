# Phase 2: Workspace Metadata Plane - Discussion Log (Assumptions Mode)

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md — this log preserves the analysis.

**Date:** 2026-03-27
**Phase:** 02-workspace-metadata-plane
**Mode:** assumptions
**Areas analyzed:** State ownership, CLI contract shape, UI surfacing strategy, Progress semantics

## Assumptions Presented

### State ownership
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Workspace status, progress, and log metadata should be mux-owned workspace-scoped state instead of GUI-local or pane-local storage. | Confident | `.planning/phases/01-notification-routing/01-CONTEXT.md`, `mux/src/lib.rs`, `PLANS.md` |

### CLI contract shape
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Phase 2 should follow the Phase 1 pattern of dedicated CLI subcommands, typed codec/client/server PDUs, and stable JSON-oriented contracts with request/response shape tests. | Confident | `.planning/ROADMAP.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, `kaku/src/cli/mod.rs`, `kaku/src/cli/notify.rs`, `kaku/src/cli/list_notifications.rs`, `crates/codec/src/lib.rs`, `crates/wezterm-client/src/client.rs`, `crates/wezterm-mux-server-impl/src/sessionhandler.rs` |

### UI surfacing strategy
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Phase 2 UI surfacing should use existing tabbar and status-title surfaces before any new overlay-first work. | Likely | `docs/KAKU_CONTROL_PLANE_SPEC.md`, `kaku-gui/src/scripting/guiwin.rs`, `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/overlay/launcher.rs`, `.planning/ROADMAP.md` |

### Progress semantics
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Workspace progress should be separate from pane runtime progress, with the GUI composing them only at render time if needed. | Likely | `mux/src/localpane.rs`, `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/tabbar.rs`, `.planning/REQUIREMENTS.md`, `PLANS.md` |

## Corrections Made

No corrections — all assumptions confirmed.

## External Research

No external research required. Codebase evidence was sufficient for the Phase 2 discussion.

---

*Audit log generated: 2026-03-27*
