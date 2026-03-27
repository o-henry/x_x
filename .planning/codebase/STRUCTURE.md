# Codebase Structure

**Analysis Date:** 2026-03-26

## Directory Layout

```text
x_x/
├── kaku/               # CLI binary, setup flows, config/doctor/update TUIs, mux client commands
├── kaku-gui/           # Native GUI binary, tabbar, overlays, command palette, term window logic
├── mux/                # Shared pane/tab/window/domain/workspace state and notifications
├── config/             # Runtime config types, key assignments, Lua config integration
├── term/               # Terminal emulation core
├── termwiz/            # Terminal UI, escape parsing, widgets, surfaces
├── window/             # Native windowing and platform backends
├── crates/             # Supporting Rust crates reused across the workspace
├── lua-api-crates/     # Lua-facing API modules exposed to config/scripting
├── assets/             # Fonts, shell integration assets, app resources
├── scripts/            # Build, release, and validation scripts
├── docs/               # Product and architecture documentation
├── .planning/          # GSD planning state and generated codebase map
├── Cargo.toml          # Workspace manifest
├── rust-toolchain.toml # Rust toolchain pin
└── Makefile            # Common dev/build/test commands
```

## Directory Purposes

**`kaku/`:**
- Purpose: ship the CLI entry point and text-first workflows.
- Contains: `main.rs`, `cli/` command modules, config/AI TUIs, init/reset/doctor/update commands.
- Key files: `kaku/src/main.rs`, `kaku/src/cli/mod.rs`, `kaku/src/config_tui/mod.rs`, `kaku/src/doctor.rs`

**`kaku-gui/`:**
- Purpose: ship the native GUI frontend and window lifecycle.
- Contains: startup flow, command registry, tabbar, overlays, render code, term window submodules.
- Key files: `kaku-gui/src/main.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/overlay/mod.rs`, `kaku-gui/src/termwindow/mod.rs`

**`mux/`:**
- Purpose: own panes, tabs, windows, workspaces, domains, and notifications.
- Contains: domain abstractions, pane/tab/window models, workspace routing, pane encoding, SSH/tmux integration helpers.
- Key files: `mux/src/lib.rs`, `mux/src/domain.rs`, `mux/src/pane.rs`, `mux/src/tab.rs`, `mux/src/window.rs`

**`config/`:**
- Purpose: central config schema and Lua integration.
- Contains: terminal/window/font/domain/keyassignment settings, serialization, config metadata, Lua bridge.
- Key files: `config/src/lib.rs`, `config/src/config.rs`, `config/src/keyassignment.rs`, `config/src/lua.rs`

**`term/`, `termwiz/`, `window/`:**
- Purpose: terminal emulation, rendering, and platform integration.
- Contains: terminal state machines, widget/layout code, input parsing, OS-specific window code.
- Key files: `term/src/lib.rs`, `termwiz/src/lib.rs`, `window/src/lib.rs`

**`crates/`:**
- Purpose: split reusable building blocks into focused crates.
- Contains: client/server transport, fonts, escape parsing, surfaces, PTY helpers, caching, Unicode helpers, etc.
- Key files: `crates/wezterm-client/src/client.rs`, `crates/wezterm-mux-server-impl/src/lib.rs`, `crates/wezterm-font/src/lib.rs`

**`lua-api-crates/`:**
- Purpose: modular Lua API adapters shared by config and scripting.
- Contains: domain-specific Lua helpers for mux, filesystem, logging, spawning, URLs, windows, and more.
- Key files: `lua-api-crates/mux/`, `lua-api-crates/plugin/`, `lua-api-crates/spawn-funcs/`

## Key File Locations

**Entry Points:**
- `kaku/src/main.rs`: CLI binary entry point and top-level subcommand routing.
- `kaku-gui/src/main.rs`: GUI binary entry point and frontend bootstrap.

**Configuration:**
- `Cargo.toml`: workspace membership, profiles, shared dependency versions.
- `rust-toolchain.toml`: toolchain pin.
- `.rustfmt.toml`: formatter settings.
- `Makefile`: build/test/format/dev tasks.
- `config/src/config.rs`: runtime configuration model.
- `config/src/keyassignment.rs`: action/key binding definitions consumed by GUI and mux.

