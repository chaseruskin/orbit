#!/usr/bin/env bash

# Set the -e option
set -e

fail() {
    echo "TEST: RELATIVE DUPLICATES - FAIL"
    echo "$1"
    exit 101
}

# --- case A: cross-library duplicates through a path dependency plan ---
cd cross/root
orbit lock --force
if ! orbit build --force --target probe --plan tsv > plan.out 2> plan.err; then
    fail "cross-library duplicates should plan, but orbit exited with an error:
$(cat plan.err)"
fi
grep -q "demo4" target/probe/blueprint.tsv || fail "the blueprint is missing the root's library demo4"
grep -q "dep4"  target/probe/blueprint.tsv || fail "the blueprint is missing the dependency's library dep4"
cd ../..

# --- case B: same-library duplicates between two non-root projects error ---
cd chain/root
# the duplicate check can fire during lock (it walks the graph too)
orbit lock --force > lock.out 2> lock.err || true
if orbit build --force --target probe --plan tsv > plan.out 2> plan.err; then
    fail "same-library duplicates between two dependency projects should refuse, but orbit planned them"
fi
cat lock.err plan.err > combined.err
grep -q "conflicts with local dependency" combined.err || fail "expected the local-dependency duplicate error, got:
$(cat combined.err)"
cd ../..

# --- case C: same-library duplicates against the root error ---
cd same/root
orbit lock --force > lock.out 2> lock.err || true
if orbit build --force --target probe --plan tsv > plan.out 2> plan.err; then
    fail "same-library duplicates against the root should refuse, but orbit planned them"
fi
cat lock.err plan.err > combined.err
grep -q "conflicts with direct dependency" combined.err || fail "expected the direct-dependency duplicate error, got:
$(cat combined.err)"
cd ../..

echo "TEST: RELATIVE DUPLICATES - PASS"
exit 0
