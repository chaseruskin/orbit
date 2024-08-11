primitive multiplexer (mux, control, dataA, dataB); 
output mux;
input control, dataA, dataB; 

  table
    // control dataA dataB mux
    010:1; 011:1; 01x:1; 000:0; 001:0; 00x:0; 101:1; 111:1; 1x1:1; 100:0; 110:0; 1x0:0; x00:0; x11:1;
  endtable

endprimitive