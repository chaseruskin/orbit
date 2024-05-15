#!/bin/bash

# Set the -e option
set -e

# install dependencies
cd ip10
orbit plan --lock-only --force
orbit install
cd ..

cd ip11
orbit plan --lock-only --force

# verify it runs without error
STDOUT=$(orbit tree --root top --format long)

# store the ideal value for later comparison
EXACT="top (ip11:0.1.0)
└─ mid (ip10:0.1.0)
   └─ d10 (ip10:0.1.0)"

# compare the output with the expected value
if [ "$STDOUT" != "$EXACT" ]; then
    echo "PUB Test - FAIL"
    echo "--- Expected ---"
    echo "$EXACT"
    echo "--- Received ---"
    echo "$STDOUT"
    exit 101
fi

# verify it runs without error
STDOUT=$(orbit plan --top top)
echo "PUB Test - PASS"

exit 0