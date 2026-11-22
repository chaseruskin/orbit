# /s10

This directory hosts code for the relative-dependency duplicate design-unit
semantics (upstream issue #23).

- `cross/`: `package config` exists in both the root (library `demo4`) and a
  path dependency (library `dep4`). Different libraries are distinct
  namespaces, so the design plans and both libraries appear in the blueprint.
- `chain/`: two path-linked projects (mid, leaf) share library `sharedlib`
  and both define `package config` — a true same-library homograph between
  two non-root projects: the planner refuses with the local-dependency error.
- `same/`: the root (library `sharedlib`) and its path dependency both define
  `package config` in the same library: the planner refuses with the
  direct-dependency error.

The `probe` target lives in this directory's regional config
(`.orbit/config.toml`), which orbit discovers by walking parent
directories — so no developer global config is touched, and the target
only echoes: the blueprint is the deliverable.
