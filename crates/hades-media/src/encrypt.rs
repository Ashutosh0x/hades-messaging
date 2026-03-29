use crate::types::{MediaAttachment, MediaError, MediaResult, MediaType};
use hades_crypto::aead;
use rand::RngCore;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct MediaEncryptor {
    file_key: [u8; 32],
    nonce: [u8; 12],
}

impl MediaEncryptor {
    pub fn new() -> Self {
        let mut file_key = [0u8; 32];
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut file_key);
        rand::thread_rng().fill_bytes(&mut nonce);

        Self { file_key, nonce }
    }

    /// Encrypt media and return attachment metadata
    pub fn encrypt(&self, data: &[u8], media_type: MediaType, filename: &str) -> MediaResult<MediaAttachment> {
        // Encrypt content
        let encrypted = aead::encrypt(&self.file_key, &self.nonce, data)?;

        // Compute hash of encrypted data (for integrity verification)
        let hash = Sha256::digest(&encrypted);

        // Build attachment metadata
        let attachment = MediaAttachment {
            id: Uuid::new_v4().to_string(),
            media_type,
            filename: filename.to_string(),
            mime_type: self.detect_mime(filename),
            size_bytes: data.len() as u64,
            compressed_size_bytes: None,
            encrypted_hash: hex::encode(hash),
            thumbnail: None,
            width: None,
            height: None,
            duration_secs: None,
            caption: None,
            is_compressed: false,
            compression_quality: None,
        };

        Ok(attachment)
    }

    /// Encrypt with compression metadata
    pub fn encrypt_with_compression(
        &self,
        compressed_data: &[u8],
        original_size: u64,
        media_type: MediaType,
        filename: &str,
        width: Option<u32>,
        height: Option<u32>,
        duration_secs: Option<f64>,
        thumbnail: Option<Vec<u8>>,
        quality: Option<u8>,
    ) -> MediaResult<(Vec<u8>, MediaAttachment)> {
        // Encrypt content
        let encrypted = aead::encrypt(&self.file_key, &self.nonce, compressed_data)?;

        // Compute hash
        let hash = Sha256::digest(&encrypted);

        let attachment = MediaAttachment {
            id: Uuid::new_v4().to_string(),
            media_type,
            filename: filename.to_string(),
            mime_type: self.detect_mime(filename),
            size_bytes: original_size,
            compressed_size_bytes: Some(compressed_data.len() as u64),
            encrypted_hash: hex::encode(hash),
            thumbnail,
            width,
            height,
            duration_secs,
            caption: None,
            is_compressed: true,
            compression_quality: quality,
        };

        Ok((encrypted, attachment))
    }

    /// Decrypt media content
    pub fn decrypt(&self, encrypted_data: &[u8], expected_hash: &str) -> MediaResult<Vec<u8>> {
        // Verify hash before decryption
        let computed_hash = Sha256::digest(encrypted_data);
        if hex::encode(computed_hash) != expected_hash {
            return Err(MediaError::Encryption(
                hades_crypto::error::CryptoError::DecryptionFailed
            ));
        }

        // Decrypt
        let decrypted = aead::decrypt(&self.file_key, &self.nonce, encrypted_data)?;
        Ok(decrypted)
    }

    /// Get the file encryption key (to be encrypted with ratchet and sent separately)
    pub fn get_file_key(&self) -> [u8; 32] {
        self.file_key
    }

    /// Get the nonce
    pub fn get_nonce(&self) -> [u8; 12] {
        self.nonce
    }

    fn detect_mime(&self, filename: &str) -> String {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "mov" => "video/quicktime",
            "mp3" => "audio/mpeg",
            "opus" => "audio/opus",
            "ogg" => "audio/ogg",
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }.to_string()
    }
}

impl Default for MediaEncryptor {
    fn default() -> Self {
        Self::new()
    }
}
