//! runtime:crypto extension - Cryptographic operations for Forge apps
//!
//! Provides secure random generation, hashing, HMAC, and symmetric encryption
//! using the ring cryptography library.

use deno_core::{op2, Extension, OpState};
use forge_weld_macro::{weld_op, weld_struct};
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::digest::{digest, SHA256, SHA384, SHA512};
use ring::hmac;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::debug;

// ============================================================================
// Error Types with Structured Codes
// ============================================================================

/// Error codes for crypto operations (8000-8009)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CryptoErrorCode {
    /// Generic crypto error
    Generic = 8000,
    /// Invalid algorithm specified
    InvalidAlgorithm = 8001,
    /// Invalid key length
    InvalidKeyLength = 8002,
    /// Encryption failed
    EncryptionFailed = 8003,
    /// Decryption failed
    DecryptionFailed = 8004,
    /// Hash operation failed
    HashFailed = 8005,
    /// HMAC operation failed
    HmacFailed = 8006,
    /// Key generation failed
    KeyGenerationFailed = 8007,
    /// Key derivation failed
    KeyDerivationFailed = 8008,
    /// Verification failed
    VerificationFailed = 8009,
}

/// Custom error type for crypto operations
#[derive(Debug, thiserror::Error, deno_error::JsError)]
pub enum CryptoError {
    #[error("[{code}] Crypto error: {message}")]
    #[class(generic)]
    Generic { code: u32, message: String },

    #[error("[{code}] Invalid algorithm: {message}")]
    #[class(generic)]
    InvalidAlgorithm { code: u32, message: String },

    #[error("[{code}] Invalid key length: {message}")]
    #[class(generic)]
    InvalidKeyLength { code: u32, message: String },

    #[error("[{code}] Encryption failed: {message}")]
    #[class(generic)]
    EncryptionFailed { code: u32, message: String },

    #[error("[{code}] Decryption failed: {message}")]
    #[class(generic)]
    DecryptionFailed { code: u32, message: String },

    #[error("[{code}] Hash failed: {message}")]
    #[class(generic)]
    HashFailed { code: u32, message: String },

    #[error("[{code}] HMAC failed: {message}")]
    #[class(generic)]
    HmacFailed { code: u32, message: String },

    #[error("[{code}] Key generation failed: {message}")]
    #[class(generic)]
    KeyGenerationFailed { code: u32, message: String },

    #[error("[{code}] Key derivation failed: {message}")]
    #[class(generic)]
    KeyDerivationFailed { code: u32, message: String },

    #[error("[{code}] Verification failed: {message}")]
    #[class(generic)]
    VerificationFailed { code: u32, message: String },
}

impl CryptoError {
    pub fn generic(message: impl Into<String>) -> Self {
        Self::Generic {
            code: CryptoErrorCode::Generic as u32,
            message: message.into(),
        }
    }

    pub fn invalid_algorithm(message: impl Into<String>) -> Self {
        Self::InvalidAlgorithm {
            code: CryptoErrorCode::InvalidAlgorithm as u32,
            message: message.into(),
        }
    }

    pub fn invalid_key_length(message: impl Into<String>) -> Self {
        Self::InvalidKeyLength {
            code: CryptoErrorCode::InvalidKeyLength as u32,
            message: message.into(),
        }
    }

    pub fn encryption_failed(message: impl Into<String>) -> Self {
        Self::EncryptionFailed {
            code: CryptoErrorCode::EncryptionFailed as u32,
            message: message.into(),
        }
    }

    pub fn decryption_failed(message: impl Into<String>) -> Self {
        Self::DecryptionFailed {
            code: CryptoErrorCode::DecryptionFailed as u32,
            message: message.into(),
        }
    }

    pub fn hash_failed(message: impl Into<String>) -> Self {
        Self::HashFailed {
            code: CryptoErrorCode::HashFailed as u32,
            message: message.into(),
        }
    }

    pub fn hmac_failed(message: impl Into<String>) -> Self {
        Self::HmacFailed {
            code: CryptoErrorCode::HmacFailed as u32,
            message: message.into(),
        }
    }

    pub fn key_generation_failed(message: impl Into<String>) -> Self {
        Self::KeyGenerationFailed {
            code: CryptoErrorCode::KeyGenerationFailed as u32,
            message: message.into(),
        }
    }

    pub fn key_derivation_failed(message: impl Into<String>) -> Self {
        Self::KeyDerivationFailed {
            code: CryptoErrorCode::KeyDerivationFailed as u32,
            message: message.into(),
        }
    }

