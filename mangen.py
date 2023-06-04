# ------------------------------------------------------------------------------
# File      : mangen.py
# Author    : Chase Ruskin
# Details   :
#   Reads a TOML file to write various forms of documentation (markdown, rust).
# Usage     : python mangen.py
# ------------------------------------------------------------------------------
import tomllib, os

# --- Configurations -----------------------------------------------------------

# define all commands expected to be in documentation
COMMANDS = [
    'new',
    'init',
    'show',
    'read',
    'get',
    'tree',
    'plan',
    'build',
    'search',
    'download',
    'install',
    'env',
    'config',
    'uninstall',
]

# define the path to the TOML file
INPUT_TOML_PATH = "./manuals.toml"

# set the location of where to place the MD files
MD_OUTPUT_DIR = './tmp/md'
# set the location of where to place the Rust files
RS_OUTPUT_DIR = './tmp/rs'

# --- Constants ----------------------------------------------------------------

# expected fields for each command
NAME = 'name'
SYNO = 'synopsis'
DESC = 'description'
OPTS = 'option'
EXPS = 'examples'
SUMM = 'summary'

SECT_END = '\n\n'

# --- Functions ----------------------------------------------------------------

def write_md(table, dest, command) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Markdown format.
    '''
    if command not in table:
        print("WARNING: unknown command", command, "in toml document")
        return 101
    
    table = table[command]
    
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)
    
    path = dest+'/'+command+'.md'

    with open(path, 'w') as md:
        # title
        md.write('# __orbit '+command+'__')
        md.write(SECT_END)
        # name section
        md.write('## __NAME__')
        md.write(SECT_END)
        md.write(table[NAME]+' - '+table[SUMM])
        md.write(SECT_END)
        # synopsis section
        md.write('## __SYNOPSIS__')
        md.write(SECT_END)
        md.write('```\n'+table[SYNO]+'\n```')
        md.write(SECT_END)
        # description section
        md.write('## __DESCRIPTION__')
        md.write(SECT_END)
        md.write(table[DESC])
        md.write(SECT_END)
        # options sections
        if table[OPTS] != None:
            md.write('## __OPTIONS__')
            md.write(SECT_END)
            for opt in table[OPTS]:
                md.write('`'+opt+'`'+'  \n')
                md.write('      '+table[OPTS][opt])
                md.write(SECT_END)
            pass
        # examples section
        md.write('## __EXAMPLES__')
        md.write(SECT_END)
        md.write('```\n'+table[EXPS]+'\n```')
        md.write(SECT_END)
        pass

    print('INFO: manual page available at:', path)
    return 0


def write_rs(table, dest, command) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Rust format.
    '''
    # @todo: implement
    return 0


# --- Application Code ---------------------------------------------------------

# open the manual data
with open(INPUT_TOML_PATH, "rb") as f:
    data = tomllib.load(f)

# compile all documentation
for cmd in COMMANDS:
    # output the markdown files
    write_md(data, MD_OUTPUT_DIR, cmd)
    # output the rust files
    write_rs(data, RS_OUTPUT_DIR, cmd)
    pass
