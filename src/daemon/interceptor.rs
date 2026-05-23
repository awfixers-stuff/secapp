use anyhow::{Context, Result};
use notify::{Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::crypto::{decrypt_file, encrypt_file, generate_file_key, sha256_hash, unwrap_file_key, wrap_file_key, MasterKey, EncryptedFileKey};
use crate::db::Database;

/// A file event detected by the watcher.
#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: PathBuf,
    pub kind: FileEventKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileEventKind {
    Created,
    Modified,
    Deleted,
}

impl FileEventKind {
    fn as_str(&self) -> &'static str {
        match self {
            FileEventKind::Created => "Created",
            FileEventKind::Modified => "Modified",
            FileEventKind::Deleted => "Deleted",
        }
    }
}

/// File interceptor that watches protected directories and encrypts/decrypts files.
pub struct Interceptor {
    config: Arc<Config>,
    db: Arc<Database>,
    master_key: Arc<RwLock<Option<MasterKey>>>,
    /// Set of files currently being processed, to avoid re-triggering on our own writes.
    processing: Arc<RwLock<HashSet<PathBuf>>>,
}

impl Interceptor {
    pub fn new(config: Arc<Config>, db: Arc<Database>, master_key: Arc<RwLock<Option<MasterKey>>>) -> Self {
        Self {
            config,
            db,
            master_key,
            processing: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Start watching protected directories for file changes.
    pub async fn watch(&self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<FileEvent>(1024);

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    if let Some(file_event) = convert_event(&event) {
                        let _ = tx.blocking_send(file_event);
                    }
                }
            },
            NotifyConfig::default(),
        )
        .context("creating file watcher")?;

        // Watch all protected paths
        for protected_path in &self.config.protected_paths {
            if protected_path.exists() {
                tracing::info!("Watching directory: {}", protected_path.display());
                watcher
                    .watch(protected_path, RecursiveMode::Recursive)
                    .with_context(|| format!("watching {}", protected_path.display()))?;
            } else {
                tracing::warn!("Protected path does not exist, skipping: {}", protected_path.display());
            }
        }

        tracing::info!("File interceptor started, processing events...");

        // Process events
        while let Some(event) = rx.recv().await {
            if let Err(e) = self.handle_event(event).await {
                tracing::error!("Error handling file event: {}", e);
            }
        }

