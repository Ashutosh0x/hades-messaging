# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Cryptographic primitives** (`hades-crypto`)
  - PQXDH key exchange (X25519 + ML-KEM-768)
  - Double Ratchet with SPQR post-quantum injection
  - ChaCha20-Poly1305 AEAD encryption
  - BLAKE3 contact fingerprints
  - HKDF-SHA256 key derivation
  - Sealed Sender v2 with double-sealed envelopes (512B/8KB/64KB buckets)
  - MTU bucket padding for traffic analysis resistance
  - Anti-forensics: zeroize-on-drop, plausible deniability volumes, emergency wipe

- **Identity management** (`hades-identity`)
  - Ed25519 identity key pairs
  - Prekey bundle generation and validation
  - Encrypted key storage with SQLCipher + Argon2id
  - Multi-device synchronization (Sesame algorithm)
  - Anonymous credentials with blind signatures and ZK proofs
  - Safety number generation for contact verification

- **Onion routing** (`hades-onion`)
  - Arti 2.0 integration with Vanguards-v2
  - Multi-hop encrypted circuit construction
  - Pluggable transports: Obfs4, WebTunnel, Snowflake, Meek
  - Bridge auto-rotation (7-30 day intervals, 5 distribution methods)
  - Cover traffic with Poisson-distributed chaff packets and timing jitter
  - Fixed-size transport cells

- **Relay server** (`hades-relay`)
  - Zero-knowledge message routing
  - WebSocket session management
  - ScyllaDB transient storage

- **Client application** (`client/`)
  - React 18 + TypeScript 5 + Vite frontend
  - Tauri 2.0 native integration
  - Premium vault lock screen with Framer Motion animations
  - Chat list with delivery status indicators (5 states)
  - 8-stage secure route establishment HUD
  - Conversation view with per-message status
  - Entropy-aware onboarding with key generation
  - BLAKE3 fingerprint verification screen
  - Zustand state management (connection, conversation, device, security, settings)
  - i18n localization support
  - Design token system

- **Infrastructure**
  - NixOS declarative relay server configuration
  - Hardened deployment with AMD SEV-SNP support (planned)

- **CI/CD pipeline**
  - 8 GitHub Actions workflows: CI, Security Audit, CodeQL, OpenSSF Scorecard, Dependency Review, Container Scan, NixOS Check, Release
  - Automated dependency updates via Dependabot (Cargo, npm, GitHub Actions)
  - SLSA Build Level 3 provenance attestation
  - CycloneDX SBOM generation (Rust + npm)
  - Code coverage with Codecov integration
  - Release drafter for automated release notes

- **Documentation**
  - Cryptography specification with 2026 research bibliography
  - Threat model with 6 adversary classes and 30+ mitigations
  - Architecture documentation with 23 Mermaid diagrams
  - Security policy (SECURITY.md)
  - Contributing guide (CONTRIBUTING.md)

## [0.1.0] - 2025-03-28

### Added

- Initial release of Hades Messaging
- Complete project structure with 6 Rust crates
- React/TypeScript frontend with Tauri 2.0
- Full cryptographic protocol implementation
- CI/CD pipeline with 8 workflows

[Unreleased]: https://github.com/Ashutosh0x/hades-messaging/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Ashutosh0x/hades-messaging/releases/tag/v0.1.0
