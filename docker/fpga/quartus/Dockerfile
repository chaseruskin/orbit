# Dock: Create a docker image containing tools and programs required
#   to run FPGA toolflow (synthesis to bitstream).

# Stage 0: Orbit layer
FROM crus800/orbit:latest

# Stage 1: Python layer
FROM brunneis/python:3.8.6-ubuntu-20.04

# Configure Python for any scripting during simulations
RUN python -m pip install --upgrade pip

COPY --from=0 / /

# Stage 2: FPGA Toolchain layer (Intel Quartus Prime Lite)
FROM crus800/quartus-lite:19.1

COPY --from=1 / /

# Install image dependencies
RUN apt-get update && apt-get install -y \
    curl \
    unzip \
    libglib2.0-0 \
    libsm6 \ 
    libxrender1 \ 
    libxext6

# Export Quartus path to the $PATH environment variable 
ENV PATH "$PATH:/opt/intelFPGA_lite/19.1/quartus/bin"

# Enter the shell
CMD ["bash"]