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

test-plan-1:
    just install
    chmod +x ./tests/env/assoc_files/run.sh
    cd ./tests/env/assoc_files; ./run.sh

test-plan-2:
    just install
    chmod +x ./tests/env/min_order/run.sh
    cd ./tests/env/min_order; ./run.sh

test-dst:
    just install
    chmod +x ./tests/env/projects/dst.sh
    cd ./tests/env/projects; ./dst.sh

test-pub: 
    just install
    chmod +x ./tests/env/projects/pub.sh
    cd ./tests/env/projects; ./pub.sh

test-all:
    just test-plan-1
    just test-plan-2
    just test-dst
    just test-dst-local
    just test-pub

# Run a DST test to verify library mappings work when using a simulator
test-dst-local:
    just install
    chmod +x ./tests/env/projects/dst-local.sh
    cd ./tests/env/projects; ./dst-local.sh