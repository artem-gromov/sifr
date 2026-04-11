# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

**Sifr** — cross-platform password manager with encrypted SQLCipher vault.  
Priority: **beauty, UX, customisation**.

Products:
- `sifr-core` — Rust library, all business logic, exposed to native clients via UniFFI
- `sifr` (cli/) — full TUI product built on Ratatui, ships on all platforms
- `clients/macos/` — Swift/SwiftUI app (Touch ID + Keychain), uses UniFFI-generated Swift bindings
- `clients/windows/`, `clients/linux/` — future native clients

## Commands

```bash
# Build everything
cargo build --all

# Run tests
cargo test --all

# Run a single test
cargo test -p sifr-core <test_name>

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format check
cargo fmt --all -- --check

# Format fix
cargo fmt --all

# Run CLI
cargo run -p sifr -- [args]
```

## Architecture

### Core library (`core/`)
- `crypto/` — Argon2id key derivation, password generator, `zeroize` on drop
- `db/` — SQLCipher connection setup; `schema.sql` embedded via `include_str!`; migrations tracked in `schema_version` table
- `models/` — `Entry`, `Category`, `Tag`; `Entry` zeroizes password/TOTP secret on drop
- `theme/` — `ThemeRegistry` loads bundled TOML themes + user TOML files; themes shared across CLI and macOS UI
- `vault.rs` — `Vault` struct: high-level open/create; key derived from master password + salt; salt stored in companion `.salt` file
- `build.rs` — UniFFI scaffolding generation for Swift/Kotlin bindings

### Theme system
TOML-defined palettes under `core/themes/`. Bundled: Dracula, Solarized Dark/Light, Nord, Catppuccin Mocha, Gruvbox Dark, Tokyo Night.  
Keys mirror terminal colour semantics (`background`, `surface`, `text`, `accent`, `green`, `red`, etc.) so any terminal theme translates directly.  
Users drop `.toml` files into their config dir; `ThemeRegistry::load_file` registers them at runtime.

### CLI (`cli/`)
Entry point only for now. Phase 1 will build the full Ratatui TUI.  
Uses `sifr-core` as a path dependency.

### macOS client (`clients/macos/`)
SwiftUI full-window app. Key integration points:
- UniFFI-generated Swift bindings from `sifr-core` (`.dylib`)
- `LocalAuthentication` for Touch ID / Face ID
- `Security.framework` Keychain for caching the derived vault key between sessions (item class `kSecClassGenericPassword`, service `"sifr"`)

### Vault file format
`.sifr` file = SQLCipher-encrypted SQLite database (AES-256, page size 4096).  
Key = Argon2id(master_password, salt).  
Salt stored alongside the vault in a `<file>.salt` companion file (16 bytes, not secret).

## Agent workflow

Main agent (orchestrator) in `main` branch.  
Worker agents run in isolated `git worktree` branches, implement features, open PRs.  
Main agent reviews diffs, runs `cargo test --all`, merges or requests changes.

When starting a new feature as a worker agent:
1. Work in your assigned worktree branch
2. Keep commits atomic and descriptive
3. `cargo fmt --all` and `cargo clippy` must pass before opening a PR
4. Open PR with `gh pr create` targeting `main`
