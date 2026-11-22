#!/usr/bin/env bash

# Set the -e option
set -e

fail() {
    echo "TEST: TARGET FACES - FAIL"
    echo "$1"
    exit 101
}

# --- case A: the manifest toolchain face binds the test verb ---
cd tool
STDOUT=$(orbit test 2>&1) || fail "orbit test with the [toolchain] face failed:
$STDOUT"
echo "$STDOUT" | grep -q "testing" || fail "orbit test did not run the bound t2 target, got:
$STDOUT"

# --- case B: a wrong-verb pick dies loudly at the CLI ---
if orbit test --target t1 > pick.out 2> pick.err; then
    fail "orbit test --target t1 should refuse at the CLI, but it ran"
fi
grep -q "is not a test target" pick.err || fail "expected the wrong-verb face, got:
$(cat pick.err)"
cd ..

# --- case C: [[target]] in the manifest teaches where targets belong ---
cd mtgt
if orbit info > info.out 2> info.err; then
    fail "a [[target]] array in Orbit.toml should be a parse error, but orbit accepted it"
fi
grep -q "unknown field \`target\`" info.err || fail "expected the unknown-field parse error, got:
$(cat info.err)"
grep -q "bind one per verb in a \[toolchain\] section" info.err || fail "expected the teaching hint, got:
$(cat info.err)"
cd ..

echo "TEST: TARGET FACES - PASS"
exit 0
