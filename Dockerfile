# Multi-stage Dockerfile for WriteMagic
FROM rust:1.75-slim as rust-builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./
COPY core/ core/
COPY ffi/ ffi/

# Build release binaries
RUN cargo build --release --workspace

# Android build stage
FROM gradle:8.5-jdk17 as android-builder

# Install Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=/opt/android-sdk/ndk/25.2.9519653
ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools

# Install Android SDK
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools \
    && cd ${ANDROID_HOME}/cmdline-tools \
    && wget https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip \
    && unzip commandlinetools-linux-9477386_latest.zip \
    && mv cmdline-tools latest \
    && rm commandlinetools-linux-9477386_latest.zip

# Accept licenses and install SDK components
RUN yes | ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager --licenses \
    && ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager \
        "platform-tools" \
        "platforms;android-34" \
        "build-tools;34.0.0" \
        "ndk;25.2.9519653"

# Set working directory
WORKDIR /app

# Copy Android project
COPY android/ android/
COPY --from=rust-builder /app/target/release/ android/app/src/main/jniLibs/

# Build Android APK
RUN cd android && chmod +x gradlew && ./gradlew assembleRelease

# Runtime stage - using distroless for security
FROM gcr.io/distroless/cc-debian12:nonroot as runtime

# Set working directory
WORKDIR /app

# Copy binaries from builder stages with proper ownership
COPY --from=rust-builder --chown=nonroot:nonroot /app/target/release/ /app/bin/
COPY --from=android-builder --chown=nonroot:nonroot /app/android/app/build/outputs/apk/release/ /app/releases/android/

# Already using nonroot user from distroless image

# Health check - using internal health endpoint (no curl in distroless)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/bin/writemagic-server", "--health-check"]

# Expose ports
EXPOSE 8080 3000

# Default command
CMD ["/app/bin/writemagic-server"]

# Development stage
FROM rust:1.75 as development

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    curl \
    wget \
    unzip \
    git \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Install additional Rust tools
RUN cargo install \
    cargo-watch \
    cargo-edit \
    cargo-audit \
    cargo-deny \
    cargo-outdated

# Install Node.js for tooling
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Set working directory
WORKDIR /workspace

# Default command for development
CMD ["cargo", "watch", "-x", "run"]

# CI stage for testing
FROM rust:1.75 as ci

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install testing tools
RUN cargo install cargo-tarpaulin cargo-audit cargo-deny

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Run tests and generate coverage
RUN cargo test --workspace --all-features
RUN cargo tarpaulin --workspace --xml

# Security audit
RUN cargo audit
RUN cargo deny check

# Linting
RUN cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format check
RUN cargo fmt --all -- --check