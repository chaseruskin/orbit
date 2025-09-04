
EXPECTED = ['b.txt', 'c.txt', 'a.txt', 'd.txt', 'a.txt', 'a.txt', 'foo.sv']

order = []
with open('./fsets/target/ll/blueprint.tsv') as f:
    import os

    for line in f.readlines():
        name = os.path.basename(line.split('\t')[-1]).strip()
        order += [name]
    pass

# Check the order is correct
if order != EXPECTED:
    print('TEST: CUSTOM_FILESETS - FAIL\nrec: ' + str(order) + '\nexp: ' + str(EXPECTED))
    exit(101)
    
print('TEST: CUSTOM_FILESETS - PASS')
exit(0)
