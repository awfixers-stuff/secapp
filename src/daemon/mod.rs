pub mod interceptor;

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::crypto::{derive_master_key, generate_salt, MasterKey, store_key_in_keyring, retrieve_key_from_keyring, remove_key_from_keyring, KeySalt};
use crate::db::Database;
use interceptor::Interceptor;

/// Daemon state and lifecycle management.
pub struct Daemon {
    config: Arc<Config>,
    db: Arc<Database>,
    master_key: Arc<RwLock<Option<MasterKey>>>,
}

impl Daemon {
    /// Create a new daemon instance.
    pub async fn new(config: Config) -> Result<Self> {
        let db_path = Config::db_path()
            .context("resolving database path")?;
        let db = Database::open(&db_path).await
            .context("opening database")?;
        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(db),
            master_key: Arc::new(RwLock::new(None)),
        })
    }

    /// Unlock the keystore with a master password.
    /// If no key exists yet, derives a new one from the password.
    pub async fn unlock(&self, password: &str) -> Result<()> {
        let existing_salt = self.db.get_key_salt().await?;

        let master_key = if let Some(key_salt) = existing_salt {
            let salt_bytes = hex::decode(&key_salt.salt)
                .context("decoding key salt")?;
            derive_master_key(
                password,
                &salt_bytes,
                self.config.encryption.memory_cost,
                self.config.encryption.time_cost,
                self.config.encryption.parallelism,
            )?
        } else {
            let salt_bytes = generate_salt();
            let salt_hex = hex::encode(salt_bytes);
            let key_salt = KeySalt { salt: salt_hex };
            let master_key = derive_master_key(
                password,
                &salt_bytes,
                self.config.encryption.memory_cost,
                self.config.encryption.time_cost,
                self.config.encryption.parallelism,
            )?;
            self.db.store_key_salt(&key_salt).await?;
            master_key
        };

        // Try to store in kernel keyring for later use
        if store_key_in_keyring(&master_key) {
            tracing::info!("Master key stored in kernel keyring");
        } else {
            tracing::warn!("Could not store master key in kernel keyring; key held in memory only");
        }

        // Set the master key in memory
        {
            let mut key_guard = self.master_key.write().await;
            *key_guard = Some(master_key);
        }

        tracing::info!("Keystore unlocked successfully");
        Ok(())
    }

    /// Lock the keystore, clearing the master key from memory.
    pub async fn lock(&self) -> Result<()> {
        // Remove from kernel keyring
        remove_key_from_keyring();

        // Clear from memory
        {
            let mut key_guard = self.master_key.write().await;
            *key_guard = None;
        }

        tracing::info!("Keystore locked");
        Ok(())
    }

    /// Check if the keystore is currently unlocked.
    pub async fn is_unlocked(&self) -> bool {
        self.master_key.read().await.is_some()
    }

    /// Try to restore the master key from the kernel keyring.
    pub async fn try_keyring_restore(&self) -> Result<bool> {
        if let Some(key_bytes) = retrieve_key_from_keyring() {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                let master_key = MasterKey { key };
                let mut key_guard = self.master_key.write().await;
                *key_guard = Some(master_key);
                tracing::info!("Master key restored from kernel keyring");
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Start the daemon: begins file monitoring and encryption.
    pub async fn run(&self) -> Result<()> {
        if !self.is_unlocked().await {
            anyhow::bail!("keystore must be unlocked before starting the daemon");
        }

        let interceptor = Interceptor::new(
            Arc::clone(&self.config),
            Arc::clone(&self.db),
            Arc::clone(&self.master_key),
        );

        tracing::info!("Starting secapp daemon...");
        interceptor.watch().await
    }

    /// Get a reference to the database for queries.
    pub fn db(&self) -> &Arc<Database> {
        &self.db
    }

    /// Get a reference to the config.
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }
}

/// Handle daemon PID file management.
pub struct PidFile {
    path: PathBuf,
}

impl PidFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Write the current PID to the file. Returns error if already running.
    pub fn write(&self) -> Result<()> {
        if self.path.exists() {
            let existing = std::fs::read_to_string(&self.path)
                .context("reading PID file")?;
            let pid: i32 = existing.trim().parse()
                .context("parsing existing PID")?;
            if is_process_running(pid) {
                anyhow::bail!("secapp daemon already running with PID {}", pid);
            }
        }
        std::fs::write(&self.path, std::process::id().to_string())
            .context("writing PID file")?;
        Ok(())
    }

    /// Remove the PID file.
    pub fn remove(&self) -> Result<()> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .context("removing PID file")?;
        }
        Ok(())
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Check if a process with the given PID is running.
fn is_process_running(pid: i32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}