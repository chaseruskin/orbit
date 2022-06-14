#!/usr/bin/env python
# ------------------------------------------------------------------------------
# File: pack.py
# Author: Chase Ruskin
# Abstract:
#   Packages project files into single folder and compresses them using zip 
#   archive format for distribution.
# Usage:    
#   python pack.py <target>
# Args:
#   <target>        platform-specific name extension
# ------------------------------------------------------------------------------
import shutil, os, sys

def pack(root: str, src: str, dst: str) -> None:
    '''Places the specified file/directory into final package directory. Fails if
    the `src` does not exist.'''
    if os.path.isfile(src) == True:
        # create missing directories for particular file
        if os.path.exists(root+os.path.dirname(dst)) == False:
            os.makedirs(root+os.path.dirname(dst))
        shutil.copy2(src, root+dst)
    elif os.path.isdir(src) == True:
        shutil.copytree(src, root+dst)
    else:
        exit('error: '+src+' does not exist in current filesystem')


def exe(target: str) -> str:
    if target.lower().count('windows') == True: 
        return '.exe'
    else:
        return ''


def main():
    global project
    if len(sys.argv) != 2:
        exit('error: accepts one argument <target>')
    target = sys.argv[1]

    project = 'orbit'
    pkg = project+'-'+target

    root = './target/'+pkg
    # clean and create new directory for packaging
    if os.path.isdir(root) == True:
        shutil.rmtree(root)
    os.mkdir(root)

    # append '.exe' to grab windows executable
    bin = '/'+project+exe()

    # place binary in bin/
    pack(root, './target/release'+bin, '/bin/'+bin)
    # place license at root
    pack(root, './LICENSE', '/LICENSE')
    # place installer at root
    pack(root, './target/release/install'+exe(), '/install'+exe())

    # compress data
    shutil.make_archive(pkg, 'zip', os.path.dirname(root), base_dir=pkg)

    # verify the zipped package exists
    if os.path.exists('./'+pkg+'.zip') == False:
        exit('error: zip package was not created')
    pass


if __name__ == "__main__":
    main()