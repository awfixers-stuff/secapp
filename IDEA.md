secapp is meant to be a way to secure the environment of your PC — SSH keys, GPG/PGP keys, environment files, logins, etc.

It does this by acting as the system keystore for your Linux box, sitting between any process and the files in ~/.local/, ~/.config/, ~/.ssh/, ~/.gnupg/ (with others configurable).

This is NOT a full password manager — it's a security application for protecting against ransomware in the user's important directories.

The database uses Turso (libsql) locally, with encrypted blob remote syncing planned as a premium feature.

---

## Design Decisions (resolved)

- **Interception**: v1 uses inotify (reactive encrypt-after-write). v2 will add FUSE for transparent proactive intercept.
- **Existing files**: Encrypt all existing files in protected directories on first run.
- **File scheme**: Encrypted blob replaces original. The daemon serves plaintext to authorized processes on access. No dual file.
- **Recovery key**: Planned for v3, not v1. Lost password = lost data for now.
- **Cloud sync**: E2E encrypted blobs to Turso cloud. Server never sees plaintext or keys.

## Design Decisions (open)

- Should we offer TPM/YubiKey integration beyond kernel keyring?
- Should we support system-wide protection with per-user keys?
- Should the config directory be ~/.secapp/ (per-user) or /etc/secapp/ (system daemon)?

---

## Component Architecture

secapp is composed of 5 components, each with a distinct responsibility and interface:

### 1. System Daemon (`secapp-daemon`)
The core privileged process. Replaces desktop keystore tools like gnome-keyring and seahorse. Runs as a high-permission system service that monitors all file activity in protected directories, enforces encryption policy, and serves plaintext to authorized processes. Owns the database, crypto key material, and interception logic.

### 2. GUI (`secapp-gui`)
Native desktop application built with gtk-rs. Authentication requires both the secapp account password and sudo password (account on first run, sudo periodically). Provides a visual surface for key management, policy configuration, encryption status, and audit log review.

### 3. TUI (`secapp-tui`)
Terminal interface for managing daemon settings. Semi-graphical (ratatui-style) interface for environments where a GUI isn't available or desired. **Open decision**: current implementation is Rust/ratatui; the new idea proposes Go. This needs resolution before any rewrite.

### 4. CLI (`secapp-cli`)
Command-line tool for setup, teardown, restart, and scripting. Already partially implemented via clap subcommands. Will grow as the daemon gains features (sync, recovery, policy management).

### 5. System Tray (`secapp-tray`)
System tray icon for GNOME/KDE. Shows daemon status (locked/unlocked/syncing) and provides quick actions (lock, unlock, open GUI). Initial target: libappindicator via gtk-rs. Extensible — could use GJS for richer tray UI in custom environments.

