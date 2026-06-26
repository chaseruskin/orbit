// Top-level CPU stub: composes the ALU with a counter-driven program counter
// and a mux for input selection.
module cpu_top (
  input  logic       clk,
  input  logic       rst_n,
  input  logic       sel,
  input  logic [7:0] imm_a,
  input  logic [7:0] imm_b,
  output logic [7:0] result,
  output logic [7:0] pc
);
  logic [7:0] op_a;

  mux2 #(.WIDTH(8)) u_mux (.a(imm_a), .b(imm_b), .sel(sel), .y(op_a));

  alu u_alu (.clk(clk), .rst_n(rst_n), .a(op_a), .b(imm_b), .y(result));

  counter #(.WIDTH(8)) u_pc (
    .clk(clk), .rst_n(rst_n), .enable(1'b1), .count(pc)
  );
endmodule
