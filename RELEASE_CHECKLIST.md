# Sifr v1 Release Checklist

Scope: release the existing functionality only, without schema migrations.

## 1. Release scope is frozen

- [x] No new features added after this checklist is approved.
- [x] Supported user flows are confirmed:
  - [x] Create vault
  - [x] Open vault
  - [x] Unlock vault
  - [x] Add/edit/delete/search entries
  - [x] Generate password
- [x] Schema migration work is explicitly postponed to v1.1+.

## 2. Security and secret handling

- [x] Confirm no secret values are logged (master password, entry password, TOTP secret).
- [x] Confirm sensitive strings are zeroized where implemented.
- [x] Confirm clipboard auto-clear behavior works for copied secrets.

## 3. Data integrity

- [x] Verify create/open/list/add/update/delete flows keep vault consistent.
- [x] Verify reopen flow preserves data.
- [x] Document backup guidance: copy vault file as-is.

## 4. Cross-platform quality gates

- [ ] CI matrix green on:
  - [ ] macOS
  - [ ] Linux
  - [ ] Windows
- [x] `cargo fmt --all -- --check` passes.
- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes.
- [x] `cargo test --all` passes.

## 5. Packaging and release artifacts

- [x] Version is set to 1.0.0.
- [x] Tag `v1.0.0` is created from release commit.
- [ ] GitHub Release is created from tag.
- [ ] Binaries are attached for:
  - [ ] macOS
  - [ ] Linux
  - [ ] Windows

## 6. User-facing docs

- [x] README contains quick start and platform notes.
- [x] Security note documents threat boundaries and recovery limits.
- [x] Changelog/release notes describe included functionality and current limits.

## 7. Final go/no-go

- [ ] Smoke test completed from built binary on each target platform.
- [ ] No P0/P1 issues remain.
- [ ] Release decision recorded as GO.
