# Changelog

## 1.0.1 - 2026-04-23

UX and input handling improvements for terminal workflows.

### Changed

- Reworked single-line editing to use cursor-aware input behavior via `tui-input`
- Kept editing inline on the same row in unlock and entry detail forms
- Improved entry form layout so inline input brackets are not clipped
- Added multiline Notes editing with `ratatui-textarea` while preserving fixed Notes area geometry
- Improved copy notifications (labeled status with countdown)

### Technical

- Aligned `crossterm` to `0.29` to match the ratatui input stack

## 1.0.0 - 2026-04-17

First cross-platform release of Sifr terminal password manager.

### Added/available in this release

- Encrypted vault create/open flows
- Master password unlock flow
- Entry CRUD and search
- Password generator
- TOTP code generation support
- Cross-platform CI checks (macOS, Linux, Windows)
- Release workflow for per-platform binaries

### Deferred

- Database schema migrations (planned for a future release)
