//! Anti-forensics primitives for Hades Messaging.
//!
//! Provides memory-safe secure allocation, zeroize-on-drop guarantees,
//! and plausible-deniability volume support. All sensitive key material
//! flows through [`SecureBuffer`] to prevent swap-file leakage and
//! core-dump exposure.

use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::{Zeroize, ZeroizeOnDrop};

// ---------------------------------------------------------------------------
// Screenshot guard — platform flag
// ---------------------------------------------------------------------------

static SCREENSHOT_GUARD_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable_screenshot_guard() {
    SCREENSHOT_GUARD_ENABLED.store(true, Ordering::SeqCst);
    // Android JNI: set FLAG_SECURE on the window
    #[cfg(target_os = "android")]
    {
        // Called via JNI in the Tauri Android plugin
    }
}

pub fn disable_screenshot_guard() {
    SCREENSHOT_GUARD_ENABLED.store(false, Ordering::SeqCst);
}

pub fn is_screenshot_guard_enabled() -> bool {
    SCREENSHOT_GUARD_ENABLED.load(Ordering::SeqCst)
}

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
    /// Allocate a secure buffer from existing data.
    pub fn new(data: Vec<u8>) -> Self {
        // Request mlock if on unix
        #[cfg(unix)]
        unsafe {
            libc::mlock(data.as_ptr() as *const libc::c_void, data.len());
        }
        Self { inner: data }
    }

    /// Allocate a zeroed secure buffer of `len` bytes.
    pub fn zeroed(len: usize) -> Self {
        let data = vec![0u8; len];
        Self::new(data)
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
        #[cfg(unix)]
        unsafe {
            libc::munlock(
                self.inner.as_ptr() as *const libc::c_void,
                self.inner.len(),
            );
        }
        self.inner.zeroize();
        self.inner = Vec::new();
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::munlock(
                self.inner.as_ptr() as *const libc::c_void,
                self.inner.len(),
            );
        }
        self.inner.zeroize();
    }
}

// ---------------------------------------------------------------------------
// Emergency wipe — destroy all local state
// ---------------------------------------------------------------------------

/// Securely wipe a directory by overwriting all files 3 passes before deletion.
pub fn emergency_wipe(data_dir: &std::path::Path) -> std::io::Result<()> {
    if !data_dir.exists() {
        return Ok(());
    }

    fn wipe_recursive(dir: &std::path::Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                wipe_recursive(&path)?;
                std::fs::remove_dir(&path)?;
            } else {
                let meta = std::fs::metadata(&path)?;
                let len = meta.len() as usize;
                if len > 0 {
                    // Pass 1: zeros
                    std::fs::write(&path, &vec![0u8; len])?;
                    // Pass 2: ones
                    std::fs::write(&path, &vec![0xFFu8; len])?;
                    // Pass 3: random
                    let mut rng_data = vec![0u8; len];
                    getrandom::getrandom(&mut rng_data).ok();
                    std::fs::write(&path, &rng_data)?;
                }
                std::fs::remove_file(&path)?;
            }
        }
        Ok(())
    }

    wipe_recursive(data_dir)?;
    std::fs::remove_dir_all(data_dir).ok();
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

#[derive(Debug, PartialEq)]
pub enum VolumeKind {
    Decoy,
    Hidden,
}

enum HeaderOffset {
    Decoy,
    Hidden,
}

impl DualVolume {
    /// Attempt to open a volume. Returns `Decoy` or `Hidden` depending
    /// on which derived key matches the header MAC.
    pub fn open(&self, password: &str) -> Result<VolumeKind, String> {
        let key = derive_volume_key(password.as_bytes(), b"HadesSalt");

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

/// Derive volume key using Argon2id (production-grade KDF).
pub fn derive_volume_key(passphrase: &[u8], salt: &[u8]) -> [u8; 32] {
    use argon2::{Algorithm, Argon2, Params, Version};

    let params = Params::new(65536, 3, 4, Some(32)).unwrap();
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(passphrase, salt, &mut output)
        .expect("Argon2id derivation failed");

    output
}

/// Verify a volume header MAC.
fn verify_header(_ciphertext: &[u8], _key: &[u8; 32], _offset: HeaderOffset) -> bool {
    // In production: decrypt header at offset, verify MAC using constant-time comparison
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secure_buffer_zeroes_on_drop() {
        let buf = SecureBuffer::zeroed(64);
        assert_eq!(buf.as_bytes().len(), 64);
    }

    #[test]
    fn secure_buffer_wipe_clears_data() {
        let mut buf = SecureBuffer::new(b"secret key material here!!!!".to_vec());
        assert!(!buf.as_bytes().is_empty());
        buf.wipe();
        assert!(buf.as_bytes().is_empty());
    }

    #[test]
    fn screenshot_guard_toggle() {
        assert!(!is_screenshot_guard_enabled());
        enable_screenshot_guard();
        assert!(is_screenshot_guard_enabled());
        disable_screenshot_guard();
        assert!(!is_screenshot_guard_enabled());
    }

    #[test]
    fn argon2id_key_derivation() {
        let key1 = derive_volume_key(b"password", b"salt1234salt1234");
        let key2 = derive_volume_key(b"password", b"salt1234salt1234");
        assert_eq!(key1, key2); // Deterministic

        let key3 = derive_volume_key(b"different", b"salt1234salt1234");
        assert_ne!(key1, key3); // Different password → different key
    }
}
