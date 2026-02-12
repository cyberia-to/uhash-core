# UHash - Cross-Platform Build System
# Usage: make help

.PHONY: all setup build clean help
.PHONY: setup-rust setup-java setup-android setup-ios
.PHONY: wasm macos ios android
.PHONY: install-ios install-android
.PHONY: test bench lint

# ============================================================================
# Configuration
# ============================================================================

SHELL := /bin/bash
PROJECT_ROOT := $(shell pwd)
DEMO_DIR := $(PROJECT_ROOT)/demo/src-tauri
WEB_DIR := $(PROJECT_ROOT)/web

# Environment
export JAVA_HOME ?= /opt/homebrew/opt/openjdk@17
export ANDROID_HOME ?= $(HOME)/Library/Android/sdk
export NDK_HOME ?= $(ANDROID_HOME)/ndk/26.1.10909125
export PATH := $(JAVA_HOME)/bin:$(ANDROID_HOME)/platform-tools:$(HOME)/.cargo/bin:$(PATH)

# Colors
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# ============================================================================
# Default & Help
# ============================================================================

all: build ## Build all targets (wasm, macos, ios, android)

help: ## Show this help
	@echo "UHash Build System"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Setup:"
	@grep -E '^setup[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "Build:"
	@grep -E '^(wasm|macos|ios|android|build):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "Run:"
	@grep -E '^run[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "Install:"
	@grep -E '^install[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "Development:"
	@grep -E '^(serve|dev|test|bench|lint|clean):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(BLUE)%-20s$(NC) %s\n", $$1, $$2}'

# ============================================================================
# Setup Targets
# ============================================================================

setup: setup-rust setup-java setup-android setup-ios ## Setup all build environments

setup-rust: ## Install Rust toolchain and targets
	@echo -e "$(BLUE)[Setup]$(NC) Rust toolchain..."
	@command -v rustup >/dev/null || (curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y)
	@rustup target add aarch64-apple-ios 2>/dev/null || true
	@rustup target add aarch64-apple-darwin 2>/dev/null || true
	@rustup target add x86_64-apple-darwin 2>/dev/null || true
	@rustup target add aarch64-linux-android 2>/dev/null || true
	@rustup target add wasm32-unknown-unknown 2>/dev/null || true
	@command -v wasm-bindgen >/dev/null || cargo install wasm-bindgen-cli
	@command -v cargo-tauri >/dev/null || cargo install tauri-cli
	@echo -e "$(GREEN)[Done]$(NC) Rust ready"

setup-java: ## Install Java 17 (via Homebrew)
	@echo -e "$(BLUE)[Setup]$(NC) Java..."
	@if [ ! -f "$(JAVA_HOME)/bin/java" ]; then \
		brew install openjdk@17 2>/dev/null || true; \
	fi
	@echo -e "$(GREEN)[Done]$(NC) Java ready"

setup-android: setup-java ## Install Android SDK and NDK
	@echo -e "$(BLUE)[Setup]$(NC) Android SDK..."
	@mkdir -p $(ANDROID_HOME)
	@# Install command line tools if needed
	@command -v sdkmanager >/dev/null || brew install --cask android-commandlinetools 2>/dev/null || true
	@# Accept licenses and install components
	@if [ -f "/opt/homebrew/share/android-commandlinetools/cmdline-tools/latest/bin/sdkmanager" ]; then \
		yes | JAVA_HOME=$(JAVA_HOME) /opt/homebrew/share/android-commandlinetools/cmdline-tools/latest/bin/sdkmanager \
			--sdk_root=$(ANDROID_HOME) --licenses 2>/dev/null || true; \
		JAVA_HOME=$(JAVA_HOME) /opt/homebrew/share/android-commandlinetools/cmdline-tools/latest/bin/sdkmanager \
			--sdk_root=$(ANDROID_HOME) \
			"platform-tools" "platforms;android-34" "build-tools;35.0.0" "ndk;26.1.10909125" 2>/dev/null || true; \
	fi
	@# Create debug keystore if needed
	@if [ ! -f "$(HOME)/.android/debug.keystore" ]; then \
		mkdir -p $(HOME)/.android; \
		keytool -genkey -v -keystore $(HOME)/.android/debug.keystore \
			-storepass android -alias androiddebugkey -keypass android \
			-keyalg RSA -keysize 2048 -validity 10000 \
			-dname "CN=Android Debug,O=Android,C=US" 2>/dev/null || true; \
	fi
	@echo -e "$(GREEN)[Done]$(NC) Android SDK ready"

