# WriteMagic Development Container
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Set environment variables
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=/opt/android-sdk/ndk/25.2.9519653
ENV PATH=${PATH}:${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64

# Install system dependencies
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        # Build essentials
        build-essential \
        pkg-config \
        libssl-dev \
        libsqlite3-dev \
        # Java 17 for Android development
        openjdk-17-jdk \
        # Additional tools
        curl \
        wget \
        unzip \
        git \
        vim \
        tmux \
        htop \
        jq \
        tree \
        # Network tools
        netcat \
        telnet \
        # For cross-compilation
        gcc-multilib \
        g++-multilib \
        libc6-dev-i386 \
    && apt-get autoremove -y && apt-get clean -y && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain and targets
USER vscode
RUN rustup update stable \
    && rustup component add rustfmt clippy \
    && rustup target add \
        aarch64-linux-android \
        armv7-linux-androideabi \
        i686-linux-android \
        x86_64-linux-android \
        aarch64-apple-ios \
        x86_64-apple-ios \
        aarch64-apple-ios-sim

# Install Rust development tools
RUN cargo install \
    cargo-edit \
    cargo-audit \
    cargo-deny \
    cargo-outdated \
    cargo-udeps \
    cargo-tarpaulin \
    cargo-watch \
    cargo-expand \
    cross

# Install Android SDK and NDK
USER root
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools \
    && cd ${ANDROID_HOME}/cmdline-tools \
    && wget https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip \
    && unzip commandlinetools-linux-9477386_latest.zip \
    && mv cmdline-tools latest \
    && rm commandlinetools-linux-9477386_latest.zip \
    && chown -R vscode:vscode ${ANDROID_HOME}

USER vscode
RUN yes | ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager --licenses \
    && ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager \
        "platform-tools" \
        "platforms;android-34" \
        "platforms;android-33" \
        "platforms;android-29" \
        "build-tools;34.0.0" \
        "ndk;25.2.9519653" \
        "cmake;3.22.1" \
        "emulator" \
        "system-images;android-29;google_apis;x86_64"

# Install Node.js for additional tooling
USER root
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Install Kotlin command-line compiler
RUN cd /opt \
    && wget https://github.com/JetBrains/kotlin/releases/download/v1.9.22/kotlin-compiler-1.9.22.zip \
    && unzip kotlin-compiler-1.9.22.zip \
    && rm kotlin-compiler-1.9.22.zip \
    && ln -s /opt/kotlinc/bin/kotlin /usr/local/bin/kotlin \
    && ln -s /opt/kotlinc/bin/kotlinc /usr/local/bin/kotlinc

# Install pre-commit
RUN pip3 install pre-commit

# Install additional development tools
RUN npm install -g \
    @commitlint/cli \
    @commitlint/config-conventional \
    markdownlint-cli \
    yaml-lint

# Install mobile development tools
RUN curl -L https://github.com/pinterest/ktlint/releases/download/0.50.0/ktlint -o /usr/local/bin/ktlint \
    && chmod +x /usr/local/bin/ktlint

# Install Docker Compose
RUN curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose \
    && chmod +x /usr/local/bin/docker-compose

# Create workspace directory and set permissions
RUN mkdir -p /workspace \
    && chown -R vscode:vscode /workspace

# Switch back to vscode user
USER vscode

# Set up shell environment
RUN echo 'export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools' >> ~/.bashrc \
    && echo 'export ANDROID_HOME=/opt/android-sdk' >> ~/.bashrc \
    && echo 'export ANDROID_NDK_HOME=/opt/android-sdk/ndk/25.2.9519653' >> ~/.bashrc \
    && echo 'export JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64' >> ~/.bashrc \
    && echo 'alias ll="ls -la"' >> ~/.bashrc \
    && echo 'alias la="ls -A"' >> ~/.bashrc \
    && echo 'alias l="ls -CF"' >> ~/.bashrc

# Set working directory
WORKDIR /workspace

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD rustc --version && java -version && echo "Container is healthy"

# Default command
CMD ["/bin/bash"]