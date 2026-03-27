#!/usr/bin/env bash

# Hades Messaging - CI/CD Secrets Configuration Helper
# Run this script to securely upload your local deployment secrets to GitHub.
# Requires GitHub CLI (gh) to be authenticated: `gh auth login`

set -e

echo "🔐 Hades GitHub Secrets Configuration"
echo "-------------------------------------"

if ! command -v gh &> /dev/null; then
    echo "❌ Error: GitHub CLI ('gh') is not installed. Please install it first."
    exit 1
fi

echo "Checking GitHub CLI authentication..."
gh auth status >/dev/null

echo "This script will prompt you for the required production secrets"
echo "and securely upload them to your GitHub repository using libsodium encryption."
echo ""

# 1. Android Keystore
echo "1. Android Keystore Base64"
echo "Provide the path to your 'hades-release.keystore' file:"
read -p "Path: " KEYSTORE_PATH
if [ -f "$KEYSTORE_PATH" ]; then
    # Base64 encode without newlines
    base64 -w0 "$KEYSTORE_PATH" | gh secret set ANDROID_KEYSTORE_BASE64
    echo "✅ ANDROID_KEYSTORE_BASE64 configured."
else
    echo "⚠️ File not found. Skipping."
fi

# 2. Keystore Password
echo ""
echo "2. Android Keystore Password"
read -sp "Password: " KS_PASS
echo ""
if [ -n "$KS_PASS" ]; then
    echo "$KS_PASS" | gh secret set ANDROID_KEYSTORE_PASSWORD
    echo "✅ ANDROID_KEYSTORE_PASSWORD configured."
fi

# 3. Key Password
echo ""
echo "3. Android Key Password"
read -sp "Password: " KEY_PASS
echo ""
if [ -n "$KEY_PASS" ]; then
    echo "$KEY_PASS" | gh secret set ANDROID_KEY_PASSWORD
    echo "✅ ANDROID_KEY_PASSWORD configured."
fi

# 4. Codecov Token
echo ""
echo "4. Codecov.io Token (for PR coverage reports)"
read -p "Token (leave blank to skip): " CC_TOKEN
if [ -n "$CC_TOKEN" ]; then
    echo "$CC_TOKEN" | gh secret set CODECOV_TOKEN
    echo "✅ CODECOV_TOKEN configured."
fi

echo ""
echo "🚀 All required secrets have been uploaded."
echo "Your GitHub Actions CI/CD release pipeline is now fully enabled!"
