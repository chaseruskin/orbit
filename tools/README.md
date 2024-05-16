# /tools

Internal supportive code for the orbit project.

- `autocl.py`: autonomous changelog utlity helper script
- `clgen.py`: creates a temporary changelog file based on the git commits from the current status to the most recent version tag
- `evalver.py`: evaluates the version in the Cargo.toml manifest with the latest version of this branch
- `pack.py`: packages project files into single folder and compresses them using zip archive format for distribution
- `mansync/mangen.py`: generates rust code `str` literal from docs markdown manual page saved to docs/src/6_commands/
- `sum.py`: computes the checksum for a list of files found from glob matching a pattern