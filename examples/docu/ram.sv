// Single-port random access memory with configurable address and data widths.
//
// This module creates a memory to read and write data.
module ram 
#(
    // Number of bits to represent the address lines
    parameter int ADDR_WIDTH,
    // Number of bits for each piece of data
    parameter int DATA_WIDTH
) (
    input logic clk, // global clock
    input logic rst,
    // Provide where to read/write data
    // 
    // This is a long description.
    input logic[ADDR_WIDTH-1:0] waddr,
    input logic[ADDR_WIDTH-1:0] raddr,
    // Set this bit to issue a write
    input logic wen,
    // Provide data only when writing
    input logic[DATA_WIDTH-1:0] wdata,
    // The outgoing data fetched from the ram
    output logic[DATA_WIDTH-1:0] rdata
);

    logic[(2**ADDR_WIDTH)-1:0][DATA_WIDTH-1:0] mem_q;

    assign rdata = (waddr == raddr) ? wdata : mem_q[raddr];

    always_ff @(posedge clk, posedge rst) begin
        if (rst == 1'b1) begin
            mem_q <= '0;
        end else begin
            if (wen == 1'b1) begin
                mem_q[waddr] <= wdata;
            end
        end

    end

endmodule



/// Foo bar is what this module does.
module foobar (
    input logic clk,
    input logic rst,
    output logic[1:0] xyz
);

    assign xyz = '0;

endmodule


package my_pkg;

  	function byte mul (input int x, y, output int res);
    	res = x*y + 1;
    	return x * y;
  	endfunction

    task sum(input [7:0] a, b, output [7:0] c);
    begin
        c = a + b;
    end
	endtask

    typedef enum bit [1:0] { RED, YELLOW, GREEN, RSVD } e_signal;

	typedef struct { bit [3:0]   signal_id;
                     bit         active;
                     bit [1:0]   timeout;
                   } e_sig_param;

endpackage
