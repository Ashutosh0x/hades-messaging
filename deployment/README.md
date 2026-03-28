# Hades Relay — Deployment

Sovereign infrastructure configuration for self-hosted Hades relay servers.

## Contents

| File | Purpose |
|------|---------|
| `configuration.nix` | NixOS declarative hardened relay server configuration |
| `Dockerfile.relay` | Container image for relay binary |
| `Caddyfile` | Reverse proxy with automatic HTTPS and security headers |
| `deploy.sh` | Automated deployment script |

## NixOS Deployment

The recommended deployment uses NixOS for declarative, reproducible server state:

```bash
# Build and deploy
nixos-rebuild switch --flake .#hades-relay
```

### Hardening Features

- **Full disk encryption** (LUKS)
- **AppArmor** mandatory access control
- **Systemd sandboxing**: ProtectSystem, NoNewPrivileges, MemoryDenyWriteExecute
- **Kernel hardening**: slab_nomerge, init_on_alloc, lockdown=confidentiality
- **Tor hidden service** with PoW DoS defense
- **Caddy** reverse proxy (auto HTTPS, HSTS, CSP)
- **Fail2ban** SSH brute-force protection
- **Coturn** TURN server for E2EE voice/video relay
- **Prometheus** node exporter for monitoring
- **Minimal surface**: only essential packages installed
- **Resource limits**: 512MB memory cap, 65K file descriptor limit

### Services

| Service | Port | Description |
|---------|------|-------------|
| hades-relay | 8443 (internal) | Message relay (WebSocket) |
| Caddy | 443 (external) | HTTPS reverse proxy |
| Tor | 9001 | Bridge relay |
| Tor HS | .onion:443 | Hidden service endpoint |
| Coturn | 3478/5349 | TURN media relay |
| SSH | 22 | Administration (key-only) |

## Docker Deployment

```bash
docker build -f Dockerfile.relay -t hades-relay .
docker run -d -p 8443:8443 hades-relay
```

## Recommended Regions

| Tier | Location | Rationale |
|------|----------|-----------|
| Primary | Iceland | Strongest privacy laws in the West |
| Primary | Switzerland | Federal Data Protection Act |
| Secondary | Romania | EU GDPR, no mandatory data retention |
| Fallback | P2P (libp2p) | No server dependency |
