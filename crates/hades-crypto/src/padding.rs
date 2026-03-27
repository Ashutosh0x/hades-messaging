//! Message padding to fixed bucket sizes.
//!
//! Prevents message-length analysis by padding all messages to one of
//! the predefined bucket sizes: 256, 1024, 4096, 16384 bytes.
//! The server only ever sees uniformly-sized ciphertexts.

use hades_common::constants::PADDING_BUCKETS;

/// Pad a message to the nearest bucket size.
///
/// Format: [message_len: u32 LE][message bytes][random padding]
///
/// The length prefix allows stripping padding after decryption.
pub fn pad_message(message: &[u8]) -> Vec<u8> {
    let total_needed = 4 + message.len(); // 4 bytes for length prefix
    let bucket = select_bucket(total_needed);

    let mut padded = Vec::with_capacity(bucket);

    // Length prefix (little-endian u32)
    padded.extend_from_slice(&(message.len() as u32).to_le_bytes());
    // Original message
    padded.extend_from_slice(message);
    // Random padding to fill bucket
    let padding_len = bucket - padded.len();
    let mut padding = vec![0u8; padding_len];
    getrandom::getrandom(&mut padding).expect("Failed to generate random padding");
    padded.extend_from_slice(&padding);

    padded
}

/// Remove padding and extract the original message.
pub fn unpad_message(padded: &[u8]) -> Option<Vec<u8>> {
    if padded.len() < 4 {
        return None;
    }

    let len = u32::from_le_bytes([padded[0], padded[1], padded[2], padded[3]]) as usize;

    if 4 + len > padded.len() {
        return None;
    }

    Some(padded[4..4 + len].to_vec())
}

/// Select the smallest bucket that fits the content.
fn select_bucket(size: usize) -> usize {
    for &bucket in PADDING_BUCKETS {
        if size <= bucket {
            return bucket;
        }
    }
    // If message exceeds largest bucket, round up to nearest multiple
    let largest = *PADDING_BUCKETS.last().unwrap();
    ((size + largest - 1) / largest) * largest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_roundtrip() {
        let msg = b"Hello, Hades!";
        let padded = pad_message(msg);
        assert_eq!(padded.len(), 256); // First bucket

        let unpadded = unpad_message(&padded).unwrap();
        assert_eq!(unpadded, msg);
    }

    #[test]
    fn test_padding_buckets() {
        // Small message → 256
        assert_eq!(pad_message(&[0u8; 10]).len(), 256);

        // 300 bytes total → 1024
        assert_eq!(pad_message(&[0u8; 300]).len(), 1024);

        // 2000 bytes → 4096
        assert_eq!(pad_message(&[0u8; 2000]).len(), 4096);

        // 5000 bytes → 16384
        assert_eq!(pad_message(&[0u8; 5000]).len(), 16384);
    }

    #[test]
    fn test_empty_message() {
        let padded = pad_message(b"");
        assert_eq!(padded.len(), 256);
        let unpadded = unpad_message(&padded).unwrap();
        assert!(unpadded.is_empty());
    }

    #[test]
    fn test_invalid_unpad() {
        assert!(unpad_message(&[]).is_none());
        assert!(unpad_message(&[0xFF, 0xFF, 0xFF, 0xFF]).is_none()); // length too large
    }
}
