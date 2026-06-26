// Cross-language demo: SystemVerilog top-level instantiating a VHDL entity
// from //primitives. The gazelle_orbit plugin should detect that
// `and_gate` resolves to a VHDL-only target and upgrade this package's
// generated rule from `verilog_library` to `hdl_library` at Resolve time.
module and_wrapper(
    input  logic a,
    input  logic b,
    output logic y
);

    and_gate u_and (
        .a(a),
        .b(b),
        .y(y)
    );

endmodule
