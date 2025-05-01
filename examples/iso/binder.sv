module tb;

    // To bind with all instances of DUT
    bind D_flipflop assertion_dff all_inst(clk, rst_n, d, q);

    // To bind with single instance of DUT
    bind tb.dff2[0] assertion_dff2 single_inst(clk, rst_n, d, q);

endmodule