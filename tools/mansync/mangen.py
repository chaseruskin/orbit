# ------------------------------------------------------------------------------
# File      : mangen.py
# Author    : Chase Ruskin
# Details   :
#   Reads a TOML file to write various forms of documentation (markdown, rust).
# Usage     : python mangen.py
# ------------------------------------------------------------------------------
import toml, os, sys

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

CWD, _ = os.path.split(sys.argv[0])
# define the path to the TOML file
INPUT_TOML_PATH = str(CWD)+"/manuals.toml"

# set the location of where to place the MD files
MD_OUTPUT_DIR = './docs/src/commands' # './mansync/md'
# set the location of where to place the Rust files
RS_OUTPUT_DIR = './src/commands/manuals' # './mansync/rs'

# --- Constants ----------------------------------------------------------------

# expected fields for each command
NAME = 'name'
SYNO = 'synopsis'
DESC = 'description'
OPTS = 'option'
EXPS = 'examples'
SUMM = 'summary'

END = '\n'
SECT_END = END+END
INDENT = '    '

# --- Functions ----------------------------------------------------------------

def write_md(table, dest: str, command: str) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Markdown format.
    '''
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
        md.write(table[DESC].strip())
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
        md.write('```\n'+table[EXPS].strip()+'\n```')
        md.write(SECT_END)
        pass

    print('INFO: Markdown manual page available at:', path)
    return 0


def write_rs(table, dest, command) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Rust format.
    '''
    table = table[command]
    
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)
    
    path = dest+'/'+command+'.rs'

    with open(path, 'w') as rs:
        # comment
        rs.write('// This manual page was automatically generated from the mangen.py tool.'+END)
        # variable declaration
        rs.write('pub const MANUAL: &str = r#"NAME'+END)
        # name section 
        rs.write(INDENT+table[NAME]+ ' - '+table[SUMM])
        rs.write(SECT_END)
        # synopsis section
        rs.write('SYNOPSIS'+END)
        rs.write(INDENT+table[SYNO])
        rs.write(SECT_END)
        # description section
        rs.write('DESCRIPTION'+END)
        lines = table[DESC].strip().splitlines()
        for line in lines:
            rs.write(INDENT+line+END)
        rs.write(END)

        # options sections
        if table[OPTS] != None:
            rs.write('OPTIONS'+END)
            for opt in table[OPTS]:
                rs.write(INDENT+opt+END)
                rs.write(INDENT+INDENT+table[OPTS][opt]+END+END)
            pass
        # examples section
        rs.write('EXAMPLES'+END)
        lines = table[EXPS].strip().splitlines()
        for line in lines:
            rs.write(INDENT+line+END)
        rs.write('"#;')
        pass

    print('INFO: Rust manual page available at:', path)
    return 0

# --- Application Code ---------------------------------------------------------

# open the manual data
data = toml.load(INPUT_TOML_PATH)

# compile all documentation
for cmd in COMMANDS:
    if cmd not in data:
        print("WARNING: unknown command", cmd, "in toml document")
        continue
    # output the markdown files
    write_md(data, MD_OUTPUT_DIR, cmd)
    # output the rust files
    write_rs(data, RS_OUTPUT_DIR, cmd)
    pass
