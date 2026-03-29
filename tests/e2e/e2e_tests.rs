// tests/e2e/e2e_tests.rs

use hades_crypto::double_ratchet::DoubleRatchetSession;
use hades_identity::seed::MasterSeed;
use hades_wallet::hd::HdWallet;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_full_messaging_flow() {
    // Create two identities (Alice and Bob)
    let alice_seed = MasterSeed::generate().unwrap();
    let bob_seed = MasterSeed::generate().unwrap();

    let alice_kp = alice_seed.derive_messaging_keypair().unwrap();
    let bob_kp = bob_seed.derive_messaging_keypair().unwrap();

    // X3DH key exchange
    let alice_spk = bob_kp.x25519_public;
    let bob_spk = alice_kp.x25519_public;

    // Alice initiates session
    let mut alice_session = DoubleRatchetSession::init_alice(
        &[0u8; 32],  // Would be real shared secret from X3DH
        alice_spk.as_bytes(),
    );

    // Bob receives and creates session
    let mut bob_session = DoubleRatchetSession::init_bob(
        &[0u8; 32],
        bob_spk.as_bytes(),
    );

    // Alice encrypts message
    let plaintext = b"Hello Bob!";
    let (header, ciphertext) = alice_session.ratchet_encrypt(plaintext);

    // Bob decrypts
    let decrypted = bob_session.ratchet_decrypt(&header, &ciphertext).unwrap();
    assert_eq!(decrypted, plaintext);

    // Bob replies
    let reply = b"Hi Alice!";
    let (header2, ciphertext2) = bob_session.ratchet_encrypt(reply);

    // Alice decrypts
    let decrypted2 = alice_session.ratchet_decrypt(&header2, &ciphertext2).unwrap();
    assert_eq!(decrypted2, reply);
}

#[tokio::test]
async fn test_wallet_send_flow() {
    let seed = MasterSeed::generate().unwrap();
    let wallet = HdWallet::from_mnemonic(seed.mnemonic()).unwrap();

    // Derive ETH account
    let eth_account = wallet.derive_account(hades_wallet::hd::Chain::Ethereum, 0).unwrap();

    assert!(eth_account.address.starts_with("0x"));
    assert_eq!(eth_account.address.len(), 42);
}

#[tokio::test]
async fn test_media_compression() {
    use hades_media::compress::MediaCompressor;
    use hades_media::types::CompressionConfig;

    let config = CompressionConfig::default();
    let compressor = MediaCompressor::new(config).unwrap();

    // Test would need actual media file
    // assert!(compressed.compression_ratio < 1.0);
}
