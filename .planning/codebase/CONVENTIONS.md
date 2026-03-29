# Coding Conventions

**Analysis Date:** 2026-03-26

## Naming Patterns

**Files:**
- Use snake_case Rust module names such as `kaku/src/cli/send_text.rs`, `kaku-gui/src/overlay/quickselect.rs`, and `mux/src/pane_encoding.rs`.

**Functions:**
- Prefer snake_case free functions and methods such as `run_cli_async` in `kaku/src/cli/mod.rs`, `compute_tab_plain_title` in `kaku-gui/src/tabbar.rs`, and `set_active_workspace` in `mux/src/lib.rs`.

**Variables:**
- Use descriptive snake_case locals such as `workspace_for_new_window` in `mux/src/lib.rs` and `domain_id_of_current_tab` in `kaku-gui/src/overlay/launcher.rs`.

**Types:**
- Use PascalCase for structs/enums/traits such as `CliCommand`, `MuxNotification`, `LauncherArgs`, and `TermWindow`.

## Code Style

**Formatting:**
- Tool used: `rustfmt` with nightly invocation from `Makefile` and `.github/workflows/ci.yml`.
- Key settings: `.rustfmt.toml` sets `edition = "2018"`, `imports_granularity = "Module"`, and `tab_spaces = 4`.

**Linting:**
- Tool used: compiler warnings plus explicit clippy allowlists at file tops in `kaku/src/main.rs`, `kaku-gui/src/main.rs`, and `mux/src/lib.rs`.
- Key rules: large legacy modules opt into targeted `#![allow(clippy::...)]` blocks instead of attempting full-clippy cleanliness.

## Import Organization

**Order:**
1. Local crate/module imports such as `use crate::...` or `use super::...`
2. Workspace/external crate imports such as `use config::...`, `use mux::...`, `use anyhow::...`
3. `std::...` imports when helpful for readability in the given file

**Path Aliases:**
- No broad path alias system detected. Modules are imported through normal crate paths such as `config::`, `mux::`, `wezterm_*`, and `::window`.

## Error Handling

**Patterns:**
- Return `anyhow::Result<T>` at command/runtime boundaries as in `kaku/src/cli/list.rs`, `kaku-gui/src/main.rs`, and `mux/src/domain.rs`.
- Add context with `.context(...)` and construct ad-hoc errors with `anyhow!` at boundary points.
- Long-lived runtime paths prefer logging and graceful degradation over panics.

## Logging

**Framework:** `log`

**Patterns:**
- Use `log::warn!`, `log::error!`, `log::debug!`, and `log::trace!` in runtime-heavy code such as `mux/src/lib.rs`, `kaku-gui/src/main.rs`, and `kaku-gui/src/tabbar.rs`.
- Notifications and logs are often paired so background runtime failures do not silently disappear.

## Comments

**When to Comment:**
- Comments explain terminal, platform, or window-manager edge cases, as seen in `kaku/src/main.rs`, `kaku-gui/src/main.rs`, and `mux/src/lib.rs`.
- Comments also mark deliberate compromises or future work with `TODO` / `FIXME` where behavior is known to be incomplete.

**JSDoc/TSDoc:**
- Not applicable.
- Rust doc comments are used selectively for traits, enums, and public behaviors, for example in `mux/src/domain.rs`, `mux/src/pane.rs`, and `kaku-gui/src/commands.rs`.

## Function Design

**Size:** 
- Small focused helpers exist, but very large orchestrator files are accepted in key surfaces such as `kaku-gui/src/termwindow/mod.rs`, `kaku/src/ai_config/tui.rs`, and `mux/src/tab.rs`.

**Parameters:**
- Typed argument structs/enums are preferred at command and UI boundaries, for example `CliCommand` in `kaku/src/cli/mod.rs`, `LauncherArgs` in `kaku-gui/src/overlay/launcher.rs`, and `RenderScreenLineParams` in `kaku-gui/src/termwindow/render/mod.rs`.

**Return Values:**
- Use `Result` for fallible operations, typed structs for structured state, and enums for mode/notification branching.

## Module Design

**Exports:**
- Module trees are composed with `mod` declarations and selective `pub use`, for example in `kaku-gui/src/overlay/mod.rs` and `kaku/src/cli/mod.rs`.

**Barrel Files:**
- `mod.rs` files act as local barrel modules in paths such as `kaku/src/cli/mod.rs`, `kaku-gui/src/overlay/mod.rs`, and `kaku-gui/src/termwindow/mod.rs`.

## Practical Guidance For New Work

**Use existing typed enums/structs first:**
- New command/notification behavior should fit into existing enums such as `MuxNotification` in `mux/src/lib.rs` and command registries in `kaku/src/cli/mod.rs` / `kaku-gui/src/commands.rs`.

**Prefer additive modules over widening orchestrators:**
- Add a focused file under `kaku/src/cli/`, `kaku-gui/src/overlay/`, or `kaku-gui/src/termwindow/` before expanding already-large files such as `kaku-gui/src/termwindow/mod.rs`.

**Keep machine-readable outputs stable:**
- Follow the `CliListResultItem` pattern in `kaku/src/cli/list.rs` and `kaku/src/cli/list_clients.rs` when adding JSON-facing command contracts.

---

*Convention analysis: 2026-03-26*
