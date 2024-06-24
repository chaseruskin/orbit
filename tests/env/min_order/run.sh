#!/bin/bash

# Set the -e option
set -e

# Install dependencies
cd lib
orbit plan --target foo --lock-only --force
orbit install --force
cd ..

# Run application workflow
cd app
orbit plan --target foo --force --top t1
cd ..

# Remove dependencies
orbit remove lib --all

# Verify the correct behavior occurred without error
python comp.py
