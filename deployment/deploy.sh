#!/usr/bin/env bash
set -euo pipefail

echo "=== Hades Deployment ==="

# 1. Build relay
echo "[1/6] Building relay server..."
cd "$(git rev-parse --show-toplevel)"
cargo build --release --package hades-relay --bin hades-relay

# 2. Build Android APK
echo "[2/6] Building Android APK..."
cd client && npm install && cd ..
cargo tauri android build -- --apk --split-per-abi

# 3. Generate checksums
echo "[3/6] Generating checksums..."
APK_DIR="gen/android/app/build/outputs/apk/release"
for apk in "$APK_DIR"/*.apk; do
    sha256sum "$apk" > "$apk.sha256"
done

# 4. Deploy relay
echo "[4/6] Deploying relay..."
if [ -f deployment/docker-compose.yml ]; then
    cd deployment
    docker compose build --no-cache
    docker compose up -d
    cd ..
fi

# 5. Verify deployment
echo "[5/6] Verifying..."
sleep 5
if curl -sf https://relay.hades.im/health > /dev/null 2>&1; then
    echo "  ✓ Relay is healthy"
else
    echo "  ✗ Relay health check failed"
    exit 1
fi

# 6. Tag release
echo "[6/6] Creating release tag..."
VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
git tag -a "v$VERSION" -m "Release v$VERSION"
echo "Tagged v$VERSION — push with: git push origin v$VERSION"

echo ""
echo "=== Deployment complete ==="
echo "Relay:   https://relay.hades.im"
echo "APKs:    $APK_DIR/"
echo "Version: v$VERSION"
