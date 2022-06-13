#!/usr/bin/env python
# ------------------------------------------------------------------------------
# File: evalver.py
# Author: Chase Ruskin
# Abstract:
#   Evaluate the version in the Cargo.toml manifest with the latest version of
#   this branch. A '1' will indicate the current manifest version is larger
#   than the previously tagged version. A '0' indicates otherwise.
# Usage:
#   python evalver.py [--version]
# Options:
#   --version       print the cargo crate manifest version and exit
# ------------------------------------------------------------------------------
import subprocess, sys
import unittest
from typing import List

def is_new_version_higher(lhs: str, rhs: str) -> bool:
    '''Checks if the new version `rhs` is larger than the previous version `lhs`.'''
    lhs = lhs.split('.', 2)
    rhs = rhs.split('.', 2)
    # compare the two versions
    for i in range(0, 3):
        if rhs[i] != lhs[i]:
            return bool(int(rhs[i]) > int(lhs[i]))
    # false if they are equivalent
    return False


def extract_crate_version(contents: List[str]) -> str:
    '''Returns the cargo crate version from the crate's manifest file contents (Cargo.toml).'''
    for line in contents:
        property = line.split('=', 1)
        if(len(property) == 2 and property[0].strip() == 'version'):
            value = property[1].strip()
            if len(value) == 0:
                return None
            # detect what quote is used to wrap value
            q = value[0]
            return value.strip(q)
    return None


def extract_latest_released_version(tags: str) -> str:
    '''Grabs the latest git tag that fits a valid version format: <MAJOR.MINOR.PATCH>.
    Assumes all tags are already available in the current git repository.'''
    tags = tags.strip()
    characters = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.']
    # find highest valid tag
    highest_ver = '0.0.0'
    for tag in tags.splitlines():
        # remove leading 'v'
        if tag.lower().startswith('v'):
            tag = tag[1:]
        # there must be 3 separate values delimited by '.'
        if tag.count('.') != 2:
            continue
        # all characters in a tag must constrain to the given set
        for c in tag:
            if c not in characters:
                break
        else:
            if is_new_version_higher(highest_ver, tag):
                highest_ver = tag
    # found zero tags
    if highest_ver == '0.0.0':
        return None
    else:
        return highest_ver


def main():
    # grab the current requested version from crate manifest
    current_version = None
    with open("./Cargo.toml", 'r') as manifest:
        current_version = extract_crate_version(manifest.readlines())
    if current_version == None:
        exit('error: bad cargo crate manifest file (could not find version)')

    # print the cargo crate manifest version and exit
    if sys.argv.count('--version'):
        print(current_version)
        exit()

    # store the last tagged version
    proc = subprocess.check_output('git tag --list', shell=True)
    last_version = extract_latest_released_version(proc.decode())
    if last_version == None:
        last_version = '0.0.0'

    # compare the two versions
    print(int(is_new_version_higher(last_version, current_version)))
    pass


if __name__ == "__main__":
    main()


class Test(unittest.TestCase):
    def test_cmp(self):
        # larger major value
        self.assertEqual(is_new_version_higher('1.0.0', '2.0.0'), True)
        # equivalent versions
        self.assertEqual(is_new_version_higher('1.20.300', '1.20.300'), False)
        # larger patch value
        self.assertEqual(is_new_version_higher('1.20.300', '1.20.301'), True)
        # larger minor value
        self.assertEqual(is_new_version_higher('1.20.301', '1.21.300'), True)
        # larger major value despite smalled minor and patch values
        self.assertEqual(is_new_version_higher('1.99.99', '2.0.0'), True)
        # larger major value (leading zeros in string)
        self.assertEqual(is_new_version_higher('01.0.0', '02.0.0'), True)
        # same values
        self.assertEqual(is_new_version_higher('0.1.1', '0.1.1'), False)
        pass


    def test_output(self):
        self.assertEqual(int(is_new_version_higher('1.0.0', '1.0.0')), 0)
        self.assertEqual(int(is_new_version_higher('1.0.0', '2.0.0')), 1)
        pass


    def test_extract_released_version(self):
        tags = ''
        self.assertEqual(extract_latest_released_version(tags), None)

        tags = """
tag1
tag2
tag3"""
        self.assertEqual(extract_latest_released_version(tags), None)

        tags = """
v1.0.0
v0.1.9
v0.1.0"""
        self.assertEqual(extract_latest_released_version(tags), '1.0.0')

        tags = """
va.b.c
V2.0.0
v3.0.0
"""
        self.assertEqual(extract_latest_released_version(tags), '3.0.0')

        tags = """
abcd
1.0.0.
02.123456789.00
"""
        self.assertEqual(extract_latest_released_version(tags), '02.123456789.00')
        pass


    def test_extract_version(self):
        # valid manifest
        manifest = [
'[package]',
'name = "orbit"',
'version = "1.0.0"',
'edition = "2018"',

'# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html',

'[dependencies]',
'colored = "2"',
'tokio = { version = "1", features = ["full"] }',
'reqwest = "0.11"',]
        self.assertEqual(extract_crate_version(manifest), '1.0.0')

        # missing version field
        manifest = [
'[package]',
'name = "orbit"',
'edition = "2018"',

'# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html',

'[dependencies]',
'colored = "2"',
'tokio = { version = "1", features = ["full"] }',
'reqwest = "0.11"',]
        self.assertEqual(extract_crate_version(manifest), None)

        # empty version field
        manifest = [
'[package]',
'name = "orbit"',
'version = ""',
'edition = "2018"',

'# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html',

'[dependencies]',
'colored = "2"',
'tokio = { version = "1", features = ["full"] }',
'reqwest = "0.11"',]
        self.assertEqual(extract_crate_version(manifest), '')

        # bunched version field
        manifest = [
'[package]',
'name = "orbit"',
'edition = "2018"',
'version="1.0.0"',
'# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html',

'[dependencies]',
'colored = "2"',
'tokio = { version = "1", features = ["full"] }',
'reqwest = "0.11"',]
        self.assertEqual(extract_crate_version(manifest), '1.0.0')

        # wrap version value in single quotes
        manifest = [
'[package]',
'name = "orbit"',
'edition = "2018"',
'version = \'1.0.0\'',
'# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html',

'[dependencies]',
'colored = "2"',
'tokio = { version = "1", features = ["full"] }',
'reqwest = "0.11"',]
        self.assertEqual(extract_crate_version(manifest), '1.0.0')
        pass
    pass