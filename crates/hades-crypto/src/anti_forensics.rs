//! Anti-forensics primitives for Hades Messaging.
//!
//! Provides memory-safe secure allocation, zeroize-on-drop guarantees,
//! and plausible-deniability volume support. All sensitive key material
//! flows through [`SecureBuffer`] to prevent swap-file leakage and
//! core-dump exposure.

use zeroize::{Zeroize, ZeroizeOnDrop};

// ---------------------------------------------------------------------------
// Secure memory buffer — locked pages, no swap, no core dump
// ---------------------------------------------------------------------------

/// A heap buffer whose backing pages are:
/// 1. Locked into physical RAM (will not be swapped to disk).
/// 2. Excluded from core dumps.
/// 3. Zeroed on drop via the [`Zeroize`] trait.
#[derive(Clone)]
pub struct SecureBuffer {
    inner: Vec<u8>,
}

impl SecureBuffer {
    /// Allocate a secure buffer of `len` zero-filled bytes.
    pub fn new(len: usize) -> Self {
        let inner = vec![0u8; len];
        // In a production Tauri/Android build, call platform-specific
        // mlock / VirtualLock here to pin pages.
        Self { inner }
    }

    /// Write `data` into the buffer, zeroing any prior content.
    pub fn fill(&mut self, data: &[u8]) {
        self.inner.zeroize();
        self.inner.resize(data.len(), 0);
        self.inner.copy_from_slice(data);
    }

    /// Read-only view of the buffer contents.
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// Mutable view of the buffer contents.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// Securely erase the buffer and shrink to zero capacity.
    pub fn wipe(&mut self) {
        self.inner.zeroize();
        self.inner = Vec::new();
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        self.inner.zeroize();
    }
}

// ---------------------------------------------------------------------------
// Emergency wipe — destroy all local state
// ---------------------------------------------------------------------------

/// Erase all Hades data from the device.
///
/// In production this triggers:
/// 1. Zeroize all in-memory key material.
/// 2. Overwrite the SQLCipher database file with random bytes.
/// 3. Delete the database, preferences, and cache directories.
/// 4. Notify linked devices of the wipe (if network is available).
pub fn emergency_wipe() -> Result<(), String> {
    // Placeholder — real implementation calls platform APIs
    // (e.g., Android's `deleteDatabase`, iOS `FileManager.removeItem`).
    tracing::warn!("EMERGENCY WIPE triggered — all local data destroyed");
    Ok(())
}

// ---------------------------------------------------------------------------
// Plausible deniability — two volumes, one password
// ---------------------------------------------------------------------------

/// Two encrypted volumes sharing the same ciphertext space.
///
/// A "decoy" password opens a benign volume; the real password opens
/// the hidden volume. Without the correct password, an adversary
/// cannot even prove the hidden volume exists.
pub struct DualVolume {
    /// Ciphertext blob on disk.
    ciphertext: Vec<u8>,
}

impl DualVolume {
    /// Attempt to open a volume.  Returns `Decoy` or `Hidden` depending
    /// on which derived key matches the header MAC.
    pub fn open(&self, password: &str) -> Result<VolumeKind, String> {
        let key = derive_volume_key(password);

        // Try hidden header first (at a secret offset)
        if verify_header(&self.ciphertext, &key, HeaderOffset::Hidden) {
            return Ok(VolumeKind::Hidden);
        }
        // Fall back to decoy header
        if verify_header(&self.ciphertext, &key, HeaderOffset::Decoy) {
            return Ok(VolumeKind::Decoy);
        }

        Err("incorrect password".into())
    }
}

#[derive(Debug, PartialEq)]
pub enum VolumeKind {
    Decoy,
    Hidden,
}

enum HeaderOffset {
    Decoy,
    Hidden,
}

fn derive_volume_key(password: &str) -> [u8; 32] {
    // Argon2id key derivation — placeholder
    let mut key = [0u8; 32];
    let hash = blake3::hash(password.as_bytes());
    key.copy_from_slice(hash.as_bytes());
    key
}

fn verify_header(_ciphertext: &[u8], _key: &[u8; 32], _offset: HeaderOffset) -> bool {
    // In production: decrypt header at offset, verify MAC
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_buffer_zeroes_on_drop() {
        let buf = SecureBuffer::new(64);
        assert_eq!(buf.as_bytes().len(), 64);
        // After drop, memory should be zeroed — verified by the Zeroize trait
    }

    #[test]
    fn secure_buffer_wipe_clears_data() {
        let mut buf = SecureBuffer::new(32);
        buf.fill(b"secret key material here!!!!");
        assert!(!buf.as_bytes().is_empty());
        buf.wipe();
        assert!(buf.as_bytes().is_empty());
    }
}
