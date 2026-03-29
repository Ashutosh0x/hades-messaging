# Hades Messaging -- Cryptographic Protocol Specification

<p>
  <a href="https://play.google.com/store/apps/details?id=im.hades.messaging"><img src="https://play.google.com/intl/en_us/badges/static/images/badges/en_badge_web_generic.png" height="50"></a>
  <a href="https://apps.apple.com/app/hades-messaging/idYOUR_APP_ID"><img src="https://developer.apple.com/assets/elements/badges/download-on-the-app-store.svg" height="34"></a>
</p>

This document defines the cryptographic protocols used in Hades Messaging
and references the peer-reviewed research that underpins each component.

---

## Protocol Summary

| Layer               | Algorithm                       | Standard / Paper                              |
|---------------------|---------------------------------|-----------------------------------------------|
| Seed Identity       | BIP-39 → BIP-32 HD derivation         | BIP-39 Spec, BIP-32/44 Standards              |
| Key Exchange        | X25519 + ML-KEM-768 (PQXDH)    | Signal PQXDH Spec, Bhargavan et al. 2024      |
| Session Ratchet     | Double Ratchet + SPQR           | Cohn-Gordon et al. 2020, Signal Braid 2024    |
| Symmetric Cipher    | ChaCha20-Poly1305               | RFC 8439                                      |
| Hash                | BLAKE3                          | BLAKE3 Spec                                   |
| KDF                 | HKDF-SHA256                     | RFC 5869                                      |
| Signatures          | Ed25519                         | RFC 8032                                      |
| Post-Quantum Sigs   | Dilithium5 (planned)            | NIST FIPS 204                                 |
| Group Messaging     | MLS / TreeKEM (CGKA)            | IETF RFC 9420                                 |
| Key Transparency    | AKD (Auditable Key Directory)   | SEEMless (Meta), CONIKS 2015                  |
| Sealed Sender       | Double-Sealed Envelopes (v2)    | Signal Sealed Sender + Hades extensions       |
| Contact Discovery   | SimplePIR                       | Henzinger et al. 2023/2025                    |
| Anonymous Auth      | Blind signatures + ZK proofs    | Chaum 1983, zkgroup                           |
| Storage Encryption  | SQLCipher (AES-256-CBC)         | Zetetic SQLCipher                             |
| Key Derivation      | Argon2id                        | RFC 9106                                      |
| Onion Routing       | Arti 2.0 + Vanguards-v2        | Tor Project 2024                              |
| Cover Traffic       | Poisson-distributed chaff       | Hades cover_traffic module                    |
| Multi-Chain Wallet  | BIP-32/44 HD + secp256k1/Ed25519 | BIP-32, BIP-44, Bitcoin/Ethereum/Solana specs |

---

## 1. Unified Seed Identity: BIP-39 → BIP-32

Hades uses a single BIP-39 mnemonic (24 words, 256-bit entropy) as the root
of all key material. Possession of the seed = possession of the account.

**Derivation paths:**
- Messaging identity: `m/13'/0'/0'` (purpose 13 = Hades messaging, non-colliding)
- Wallet keys: `m/44'/{coin_type}'/0'/0/0` (BIP-44 standard per chain)

**Messaging key derivation flow:**
1. Generate 24-word BIP-39 mnemonic (256-bit entropy, CSPRNG).
2. Derive 512-bit master seed via PBKDF2-HMAC-SHA512 (2048 iterations).
3. Derive child key at `m/13'/0'/0'` using BIP-32 hardened derivation.
4. First 32 bytes → Ed25519 signing key.
5. SHA-512(Ed25519 seed)[0..32] → X25519 static secret (clamped per curve25519 spec).
6. Hades ID = BLAKE3(Ed25519 public key).

**Wallet key derivation:**
Each chain derives keys at standard BIP-44 paths:
- Bitcoin: `m/44'/0'/0'/0/0` (secp256k1 → P2WPKH)
- Ethereum: `m/44'/60'/0'/0/0` (secp256k1 → Keccak-256 address)
- Solana: `m/44'/501'/0'/0/0` (Ed25519)
- EVM chains share the Ethereum path with chain-specific EIP-155 signing.

All intermediate seed material implements `ZeroizeOnDrop`.

**References:**
- BIP-39: Mnemonic code for generating deterministic keys
- BIP-32: Hierarchical Deterministic Wallets
- BIP-44: Multi-Account Hierarchy for Deterministic Wallets

---

## 2. Key Exchange: PQXDH

Hades uses a hybrid post-quantum key exchange combining X25519 (classical)
with ML-KEM-768 (post-quantum). This ensures that even a "harvest now,
decrypt later" adversary with future quantum capabilities cannot break
session keys established today.

**Protocol flow:**
1. Alice fetches Bob's prekey bundle (IdentityKey, SignedPreKey, OneTimePreKey, KyberPreKey).
2. Alice performs X25519 DH with each classical key.
3. Alice encapsulates a shared secret using Bob's KyberPreKey (ML-KEM Encaps).
4. All shared secrets are combined via HKDF to produce the root key.

**References:**
- Signal PQXDH Specification: signal.org/docs/specifications/pqxdh/
- Bhargavan et al., "Formal Verification of PQXDH" (USENIX Security 2024)
- Apple PQ3: security.apple.com/blog/imessage-pq3/

---

## 3. Session Ratchet: Double Ratchet + SPQR

Each message uses a unique symmetric key derived from the Double Ratchet,
providing per-message forward secrecy and post-compromise security.

The SPQR extension periodically injects fresh ML-KEM key exchanges into
the ratchet, ensuring post-quantum forward secrecy for ongoing sessions.

