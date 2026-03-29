# External Integrations

**Analysis Date:** 2026-03-26

## APIs & External Services

**Terminal/mux transport:**
- Internal mux RPC transport - CLI and GUI communicate through `wezterm-client` and `wezterm-mux-server-impl`.
  - SDK/Client: `wezterm-client` in `kaku/src/cli/mod.rs` and `kaku-gui/src/main.rs`
  - Auth: TLS credentials flow implemented by `kaku/src/cli/tls_creds.rs` and client/server crates

**SSH and remote domains:**
- SSH-backed domain support - remote panes/domains implemented in `mux/src/ssh.rs`, `mux/src/ssh_agent.rs`, and `crates/wezterm-ssh/`.
  - SDK/Client: `wezterm-ssh`, `ssh2`, and `libssh-rs` declared in `Cargo.toml` and `mux/Cargo.toml`
  - Auth: SSH agent / SSH config integration via local environment and SSH config files; no repo-stored env var contract detected

**Update/download:**
- Release/download fetching - update and download code paths exist in `kaku/src/update.rs` and `kaku-gui/src/download.rs`.
  - SDK/Client: `reqwest` / `http_req`
  - Auth: Not detected in repository code paths; public release/download flows appear unauthenticated

## Data Storage

**Databases:**
- Local SQLite-capable persistence - `rusqlite` is a dependency in `kaku/Cargo.toml`.
  - Connection: no environment-variable contract detected in current checked-in CLI/GUI code
  - Client: `rusqlite`

**File Storage:**
- Local filesystem only - shell assets in `assets/`, fonts/resources in `assets/` and app bundle output from `scripts/build.sh`, plus user config under `~/.config/kaku/` documented in `README.md`.

**Caching:**
- In-memory caches only in core control surfaces today, such as glyph/shape/line caches in `kaku-gui/src/glyphcache.rs`, `kaku-gui/src/shapecache.rs`, and render caches in `kaku-gui/src/termwindow/render/mod.rs`.

## Authentication & Identity

**Auth Provider:**
- Custom local identity for mux clients - current active client/workspace tracking lives in `mux/src/client.rs` and `mux/src/lib.rs`.
  - Implementation: local client IDs and mux-managed identity, with TLS/SSH support handled by the transport/domain layers rather than an external auth SaaS.

## Monitoring & Observability

**Error Tracking:**
- None detected as an external service.

**Logs:**
- In-process logging via the `log` crate across workspace crates, plus environment/bootstrap logging support in `crates/env-bootstrap/src/ringlog.rs`.

## CI/CD & Deployment

**Hosting:**
- Local desktop application distribution and DMG/app bundle build flow in `Makefile`, `scripts/build.sh`, and `scripts/notarize.sh`.

**CI Pipeline:**
- GitHub Actions in `.github/workflows/ci.yml` and `.github/workflows/update-contributors.yml`.

## Environment Configuration

**Required env vars:**
- No required secret env var contract is documented in checked-in Rust code for normal local development.
- Runtime/user environment configuration is driven primarily by Lua config under `~/.config/kaku/` per `README.md`.

**Secrets location:**
- Not detected in repository source.
- SSH credentials and OS keychain/agent state are implied by SSH support in `mux/src/ssh.rs` and `crates/wezterm-ssh/`, but not stored in-repo.

## Webhooks & Callbacks

**Incoming:**
- None detected as HTTP webhook endpoints.

**Outgoing:**
- Network calls for update/download and remote domain operations only; no webhook fan-out detected in checked-in code.

## Internal Integration Boundaries

**CLI to mux client:**
- `kaku/src/cli/mod.rs` creates a headless `mux::connui::ConnectionUI` and a `wezterm_client::client::Client`, then forwards subcommands to typed command modules in `kaku/src/cli/`.

**GUI to mux/runtime:**
- `kaku-gui/src/main.rs` sets up frontend/runtime state, attaches domains, spawns tabs, and updates mux domains.
- `kaku-gui/src/termwindow/mod.rs`, `kaku-gui/src/tabbar.rs`, and `kaku-gui/src/overlay/` subscribe to mux state and notifications.

**Mux core to panes/tabs/windows:**
- `mux/src/lib.rs` owns registries and `MuxNotification`.
- `mux/src/domain.rs` abstracts spawn/split/move semantics.
- `mux/src/pane.rs`, `mux/src/tab.rs`, and `mux/src/window.rs` implement pane/tab/window state and behavior.

## Control-Plane-Relevant Extension Points

**Preferred existing surfaces:**
- CLI machine-readable commands in `kaku/src/cli/`.
- command-palette / launcher integration in `kaku-gui/src/commands.rs` and `kaku-gui/src/overlay/launcher.rs`.
- tab and status affordances in `kaku-gui/src/tabbar.rs`.
- overlay workflows in `kaku-gui/src/overlay/mod.rs`.
- live pane/window UI state in `kaku-gui/src/termwindow/mod.rs`.
- event propagation in `mux/src/lib.rs`.

---

*Integration audit: 2026-03-26*
