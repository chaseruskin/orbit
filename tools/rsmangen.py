#!/usr/bin/env python
# ------------------------------------------------------------------------------
# File: rsmangen.py
# Author: Chase Ruskin
# Abstract:
#   Generates rust code str literal from docs markdown manual page found under
#   docs/src/6_commands/.
# Usage:    
#   python rsmangen.py
# ------------------------------------------------------------------------------
import os, glob

def main():
    # detect all command files
    commands = glob.glob('./docs/src/6_commands/*.md')

    for cmd in commands:
        name: str = os.path.basename(cmd).rsplit('_')[1].split('.')[0]
        if os.path.basename(cmd).rsplit('_')[0] == '0':
            continue
        first = True
        transform = '// This manual page was automatically generated from the rsmangen.py tool.\npub const MANUAL: &str = "\\\n'
        with open(cmd, 'r') as f:
            for line in f.readlines():
                # remove bullets
                if line.startswith('    - '):
                    line = line[0:3] + line[6:]
                elif line.startswith('      '):
                    line = '  ' + line[6:]
                # add indentation to body text
                transform_line = '    ' + line.replace('`', '')
                # skip title line
                if line.startswith('# '):
                    continue
                if line.startswith('## '):
                    transform_line = line[3:].replace('__', '')
                    if first == False:
                        transform_line = '\n' + transform_line
                    first = False
                # skip code syntax
                elif line.startswith('```'):
                    continue
                elif len(line) == 1:
                    continue
                # ensure quotes are preserved and remove single backslashes
                transform_line = transform_line.replace('\\', '').replace('\"', '\\"')
                transform += transform_line
            pass

        # add closing rust syntax and write to file
        transform += '";'
        output = './src/commands/manuals/'+name+'.rs'
        with open(output, 'w') as f:
            f.write(transform)

        print('info: file written to '+output)
        pass
    pass


if __name__ == "__main__":
    main()