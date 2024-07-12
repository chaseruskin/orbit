// specify rtl adder for top.a1, gate-level adder for top.a2 

config cfg1;
    design rtlLib.top;
    default liblist rtlLib;
    instance top.a2 liblist gateLib;
endconfig