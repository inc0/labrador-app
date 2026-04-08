RUNTIME ?= $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null)

# ── Linux AppImage ─────────────────────────────────────────────────────────────

.PHONY: linux
linux: docker/Dockerfile.linux
	$(RUNTIME) build -f docker/Dockerfile.linux -t labrador-linux .
	$(RUNTIME) run --rm \
		-v "$(CURDIR)":/app:Z \
		-v labrador-node-linux:/app/node_modules \
		-v labrador-cargo-linux:/usr/local/cargo/registry \
		-v labrador-target-linux:/app/src-tauri/target \
		labrador-linux
	@echo "AppImage: src-tauri/target/release/bundle/appimage/"

# ── Android APK + AAB ─────────────────────────────────────────────────────────

.PHONY: android
android: docker/Dockerfile.android
	$(RUNTIME) build -f docker/Dockerfile.android -t labrador-android .
	$(RUNTIME) run --rm \
		-v "$(CURDIR)":/app:Z \
		-v labrador-node-android:/app/node_modules \
		-v labrador-cargo-android:/usr/local/cargo/registry \
		-v labrador-target-android:/app/src-tauri/target \
		-v labrador-gradle:/root/.gradle \
		-e ANDROID_HOME=/opt/android-sdk \
		-e NDK_HOME=/opt/android-sdk/ndk/27.0.12077973 \
		labrador-android
	@echo "APK: src-tauri/gen/android/app/build/outputs/apk/"

# ── Clean volumes ──────────────────────────────────────────────────────────────

.PHONY: clean
clean:
	$(RUNTIME) volume rm -f \
		labrador-node-linux \
		labrador-node-android \
		labrador-cargo-linux \
		labrador-cargo-android \
		labrador-target-linux \
		labrador-target-android \
		labrador-gradle
