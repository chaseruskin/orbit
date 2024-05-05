# Dock: Create a docker image containing tools and programs required
#   to run HDL simulations.

LABEL maintainer="Chase Ruskin"

# Stage 0: Orbit layer
FROM cdotrus/orbit:latest-ubuntu-18.04

# Stage 1: Python layer
FROM brunneis/python:3.8.0-ubuntu-18.04

# Configure Python for any scripting during simulations
RUN python -m pip install --upgrade pip

COPY --from=0 / /

# Stage 2: FPGA Toolchain layer (ModelSim-Intel)
FROM cdotrus/modelsim-intel:19.1-ubuntu-18.04

COPY --from=1 / /

# Enter the shell
CMD ["bash"]