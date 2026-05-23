use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use anyhow::{Context, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Size of the AES-256 key in bytes.
const KEY_SIZE: usize = 32;
/// Size of the AES-GCM nonce in bytes.
const NONCE_SIZE: usize = 12;
/// Size of the Argon2id salt in bytes.
const SALT_SIZE: usize = 32;

/// Master key derived from the user's password.
#[derive(Debug)]
pub struct MasterKey {
    pub key: [u8; KEY_SIZE],
}

/// Per-file encryption key, encrypted under the master key.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedFileKey {
    /// Hex-encoded encrypted key (AES-256-GCM wrapped).
    pub encrypted_key: String,
    /// Hex-encoded nonce used to encrypt the key.
    pub nonce: String,
}

/// Salt for Argon2id key derivation, stored alongside the key.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeySalt {
    /// Hex-encoded salt bytes.
    pub salt: String,
}

/// Derive a master key from a password using Argon2id.
pub fn derive_master_key(
    password: &str,
    salt: &[u8],
    memory_cost: u32,
    time_cost: u32,
    parallelism: u32,
) -> Result<MasterKey> {
    let params = Params::new(memory_cost, time_cost, parallelism, Some(KEY_SIZE))
        .map_err(|e| anyhow::anyhow!("invalid Argon2id parameters: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_SIZE];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("Argon2id key derivation failed: {}", e))?;

    Ok(MasterKey { key })
}

/// Generate a random salt for key derivation.
pub fn generate_salt() -> [u8; SALT_SIZE] {
    let mut salt = [0u8; SALT_SIZE];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Encrypt data using AES-256-GCM with a per-file key.
/// Returns the ciphertext (nonce || ciphertext || tag).
pub fn encrypt_file(plaintext: &[u8], file_key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(file_key)
        .map_err(|e| anyhow::anyhow!("invalid file key: {}", e))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt data encrypted by `encrypt_file`.
/// Expects input format: nonce || ciphertext || tag.
pub fn decrypt_file(ciphertext: &[u8], file_key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
    if ciphertext.len() < NONCE_SIZE + 16 {
        anyhow::bail!(
            "ciphertext too short (expected at least {} bytes, got {})",
            NONCE_SIZE + 16,
            ciphertext.len()
        );
    }
    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new_from_slice(file_key)
        .map_err(|e| anyhow::anyhow!("invalid file key: {}", e))?;
    let plaintext = cipher
        .decrypt(nonce, encrypted)
        .map_err(|e| anyhow::anyhow!("decryption failed: {}", e))?;
    Ok(plaintext)
}

/// Generate a random per-file key.
pub fn generate_file_key() -> [u8; KEY_SIZE] {
    let mut key = [0u8; KEY_SIZE];
    OsRng.fill_bytes(&mut key);
    key
}

/// Wrap (encrypt) a per-file key under the master key.
/// The file key is encrypted with AES-256-GCM using a derived subkey
/// from the master key. This avoids reusing the master key directly.
pub fn wrap_file_key(master_key: &MasterKey, file_key: &[u8; KEY_SIZE]) -> Result<EncryptedFileKey> {
    let subkey = derive_wrapping_key(master_key)?;
    let cipher = Aes256Gcm::new_from_slice(&subkey)
        .map_err(|e| anyhow::anyhow!("invalid wrapping key: {}", e))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted = cipher
        .encrypt(&nonce, file_key.as_slice())
        .map_err(|e| anyhow::anyhow!("key wrapping failed: {}", e))?;

    Ok(EncryptedFileKey {
        encrypted_key: hex::encode(&encrypted),
        nonce: hex::encode(nonce.as_slice()),
    })
}

/// Unwrap (decrypt) a per-file key from its encrypted form.
pub fn unwrap_file_key(master_key: &MasterKey, encrypted: &EncryptedFileKey) -> Result<[u8; KEY_SIZE]> {
    let subkey = derive_wrapping_key(master_key)?;
    let cipher = Aes256Gcm::new_from_slice(&subkey)
        .map_err(|e| anyhow::anyhow!("invalid wrapping key: {}", e))?;

    let nonce_bytes = hex::decode(&encrypted.nonce)
        .context("invalid hex in wrapped key nonce")?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let encrypted_key_bytes = hex::decode(&encrypted.encrypted_key)
        .context("invalid hex in wrapped key")?;

    let decrypted = cipher
        .decrypt(nonce, encrypted_key_bytes.as_slice())
        .map_err(|e| anyhow::anyhow!("key unwrapping failed: {}", e))?;

    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&decrypted);
    Ok(key)
}

/// Compute the SHA-256 hash of data (for integrity checking).
pub fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Derive a key-wrapping subkey from the master key using HKDF-SHA256.
fn derive_wrapping_key(master_key: &MasterKey) -> Result<[u8; KEY_SIZE]> {
    let hkdf = Hkdf::<Sha256>::new(None, &master_key.key);
    let mut subkey = [0u8; KEY_SIZE];
    hkdf.expand(b"secapp-key-wrapping-v1", &mut subkey)
        .map_err(|e| anyhow::anyhow!("HKDF expansion failed: {}", e))?;
    Ok(subkey)
}

/// Compute the encrypted file size from the plaintext size.
/// Encrypted = nonce (12) + plaintext + tag (16).
pub fn encrypted_size(plaintext_len: usize) -> usize {
    plaintext_len + NONCE_SIZE + 16
}

/// Compute the plaintext size from an encrypted blob size.
/// Returns None if the encrypted size is too small.
pub fn decrypted_size(encrypted_len: usize) -> Option<usize> {
    encrypted_len.checked_sub(NONCE_SIZE + 16)
}

/// Try to store the master key in the kernel keyring.
/// Returns true if stored successfully.
pub fn store_key_in_keyring(master_key: &MasterKey) -> bool {
    use keyutils::keytypes::User;

    let mut keyring = match keyutils::Keyring::attach_or_create(keyutils::SpecialKeyring::Session) {
        Ok(kr) => kr,
        Err(_) => {
            tracing::warn!("could not attach to session keyring");
            return false;
        }
    };

    match keyring.add_key::<User, &str, &[u8]>("secapp:master", &master_key.key[..]) {
        Ok(_key) => {
            tracing::info!("master key stored in kernel keyring");
            true
        }
        Err(e) => {
            tracing::warn!("could not store master key in kernel keyring: {:?}", e);
            false
        }
    }
}

/// Retrieve the master key from the kernel keyring.
/// Returns the raw key bytes if found.
pub fn retrieve_key_from_keyring() -> Option<Vec<u8>> {
    use keyutils::keytypes::User;

    let keyring = keyutils::Keyring::attach(keyutils::SpecialKeyring::Session).ok()?;

    let key = keyring.search_for_key::<User, &str, _>("secapp:master", None).ok()?;

    let key_data = key.read().ok()?;

    if key_data.len() == KEY_SIZE {
        Some(key_data)
    } else {
        None
    }
}

/// Remove the master key from the kernel keyring.
pub fn remove_key_from_keyring() -> bool {
    use keyutils::keytypes::User;

    let keyring = match keyutils::Keyring::attach(keyutils::SpecialKeyring::Session) {
        Ok(kr) => kr,
        Err(_) => return false,
    };

    if let Ok(key) = keyring.search_for_key::<User, &str, _>("secapp:master", None) {
        key.revoke().is_ok()
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = generate_file_key();
        let plaintext = b"Hello, secapp! This is a test file with some content.";
        let encrypted = encrypt_file(plaintext, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn encrypt_decrypt_empty_file() {
        let key = generate_file_key();
        let plaintext = b"";
        let encrypted = encrypt_file(plaintext, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn encrypt_decrypt_large_file() {
        let key = generate_file_key();
        let plaintext: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
        let encrypted = encrypt_file(&plaintext, &key).unwrap();
        let decrypted = decrypt_file(&encrypted, &key).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn wrong_key_fails() {
        let key1 = generate_file_key();
        let key2 = generate_file_key();
        let plaintext = b"secret data";
        let encrypted = encrypt_file(plaintext, &key1).unwrap();
        let result = decrypt_file(&encrypted, &key2);
        assert!(result.is_err(), "decryption with wrong key should fail");
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = generate_file_key();
        let plaintext = b"secret data";
        let mut encrypted = encrypt_file(plaintext, &key).unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;
        let result = decrypt_file(&encrypted, &key);
        assert!(result.is_err(), "decryption of tampered data should fail");
    }

    #[test]
    fn key_wrapping_roundtrip() {
        let salt = generate_salt();
        let master_key = derive_master_key("test-password", &salt, 1024, 1, 1).unwrap();
        let file_key = generate_file_key();
        let wrapped = wrap_file_key(&master_key, &file_key).unwrap();
        let unwrapped = unwrap_file_key(&master_key, &wrapped).unwrap();
        assert_eq!(file_key, unwrapped);
    }

    #[test]
    fn key_wrapping_wrong_master_fails() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let master1 = derive_master_key("password-1", &salt1, 1024, 1, 1).unwrap();
        let master2 = derive_master_key("password-2", &salt2, 1024, 1, 1).unwrap();
        let file_key = generate_file_key();
        let wrapped = wrap_file_key(&master1, &file_key).unwrap();
        let result = unwrap_file_key(&master2, &wrapped);
        assert!(result.is_err(), "unwrapping with wrong master key should fail");
    }

    #[test]
    fn master_key_derivation_consistent() {
        let salt = generate_salt();
        let key1 = derive_master_key("password", &salt, 1024, 1, 1).unwrap();
        let key2 = derive_master_key("password", &salt, 1024, 1, 1).unwrap();
        assert_eq!(key1.key, key2.key);
    }

    #[test]
    fn master_key_derivation_different_password() {
        let salt = generate_salt();
        let key1 = derive_master_key("password-1", &salt, 1024, 1, 1).unwrap();
        let key2 = derive_master_key("password-2", &salt, 1024, 1, 1).unwrap();
        assert_ne!(key1.key, key2.key);
    }

    #[test]
    fn sha256_deterministic() {
        let data = b"test data";
        let hash1 = sha256_hash(data);
        let hash2 = sha256_hash(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn encrypted_size_calculation() {
        assert_eq!(encrypted_size(0), NONCE_SIZE + 16);
        assert_eq!(encrypted_size(100), 100 + NONCE_SIZE + 16);
        assert_eq!(decrypted_size(encrypted_size(100)), Some(100));
        assert_eq!(decrypted_size(10), None);
    }
}