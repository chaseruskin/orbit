# gazelle_orbit example

A six-package HDL workspace that exercises the `gazelle_orbit` plugin
across VHDL, Verilog, and SystemVerilog with cross-package, cross-library,
and code-generator-output dependencies. Uses [rules_vhdl] and
[rules_verilog] for the generated rule kinds.

[rules_vhdl]: https://github.com/hw-bzl/rules_vhdl
[rules_verilog]: https://github.com/hw-bzl/rules_verilog

## Layout

```
primitives/       VHDL — inverter, and_gate                 (library: primitives)
gates/            VHDL — nand_gate (uses primitives.*),     (library: gates)
                         or_gate
vutils/           Verilog — reg_n, mux2, counter (uses reg_n)
uart_regs/        Code-gen wrapper — verilog_library wrapping
                  system_rdl_library + verilog_system_rdl_library;
                  tags declare orbit_library + extra unit names

cpu/              Top-level (SystemVerilog)
                  alu      : pure SV using vutils.reg_n
                  cpu_top  : :alu + vutils.mux2 + vutils.counter

dsp/              Top-level (Verilog + SV)
                  multiplier : pure Verilog using vutils.reg_n
                  dsp_top    : :multiplier + vutils.counter +
                               uart_regs (module) + uart_regs_pkg (SV import)
```

Both `cpu` and `dsp` share `vutils`; `dsp` additionally consumes the
SystemRDL-generated `uart_regs` library. The plugin resolves every
reference automatically.

> Note: rules_verilog's `verilog_library` requires `VerilogInfo` in its
> deps and rules_vhdl's `vhdl_library` requires `VhdlInfo`, so the two
> ecosystems don't currently mix in a single deps list. The plugin still
> detects mixed-language references in HDL source if you write them — it
> just leaves you on the hook for getting the providers to align (e.g.
> a wrapper rule). The example keeps each top-level project within one
> language so the generated targets build end-to-end.

## Code generators

`uart_regs/` has no `Orbit.toml` — it's a Bazel-only package whose
`verilog_library` rule is hand-authored to wrap a code generator's output.
The example uses [`rules_systemrdl`](https://hw-bzl.github.io/rules_systemrdl/):
`system_rdl_library` compiles the `.rdl` file, `verilog_system_rdl_library`
adapts the PeakRDL output to a Verilog library, and a thin `verilog_library`
wrapper exposes it under a stable name with extra unit tags. The rule lists
extra unit names provided by the generator via
`tags = ["orbit_unit=<name>"]`, so downstream HDL that imports
`uart_regs_pkg` (in addition to instantiating the `uart_regs` module) gets
a single `//uart_regs` dep wired in automatically. See
[`uart_regs/BUILD.bazel`](uart_regs/BUILD.bazel) for the pattern.

Actually building the `:uart_regs` target requires a working
`rules_systemrdl` toolchain (PeakRDL + a Python toolchain — see the
project's docs). `bazel run //:gazelle` doesn't need any of that; it only
reads BUILD files as text and emits the correct deps regardless.

## Try it

```bash
bazel run //:gazelle
```

That's the only command. The plugin handles `orbit lock` recursively for
each project before analyzing it, so you never have to run orbit by hand.

The generated `gates/BUILD.bazel`, `cpu/BUILD.bazel`, and `dsp/BUILD.bazel`
will all contain `deps = [...]` lists that point to the correct cross-
package labels — `//primitives:and_gate`, `//vutils:reg_n`, `:alu`, etc. —
derived from the VHDL `use` clauses and Verilog/SystemVerilog module
instantiations in the source files.

Re-run `bazel run //:gazelle` any time you add/remove design units or
change references; the BUILD files (and lockfiles) will be kept in sync
automatically.
