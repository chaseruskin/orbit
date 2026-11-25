# /s14

This directory hosts the plan-closure system test for multi-unit files
whose architectures carry processes.

- `libx/`: a technology-mapper-shaped library. `rtl/mult.vhd` holds two
  entity/architecture pairs; the first architecture carries processes, so
  a parser that loses track of the primary-end nesting swallows the
  second pair. The second entity (`gen_mult`) is the one the next file
  instantiates — through the package-declared component in
  `rtl/allmul.vhd` (`use techmap.allmul.all`), the GRLIB default-binding
  shape.
- `proj/`: a design project with a path dependency on `libx`; its
  top -> techmult chain must pull `mult.vhd` into the build-plan
  closure (build plans exclude testbenches, so `top.vhd` is the
  deepest asserted row).

Before the fix the blueprint dropped `mult.vhd` (the entity after the
process-bearing architecture vanished from the graph), leaving
downstream compiles to fail on an unbound component.

The `probe` target lives in this directory's regional config
(`.orbit/config.toml`), which orbit discovers by walking parent
directories — so no developer global config is touched, and the target
only echoes: the blueprint is the deliverable.
