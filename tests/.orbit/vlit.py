# Project: Orbit
# Target: vlit
# 
# Simulate verilog code with Verilator.

import subprocess
import os

v_src = []
with open(os.environ.get('ORBIT_BLUEPRINT'), 'r') as fd:
    steps = fd.readlines()
    for step in steps:
        fset, lib, path = step.strip().split('\t')
        if fset == 'VLOG':
            v_src += [path]
        pass

child = subprocess.Popen(['verilator', '--lint-only'] + v_src)
child.wait()