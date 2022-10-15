# download base image ubuntu 18.04
FROM ubuntu:18.04

# LABEL about the custom image
LABEL maintainer="c-rus"
LABEL version="0.1"
LABEL description="This is a custom Docker image for Orbit HDL package manager."


RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*


# copy from action repository to the container
COPY orbit /usr/local/bin/orbit
COPY docker/entrypoint.sh /entrypoint.sh

# Code file to execute when the docker container starts up (`entrypoint.sh`)
ENTRYPOINT ["/entrypoint.sh"]