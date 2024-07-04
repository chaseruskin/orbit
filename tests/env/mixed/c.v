/* verilator lint_off MULTITOP */

module my_mux (input       [2:0] 	a, b, c, 		// Three 3-bit inputs
                           [1:0]	sel, 			  // 2-bit select signal to choose from a, b, c
               output reg  [2:0] 	out); 			// Output 3-bit signal

  // This always block is executed whenever a, b, c or sel changes in value
  always @ (a, b, c, sel) begin
    case(sel)
      2'b00    : out = a; 		// If sel=0, output is a
      2'b01    : out = b; 		// If sel=1, output is b
      2'b10    : out = c; 		// If sel=2, output is c
      default  : out = 0; 		// If sel is anything else, out is always 0
    endcase
  end
endmodule

// Source file for behavioral mux selector
module mux_gp (
    din_0,      // Mux 1st input
    din_1,      // Mux 2nd input
    sel,        // Selector
    mux_out     // Mux chosen output
);
    input din_0, din_1, sel;
    output mux_out;

    assign mux_out = (sel) ? din_1 : din_0;

endmodule 


module tb_top;
	wire [1:0]  a;
	wire        b, c;

	mydesign d0  (a[0], b, a[1], c);  // a[0] is at position 1 so it is automatically connected to x
	                                  // b is at position 2 so it is automatically connected to y
	                                  // a[1] is at position 3 so it is connected to z
	                                  // c is at position 4, and hence connection is with o

    mydesign d1  ( .x (a[0]),    // signal "x" in mydesign should be connected to "a[0]" in this module (design_top)
	               .y (b),       // signal "y" in mydesign should be connected to "b" in this module (design_top)
	               .z (a[1]),
	               .o (c));
endmodule