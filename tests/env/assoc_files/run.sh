#!/bin/bash

# Set the -e option
set -e

# Install dependencies
# ... None

# Run tested workflow
cd .
orbit plan --force

# Verify the correct behavior occurred without error
python comp.py
