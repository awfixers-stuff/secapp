use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_DIR: &str = ".secapp";
const CONFIG_FILE: &str = "config.json";

/// Default protected directories relative to $HOME
const DEFAULT_PROTECTED_DIRS: &[&str] = &[".config", ".local", ".ssh", ".gnupg"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Directories to protect (absolute paths)
    pub protected_paths: Vec<PathBuf>,
    /// Encryption settings
    pub encryption: EncryptionConfig,
    /// Daemon settings
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Argon2id memory cost in KiB
    pub memory_cost: u32,
    /// Argon2id time iterations
    pub time_cost: u32,
    /// Argon2id parallelism
    pub parallelism: u32,
    /// File encryption algorithm
    pub algorithm: EncryptionAlgorithm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Polling interval for file changes in milliseconds
    pub poll_interval_ms: u64,
    /// Whether to start on boot
    pub autostart: bool,
    /// PID file location
    pub pid_file: PathBuf,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        Self {
            protected_paths: DEFAULT_PROTECTED_DIRS
                .iter()
                .map(|d| home.join(d))
                .collect(),
            encryption: EncryptionConfig::default(),
            daemon: DaemonConfig::default(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            // OWASP recommended minimums for Argon2id
            memory_cost: 19_456, // 19 MiB
            time_cost: 2,
            parallelism: 1,
            algorithm: EncryptionAlgorithm::Aes256Gcm,
        }
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
        Self {
            poll_interval_ms: 1000,
            autostart: false,
            pid_file: home.join(DEFAULT_CONFIG_DIR).join("secapp.pid"),
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from disk, or create default if not found.
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("reading config from {}", config_path.display()))?;
            let config: Config = serde_json::from_str(&content)
                .with_context(|| format!("parsing config from {}", config_path.display()))?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to disk.
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating config dir {}", parent.display()))?;
        }
        let content = serde_json::to_string_pretty(self)
            .context("serializing config")?;
        std::fs::write(&config_path, content)
            .with_context(|| format!("writing config to {}", config_path.display()))?;
        Ok(())
    }

    /// Add a path to the protected list if not already present.
    pub fn add_protected_path(&mut self, path: &Path) -> bool {
        let abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        };
        if self.protected_paths.contains(&abs) {
            return false;
        }
        self.protected_paths.push(abs);
        true
    }

    /// Remove a path from the protected list.
    pub fn remove_protected_path(&mut self, path: &Path) -> bool {
        let original_len = self.protected_paths.len();
        self.protected_paths.retain(|p| p != path);
        self.protected_paths.len() != original_len
    }

    /// Get the path to the config directory.
    pub fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("cannot determine home directory")?;
        Ok(home.join(DEFAULT_CONFIG_DIR))
    }

    /// Get the path to the config file.
    fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE))
    }

    /// Get the path to the database file.
    pub fn db_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("secapp.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_protected_paths() {
        let config = Config::default();
        assert!(!config.protected_paths.is_empty());
        // Should contain at least .ssh and .config
        let has_ssh = config.protected_paths.iter().any(|p| p.to_string_lossy().contains(".ssh"));
        let has_config = config.protected_paths.iter().any(|p| p.to_string_lossy().contains(".config"));
        assert!(has_ssh, "default should protect .ssh");
        assert!(has_config, "default should protect .config");
    }

    #[test]
    fn add_remove_protected_path() {
        let mut config = Config::default();
        let test_path = PathBuf::from("/tmp/secapp-test-dir");
        assert!(config.add_protected_path(&test_path));
        assert!(!config.add_protected_path(&test_path)); // duplicate
        assert!(config.remove_protected_path(&test_path));
        assert!(!config.remove_protected_path(&test_path)); // already removed
    }

    #[test]
    fn encryption_config_defaults_sane() {
        let enc = EncryptionConfig::default();
        assert_eq!(enc.algorithm, EncryptionAlgorithm::Aes256Gcm);
        assert!(enc.memory_cost >= 19_456, "OWASP minimum memory cost");
        assert!(enc.time_cost >= 2, "OWASP minimum time cost");
    }

    #[test]
    fn config_roundtrip_json() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.protected_paths, parsed.protected_paths);
        assert_eq!(config.encryption.algorithm, parsed.encryption.algorithm);
    }
}