# Contributing to Hades Messaging

Thank you for your interest in contributing to Hades. This project implements critical cryptographic protocols and privacy infrastructure — contributions are held to a high standard of security and code quality.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Security Vulnerabilities](#security-vulnerabilities)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Architecture Overview](#architecture-overview)
- [Coding Standards](#coding-standards)
- [Pull Request Process](#pull-request-process)
- [Commit Convention](#commit-convention)
- [Testing Requirements](#testing-requirements)
- [Security-Critical Code](#security-critical-code)

## Code of Conduct

This project follows a zero-tolerance policy for harassment, discrimination, or bad-faith behavior. Be respectful, constructive, and professional.

## Security Vulnerabilities

> **⚠️ DO NOT open public issues for security vulnerabilities.**

If you discover a security vulnerability, please report it responsibly:

- **Email:** [security@hades.im](mailto:security@hades.im)
- Include: description, reproduction steps, potential impact, suggested fix
- We will acknowledge receipt within 48 hours
- We follow coordinated disclosure with a 90-day timeline

See [SECURITY.md](SECURITY.md) for full details.

## Getting Started

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Backend, cryptography, Tor integration |
| Node.js | 20+ | Frontend build tooling |
| Android Studio | Latest | Android SDK/NDK for mobile builds |
| Java | 17+ | Android build system (comes with Android Studio) |

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Ashutosh0x/hades-messaging.git
cd hades-messaging

# Install Tauri CLI
cargo install tauri-cli

# Install frontend dependencies
cd client && npm install && cd ..

# Configure Android SDK (if building for Android)
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
export PATH=$PATH:$ANDROID_HOME/platform-tools

# Run Rust tests
cargo test --workspace

# Run frontend dev server
cd client && npm run dev
```

## Architecture Overview

Hades uses a clean crate separation:

| Crate | Purpose | Security Level |
|-------|---------|---------------|
| `hades-crypto` | Cryptographic primitives (PQXDH, Double Ratchet, Sealed Sender) | 🔴 Critical |
| `hades-identity` | Identity management, BIP-39 seed, key bundles, device sync | 🔴 Critical |
| `hades-onion` | Tor integration, onion routing, cover traffic | 🔴 Critical |
| `hades-wallet` | Multi-chain HD wallet (BTC, ETH, SOL, 9 more chains) | 🔴 Critical |
| `hades-relay` | Message relay server, challenge-response auth | 🟡 High |
| `hades-proto` | Protocol definitions (protobuf) | 🟡 High |
| `hades-common` | Shared types and utilities | 🟢 Standard |
| `src-tauri/` | Tauri 2.0 bridge (commands, DB, pipeline, WebSocket) | 🟡 High |
| `client/` | React/TypeScript frontend (19 screens, 10 stores) | 🟢 Standard |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed diagrams.

## Coding Standards

### Rust

- **Format:** All code must pass `cargo fmt --all -- --check`
- **Lint:** All code must pass `cargo clippy --workspace --all-targets --all-features` with zero warnings
- **Forbidden patterns in production code:**
  - `unwrap()` — use `expect()` with context or proper error handling
  - `panic!()` — handle errors gracefully
  - `todo!()` — not allowed in merged code
  - `dbg!()` — use `tracing` macros instead
- **Error handling:** Use `thiserror` for library errors, `anyhow` for application errors
- **Logging:** Use the `tracing` crate exclusively
- **Memory safety:** All sensitive data must implement `Zeroize` and `ZeroizeOnDrop`

### TypeScript/React

- **Lint:** Must pass `eslint` with zero warnings
- **Types:** Must pass `tsc --noEmit` with strict mode
- **Format:** Must pass `prettier --check`
- **State management:** Use Zustand stores (see `client/src/store/`)
- **Animations:** Use Framer Motion for UI animations
- **Styling:** Use CSS modules or design tokens from `client/src/design/`

### General

- No hardcoded secrets, API keys, or URLs
- No `console.log` in production code (use proper logging)
- All public APIs must have documentation comments
- Maximum function length: ~50 lines (extract helpers)

## Pull Request Process

### Before Opening a PR

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feat/your-feature-name
   ```

2. **Run the full check suite locally:**
   ```bash
   # Rust
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features

   # Frontend
   cd client
   npx tsc --noEmit
   npx eslint src/ --max-warnings=0
   npx prettier --check "src/**/*.{ts,tsx,css,json}"
   npm test
   ```

3. **Ensure your branch is up to date** with `main`

### PR Requirements

- [ ] Descriptive title following [commit convention](#commit-convention)
- [ ] Description explaining **what** and **why** (not just what)
- [ ] All CI checks passing (CI Pass Gate must be green)
- [ ] Tests added/updated for new functionality
- [ ] Documentation updated if APIs changed
- [ ] No unrelated changes bundled in

### Review Process

- All PRs require at least **1 approval** from a code owner
- Security-critical paths (crypto, identity, onion, deployment) require review by `@Ashutosh0x`
- Stale approvals are dismissed when new commits are pushed
- Branch must be up to date with `main` before merging

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `security` | Security fix or improvement |
| `refactor` | Code refactoring (no feature/fix) |
| `docs` | Documentation changes |
| `test` | Adding or updating tests |
| `ci` | CI/CD pipeline changes |
| `deps` | Dependency updates |
| `perf` | Performance improvements |
| `chore` | Maintenance tasks |

### Scopes

| Scope | Crate/Area |
|-------|------------|
| `crypto` | `hades-crypto` |
| `identity` | `hades-identity` |
| `onion` | `hades-onion` |
| `wallet` | `hades-wallet` |
| `relay` | `hades-relay` |
| `proto` | `hades-proto` |
| `common` | `hades-common` |
| `tauri` | `src-tauri/` |
| `client` | Frontend |
| `ci` | CI/CD workflows |
| `deploy` | Deployment/infrastructure |

### Examples

```
feat(crypto): implement ML-KEM-768 key encapsulation
fix(onion): correct guard node rotation interval
security(identity): zeroize prekey material on drop
docs(crypto): update PQXDH protocol specification
ci: add coverage threshold enforcement
deps(rust): bump chacha20poly1305 to 0.10.2
```

## Testing Requirements

### Rust Tests

- **Unit tests:** Required for all new functions
- **Integration tests:** Required for cross-crate interactions
- **Crypto tests:** Must verify:
  - Correctness (encrypt → decrypt roundtrip)
  - Key material zeroization
  - Error cases (invalid keys, tampered ciphertext)
  - Edge cases (empty messages, maximum-size messages)

```bash
# Run all tests
cargo test --workspace --all-features

# Run crypto-specific tests with single thread (for zeroize verification)
cargo test -p hades-crypto --all-features -- --test-threads=1

# Run with logging
RUST_LOG=debug cargo test --workspace
```

### Frontend Tests

- **Component tests:** Required for new UI components
- **Store tests:** Required for Zustand store changes
- **Use Vitest** as the test runner

```bash
cd client
npx vitest run --coverage
```

### Coverage

- Minimum coverage threshold: **60%**
- Coverage is automatically checked in CI
- Upload to [Codecov](https://codecov.io/) on every PR

## Security-Critical Code

Changes to the following areas receive extra scrutiny:

### Cryptographic Code (`hades-crypto`)

- All key material must implement `Zeroize` + `ZeroizeOnDrop`
- No timing-dependent comparisons (use `subtle::ConstantTimeEq`)
- CSPRNG usage only via `OsRng` or properly seeded `ChaChaRng`
- All cryptographic operations must be tested against known test vectors
- Protocol changes require updating `docs/CRYPTOGRAPHY.md`

### Identity Management (`hades-identity`)

- Key storage must use SQLCipher with Argon2id key derivation
- Prekey bundles must be validated before use
- Device linking requires mutual authentication

### Onion Routing (`hades-onion`)

- Circuit construction must enforce minimum hop count
- Cover traffic parameters must not be user-configurable in production
- Guard node selection must follow Tor's Vanguards-v2 spec

### What Triggers Security Review

- Any change to `crates/hades-crypto/`
- Any change to `crates/hades-identity/`
- Any change to `crates/hades-onion/`
- Any change to `crates/hades-wallet/`
- Any change to `deployment/`
- Any change to `docs/CRYPTOGRAPHY.md` or `docs/THREAT_MODEL.md`
- Any new dependency in security-critical crates

---

## Questions?

- **General:** [support@hades.im](mailto:support@hades.im)
- **Security:** [security@hades.im](mailto:security@hades.im)
- **Community:** [Matrix](https://matrix.to/#/#hades:matrix.org)

**True Privacy is Sovereignty.**
