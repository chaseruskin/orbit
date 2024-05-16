#!/bin/bash

# Set the -e option
set -e

# install dependencies
cd ip-b
orbit plan --lock-only --force
orbit install --force
cd ..

cd ip-a
orbit plan --lock-only --force
orbit install --force
cd ..

cd ip-c
orbit plan --lock-only --force

# verify DST runs without error
STDOUT=$(orbit tree --root entity_c)

# store the ideal value for later comparison
EXACT="entity_c
├─ dupe
└─ entity_a
   └─ dupe_9b17a38c0b"

orbit remove ip-b --all
orbit remove ip-a --all

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
