#!/usr/bin/env bash

# Set the -e option
set -e

cd sub
orbit lock --force
cd ..

cd top
orbit lock --force
# verify the ip dependency graph only has 1 aka version
STDOUT=$(orbit tree -e project)

cd ..

# Since there are paths in this expected output, it may vary from OS to OS (Windows and Linux).
OS=$(uname -s)

# Verify the correct behavior occurred without error
python comp.py "$STDOUT"