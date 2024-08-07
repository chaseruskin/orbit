
module dupe2 (
    input en,
    output led
);

    assign led = (en == 1'b1) ? 1'b1 : 1'b0;

endmodule