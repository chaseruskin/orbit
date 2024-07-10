interface apb_if (input pclk);
	logic [31:0]    paddr;
	logic [31:0]    pwdata;
	logic [31:0]    prdata;
	logic           penable;
	logic           pwrite;
	logic           psel;
endinterface