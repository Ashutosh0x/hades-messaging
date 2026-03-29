use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    pub id: String,
    pub media_type: MediaType,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub compressed_size_bytes: Option<u64>,
    pub encrypted_hash: String,
    pub thumbnail: Option<Vec<u8>>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f64>,
    pub caption: Option<String>,
    pub is_compressed: bool,
    pub compression_quality: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub max_image_size_kb: u32,
    pub max_video_size_mb: u32,
    pub max_video_duration_secs: u32,
    pub max_image_dimension: u32,
    pub image_quality: u8,
    pub video_bitrate_kbps: u32,
    pub audio_bitrate_kbps: u32,
    pub prefer_webp: bool,
    pub prefer_h264: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            max_image_size_kb: 1024,        // 1MB
            max_video_size_mb: 16,          // 16MB
            max_video_duration_secs: 180,   // 3 minutes
            max_image_dimension: 1920,      // 1080p max
            image_quality: 80,
            video_bitrate_kbps: 2500,
            audio_bitrate_kbps: 128,
            prefer_webp: true,
            prefer_h264: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("FFmpeg not installed")]
    FFmpegNotInstalled,

    #[error("FFmpeg error: {0}")]
    FFmpeg(String),

    #[error("FFprobe error: {0}")]
    FFprobe(String),

    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Encryption failed: {0}")]
    Encryption(#[from] hades_crypto::error::CryptoError),

    #[error("File too large: {size} bytes (max {max})")]
    FileTooLarge { size: u64, max: u64 },

    #[error("Invalid media type: {0}")]
    InvalidMediaType(String),

    #[error("Duration exceeded: {duration}s (max {max}s)")]
    DurationExceeded { duration: f64, max: u32 },
}

pub type MediaResult<T> = Result<T, MediaError>;
