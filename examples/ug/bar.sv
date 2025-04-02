module bar (
    input logic a,
    input logic b,
    output logic c,
);

    foo ux (
        .a(a),
        .b(b),
        .c(c)
    );

endmodule