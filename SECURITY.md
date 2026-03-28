# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| latest  | Active support  |
| < latest| Critical fixes only |

## Reporting a Vulnerability

**DO NOT** open public issues for security vulnerabilities.

### Contact

**security@hades.im**

### What to Include

1. **Description** of the vulnerability
2. **Steps to reproduce** (minimal proof of concept)
3. **Potential impact** assessment
4. **Affected component** (hades-crypto, hades-onion, hades-identity, hades-wallet, client, relay, src-tauri)
5. **Suggested fix** (if any)

### Response Timeline

| Stage | SLA |
|-------|-----|
| Acknowledgment | 24 hours |
| Triage & severity | 48 hours |
| Fix development | 7 days (critical), 30 days (high) |
| Disclosure | Coordinated, 90 days max |

### Severity Classification

| Severity | Examples |
|----------|---------|
| **Critical** | RCE, key exfiltration, authentication bypass, sealed sender de-anonymization, wallet private key exposure |
| **High** | Metadata leakage, traffic correlation, cryptographic weakness, wallet transaction manipulation |
| **Medium** | Denial of service, information disclosure (non-key material) |
| **Low** | UI issues, non-exploitable edge cases |

### Bug Bounty

We are establishing a formal bug bounty program. Contact security@hades.im for details.

### PGP Key

Our security team's PGP key will be published at:
- `https://hades.im/.well-known/security.txt`
- `https://keys.openpgp.org` (search: security@hades.im)

## Security Measures

This project employs:
- Automated dependency auditing (Dependabot, cargo-audit, npm audit)
- CodeQL semantic analysis on every PR
- OpenSSF Scorecard monitoring
- SLSA Build Level 3 provenance on releases
- Signed release artifacts with SHA-256/SHA-512 checksums
- Reproducible builds (planned)
- BIP-39 seed + all wallet keys encrypted at rest (SQLCipher + Argon2id)
- Zeroize-on-drop for all key material (messaging and wallet)
- Challenge-response relay authentication (Ed25519 nonce signing)
- 9 CI/CD workflows with security audit, container scan, and dependency review
