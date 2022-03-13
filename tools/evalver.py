# ------------------------------------------------------------------------------
# File     : evalver.py
# Author   : Chase Ruskin
# Abstract :
#   Evaluate the version in the Cargo.toml manifest with the latest version of
#   this branch. A '1' will indicate the requested manifest version is larger
#   than the previously tagged version. A '0' indicates otherwise.
# ------------------------------------------------------------------------------

import subprocess

# grab the current requested version from crate manifest
VER = '0.0.0'
with open("./Cargo.toml", 'r') as manifest:
    for l in manifest.readlines():
        property = l.split('=', 1)
        if(len(property) == 2 and property[0].strip() == 'version'):
            VER = property[1].strip().strip('\"')
            break
    pass

# store the last tagged version
PREV_VER = '0.0.0'
try:
    proc = subprocess.check_output('git describe --tags --abbrev=0 2> /dev/null', shell=True)
    PREV_VER = proc.decode().strip().lstrip('v')
except:
    pass

ver = VER.split('.', 2)
prev_ver = PREV_VER.split('.', 2)

# compare the two versions
for i in range(0, 3):
    # stop if previous version is larger or the entire versions are equal
    if ver[i] < prev_ver[i] or (i == 2 and ver[i] == prev_ver[i]):
        print('0')
        break
else:
    print('1')
