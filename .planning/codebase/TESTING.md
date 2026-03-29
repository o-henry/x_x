# Testing Patterns

**Analysis Date:** 2026-03-26

## Test Framework

**Runner:**
- Rust built-in test harness for unit/integration tests across workspace crates.
- Config: no dedicated `nextest.toml` detected; orchestration lives in `Makefile` and `.github/workflows/ci.yml`.

**Assertion Library:**
- Standard `assert!` / `assert_eq!`
- `k9` snapshot assertions in files such as `mux/src/pane.rs`, `term/src/test/mod.rs`, and `kaku-gui/src/shapecache.rs`
- `rstest` for parameterized integration tests in `crates/wezterm-ssh/tests/e2e/*.rs`

**Run Commands:**
```bash
make test              # Run primary nextest suites
make check             # Run cargo check across key crates
cargo nextest run --locked -E 'not test(shapecache::test::ligatures_jetbrains)'  # Main CI test command
```

## Test File Organization

**Location:**
- Mostly colocated inline `#[cfg(test)]` modules inside production files such as `kaku/src/utils.rs`, `kaku/src/doctor.rs`, `mux/src/pane.rs`, `kaku-gui/src/commands.rs`, and `kaku-gui/src/tabbar.rs`.
- Some dedicated integration test directories exist in `term/src/test/`, `crates/wezterm-dynamic/tests/`, `crates/wezterm-ssh/tests/`, and shell smoke tests in `assets/shell-integration/tests/`.

**Naming:**
- Inline modules typically use `mod tests` or `mod test`.
- Integration tests use descriptive filenames such as `crates/wezterm-ssh/tests/e2e/sftp.rs` and `term/src/test/selection.rs`.

**Structure:**
```text
production_file.rs
  #[cfg(test)]
  mod tests { ... }

crate/tests/
  integration_case.rs

assets/shell-integration/tests/
  *.sh
```

## Test Structure

**Suite Organization:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specific_behavior_is_preserved() {
        // arrange
        // act
        // assert
    }
}
```

**Patterns:**
- Setup is usually lightweight and local to the file, as in `mux/src/pane.rs` where `FakePane` is defined inline for behavioral tests.
- Teardown is implicit through scope drop and temporary values rather than formal fixture frameworks.
- Assertions mix direct equality checks with snapshot verification for terminal/render output.

## Mocking

**Framework:** hand-rolled fakes and test-only structs

**Patterns:**
```rust
struct FakePane {
    lines: Mutex<Vec<Line>>,
}

impl Pane for FakePane {
    fn get_lines(&self, lines: Range<StableRowIndex>) -> (StableRowIndex, Vec<Line>) {
        // test-specific fake behavior
    }
}
```

**What to Mock:**
- Trait-backed runtime abstractions such as `Pane` or input/render helpers when testing layout and parsing logic in isolation.

**What NOT to Mock:**
- Pure parsing, formatting, or state-transition logic that can be exercised directly through real structs and enums.

## Fixtures and Factories

**Test Data:**
```rust
fn physical_lines_from_text(text: &str, width: usize) -> Vec<Line> {
    // helper used to construct terminal lines for pane tests
}
```

**Location:**
- Usually embedded inside the same file’s `#[cfg(test)]` module, for example `mux/src/pane.rs` and `term/src/test/mod.rs`.

## Coverage

**Requirements:** None enforced as an explicit numeric threshold in checked-in config.

**View Coverage:**
```bash
Not detected in repository automation
```

## Test Types

**Unit Tests:**
- Dominant style across the workspace.
- Focus on parser behavior, config normalization, pane/tab logic, render helpers, and command derivation in files such as `kaku/src/config_tui/mod.rs`, `mux/src/pane_encoding.rs`, and `kaku-gui/src/commands.rs`.

**Integration Tests:**
- Present in dedicated crate test folders, notably `crates/wezterm-ssh/tests/` and `crates/wezterm-dynamic/tests/`.
- Shell integration smoke tests are separate shell scripts under `assets/shell-integration/tests/` and run in CI.

**E2E Tests:**
- No GUI end-to-end browser/UI harness detected.
- Closest equivalent is shell smoke testing plus SSH integration tests and full app bundle validation in `.github/workflows/ci.yml`.

## Common Patterns

**Async Testing:**
```rust
// Async-heavy code is usually tested through sync helpers or specific crate-level test setups.
// No single workspace-wide async test harness pattern is enforced.
```

**Error Testing:**
```rust
let result = some_function(...);
assert!(result.is_err());
```

## CI-Enforced Checks

**Format gate:**
- `.github/workflows/ci.yml` runs `make fmt` and `make fmt-check`.

**Unit/integration gate:**
- `.github/workflows/ci.yml` runs `cargo nextest run --locked -E 'not test(shapecache::test::ligatures_jetbrains)'`.
- `.github/workflows/ci.yml` separately runs `cargo nextest run --locked -p wezterm-escape-parser`.

**Shell smoke gate:**
- `.github/workflows/ci.yml` runs:
  - `assets/shell-integration/tests/normalize_kaku_source_line_smoke.sh`
  - `assets/shell-integration/tests/cleanup_legacy_inline_block_smoke.sh`
  - `assets/shell-integration/tests/z_jump_provider_smoke.sh`
  - `assets/shell-integration/tests/starship_rprompt_smoke.sh`

## Control-Plane Testing Guidance

**For new mux state:**
- Add inline tests in `mux/src/lib.rs`, `mux/src/pane.rs`, `mux/src/tab.rs`, or `mux/src/window.rs` using small fakes rather than end-to-end UI scaffolding.

**For new CLI contracts:**
- Follow the existing serialization-focused pattern in `kaku/src/cli/list.rs` and assert on stable struct fields or rendered JSON where possible.

**For new GUI markers/overlays:**
- Prefer local logic tests in `kaku-gui/src/tabbar.rs`, `kaku-gui/src/commands.rs`, `kaku-gui/src/overlay/*`, or `kaku-gui/src/termwindow/*` before attempting full UI integration coverage.

---

*Testing analysis: 2026-03-26*
