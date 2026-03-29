# Technology Stack

**Analysis Date:** 2026-03-26

## Languages

**Primary:**
- Rust 2018 edition - workspace-wide implementation language across `kaku/`, `kaku-gui/`, `mux/`, `config/`, `term/`, `termwiz/`, `window/`, and supporting crates declared in `Cargo.toml`.

**Secondary:**
- Lua - end-user configuration surface and embedded scripting integration via `config/src/lua.rs`, `lua-api-crates/`, and `kaku-gui/src/scripting/`.
- Shell scripts - build, release, and shell-integration automation in `scripts/` and `assets/shell-integration/tests/`.
- WGSL / GLSL - GPU shader assets in `kaku-gui/src/shader.wgsl`, `kaku-gui/src/glyph-frag.glsl`, and `kaku-gui/src/glyph-vertex.glsl`.

## Runtime

**Environment:**
- Rust toolchain `1.93.0` pinned in `rust-toolchain.toml`.
- Native desktop runtime with platform-specific branches in `kaku-gui/src/main.rs`, `window/src/`, and target-specific dependencies in `kaku-gui/Cargo.toml` and `Cargo.toml`.

**Package Manager:**
- Cargo workspace - root manifest in `Cargo.toml`.
- Lockfile: present in `Cargo.lock`.

## Frameworks

**Core:**
- Clap 4 - CLI parsing in `kaku/src/main.rs` and `kaku/src/cli/mod.rs`.
- Custom mux/domain model - pane, tab, workspace, and notification core in `mux/src/lib.rs`, `mux/src/domain.rs`, `mux/src/pane.rs`, `mux/src/tab.rs`, and `mux/src/window.rs`.
- WezTerm-derived terminal/rendering crates - `term/`, `termwiz/`, `window/`, `crates/wezterm-*`, and `deps/*` provide terminal emulation, rendering, fonts, client/server transport, and platform glue.

**Testing:**
- Built-in Rust test harness - inline `#[cfg(test)]` modules across core crates such as `kaku/src/utils.rs`, `kaku/src/config_tui/mod.rs`, `mux/src/pane.rs`, and `kaku-gui/src/termwindow/resize.rs`.
- `cargo-nextest` - primary test runner in `Makefile` and `.github/workflows/ci.yml`.
- `k9` snapshot assertions - snapshot-heavy tests in files such as `mux/src/pane.rs`, `term/src/test/mod.rs`, and `kaku-gui/src/shapecache.rs`.
- `rstest` - parameterized tests in `crates/wezterm-ssh/tests/e2e/*.rs`.

**Build/Dev:**
- `cargo` - primary build/check/test tool in `Makefile` and `.github/workflows/ci.yml`.
- `cargo-watch` - local dev loop in `Makefile`.
- `rustfmt` nightly - formatting in `.rustfmt.toml`, `Makefile`, and `.github/workflows/ci.yml`.
- `wgpu` - GUI rendering backend dependency in `kaku-gui/Cargo.toml`.

## Key Dependencies

**Critical:**
- `mux` - shared pane/tab/window/workspace state and notifications, used by both `kaku` and `kaku-gui`.
- `config` - centralized configuration model and Lua config bridge in `config/src/lib.rs`, `config/src/config.rs`, and `config/src/lua.rs`.
- `wezterm-client` and `wezterm-mux-server-impl` - client/server transport used by CLI and GUI attach flows in `kaku/src/cli/mod.rs` and `kaku-gui/src/main.rs`.
- `portable-pty` - PTY spawning and pane process control in `mux/src/domain.rs`, `mux/src/localpane.rs`, and CLI spawn/split paths.
- `termwiz` and `wezterm-term` - terminal surface, escape parsing, progress, and rendering primitives used heavily in `mux/` and `kaku-gui/`.
- `mlua` and `mux-lua` - Lua configuration and scripting integration in `config/src/lua.rs`, `kaku-gui/src/scripting/`, and `lua-api-crates/`.

**Infrastructure:**
- `rusqlite` - local persistence dependency included by `kaku/Cargo.toml`.
- `serde` and `serde_json` - stable machine-readable CLI output and config serialization in `kaku/src/cli/list.rs`, `kaku/src/cli/list_clients.rs`, and config crates.
- `ratatui` and `crossterm` - text UI stack used by `kaku/src/config_tui/`, `kaku/src/ai_config/tui.rs`, and related command UIs.
- `rayon` - parallel scoring/filtering work in `kaku-gui/src/overlay/launcher.rs`.
- `reqwest`, `http_req`, `git2`, and `openssl` - update/download, network, and VCS integration dependencies declared in `Cargo.toml`.

## Configuration

**Environment:**
- User-facing runtime configuration is loaded from Lua config via `config/src/lua.rs` and referenced in `README.md` as `~/.config/kaku/kaku.lua`.
- Assistant-specific settings are described in `README.md` as `~/.config/kaku/assistant.toml`.
- Shell integration assets live under `assets/shell-integration/` and are provisioned by `kaku/src/init.rs` and validated by `kaku/src/doctor.rs`.

**Build:**
- Root workspace configuration in `Cargo.toml`.
- Toolchain pin in `rust-toolchain.toml`.
- Formatter settings in `.rustfmt.toml`.
- Automation in `Makefile`.
- CI in `.github/workflows/ci.yml`.

## Platform Requirements

**Development:**
- Rust `1.93.0` plus nightly `rustfmt` per `rust-toolchain.toml`, `Makefile`, and `.github/workflows/ci.yml`.
- macOS-focused development path is the best-supported path based on `README.md` and CI runners in `.github/workflows/ci.yml`.

**Production:**
- Local desktop terminal app plus CLI binaries built from `kaku/` and `kaku-gui/`.
- Current published product target is macOS according to `README.md`, even though the workspace contains Windows and Wayland branches.

---

*Stack analysis: 2026-03-26*
