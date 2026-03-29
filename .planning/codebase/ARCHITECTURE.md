# Architecture

**Analysis Date:** 2026-03-26

## Pattern Overview

**Overall:** Rust workspace with a shared multiplexer core, a typed CLI client surface, and a native GUI frontend layered over WezTerm-derived terminal/rendering crates.

**Key Characteristics:**
- Shared state is centralized in `mux/src/lib.rs` and exposed through pane/tab/window/domain abstractions.
- Both `kaku` and `kaku-gui` consume the same mux/config primitives rather than duplicating state models.
- UI behavior is additive and event-driven through `MuxNotification`, overlays, and render invalidation instead of polling loops.

## Layers

**Workspace / build layer:**
- Purpose: workspace membership, shared dependency graph, profiles, and toolchain policy.
- Location: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `Makefile`
- Contains: workspace members, profiles, dependency versions, top-level task commands
- Depends on: Cargo and rustup toolchains
- Used by: every crate in the repository

**Configuration and scripting layer:**
- Purpose: runtime configuration, key assignments, Lua interoperability, shell defaults, and metadata types.
- Location: `config/src/`, `lua-api-crates/`, `kaku/src/assistant_config.rs`
- Contains: config structs, key assignment enums, Lua bridge code, assistant/tool config, shell/domain options
- Depends on: `serde`, `mlua`, `wezterm-dynamic`, and shared workspace crates
- Used by: `kaku/`, `kaku-gui/`, `mux/`, and embedded Lua APIs

**Mux domain/state layer:**
- Purpose: own panes, tabs, windows, domains, clients, workspaces, and notifications.
- Location: `mux/src/lib.rs`, `mux/src/domain.rs`, `mux/src/pane.rs`, `mux/src/tab.rs`, `mux/src/window.rs`
- Contains: `Mux`, `MuxNotification`, `Domain`, `Pane`, `Tab`, `Window`, workspace routing, pane output dispatch
- Depends on: config types, portable PTY support, terminal primitives, and client/server transport crates
- Used by: CLI commands, GUI windows, overlays, and spawn/attach flows

**CLI command layer:**
- Purpose: expose mux functionality as typed subcommands with stable human/table and JSON output when available.
- Location: `kaku/src/main.rs`, `kaku/src/cli/`, `kaku/src/config_cmd.rs`, `kaku/src/doctor.rs`, `kaku/src/init.rs`, `kaku/src/update.rs`
- Contains: clap parsing, subcommand routing, machine-readable command modules, TUI commands, setup/reset/doctor flows
- Depends on: `mux`, `config`, `wezterm-client`, `serde_json`, `tabout`, and TUI crates
- Used by: local shell invocations and scripts integrating with Kaku

**GUI shell layer:**
- Purpose: start the native app, attach domains, own windows, coordinate overlays, and bridge mux state into rendering.
- Location: `kaku-gui/src/main.rs`, `kaku-gui/src/frontend.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/tabbar.rs`, `kaku-gui/src/overlay/`, `kaku-gui/src/termwindow/`
- Contains: startup flow, command definitions, launcher/quickselect/copy overlays, tabbar computation, pane and window interaction handling
- Depends on: `mux`, `config`, `window`, `wezterm-*` crates, and render caches
- Used by: the desktop GUI binary and any future control-plane UI affordances

**Rendering / platform layer:**
- Purpose: font shaping, line layout, GPU/window integration, terminal emulation, and OS-specific window behavior.
- Location: `term/`, `termwiz/`, `window/`, `crates/wezterm-font/`, `crates/wezterm-surface/`, `deps/`
- Contains: terminal state machines, escape parsing, surface models, fonts, GPU/window adapters, platform backends
- Depends on: system graphics/font libraries and low-level Rust crates
- Used by: `mux/` and `kaku-gui/`

## Data Flow

**CLI pane-control flow:**

1. `kaku/src/main.rs` parses `SubCommand::Cli` and forwards to `kaku/src/cli/mod.rs`.
2. `kaku/src/cli/mod.rs` opens a `wezterm_client::client::Client` using a headless `mux::connui::ConnectionUI`.
3. Individual command modules such as `kaku/src/cli/list.rs`, `kaku/src/cli/spawn_command.rs`, and `kaku/src/cli/rename_workspace.rs` invoke mux RPCs and format output.

**GUI runtime flow:**

1. `kaku-gui/src/main.rs` loads config, establishes domains, and ensures at least one tab/pane exists for the active workspace.
2. `kaku-gui/src/termwindow/mod.rs` subscribes to mux/window state and reacts to user input, alerts, and rendering invalidations.
3. `kaku-gui/src/tabbar.rs`, `kaku-gui/src/overlay/`, and `kaku-gui/src/termwindow/render/` derive visible UI from pane/tab/workspace state and terminal surfaces.

**Pane output flow:**

1. Domain/pane creation goes through `mux/src/domain.rs` and concrete pane implementations such as `mux/src/localpane.rs`.
2. `mux/src/lib.rs` reads pane output, parses terminal actions, applies them to panes, and emits `MuxNotification::PaneOutput` or alert notifications.
3. GUI renderers and subscribers redraw or surface state based on those notifications.

