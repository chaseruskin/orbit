
EXPECTED = ['ent1.vhd', 'arch1.vhd', 'arch2.vhd', 'top.vhd', 'top_arch.vhd']

order = []
with open('./build/blueprint.tsv') as f:
    import os

    for line in f.readlines():
        name = os.path.basename(line.split('\t')[-1]).strip()
        order += [name]
    pass

# Check the order is correct
if order != EXPECTED:
    print('Test- FAIL ' + str(order) + ' /= ' + str(EXPECTED))
    exit(101)
    
print('Test- PASS')
exit(0)
