# Dock: Create a docker image containing tools and programs required
#   to run FPGA toolflow (synthesis to bitstream).

# Stage 0: Orbit layer
FROM crus800/orbit:latest

# Stage 1: Ubuntu layer
FROM brunneis/python:3.8.6-ubuntu-20.04

# Configure Python for any scripting by toolchain
RUN python -m pip install --upgrade pip

# Install image dependencies
RUN apt-get update && apt-get install -y \
curl \
unzip

COPY --from=0 / /

# Stage 2: FPGA Toolchain (Intel Quartus Prime)
FROM chriz2600/quartus-lite:19.1.0

COPY --from=1 / /

# Add Quartus binaries to path 
RUN echo "export PATH="$PATH:/opt/intelFPGA_lite/19.1/quartus/bin"" > ~/.bashrc

# Enter the shell
CMD ["bash"]