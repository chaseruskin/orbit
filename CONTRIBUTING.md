# Contributing

## Writing code

Code should be modular components limited in scope to perform a single operation. When applicable, tests should accompany functional blocks of code to verify the code behaves as intended. Comments should describe what a particular function does, any assumptions outside the code's scope, and any possible errors to take caution for.

## Writing documentation

The Book of Orbit is kept in `docs/src`. There are four main sections to write:
1. _tutorials_ - for users to get their hands dirty and learn by doing
2. _user guide_ - how to do common and popular tasks
3. _topic guide_ - general information about how the program works
4. _reference_ - detailed and exact information

Original man pages for orbit subcommands are written in `docs/src/6_commands/`. The `mangen.py` script converts the formatted .md file into a nicer Rust string literal for a .rs file saved in `src/commands/manuals/`.

## Releasing a new version of orbit

The CI/CD pipeline handles redundant steps in the release process to encourage
fast and incremental development.

As the developer, there are still a few housekeeping tasks to handle when preparing for a release.

During development for the next version, it is extremely helpful to keep the changelog up-to-date with the changes being made to the codebase. To help this process, use the `clgen.py` script to automatically parse recent git commit subjects to help document the changes.

Once the changelog is written, remove the "- unreleased" label (if exists) in the changelog on the upcoming version's line.

Finally, update the `Cargo.toml` file with the new version number.

Push these changes to the remote repository. The CI/CD workflow will be triggered to follow the complete pipeline from build to release. The release will not happen if there is an error in any step in the process.