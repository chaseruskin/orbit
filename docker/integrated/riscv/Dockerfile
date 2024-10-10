# Dock: Create a docker image containing tools and programs required
#   to run HDL simulations.

# Stage 0: Orbit layer (1st step)
FROM chaseruskin/orbit:latest-ubuntu-18.04

LABEL maintainer="Chase Ruskin"

# Stage 1: Python layer
FROM brunneis/python:3.8.0-ubuntu-18.04

# Configure Python for any scripting during simulations
RUN python -m pip install --upgrade pip

COPY --from=0 / /

# Stage 2: RISCV Toolchain layer
FROM chaseruskin/riscv64:10.2-ubuntu-18.04

COPY --from=1 / /

# Stage 3: FPGA Toolchain layer (ModelSim-Intel)
FROM chaseruskin/modelsim-intel:19.1-ubuntu-18.04

COPY --from=2 / /

# Stage 4: SystemVerilog tools
FROM chaseruskin/sysv:1.0.0-ubuntu-18.04

COPY --from=3 / /

# Enter the shell
CMD ["bash"]