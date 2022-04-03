# ------------------------------------------------------------------------------
# File: clgen.py
# Author: Chase Ruskin
# Abstract:
#   Creates a temporary changelog file based on the git commits from the current
#   status to the most recent version tag.
# Examples:
#   feat: adds auto-changelog script (close #1)
#   mod: asks user for path in initial setup
#   docs: adds section additional help text (#3)
# Usage:
#   pyton clgen.py [--verbose]
# Options:
#   --verbose       print out commits being skipped (no prefix)
# ------------------------------------------------------------------------------
import subprocess
import sys
from evalver import extract_latest_released_version

FILEPATH = "./CHANGELOG_MERGE.md"

prefix_map = {
    'feat'   : 'Features',
    'mod'    : 'Other Changes',
    'fix'    : 'Fixes',
    'docs'   : 'Documentation',
    'test'   : 'Tests', 
    'deprec' : 'Deprecates',
    'remove' : 'Removes',
    'perf'   : 'Other Changes',
    'ci'     : 'Continuous Integration',
}

# only these prefixes will be added to the changelog
include = ['feat', 'fix', 'mod', 'deprec', 'remove', 'docs']

# prepare mapping dictionary with empty lists for each category
mapping = dict()
for p in prefix_map.values():
    mapping[p] = []


def parse(prefix, commit):
    '''Given a commit, try if it has the given prefix. Will remove Github keywords
    that trigger closing the issue ('close', 'resolve', 'fix').'''
    global mapping, prefix_map
    key=prefix+':'
    #find the rightmost parentheses that comes before the issue '#' symbol
    if commit.rfind('#') > commit.rfind('(') > 0:
        subject, issue = commit.rsplit('(', 1)
        # check which closing is used (accepts -s or -ed forms)
        closers = ['close ', 'resolve ', 'fix ', 
                   'closes ', 'resolves ', 'fixes ', 
                   'closed ', 'resolved ', 'fixed ']
        issue = issue.lower()
        for c in closers:
            issue = issue.replace(c, '')
        #recombine the commit together
        commit = subject + '(' + issue

    if commit.startswith(key):
        mapping[prefix_map[prefix]] += [commit.replace(key, '-', 1)]
        return True

    return False


def main():
    verbose = sys.argv.count('--verbose')

    # store the last tagged version
    proc = subprocess.check_output('git tag --list', shell=True)
    last_version = extract_latest_released_version(proc.decode())
    if last_version == None:
        last_version = ''

    # create version range for filtering git commit log
    if len(last_version):
        last_version+='..'

    # gather data about latest commits
    commits = ''
    try:
        proc = subprocess.check_output('git log --pretty=format:"%s" '+last_version, shell=True)
        commits = proc.decode().strip()
    except:
        exit('error: no commits found in the repository')

    # iterate through the available commits
    for c in commits.splitlines(keepends=False):
        if len(c) == 0:
            continue
        for f in prefix_map.keys():
            if parse(f, c):
                break
        else:
            if verbose == True:
                print('warning: skipping \''+c+"\'")
        pass

    # write the data to a changelog file in markdown format
    with open(FILEPATH, 'w') as log:
        empty = True
        for k in include:
            entries = mapping[prefix_map[k]]
            if len(entries):
                log.write('\n### '+prefix_map[k]+'\n')
                empty = False
            for e in entries:
                log.write(e+'\n')
            pass
        if empty:
            log.write("_There are no documented changes for this release._\n\n")

    print('info: Changelog written to:',FILEPATH)


if __name__ == "__main__":
    main()