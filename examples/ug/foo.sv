module foo #(
    parameter int WIDTH = 3'd1,
    parameter int BITS = 20,
    parameter ROUNDER = 1.23,
    parameter WORD = "hello world!"
) (
    input logic a = 1'bx,
    input logic b,
    output logic[WIDTH-1:0] c
);

    logic xyz = 'x;

    assign c = a & b;

endmodule