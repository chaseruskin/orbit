// Up-counter built on top of reg_n.
module counter #(
  parameter WIDTH = 8
) (
  input  wire             clk,
  input  wire             rst_n,
  input  wire             enable,
  output wire [WIDTH-1:0] count
);
  wire [WIDTH-1:0] next = enable ? count + 1'b1 : count;

  reg_n #(.WIDTH(WIDTH)) u_reg (
    .clk(clk), .rst_n(rst_n), .d(next), .q(count)
  );
endmodule
