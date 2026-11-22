# /s11

This directory hosts the tool-selection faces (upstream issue #25).

- `tool/`: a project whose `[toolchain]` face binds `test = "t2"` and
  `build = "t1"` — `orbit test` (no `--target`) runs the bound target.
- `tool/`, case B: `orbit test --target t1` dies at the CLI — `t1`
  declares `test = false`, and the mismatch is named where it happens.
- `mtgt/`: a `[[target]]` array written into Orbit.toml is a parse
  error that teaches where targets belong (the config files; bind one
  per verb in `[toolchain]`).

Targets live in this directory's regional config (`.orbit/config.toml`),
which orbit discovers by walking parent directories — no developer
global config is touched.
