---
title: Terms of Service
description: Terms governing your use of secapp.
lastUpdated: 2026-05-23
---

# Terms of Service

**Effective date**: May 23, 2026

These Terms of Service govern your use of secapp, the local-first security application for Linux. By using secapp, you agree to these terms.

## 1. Acceptance

By installing, running, or otherwise using secapp, you accept these Terms and the [AWFixer Source Available License](/legal/license) that governs the software itself. If you do not agree, do not use secapp.

## 2. Description

secapp is a system keystore that protects sensitive directories on your Linux machine against ransomware and unauthorized access. It monitors configured directories, encrypts files at rest using AES-256-GCM, and serves plaintext only to authorized processes.

**secapp is not a password manager.** It is a security application that makes your important files opaque to ransomware and unauthorized processes.

## 3. Use License

Your use of secapp is governed by the [AWFixer Source Available License v0.4](/legal/license). Key restrictions include:

- You may view, run, and internally evaluate the software
- You may **not** use it to develop competitive products or functionally equivalent software
- You may **not** use it to train AI/ML systems
- Redistribution requires a separate commercial license
- The license converts to AGPL-3.0 after four years from initial publication

See the full license for complete terms.

## 4. Personal and Small Entity Use

Natural persons and small entities (≤10 employees or <$2M USD annual revenue) may use, modify, and run secapp for personal, educational, or internal business purposes, including limited production use, provided they do not create or operate a competitive offering and comply with the license restrictions.

Entities that exceed these thresholds have a 180-day grace period to obtain a commercial license or cease use.

## 5. No Warranty

secapp is provided "as is" and "as available", without warranty of any kind, express or implied. This includes warranties of merchantability, fitness for a particular purpose, title, and non-infringement.

**In v1, there is no recovery mechanism.** If you lose your master password, your encrypted files cannot be recovered. Recovery keys are planned for v3.

## 6. Limitation of Liability

To the maximum extent permitted by law, the licensor is not liable for any indirect, incidental, special, consequential, or punitive damages, or any loss of profits, revenue, data, or goodwill arising from your use of secapp.

## 7. Security Considerations

You acknowledge that:

- secapp protects against offline access to sensitive files and unauthorized process access
- It does **not** protect against a compromised kernel, root-level attackers, or physical access while the keystore is unlocked
- Encrypted files are replaced on disk — there is no dual-file scheme
- You are responsible for remembering your master password (no recovery in v1)

## 8. Compliance and Audit

You agree to maintain records demonstrating compliance with the license, particularly regarding AI use restrictions and competitive offering prohibitions. Upon written request (limited to once per calendar year), you may be asked to provide a compliance certification.

See [License Section 5](/legal/license) for full compliance requirements.

## 9. Termination

Your rights terminate automatically upon any breach of the Source Available License, particularly Sections 3 (competitive use), 4 (AI restrictions), and 5.1 (log tampering). For other breaches, a 30-day cure period applies.

## 10. Governing Law

These Terms and any disputes are governed by the laws of the State of Delaware, U.S.A. Exclusive venue is the United States District Court for the District of Delaware, with a Chancery carve-out for equitable claims.

## 11. Changes

We may update these Terms from time to time. Changes will be posted on this page with an updated effective date. Continued use after changes constitutes acceptance.

## 12. Contact

For questions about these Terms, open an issue on the secapp repository.