# Kaku Agent Control Plane

## What This Is

This is a local-only fork of Kaku for solo Unity indie development. It preserves vanilla Kaku's feel while extending the existing Rust CLI, mux, tabbar, overlay, and termwindow surfaces into a Rust-only agent control plane with notification routing, workspace metadata, and task-pane lifecycle controls.

The product is intentionally additive: it should still feel like Kaku, not a different terminal shell or app shell. GSD is used here only for planning and state tracking; the canonical product definition lives in `docs/KAKU_CONTROL_PLANE_SPEC.md`.

## Core Value

A solo Unity developer can see which pane needs attention, keep failed work visible, and recover or navigate quickly without leaving Kaku's existing UX model.

## Requirements

### Validated

- ✓ Pane and tab creation, splitting, and focus/navigation already work and must remain intact — existing
- ✓ Workspace switching already works and must remain intact — existing
- ✓ Existing `kaku cli` pane-management commands (`list`, `split-pane`, `spawn`, `send-text`, `get-text`, `set-tab-title`) already work and must remain intact — existing
- ✓ Existing tabbar rendering and current Kaku look/feel already work and must remain intact — existing
- ✓ Existing mux behavior that current Kaku depends on already works and must remain intact — existing

### Active

- [ ] Add a mux-owned notification store with unread routing, machine-readable CLI commands, and visual unread markers.
- [ ] Add workspace-scoped status, progress, and log metadata without introducing a new app shell.
- [ ] Add a searchable Task Center overlay for active, unread, failed, and running work.
- [ ] Add task-pane lifecycle controls for remain-on-exit, rerun/respawn, silence watchdog, and pipe-pane output teeing.
- [ ] Harden the control plane with targeted tests, docs, stable command contracts, and regression checks.

### Out of Scope

- Embedded browser — explicitly excluded from v1 and not needed for the core control-plane value.
- PR or GitHub UI — outside the local terminal control-plane goal.
- Port scanner — not part of the developer attention-routing workflow.
- Full detached tmux server semantics — too broad for v1 and would pull the fork away from Kaku's model.
- Large visual redesign — the result must still feel like Kaku.
- cmux visual clone — behavior inspiration is allowed, but the implementation and presentation must remain Kaku-native.
- New socket daemon unless clearly required later — avoid widening architecture before the current extension points are exhausted.
- Any Swift, Xcode, React, Electron, Tauri, or WebView-based addition — hard repo constraint.

## Context

- The repository is a brownfield Rust workspace with a shared `mux` core, typed CLI surface in `kaku/src/cli`, and native GUI shell in `kaku-gui/`.
- The preferred extension points for this work are `kaku/src/cli`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay`, `kaku-gui/src/termwindow`, and `mux/src`.
- The control plane is meant for a solo Unity workflow with planner, worker, reviewer, build, test, log, and git panes that need fast attention routing.
- Existing codebase mapping in `.planning/codebase/` shows that state updates are already largely event-driven through `MuxNotification`, overlays, and render invalidation paths.
- The main architectural risk is coupling too much new logic into already-large orchestrator surfaces such as `mux/src/lib.rs`, `mux/src/tab.rs`, `kaku-gui/src/commands.rs`, and `kaku-gui/src/termwindow/mod.rs`.

## Constraints

- **Tech stack**: Rust-only implementation — preserve the repo's Rust-first architecture and avoid sidecar stacks.
- **Architecture**: Extend existing Kaku surfaces first — reuse `mux`, CLI, tabbar, overlay, and termwindow paths before adding new modules.
- **Product scope**: Local-only fork — this is not an upstream PR workflow or multi-user product.
- **UX**: Preserve current Kaku look and feel — the result must feel additive rather than like a new shell.
- **Behavior**: Reimplement inspired semantics in a Kaku-native way — no copied, ported, or translated code from cmux or tmux.
- **Execution style**: Phase-by-phase additive delivery — no giant refactors, no broad speculative rewrites, and each phase must compile independently.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use `docs/KAKU_CONTROL_PLANE_SPEC.md` as the canonical product definition | The repo already defines GSD as workflow/state management only, not source-of-truth scope | — Pending |
| Treat existing Kaku pane, tab, workspace, CLI, tabbar, and mux behavior as validated baseline behavior | The fork is additive and must not break the workflows already relied on | — Pending |
| Start with notification routing and workspace metadata before lifecycle hardening | This matches the repo plan and the highest-priority solo Unity attention-routing needs | — Pending |
| Keep planning docs local-only | The current worktree already ignores `.planning/`, and this fork uses GSD for local workflow support | — Pending |
| Skip project-level ecosystem research during initialization | The canonical spec, brownfield codebase map, and phase plan are already specific enough to avoid generic domain research noise | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-26 after initialization*
