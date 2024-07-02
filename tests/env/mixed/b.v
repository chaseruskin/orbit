`define WOW

module my_pkg;
    const int A = 10;
endmodule

// Source file for behavioral mux selector
(* hello *)
module b # (WIDTH = A) (
    din_0,      // Mux 1st input
    din_1,      // Mux 2nd input
    sel,        // Selector
    mux_out     // Mux chosen output
);
    input din_0, din_1, sel ;
    output mux_out;

    wire  mux_out;

    assign mux_out = (sel) ? din_1 : din_0;

endmodule 


module c;

endmodule