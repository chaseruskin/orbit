module example_entry(
     reset,
     clk,
     reset_val_a,

     value_a_in, 
     value_b_in, 
     value_a_in_en, 
     value_b_in_en, 
     value_a_out_en,
     value_b_out_en,

     value_a_out,
     value_b_out
     )

input reset;
input clk;

input [4:0] value_a_in;
input [4:0] reset_val_a;
input [15:0] value_b_in;
input 	value_a_in_en, value_b_in_en, value_a_out_en,  value_b_out_en;

output [4:0] value_a_out;
output [15:0] value_b_out; 

reg [4:0] stored_a;
reg [15:0] stored_b;

always @(posedge clk)
begin
   if (reset)
   begin
      stored_a <= `SD reset_val_a;
      stored_b <= `SD 32'd0;
   end
   else
   begin
      if (value_a_in_en)
         stored_a <= `SD value_a_in;
      if (value_b_in_en)
         stored_b <= `SD value_b_in;
   end
end 

assign value_a_out = value_a_out_en ? stored_a : 5'd0;
assign value_b_out = value_b_out_en ? stored_b : 16'd0; 

endmodule

/*
Note that I output a 0 in the case where the module is not enabled. This way in the module that declares the array and passes in a wired-or 
bus only one bus will be driving it's values. You need to be sure that your enable signals are only one hot. That is no more than one thing 
can be enabled for a given value.

Note that you could combine an enable signal for more than one value if that works better.

Note I assume that all entries have their b value reset to zero, but their a value is special (0-32 perhaps)
*/




module example_table(
       clock,
       reset,
       //... more inputs/outputs
       )
 
input clock;
input reset;
//... more inputs/outputs

//Declare Reset Values
wire [32*5-1:0] reset_a = {4'd0, 4'd1, 4'd2, 4'd3, ..., 4'd31}

//Wires for values to be written/enables (could be logic or input)
wire [4:0] value_a; //One value to all units, use enable to determine which one(s) latch it
wire [15:0] value_b;
wire [31:0] a_in_en;
wire [31:0] b_in_en;
wire [31:0] a_out_en;
wire [31:0] b_out_en;

//Declare the output bus as a wired-or (could use tristate and drive 16'dz in entry, but wor is faster)
wor [4:0] a_out;
wor [16:0] b_out;

//Example of indexing (assume a_wr_idx is input or internal value)
wire [5:0] a_wr_idx;
wire a_wr_en;

assign a_in_en = a_wr_en ? 32'd1 << a_wr_idx : 32'd0; //Enable just the idexed one, or none if not enabled

example_entry table[31:0](
       .reset(reset),
       .clk(clock),
       .reset_val_a(reset_a),
       .value_a_in(value_a), 
       .value_b_in(value_b), 
       .value_a_in_en(a_in_en), 
       .value_b_in_en(b_in_en), 
       .value_a_out_en(a_out_en),
       .value_b_out_en(a_out_en),
       .value_a_out(a_out),
       .value_b_out(b_out)
       )

endmodule

/*
Note See how I passed in the reset values for A

Note Here I enabled A to write on an a_write_idx. There are other ways to figure out how to assign the enbles, hopefully you can figure it 
out.
*/