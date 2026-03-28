# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **BIP-39 Unified Seed Identity** (`hades-identity`)
  - Single 24-word mnemonic governs messaging identity + all wallet keys
  - Deterministic derivation: `m/13'/0'/0'` (messaging) + `m/44'/…` (wallet)
  - Ed25519 signing key + X25519 key exchange derived from seed
  - Hades ID = BLAKE3(ed25519_public_key)
  - Challenge-response relay authentication (Ed25519 nonce signing)
  - Account recovery from seed phrase
  - Safety number generation (Signal-style, deterministic + symmetric)

- **Multi-chain HD wallet** (`hades-wallet`)
  - BIP-32/44 hierarchical deterministic key derivation
  - 12 supported chains: Bitcoin, Ethereum, Solana, Polygon, Arbitrum, Optimism, Avalanche, Base, BNB Smart Chain, Litecoin, Dogecoin, Tron
  - Bitcoin: P2WPKH address generation, UTXO transaction building
  - Ethereum + EVM: EIP-155 signed transactions, gas estimation
  - Solana: Ed25519 transactions, SOL transfers
  - Real-time balance fetching via multi-chain RPC
  - In-chat crypto transfers with wallet messages
  - Transaction history with background confirmation tracking
  - Token price feeds
  - RPC response caching

- **Wallet UI** (`client/`)
  - Multi-chain wallet dashboard (Wallet.tsx)
  - Cross-chain send flow with gas estimation (WalletSend.tsx)
  - Receive screen with address + QR display (WalletReceive.tsx)
  - Transaction history view (WalletHistory.tsx)
  - In-chat crypto send sheet (InChatSendSheet.tsx)
  - Crypto transfer message bubbles (CryptoTransferBubble.tsx)
  - Chain badge and token selector components
  - walletStore Zustand state management

- **Tauri bridge commands** (`src-tauri/`)
  - Auth commands: create_identity, unlock_vault, restore_identity, has_identity, get_auth_state
  - Contact commands: get_contact_link, get_contact_qr, add_contact_from_bundle, get_contact_wallet_address
  - Wallet commands: wallet_init, wallet_import, wallet_get_balance, wallet_send, wallet_get_transactions, wallet_get_address, wallet_estimate_fee, wallet_export_mnemonic
  - Biometric authentication (biometric_available, biometric_authenticate)
  - Push notification registration (register_push)
  - Burn timer background task for disappearing messages
  - Cryptographic message pipeline (encrypt → pad → seal → send)
  - WebSocket relay connection management

- **Voice/Video calling** (`client/`)
  - Incoming and outgoing call screens
  - Active voice call UI with controls
  - Active video call UI with camera toggle
  - Call history log
  - callStore Zustand state management

- **Contacts system** (`client/` + `src-tauri/`)
  - Contact list management (Contacts.tsx)
  - Add contact via QR code or link (AddContact.tsx)
  - Contact bundle exchange protocol
  - contactStore Zustand state management

- **Recovery phrase UI** (`client/`)
  - 24-word seed phrase display and backup (RecoveryPhrase.tsx)
  - Recovery phrase verification flow

- **New UI components** (`client/`)
  - TorStatusBar: Tor connection status indicator
  - VoiceRecorder: Audio recording with waveform visualization
  - ReactionPicker: Emoji reaction selector
  - ReplyPreview: Message reply preview
  - ActionSheet: Bottom sheet actions
  - MessageBubble: Chat bubble with reactions/reply support
  - Toast notification system (ToastContainer + toastStore)

- **Additional Zustand stores** (`client/`)
  - networkStore: Tor connection + network health status
  - toastStore: Toast notification queue management

### Changed

- **Identity system**: Migrated from CSPRNG-only key generation to BIP-39 seed-based deterministic derivation
- **Architecture**: Expanded from 6 to 7 Rust crates (added hades-wallet)
- **Tauri bridge**: Refactored commands into modular files (auth_commands.rs, contact_commands.rs, wallet_commands.rs)
- **Database layer**: Added wallet accounts, wallet transactions, reactions, connection pool tables
- **Frontend routes**: Expanded from 7 to 18 routes (wallet, contacts, calls, recovery phrase)

## [0.1.0] - 2026-03-28

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
  - Screenshot guard (platform API)
  - Sender key distribution for group messaging
  - Audio cryptographic processing
  - Encrypted search indexing
  - Encrypted notification payloads

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
  - Bridge auto-rotation (7–30 day intervals, 5 distribution methods)
  - Cover traffic with Poisson-distributed chaff packets and timing jitter
  - Fixed-size transport cells

- **Relay server** (`hades-relay`)
  - Zero-knowledge message routing
  - WebSocket session management with Axum
  - Challenge-response authentication
  - ScyllaDB transient storage
  - Rate limiting with Governor
  - Server-side prekey storage
  - Offline message queuing

- **Client application** (`client/`)
  - React 18 + TypeScript 5 + Vite frontend
  - Tauri 2.0 native integration
  - Premium vault lock screen with Framer Motion animations
  - Chat list with delivery status indicators (5 states)
  - 8-stage secure route establishment HUD
  - Conversation view with per-message status
  - Entropy-aware onboarding with key generation
  - BLAKE3 fingerprint verification screen
  - Settings screen with privacy, security, and network controls
  - Profile settings
  - Zustand state management (10 stores)
  - i18n localization support
  - Design token system

- **Infrastructure**
  - NixOS declarative relay server configuration
  - Hardened deployment: LUKS FDE, AppArmor, systemd sandboxing
  - Tor hidden service configuration with PoW defense
  - Coturn TURN server for E2EE voice/video relay
  - Caddy reverse proxy with security headers
  - Fail2ban, Prometheus node exporter
  - Docker containerization (Dockerfile.relay)
  - Deployment automation script (deploy.sh)

- **CI/CD pipeline**
  - 9 GitHub Actions workflows: CI, Security Audit, CodeQL, OpenSSF Scorecard, Dependency Review, Container Scan, NixOS Check, Wallet Tests, Release
  - Automated dependency updates via Dependabot (Cargo, npm, GitHub Actions, Docker, NixOS)
  - SLSA Build Level 3 provenance attestation
  - CycloneDX SBOM generation (Rust + npm)
  - Code coverage with Codecov integration
  - Release drafter for automated release notes
  - CODEOWNERS with security-critical path enforcement

- **Documentation**
  - Cryptography specification with 2026 research bibliography
  - Threat model with 6 adversary classes and 30+ mitigations
  - Architecture documentation with 23+ Mermaid diagrams
  - Security policy (SECURITY.md) with severity classification and SLAs
  - Contributing guide (CONTRIBUTING.md) with coding standards
  - cargo-deny configuration (deny.toml)

[Unreleased]: https://github.com/Ashutosh0x/hades-messaging/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Ashutosh0x/hades-messaging/releases/tag/v0.1.0
