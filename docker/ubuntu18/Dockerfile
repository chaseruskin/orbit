# download base image ubuntu 18.04
FROM ubuntu:18.04

# Specific commit to use when installing from git
ARG REV

# LABEL about the custom image
LABEL maintainer="crus800"
LABEL description="Orbit HDL package manager on Ubuntu 18.04."

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    libssl-dev \
    curl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Download Rust toolchain, compile, move orbit to bin/, and clean up Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && $HOME/.cargo/bin/cargo install --bin orbit --git https://github.com/c-rus/orbit.git --rev $REV \
    && cp "$HOME/.cargo/bin/orbit" "/usr/local/bin/orbit" \
    && $HOME/.cargo/bin/rustup self uninstall -y

COPY ./entrypoint.sh /entrypoint.sh

# Code file to execute when the docker container starts up (`entrypoint.sh`)
ENTRYPOINT ["/entrypoint.sh"]