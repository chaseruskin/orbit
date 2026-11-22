#!/usr/bin/env bash

# Set the -e option
set -e

fail() {
    echo "TEST: STD BLUEPRINT - FAIL"
    echo "$1"
    exit 101
}

# --- case A: declared stds ride every row as the trailing field ---
cd declared/root
orbit lock --force > lock.out 2> lock.err
orbit build --force --target probe --plan tsv > build.out 2> build.err || fail "build failed:
$(cat build.err)"
grep -q "demo4.*config\.vhd	93$" target/probe/blueprint.tsv || fail "the root's row lacks its std 93:
$(cat target/probe/blueprint.tsv)"
grep -q "dep4.*config\.vhd	2008$" target/probe/blueprint.tsv || fail "the dependency's row lacks its std 2008:
$(cat target/probe/blueprint.tsv)"
# json scheme carries the std key too
orbit build --force --target probe --plan json > build.out 2> build.err || fail "json build failed:
$(cat build.err)"
grep -q '"std": "93"' target/probe/blueprint.json || fail "the json blueprint lacks the root's std"
grep -q '"std": "2008"' target/probe/blueprint.json || fail "the json blueprint lacks the dependency's std"
cd ../..

# --- case B: no std anywhere keeps the 3-field blueprint ---
cd undeclared/prj
orbit lock --force > lock.out 2> lock.err
orbit build --force --target probe --plan tsv > build.out 2> build.err || fail "build failed:
$(cat build.err)"
awk -F '\t' 'NF != 3 { print "row with " NF " fields: " $0; bad = 1 } END { exit bad }' target/probe/blueprint.tsv || fail "a blueprint row gained a std field with no std declared"
cd ../..

# --- case C: a std naming the wrong language refuses at planning ---
cd mismatch/prj
orbit lock --force > lock.out 2> lock.err
if orbit build --force --target probe --plan tsv > build.out 2> build.err; then
    fail "a vhdl-only project with std = 2012 should refuse at planning"
fi
grep -q 'unknown std "2012" (expected one of: 87 93 2008 2019)' build.err || fail "expected the vocabulary face, got:
$(cat build.err)"
cd ../..

echo "TEST: STD BLUEPRINT - PASS"
exit 0
