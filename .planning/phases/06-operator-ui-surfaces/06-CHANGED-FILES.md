# Phase 06 Changed Files

## Summary

Phase 06 kept the operator workflow inside existing Kaku-native surfaces instead of adding a dashboard shell. The work added one `TermWindow` controller seam, upgraded the Task Center overlay into the primary operator surface, added compact tabbar discovery affordances, and closed the phase with verification and documentation artifacts.

## Code and Test Files

- `crates/codec/src/lib.rs`
  Added the typed `ListWorkspaceProgress` request/response transport used by the GUI metadata refresh path.
- `crates/wezterm-client/src/client.rs`
  Added the `list_workspace_progress` client wrapper so the GUI reads progress through the same client-domain contract style as status.
- `crates/wezterm-mux-server-impl/src/sessionhandler.rs`
  Served the additive progress-list request from mux-owned progress state.
- `kaku-gui/src/termwindow/mod.rs`
  Added scoped Task Center opening, shared workspace status/progress refresh and mutation helpers, and the remain-on-exit row-action handoff used by operator UI surfaces.
- `kaku-gui/src/overlay/task_center.rs`
  Reworked the overlay into the common-path operator surface with one-line action rows, metadata prompt/confirm modes, mouse parity, and targeted Task Center coverage.
- `kaku-gui/src/tabbar.rs`
  Added the compact ` · ops` operator marker and the actionable tabbar hit region used for discoverability.
- `kaku-gui/src/termwindow/mouseevent.rs`
  Routed operator-marker hover and click behavior into scoped Task Center opening without disturbing normal tabbar behavior.
- `kaku-gui/src/termwindow/render/fancy_tab_bar.rs`
  Kept the fancy tabbar renderer exhaustive for the new operator-marker item without widening its behavior.

## Closeout Docs

- `.planning/phases/06-operator-ui-surfaces/06-UAT.md`
  Records the automated verification slice, the live runtime reattempt, and the exact outcomes for the Phase 06 operator workflow.
- `.planning/phases/06-operator-ui-surfaces/06-LIMITATIONS.md`
  Lists only the remaining operator-UI boundaries that still matter after the targeted verification pass.
- `docs/KAKU_CONTROL_PLANE_SPEC.md`
  Updated to describe the shipped Phase 06 operator path: Task Center as the primary operator surface, prompt/confirm metadata editing, mouse+keyboard parity, and compact tabbar discovery affordances.
- `PLANS.md`
  Marks Phase 06 complete and aligns the repo plan language with the shipped operator workflow and residual limitations.

## Phase 06 Task Commits

- `68f2a3f` - test(06-01): add failing progress list transport coverage
- `68555d2` - feat(06-01): add workspace progress list transport
- `bb3f401` - feat(06-01): add termwindow workspace metadata helpers
- `1727c7d` - test(06-02): add failing operator row visibility coverage
- `73199b5` - feat(06-02): redesign task center operator rows
- `3648e63` - test(06-02): add failing task center mouse parity tests
- `bf0c23a` - feat(06-02): add task center mouse parity
- `9df1e48` - test(06-03): add tabbar operator marker coverage
- `4a27ab1` - feat(06-03): add compact tabbar operator markers
- `fc68f12` - test(06-03): add failing operator marker mouse routing coverage
- `c711c58` - feat(06-03): route tabbar operator clicks into task center
