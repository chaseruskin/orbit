// Parameterized synchronous register.
module reg_n #(
  parameter WIDTH = 8
) (
  input  wire             clk,
  input  wire             rst_n,
  input  wire [WIDTH-1:0] d,
  output reg  [WIDTH-1:0] q
);
  always @(posedge clk or negedge rst_n) begin
    if (!rst_n) q <= {WIDTH{1'b0}};
    else        q <= d;
  end
endmodule
