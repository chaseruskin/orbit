#!/usr/bin/env bash

# Set the -e option
set -e

fail() {
    echo "TEST: CLOSURE MULTI-UNIT - FAIL"
    echo "$1"
    exit 101
}

# A multi-unit file whose first architecture carries processes must still
# contribute its later design units to the plan closure: the dependent
# `gen_mult` entity (instantiated through the package-declared component)
# rides `mult.vhd`, and dropping it starves downstream compiles (black box).
cd proj
orbit lock --force
if ! orbit build --force --target probe --plan tsv > plan.out 2> plan.err; then
    fail "the multi-unit closure should plan, but orbit exited with an error:
$(cat plan.err)"
fi
grep -q "mult.vhd" target/probe/blueprint.tsv || fail "the closure is missing mult.vhd: the unit after a process-bearing architecture was swallowed"
grep -q "techmult.vhd" target/probe/blueprint.tsv || fail "the closure is missing techmult.vhd"
grep -q "top.vhd" target/probe/blueprint.tsv || fail "the closure is missing top.vhd"
cd ..
