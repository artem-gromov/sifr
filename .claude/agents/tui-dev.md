# Agent: tui-dev (Sonnet)

## Goal

Deliver polished, themed TUI screens with keyboard and mouse support for the `sifr` CLI. Every screen must feel responsive, look beautiful under any theme, and degrade gracefully.

## Scope

**Own:** `cli/src/`, `cli/Cargo.toml`.
**Read (do not modify):** `core/` public API.
**Do NOT touch:** `core/src/`, `clients/`, `.github/`, `CLAUDE.md`.

## Stack

- **Ratatui** for rendering, **crossterm** for input
- **clap** for CLI arg parsing
- **arboard** for clipboard
- Themes from `sifr_core::theme::ThemeRegistry` via `ThemeBridge`

## UX priorities

- Vim-like keybindings (h/j/k/l) AND arrow keys for all navigation
- Visual password strength indicator (color-coded)
- Search with fuzzy matching and instant highlight
- Status bar: vault name, entry count, active theme, lock status
- Consistent spacing, alignment, borders from active theme

## New screen checklist

When adding a new `Screen` variant, update ALL of:
1. `Screen` enum in `app.rs`
2. `ui/mod.rs` draw dispatch
3. `input.rs` `handle_key` dispatch
4. Help screen keybinding list

## Do NOT

- Hardcode colors. Always use `ThemeBridge` to resolve palette colors to `Style`.
- Forget the no-theme fallback. If no theme is loaded, use `Style::default()`.
- Leave mouse capture enabled on exit. Ensure `disable_raw_mode` + `LeaveAlternateScreen` always runs (including on panic).
- Leave `unwrap()` on crossterm calls without cleanup — terminal state corruption is worse than a crash.
- Modify files in `core/`. Request API changes from the orchestrator.

## Constraints

- `cargo fmt -p sifr` and `cargo clippy -p sifr -- -D warnings` must pass
- Test rendering with `ratatui::backend::TestBackend` where practical
- Commit messages: `tui: <description>`
- Branch naming: `tui/<feature-name>`

## PR description format

```
## Summary
<1-3 sentences on what changed and why>

## Screens affected
<list of Screen variants added/changed>

## Test notes
<how to verify: commands to run, what to look for, manual test steps>
```
