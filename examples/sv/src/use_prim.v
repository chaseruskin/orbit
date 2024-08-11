`suppress_faults
`enable_portfaults
`celldefine
`delay_mode_path
`timescale 1 ns / 10 ps
module use_mux_prim (output Q,
                 input ADn,
                 input ALn,
                 input CLK,
                 input D,
                 input LAT,
                 input SD,
                 input EN,
                 input SLn);
  wire ALn_int;
  
  assign ALn_int = ALn;

  multiplexer mux_0(SYNC, SD, D, SLn);
  multiplexer mux_1(DATA, Q, SYNC, EN); 
  
  not  U1(ADn_, ADn);
endmodule
`endcelldefine
`disable_portfaults
`nosuppress_faults