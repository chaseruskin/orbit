module top;
    logic phi1, phi2;
    wire [8:1] cmd; // cannot be logic (two bidirectional drivers) logic [15:0] data;
    test main (phi1, data, write, phi2, cmd, enable);
    cpu cpu1 (phi1, data, write);
    mem mem1 (phi2, cmd, enable);
endmodule