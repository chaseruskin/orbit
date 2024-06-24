#!/bin/bash

# Set the -e option
set -e

orbit config --append include="$PWD/.orbit/config.toml"

# install dependencies
cd ip-b
orbit plan --lock-only --force --target foo
orbit install --force
cd ..

cd ip-a
orbit plan --lock-only --force --target foo
orbit install --force
cd ..

cd ip-c
orbit plan --lock-only --force --target foo

# verify DST runs without error
STDOUT=$(orbit tree --root entity_c)

# store the ideal value for later comparison
EXACT="entity_c
├─ dupe
└─ entity_a
   └─ dupe_9b17a38c0b"

orbit remove ip-b --all
orbit remove ip-a --all

orbit config --pop include

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
