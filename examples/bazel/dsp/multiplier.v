// Tiny bit-serial multiplier. Demonstrates Verilog instantiating a Verilog
// module from a sibling package (vutils.reg_n).
module multiplier (
  input  wire       clk,
  input  wire       rst_n,
  input  wire       a,
  input  wire       b,
  output wire       partial_q
);
  wire partial = a & b;

  reg_n #(.WIDTH(1)) u_reg (
    .clk(clk), .rst_n(rst_n), .d(partial), .q(partial_q)
  );
endmodule