    pub fn verification_failed(message: impl Into<String>) -> Self {
        Self::VerificationFailed {
            code: CryptoErrorCode::VerificationFailed as u32,
            message: message.into(),
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Encrypted data with ciphertext, IV, and authentication tag
#[weld_struct]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
    pub tag: Vec<u8>,
}

// ============================================================================
// Capability Checker
// ============================================================================

/// Capability checker trait for crypto operations
pub trait CryptoCapabilityChecker: Send + Sync {
    fn check_crypto(&self) -> Result<(), String>;
}

/// Default permissive checker
pub struct PermissiveChecker;

impl CryptoCapabilityChecker for PermissiveChecker {
    fn check_crypto(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Wrapper to store capability checker in OpState
pub struct CryptoCapabilities {
    pub checker: Arc<dyn CryptoCapabilityChecker>,
}

impl Default for CryptoCapabilities {
    fn default() -> Self {
        Self {
            checker: Arc::new(PermissiveChecker),
        }
    }
}

// ============================================================================
// Helper Functions (Internal implementations used by ops and tests)
// ============================================================================

/// Get the HMAC algorithm from string
fn get_hmac_algorithm(algorithm: &str) -> Result<hmac::Algorithm, CryptoError> {
    match algorithm.to_lowercase().as_str() {
        "sha256" | "sha-256" => Ok(hmac::HMAC_SHA256),
        "sha384" | "sha-384" => Ok(hmac::HMAC_SHA384),
        "sha512" | "sha-512" => Ok(hmac::HMAC_SHA512),
        _ => Err(CryptoError::invalid_algorithm(format!(
            "Unsupported HMAC algorithm: {}. Use sha256, sha384, or sha512",
            algorithm
        ))),
    }
}

/// Generate cryptographically secure random bytes (internal implementation)
fn random_bytes_impl(size: u32) -> Result<Vec<u8>, CryptoError> {
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; size as usize];
    rng.fill(&mut bytes)
        .map_err(|_| CryptoError::generic("Failed to generate random bytes"))?;
    Ok(bytes)
}

/// Generate a random UUID v4 (internal implementation)
fn random_uuid_impl() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Compute HMAC signature (internal implementation)
fn hmac_impl(algorithm: &str, key: &[u8], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let algo = get_hmac_algorithm(algorithm)?;
    let key = hmac::Key::new(algo, key);
    let tag = hmac::sign(&key, data);
    Ok(tag.as_ref().to_vec())
}

/// Compute hash (internal implementation)
fn compute_hash(algorithm: &str, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let result = match algorithm.to_lowercase().as_str() {
        "sha256" | "sha-256" => digest(&SHA256, data),
        "sha384" | "sha-384" => digest(&SHA384, data),
        "sha512" | "sha-512" => digest(&SHA512, data),
        _ => {
            return Err(CryptoError::invalid_algorithm(format!(
                "Unsupported hash algorithm: {}. Use sha256, sha384, or sha512",
                algorithm
            )))
        }
    };
    Ok(result.as_ref().to_vec())
}

/// Encrypt data using AES-256-GCM (internal implementation)
fn encrypt_impl(
    algorithm: &str,
    key: &[u8],
    data: &[u8],
    iv: Option<&[u8]>,
) -> Result<EncryptedData, CryptoError> {
    // Validate algorithm
    if !matches!(
        algorithm.to_lowercase().as_str(),
        "aes-256-gcm" | "aes256gcm" | "aes-gcm"
    ) {
        return Err(CryptoError::invalid_algorithm(format!(
            "Unsupported encryption algorithm: {}. Use aes-256-gcm",
            algorithm
        )));
    }

    // Validate key length (AES-256 requires 32 bytes)
    if key.len() != 32 {
        return Err(CryptoError::invalid_key_length(format!(
            "AES-256 requires 32-byte key, got {} bytes",
            key.len()
        )));
    }

    // Generate or use provided IV (12 bytes for GCM)
    let iv_bytes = match iv {
        Some(iv) => {
            if iv.len() != 12 {
                return Err(CryptoError::invalid_key_length(format!(
                    "AES-GCM requires 12-byte IV, got {} bytes",
                    iv.len()
                )));
            }
            iv.to_vec()
        }
        None => {
            let rng = SystemRandom::new();
            let mut generated_iv = vec![0u8; 12];
            rng.fill(&mut generated_iv)
                .map_err(|_| CryptoError::encryption_failed("Failed to generate IV"))?;
            generated_iv
        }
    };

    // Create key and encrypt
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| CryptoError::encryption_failed("Failed to create encryption key"))?;
    let sealing_key = LessSafeKey::new(unbound_key);

    let nonce = Nonce::try_assume_unique_for_key(&iv_bytes)
        .map_err(|_| CryptoError::encryption_failed("Invalid nonce"))?;

    let mut in_out = data.to_vec();
    let tag = sealing_key
        .seal_in_place_separate_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| CryptoError::encryption_failed("Encryption failed"))?;

    Ok(EncryptedData {
        ciphertext: in_out,
        iv: iv_bytes,
        tag: tag.as_ref().to_vec(),
    })
}

/// Decrypt data using AES-256-GCM (internal implementation)
fn decrypt_impl(algorithm: &str, key: &[u8], encrypted: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
    // Validate algorithm
    if !matches!(
        algorithm.to_lowercase().as_str(),
        "aes-256-gcm" | "aes256gcm" | "aes-gcm"
    ) {
        return Err(CryptoError::invalid_algorithm(format!(
            "Unsupported decryption algorithm: {}. Use aes-256-gcm",
            algorithm
        )));
    }

    // Validate key length
    if key.len() != 32 {
        return Err(CryptoError::invalid_key_length(format!(
            "AES-256 requires 32-byte key, got {} bytes",
            key.len()
        )));
    }

    // Create key
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .map_err(|_| CryptoError::decryption_failed("Failed to create decryption key"))?;
    let opening_key = LessSafeKey::new(unbound_key);

    let nonce = Nonce::try_assume_unique_for_key(&encrypted.iv)
        .map_err(|_| CryptoError::decryption_failed("Invalid nonce"))?;

    // Combine ciphertext and tag for ring's API
    let mut in_out = encrypted.ciphertext.clone();
    in_out.extend_from_slice(&encrypted.tag);

    let plaintext = opening_key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| CryptoError::decryption_failed("Decryption failed - invalid key or data"))?;

