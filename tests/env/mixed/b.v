`define WOW
/* verilator lint_off MULTITOP */

module my_pkg;
    const integer A = 10;
endmodule

// Source file for behavioral mux selector
(* hello *)
module b # (parameter integer NUM = 10, X = 5) (
    din_0,      // Mux 1st input
    din_1,      // Mux 2nd input
    sel = 1,        // Selector
    mux_out     // Mux chosen output
);
    input din_0, din_1, sel ;
    output mux_out;

    wire mux_out = 1;

    assign mux_out = (sel) ? din_1 : din_0;

endmodule 


module c;

endmodule


module register #(parameter WIDTH = 8)
(
	input wire clk, rst, wen,
	input [WIDTH-1:0] D,
	output [WIDTH-1:0] Q
);

	reg [WIDTH-1:0] val;

	assign Q = val;

	always@(posedge clk)
	begin
		if (rst)
			val<=0;
		else if(wen)
			val<=D;
	end

	
	

endmodule