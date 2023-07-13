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
import unittest

# the series of characters to match and replace with version
CARGO_CRATE_VERSION = 'CARGO_CRATE_VERSION'


def replace_all(s: str, key: str, value: str) -> str:
    return s.replace(key, value)


def main():
    if len(sys.argv) != 2:
        exit("ERROR: Invalid number of arguments provided")

    # get the file
    file = sys.argv[1]

    current_version = '1.0.0'
    with open("./Cargo.toml", 'r') as manifest:
        current_version = extract_crate_version(manifest.readlines())
    if current_version is None:
        current_version = '1.0.0'

    swap = ''
    with open(file, 'r') as f:
        contents = f.read()
        swap = replace_all(contents, CARGO_CRATE_VERSION, current_version)
        if swap == contents:
            exit("ERROR: No replacement occurred for inserting version number")
        pass

    with open(file, 'w') as f:
        f.write(swap)
        pass
    pass


if __name__ == "__main__":
    main()


class Test(unittest.TestCase):
    def test_replace(self):
        text = """\
### Unix
1. Open a terminal to where Orbit was downloaded.
2. Unzip the prebuilt package.
```
$ unzip orbit-CARGO_CRATE_VERSION-x86_64-macos.zip
```
3. Move the executable to a location already set in the PATH environment variable. 
```
$ mv ./orbit-CARGO_CRATE_VERSION-x86_64-macos/bin/orbit /usr/local/bin/orbit
```"""
        swap = replace_all(text, CARGO_CRATE_VERSION, '1.2.3')
        # replace placeholder with version
        expected = """\
### Unix
1. Open a terminal to where Orbit was downloaded.
2. Unzip the prebuilt package.
```
$ unzip orbit-1.2.3-x86_64-macos.zip
```
3. Move the executable to a location already set in the PATH environment variable. 
```
$ mv ./orbit-1.2.3-x86_64-macos/bin/orbit /usr/local/bin/orbit
```"""
        self.assertEqual(swap, expected)
        pass
    pass