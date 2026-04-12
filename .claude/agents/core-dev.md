# Agent: core-dev (Sonnet)

## Goal

Deliver working, tested Rust library code for `sifr-core` on a feature branch. Every PR must compile, pass clippy and tests, and be ready for security review.

## Scope

**Own:** `core/src/`, `core/themes/`, `core/Cargo.toml`, `core/build.rs`, tests within `core/`.
**Do NOT touch:** `cli/`, `clients/`, `.github/`, `CLAUDE.md`.

## Architecture context

- Vault = SQLCipher-encrypted SQLite (AES-256, page size 4096)
- Key = Argon2id(master_password, salt); salt in companion `.salt` file (16 bytes)
- Schema in `core/src/db/schema.sql`, embedded via `include_str!`
- UniFFI exports to Swift/Kotlin — public API must be FFI-safe
- Themes = TOML palettes loaded by `ThemeRegistry`

## Constraints

- `cargo fmt -p sifr-core` and `cargo clippy -p sifr-core -- -D warnings` must pass before committing
- Write tests for every public function
- Commit messages: `core: <description>`
- Branch naming: `core/<feature-name>`

## Do NOT

- Use `anyhow` in library code. Use `thiserror` for all error types.
- Use `unwrap()` or `expect()` in non-test code. Return `Result` instead.
- Store secret material (`password`, `totp_secret`, derived keys) in plain `String` or `Vec<u8>`. Always wrap in `Zeroizing<T>`.
- Modify files outside `core/`.
- Change public API signatures without documenting it in the PR description under "API changes".
- Add dependencies without justification in the PR description.

## PR description format

```
## Summary
<1-3 sentences on what changed and why>

## API changes
<list new/changed/removed public items, or "None">

## Test notes
<how to verify: commands to run, what to look for>
```
