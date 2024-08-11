/*--------------------------------------------------------------------
NAME : SLE_Prim
TYPE : FF/Latch
EQN  : Q = FF/LATCH
---------------------------------------------------------------------*/
`suppress_faults
`enable_portfaults
`celldefine
`delay_mode_path
`timescale 1 ns / 10 ps
module SLE_Prim (output Q,
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

  UDP_MUX2 mux_0(SYNC, SD, D, SLn);
  UDP_MUX2 mux_1(DATA, Q, SYNC, EN);
  
  UDP_DFF  DFF_0(QFF, ALn_int, ADn_, DATA, CLK);
  UDP_DL   DL_1(QL, ALn_int, ADn_, DATA, CLK);
  UDP_MUX2 mux_2(Q, QFF, QL, LAT);  
  
  not  U1(ADn_, ADn);
endmodule
`endcelldefine
`disable_portfaults
`nosuppress_faults