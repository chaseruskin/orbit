# download base image ubuntu 18.04
FROM ubuntu:18.04

# LABEL about the custom image
LABEL maintainer="c-rus"
LABEL version="0.1"
LABEL description="This is a custom Docker image for Orbit HDL package manager."

# copy from action repository to the container
COPY /usr/local/bin/orbit /usr/local/bin/orbit
COPY docker/entrypoint.sh /entrypoint.sh

# Code file to execute when the docker container starts up (`entrypoint.sh`)
ENTRYPOINT ["/entrypoint.sh"]