# Author: Chase Ruskin
# Details:
#   A series of commands to run at convenience.

# Install the debug build in placement of old binary for local system testing
install:
    cargo build
    cp ./target/debug/orbit "$HOME/.cargo/bin/orbit"

# Run partial section of tests by specifying the modules in MODS
codev MODS:
    cargo watch -c -x check -x "test -- {{ MODS }}" --ignore test/data

# Synchronize documentation from TOML file to markdown and Rust
mansync:
    python ./tools/mansync.py

# Generate a summary of changelog notes for the next version release
chlog:
    python ./tools/clgen.py --verbose

# Updates the source code with the license header
lic:
    python ./tools/license.py

# Run the documentation book on the local host for testing before going live
docs:
    mdbook serve ./docs

# Run all the possible tests available for this project
fulltest:
    cargo check
    cargo test
    python -m unittest discover -s ./tools -p '*.py'

# Sort the glossary contents
sortgloss:
    echo "$(python ./tools/sort_gloss.py)" > ./docs/src/glossary.md

# Organize commands required for end-to-end system-level tests

run-sys-tests:
    just test-plan-1
    just test-plan-2
    just test-dst
    just test-pub
    just test-partv

# Run all system tests
test-all:
    just test-plan-1
    just test-plan-2
    just test-dst
    just test-dst-local
    just test-pub
    just test-partv

# Planning stage
test-plan-1:
    chmod +x ./tests/env/assoc_files/run.sh
    cd ./tests/env/assoc_files; ./run.sh

# Planning stage
test-plan-2:
    chmod +x ./tests/env/min_order/run.sh
    cd ./tests/env/min_order; ./run.sh

# Dst algorithm
test-dst:
    chmod +x ./tests/env/projects/dst.sh
    cd ./tests/env/projects; ./dst.sh

# Dst test to verify library mappings work when using a simulator
test-dst-local:
    chmod +x ./tests/env/projects/dst-local.sh
    cd ./tests/env/projects; ./dst-local.sh

# Using 'public' in manifest
test-pub: 
    chmod +x ./tests/env/projects/pub.sh
    cd ./tests/env/projects; ./pub.sh

# Partial version for dependency
test-partv:
    chmod +x ./tests/partialv/test.sh
    cd ./tests/partialv; ./test.sh
