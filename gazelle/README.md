# gazelle_orbit

A [Gazelle](https://github.com/bazel-contrib/bazel-gazelle) language
extension that generates `vhdl_library` and `verilog_library` Bazel
targets from HDL sources by delegating parsing and dependency analysis
to the [orbit](https://github.com/chaseruskin/orbit) toolchain.

One `bazel run //:gazelle` walks your workspace, runs `orbit analyze`
under the hood, and writes per-package `BUILD.bazel` files whose `srcs`
and `deps` lists reflect the entities, modules, packages, `use` clauses,
and instantiations in your HDL. Cross-package and cross-library
references resolve automatically through a workspace-wide index.

A working example lives in [`//examples/bazel`](../examples/bazel) — six
packages across VHDL, Verilog, SystemVerilog, with code-gen integration
via [rules_systemrdl].

[rules_systemrdl]: https://hw-bzl.github.io/rules_systemrdl/

## Quick start

In your `MODULE.bazel`:

```python
bazel_dep(name = "gazelle", version = "0.51.0")
bazel_dep(name = "gazelle_orbit", version = "<latest>")
bazel_dep(name = "rules_vhdl", version = "<latest>")
bazel_dep(name = "rules_verilog", version = "<latest>")
```

In your root `BUILD.bazel`:

```python
load("@gazelle//:def.bzl", "gazelle")

gazelle(
    name = "gazelle",
    gazelle = "@gazelle_orbit",
)
```

Then:

```bash
bazel run //:gazelle
```

That's the entire wiring. The `orbit` CLI is bundled as a runfile of the
plugin — no separate install, no `--orbit_bin` flag, no `PATH` setup.
The plugin also recursively runs `orbit lock` on every Orbit project it
encounters, so you never have to invoke orbit by hand either.

## How dependencies are resolved

Each HDL source is parsed for its design units and references:

| Source construct                     | Becomes a Bazel dep            |
| ------------------------------------ | ------------------------------ |
| VHDL `use lib.pkg;`                  | label that provides `lib.pkg`  |
| VHDL `entity lib.entity_name`        | label that provides `lib.entity_name` |
| VHDL `use work.pkg;`                 | label in the *same* HDL library |
| Verilog `mod_name u_inst(...)`       | label that provides `mod_name` |
| SystemVerilog `import pkg::*;`       | label that provides `pkg`      |

Resolution order for each import:

1. Gazelle's built-in `# gazelle:resolve` directive (see below).
2. Library-qualified workspace index lookup (`<library>.<unit>`).
3. Unqualified workspace index lookup (`<unit>`).

The first matching workspace rule wins. For the index lookups, every
rule's library + unit names are registered via [`Imports()`](resolve.go),
which reads them from a public `library` attr, a plugin-stashed private
attr, or a tag (see "Hand-authored rules" below).

## Rule kinds

The plugin generates two kinds:

| Kind             | Loaded from                       | Has `library` attr? |
| ---------------- | --------------------------------- | ------------------- |
| `vhdl_library`   | `@rules_vhdl//vhdl:defs.bzl`      | Yes (defaults to `work`) |
| `verilog_library`| `@rules_verilog//verilog:defs.bzl`| No — Verilog has a flat module namespace |

Generated rules also carry `visibility = ["//visibility:public"]` by
default. User overrides stick on subsequent runs (visibility isn't in
the plugin's merge-attrs list).

Every plugin-generated rule carries the bare `gazelle_orbit` tag so
auto-managed targets are obvious in BUILD files and easy to query:

```bash
bazel query 'attr(tags, "\bgazelle_orbit\b", //...)'
```

User-added tags survive subsequent gazelle runs — the plugin
union-merges its marker with whatever's already on the rule. Distinct
from the prefixed tags `orbit_library=…` and `orbit_unit=…`, which carry
metadata on *hand-authored* rules (see "Hand-authored rules" below).

The plugin also recognizes the following hand-authored rule kinds as
read-only library providers (they get indexed but never modified or
generated):

* `verilog_system_rdl_library` (from [rules_systemrdl])

If you have another codegen ruleset whose outputs you want gazelle_orbit
to index directly, open an issue — adding more is one line in
`codegenLibraryKinds`.

## Hand-authored rules (code generators, vendor IP, …)

Two mechanisms tell gazelle_orbit how to wire a dep to a rule it didn't
generate itself: **directives** (canonical) and **tags** (ergonomic).
Either alone is sufficient; the most idiomatic Gazelle pattern is the
directive.

### Canonical: `# gazelle:resolve` directive

[Gazelle's built-in](https://github.com/bazel-contrib/bazel-gazelle/blob/master/Directives.md)
`# gazelle:resolve` directive is the standard way across all Gazelle
plugins to override import resolution. We consume it for the `orbit`
language:

```python
# Workspace-root BUILD.bazel (or any directory above the consumers)
# gazelle:resolve orbit uart_regs       //uart_regs:uart_regs_verilog
# gazelle:resolve orbit uart_regs_pkg   //uart_regs:uart_regs_verilog
# gazelle:resolve orbit ieee.std_logic_1164  @ext_libs//vhdl:ieee_std_logic_1164
```

Format: `# gazelle:resolve orbit <import> <label>`.

The `<import>` key matches the way the reference appears in source:

| Source style                          | Use this `<import>` form |
| ------------------------------------- | ------------------------ |
| VHDL `use ieee.std_logic_1164;`       | `ieee.std_logic_1164` (library-qualified) |
| Verilog `mod_name u_inst (…)`         | `mod_name` (unqualified) |
| SystemVerilog `import pkg_name::*;`   | `pkg_name` (unqualified) |

Directives are inherited downward from the directory they live in. Put
them at the workspace root for cross-cutting overrides (vendor IP,
standard libraries, codegen outputs anywhere in the tree). Put them in a
subdirectory's BUILD.bazel to scope them to that subtree.

When multiple imports resolve to the same label (e.g. a SystemRDL block
that emits both a module and a package), one directive per import. Yes,
that means typing the label twice, but every Gazelle plugin works this
way — users familiar with other Gazelle setups will pattern-match
instantly.

### Ergonomic: tags on the rule

When you'd rather keep the unit list co-located with the rule than
repeat the label N times in directives, attach `orbit_unit` / `orbit_library`
entries to the rule's `tags`:

```python
verilog_system_rdl_library(
    name = "uart_regs_verilog",
    lib = ":uart_regs",
    tags = [
        # The HDL library this rule belongs to. Required for
        # verilog_library / verilog_system_rdl_library to participate
        # in library-qualified lookups (rules_verilog has no `library`
        # attr). vhdl_library has a public `library` attr; use that.
        "orbit_library=uart_regs",

        # Extra design unit names beyond the rule's own `name`. Useful
        # when a single rule provides multiple units (e.g. SystemRDL
        # emits both a `uart_regs` module and a `uart_regs_pkg` package).
        # The rule's `name` is always indexed implicitly.
        "orbit_unit=uart_regs",
        "orbit_unit=uart_regs_pkg",
    ],
)
```

Downstream HDL that says `import uart_regs_pkg::*;` or
`uart_regs u_inst(...)` resolves to `//uart_regs:uart_regs_verilog`
automatically — both refs collapse into a single dep.

Tags have the advantage that the data travels with the rule (rename or
relocate the rule and the unit list moves with it). The trade-off is
they're a non-standard Gazelle mechanism — users coming from other
plugins won't recognize the `orbit_unit=` convention without docs.

### Which to use

| Use directives when…                       | Use tags when…                          |
| ------------------------------------------ | --------------------------------------- |
| The label is external (vendor IP, stdlib)  | The rule is in your workspace and you control its BUILD file |
| You want cross-cutting overrides at the workspace root | The data is per-rule and shouldn't apply to siblings |
| You're following canonical Gazelle patterns | You want the unit list to move/rename with the rule |
| You have only one or two imports per label | You have many units provided by one rule (avoids label repetition) |

## Other directives

| Directive                          | Effect |
| ---------------------------------- | ------ |
| `# gazelle:orbit_disable true`     | Skip rule generation in this subtree |
| `# gazelle:orbit_library <name>`   | Override the HDL library name reported by orbit for units in this subtree |
| `# gazelle:orbit_ignore <unit>...` | Skip the named design units when generating rules |

## How it works

1. **Per-directory traversal.** Gazelle visits each directory in
   dependency order. The plugin first ensures a fresh `Orbit.lock`
   exists for the project (locking path-based deps depth-first), then
   invokes `orbit analyze --json` and parses the result.

2. **Bucketed rule generation.** Units that share a source-file set
   (e.g. a VHDL entity + its architecture in different files, or two
   Verilog modules in the same `.v` file) collapse into one rule whose
   `name` is one of the units; the rest are recorded as extra unit
   names in a private attr so `Imports()` can index them all.

3. **Workspace-wide indexing.** Gazelle calls `Imports()` on every rule
   of a known kind across the workspace. The plugin indexes each unit
   twice — unqualified (`<unit>`) and qualified (`<library>.<unit>`).

4. **Resolution.** For every rule the plugin generated, it resolves
   each detected reference through `# gazelle:resolve` overrides first,
   then the library-qualified index, then the unqualified index, and
   writes the resulting labels to `deps`.
