#!/bin/bash

# Set the -e option
set -e

# install dependencies
cd ip-b
orbit plan --lock-only --force
orbit install
cd ..

cd ip-a
orbit plan --lock-only --force
orbit install
cd ..

cd ip-c
orbit plan --lock-only --force
# verify DST runs without error
STDOUT=$(orbit tree --root entity_c)

# store the ideal value for later comparison
EXACT="entity_c
├─ dupe
└─ entity_a
   └─ dupe_87678be5e3"

# compare the output with the expected value
if [ "$STDOUT" = "$EXACT" ]; then
    echo "DST Test - PASS"
else
    echo "DST Test - FAIL"
    exit 101
fi
exit 0
