import os
import sys

STDOUT = sys.argv[1]

EXACT = """top:0.1.0
└── sub:0.1.0 ("""+os.path.abspath(os.path.curdir).replace('\\', '/')+'/sub)'

if STDOUT != EXACT:
    print('TEST: RELATIVE DEPENDENCY - FAIL')
    print('--- Expected ---')
    print(EXACT)
    print('--- Received ---')
    print(STDOUT)
    exit(101)

print('TEST: RELATIVE DEPENDENCY - PASS')
exit(0)