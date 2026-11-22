# /s12

This directory hosts single-candidate target inference (upstream issue
#25): with no CLI flag, no `ORBIT_TARGET`, no manifest `[toolchain]`
face, and no config default, exactly one target in scope is inferred.

The regional config (`.orbit/config.toml`) defines the single `solo`
target. The live face needs a candidate-free global config (true on a
fresh CI runner); where the developer's global config defines targets,
the script says so loudly and the unit tests carry the proof.