    Ok(plaintext.to_vec())
}

/// Generate a random encryption key (internal implementation)
fn generate_key_impl(algorithm: &str, length: Option<u32>) -> Result<Vec<u8>, CryptoError> {
    let key_length = match algorithm.to_lowercase().as_str() {
        "aes-128-gcm" | "aes128gcm" => 16,
        "aes-256-gcm" | "aes256gcm" | "aes-gcm" => 32,
        "hmac-sha256" | "hmac-sha384" | "hmac-sha512" => length.unwrap_or(32) as usize,
        _ => {
            return Err(CryptoError::invalid_algorithm(format!(
                "Unsupported algorithm for key generation: {}",
                algorithm
            )))
        }
    };

    let rng = SystemRandom::new();
    let mut key = vec![0u8; key_length];
    rng.fill(&mut key)
        .map_err(|_| CryptoError::key_generation_failed("Failed to generate random key"))?;

    Ok(key)
}

/// Derive a key from a password using PBKDF2 (internal implementation)
fn derive_key_impl(
    password: &str,
    salt: &[u8],
    iterations: u32,
    key_length: u32,
) -> Result<Vec<u8>, CryptoError> {
    if iterations == 0 {
        return Err(CryptoError::key_derivation_failed(
            "Iterations must be greater than 0",
        ));
    }

    if salt.len() < 8 {
        return Err(CryptoError::key_derivation_failed(
            "Salt should be at least 8 bytes",
        ));
    }

    let iterations = NonZeroU32::new(iterations)
        .ok_or_else(|| CryptoError::key_derivation_failed("Invalid iteration count"))?;

    let mut key = vec![0u8; key_length as usize];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iterations,
        salt,
        password.as_bytes(),
        &mut key,
    );

    Ok(key)
}

