# Deferred Items

## 2026-03-27

- `kaku-gui/src/overlay/task_center.rs`: the `cargo test --locked -p kaku-gui tabbar -- --nocapture` verification slice is currently blocked by pre-existing test references to `TaskCenterOverlay::row_text_for_test` and `TaskCenterOverlay::header_lines_for_test`. This is unrelated to `06-03`'s tabbar and mouse-routing changes, so it was left untouched per execution scope rules.
