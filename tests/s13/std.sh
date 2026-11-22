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

# --- case A2: custom-fileset rows carry the owning project's std ---
cd declared/root
orbit build --force --target probe-cfs --plan tsv > cfs.out 2> cfs.err || fail "custom-fileset build failed:
$(cat cfs.err)"
grep -q "ALLHDL	dep4.*config\.vhd	2008$" target/probe-cfs/blueprint.tsv || fail "the custom-fileset dependency row lacks its std:
$(cat target/probe-cfs/blueprint.tsv)"
grep -q "ALLHDL	demo4.*root_unit\.vhd	93$" target/probe-cfs/blueprint.tsv || fail "the custom-fileset root row lacks its std:
$(cat target/probe-cfs/blueprint.tsv)"
cd ../..

# --- case C2: a strict target refuses the mixed-std graph at the CLI ---
cd declared/root
if orbit build --force --target strict --plan tsv > strict.out 2> strict.err; then
    fail "the strict target should refuse the mixed 93/2008 graph, but it planned"
fi
grep -q 'forbids mixing language standards' strict.err || fail "expected the mixed-std face, got:
$(cat strict.err)"
grep -Eq 'declares: (93 2008|2008 93)$' strict.err || fail "expected both declared stds named, got:
$(cat strict.err)"
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
