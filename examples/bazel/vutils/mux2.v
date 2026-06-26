// Parameterized 2-to-1 multiplexer.
module mux2 #(
  parameter WIDTH = 8
) (
  input  wire [WIDTH-1:0] a,
  input  wire [WIDTH-1:0] b,
  input  wire             sel,
  output wire [WIDTH-1:0] y
);
  assign y = sel ? b : a;
endmodule