setup-ios: ## Verify iOS build environment (requires Xcode)
	@echo -e "$(BLUE)[Setup]$(NC) iOS environment..."
	@command -v xcodebuild >/dev/null || (echo -e "$(RED)[Error]$(NC) Xcode not installed" && exit 1)
	@echo -e "$(GREEN)[Done]$(NC) iOS ready"

# ============================================================================
# Build Targets
# ============================================================================

build: wasm macos ios android ## Build all platforms

wasm: setup-rust ## Build WASM package
	@echo -e "$(BLUE)[Build]$(NC) WASM..."
	@mkdir -p $(PROJECT_ROOT)/demo/dist/wasm
	@cd $(WEB_DIR) && cargo build --release --target wasm32-unknown-unknown
	@wasm-bindgen $(WEB_DIR)/target/wasm32-unknown-unknown/release/uhash_web.wasm \
		--out-dir $(PROJECT_ROOT)/demo/dist/wasm --target web
	@echo -e "$(GREEN)[Done]$(NC) WASM: $(PROJECT_ROOT)/demo/dist/wasm/uhash_web_bg.wasm"

macos: setup-rust ## Build macOS app (.dmg)
	@echo -e "$(BLUE)[Build]$(NC) macOS..."
	@cd $(DEMO_DIR) && cargo tauri build
	@echo -e "$(GREEN)[Done]$(NC) macOS: $(DEMO_DIR)/target/release/bundle/dmg/"

ios: setup-rust setup-ios ## Build iOS app (.ipa)
	@echo -e "$(BLUE)[Build]$(NC) iOS..."
	@cd $(DEMO_DIR) && [ -d "gen/apple" ] || cargo tauri ios init
	@# Fix Xcode PATH for cargo
	@if [ -f "$(DEMO_DIR)/gen/apple/uhash-demo.xcodeproj/project.pbxproj" ]; then \
		grep -q 'export PATH=.*cargo' $(DEMO_DIR)/gen/apple/uhash-demo.xcodeproj/project.pbxproj || \
		sed -i '' 's/shellScript = "cargo/shellScript = "export PATH=\\"$$HOME\/.cargo\/bin:$$PATH\\" \&\& cargo/g' \
			$(DEMO_DIR)/gen/apple/uhash-demo.xcodeproj/project.pbxproj; \
	fi
	@cd $(DEMO_DIR) && cargo tauri ios build
	@echo -e "$(GREEN)[Done]$(NC) iOS: $(DEMO_DIR)/gen/apple/build/arm64/"

android: setup-rust setup-android ## Build Android app (.apk)
	@echo -e "$(BLUE)[Build]$(NC) Android..."
	@cd $(DEMO_DIR) && [ -d "gen/android" ] || \
		JAVA_HOME=$(JAVA_HOME) ANDROID_HOME=$(ANDROID_HOME) NDK_HOME=$(NDK_HOME) cargo tauri android init
	@echo "sdk.dir=$(ANDROID_HOME)" > $(DEMO_DIR)/gen/android/local.properties
	@# Build Rust library
	@cd $(DEMO_DIR) && JAVA_HOME=$(JAVA_HOME) ANDROID_HOME=$(ANDROID_HOME) NDK_HOME=$(NDK_HOME) \
		cargo tauri android build --target aarch64 2>&1 | grep -v "WebSocket" || true
	@# Build APK with Gradle
	@cd $(DEMO_DIR)/gen/android && JAVA_HOME=$(JAVA_HOME) ./gradlew --no-daemon \
		-x rustBuildArm64Release -x rustBuildArmRelease -x rustBuildX86Release -x rustBuildX86_64Release \
		assembleArm64Release 2>&1 | tail -10
	@# Sign APK
	@$(MAKE) -s sign-apk
	@echo -e "$(GREEN)[Done]$(NC) Android: $(DEMO_DIR)/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-signed.apk"

