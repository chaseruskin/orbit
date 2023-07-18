# Dock: Create a docker image containing tools and programs required
#   to run FPGA toolflow (synthesis to bitstream).

# Stage 0: OS layer (Ubuntu 22.04) 
FROM ubuntu:22.04

# LABEL about the custom image
LABEL maintainer="c-rus"
LABEL description="Docker image for FPGA toolchain Intel Quartus Prime Lite 19.1."

# Set the timezone
ENV TZ=US/Eastern
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# Install image dependencies
RUN apt-get update && apt-get -y install \
    tzdata \
    git \
    expect \
    libtcmalloc-minimal4 \
    locales \
    wget \
    curl \
    unzip

# Set LOCALE to UTF8
RUN echo "en_US.UTF-8 UTF-8" > /etc/locale.gen && \
    locale-gen en_US.UTF-8 && \
    /usr/sbin/update-locale LANG=en_US.UTF-8

# Download and install Intel Quartus Prime Lite
RUN mkdir -p /root/quartus && \
    cd /root/quartus && \
    wget -q https://cdrdv2.intel.com/v1/dl/getContent/664524/664610?filename=Quartus-lite-19.1.0.670-linux.tar && \
    tar -xvf Quartus-lite-19.1.0.670-linux.tar && \
    chmod +x Quartus-lite-19.1.0.670-linux.tar && \
    ./QuartusLiteSetup-19.1.0.670-linux.run --installdir /opt/intelFPGA_lite/19.1 --accept_eula 1 --mode unattended --disable-components modelsim_ase && \
    rm -rf /opt/intelFPGA_lite/19.1/nios2eds && \
    rm -rf /root/quartus

# Add Quartus binaries to path 
RUN echo "export PATH="$PATH:/opt/intelFPGA_lite/19.1/quartus/bin"" > ~/.bashrc

# Enter the shell
CMD ["bash"]