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

# # verify DST runs without error
# STDOUT=$(orbit tree --root entity_c)

orbit b --top entity_c --target gsim -- --lint

orbit remove ip-b --all
orbit remove ip-a --all

# must run to completion with no errors
echo "DST Library Match Test - PASS"
exit 0
