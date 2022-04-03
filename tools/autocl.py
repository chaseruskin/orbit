# ------------------------------------------------------------------------------
# File: autocl.py
# Author: Chase Ruskin
# Abstract:
#   A autonomous changelog utlity helper script. It reads the project's c
#   changelog before releasing a new version to verify it and extract the most
#   recent version's information. If successful, prints the information to 
#   stdout, else exits with error code and error message.
# Steps:
#   1. verifies changelog file exists (./CHANGELOG.md)
#   2. extracts most recent version header from changelog (## ...)
#   3. verifies the verion header's version matches <version>
#   4. verifies the changelog version header is ready for release (finalized)
#   5. ensures there is information listed for the changelog version entry
#   6. prints changelog's most recent version entry information
# Usage:    
#   pyton autocl.py <version>
# Args:
#   <version>   the requested version to release
# ------------------------------------------------------------------------------
import unittest
import sys, os
from typing import List


def is_finalized(version_header: str) -> bool:
    '''Detects if a verion is ready (no 'unreleased' keyword on same line).'''
    return version_header.lower().count('unreleased') == 0


def versions_match(requested_version: str, version_header: str) -> bool:
    '''Verify the current version `requested_version` matches the most recent changelog entry.'''
    # process version_header
    version_header = version_header[2:].strip()
    # stop at first whitespace
    version = version_header.split(maxsplit=1)[0]
    # drop leading 'v'
    if version.lower().startswith('v'):
        version = version[1:]
    return version == requested_version


def get_version_header(contents: List[str]) -> str:
    '''Returns the most recent version entry in the changelog contents. Returns `None` if no entry exists.'''
    for line in contents:
        if line.startswith('## ') == True:
            return line
    return None


def extract_recent_version_changes(contents: List[str]) -> str:
    '''Returns the information about the changes for the top-listed version.'''
    info = ''
    recording = False
    for line in contents:
        if line.startswith('## '):
            recording = not recording
            if recording == False:
                break
        elif recording == True:
            info += line
    info = info.strip()
    return info


def main():
    if len(sys.argv) != 2:
        exit('error: accepts a single argument <version>')

    current_version = sys.argv[1]

    filepath = './CHANGELOG.md'

    if os.path.exists(filepath) == False:
        exit('error: changelog file not found '+filepath)

    with open('./CHANGELOG.md', 'r') as changelog:
        contents = changelog.readlines()
        version_header = get_version_header(contents)

        if version_header == None:
            exit('error: no version entry found in changelog')

        if versions_match(current_version, version_header) == False:
            exit('error: changelog is not sync with Cargo crate manifest version '+current_version)

        if is_finalized(version_header) == False:
            exit('error: most recent changelog version entry '+current_version+' is marked as \'unreleased\'')

        info = extract_recent_version_changes(contents)

        if len(info) == 0:
            exit('error: document changes before releasing version '+current_version)

        print(info)
    pass


if __name__ == "__main__":
    main()


class Test(unittest.TestCase):
    def test_versions_match(self):
        # versions are valid matchings
        self.assertEqual(versions_match('0.1.0', '## 0.1.0'), True)
        self.assertEqual(versions_match('0.1.0', '## 0.1.0 - unreleased'), True)
        self.assertEqual(versions_match('0.1.0', '## v0.1.0 '), True)
        self.assertEqual(versions_match('0.1.0', '## V0.1.0 '), True)
        # versions do not match
        self.assertEqual(versions_match('0.1.0', '## abc '), False)
        self.assertEqual(versions_match('0.1.1', '## 0.1.0'), False)
        pass


    def test_is_finalized(self):
        self.assertEqual(is_finalized('## 0.1.0'), True)
        self.assertEqual(is_finalized('## 0.1.0 - ready'), True)
        self.assertEqual(is_finalized('## 0.1.0 unreleased'), False)
        self.assertEqual(is_finalized('## 0.1.0 UNRELEASED'), False)
        pass


    def test_get_version_header(self):
        # note: these tests assert extracting info without inserting \n because .md file handles it
        contents = """\
# Changelog

## 0.1.1

- change 1
- fix 1

## 0.1.0

- feature 1
- feature 2
"""
        self.assertEqual(get_version_header(contents.splitlines()), '## 0.1.1') 

        # no version entry found
        contents = """\
# Changelog

Here is some text ##.
"""
        self.assertEqual(get_version_header(contents.splitlines()), None) 
        pass


    def test_extract_info(self):
        contents = """\
# Changelog

## 0.1.1

- change 1
- fix 1

## 0.1.0

- feature 1
- feature 2
"""
        self.assertEqual(extract_recent_version_changes(contents.splitlines()), '- change 1- fix 1')

        contents = """\
# Changelog

## 0.1.1
Introduction.
### Changes
- change 1
- fix 1

## 0.1.0

- feature 1
- feature 2
"""
        self.assertEqual(extract_recent_version_changes(contents.splitlines()), """Introduction.### Changes- change 1- fix 1""")

        contents = """\
# Changelog

## 0.1.1
\t\n 

## 0.1.0

- feature 1
- feature 2
"""
        self.assertEqual(extract_recent_version_changes(contents.splitlines()), '')

        contents = """\
# Changelog

## 0.1.0 -

- implements command-line interface
- adds `--upgrade` flag

## 0.0.0"""
        self.assertEqual(extract_recent_version_changes(contents.splitlines()), "- implements command-line interface- adds `--upgrade` flag")
        pass