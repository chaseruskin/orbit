
EXPECTED = ['casting.vhd', 'drivers.vhd', 't1_tb.vhd']

order = []
with open('./app/target/foo/blueprint.tsv') as f:
    import os

    for line in f.readlines():
        name = os.path.basename(line.split('\t')[-1]).strip()
        order += [name]
    pass

# Check the order is correct
if order != EXPECTED:
    print('TEST: MIN_ORDER - FAIL ' + str(order) + ' /= ' + str(EXPECTED))
    exit(101)
    
print('TEST: MIN_ORDER - PASS')
exit(0)
