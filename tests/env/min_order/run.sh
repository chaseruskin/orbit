#!/bin/bash

# Set the -e option
set -e

# Install dependencies
cd lib
orbit plan --lock-only --force
orbit install --force
cd ..

# Run application workflow
cd app
orbit plan --force --top t1
cd ..

# Remove dependencies
orbit uninstall lib --full

# Verify the correct behavior occurred without error
python comp.py
