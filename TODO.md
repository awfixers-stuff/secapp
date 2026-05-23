# secapp TODO

## Component Architecture

secapp is composed of 5 components. The v1 MVP in this repo implements the daemon, TUI, and CLI in a single Cargo workspace. The GUI and system tray will be separate crates/binaries added in later phases.

| Component | Crate/Binary | Status | Notes |
|---|---|---|---|
| System Daemon | `secapp-daemon` (in `src/daemon/`) | v1 MVP | Core privileged process, replaces gnome-keyring |
| GUI | `secapp-gui` | Planned (v2+) | gtk-rs native app, dual auth (account + sudo) |
| TUI | `secapp-tui` (in `src/tui/`) | v1 MVP | ratatui terminal interface |
| CLI | `secapp` (in `src/main.rs`) | v1 MVP | clap subcommands for setup/teardown/restart |
| System Tray | `secapp-tray` | Planned (v2+) | libappindicator for GNOME/KDE, extensible via GJS |

**Open decision**: The original idea notes consider rewriting the TUI in Go. Current implementation is Rust/ratatui. This needs resolution before any rewrite.

---

## v1 — MVP (Current)

### Core
- [x] Project scaffold (Cargo.toml, module structure)
- [x] Config module (JSON config, protected paths, Argon2id params)
- [x] Crypto module (AES-256-GCM, Argon2id key derivation, HKDF key wrapping, SHA-256)
- [x] Database module (libsql schema + migrations, CRUD for files/logs/policies/salts)
- [x] Daemon module (lifecycle, unlock/lock, keyring restore)
- [x] File interceptor (inotify-based monitoring, auto-encrypt new/modified files)
- [x] TUI (ratatui: directories, logs, policies, status tabs)
- [x] CLI (clap: start, daemon, init, list, logs, policy, add-path, remove-path, lock)

### Hardening
- [ ] **Existing-file encryption on first run** — scan protected directories at daemon start and encrypt any plaintext files not yet tracked
- [ ] **Policy enforcement** — before encrypting/checking access, consult the access control policy table to decide allow/deny/log per (path glob, process pattern)
- [ ] **Graceful shutdown** — handle SIGTERM/SIGINT in daemon mode, clean up PID file, lock keystore on exit
- [ ] **Re-encryption on password change** — when master password changes, re-encrypt all wrapped file keys with new master key
- [ ] **File integrity verification** — on decrypt, compare SHA-256 hash against stored original hash; alert on mismatch

### Testing
- [x] Unit tests: crypto (encrypt/decrypt roundtrip, wrong key, tampered data, key wrapping)
- [x] Unit tests: db (protected files, access logs, policies, key salt CRUD)
- [x] Unit tests: config (defaults, add/remove paths, JSON roundtrip)
- [ ] Integration test: end-to-end daemon start → file creation → encryption observed on disk
- [ ] Integration test: decryption via `decrypt_file_by_path` produces original plaintext
- [ ] Integration test: policy override (deny rule blocks encryption/decryption)

### Distribution
- [ ] systemd unit file for `secapp daemon`
- [ ] README with build, install, and usage instructions
- [ ] Man page or `--help` polish for all subcommands

---

## v2 — FUSE, GUI & System Tray

### FUSE Transparent Decryption
- [ ] Replace/supplement inotify with FUSE filesystem mount
- [ ] Mount protected directories at a virtual path (e.g. `~/.config` backed by encrypted store at `~/.secapp/store/config/`)
- [ ] Authorized processes read plaintext transparently; unauthorized see ciphertext or get EACCES
- [ ] Lazy decryption: files decrypted into a memory/TTL cache only when accessed
- [ ] On unmount / daemon stop: purge plaintext cache, ensure all files encrypted at rest

### Access Control
- [ ] Per-process policy engine: allowlist/denylist by binary path, UID, or cgroup
- [ ] `/proc/self` inspection to identify calling process on each FUSE operation
- [ ] Rate limiting: block processes that exceed configured read/write thresholds (ransomware heuristic)

### GUI (`secapp-gui`)
- [ ] Scaffold `secapp-gui` crate with gtk-rs
- [ ] Dual authentication flow: account password on first run, sudo password required periodically
- [ ] Key management view (view protected keys, add/remove directories)
- [ ] Policy configuration view (allowlist/denylist per path glob, process pattern)
- [ ] Encryption status dashboard (files tracked, encrypted, last sync)
- [ ] Audit log viewer (filterable access log from database)

### System Tray (`secapp-tray`)
- [ ] Scaffold `secapp-tray` crate with libappindicator via gtk-rs
- [ ] Tray icon showing daemon status (locked/unlocked/syncing)
- [ ] Quick actions: lock, unlock, open GUI
- [ ] Desktop notification on ransomware detection or integrity failure

---

## v3 — Recovery & Cloud Sync

### Recovery Key
- [ ] Generate a 256-bit recovery key at initial setup (Argon2id-derived from random bytes, not a password)
- [ ] Print recovery key to terminal with instructions to store offline
- [ ] `secapp recover --key <recovery-key>` command to unlock keystore when password is lost
- [ ] Recovery key can optionally be split using Shamir's Secret Sharing (e.g., 3-of-5)

### Turso Cloud Sync (Premium)
- [ ] Encrypted blob upload: per-file encrypted blobs pushed to Turso cloud via libsql embedded replica
- [ ] E2E encryption: server never sees plaintext or file keys; only encrypted blobs + metadata
- [ ] Conflict resolution: last-write-wins with timestamp resolution
- [ ] `secapp sync push` / `secapp sync pull` commands
- [ ] Automatic sync on daemon start (if configured)

---

## Open Design Decisions

- **File naming on disk**: encrypted files use `<original_ext>.secapp` alongside original. v2 FUSE will eliminate dual-file scheme.
- **Kernel keyring**: currently stores master key in session keyring (`keyutils`). Is this sufficient, or should we offer TPM/YubiKey integration?
- **Multi-user**: current design is single-user per home directory. Should we support system-wide protection with per-user keys?
- **The `.secapp` config directory**: currently at `~/.secapp/`. Should this be `/etc/secapp/` for system daemon mode?
- **TUI language**: current implementation is Rust/ratatui. The original idea proposes Go. A rewrite would change the dependency profile and build toolchain. Needs resolution.
- **System daemon vs per-user daemon**: the component architecture frames the daemon as a system-level service (replacing gnome-keyring). This conflicts with the current per-user design. Need to decide: run as root with multi-user support, or per-user daemon with user-level permissions?

---

## Known Issues / Tech Debt

- inotify-based interceptor has a race condition: brief window where plaintext exists on disk before encryption
- `remove_file` of original plaintext can fail if the writing process still holds the file open
- TUI password input shows characters briefly (should mask with `*` consistently)
- Database tests use `std::env::temp_dir()` with PID-based names — could collide under pathological conditions
- `decrypt_file` and `unwrap_file_key` are public but currently only used by the interceptor; CLI shortcuts need master key plumbing
- PID file management uses best-effort `Drop` cleanup; should use `scopeguard` or structured shutdown for robustness