**Core Logic:**
- `mux/src/lib.rs`: global mux state and notification hub.
- `mux/src/domain.rs`: domain trait and spawn/split abstractions.
- `mux/src/pane.rs`: pane trait and pane-level helpers.
- `mux/src/tab.rs`: pane tree layout and tab behavior.
- `mux/src/window.rs`: workspace-scoped windows and active-tab handling.
- `kaku-gui/src/termwindow/mod.rs`: high-level GUI window orchestration.
- `kaku-gui/src/tabbar.rs`: tab title and status rendering.
- `kaku/src/cli/`: machine-oriented pane/tab/workspace command implementations.

**Testing:**
- Inline unit tests inside source files such as `kaku/src/utils.rs`, `mux/src/pane.rs`, and `kaku-gui/src/tabbar.rs`.
- Dedicated test trees in `term/src/test/`, `crates/wezterm-dynamic/tests/`, `crates/wezterm-ssh/tests/`, and `assets/shell-integration/tests/`.

## Naming Conventions

**Files:**
- Rust module files are snake_case, for example `kaku/src/cli/send_text.rs` and `kaku-gui/src/overlay/quickselect.rs`.
- Entry-point and registry modules commonly use `main.rs`, `mod.rs`, or domain nouns such as `commands.rs` and `tabbar.rs`.

**Directories:**
- Crate roots use concise product or subsystem names such as `kaku/`, `kaku-gui/`, `mux/`, `term/`, and `window/`.
- Feature-oriented subdirectories group related code, for example `kaku/src/cli/`, `kaku-gui/src/overlay/`, and `kaku-gui/src/termwindow/render/`.

## Where to Add New Code

**New control-plane CLI feature:**
- Primary code: `kaku/src/cli/` plus subcommand registration in `kaku/src/cli/mod.rs`
- Tests: inline tests in the new module or nearby command modules; add broader integration checks only if needed

**New mux-owned runtime metadata or notifications:**
- Implementation: `mux/src/lib.rs` for shared store/notifications
- Related types: `mux/src/pane.rs`, `mux/src/tab.rs`, `mux/src/window.rs`, or `mux/src/domain.rs` depending on scope
- Tests: colocated `#[cfg(test)]` modules in the touched mux files

**New GUI affordance using existing surfaces:**
- Tab/status visuals: `kaku-gui/src/tabbar.rs`
- Command palette integration: `kaku-gui/src/commands.rs`
- Overlay workflow: new or existing files under `kaku-gui/src/overlay/`
- Window behavior / focus / pane lifecycle UI: `kaku-gui/src/termwindow/`

**Utilities:**
- Shared helpers: prefer the smallest existing crate or module that already owns that concern instead of adding a new top-level crate.

## Special Directories

**`.planning/`:**
- Purpose: GSD planning state and generated brownfield map documents
- Generated: Yes
- Committed: project-dependent; currently present in the working tree

**`assets/shell-integration/tests/`:**
- Purpose: shell integration smoke tests run from CI
- Generated: No
- Committed: Yes

**`deps/`:**
- Purpose: vendored/native dependency wrappers such as Cairo, fontconfig, freetype, and harfbuzz
- Generated: No
- Committed: Yes

## Structure Guidance For This Fork

**Control-plane work placement:**
- Notification and unread state should start in `mux/src/` and surface upward into `kaku/src/cli/` and `kaku-gui/src/tabbar.rs`.
- Workspace status/progress/log state should stay mux-owned and render through `kaku-gui/src/tabbar.rs`, `kaku-gui/src/overlay/`, and `kaku-gui/src/termwindow/`.
- Task Center should prefer `kaku-gui/src/overlay/` plus existing command/launcher hooks in `kaku-gui/src/commands.rs`.
- Task-pane lifecycle should extend `mux/src/` first, then add UI hooks in `kaku-gui/src/termwindow/`.

**Files to avoid as first-choice extension points:**
- `kaku-gui/src/main.rs` and `kaku/src/main.rs` are entry-point-heavy routing files; only touch them when registration is required.
- `kaku-gui/src/termwindow/mod.rs` is central and very large, so prefer its submodules when a more local extension point exists.

---

*Structure analysis: 2026-03-26*
