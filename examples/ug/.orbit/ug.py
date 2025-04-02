

def read_a_bp():
    import os

    entries = []
    with open(os.environ['ORBIT_BLUEPRINT']) as bp:
        for line in bp.readlines():
            entries += [tuple(line.strip().split('\t'))]
    print(entries)
    return entries


def proc_builtin_fs(entries):
    for entry in entries:
        fileset, library, filepath = entry
        if fileset == 'VHDL':
            print('TODO: process vhdl file '+filepath+' into library '+library)
        elif fileset == 'VLOG':
            print('TODO: process verilog file '+filepath+' into library '+library)
        elif fileset == 'SYSV':
            print('TODO: process systemverilog file '+filepath+' into library '+library)

def proc_custom_fs(entries):
    for entry in entries:
        fileset, library, filepath = entry
        if fileset == 'MYSET':
            print('TODO: handle the file '+filepath+' under the custom fileset '+fileset)

def handle_add_args():
    import sys
    is_verbose = False
    show_help = False
    for arg in sys.argv[1:]:
        if arg == '--verbose':
            is_verbose = True
        elif arg == '--help':
            show_help = True
        else:
            print('TODO: handle target argument '+arg)
    
    print(is_verbose, show_help)

def main():
    entries = read_a_bp()
    proc_builtin_fs(entries)
    myset = proc_custom_fs(entries)
    handle_add_args()

if __name__ == "__main__":
    main()