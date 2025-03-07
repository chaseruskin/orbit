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