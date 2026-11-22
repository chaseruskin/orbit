# /s13

This directory hosts the language-standard face in the blueprint
(upstream issue #24).

- `declared/`: the root declares `std = "93"`, its path dependency
  `std = "2008"` — every blueprint row carries its owning project's std
  as a trailing field (tsv) or key (json), mixed across libraries.
- `undeclared/`: no std anywhere — the blueprint stays byte-compatible
  with the 3-field format.
- `mismatch/`: `std = "2012"` on a VHDL-only project refuses at
  planning — the token names the wrong language's vocabulary.
