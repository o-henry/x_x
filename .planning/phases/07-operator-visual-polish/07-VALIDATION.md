---
phase: 07
slug: operator-visual-polish
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-28
---

# Phase 07 — Validation Strategy

> Validation contract for the reference-driven native shell restructuring work.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` / `cargo check` plus screenshot-based native GUI UAT |
| **Config file** | none — existing workspace test infrastructure |
| **Quick run command** | `cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture` |
| **Structural run command** | `cargo test --locked -p kaku-gui operator_nav -- --nocapture && cargo check --locked -p kaku-gui` |
| **Full suite command** | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo test --locked -p kaku-gui operator_nav -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` |
| **Estimated runtime** | ~240 seconds plus screenshot capture time |

---

## Sampling Rate

- **Before trusting Wave 2:** keep `tabbar`, `task_center`, and `operator_nav` slices green together
- **After every task commit:** run the relevant slice for the touched surface
- **After Wave 1:** capture a screenshot for rail + chrome comparison before touching hierarchy
- **After Wave 2:** perform the blocking structural screenshot gate before any overlay-only polish
- **Before final closeout:** the full suite above must be green and screenshots must be compared against `/tmp/kaku_ref_003.png`, `/tmp/kaku_ref_015.png`, and `/tmp/kaku_ref_030.png`
- **Max feedback latency:** 240 seconds for automated checks; screenshot review immediately after structural passes

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | POLISH-03, POLISH-04 | unit/compile | `cargo test --locked -p kaku-gui operator_nav -- --nocapture && cargo check --locked -p kaku-gui` | MISSING — Wave 1 creates/extends operator-nav coverage in `paint.rs` and `mouseevent.rs` | ⬜ pending |
| 07-01-02 | 01 | 1 | POLISH-02, POLISH-03, POLISH-04 | unit/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui operator_nav -- --nocapture && cargo check --locked -p kaku-gui` | ✅ existing files | ⬜ pending |
| 07-02-01 | 02 | 1 | POLISH-01, POLISH-02, POLISH-03, POLISH-04 | unit | `cargo fmt --all --check && cargo test --locked -p kaku-gui tabbar -- --nocapture` | ✅ | ⬜ pending |
| 07-02-02 | 02 | 1 | POLISH-01, POLISH-02, POLISH-03, POLISH-04 | unit/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 07-03-01 | 03 | 2 | POLISH-03, POLISH-04 | regression/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui operator_nav -- --nocapture && cargo check --locked -p kaku-gui` | ✅ existing files | ⬜ pending |
| 07-03-02 | 03 | 2 | POLISH-03, POLISH-04 | manual structural gate | `cargo check --locked -p kaku-gui` | ✅ screenshots required from live runtime | ⬜ pending |
| 07-04-01 | 04 | 3 | POLISH-02, POLISH-03, POLISH-04 | unit | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture` | ✅ | ⬜ pending |
| 07-04-02 | 04 | 3 | POLISH-02, POLISH-03, POLISH-04 | unit/compile | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |
| 07-05-01 | 05 | 4 | POLISH-01, POLISH-02, POLISH-03, POLISH-04 | regression/manual-docs | `cargo fmt --all --check && cargo test --locked -p kaku-gui task_center -- --nocapture && cargo test --locked -p kaku-gui tabbar -- --nocapture && cargo test --locked -p kaku-gui operator_nav -- --nocapture && cargo check --locked -p wezterm-client -p wezterm-mux-server-impl -p kaku-gui` | MISSING — Wave 4 creates `07-UAT.md`, `07-LIMITATIONS.md`, `07-CHANGED-FILES.md` | ⬜ pending |
| 07-05-02 | 05 | 4 | POLISH-01, POLISH-02, POLISH-03, POLISH-04 | docs/compile | `rg -n "Phase 7|operator visual polish|left rail|Task Center" docs/KAKU_CONTROL_PLANE_SPEC.md PLANS.md && cargo check --locked -p kaku-gui` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Phase 07 starts with three explicit validation obligations:

- Add or extend an `operator_nav`-scoped regression slice in `kaku-gui` so the persistent rail rewrite is not verified only by screenshots.
- Treat screenshot capture as a first-class checkpoint after the rail/chrome/hierarchy pass instead of waiting for final polish.
- Carry Phase 06 behavior-preservation checks forward so focus, clear unread, rerun, remain-on-exit, and metadata edits are re-proven after the structural rewrite.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Persistent rail matches the reference structure | POLISH-03 | Requires real runtime screenshots and qualitative comparison | Launch a fresh runtime, capture full-window screenshots, and verify the rail is slim, edge-attached, full-height, and no longer card-like compared with `/tmp/kaku_ref_003.png` and `/tmp/kaku_ref_015.png`. |
| Top chrome stops reading like stock Kaku tabs | POLISH-01, POLISH-02, POLISH-03 | Requires visual comparison of density, height, and marker language | Compare the captured top strip to `/tmp/kaku_ref_003.png` and `/tmp/kaku_ref_030.png`; confirm tab capsules/padding/helper text are materially reduced and the chrome keeps a DM Mono-first dense cadence. |
| Main work-surface hierarchy is honest | POLISH-03 | Automated tests cannot judge whether the viewport still looks boxed or overlay-driven | Confirm the primary work area begins immediately to the right of the rail and dominates the window; if any secondary context treatment exists, it must read as persistent structure instead of a popup. |
| Phase 06 workflow still works after the polish pass | POLISH-04 | Requires live interaction with focus, rerun, and metadata edit flows | Open Task Center, focus an item, clear unread, rerun or toggle remain-on-exit on a task-pane row when available, and edit workspace status/progress to confirm the controller seam still works after the structural rewrite. |

---

## Fast Manual Command Smoke Anchors

Use these during Wave 2 and Wave 4 UAT so the screenshots and action checks have reproducible state:

```bash
./target/debug/kaku-gui start --always-new-process
./target/debug/kaku cli notify --title "phase7-unread" --body "visual-polish" --kind task
./target/debug/kaku cli set-status "phase7 blocked"
./target/debug/kaku cli set-progress 42
./target/debug/kaku cli list-task-panes --format json
```

---

## Stop Condition

Stop Phase 07 after the dedicated structural pass if either of the following remains true in the screenshots:

- the rail still reads as a card, branded side panel, or boxed nav block
- the top chrome still reads like stock Kaku tab UI with padded capsule treatment

If either stop condition is hit, do **not** continue with overlay-only polish as if the phase were on track. Record the mismatch in `07-LIMITATIONS.md` and elevate the next step to a dedicated Rust-native container/widget architecture phase.

---

## Validation Sign-Off

- [x] Every planned auto task has an automated verification command
- [x] Structural screenshot review is a blocking gate before overlay polish
- [x] Phase 06 behavior-preservation checks are carried forward explicitly
- [x] The stop-condition is documented so the renderer limit is surfaced honestly
- [x] `nyquist_compliant: true` is set in frontmatter

**Approval:** pending
