#!/usr/bin/env bash

# Set the -e option
set -e

# Install dependencies
# ... None

# Run tested workflow
cd fsets
orbit build --force --target ll
cd ..

# Verify the correct behavior occurred without error
python comp.py
