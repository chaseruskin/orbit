
EXPECTED = ['dep.py', 'top.py', 'dep.sv', 'top.sv']

order = []
with open('./top/target/rr/blueprint.tsv') as f:
    import os

    for line in f.readlines():
        name = os.path.basename(line.split('\t')[-1]).strip()
        order += [name]
    pass

# Check the order is correct
if order != EXPECTED:
    print('TEST: RECURSIVE_FILESETS - FAIL ' + str(order) + ' /= ' + str(EXPECTED))
    exit(101)
    
print('TEST: RECURSIVE_FILESETS - PASS')
exit(0)