sign-apk: ## Sign Android APK with debug keystore
	@APK_DIR=$(DEMO_DIR)/gen/android/app/build/outputs/apk/arm64/release; \
	if [ -f "$$APK_DIR/app-arm64-release-unsigned.apk" ]; then \
		JAVA_HOME=$(JAVA_HOME) $(ANDROID_HOME)/build-tools/35.0.0/apksigner sign \
			--v1-signing-enabled true --v2-signing-enabled true \
			--ks $(HOME)/.android/debug.keystore --ks-pass pass:android \
			--out "$$APK_DIR/app-arm64-release-signed.apk" \
			"$$APK_DIR/app-arm64-release-unsigned.apk"; \
	fi

# ============================================================================
# Run Targets
# ============================================================================

run-macos: macos ## Build and run macOS app
	@echo -e "$(BLUE)[Run]$(NC) macOS..."
	@open "$(DEMO_DIR)/target/release/bundle/macos/UHash Demo.app"

run-ios: ## Build and run iOS app on first available device/simulator
	@echo -e "$(BLUE)[Run]$(NC) iOS..."
	@cd $(DEMO_DIR) && cargo tauri ios dev

run-android: ## Build and run Android app on first available device/emulator
	@echo -e "$(BLUE)[Run]$(NC) Android..."
	@cd $(DEMO_DIR) && JAVA_HOME=$(JAVA_HOME) ANDROID_HOME=$(ANDROID_HOME) NDK_HOME=$(NDK_HOME) \
		cargo tauri android dev

run-web: wasm ## Build WASM and open in browser
	@echo -e "$(BLUE)[Run]$(NC) Web (WASM)..."
	@cd $(PROJECT_ROOT)/demo/dist && python3 -m http.server 8000 &
	@sleep 1 && open http://localhost:8000

# ============================================================================
# Install Targets
# ============================================================================

install-ios: ## Install iOS app to connected device
	@echo -e "$(BLUE)[Install]$(NC) iOS..."
	@IPA=$$(ls $(DEMO_DIR)/gen/apple/build/arm64/*.ipa 2>/dev/null | head -1); \
	if [ -f "$$IPA" ]; then \
		DEVICE=$$(xcrun devicectl list devices 2>/dev/null | grep -o '[0-9A-F\-]\{36\}' | head -1); \
		if [ -n "$$DEVICE" ]; then \
			xcrun devicectl device install app --device "$$DEVICE" "$$IPA"; \
			echo -e "$(GREEN)[Done]$(NC) iOS app installed"; \
		else \
			echo -e "$(RED)[Error]$(NC) No iOS device connected"; \
		fi \
	else \
		echo -e "$(RED)[Error]$(NC) iOS IPA not found. Run 'make ios' first."; \
	fi

install-android: ## Install Android app to connected device
	@echo -e "$(BLUE)[Install]$(NC) Android..."
	@APK=$(DEMO_DIR)/gen/android/app/build/outputs/apk/arm64/release/app-arm64-release-signed.apk; \
	if [ -f "$$APK" ]; then \
		$(ANDROID_HOME)/platform-tools/adb install "$$APK"; \
		echo -e "$(GREEN)[Done]$(NC) Android app installed"; \
	else \
		echo -e "$(RED)[Error]$(NC) Android APK not found. Run 'make android' first."; \
	fi

# ============================================================================
# Development Targets
# ============================================================================

serve: wasm ## Serve web version locally (WASM)
	@echo -e "$(BLUE)[Serve]$(NC) Starting web server at http://localhost:8000"
	@cd $(PROJECT_ROOT)/demo/dist && python3 -m http.server 8000

dev: setup-rust ## Run Tauri dev server
	@cd $(DEMO_DIR) && cargo tauri dev

test: ## Run tests
	@cargo test

bench: ## Run benchmarks
	@cargo bench

lint: ## Run clippy and fmt check
	@cargo fmt --check
	@cargo clippy -- -D warnings

clean: ## Clean build artifacts
	@cargo clean
	@rm -rf $(WEB_DIR)/pkg
	@rm -rf $(DEMO_DIR)/gen/android/app/build
	@rm -rf $(DEMO_DIR)/gen/apple/build
	@echo -e "$(GREEN)[Done]$(NC) Cleaned"
