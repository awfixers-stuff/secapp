# secapp

**Linux system keystore protecting sensitive directories against ransomware.**

secapp sits between your processes and the files in `~/.config/`, `~/.local/`, `~/.ssh/`, `~/.gnupg/` (and any others you configure), encrypting them at rest and serving plaintext only to authorized processes. It is **not** a password manager — it is a security application that makes your important files opaque to ransomware and unauthorized access.

## How it works

1. **Inotify monitoring** — the daemon watches configured directories for file creation and modification events.
2. **Automatic encryption** — when a file is written, secapp encrypts it with a per-file AES-256-GCM key, replacing the plaintext on disk with an encrypted blob (`.secapp` extension).
3. **Key wrapping** — each per-file key is wrapped (encrypted) under a master key derived from your password via Argon2id. The master key is optionally stored in the Linux kernel keyring for the session duration.
4. **Access control** — policy rules (allow/deny/log) match on path globs and process names. Future versions will add per-process allowlisting via FUSE.
5. **Audit logging** — every encryption, decryption, and policy decision is recorded in the local database for review via the TUI or CLI.

Encrypted files never exist alongside plaintext — the original is replaced. The daemon holds the master key in memory (and optionally in the kernel session keyring) so authorized processes can decrypt on demand.

## Component architecture

| Component | Binary | Status | Description |
|---|---|---|---|
| System Daemon | `secapp daemon` | v1 | Privileged process that monitors filesystem events and manages encryption |
| TUI | `secapp start` | v1 | ratatui terminal interface for status, logs, policy, and directory management |
| CLI | `secapp <subcommand>` | v1 | clap-based CLI for scripted and headless operation |
| GUI | `secapp-gui` | Planned | gtk-rs native desktop app with dual authentication |
| System Tray | `secapp-tray` | Planned | GNOME/KDE tray icon with lock/unlock and status |

## Quick start

### Build

```bash
cargo build --release
```

### Initialize

```bash
secapp init
```

Creates `~/.secapp/config.json` with default protected directories (`~/.config`, `~/.local`, `~/.ssh`, `~/.gnupg`) and Argon2id parameters.

### Start the daemon (with TUI)

```bash
secapp start
```

Enter your master password when prompted. The daemon begins watching protected directories and the TUI opens.

### Start the daemon (headless)

```bash
secapp daemon
```

Same as above but without the TUI interface.

### Lock the keystore

```bash
secapp lock
```

Evicts the master key from memory and the kernel keyring. Encrypted files remain encrypted until you unlock again.

## CLI reference

```
secapp <COMMAND>

Commands:
  start       Start the daemon with TUI interface
  daemon      Start the daemon in background (no TUI)
  init        Initialize secapp configuration
  lock        Lock the keystore (clear master key from memory)
  list        List protected directories and their status
  logs        Show access logs [default: 50 entries]
  encrypt     Encrypt a specific file
  decrypt     Decrypt a specific file
  add-path    Add a directory to the protected list
  remove-path Remove a directory from the protected list
  policy      Manage access policies

Policy subcommands:
  secapp policy add <PATH_GLOB> <PROCESS_PATTERN> <allow|deny|log> [--priority N]
  secapp policy remove <ID>
  secapp policy list
```

### Examples

```bash
# Initialize with default directories
secapp init

# Add a custom protected directory
secapp add-path ~/projects/secrets

# Allow ssh to access all .ssh files
secapp policy add "~/.ssh/*" "ssh" allow --priority 10

# Deny all other processes from .ssh
secapp policy add "~/.ssh/*" "*" deny --priority 0

# View recent access logs
secapp logs --limit 100

# Lock the keystore before walking away
secapp lock
```

## Architecture

### Crypto

- **Key derivation**: Argon2id (OWASP-recommended params: 19456 KiB memory, 2 iterations, parallelism 1)
- **File encryption**: AES-256-GCM with per-file random keys
- **Key wrapping**: Per-file keys are encrypted under a subkey derived from the master key via HKDF-SHA256. The master key itself is never used directly for file encryption.
- **Integrity**: SHA-256 hashes of original plaintext are stored alongside encrypted files for tamper detection.
- **Key storage**: Master key can be cached in the Linux kernel session keyring (`keyutils`) so the daemon survives re-authentication without re-entering the password.

### Database

