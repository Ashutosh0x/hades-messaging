# Installation Guide

## Download Hades Messenger

<p align="center">
  <a href="https://play.google.com/store/apps/details?id=im.hades.messenger">
    <img src="assets/google-play-badge.png" alt="Get it on Google Play" height="80">
  </a>
  &nbsp;&nbsp;&nbsp;
  <a href="https://apps.apple.com/app/hades-messenger/id0000000000">
    <img src="assets/app-store-badge.png" alt="Download on the App Store" height="80">
  </a>
</p>

<p align="center">
  <a href="https://github.com/Ashutosh0x/hades-messaging/releases/latest">
    <img src="https://img.shields.io/badge/Desktop-Windows%20%7C%20macOS%20%7C%20Linux-0d1117?style=for-the-badge&logo=tauri&logoColor=24C8D8" alt="Desktop Downloads">
  </a>
</p>

---

## Platform Availability

| Platform | Status | Download |
|----------|--------|----------|
| **Android** (8.0+) | ✅ Available | [Google Play](https://play.google.com/store/apps/details?id=im.hades.messenger) \| [APK](https://github.com/Ashutosh0x/hades-messaging/releases/latest) |
| **iOS** (15.0+) | ✅ Available | [App Store](https://apps.apple.com/app/hades-messenger/id0000000000) |
| **Windows** (10+) | ✅ Available | [.msi Installer](https://github.com/Ashutosh0x/hades-messaging/releases/latest) |
| **macOS** (12+) | ✅ Available | [.dmg Installer](https://github.com/Ashutosh0x/hades-messaging/releases/latest) |
| **Linux** | ✅ Available | [AppImage](https://github.com/Ashutosh0x/hades-messaging/releases/latest) \| [.deb](https://github.com/Ashutosh0x/hades-messaging/releases/latest) |

---

## Android

### Google Play Store (Recommended)

The easiest way to install Hades on Android:

1. Open [Hades on Google Play](https://play.google.com/store/apps/details?id=im.hades.messenger)
2. Tap **Install**
3. Open the app and follow the onboarding flow to generate your seed phrase

> **Requirements:** Android 8.0 (API 26) or higher, ARM64 recommended

### Direct APK Download

For users who prefer sideloading or are in regions where Google Play is unavailable:

1. Download the latest APK from [GitHub Releases](https://github.com/Ashutosh0x/hades-messaging/releases/latest)
   - `hades-arm64-v8a-release.apk` — Most modern phones (ARM64)
   - `hades-armeabi-v7a-release.apk` — Older 32-bit ARM devices
   - `hades-x86_64-release.apk` — Emulators and x86 devices
2. Enable **Install from Unknown Sources** in your device settings
3. Open the downloaded APK and tap **Install**

### Verify APK Signature

```bash
# Verify the APK was signed with the official Hades release key
apksigner verify --verbose --print-certs hades-arm64-v8a-release.apk

# Verify SHA256 checksum
sha256sum hades-arm64-v8a-release.apk
# Compare with the checksum in CHECKSUMS.sha256 from the release
```

---

## iOS

### App Store (Recommended)

1. Open [Hades on the App Store](https://apps.apple.com/app/hades-messenger/id0000000000)
2. Tap **Get** to install
3. Open the app and follow the onboarding flow

> **Requirements:** iOS 15.0 or higher, iPhone 8 or later recommended

### TestFlight (Beta)

To join the beta program for early access to new features:

1. Install [TestFlight](https://apps.apple.com/app/testflight/id899247664) from the App Store
2. Open the [Hades TestFlight invite link](https://testflight.apple.com/join/HADES_CODE)
3. Tap **Accept** → **Install**

---

## Desktop

Desktop builds use [Tauri 2.0](https://tauri.app/) for native performance with a minimal footprint.

### Windows

1. Download `Hades_x.y.z_x64-setup.msi` from [GitHub Releases](https://github.com/Ashutosh0x/hades-messaging/releases/latest)
2. Run the installer and follow the prompts
3. Hades will appear in your Start Menu

> **Requirements:** Windows 10 (1803+) or Windows 11, WebView2 runtime (auto-installed)

### macOS

1. Download `Hades_x.y.z_universal.dmg` from [GitHub Releases](https://github.com/Ashutosh0x/hades-messaging/releases/latest)
2. Open the `.dmg` and drag Hades to your Applications folder
3. On first launch, right-click → **Open** to bypass Gatekeeper

> **Requirements:** macOS 12 (Monterey) or higher, Apple Silicon and Intel supported

### Linux

**AppImage (Universal):**
```bash
wget https://github.com/Ashutosh0x/hades-messaging/releases/latest/download/Hades_x.y.z_amd64.AppImage
chmod +x Hades_*.AppImage
./Hades_*.AppImage
```

**Debian/Ubuntu (.deb):**
```bash
wget https://github.com/Ashutosh0x/hades-messaging/releases/latest/download/hades_x.y.z_amd64.deb
sudo dpkg -i hades_*.deb
sudo apt-get install -f  # Install dependencies if needed
```

> **Requirements:** glibc 2.31+, WebKitGTK 4.1+

---

## Verify Downloads

All official releases include cryptographic verification artifacts:

### SHA256 Checksums

Each release publishes a `CHECKSUMS.sha256` file:

```bash
# Download the checksum file
wget https://github.com/Ashutosh0x/hades-messaging/releases/latest/download/CHECKSUMS.sha256

# Verify your download
sha256sum -c CHECKSUMS.sha256 --ignore-missing
```

### GPG Signatures

Release artifacts are signed with the Hades release key:

```bash
# Import the Hades release key
gpg --keyserver keyserver.ubuntu.com --recv-keys HADES_GPG_KEY_ID

# Verify signature
gpg --verify Hades_x.y.z_amd64.AppImage.sig Hades_x.y.z_amd64.AppImage
```

### SLSA Provenance

All releases include [SLSA Level 3](https://slsa.dev/) provenance attestations, verifiable via the [slsa-verifier](https://github.com/slsa-framework/slsa-verifier):

```bash
slsa-verifier verify-artifact \
  --provenance-path provenance.intoto.jsonl \
  --source-uri github.com/Ashutosh0x/hades-messaging \
  Hades_x.y.z_amd64.AppImage
```

---

## Build from Source

For maximum trust, build Hades from source. See the [README Quick Start](../README.md#quick-start-development) for development setup.

### Prerequisites

| Tool | Version | Installation |
|------|---------|-------------|
| Rust | 1.75+ | [rustup.rs](https://rustup.rs/) |
| Node.js | 20+ | [nodejs.org](https://nodejs.org/) |
| Tauri CLI | Latest | `cargo install tauri-cli` |
| Android Studio | Latest | [developer.android.com](https://developer.android.com/studio) (mobile only) |

### Build Commands

```bash
# Clone the repository
git clone https://github.com/Ashutosh0x/hades-messaging.git
cd hades-messaging

# Install dependencies
cargo install tauri-cli
cd client && npm install && cd ..

# Desktop release build
cargo tauri build

# Android release build (APK)
npm run tauri android build -- --apk --split-per-abi

# Android release build (AAB for Play Store)
npm run tauri android build -- --aab
```

### Reproducible Builds

Hades supports reproducible builds. To verify that a release binary matches the source:

```bash
# Build from the tagged source
git checkout v1.0.0
cargo tauri build --bundles none

# Compare the output hash with the published checksum
sha256sum src-tauri/target/release/hades
```

---

## Post-Installation

After installing Hades on any platform:

1. **Generate Seed Phrase** — Create your 24-word BIP-39 mnemonic during onboarding
2. **Back Up Seed Phrase** — Write it down on paper and store in a secure location
3. **Set Passphrase** — Choose a strong passphrase for local vault encryption
4. **Enable Biometrics** — Optionally enable fingerprint/Face ID unlock
5. **Verify Tor Connection** — Check the Tor status indicator shows a successful circuit

> ⚠️ **Your seed phrase is the master key to both your messaging identity and your crypto wallet. If you lose it, your account cannot be recovered. Hades has no server-side account recovery.**

---

## Troubleshooting

### Android

| Issue | Solution |
|-------|----------|
| "App not installed" error | Enable **Install from Unknown Sources** in Settings → Security |
| Play Protect warning | Tap **Install Anyway** — this occurs with sideloaded APKs |
| Tor connection fails | Check that your network allows outbound connections on port 9001 |

### iOS

| Issue | Solution |
|-------|----------|
| App crashes on launch | Ensure iOS 15.0+ and restart the device |
| Notifications not working | Enable notifications in Settings → Hades → Notifications |

### Desktop

| Issue | Solution |
|-------|----------|
| Windows: "WebView2 not found" | Download [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| macOS: "App is damaged" | Run `xattr -cr /Applications/Hades.app` in Terminal |
| Linux: Missing libraries | Install WebKitGTK: `sudo apt install libwebkit2gtk-4.1-dev` |

---

## Uninstallation

### Android
Settings → Apps → Hades → Uninstall

### iOS
Long-press the Hades icon → Remove App → Delete App

### Windows
Settings → Apps → Hades → Uninstall

### macOS
Drag Hades from Applications to Trash

### Linux
```bash
# AppImage — simply delete the file
rm Hades_*.AppImage

# .deb
sudo dpkg -r hades
```

> **Note:** Uninstalling Hades deletes the local encrypted database. Ensure you have your seed phrase backed up before uninstalling.
