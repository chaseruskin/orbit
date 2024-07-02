module Mux_4_To_1 (input [1:0] i_Select,
                   input  i_Data1,
                   input  i_Data2,
                   input  i_Data3,
                   input  i_Data4,
                   output o_Data);
 
  reg r_Data;
   
assign o_Data = i_Select[1] ? (i_Select[0] ? i_Data4 : i_Data3) :
                              (i_Select[0] ? i_Data2 : i_Data1);
 
 
// Alternatively:
/* verilator lint_off COMBDLY */
always @(*)
begin
  case (i_Select)
    2'b00 : r_Data <= i_Data1;
    2'b01 : r_Data <= i_Data2;
    2'b10 : r_Data <= i_Data3;
    2'b11 : r_Data <= i_Data4;
  endcase // case (i_Select)
end
 
assign o_Data = r_Data;
     
endmodule // Mux_4_To_1