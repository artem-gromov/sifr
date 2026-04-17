# Sifr

Sifr is a cross-platform terminal password manager with an encrypted SQLCipher vault.

Version: 1.0.0

## Included in v1

- Encrypted vault create/open
- Master password unlock
- Entry management (add, edit, delete, search)
- Password generation
- TOTP code generation for stored secrets
- Vault picker and terminal UI workflow

Schema migrations are intentionally postponed to a future release.

## Supported platforms

- macOS
- Linux
- Windows

The repository CI validates formatting, linting, and tests on all three platforms.

## Quick start

Build:

```bash
cargo build --all
```

Create a vault:

```bash
cargo run -p sifr -- new ./my-vault.sifr
```

Open a vault:

```bash
cargo run -p sifr -- open ./my-vault.sifr
```

Run without args (opens last vault or picker):

```bash
cargo run -p sifr --
```

Generate a password:

```bash
cargo run -p sifr -- gen --length 24
```

## Backup guidance

To back up your data, copy the vault file to a safe location.

## Security notes (v1)

- The vault is encrypted with SQLCipher.
- The vault key is derived from the master password with Argon2id.
- Secret fields are zeroized in memory where implemented.
- Clipboard copy of secrets is temporary and auto-clears.
- If the master password is lost, vault recovery is not possible.

See SECURITY.md for more details.

## Development checks

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

## Release process

- Follow RELEASE_CHECKLIST.md.
- Push tag `v1.0.0` to trigger GitHub release artifacts workflow.