secapp uses libsql (Turso's open-source SQLite fork) for all local state, stored at `~/.secapp/secapp.db`. The schema tracks:

- **Protected files** — path, encrypted key, nonce, original/encrypted size, encryption status, timestamps
- **Access logs** — file path, process PID/name/cmdline, action, allowed/denied, timestamp
- **Policies** — path pattern, process pattern, action (allow/deny/log), priority
- **Key metadata** — salts and key-wrapping nonces for master key rotation

### Configuration

`~/.secapp/config.json`:

```json
{
  "protected_paths": ["/home/user/.config", "/home/user/.local", "/home/user/.ssh", "/home/user/.gnupg"],
  "encryption": {
    "memory_cost": 19456,
    "time_cost": 2,
    "parallelism": 1,
    "algorithm": "aes-256-gcm"
  },
  "daemon": {
    "poll_interval_ms": 1000,
    "autostart": false,
    "pid_file": "/home/user/.secapp/secapp.pid",
    "log_level": "info"
  }
}
```

### TUI

The ratatui-based TUI provides four tabs:

| Tab | Content |
|---|---|
| **Directories** | Protected directories and their file counts/encryption status |
| **Logs** | Scrollable access log with process attribution |
| **Policies** | Active allow/deny/log rules with priorities |
| **Status** | Daemon state, keyring status, protected file stats |

## Roadmap

### v1 — MVP (current)

- [x] Project scaffold, config, crypto, database, daemon, interceptor, TUI, CLI
- [ ] Existing-file encryption on first run
- [ ] Policy enforcement before encrypt/decrypt
- [ ] Graceful shutdown (SIGTERM/SIGINT, PID cleanup, keystore lock)
- [ ] Re-encryption on password change
- [ ] File integrity verification (SHA-256 mismatch alerts)
- [ ] Integration tests (daemon E2E, decrypt verification, policy override)
- [ ] systemd unit file, README, man page

### v2 — FUSE, GUI & system tray

- FUSE filesystem mount for transparent read-through decryption
- Per-process policy engine with `/proc/self` inspection
- Rate limiting as a ransomware heuristic
- `secapp-gui` — gtk-rs native app with dual authentication
- `secapp-tray` — system tray for GNOME/KDE

### v3 — Recovery & cloud sync

- 256-bit recovery key (Argon2id-derived from random bytes)
- Shamir's Secret Sharing for key splitting
- Turso cloud sync (E2E encrypted blobs, last-write-wins conflict resolution)

See [TODO.md](TODO.md) for the full task list and [IDEA.md](IDEA.md) for design decisions.

## Security model

**Threat model**: secapp protects against offline access to sensitive files (stolen disk, ransomware exfiltration) and unauthorized process access (ransomware encrypting or reading protected directories). It does **not** protect against a compromised kernel, root-level attackers, or physical access while the keystore is unlocked.

**Important**: There is no recovery mechanism in v1. If you lose your master password, your encrypted files cannot be recovered. Recovery keys are planned for v3.

**Key lifecycle**:
1. On `init`, a random salt is generated and the master key is derived from your password.
2. On `start`/`daemon`, you enter the password to derive the master key and unlock the keystore.
3. The master key is cached in the kernel session keyring for re-authentication.
4. On `lock`, the master key is purged from memory and the kernel keyring.
5. Per-file keys are generated randomly, used once for encryption, then stored wrapped under the master key.

## Development

### Prerequisites

- Rust 2024 edition (1.85+)
- C compiler (for libsql native build)
- Linux (kernel keyring, inotify)

> **Note**: secapp is a pure Rust project. All components — daemon, TUI, CLI, and the planned GUI and system tray — are written in Rust and use Rust-native libraries (ratatui, gtk-rs, libappindicator-rs, clap, tokio, libsql). There are no Go, Python, or shell dependencies for core logic.

### Running tests

```bash
cargo test
```

### Running the daemon locally

```bash
cargo run -- init
cargo run -- start
```

### Project structure

```
src/
  main.rs            CLI entry point and subcommand routing
  config/mod.rs      JSON configuration, protected paths, Argon2id params
  crypto/mod.rs      AES-256-GCM, Argon2id, HKDF, SHA-256, kernel keyring
  db/mod.rs          libsql schema, migrations, CRUD operations
  daemon/
    mod.rs           Daemon lifecycle, unlock/lock, PID management
    interceptor.rs   inotify watcher, auto-encrypt, file event handling
  tui/
    mod.rs           TUI module root
    app.rs           ratatui application, tabs, rendering, input handling
```

## License

Private project. All rights reserved.