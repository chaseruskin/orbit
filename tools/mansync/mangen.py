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
RS_OUTPUT_DIR = './src/commands/manuals' # './mansync/rs/man'
RS_HELP_OUTPUT_DIR = './src/commands/helps' # './mansync/rs/help

# --- Constants ----------------------------------------------------------------

# expected fields for each command
NAME = 'name'
SYNO = 'synopsis'
DESC = 'description'
OPTS = 'options'
ARGS = 'args'
EXPS = 'examples'
SUMM = 'summary'
HELP = 'help'

END = '\n'
SECT_END = END+END
INDENT = ' ' * 4

# --- Functions ----------------------------------------------------------------

def is_populated(table, key: str) -> bool:
    '''
    Checks a TOML `table` for if the given `key` is a valid entry with some value.
    '''
    return key in table and table[key] != None


def write_md_manual(table, dest: str, command: str) -> int:
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

        # options section
        if is_populated(table, OPTS) == True or is_populated(table, ARGS) == True:
            md.write('## __OPTIONS__')
            md.write(SECT_END)
            pass

        if is_populated(table, ARGS) == True:
            for opt in table[ARGS]:
                md.write('`'+opt+'`'+'  \n')
                md.write('      '+table[ARGS][opt])
                md.write(SECT_END)
            pass
        if is_populated(table, OPTS) == True:
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
    return 1


def write_rs_manual(table, dest, command) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Rust format.
    '''
    table = table[command]
    
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)

    # check if added to mod.rs
    mod_exists = False
    with open(dest+'/'+'mod.rs', 'r') as mod:
        mod_exists = mod.read().count('pub mod '+command+';') > 0
    if mod_exists == False:
        with open(dest+'/'+'mod.rs', 'a') as mod:
            mod.write('pub mod '+command+';')
        pass
    
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
            # flip any grave ticks to single quotes
            rs.write(INDENT+line.replace('`', '\'')+END)
        rs.write(END)

        # options section
        if is_populated(table, OPTS) == True or is_populated(table, ARGS) == True:
            rs.write('OPTIONS'+END)
            pass

        if is_populated(table, ARGS) == True:
            for opt in table[ARGS]:
                rs.write(INDENT+opt+END)
                rs.write(INDENT+INDENT+table[ARGS][opt]+END+END)
            pass
        if is_populated(table, OPTS) == True:
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
    return 1


def write_rs_help(table, dest, command) -> int:
    '''
    Writes the TOML `table` for a particular `command` to the `dest` folder in
    Rust format as quick help.
    '''
    table = table[command]

    if is_populated(table, HELP) == False:
        print('WARNING: No help text for command:', command)
        return 0
    
    # fill in intermediate directory structure
    os.makedirs(dest, exist_ok=True)

    # check if added to mod.rs
    mod_exists = False
    with open(dest+'/'+'mod.rs', 'r') as mod:
        mod_exists = mod.read().count('pub mod '+command+';') > 0
    if mod_exists == False:
        with open(dest+'/'+'mod.rs', 'a') as mod:
            mod.write('pub mod '+command+';')
        pass

    path = dest+'/'+command+'.rs'

    with open(path, 'w') as rs:
        # comment
        rs.write('// This help page was automatically generated from the mangen.py tool.'+END)
        # variable declaration
        rs.write('pub const HELP: &str = r#"')
        # quick help body
        rs.write(table[HELP].strip())
        # add closing remark
        rs.write(SECT_END+"Use 'orbit help "+command+"' to read more about the command."+END)
        rs.write('"#;')
        pass

    print('INFO: Rust help page available at:', path)
    return 1


# --- Application Code ---------------------------------------------------------

def main():
    # open the manual data
    data = toml.load(INPUT_TOML_PATH)
    
    score = 0
    total = 3 * len(COMMANDS)

    # compile all documentation
    for cmd in COMMANDS:
        if cmd not in data:
            print("ERROR: Missing command in TOML file: '"+cmd+"'")
            print()
            continue
        # output the markdown files
        score += write_md_manual(data, MD_OUTPUT_DIR, cmd)
        # output the rust files
        score += write_rs_manual(data, RS_OUTPUT_DIR, cmd)
        # output the help files
        score += write_rs_help(data, RS_HELP_OUTPUT_DIR, cmd)
        print()
        pass

    print('DOCUMENTATION SCORE:', score, '/', total)


if __name__ == '__main__':
    main()