**References:**
- Signal "Braid: Post-Quantum End-to-End Encryption" (2024)
- Cohn-Gordon et al., "A Formal Security Analysis of the Signal Messaging Protocol" (J. Cryptology, 2020)

---

## 4. Group Messaging: MLS (RFC 9420)

Group conversations use Messaging Layer Security with TreeKEM for
O(log N) key updates. Known limitations:

- Inactive members degrade group forward secrecy (Quarantined-TreeKEM, CCS 2024)
- Practical performance diverges from theoretical bounds (Soler et al., arXiv 2025)

**References:**
- IETF RFC 9420: "The Messaging Layer Security (MLS) Protocol"
- Alwen et al., "Security Analysis and Improvements for the IETF MLS Standard" (CRYPTO 2020)
- Quarantined-TreeKEM (ACM CCS 2024)

---

## 5. Key Transparency: AKD

Hades publishes identity keys to an Auditable Key Directory so users can
detect server-side key swaps (MITM). Each client monitors its own entry
and raises an alert if the key changes without a local key rotation.

**References:**
- Melara et al., "CONIKS: Bringing Key Transparency to End Users" (USENIX Security 2015)
- Chase & Meiklejohn, "Transparency Overlays and Applications" (CCS 2022) -- "SEEMless"
- Apple iMessage Contact Key Verification Spec (2024)

---

## 6. Private Contact Discovery: SimplePIR

Alice queries the server's user database without revealing which record
she is looking for. The server processes the entire database against her
encrypted query and returns an encrypted response.

**References:**
- Henzinger et al., "SimplePIR: High-Throughput PIR" (USENIX Security 2023/2025)
- Kales et al., "Mobile Private Contact Discovery at Scale" (USENIX Security 2019)
- Lin et al., "Finding Balance in Unbalanced PSI" (IACR CiC, 2025)

---

## 7. Sealed Sender v2

Double-layered metadata encryption:
- **Outer seal**: encrypted to the relay (peeled at the relay hop).
- **Inner seal**: encrypted to the recipient (only decrypted on device).

Messages are padded to fixed size buckets (512B, 8KB, 64KB) to prevent
packet-length analysis.

---

## 8. Cover Traffic

Chaff packets (Poisson-distributed, CSPRNG-filled) are sent when the app
is active to mask real message timing. Timing jitter (50-500ms) is added
to every real message send.

---

## 9. Anti-Forensics

- All key material uses `zeroize`-on-drop.
- Emergency wipe destroys the SQLCipher database and all cached data.
- Plausible deniability dual-volume: two passwords open two different
  encrypted stores sharing the same ciphertext space.

---

## 10. Onion Routing: Arti 2.0

- Vanguard-v2 multi-layer guard rotation (Fixed -> L2 -> L3).
- Pluggable transports: Obfs4, WebTunnel, Snowflake 2.0, Meek.
- Bridge auto-rotation every 7-30 days.
- Conflux multi-path circuits for latency reduction.

**References:**
- Tor Project, "Vanguards: Protecting Tor Hidden Services" (2024)
- Arti 2.0 Roadmap (Tor Project, 2025)

---

## 11. Known Limitations

1. **Authentication in PQXDH is not quantum-secure.** An active quantum
   adversary can mount unknown key-share attacks. Mitigation: Dilithium5
   post-quantum signatures (planned).

2. **MLS inactive members.** Users offline for extended periods degrade
   group forward secrecy. Mitigation: Quarantined-TreeKEM (CCS 2024).

3. **Deniability in post-quantum context** requires further research.
   Hades currently provides offline deniability but not online deniability
   against a malicious quantum adversary.

4. **Cover traffic increases bandwidth.** On metered connections, users
   may disable cover traffic, accepting reduced timing-analysis resistance.

---

## Implementation Reference

Each protocol layer maps to a Rust crate module:

| Protocol Layer | Crate | Source File |
|----------------|-------|-------------|
| BIP-39 seed + messaging key | hades-identity | `seed.rs` |
| Account recovery | hades-identity | `recovery.rs` |
| PQXDH key exchange | hades-crypto | `pqxdh.rs` |
| Double Ratchet + SPQR | hades-crypto | `double_ratchet.rs` |
| ChaCha20-Poly1305 AEAD | hades-crypto | `aead.rs` |
| BLAKE3 fingerprints | hades-crypto | `fingerprint.rs` |
| HKDF key derivation | hades-crypto | `kdf.rs` |
| Sealed Sender v2 | hades-crypto | `sealed_sender_v2.rs` |
| Sender key distribution | hades-crypto | `sender_keys.rs` |
| Anti-forensics | hades-crypto | `anti_forensics.rs` |
| MTU padding | hades-crypto | `padding.rs` |
| Screenshot guard | hades-crypto | `screenshot_guard.rs` |
| Cover traffic | hades-onion | `cover_traffic.rs` |
| Pluggable transports | hades-onion | `pluggable_transport.rs` |
| Bridge rotation | hades-onion | `bridge_rotation.rs` |
| Onion encryption | hades-onion | `onion_encrypt.rs` |
| Anonymous credentials | hades-identity | `anonymous_credentials.rs` |
| Identity management | hades-identity | `identity.rs` |
| HD wallet derivation | hades-wallet | `hd.rs` |
| Bitcoin transactions | hades-wallet | `chains/bitcoin.rs` |
| Ethereum transactions | hades-wallet | `chains/ethereum.rs` |
| Solana transactions | hades-wallet | `chains/solana.rs` |
| Multi-chain RPC | hades-wallet | `rpc.rs` |
| Transaction service | hades-wallet | `transaction.rs` |
| Relay authentication | hades-relay | `auth.rs` |
| Message routing | hades-relay | `router.rs` |

For the threat model and adversary analysis, see [THREAT_MODEL.md](THREAT_MODEL.md).
