# download base image ubuntu 18.04
FROM ubuntu:22.04

# LABEL about the custom image
LABEL maintainer="Chase Ruskin"
LABEL description="Minimum runtime dependencies with orbit on Ubuntu 22.04."

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# copy from action repository to the container
COPY ../../orbit /usr/local/bin/orbit
COPY ./docker/standalone/ubuntu22/entrypoint.sh /entrypoint.sh

# Code file to execute when the docker container starts up (`entrypoint.sh`)
ENTRYPOINT ["/entrypoint.sh"]