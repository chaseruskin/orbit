"""hdl_library: a single rule that bridges VHDL and Verilog/SystemVerilog.

`hdl_library` is the home for design units that mix Verilog/SystemVerilog and
VHDL — either in the same package's sources or via cross-language `deps`. It
provides BOTH `VhdlInfo` and `VerilogInfo` (only the ones whose language is
present), so consumers that walk either provider see the right slice of the
graph.

Cross-language deps stay correct under transitive walking because each
dep's `VhdlInfo.deps` / `VerilogInfo.deps` propagates through this rule's
own providers — unlike the `data = [...]` workaround which is leaf-only and
silently drops same-language transitive sources behind a cross-language ref.

Rule emit policy in `gazelle_orbit`: this rule is emitted whenever a package
contains HDL sources or has cross-language deps. Single-language packages
also get this rule — the kind-name uniformity simplifies the workspace
mental model and BUILD merging logic. (Single-language users who prefer the
language-specific rule names can continue to hand-author `vhdl_library` /
`verilog_library`; the gazelle plugin recognizes them as read-only index
participants via `codegenLibraryKinds`.)

Example (mixed):

    load("@gazelle_orbit//hdl:defs.bzl", "hdl_library")

    hdl_library(
        name = "axis_bridge",
        srcs = [
            "axis_bridge.sv",      # SV consumer
            "axis_reg_slice.vhd",  # VHDL entity instantiated by axis_bridge
        ],
        library = "k2space",       # VHDL library for the .vhd half
        deps = [
            "//cores/amba:axi_lite_pkg",  # vhdl_library label
            "//cores/util:debug_pkg",     # verilog_library label
        ],
    )

A `verilog_library` and a `vhdl_library` can both appear in `deps`; the rule
partitions them by provider type and walks each chain independently.
"""

load("@rules_verilog//verilog:defs.bzl", "VerilogInfo")
load("@rules_vhdl//vhdl:defs.bzl", "VhdlInfo")

_VHDL_EXTS = ("vhd", "vhdl")
_VERILOG_SRC_EXTS = ("v", "sv")
_VERILOG_HDR_EXTS = ("vh", "svh")

def _hdl_library_impl(ctx):
    vhdl_srcs = []
    verilog_srcs = []
    for f in ctx.files.srcs:
        if f.extension in _VHDL_EXTS:
            vhdl_srcs.append(f)
        elif f.extension in _VERILOG_SRC_EXTS:
            verilog_srcs.append(f)
        elif f.extension in _VERILOG_HDR_EXTS:
            # An `.svh` / `.vh` accidentally in srcs (instead of hdrs) is
            # almost always a typo; surface it explicitly.
            fail(("`{tgt}` lists `{f}` in `srcs`. Header files (`.vh`, " +
                  "`.svh`) belong in the `hdrs` attribute.").format(
                tgt = ctx.label,
                f = f.short_path,
            ))
        else:
            fail("`{tgt}` lists unsupported source `{f}` in `srcs`.".format(
                tgt = ctx.label,
                f = f.short_path,
            ))

    verilog_hdrs = list(ctx.files.hdrs)
    data_files = list(ctx.files.data)

    vhdl_dep_infos = [d[VhdlInfo] for d in ctx.attr.deps if VhdlInfo in d]
    verilog_dep_infos = [d[VerilogInfo] for d in ctx.attr.deps if VerilogInfo in d]

    providers = [DefaultInfo(
        files = depset(ctx.files.srcs + verilog_hdrs + data_files),
    )]

    # Emit each language's provider when this target contributes something
    # to it (its own srcs OR a dep chain in that language) OR — for VHDL —
    # when the user explicitly set `library` to a non-default name. The
    # explicit-library case carries the binding to consumers that key
    # Verilog sources by VHDL library: `rules_vunit`'s `vunit_test` reads
    # `module[VhdlInfo].library` as the library it adds Verilog srcs into,
    # falling back to `"work"` (which VUnit rejects as reserved) if no
    # `VhdlInfo` is present. Emitting an empty-srcs/empty-deps `VhdlInfo`
    # here lets a pure-SystemVerilog `hdl_library` with `library =
    # "k2space"` propagate that name so its srcs land in `k2space` instead
    # of `work`.
    has_explicit_vhdl_library = ctx.attr.library and ctx.attr.library != "work"
    if vhdl_srcs or vhdl_dep_infos or has_explicit_vhdl_library:
        providers.append(VhdlInfo(
            srcs = depset(vhdl_srcs),
            data = depset(data_files),
            library = ctx.attr.library,
            standard = ctx.attr.vhdl_standard,
            top_entity = ctx.attr.top_entity,
            deps = depset(
                vhdl_dep_infos,
                order = "postorder",
                transitive = [d.deps for d in vhdl_dep_infos],
            ),
        ))

    if verilog_srcs or verilog_hdrs or verilog_dep_infos:
        providers.append(VerilogInfo(
            srcs = depset(verilog_srcs),
            hdrs = depset(verilog_hdrs),
            includes = depset(ctx.attr.includes),
            data = depset(data_files),
            standard = ctx.attr.verilog_standard,
            top_module = ctx.attr.top_module,
            deps = depset(
                verilog_dep_infos,
                order = "postorder",
                transitive = [d.deps for d in verilog_dep_infos],
            ),
        ))

    return providers

