#!/usr/bin/env bash

# Set the -e option
set -e

# install dependencies
cd ip-b
orbit lock --force
orbit install --force
cd ..

cd ip-a
orbit lock --force
orbit install --force
cd ..

cd ip-c
orbit lock --force

# verify DST runs without error
STDOUT=$(orbit tree entity_c)

# store the ideal value for later comparison
EXACT="entity_c
├─ entity_a
│  ├─ dupe_044588b88a
│  └─ dupe2_044588b88a
└─ dupe"

orbit remove ip-b --force
orbit remove ip-a --force

# compare the output with the expected value
if [ "$STDOUT" = "$EXACT" ]; then
    echo "TEST: DST - PASS"
else
    echo "TEST: DST - FAIL"
    echo "--- Expected ---"
    echo "$EXACT"
    echo "--- Received ---"
    echo "$STDOUT"
    exit 101
fi
exit 0
