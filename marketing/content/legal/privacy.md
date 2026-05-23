---
title: Privacy Policy
description: How secapp handles your data and privacy.
lastUpdated: 2026-05-23
---

# Privacy Policy

**Effective date**: May 23, 2026

secapp is a local-first security application. We collect virtually no data about you. This policy explains what minimal information we handle and how.

## 1. Data We Collect

### 1.1 Local Data

secapp operates entirely on your local machine. All sensitive data — including encrypted files, cryptographic keys, access logs, and configuration — is stored locally at `~/.secapp/`. None of this data leaves your machine unless you explicitly configure cloud sync (a future premium feature).

### 1.2 Telemetry

secapp does **not** collect telemetry, analytics, crash reports, or usage statistics. No data is sent to any server during normal operation.

### 1.3 Compliance Logging

The secapp daemon maintains local, on-device logs of file access events and policy decisions. These logs exist solely for:

- Verifying encryption and access policies are functioning correctly
- Detecting potential ransomware or unauthorized access patterns
- Providing you with an audit trail via the TUI or CLI

These logs are **not transmitted** to us or any third party, except for minimal tamper-detection alerts (e.g., notification that logging has been disabled or deleted). See the [License](/legal/license) Section 5.1 for details.

## 2. Data We Do Not Collect

We do not collect, store, or process:

- Personal identification information
- File contents or filenames
- Browsing or usage patterns
- Device fingerprints
- IP addresses
- Location data

## 3. Cloud Sync (Future Feature)

When cloud sync via Turso becomes available, the following will apply:

- All data uploaded to Turso cloud is **end-to-end encrypted**. The server never sees plaintext file contents, keys, or filenames.
- Only encrypted blobs and minimal metadata are transmitted.
- Sync is opt-in and can be disabled at any time.
- A separate privacy supplement will be published before this feature launches.

## 4. Third-Party Services

secapp does not integrate with third-party analytics, advertising, or tracking services.

When cloud sync launches, Turso (the database provider) will process only encrypted blobs. Their privacy policy is available at [turso.tech/privacy](https://turso.tech/privacy).

## 5. Data Retention

- **Local data**: Retained until you delete it. Uninstalling secapp removes the daemon and TUI, but encrypted files and `~/.secapp/` remain until manually removed.
- **Compliance logs**: Stored locally. You may delete them, but doing so may constitute a breach of the Source Available License (Section 5.1).
- **No server-side data**: We retain no server-side data about your usage.

## 6. Security

secapp uses:

- **AES-256-GCM** for file encryption with per-file random keys
- **Argon2id** for master key derivation (OWASP-recommended parameters)
- **HKDF-SHA256** for key wrapping (the master key is never used directly)
- **Linux kernel keyring** integration for secure key caching during sessions

Encrypted files replace plaintext on disk. No dual-file scheme exists — the original is securely replaced.

## 7. Your Rights

You have the right to:

- Access all data stored on your machine (it's all in `~/.secapp/`)
- Delete any or all local data at any time
- Refuse cloud sync (it's opt-in)
- Audit the source code (it's source-available under the AWFixer Source Available License)

## 8. Children's Privacy

secapp is not directed at children under 13. We do not knowingly collect data from children.

## 9. Changes to This Policy

We may update this policy from time to time. Changes will be posted on this page with an updated effective date. Material changes will be noted in secapp release notes.

## 10. Contact

For privacy-related inquiries, open an issue on the secapp repository or contact the maintainer directly.