
module  simple_mux(
    din_0, // Mux first input
    din_1, // Mux Second input
    sel, // Select input
    mux_out // Mux output
);

 input din_0, din_1, sel ;

 output mux_out;

 wire  mux_out;

 assign mux_out = (sel) ? din_1 : din_0;
 
 endmodule 