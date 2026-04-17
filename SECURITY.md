# Security Notes

## Threat model boundary (v1)

Sifr v1 is designed to protect vault contents at rest using strong encryption and a master-password-derived key.

Sifr v1 does not protect against:

- A fully compromised host system
- Malware with access to process memory/input
- Shoulder surfing or local screen capture

## Encryption model

- Vault storage uses SQLCipher.
- Encryption key is derived from the master password using Argon2id.
- Salt is stored in the SQLCipher file header.

## Secret handling

- Sensitive fields are zeroized where implemented.
- Clipboard copies are temporary and auto-cleared after timeout.
- Secret values must not be logged.

## Data recovery

- There is no password reset or key escrow in v1.
- Losing the master password means vault data cannot be recovered.

## Operational guidance

- Keep system and dependencies updated.
- Back up vault files securely.
- Protect endpoints with full-disk encryption and strong OS authentication.
