# Labrador

A mobile-first lab notebook app. Photograph handwritten notes and lab images, get AI-transcribed and wikilinked markdown, review in-app, and publish as a GitHub Pull Request to your [LogSeq](https://logseq.com/) vault.

**Stack:** Tauri v2 · Svelte 5 · Tailwind CSS v4 · Rust · Gemini API · GitHub REST API

---

## How it works

1. Capture one or more photos (handwritten notes + lab images)
2. Labrador classifies each image (notes vs. photos), OCRs the notes, generates alt-text for photos, and links concepts as `[[wikilinks]]`
3. Review and edit the generated markdown and PR title in-app
4. Tap **Open PR** → a pull request is created on your GitHub vault repo
5. Review, edit, and merge the PR on GitHub; `git pull` locally to sync with LogSeq

---

## Building

### Prerequisites

You need Rust, Node.js, and the Tauri mobile toolchains.

#### GitHub OAuth App

Labrador uses the GitHub Device Flow for authentication, which requires a registered GitHub OAuth App.

1. Go to **GitHub → Settings → Developer settings → OAuth Apps → New OAuth App**
2. Set **Homepage URL** to anything (e.g. `https://github.com/your-username/labrador`)
3. Leave **Callback URL** blank
4. Click **Register application**
5. On the app page, click **Enable Device Flow**
6. Copy the **Client ID**

Pass it at build time:
```bash
GITHUB_CLIENT_ID=<your-client-id> cargo tauri build
```

Or for Docker builds:
```bash
GITHUB_CLIENT_ID=<your-client-id> make linux
``` The easiest path is the provided Docker image that has everything pre-installed.

### Linux and Android — Podman / Docker

Use the `Makefile` which handles SELinux volume labels, node_modules isolation, and build caching automatically. Works with either Podman (default on Fedora/Bazzite) or Docker.

```bash
git clone <repo-url>
cd labrador

# Standalone Linux AppImage (bundles WebKit + all GTK libs — runs anywhere)
make linux

# Android APK + AAB
make android

# Clean cached volumes (force full rebuild next time)
make clean

# Override runtime explicitly
make linux RUNTIME=docker
```

Outputs:
- `src-tauri/target/release/bundle/appimage/*.AppImage` — standalone Linux binary
- `src-tauri/gen/android/app/build/outputs/apk/**/*.apk` — Android APK
- `src-tauri/gen/android/app/build/outputs/bundle/**/*.aab` — Android App Bundle

Cargo registry, Rust target dir, node_modules, and Gradle cache are stored in named volumes so subsequent builds are fast.

### macOS and iOS — GitHub Actions

Apple's toolchain (`hdiutil`, Xcode, codesigning) only runs on macOS and cannot be cross-compiled from Linux. macOS and iOS builds are handled by `.github/workflows/build.yml` on hosted macOS runners.

To trigger a build: push to `main` or `git push origin v1.0.0` (any `v*` tag). Artifacts are uploaded to the Actions run:

| Artifact | Contents |
|---|---|
| `macos-dmg` | Universal `.dmg` (x86-64 + Apple Silicon) |
| `ios-ipa` | Unsigned `.ipa` for sideloading |

To build locally on a Mac:

```bash
npm ci

# macOS — universal binary (arm64 + x86_64), all Rust deps statically linked
cargo tauri build --target universal-apple-darwin --bundles dmg

# iOS
cargo tauri ios build
```

### Without Docker (native)

Install the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS, then:

```bash
npm install

# Desktop
npm run tauri build

# Android (requires Android Studio + NDK)
npm run tauri android build

# iOS (requires Xcode on macOS)
npm run tauri ios build
```

---

## Installing on Android

1. Enable **Install from unknown sources** in Android Settings → Security (or per-browser setting)
2. Transfer the `.apk` from `src-tauri/gen/android/app/build/outputs/apk/universal/release/` to your device (USB, ADB, or cloud storage)
3. Open the file on the device and tap **Install**

Via ADB:
```bash
adb install src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

For Play Store distribution, use the `.aab` from the same directory and upload via Google Play Console.

---

## Installing on iPhone

Download the `ios-ipa` artifact from the GitHub Actions run (or build locally on a Mac).

- **AltStore / Sideloadly (free):** Sign the `.ipa` with a free Apple ID and sideload. Re-sign every 7 days.
- **Direct install (your own device):** Register your UDID in your Apple Developer account; use Xcode or `ios-deploy` to install the signed `.ipa`.
- **TestFlight / App Store:** Archive in Xcode, add signing certs to the Actions secrets, and upload via Xcode Organizer or `xcrun altool`.

---

## Configuration

On first launch Labrador will prompt you to configure three things in **Settings** (gear icon):

### 1. GitHub authentication

Labrador uses the GitHub OAuth Device Flow — no redirect URI or hosted server required.

1. Tap **Connect GitHub**
2. A user code is displayed — open the shown URL on any browser and enter the code
3. Once authorized, the token is stored securely on-device

### 2. Vault repository

The GitHub repo that will receive your notes as Pull Requests. It should be a LogSeq graph repo (or any markdown repo).

- **Owner:** your GitHub username or org (e.g. `my-username`)
- **Repo:** repository name (e.g. `my-vault`)
- **Base branch:** branch to target with PRs (default: `main`)

Tap **Save repo**.

### 3. Gemini API key

Labrador uses Google Gemini (`gemini-2.5-flash`) for image classification, OCR, alt-text generation, and wikilink insertion.

1. Get a free API key at [aistudio.google.com](https://aistudio.google.com/)
2. Paste it into the **Gemini API key** field and tap **Save key**

---

## Usage

1. Open the app — the camera view is the home screen
2. Tap the shutter button to capture a photo; repeat to add more
3. Thumbnails accumulate at the bottom; tap one to remove it
4. Tap **Process** when you have all your images
5. Wait for AI processing (classify → transcribe → link)
6. On the Preview screen: edit the **PR title** and the **markdown** as needed
7. Tap **Open PR** to publish, or **Discard** to start over
8. On GitHub: review the PR, edit the markdown file if needed, and merge
9. `git pull` in your local vault clone → notes appear in LogSeq

---

## Vault structure

Labrador writes two kinds of files to your repo:

```
pages/
  Your Note Title.md   ← the transcribed, linked markdown note
assets/
  <uuid>.jpg           ← source note images and supplemental photos
```

Notes are in LogSeq bullet format with `tags::` frontmatter and `[[wikilinks]]`.

---

## Local vault sync (LogSeq)

LogSeq doesn't natively sync with GitHub. Recommended desktop workflow:

```bash
git clone https://github.com/your-username/your-vault ~/Documents/my-vault
# Point LogSeq to ~/Documents/my-vault

# After merging a PR:
cd ~/Documents/my-vault && git pull
```

For mobile LogSeq access, sync the local clone to your phone via [Syncthing](https://syncthing.net/).
