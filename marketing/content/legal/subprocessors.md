---
title: Subprocessors
description: List of third-party subprocessors used by secapp cloud services.
lastUpdated: 2026-05-23
---

# Subprocessors

secapp is a local-first application. In its current version, **no subprocessors are used** because no data leaves your machine.

This page will be updated if and when cloud sync capabilities (planned for v3) are introduced, at which point subprocessors handling encrypted blobs will be listed here.

## Current Status

| Subprocessor | Purpose | Location | Data Processed |
|---|---|---|---|
| *None* | — | — | — |

secapp operates entirely on your local machine. All encryption, decryption, key management, and logging happens on-device. No data is transmitted to any server.

## Future: Turso Cloud Sync

When cloud sync becomes available as a premium feature, the following subprocessor arrangement will apply:

| Subprocessor | Purpose | Location | Data Processed |
|---|---|---|---|
| Turso, Inc. | Encrypted blob storage and sync | United States / EU | E2E encrypted blobs only —Turso never sees plaintext, keys, or filenames |

### Important notes about cloud sync:

- **End-to-end encryption**: All data is encrypted on your device before upload. Turso receives only opaque encrypted blobs.
- **No plaintext exposure**: The Turso server cannot read your data because it never has access to your master key or file keys.
- **Opt-in**: Cloud sync is not enabled by default. You must explicitly configure it.
- **Conflict resolution**: Last-write-wins with timestamp resolution. No manual merge is required.

A detailed subprocessor list and data processing agreement will be published before cloud sync launches.

## Changes

We will update this page within 30 days of adding or changing any subprocessor. For material changes, we will also provide notice through secapp release notes.

## Contact

For questions about subprocessors or data processing, open an issue on the secapp repository.