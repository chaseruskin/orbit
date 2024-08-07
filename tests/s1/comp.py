
EXPECTED = ['ent1.vhd', 'arch1.vhd', 'arch2.vhd', 'top.vhd', 'top_arch.vhd']

order = []
with open('./target/foo/blueprint.tsv') as f:
    import os

    for line in f.readlines():
        name = os.path.basename(line.split('\t')[-1]).strip()
        order += [name]
    pass

# Check the order is correct
if order != EXPECTED:
    print('TEST: ASSOCIATED_FILES - FAIL ' + str(order) + ' /= ' + str(EXPECTED))
    exit(101)
    
print('TEST: ASSOCIATED_FILES - PASS')
exit(0)
