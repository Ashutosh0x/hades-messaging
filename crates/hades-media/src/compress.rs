use crate::types::*;
use ffmpeg_sidecar::command::{ffmpeg_is_installed, FFmpegCommand};
use image::{DynamicImage, ImageFormat, ImageReader};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub struct MediaCompressor {
    config: CompressionConfig,
    temp_dir: PathBuf,
}

impl MediaCompressor {
    pub fn new(config: CompressionConfig) -> MediaResult<Self> {
        let temp_dir = std::env::temp_dir().join("hades_media");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self { config, temp_dir })
    }

    /// Compress any media file based on type
    pub fn compress(&self, input_path: &Path) -> MediaResult<CompressedMedia> {
        let mime_type = self.detect_mime_type(input_path)?;
        let media_type = self.mime_to_media_type(&mime_type)?;

        match media_type {
            MediaType::Image => self.compress_image(input_path),
            MediaType::Video => self.compress_video(input_path),
            MediaType::Audio => self.compress_audio(input_path),
            MediaType::Document => self.compress_document(input_path),
        }
    }

    /// Compress image with quality/size constraints
    pub fn compress_image(&self, input_path: &Path) -> MediaResult<CompressedMedia> {
        // Load image
        let img = ImageReader::open(input_path)?
            .with_guessed_format()?
            .decode()?;

        let (orig_width, orig_height) = img.dimensions();
        let orig_size = std::fs::metadata(input_path)?.len();

        // Resize if too large
        let img = self.resize_if_needed(img);

        // Compress with iterative quality reduction
        let (compressed_data, final_format, quality) =
            self.compress_image_iterative(&img)?;

        let compressed_size = compressed_data.len() as u64;

        // Generate thumbnail
        let thumbnail = self.generate_thumbnail(&img)?;

        Ok(CompressedMedia {
            data: compressed_data,
            original_size: orig_size,
            compressed_size,
            compression_ratio: if orig_size > 0 {
                compressed_size as f64 / orig_size as f64
            } else {
                1.0
            },
            mime_type: format!("image/{}", final_format.to_lowercase()),
            width: Some(img.width()),
            height: Some(img.height()),
            duration_secs: None,
            thumbnail: Some(thumbnail),
            quality: Some(quality),
            format: final_format,
        })
    }

    /// Compress video with FFmpeg
    pub fn compress_video(&self, input_path: &Path) -> MediaResult<CompressedMedia> {
        if !ffmpeg_is_installed() {
            return Err(MediaError::FFmpegNotInstalled);
        }

        // Get original duration
        let duration = Self::get_media_duration(input_path)?;

        // Check duration limit
        if duration > self.config.max_video_duration_secs as f64 {
            return Err(MediaError::DurationExceeded {
                duration,
                max: self.config.max_video_duration_secs,
            });
        }

        let orig_size = std::fs::metadata(input_path)?.len();

        // Create temp output file
        let output_file = NamedTempFile::new_in(&self.temp_dir)?;
        let output_path = output_file.path();

        // Build FFmpeg command
        let mut cmd = FFmpegCommand::new();
        cmd.input(input_path);

        // Video codec
        if self.config.prefer_h264 {
            cmd.args(&["-c:v", "libx264", "-preset", "medium", "-crf", "23"]);
        } else {
            cmd.args(&["-c:v", "libvpx-vp9", "-crf", "30", "-b:v", "0"]);
        }

        // Audio codec
        cmd.args(&["-c:a", "aac", "-b:a", &format!("{}k", self.config.audio_bitrate_kbps)]);

        // Resolution limit
        cmd.args(&[
            "-vf",
            &format!(
                "scale='min({},iw)':'min({},ih)':force_original_aspect_ratio=decrease",
                self.config.max_image_dimension,
                self.config.max_image_dimension / 2
            ),
        ]);

        // Duration limit (trim if needed)
        cmd.args(&["-t", &self.config.max_video_duration_secs.to_string()]);

        // Output settings
        cmd.args(&[
            "-movflags", "+faststart",
            "-maxrate", &format!("{}k", self.config.video_bitrate_kbps * 2),
            "-bufsize", &format!("{}k", self.config.video_bitrate_kbps * 4),
        ]);

        cmd.arg(output_path);

        // Execute
        let output = cmd.output()?;
        if !output.status.success() {
            return Err(MediaError::FFmpeg(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Get compressed size
        let compressed_size = std::fs::metadata(output_path)?.len();
        let compressed_data = std::fs::read(output_path)?;

        // Generate thumbnail from first frame
        let thumbnail = self.extract_video_thumbnail(input_path)?;

        Ok(CompressedMedia {
            data: compressed_data,
            original_size: orig_size,
            compressed_size,
            compression_ratio: if orig_size > 0 {
                compressed_size as f64 / orig_size as f64
            } else {
                1.0
            },
            mime_type: "video/mp4".to_string(),
            width: None,  // Would need ffprobe to get exact
            height: None,
            duration_secs: Some(duration),
            thumbnail,
            quality: None,
            format: "mp4".to_string(),
        })
    }

    /// Compress audio
    pub fn compress_audio(&self, input_path: &Path) -> MediaResult<CompressedMedia> {
        if !ffmpeg_is_installed() {
            return Err(MediaError::FFmpegNotInstalled);
        }

        let orig_size = std::fs::metadata(input_path)?.len();
        let duration = Self::get_media_duration(input_path)?;

        let output_file = NamedTempFile::new_in(&self.temp_dir)?;
        let output_path = output_file.path();

        let output = FFmpegCommand::new()
            .input(input_path)
            .args(&[
                "-c:a", "libopus",
                "-b:a", &format!("{}k", self.config.audio_bitrate_kbps),
                "-vbr", "on",
            ])
            .arg(output_path)
            .output()?;

        if !output.status.success() {
            return Err(MediaError::FFmpeg(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let compressed_size = std::fs::metadata(output_path)?.len();
        let compressed_data = std::fs::read(output_path)?;

        Ok(CompressedMedia {
            data: compressed_data,
            original_size: orig_size,
            compressed_size,
            compression_ratio: if orig_size > 0 {
                compressed_size as f64 / orig_size as f64
            } else {
                1.0
            },
            mime_type: "audio/opus".to_string(),
            width: None,
            height: None,
            duration_secs: Some(duration),
            thumbnail: None,
            quality: None,
            format: "opus".to_string(),
        })
    }

    /// Documents: no compression, just validate size
    pub fn compress_document(&self, input_path: &Path) -> MediaResult<CompressedMedia> {
        let orig_size = std::fs::metadata(input_path)?.len();
        let max_size = 100 * 1024 * 1024; // 100MB max for documents

        if orig_size > max_size {
            return Err(MediaError::FileTooLarge {
                size: orig_size,
                max: max_size as u64,
            });
        }

        let compressed_data = std::fs::read(input_path)?;

        Ok(CompressedMedia {
            data: compressed_data,
            original_size: orig_size,
            compressed_size: orig_size,
            compression_ratio: 1.0,
            mime_type: self.detect_mime_type(input_path)?,
            width: None,
            height: None,
            duration_secs: None,
            thumbnail: None,
            quality: None,
            format: "original".to_string(),
        })
    }

    fn resize_if_needed(&self, img: DynamicImage) -> DynamicImage {
        let (width, height) = img.dimensions();
        let max_dim = self.config.max_image_dimension;

        if width > max_dim || height > max_dim {
            let scale = (max_dim as f32) / (width.max(height) as f32);
            let new_width = (width as f32 * scale) as u32;
            let new_height = (height as f32 * scale) as u32;
            img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
        } else {
            img
        }
    }

    fn compress_image_iterative(
        &self,
        img: &DynamicImage,
    ) -> MediaResult<(Vec<u8>, String, u8)> {
        let max_size = (self.config.max_image_size_kb * 1024) as usize;
        let mut quality = self.config.image_quality;
        let format = if self.config.prefer_webp {
            ImageFormat::WebP
        } else {
            ImageFormat::Jpeg
        };
        let format_str = if self.config.prefer_webp { "webp" } else { "jpeg" };

        loop {
            let mut buffer = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buffer), format)?;

            if buffer.len() <= max_size || quality <= 10 {
                return Ok((buffer, format_str.to_string(), quality));
            }

            quality = quality.saturating_sub(5);
        }
    }

    fn generate_thumbnail(&self, img: &DynamicImage) -> MediaResult<Vec<u8>> {
        let thumb = img.resize(200, 200, image::imageops::FilterType::Lanczos3);
        let mut buffer = Vec::new();
        thumb.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::WebP)?;
        Ok(buffer)
    }

    fn extract_video_thumbnail(&self, video_path: &Path) -> MediaResult<Option<Vec<u8>>> {
        if !ffmpeg_is_installed() {
            return Ok(None);
        }

        let output_file = NamedTempFile::new_in(&self.temp_dir)?;
        let output_path = output_file.path();

        let output = FFmpegCommand::new()
            .input(video_path)
            .args(&[
                "-ss", "00:00:01",  // First second
                "-vframes", "1",
                "-vf", "scale=200:-1",
                "-f", "webp",
            ])
            .arg(output_path)
            .output()?;

        if output.status.success() {
            Ok(Some(std::fs::read(output_path)?))
        } else {
            Ok(None)
        }
    }

    fn detect_mime_type(&self, path: &Path) -> MediaResult<String> {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        Ok(match ext.as_str() {
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
            "xls" | "xlsx" => "application/vnd.ms-excel",
            "ppt" | "pptx" => "application/vnd.ms-powerpoint",
            "zip" => "application/zip",
            _ => "application/octet-stream",
        }.to_string())
    }

    fn mime_to_media_type(&self, mime: &str) -> MediaResult<MediaType> {
        if mime.starts_with("image/") {
            Ok(MediaType::Image)
        } else if mime.starts_with("video/") {
            Ok(MediaType::Video)
        } else if mime.starts_with("audio/") {
            Ok(MediaType::Audio)
        } else {
            Ok(MediaType::Document)
        }
    }

    fn get_media_duration(path: &Path) -> MediaResult<f64> {
        use ffmpeg_sidecar::command::FFprobeCommand;

        let output = FFprobeCommand::new()
            .args(&[
                "-v", "error",
                "-show_entries", "format=duration",
                "-of", "default=noprint_wrappers=1:nokey=1",
            ])
            .arg(path)
            .output()?;

        if !output.status.success() {
            return Err(MediaError::FFprobe(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let duration = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<f64>()
            .map_err(|e| MediaError::FFprobe(e.to_string()))?;

        Ok(duration)
    }
}

#[derive(Debug)]
pub struct CompressedMedia {
    pub data: Vec<u8>,
    pub original_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
    pub mime_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration_secs: Option<f64>,
    pub thumbnail: Option<Vec<u8>>,
    pub quality: Option<u8>,
    pub format: String,
}
