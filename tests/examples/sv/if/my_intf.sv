interface data_if #(
  parameter type data_t
);

logic vld;
logic ray;
logic data_t data;

modport src(
  output vld, data,
  input rdy
);

modport dst(
  input vld, data,
  output rdy
);

endinterface