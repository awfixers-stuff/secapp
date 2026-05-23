use anyhow::{Context, Result};
use libsql::params;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::crypto::{EncryptedFileKey, KeySalt};

/// Database handle wrapping libsql connection.
pub struct Database {
    conn: libsql::Connection,
}

/// Metadata for a protected file tracked in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedFile {
    pub id: i64,
    pub path: String,
    pub encrypted_key: String,
    pub key_nonce: String,
    pub original_hash: String,
    pub encrypted_hash: String,
    pub original_size: i64,
    pub encrypted_size: i64,
    pub is_encrypted: bool,
    pub created_at: String,
    pub modified_at: String,
}

/// Access log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessLog {
    pub id: i64,
    pub file_path: String,
    pub process_pid: i64,
    pub process_name: String,
    pub process_cmdline: String,
    pub action: String,
    pub allowed: bool,
    pub timestamp: String,
}

/// Access policy for a path or process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: i64,
    pub path_pattern: String,
    pub process_pattern: String,
    pub action: String, // "allow", "deny", "log"
    pub priority: i64,
    pub created_at: String,
}

/// Key metadata stored for master key rotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: i64,
    pub salt: String,
    pub key_wrapping_nonce: String,
    pub created_at: String,
}

impl Database {
    /// Open or create the database at the given path.
    pub async fn open(db_path: &Path) -> Result<Self> {
        let db = libsql::Builder::new_local(db_path)
            .build()
            .await
            .with_context(|| format!("opening database at {}", db_path.display()))?;
        let conn = db.connect().context("connecting to database")?;
        let database = Database { conn };
        database.run_migrations().await?;
        Ok(database)
    }

    /// Run database migrations to ensure schema is up to date.
    async fn run_migrations(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS protected_files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path TEXT NOT NULL UNIQUE,
                    encrypted_key TEXT NOT NULL,
                    key_nonce TEXT NOT NULL,
                    original_hash TEXT NOT NULL DEFAULT '',
                    encrypted_hash TEXT NOT NULL DEFAULT '',
                    original_size INTEGER NOT NULL DEFAULT 0,
                    encrypted_size INTEGER NOT NULL DEFAULT 0,
                    is_encrypted INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE INDEX IF NOT EXISTS idx_protected_files_path ON protected_files(path);

                CREATE TABLE IF NOT EXISTS access_logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    file_path TEXT NOT NULL,
                    process_pid INTEGER NOT NULL,
                    process_name TEXT NOT NULL DEFAULT '',
                    process_cmdline TEXT NOT NULL DEFAULT '',
                    action TEXT NOT NULL,
                    allowed INTEGER NOT NULL DEFAULT 1,
                    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE INDEX IF NOT EXISTS idx_access_logs_timestamp ON access_logs(timestamp);
                CREATE INDEX IF NOT EXISTS idx_access_logs_path ON access_logs(file_path);

                CREATE TABLE IF NOT EXISTS policies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    path_pattern TEXT NOT NULL,
                    process_pattern TEXT NOT NULL DEFAULT '*',
                    action TEXT NOT NULL CHECK (action IN ('allow', 'deny', 'log')),
                    priority INTEGER NOT NULL DEFAULT 0,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );

                CREATE INDEX IF NOT EXISTS idx_policies_path ON policies(path_pattern);