hdl_library = rule(
    doc = ("Mixed-language HDL library. Sources may be `.vhd` / `.vhdl` " +
           "(VHDL) and/or `.v` / `.sv` (Verilog / SystemVerilog). Deps " +
           "may be `vhdl_library`, `verilog_library`, or `hdl_library` " +
           "targets in any combination — each dep's `VhdlInfo` and " +
           "`VerilogInfo` is walked independently."),
    implementation = _hdl_library_impl,
    attrs = {
        "data": attr.label_list(
            doc = ("Data files needed during compilation or simulation. " +
                   "Flows into both `VhdlInfo.data` and `VerilogInfo.data` " +
                   "of this target so the consumer's file-extension dispatch " +
                   "can read each file correctly."),
            allow_files = True,
        ),
        "deps": attr.label_list(
            doc = ("Other HDL libraries this target depends on. Accepts " +
                   "any combination of `vhdl_library` / `verilog_library` / " +
                   "`hdl_library` targets — the rule partitions them by " +
                   "provider type at analysis time."),
            providers = [[VhdlInfo], [VerilogInfo]],
        ),
        "hdrs": attr.label_list(
            doc = "Verilog/SystemVerilog header files (`.vh`, `.svh`).",
            allow_files = [".vh", ".svh"],
        ),
        "includes": attr.string_list(
            doc = ("Verilog/SystemVerilog include search paths. Flows into " +
                   "`VerilogInfo.includes`; ignored for VHDL."),
        ),
        "library": attr.string(
            doc = ("VHDL library name for `.vhd` / `.vhdl` files in `srcs`. " +
                   "Mirrors `vhdl_library.library`. Ignored when this " +
                   "target has no VHDL sources of its own."),
            default = "work",
        ),
        "srcs": attr.label_list(
            doc = ("HDL source files. `.vhd` / `.vhdl` are compiled into " +
                   "the VHDL `library`; `.v` / `.sv` are compiled into the " +
                   "Verilog flat namespace. `.vh` / `.svh` headers belong " +
                   "in `hdrs`, not `srcs`."),
            allow_files = [".vhd", ".vhdl", ".v", ".sv"],
        ),
        "top_entity": attr.string(
            doc = ("VHDL top-entity name. Mirrors `vhdl_library.top_entity`. " +
                   "Ignored when this target has no VHDL sources of its own."),
            default = "",
        ),
        "top_module": attr.string(
            doc = ("Verilog/SystemVerilog top-module name. Mirrors " +
                   "`verilog_library.top_module`. Ignored when this target " +
                   "has no Verilog sources of its own."),
            default = "",
        ),
        "verilog_standard": attr.string(
            doc = ("Verilog/SystemVerilog standard version. Mirrors " +
                   "`verilog_library.standard`. Empty string means not " +
                   "specified; consumer rules apply their default."),
            default = "",
            values = ["", "1995", "2001", "2005", "2009", "2012", "2017", "2023"],
        ),
        "vhdl_standard": attr.string(
            doc = ("VHDL standard version. Mirrors `vhdl_library.standard`. " +
                   "Empty string means not specified; consumer rules apply " +
                   "their default."),
            default = "",
            values = ["", "1993", "2000", "2002", "2008", "2019"],
        ),
    },
)
