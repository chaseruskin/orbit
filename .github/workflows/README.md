# /workflows

GitHub CI/CD scripts.

- `build.yml`: calls cargo build and cargo test for main orbit executable

- `check-release.yml`: passes a parameter given value if Cargo.toml version field is new

- `docs.yml`: builds gh pages documentation website

- `integrity.yml`: computes and stores checksums for every build artifact

- `pipeline.yml`: glues all sub-workflows together

- `release.yml`: generates release notes along with a released artifact and pushes a new git tag for orbit repository

- `tools.yml`: tests functionality for internal supportive scripts