module top;

localparam width = 1;
localparam guarded = 1'b1;

wire             CLK;
wire             RST;
wire [width-1:0] D_IN;
wire             ENQ;
wire             FULL_N;
wire [width-1:0] D_OUT;
wire             DEQ;
wire             EMPTY_N;
wire             CLR;

FIFO2 #(
  .width  (width),
  .guarded(guarded)
) ux (
  .CLK    (CLK),
  .RST    (RST),
  .D_IN   (D_IN),
  .ENQ    (ENQ),
  .FULL_N (FULL_N),
  .D_OUT  (D_OUT),
  .DEQ    (DEQ),
  .EMPTY_N(EMPTY_N),
  .CLR    (CLR)
);

endmodule
