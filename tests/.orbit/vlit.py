# Project: Orbit
# Target: vlit
# 
# Simulate verilog code with Verilator.

import subprocess
import os

with open(os.environ.get('ORBIT_BLUEPRINT'), 'r') as fd:
    steps = fd.readlines()
    for step in steps:
        fset, lib, path = step.strip().split('\t')
        child = subprocess.Popen(['verilator', '--lint-only', path])
        child.wait()
        pass