<!--This changelog follows a very particular format. Only the title 'changelog' may begin with 1 pound symbol '#'. Every version partition must begin with 2 pound symbols '##'. Any section under a version must begin wtih 3 pound symbols '###'. This is important for the auto-changelog extraction occuring during the CI/CD pipeline to list only the current verion's changes with every release. -->

# Changelog

## 0.1.0 - unreleased

### Features

- adds support for the ini configuration file format to store key value pairs

- creates home folder at ~/.orbit if `ORBIT_HOME` enviornment variable is not set

- adds `--upgrade` flag for self-updating binary with latest Github release for user's targeted OS and architecture

- adds command-line interface with helpful misspelling suggestions and argument input validation