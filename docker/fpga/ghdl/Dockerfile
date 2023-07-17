# Dock: Create a docker image containing tools and programs required
#   to run VHDL simulations.

# Stage 0: Orbit layer
FROM crus800/orbit:latest

# Stage 1: Ubuntu layer
FROM brunneis/python:3.8.6-ubuntu-20.04

# Configure Python for any scripting during simulations
RUN python -m pip install --upgrade pip

# Install image dependencies
RUN apt-get update && apt-get install -y \
curl \
unzip

COPY --from=0 / /

# Stage 2: VHDL Simulator layer (GHDL)
FROM ghdl/ghdl:ubuntu22-mcode

COPY --from=1 / /

# Enter the shell
CMD ["bash"]