**State Management:**
- Shared mutable runtime state is held centrally inside `Mux` (`mux/src/lib.rs`) using `RwLock`/`Mutex` protected maps for panes, tabs, windows, domains, clients, and subscribers.
- GUI-local transient state lives inside `TermWindow` and overlay state machines in `kaku-gui/src/termwindow/mod.rs` and `kaku-gui/src/overlay/`.

## Key Abstractions

**Mux:**
- Purpose: global registry and notification hub for panes, tabs, windows, workspaces, and clients.
- Examples: `mux/src/lib.rs`
- Pattern: singleton-style shared runtime accessed with `Mux::get()` / `Mux::try_get()`

**Domain:**
- Purpose: abstract spawning, splitting, moving, and attaching panes across local, SSH, serial, or client-backed domains.
- Examples: `mux/src/domain.rs`, `mux/src/ssh.rs`, `mux/src/tmux.rs`
- Pattern: async trait with typed default behavior plus domain-specific implementations

**Pane / Tab / Window:**
- Purpose: represent renderable execution units, pane trees, and workspace-scoped windows.
- Examples: `mux/src/pane.rs`, `mux/src/tab.rs`, `mux/src/window.rs`
- Pattern: ID-addressable runtime graph with event notifications on mutation

**TermWindow:**
- Purpose: own GUI window behavior, input routing, overlay lifecycle, render state, and mux interaction.
- Examples: `kaku-gui/src/termwindow/mod.rs`
- Pattern: large façade/orchestrator over many focused submodules in `kaku-gui/src/termwindow/`

**Overlay helpers:**
- Purpose: temporary interactive UI surfaces layered on top of panes/tabs without changing the base shell architecture.
- Examples: `kaku-gui/src/overlay/mod.rs`, `kaku-gui/src/overlay/launcher.rs`, `kaku-gui/src/overlay/copy.rs`
- Pattern: create a temporary `TermWizTerminal` pane and cancel it through `TermWindow` scheduling callbacks

## Entry Points

**CLI binary:**
- Location: `kaku/src/main.rs`
- Triggers: `kaku ...` invocations from the shell
- Responsibilities: parse subcommands, initialize config, route to CLI/TUI/setup/update flows

**CLI command registry:**
- Location: `kaku/src/cli/mod.rs`
- Triggers: `kaku cli ...`
- Responsibilities: establish client transport, define subcommands, and dispatch to command modules

**GUI binary:**
- Location: `kaku-gui/src/main.rs`
- Triggers: `kaku-gui` / app bundle launch
- Responsibilities: initialize native app runtime, attach domains, spawn initial tabs, and start frontend windows

**GUI command/overlay surfaces:**
- Location: `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay/mod.rs`, `kaku-gui/src/tabbar.rs`
- Triggers: key assignments, menus, palette, launcher, and tab rendering hooks
- Responsibilities: enumerate UI actions and convert mux/runtime state into interactive surfaces

## Error Handling

**Strategy:** `anyhow::Result` is the dominant boundary type for command, setup, and runtime operations, with targeted enums where stable typed errors matter.

**Patterns:**
- Attach context at failure boundaries using `Context` and `anyhow!`, as seen in `kaku/src/main.rs`, `kaku-gui/src/main.rs`, and `mux/src/domain.rs`.
- Emit runtime errors to logs and notifications instead of panicking in long-lived UI paths, as seen around config reload and mux updates in `kaku-gui/src/main.rs`.

## Cross-Cutting Concerns

**Logging:** `log` crate usage is widespread across `kaku/`, `kaku-gui/`, and `mux/`.
**Validation:** clap argument validation in `kaku/src/main.rs` and `kaku/src/cli/mod.rs`, plus typed config parsing in `config/src/`.
**Authentication:** local mux identity in `mux/src/client.rs` and transport-level SSH/TLS support via `mux/src/ssh.rs`, `kaku/src/cli/tls_creds.rs`, and `crates/wezterm-ssh/`.

## Control-Plane-Relevant Extension Points

**Notification / unread routing candidates:**
- `mux/src/lib.rs` - add new mux-owned stores and notification variants.
- `mux/src/pane.rs` and `mux/src/tab.rs` - attach pane/tab-level attention metadata.
- `kaku/src/cli/` - add stable machine-readable commands for new control-plane state.
- `kaku-gui/src/tabbar.rs` - render unread or progress markers with existing tab title computation.

**Workspace metadata candidates:**
- `mux/src/window.rs` and `mux/src/lib.rs` - workspace-scoped data ownership and lookup.
- `kaku-gui/src/tabbar.rs` and `kaku-gui/src/termwindow/mod.rs` - status/right-surface display.
- `kaku-gui/src/overlay/launcher.rs` or new files under `kaku-gui/src/overlay/` - searchable workspace/task center UI.

---

*Architecture analysis: 2026-03-26*
