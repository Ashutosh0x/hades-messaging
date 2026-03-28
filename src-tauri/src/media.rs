use crate::error::{AppError, AppResult};
use std::path::Path;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaAttachment {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub encrypted_key: Vec<u8>,
    pub encrypted_hash: String,
    pub thumbnail: Option<Vec<u8>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f64>,
}

/// Encrypt a file with a random key for E2EE file sharing
pub fn encrypt_file(path: &Path) -> AppResult<(Vec<u8>, MediaAttachment)> {
    let file_data = std::fs::read(path)
        .map_err(|e| AppError::Internal(format!("Read file failed: {}", e)))?;

    if file_data.len() as u64 > MAX_FILE_SIZE {
        return Err(AppError::Internal(format!("File too large: {} bytes", file_data.len())));
    }

    let mut file_key = [0u8; 32];
    let mut nonce = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut file_key);
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);

    // Encrypt with ChaCha20-Poly1305
    use chacha20poly1305::{aead::Aead, KeyInit, ChaCha20Poly1305};
    let cipher = ChaCha20Poly1305::new_from_slice(&file_key)
        .map_err(|e| AppError::Crypto(format!("Cipher init failed: {}", e)))?;
    let encrypted = cipher.encrypt(&nonce.into(), file_data.as_ref())
        .map_err(|e| AppError::Crypto(format!("Encrypt failed: {}", e)))?;

    let mut output = Vec::with_capacity(12 + encrypted.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&encrypted);

    let hash = blake3::hash(&output);

    let filename = path.file_name()
        .and_then(|n| n.to_str()).unwrap_or("file").to_string();
    let mime_type = mime_from_extension(&filename);

    let attachment = MediaAttachment {
        id: uuid_v4(), filename, mime_type, size: file_data.len() as u64,
        encrypted_key: file_key.to_vec(), encrypted_hash: hash.to_hex().to_string(),
        thumbnail: None, width: None, height: None, duration_secs: None,
    };

    zeroize::Zeroize::zeroize(&mut file_key);
    Ok((output, attachment))
}

/// Decrypt a file attachment
pub fn decrypt_file(encrypted_data: &[u8], file_key: &[u8; 32]) -> AppResult<Vec<u8>> {
    if encrypted_data.len() < 12 {
        return Err(AppError::Crypto("Encrypted data too short".into()));
    }
    let nonce: [u8; 12] = encrypted_data[..12].try_into().unwrap();
    let ciphertext = &encrypted_data[12..];

    use chacha20poly1305::{aead::Aead, KeyInit, ChaCha20Poly1305};
    let cipher = ChaCha20Poly1305::new_from_slice(file_key)
        .map_err(|e| AppError::Crypto(format!("Cipher init failed: {}", e)))?;
    cipher.decrypt(&nonce.into(), ciphertext)
        .map_err(|e| AppError::Crypto(format!("Decrypt failed: {}", e)))
}

fn mime_from_extension(filename: &str) -> String {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg", "png" => "image/png",
        "gif" => "image/gif", "webp" => "image/webp",
        "mp4" => "video/mp4", "webm" => "video/webm",
        "mp3" => "audio/mpeg", "ogg" => "audio/ogg",
        "pdf" => "application/pdf", "zip" => "application/zip",
        _ => "application/octet-stream",
    }.to_string()
}

fn uuid_v4() -> String {
    let mut b = [0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut b);
    b[6] = (b[6] & 0x0F) | 0x40;
    b[8] = (b[8] & 0x3F) | 0x80;
    format!("{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        b[0],b[1],b[2],b[3],b[4],b[5],b[6],b[7],b[8],b[9],b[10],b[11],b[12],b[13],b[14],b[15])
}
