# ------------------------------------------------------------------------------
# File     : autocl.py
# Author   : Chase Ruskin
# Abstract :
#   Auto-generate a changelog based on the git commits from the current status
#   to the last released version.
# Examples :
#   feat: adds auto-changelog script (close #1)
#   mod: asks user for path in initial setup
#   docs: adds section additional help text (#3)
# Todo     :
#   - [ ] prepend/insert to an existing changelog file tracked by git
#   - [ ] for a github actions release, take all notes past version header up
#           up until next version header
# ------------------------------------------------------------------------------
from asyncio.subprocess import STDOUT
import subprocess
from datetime import date

LOGFILE = "CHANGELOG"

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
    # grab the current upcoming version from crate manifest
    VER = None
    with open("./Cargo.toml", 'r') as manifest:
        for l in manifest.readlines():
            property = l.split('=', 1)
            if(len(property) == 2 and property[0].strip() == 'version'):
                VER = 'v'+property[1].strip().strip('\"')
                break
        pass

    if VER.startswith('v0.0'):
        exit("autocl-error: Crate version is too low for a release -> "+VER)

    # store the last tagged version
    PREV_VER = ''
    try:
        proc = subprocess.check_output('git describe --tags --abbrev=0', shell=True)
        PREV_VER = proc.decode().strip()
    except:
        # do not exit on this error; just allow no range on `git log` call
        # exit('autocl-error: No previous versions detected')
        pass

    # verify the version exists and is not the same as the previous version
    if VER == None:
        exit("autocl-error: Could not find Cargo crate version")
    elif VER == PREV_VER:
        exit("autocl-error: Cannot make changelog due to equal versions -> "+VER)

    # create version range for filtering git commit log
    if len(PREV_VER):
        PREV_VER+='..'

    # gather data about latest commits
    commits = ''
    try:
        proc = subprocess.check_output('git log --pretty=format:"%s" '+PREV_VER, shell=True)
        commits = proc.decode().strip()
    except:
        exit('autocl-error: No commits found in the repository')

    # iterate through the available commits
    for c in commits.splitlines(keepends=False):
        if len(c) == 0:
            continue
        for f in prefix_map.keys():
            if parse(f, c):
                break
        else:
            print('autocl-warning: Skipping commit \"'+c+"\"")
        pass

    logpath ="./"+LOGFILE+".md"

    # write the data to a changelog file in markdown format
    with open(logpath, 'w') as log:
        empty = True
        log.write('# '+VER+' ('+date.strftime(date.today(),'%F')+')\n')
        for k in include:
            entries = mapping[prefix_map[k]]
            if len(entries):
                log.write('\n## '+prefix_map[k]+'\n')
                empty = False
            for e in entries:
                log.write(e+'\n')
            pass
        if empty:
            log.write("_There are no documented changes for this release._\n\n")

    print('autocl-info: Changelog written to:',logpath)


if __name__ == "__main__":
    main()