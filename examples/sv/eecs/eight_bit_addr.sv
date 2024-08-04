module eight_bit_addr(
    input en, cin,
    input [7:0] a, b,
    output [7:0] sum,
    output reg cout);

    wire [6:0] carries;
    
    one_bit_addr addr [7:0] (
        .en(en), .a(a), .b(b), .cin({carries,cin}),
        .sum(sum), .cout({cout,carries})
);
endmodule