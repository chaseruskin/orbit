# Contributing

## Writing code

todo

## Writing documentation

todo 

## Releasing a new version of orbit

The CI/CD pipeline handles redundant steps in the release process to encourage
fast and incremental development.

As the developer, there are still a few housekeeping tasks to handle when preparing for a release.

During development for the next version, it is extremely helpful to keep the changelog up-to-date with the changes being made to the codebase. To help this process, use the `clgen.py` script to automatically parse recent git commit subjects to help document the changes.

Once the changelog is written, remove the "- unreleased" label (if exists) in the changelog on the upcoming version's line.

Finally, update the `Cargo.toml` file with the new version number.

Push these changes to the remote repository. The CI/CD workflow will be triggered to follow the complete pipeline from build to release. The release will not happen if there is an error in any step in the process.