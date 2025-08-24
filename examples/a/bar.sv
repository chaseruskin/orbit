module bar #(
    parameter P1 = 1,
    parameter P2 = 2,
    parameter P3,
    parameter P4 = "hello world"
) (
    input logic clk,
    input rst,
    output[1:0] foo
);

endmodule