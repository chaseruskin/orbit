module m(wire [31:0] bus, logic clk); 
    logic res, scan;
    // ...
    mutex check_bus(bus, posedge clk, res); 
    always @(posedge clk) scan <= res;
endmodule: m