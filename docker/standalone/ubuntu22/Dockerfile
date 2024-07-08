# download base image ubuntu 18.04
FROM ubuntu:22.04

# LABEL about the custom image
LABEL maintainer="Chase Ruskin"
LABEL description="Minimum runtime dependencies for Orbit on Ubuntu 22.04."

RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    git \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Download Rust toolchain, compile, move orbit to bin/, and clean up Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN $HOME/.cargo/bin/cargo install --bin orbit --git https://github.com/cdotrus/orbit.git --rev $REV
RUN cp "$HOME/.cargo/bin/orbit" "/usr/local/bin/orbit" 
RUN $HOME/.cargo/bin/rustup self uninstall -y

COPY ./docker/standalone/ubuntu22/entrypoint.sh /entrypoint.sh

# Code file to execute when the docker container starts up (`entrypoint.sh`)
ENTRYPOINT ["/entrypoint.sh"]