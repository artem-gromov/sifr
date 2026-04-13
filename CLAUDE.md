# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

## Project

**Sifr** — cross-platform password manager with encrypted SQLCipher vault.  
Priority: **UX, security**.

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

## Multi-agent system

**Orchestrator** (Opus 4.6) — plans, delegates, reviews PRs, merges. Does not write feature code.  
**Worker agents** (Sonnet) — implement features in isolated git worktrees.  
Role definitions: `.claude/agents/`

| Agent | Scope | Branch prefix |
|-------|-------|---------------|
| `core-dev` | `core/` — crypto, DB, models, vault, themes | `core/` |
| `tui-dev` | `cli/` — Ratatui TUI, UX, keybindings | `tui/` |
| `security-reviewer` | all (read-only) — audit report, no code | — |

### Dispatch rules
- Agents are spawned with `model: "sonnet"` and `isolation: "worktree"`
- Each agent receives ONLY its role file + specific task + relevant code snippets
- `core-dev` and `tui-dev` can run in parallel when tasks are independent
- `security-reviewer` runs after implementation, before merge
- PRs are squash-merged, branches deleted after merge
