#!/usr/bin/env python

## File: sync_docs_version.py
## Author: Chase Ruskin
## Details:
##  Searches a md-book page for CARGO_CRATE_VERSION and replaces
##  it with the Crate's version from the manifest. 
##
## Usage:    
##   python sync_docs_version.py <file>
## Args:
##   <file>   a relative filepath to a markdown document
## 

import sys
from evalver import extract_crate_version

if len(sys.argv) != 2:
    exit("ERROR: Invalid number of arguments provided")

# get the file
file = sys.argv[1]
# the series of characters to match and replace with version
CARGO_CRATE_VERSION = 'CARGO_CRATE_VERSION'

current_version = '1.0.0'
with open("./Cargo.toml", 'r') as manifest:
    current_version = extract_crate_version(manifest.readlines())
if current_version is None:
    current_version = '1.0.0'

swap = ''
with open(file, 'r') as f:
    contents = f.read()
    swap = contents.replace(CARGO_CRATE_VERSION, current_version)
    if swap == contents:
        exit("ERROR: No replacement occurred for inserting version number")
    pass

with open(file, 'w') as f:
    f.write(swap)
    pass