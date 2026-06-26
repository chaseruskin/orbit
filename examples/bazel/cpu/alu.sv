// Tiny ALU that NANDs two 8-bit inputs and registers the result.
// Demonstrates SystemVerilog instantiating a Verilog module from a sibling
// package (vutils.reg_n).
module alu (
  input  logic       clk,
  input  logic       rst_n,
  input  logic [7:0] a,
  input  logic [7:0] b,
  output logic [7:0] y
);
  logic [7:0] nand_out;
  assign nand_out = ~(a & b);

  // Verilog register from vutils.
  reg_n #(.WIDTH(8)) u_reg (
    .clk(clk), .rst_n(rst_n), .d(nand_out), .q(y)
  );
endmodule
