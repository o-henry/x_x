# Phase 4: Task-Pane Lifecycle - Discussion Log (Assumptions Mode)

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md — this log preserves the analysis.

**Date:** 2026-03-27
**Phase:** 04-task-pane-lifecycle
**Mode:** assumptions
**Areas analyzed:** lifecycle ownership, remain-on-exit semantics, rerun/respawn model, control surface, watchdog silence and tee behavior

## Assumptions Presented

### Lifecycle ownership
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Task-pane lifecycle state should be mux-owned pane-scoped state rather than GUI-local cache. | Confident | `mux/src/localpane.rs`, `mux/src/pane.rs`, `mux/src/task_center.rs`, `mux/src/lib.rs` |

### Remain-on-exit semantics
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Remain-on-exit should extend existing `ExitBehavior` / `DeadPendingClose` handling instead of creating a second pane lifecycle model. | Confident | `mux/src/localpane.rs`, `PLANS.md` |

### Rerun and respawn model
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Runtime rerun user vars remain the seed, but mux should persist enough lifecycle metadata after exit for durable rerun affordances. | Likely | `kaku-gui/src/termwindow/mod.rs`, `mux/src/task_center.rs`, `.planning/phases/03-task-center-overlay/03-LIMITATIONS.md` |

### Control surface
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Phase 4 lifecycle actions should be CLI-first, with Task Center remaining a consumer of lifecycle state and actions. | Likely | `kaku/src/cli/mod.rs`, `.planning/phases/03-task-center-overlay/03-CONTEXT.md`, `PLANS.md` |

### Watchdog silence and tee behavior
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Watchdog silence should be per-pane lifecycle state, and pipe-pane/tee should duplicate output without changing pane rendering or interaction flow. | Likely | `.planning/ROADMAP.md`, `docs/KAKU_CONTROL_PLANE_SPEC.md`, `mux/src/lib.rs` |

## Corrections Made

No corrections — all assumptions confirmed.