/// Verify an HMAC signature (internal implementation)
fn verify_impl(algorithm: &str, key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
    let algo = get_hmac_algorithm(algorithm)?;
    let key = hmac::Key::new(algo, key);

    match hmac::verify(&key, data, signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

// ============================================================================
// Operations (Deno ops that delegate to internal implementations)
// ============================================================================

/// Generate cryptographically secure random bytes
#[weld_op]
#[op2]
#[serde]
fn op_crypto_random_bytes(#[smi] size: u32) -> Result<Vec<u8>, CryptoError> {
    debug!(size = size, "crypto.random_bytes");
    random_bytes_impl(size)
}

/// Generate a random UUID v4
#[weld_op]
#[op2]
#[string]
fn op_crypto_random_uuid() -> String {
    debug!("crypto.random_uuid");
    random_uuid_impl()
}

/// Hash data using specified algorithm
#[weld_op]
#[op2]
#[serde]
fn op_crypto_hash(
    #[string] algorithm: String,
    #[serde] data: Vec<u8>,
) -> Result<Vec<u8>, CryptoError> {
    debug!(algorithm = %algorithm, len = data.len(), "crypto.hash");
    compute_hash(&algorithm, &data)
}

/// Hash data and return hex string
#[weld_op]
#[op2]
#[string]
fn op_crypto_hash_hex(
    #[string] algorithm: String,
    #[serde] data: Vec<u8>,
) -> Result<String, CryptoError> {
    debug!(algorithm = %algorithm, len = data.len(), "crypto.hash_hex");
    let hash = compute_hash(&algorithm, &data)?;
    Ok(hex::encode(hash))
}

/// Compute HMAC signature
#[weld_op]
#[op2]
#[serde]
fn op_crypto_hmac(
    #[string] algorithm: String,
    #[serde] key: Vec<u8>,
    #[serde] data: Vec<u8>,
) -> Result<Vec<u8>, CryptoError> {
    debug!(algorithm = %algorithm, key_len = key.len(), data_len = data.len(), "crypto.hmac");
    hmac_impl(&algorithm, &key, &data)
}

/// Encrypt data using AES-256-GCM
#[weld_op]
#[op2]
#[serde]
fn op_crypto_encrypt(
    #[string] algorithm: String,
    #[serde] key: Vec<u8>,
    #[serde] data: Vec<u8>,
    #[serde] iv: Option<Vec<u8>>,
) -> Result<EncryptedData, CryptoError> {
    debug!(algorithm = %algorithm, key_len = key.len(), data_len = data.len(), "crypto.encrypt");
    encrypt_impl(&algorithm, &key, &data, iv.as_deref())
}

/// Decrypt data using AES-256-GCM
#[weld_op]
#[op2]
#[serde]
fn op_crypto_decrypt(
    #[string] algorithm: String,
    #[serde] key: Vec<u8>,
    #[serde] encrypted: EncryptedData,
) -> Result<Vec<u8>, CryptoError> {
    debug!(algorithm = %algorithm, key_len = key.len(), "crypto.decrypt");
    decrypt_impl(&algorithm, &key, &encrypted)
}

/// Generate a random encryption key
#[weld_op]
#[op2]
#[serde]
fn op_crypto_generate_key(
    #[string] algorithm: String,
    #[smi] length: Option<u32>,
) -> Result<Vec<u8>, CryptoError> {
    debug!(algorithm = %algorithm, length = ?length, "crypto.generate_key");
    generate_key_impl(&algorithm, length)
}

/// Derive a key from a password using PBKDF2
#[weld_op]
#[op2]
#[serde]
fn op_crypto_derive_key(
    #[string] password: String,
    #[serde] salt: Vec<u8>,
    #[smi] iterations: u32,
    #[smi] key_length: u32,
) -> Result<Vec<u8>, CryptoError> {
    debug!(iterations = iterations, key_length = key_length, "crypto.derive_key");
    derive_key_impl(&password, &salt, iterations, key_length)
}

/// Verify an HMAC signature
#[weld_op]
#[op2]
fn op_crypto_verify(
    #[string] algorithm: String,
    #[serde] key: Vec<u8>,
    #[serde] data: Vec<u8>,
    #[serde] signature: Vec<u8>,
) -> Result<bool, CryptoError> {
    debug!(algorithm = %algorithm, "crypto.verify");
    verify_impl(&algorithm, &key, &data, &signature)
}

// ============================================================================
// State Initialization
// ============================================================================

/// Initialize crypto state in OpState
pub fn init_crypto_state(
    op_state: &mut OpState,
    capabilities: Option<Arc<dyn CryptoCapabilityChecker>>,
) {
    if let Some(caps) = capabilities {
        op_state.put(CryptoCapabilities { checker: caps });
    }
}

// ============================================================================
// Extension Registration
// ============================================================================

// Include generated extension! macro from build.rs
include!(concat!(env!("OUT_DIR"), "/extension.rs"));

pub fn crypto_extension() -> Extension {
    runtime_crypto::ext()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = CryptoError::invalid_algorithm("test");
        match err {
            CryptoError::InvalidAlgorithm { code, .. } => {
                assert_eq!(code, CryptoErrorCode::InvalidAlgorithm as u32);
            }
            _ => panic!("Wrong error type"),
        }
    }

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes_impl(32).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_random_uuid() {
        let uuid = random_uuid_impl();
        assert_eq!(uuid.len(), 36); // UUID v4 format
    }

    #[test]
    fn test_hash() {
        let data = b"hello world";
        let hash = compute_hash("sha256", data).unwrap();
        assert_eq!(hash.len(), 32); // SHA-256 produces 32 bytes
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key_impl("aes-256-gcm", None).unwrap();
        let data = b"secret message";

        let encrypted = encrypt_impl("aes-256-gcm", &key, data, None).unwrap();

        let decrypted = decrypt_impl("aes-256-gcm", &key, &encrypted).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_hmac_verify() {
        let key = b"secret key";
        let data = b"message to authenticate";

        let signature = hmac_impl("sha256", key, data).unwrap();

        let valid = verify_impl("sha256", key, data, &signature).unwrap();
        assert!(valid);

        // Wrong signature should fail
        let invalid = verify_impl("sha256", key, data, &vec![0u8; 32]).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_derive_key() {
        let password = "password123";
        let salt = b"saltsalt";
        let key = derive_key_impl(password, salt, 10000, 32).unwrap();
        assert_eq!(key.len(), 32);
    }
}
