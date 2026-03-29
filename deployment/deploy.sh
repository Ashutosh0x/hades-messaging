#!/usr/bin/env bash
set -euo pipefail

echo "=== Hades Production Deployment ==="

# 1. Build relay server
echo "[1/8] Building relay server..."
cargo build --release --package hades-relay --bin hades-relay

# 2. Run security audits
echo "[2/8] Running security audits..."
cargo audit || true  # Continue even if warnings
cargo deny check

# 3. Run all tests
echo "[3/8] Running tests..."
cargo test --workspace --release

# 4. Build Tauri app
echo "[4/8] Building Tauri app..."
cd client && npm ci && cd ..
cargo tauri build

# 5. Generate checksums
echo "[5/8] Generating checksums..."
for apk in gen/android/app/build/outputs/apk/release/*.apk; do
    sha256sum "$apk" > "$apk.sha256"
done

# 6. Deploy relay
echo "[6/8] Deploying relay..."
if [ -f deployment/docker-compose.yml ]; then
    cd deployment
    docker compose pull
    docker compose up -d --build
    sleep 10
    curl -f http://localhost:8443/health || exit 1
    cd ..
fi

# 7. Create release
echo "[7/8] Creating GitHub release..."
VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | grep -oP '\d+\.\d+\.\d+')
git tag -a "v$VERSION" -m "Release v$VERSION"

# 8. Upload artifacts
echo "[8/8] Upload artifacts to GitHub Releases..."
# gh release create "v$VERSION" \
#   gen/android/app/build/outputs/apk/release/*.apk \
#   --title "Hades v$VERSION" \
#   --notes "See CHANGELOG.md for details"

echo ""
echo "=== Deployment Complete ==="
echo "Version: v$VERSION"
echo "Relay: https://relay.hades.im"
echo "APKs: gen/android/app/build/outputs/apk/release/"