                CREATE TABLE IF NOT EXISTS key_metadata (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    salt TEXT NOT NULL,
                    key_wrapping_nonce TEXT NOT NULL DEFAULT '',
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                ",
            )
            .await
            .context("running database migrations")?;
        Ok(())
    }

    // --- Protected files ---

    /// Register a file as protected and store its encrypted key.
    pub async fn add_protected_file(
        &self,
        path: &str,
        encrypted_key: &EncryptedFileKey,
        original_hash: &str,
        encrypted_hash: &str,
        original_size: i64,
        encrypted_size: i64,
        is_encrypted: bool,
    ) -> Result<i64> {
        let _result = self
            .conn
            .execute(
                "INSERT INTO protected_files (path, encrypted_key, key_nonce, original_hash, encrypted_hash, original_size, encrypted_size, is_encrypted)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(path) DO UPDATE SET
                    encrypted_key = ?2,
                    key_nonce = ?3,
                    original_hash = ?4,
                    encrypted_hash = ?5,
                    original_size = ?6,
                    encrypted_size = ?7,
                    is_encrypted = ?8,
                    modified_at = datetime('now')",
                params![
                    path,
                    encrypted_key.encrypted_key.clone(),
                    encrypted_key.nonce.clone(),
                    original_hash,
                    encrypted_hash,
                    original_size,
                    encrypted_size,
                    is_encrypted as i32,
                ],
            )
            .await
            .context("inserting protected file")?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get a protected file by its path.
    pub async fn get_protected_file(&self, path: &str) -> Result<Option<ProtectedFile>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, path, encrypted_key, key_nonce, original_hash, encrypted_hash, original_size, encrypted_size, is_encrypted, created_at, modified_at FROM protected_files WHERE path = ?1",
                params![path],
            )
            .await
            .context("querying protected file")?;

        if let Some(row) = rows.next().await.context("fetching protected file row")? {
            Ok(Some(ProtectedFile {
                id: row.get(0)?,
                path: row.get(1)?,
                encrypted_key: row.get(2)?,
                key_nonce: row.get(3)?,
                original_hash: row.get(4)?,
                encrypted_hash: row.get(5)?,
                original_size: row.get(6)?,
                encrypted_size: row.get(7)?,
                is_encrypted: row.get::<i32>(8)? != 0,
                created_at: row.get(9)?,
                modified_at: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all protected files.
    pub async fn list_protected_files(&self) -> Result<Vec<ProtectedFile>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, path, encrypted_key, key_nonce, original_hash, encrypted_hash, original_size, encrypted_size, is_encrypted, created_at, modified_at FROM protected_files ORDER BY path",
                params![],
            )
            .await
            .context("listing protected files")?;

        let mut files = Vec::new();
        while let Some(row) = rows.next().await.context("fetching protected file row")? {
            files.push(ProtectedFile {
                id: row.get(0)?,
                path: row.get(1)?,
                encrypted_key: row.get(2)?,
                key_nonce: row.get(3)?,
                original_hash: row.get(4)?,
                encrypted_hash: row.get(5)?,
                original_size: row.get(6)?,
                encrypted_size: row.get(7)?,
                is_encrypted: row.get::<i32>(8)? != 0,
                created_at: row.get(9)?,
                modified_at: row.get(10)?,
            });
        }
        Ok(files)
    }

    /// Remove a protected file from tracking.
    pub async fn remove_protected_file(&self, path: &str) -> Result<bool> {
        let result = self
            .conn
            .execute(
                "DELETE FROM protected_files WHERE path = ?1",
                params![path],
            )
            .await
            .context("removing protected file")?;
        Ok(result > 0)
    }

    /// Update the encryption status of a file.
    pub async fn set_file_encrypted(&self, path: &str, is_encrypted: bool) -> Result<()> {
        self.conn
            .execute(
                "UPDATE protected_files SET is_encrypted = ?2, modified_at = datetime('now') WHERE path = ?1",
                params![path, is_encrypted as i32],
            )
            .await
            .context("updating file encryption status")?;
        Ok(())
    }

    // --- Access logs ---

    /// Log an access event.
    pub async fn log_access(
        &self,
        file_path: &str,
        process_pid: i64,
        process_name: &str,
        process_cmdline: &str,
        action: &str,
        allowed: bool,
    ) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO access_logs (file_path, process_pid, process_name, process_cmdline, action, allowed) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![file_path, process_pid, process_name, process_cmdline, action, allowed as i32],
            )
            .await
            .context("inserting access log")?;
        Ok(self.conn.last_insert_rowid())
    }

    /// List recent access logs.
    pub async fn list_access_logs(&self, limit: i64) -> Result<Vec<AccessLog>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, file_path, process_pid, process_name, process_cmdline, action, allowed, timestamp FROM access_logs ORDER BY timestamp DESC LIMIT ?1",
                params![limit],
            )
            .await
            .context("listing access logs")?;

        let mut logs = Vec::new();
        while let Some(row) = rows.next().await.context("fetching access log row")? {
            logs.push(AccessLog {
                id: row.get(0)?,
                file_path: row.get(1)?,
                process_pid: row.get(2)?,
                process_name: row.get(3)?,
                process_cmdline: row.get(4)?,
                action: row.get(5)?,
                allowed: row.get::<i32>(6)? != 0,
                timestamp: row.get(7)?,
            });
        }
        Ok(logs)
    }

    /// List access logs for a specific file path.
    pub async fn list_access_logs_for_path(&self, path: &str, limit: i64) -> Result<Vec<AccessLog>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, file_path, process_pid, process_name, process_cmdline, action, allowed, timestamp FROM access_logs WHERE file_path = ?1 ORDER BY timestamp DESC LIMIT ?2",
                params![path, limit],
            )
            .await
            .context("listing access logs for path")?;

        let mut logs = Vec::new();
        while let Some(row) = rows.next().await.context("fetching access log row")? {
            logs.push(AccessLog {
                id: row.get(0)?,
                file_path: row.get(1)?,
                process_name: row.get(3)?,
                process_cmdline: row.get(4)?,
                action: row.get(5)?,
                allowed: row.get::<i32>(6)? != 0,
                process_pid: row.get(2)?,
                timestamp: row.get(7)?,
            });
        }
        Ok(logs)
    }

    // --- Policies ---

    /// Add an access policy.
    pub async fn add_policy(
        &self,
        path_pattern: &str,
        process_pattern: &str,
        action: &str,
        priority: i64,
    ) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO policies (path_pattern, process_pattern, action, priority) VALUES (?1, ?2, ?3, ?4)",
                params![path_pattern, process_pattern, action, priority],
            )
            .await
            .context("inserting policy")?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get the matching policy for a path and process, ordered by priority (highest first).
    pub async fn get_policy(&self, path: &str, process_name: &str) -> Result<Option<Policy>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, path_pattern, process_pattern, action, priority, created_at FROM policies
                 WHERE ?1 GLOB path_pattern AND (?2 GLOB process_pattern OR process_pattern = '*')
                 ORDER BY priority DESC LIMIT 1",
                params![path, process_name],
            )
            .await
            .context("querying policy")?;

        if let Some(row) = rows.next().await.context("fetching policy row")? {
            Ok(Some(Policy {
                id: row.get(0)?,
                path_pattern: row.get(1)?,
                process_pattern: row.get(2)?,
                action: row.get(3)?,
                priority: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all policies.
    pub async fn list_policies(&self) -> Result<Vec<Policy>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, path_pattern, process_pattern, action, priority, created_at FROM policies ORDER BY priority DESC",
                params![],
            )
            .await
            .context("listing policies")?;

        let mut policies = Vec::new();
        while let Some(row) = rows.next().await.context("fetching policy row")? {
            policies.push(Policy {
                id: row.get(0)?,
                path_pattern: row.get(1)?,
                process_pattern: row.get(2)?,
                action: row.get(3)?,
                priority: row.get(4)?,
                created_at: row.get(5)?,
            });
        }
        Ok(policies)
    }

    /// Remove a policy.
    pub async fn remove_policy(&self, id: i64) -> Result<bool> {
        let result = self
            .conn
            .execute("DELETE FROM policies WHERE id = ?1", params![id])
            .await
            .context("removing policy")?;
        Ok(result > 0)
    }

    // --- Key metadata ---

    /// Store key derivation salt for the current master key.
    pub async fn store_key_salt(&self, salt: &KeySalt) -> Result<i64> {
        // Remove any existing salt (only one active key at a time)
        self.conn
            .execute("DELETE FROM key_metadata", params![])
            .await
            .context("clearing old key metadata")?;

        self.conn
            .execute(
                "INSERT INTO key_metadata (salt) VALUES (?1)",
                params![salt.salt.clone()],
            )
            .await
            .context("inserting key salt")?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get the stored key salt.
    pub async fn get_key_salt(&self) -> Result<Option<KeySalt>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, salt, key_wrapping_nonce, created_at FROM key_metadata ORDER BY id DESC LIMIT 1",
                params![],
            )
            .await
            .context("querying key salt")?;

        if let Some(row) = rows.next().await.context("fetching key salt row")? {
            Ok(Some(KeySalt {
                salt: row.get(1)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{generate_file_key, generate_salt, wrap_file_key, derive_master_key};

    async fn test_db() -> Database {
        let db_path = std::env::temp_dir().join(format!("secapp_test_{}_{}.db", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let db = Database::open(&db_path).await.unwrap();
        db
    }

    async fn cleanup_db(db_path: &std::path::Path) {
        let _ = std::fs::remove_file(db_path);
        let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
        let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
    }

    #[tokio::test]
    async fn test_database_open_and_migrate() {
        let db = test_db().await;
        // Should have created all tables
        let files = db.list_protected_files().await.unwrap();
        assert!(files.is_empty());
    }

    #[tokio::test]
    async fn test_protected_file_crud() {
        let db = test_db().await;
        let salt = generate_salt();
        let master_key = derive_master_key("test", &salt, 1024, 1, 1).unwrap();
        let file_key = generate_file_key();
        let wrapped = wrap_file_key(&master_key, &file_key).unwrap();

        let id = db.add_protected_file(
            "/home/user/.ssh/id_rsa",
            &wrapped,
            "abc123",
            "def456",
            1000,
            1028,
            true,
        ).await.unwrap();
        assert!(id > 0);

        let file = db.get_protected_file("/home/user/.ssh/id_rsa").await.unwrap().unwrap();
        assert_eq!(file.path, "/home/user/.ssh/id_rsa");
        assert_eq!(file.original_size, 1000);
        assert!(file.is_encrypted);

        let files = db.list_protected_files().await.unwrap();
        assert_eq!(files.len(), 1);

        assert!(db.remove_protected_file("/home/user/.ssh/id_rsa").await.unwrap());
        assert!(db.get_protected_file("/home/user/.ssh/id_rsa").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_access_log_crud() {
        let db = test_db().await;

        let id = db.log_access(
            "/home/user/.ssh/id_rsa",
            12345,
            "ssh-agent",
            "/usr/bin/ssh-agent",
            "read",
            true,
        ).await.unwrap();
        assert!(id > 0);

        let logs = db.list_access_logs(10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].process_name, "ssh-agent");
        assert!(logs[0].allowed);

        let path_logs = db.list_access_logs_for_path("/home/user/.ssh/id_rsa", 10).await.unwrap();
        assert_eq!(path_logs.len(), 1);
    }

    #[tokio::test]
    async fn test_policy_crud() {
        let db = test_db().await;

        let id = db.add_policy("/home/user/.ssh/*", "*", "allow", 10).await.unwrap();
        assert!(id > 0);

        let id2 = db.add_policy("/home/user/.ssh/*", "malware", "deny", 20).await.unwrap();
        assert!(id2 > 0);

        // Higher priority policy wins
        let policy = db.get_policy("/home/user/.ssh/id_rsa", "malware").await.unwrap().unwrap();
        assert_eq!(policy.action, "deny");
        assert_eq!(policy.priority, 20);

        let policies = db.list_policies().await.unwrap();
        assert_eq!(policies.len(), 2);

        assert!(db.remove_policy(id).await.unwrap());
        assert!(db.remove_policy(id2).await.unwrap());
    }

    #[tokio::test]
    async fn test_key_salt_roundtrip() {
        let db = test_db().await;
        let salt = KeySalt { salt: hex::encode(generate_salt()) };

        db.store_key_salt(&salt).await.unwrap();
        let loaded = db.get_key_salt().await.unwrap().unwrap();
        assert_eq!(salt.salt, loaded.salt);
    }
}