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
STDOUT=$(orbit tree --root entity_c)

# store the ideal value for later comparison
EXACT="entity_c
├─ entity_a
│  ├─ dupe_b8da6e529b
│  └─ dupe2_b8da6e529b
└─ dupe"

orbit remove ip-b --force
orbit remove ip-a --force

# compare the output with the expected value
if [ "$STDOUT" = "$EXACT" ]; then
    echo "DST Test - PASS"
else
    echo "DST Test - FAIL"
    echo "--- Expected ---"
    echo "$EXACT"
    echo "--- Received ---"
    echo "$STDOUT"
    exit 101
fi
exit 0