        Ok(())
    }

    /// Handle a single file event.
    async fn handle_event(&self, event: FileEvent) -> Result<()> {
        let path_str = event.path.to_string_lossy().to_string();

        // Skip if we're already processing this file (avoid re-entrancy)
        {
            let processing = self.processing.read().await;
            if processing.contains(&event.path) {
                return Ok(());
            }
        }

        // Check if this path is under a protected directory
        if !self.is_protected(&event.path) {
            return Ok(());
        }

        // Mark as processing
        {
            let mut processing = self.processing.write().await;
            processing.insert(event.path.clone());
        }

        let result = match event.kind {
            FileEventKind::Created | FileEventKind::Modified => {
                self.handle_create_or_modify(&event.path).await
            }
            FileEventKind::Deleted => {
                self.handle_delete(&event.path).await
            }
        };

        // Remove from processing set
        {
            let mut processing = self.processing.write().await;
            processing.remove(&event.path);
        }

        if let Err(e) = &result {
            tracing::error!("Error processing {} for {}: {}", event.kind.as_str(), path_str, e);
        }

        result
    }

    /// Handle file creation or modification: encrypt the file if not already encrypted.
    async fn handle_create_or_modify(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();

        // Skip files that are already in our encrypted format (have .secapp extension)
        if path.extension().is_some_and(|ext| ext == "secapp") {
            return Ok(());
        }

        // Skip directories
        if path.is_dir() {
            return Ok(());
        }

        // Read the file
        let plaintext = match std::fs::read(path) {
            Ok(data) => data,
            Err(e) => {
                tracing::debug!("Cannot read file {}: {}", path_str, e);
                return Ok(()); // File may have been deleted between event and read
            }
        };

        // Skip empty files
        if plaintext.is_empty() {
            return Ok(());
        }

        let master_key_guard = self.master_key.read().await;
        let master_key = match master_key_guard.as_ref() {
            Some(key) => key,
            None => {
                tracing::warn!("No master key available, skipping encryption of {}", path_str);
                return Ok(());
            }
        };

        // Check if file is already tracked
        if let Some(existing) = self.db.get_protected_file(&path_str).await? {
            if existing.is_encrypted {
                tracing::debug!("File already encrypted: {}", path_str);
                return Ok(());
            }
        }

        // Generate per-file key and encrypt
        let file_key = generate_file_key();
        let encrypted_key = wrap_file_key(master_key, &file_key)?;
        let original_hash = hex::encode(sha256_hash(&plaintext));
        let ciphertext = encrypt_file(&plaintext, &file_key)?;
        let encrypted_hash = hex::encode(sha256_hash(&ciphertext));

        // Write encrypted file alongside original
        let encrypted_path = path.with_extension(
            format!("{}.secapp", path.extension().unwrap_or_default().to_string_lossy())
        );

        std::fs::write(&encrypted_path, &ciphertext)
            .with_context(|| format!("writing encrypted file {}", encrypted_path.display()))?;

        // Remove original plaintext file
        if let Err(e) = std::fs::remove_file(path) {
            tracing::warn!("Failed to remove original file {}: {}", path_str, e);
        }

        // Register in database
        self.db.add_protected_file(
            &path_str,
            &encrypted_key,
            &original_hash,
            &encrypted_hash,
            plaintext.len() as i64,
            ciphertext.len() as i64,
            true,
        ).await?;

        // Log the encryption
        self.db.log_access(
            &path_str,
            std::process::id() as i64,
            "secapp-daemon",
            "secapp-daemon",
            "encrypt",
            true,
        ).await?;

        tracing::info!("Encrypted file: {} -> {}", path_str, encrypted_path.display());
        Ok(())
    }

    /// Handle file deletion: remove from tracking.
    async fn handle_delete(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy().to_string();
        let encrypted_path = path.with_extension(
            format!("{}.secapp", path.extension().unwrap_or_default().to_string_lossy())
        );

        if encrypted_path.exists() {
            std::fs::remove_file(&encrypted_path)
                .with_context(|| format!("removing encrypted file {}", encrypted_path.display()))?;
        }

        self.db.remove_protected_file(&path_str).await?;
        tracing::info!("Removed tracking for deleted file: {}", path_str);
        Ok(())
    }

    /// Decrypt a protected file, returning the plaintext.
    pub async fn decrypt_file_by_path(&self, path: &Path) -> Result<Vec<u8>> {
        let path_str = path.to_string_lossy().to_string();
        let file_record = self.db.get_protected_file(&path_str)
            .await?
            .context(format!("file not tracked: {}", path_str))?;

        if !file_record.is_encrypted {
            anyhow::bail!("file is not encrypted: {}", path_str);
        }

        let master_key_guard = self.master_key.read().await;
        let master_key = master_key_guard.as_ref()
            .context("no master key available")?;

        let encrypted_key = EncryptedFileKey {
            encrypted_key: file_record.encrypted_key,
            nonce: file_record.key_nonce,
        };
        let file_key = unwrap_file_key(master_key, &encrypted_key)?;

        // Find the encrypted file
        let encrypted_path = path.with_extension(
            format!("{}.secapp", path.extension().unwrap_or_default().to_string_lossy())
        );

        let ciphertext = std::fs::read(&encrypted_path)
            .with_context(|| format!("reading encrypted file {}", encrypted_path.display()))?;

        let plaintext = decrypt_file(&ciphertext, &file_key)?;

        // Verify hash
        let current_hash = hex::encode(sha256_hash(&plaintext));
        if current_hash != file_record.original_hash {
            tracing::warn!("Hash mismatch for decrypted file {}: expected {}, got {}",
                path_str, file_record.original_hash, current_hash);
        }

        // Log the decryption
        self.db.log_access(
            &path_str,
            std::process::id() as i64,
            "secapp-daemon",
            "secapp-daemon",
            "decrypt",
            true,
        ).await?;

        Ok(plaintext)
    }

    /// Check if a path falls under any protected directory.
    fn is_protected(&self, path: &Path) -> bool {
        self.config.protected_paths.iter().any(|protected| {
            path.starts_with(protected)
        })
    }
}

/// Convert a notify event to our internal FileEvent.
fn convert_event(event: &Event) -> Option<FileEvent> {
    let kind = match event.kind {
        EventKind::Create(_) => FileEventKind::Created,
        EventKind::Modify(_) => FileEventKind::Modified,
        EventKind::Remove(_) => FileEventKind::Deleted,
        _ => return None,
    };

    let path = event.paths.first()?.clone();
    Some(FileEvent { path, kind })
}