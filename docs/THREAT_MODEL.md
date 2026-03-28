# Hades Messaging -- Threat Model

This document describes the adversary classes Hades is designed to resist,
the specific threats within each class, and the technical mitigations
implemented or planned.

---

## Adversary Classes

### 1. Global Passive Adversary (GPA)

**Capability:** Observes all Internet traffic (e.g. nation-state SIGINT).

| Threat                       | Mitigation                                      | Status      |
|------------------------------|-------------------------------------------------|-------------|
| Traffic correlation          | Cover traffic (Poisson chaff)                   | Implemented |
| Packet length analysis       | MTU bucketing (512B / 8KB / 64KB)               | Implemented |
| Timing correlation           | Timing jitter (50-500ms per message)            | Implemented |
| Social graph inference       | Sealed Sender v2 (double-sealed envelopes)      | Implemented |
| Tor circuit fingerprinting   | Vanguard-v2 multi-layer guards                  | Planned     |
| Multi-jurisdiction tapping   | Geographic relay distribution (IS/CH/RO)        | Planned     |

### 2. Active Network Adversary

**Capability:** Injects, modifies, or drops packets (e.g. ISP, firewall).

| Threat                       | Mitigation                                      | Status      |
|------------------------------|-------------------------------------------------|-------------|
| DPI blocking of Tor          | Pluggable transports (Obfs4, WebTunnel)         | Implemented |
| Bridge enumeration           | Auto-rotation every 7-30 days                   | Implemented |
| Certificate spoofing         | Certificate pinning + onion-only endpoints      | Planned     |
| Packet injection / replay    | AEAD (ChaCha20-Poly1305) per message            | Implemented |
| DNS hijacking                | .onion addresses only (no DNS)                  | Planned     |

### 3. Server Compromise

**Capability:** Full control of the relay (root access, RAM inspection).

| Threat                       | Mitigation                                      | Status      |
|------------------------------|-------------------------------------------------|-------------|
| Read message content         | E2EE (messages never decrypted on server)       | Implemented |
| Identify senders             | Sealed Sender (server never sees sender)        | Implemented |
| Swap public keys (MITM)      | Key Transparency (AKD)                          | Planned     |
| Extract keys from RAM        | AMD SEV-SNP encrypted VM                        | Planned     |
| Log metadata                 | Zero-persistence relay (NixOS stateless)        | Planned     |

### 4. Endpoint Compromise

**Capability:** Malware on the user's device.

| Threat                       | Mitigation                                      | Status      |
|------------------------------|-------------------------------------------------|-------------|
| Read decrypted messages      | SQLCipher AES-256 with Argon2id                 | Implemented |
| Extract keys from memory     | `zeroize`-on-drop for all key material          | Implemented |
| Core dump / swap exposure    | Secure memory (mlock, MADV_DONTDUMP)            | Implemented |
| Screenshot capture           | Screenshot guard (platform API)                 | Planned     |
| Physical device seizure      | Emergency wipe (local + remote trigger)          | Implemented |
| Coerced unlock               | Plausible deniability (dual-volume)             | Implemented |
| Wallet key exfiltration      | BIP-39 seed encrypted in SQLCipher vault        | Implemented |
| Wallet transaction tampering | All signing done locally, keys never leave device| Implemented |

### 5. Quantum Adversary

**Capability:** Access to a cryptographically relevant quantum computer.

| Threat                       | Mitigation                                      | Status      |
|------------------------------|-------------------------------------------------|-------------|
| Break key exchange           | PQXDH: ML-KEM-768 + X25519 hybrid              | Implemented |
| Break session keys           | SPQR: periodic ML-KEM ratchet injection         | Implemented |
| Break signatures             | Dilithium5 post-quantum signatures              | Planned     |
| Harvest-now-decrypt-later    | All sessions use PQ key exchange from day one   | Implemented |

### 6. Legal Coercion

**Capability:** Court orders, national security letters, gag orders.

| Threat                       | Mitigation                                      | Status      |
|------------------------------|-------------------------------------------------|-------------|
| Backdoor demand              | Reproducible builds (binary matches source)     | Planned     |
| Data production order        | Zero-knowledge server (nothing to produce)      | Planned     |
| Key disclosure order         | Multi-party key sharding                        | Planned     |
| Gag order                    | Warrant canary system                           | Planned     |
| Jurisdiction seizure         | .onion identity + multi-jurisdiction deployment | Planned     |

---

## Attack Surface Summary

```
                             ADVERSARY
                                |
          +---------+-----------+-----------+---------+
          |         |           |           |         |
       Network   Server     Client     Quantum    Legal
          |         |           |           |         |
    [Tor+PT]   [Sealed]   [SQLCipher] [PQXDH]  [Repro]
    [Chaff]    [AKD]      [Zeroize]  [SPQR]   [Canary]
    [Jitter]   [SEV-SNP]  [DualVol]  [Dili5]  [Shard]
```

