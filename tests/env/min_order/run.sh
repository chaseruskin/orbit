#!/usr/bin/env bash

# Set the -e option
set -e

# Install dependencies
cd lib
orbit lock --force
orbit install --force
cd ..

# Run application workflow
cd app
orbit test --target foo --force --dut t1
cd ..

# Remove dependencies
orbit remove lib --force

# Verify the correct behavior occurred without error
python comp.py
