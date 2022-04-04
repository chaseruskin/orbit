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

def pack(src: str, dst: str) -> None:
    '''Places the specified file/directory into final package directory. Fails if
    the `src` does not exist.'''
    root = './target/orbit'
    if os.path.isfile(src) == True:
        # create missing directories for particular file
        if os.path.exists(root+os.path.dirname(dst)) == False:
            os.makedirs(root+os.path.dirname(dst))
        shutil.copy2(src, root+dst)
    elif os.path.isdir(src) == True:
        shutil.copytree(src, root+dst)
    else:
        exit('error: '+src+' does not exist in current filesystem')


def main():
    if len(sys.argv) != 2:
        exit('error: accepts one argument <target>')
    target = sys.argv[1]

    project = 'orbit'

    # clean and create new directory for packaging
    if os.path.isdir('./target/'+project) == True:
        shutil.rmtree('./target/'+project)
    os.mkdir('./target/'+project)

    bin = '/'+project 
    # append '.exe' to grab windows executable
    if target.lower().count('windows') == True: bin += '.exe'

    # place binary in bin/
    pack('./target/release'+bin, '/bin/'+bin)
    # place license at root
    pack('./LICENSE', '/LICENSE')

    # compress data
    shutil.make_archive(project+'-'+target, 'zip', './target', base_dir=project)
    pass


if __name__ == "__main__":
    main()