---

## Design Principles

1. **Defense in depth.** No single layer is trusted. Compromise of any one
   component (server, network, device) does not break confidentiality.

2. **Fail closed.** If a cryptographic operation fails, the message is
   dropped -- never sent in plaintext.

3. **Minimal trust.** The relay is untrusted by design. It cannot read
   messages, identify senders, or reconstruct the social graph.

4. **Crypto agility.** All algorithms are behind trait abstractions.
   Swapping ChaCha20 for AES-256-GCM or X25519 for X448 requires no
   protocol changes.

5. **Reproducibility.** Every release binary can be verified against the
   source code hash. Users who cannot verify the build should not trust it.

---

## Recommended Deployment Regions

| Tier     | Location      | Provider       | Rationale                          |
|----------|---------------|----------------|------------------------------------|
| Primary  | Iceland       | FlokiNET       | Strongest privacy laws in the West |
| Primary  | Switzerland   | Private Layer   | Federal Data Protection Act        |
| Secondary| Romania       | M247           | EU GDPR, no mandatory data retention|
| Fallback | P2P (libp2p)  | N/A            | No server dependency               |

---

## References

1. Cohn-Gordon et al., "A Formal Security Analysis of the Signal Messaging Protocol" (J. Cryptology, 2020)
2. IETF RFC 9420: "The Messaging Layer Security (MLS) Protocol" (2023)
3. Henzinger et al., "SimplePIR: High-Throughput PIR" (USENIX Security 2023)
4. Signal, "Braid: Post-Quantum End-to-End Encryption" (2024)
5. Bhargavan et al., "Formal Verification of PQXDH" (USENIX Security 2024)
6. Tor Project, "Vanguards: Protecting Tor Hidden Services" (2024)
7. AMD, "SEV-SNP: Strengthening the Trusted Execution Environment" (2025)
8. Melara et al., "CONIKS: Bringing Key Transparency to End Users" (USENIX Security 2015)
9. Rosler et al., "More is Less: On the End-to-End Security of Group Chats" (EuroS&P 2018)
10. Basin et al., "A Formal Analysis of the iMessage PQ3 Messaging Protocol" (2024)

---

## Implementation Cross-Reference

| Mitigation | Implementing Module |
|------------|--------------------|
| BIP-39 seed identity | `hades-identity/src/seed.rs` |
| Account recovery | `hades-identity/src/recovery.rs` |
| Cover traffic (Poisson chaff) | `hades-onion/src/cover_traffic.rs` |
| MTU bucketing | `hades-crypto/src/sealed_sender_v2.rs` |
| Timing jitter | `hades-onion/src/cover_traffic.rs` |
| Sealed Sender v2 | `hades-crypto/src/sealed_sender_v2.rs` |
| Pluggable transports | `hades-onion/src/pluggable_transport.rs` |
| Bridge auto-rotation | `hades-onion/src/bridge_rotation.rs` |
| Zeroize-on-drop | `hades-crypto/src/anti_forensics.rs` |
| Emergency wipe | `hades-crypto/src/anti_forensics.rs` |
| Dual-volume deniability | `hades-crypto/src/anti_forensics.rs` |
| Screenshot guard | `hades-crypto/src/screenshot_guard.rs` |
| Anonymous auth | `hades-identity/src/anonymous_credentials.rs` |
| NixOS hardened relay | `deployment/configuration.nix` |
| BLAKE3 fingerprints | `hades-crypto/src/fingerprint.rs` |
| HD wallet key derivation | `hades-wallet/src/hd.rs` |
| Bitcoin transactions | `hades-wallet/src/chains/bitcoin.rs` |
| Ethereum transactions | `hades-wallet/src/chains/ethereum.rs` |
| Solana transactions | `hades-wallet/src/chains/solana.rs` |
| Relay challenge-response auth | `hades-relay/src/auth.rs` |
| Rate limiting | `hades-relay/src/rate_limit.rs` |

---

## Privacy of Delivery Receipts

Hades delivery and read receipts are designed to leak zero metadata:

| Receipt Type | Transport | Server Knowledge |
|-------------|-----------|------------------|
| Delivery ACK | Relay-mediated | Server learns "a message was delivered" but not to whom (sealed sender) |
| Read receipt | E2EE control message | Server learns nothing (client-to-client encrypted) |
| Typing indicator | Ephemeral E2EE | Server learns nothing |

Users can globally disable read receipts in Settings. If disabled, the Read
state never triggers and no E2EE control messages are sent.

For the full protocol specification, see [CRYPTOGRAPHY.md](CRYPTOGRAPHY.md).
