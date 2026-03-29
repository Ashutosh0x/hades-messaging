use crate::error::CryptoError;
use crate::kdf;
use crate::aead;
use zeroize::Zeroize;

use hkdf::Hkdf;
use sha2::Sha256;

/// Derives SRTP keys from the Double Ratchet root key
/// This allows the call to inherit the security of the Double Ratchet session.
/// Returns: (audio_key, video_key, rtp_key, mix_key)
pub fn derive_srtp_keys(root_key: &[u8; 32]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>), CryptoError> {
    let hk = Hkdf::<Sha256>::new(None, root_key);

    let mut audio_key = [0u8; 32];
    hk.expand(b"HadesAudioKey", &mut audio_key)
        .map_err(|_| CryptoError::Kdf("Audio key derivation failed".to_string()))?;

    let mut video_key = [0u8; 32];
    hk.expand(b"HadesVideoKey", &mut video_key)
        .map_err(|_| CryptoError::Kdf("Video key derivation failed".to_string()))?;

    let mut rtp_key = [0u8; 32];
    hk.expand(b"HadesWebRTC", &mut rtp_key)
        .map_err(|_| CryptoError::Kdf("RTP key derivation failed".to_string()))?;

    let mut mix_key = [0u8; 32];
    hk.expand(b"MixKey", &mut mix_key)
        .map_err(|_| CryptoError::Kdf("Mix key derivation failed".to_string()))?;

    Ok((
        audio_key.to_vec(),
        video_key.to_vec(),
        rtp_key.to_vec(),
        mix_key.to_vec(),
    ))
}

/// Encrypts audio frames for Signal-style "ZK encrypted" calls
pub fn encrypt_audio_frame(
    key: &[u8; 32],
    sequence_number: u32,
    sample_rate: u32,
    payload: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    // 1. Header (Sequence # || Sample Rate || Padding || Payload)
    let mut frame = Vec::with_capacity(payload.len() + 32); 
    frame.extend_from_slice(&sequence_number.to_be_bytes());
    frame.extend_from_slice(&sample_rate.to_be_bytes());

    // 2. Padding (PKCS#7 inspired padding to fixed buckets e.g., 1400 up to max MTU)
    // To simplify for this phase, we add a fixed random pad
    let mut padding = vec![0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut padding);
    frame.extend_from_slice(&padding);

    // 3. Append payload
    frame.extend_from_slice(payload);

    // 4. Encrypt with AEAD (ChaCha20-Poly1305)
    let mut nonce = [0u8; 12];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);

    let ciphertext = aead::encrypt(key, &nonce, &frame, &[])?;

    let mut output = Vec::with_capacity(12 + ciphertext.len());
    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Decrypt an audio frame
pub fn decrypt_audio_frame(
    key: &[u8; 32],
    encrypted_payload: &[u8]
) -> Result<Vec<u8>, CryptoError> {
    if encrypted_payload.len() < 12 {
        return Err(CryptoError::InvalidLength);
    }

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&encrypted_payload[0..12]);
    let ciphertext = &encrypted_payload[12..];

    let plaintext = aead::decrypt(key, &nonce, ciphertext, &[])?;

    // Minimal check on the structure: 4 bytes sec num + 4 bytes sample + 32 bytes pad
    if plaintext.len() < 40 {
        return Err(CryptoError::InvalidLength);
    }

    // Skip the 40 bytes of header/padding
    Ok(plaintext[40..].to_vec())
}
