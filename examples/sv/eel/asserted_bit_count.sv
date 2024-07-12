/*
File: asserted_bit_count.sv
Author: Chase Ruskin
Details:
    Various implementations utilizing a finite state machine and datapath for 
    counting the number of bits asserted ('1') for a generic-width input.
*/

// --- Disabled linting rules ---
/* verilator lint_off MULTITOP */
/* verilator lint_off DECLFILENAME */


//! 1-process FSMD for counting the number of bits asserted in the input.
module asserted_bit_count_fsmd_1p
#(
    parameter int WIDTH = 8
)
(
    input logic 		                clk,
    input logic 		                rst,
    input logic 		                go,
    input logic [WIDTH-1:0] 	        in,

    output logic [$clog2(WIDTH+1)-1:0]  out,
    output logic 		                done 
);
    // define a set of states for the FSM
    typedef enum { START, COMPUTE, WAIT } state_t;

    // capture the current state in a register
    state_t state_r;
    // have the ability to gatekeep the incoming data for processing over multiple cycles
    logic [$bits(in)-1:0] in_r;
    // store intermediate results into `out_r` register
    logic [$bits(out)-1:0] out_r;
    // store the output validation signal
    logic done_r;

    // @note: non-blocking assignments (`<=`) are updated at the end of the block
    always_ff @(posedge clk or posedge rst) begin
        // reset registers on rising edge of `rst` signal
        if (rst == 1'b1) begin
            in_r <= '0;
            out_r <= '0;
            state_r <= START;
            done_r <= 1'b0;
        end else begin
            case (state_r)
                START: begin
                    // read incoming data from external input
                    in_r <= in;
                    // reset internal output registers
                    out_r <= '0;
                    done_r <= 1'b0;

                    // transition to compute state on enable
                    if (go == 1'b1) begin state_r <= COMPUTE; end
                end
                COMPUTE: begin
                    // update the `in_r` register by moving it to an encoding with 1 less bit
                    in_r <= in_r & (in_r - ($bits(in_r))'(1));

                    // transition to completed state when `in_r` reaches 0
                    if (in_r == '0) begin 
                        state_r <= WAIT; 
                    // increment the internal counter recording the number of bits asserted
                    end else begin
                        out_r <= out_r + ($bits(out_r))'(1);
                    end
                end
                WAIT: begin
                    // signal to the outside the alogrithm is complete
                    done_r <= 1'b1;

                    // allow incoming data to be stored for future computation
                    in_r <= in;

                    // wait for another enable
                    if (go == 1'b1) begin
                        done_r <= 1'b0;
                        // reset internal counter
                        out_r <= '0;
                        state_r <= COMPUTE;
                    end
                end
            endcase
        end
    end

    // drive external output signals from internal registers
    assign out = out_r;
    assign done = done_r;

endmodule


//! 2-process FSMD for counting the number of bits asserted in the input.
module asserted_bit_count_fsmd_2p
#(
    parameter int WIDTH = 8
)
(
    input logic 		                clk,
    input logic 		                rst,
    input logic 		                go,
    input logic [WIDTH-1:0] 	        in,
    output logic [$clog2(WIDTH+1)-1:0]  out,
    output logic 		                done 
);

    // define a set of states for the FSM
    typedef enum { START, COMPUTE, WAIT } state_t;

    // capture the current state in a register
    state_t state_r, next_state;

    // have the ability to gatekeep the incoming data for processing over multiple cycles
    logic [$bits(in)-1:0] num_r, next_num;
    // store intermediate results into `count_r` register
    logic [$bits(out)-1:0] count_r, next_count;

    // sequential logic
    always_ff @(posedge clk or posedge rst) begin
        if (rst == 1'b1) begin
            state_r <= START;
            num_r <= '0;
            count_r <= '0;
        end else begin 
            state_r <= next_state;
            num_r <= next_num;
            count_r <= next_count;
        end
    end

    // combinational logic
    always_comb begin
        // assign defaults
        next_state = state_r;
        next_num = num_r;
        done = 1'b0;
        next_count = count_r;
        out = count_r;

        case (state_r)
            START: begin
                next_num = in;
                // restart the internal counter
                next_count = '0;

                if (go == 1'b1) begin next_state = COMPUTE; end
            end
            COMPUTE: begin
                // update the `num_r` register by moving it to an encoding with 1 less bit
                next_num = num_r & (num_r - ($bits(num_r))'(1));

                // transition to completed state if the next number to check is '0'
                if (num_r == '0) begin 
                    next_state = WAIT; 
                // increment the internal counter recording the number of bits asserted
                end else begin
                    next_count = count_r + ($bits(count_r))'(1);
                end
            end
            WAIT: begin
                // signal that the algorithm is complete
                done = 1'b1;
                // restart the internal counter
                next_count = '0;
                // capture the incoming to prepare for another computation
                next_num = in;

                if (go == 1'b1) begin next_state = COMPUTE; end
            end
        endcase
    end
endmodule


//! Structural FSM+D for counting the number of bits asserted in the input.
module asserted_bit_count_fsm_plus_d
#(
    parameter int WIDTH = 8
)
(
    input logic 		                clk,
    input logic 		                rst,
    input logic 		                go,
    input logic [WIDTH-1:0] 	        in,
    output logic [$clog2(WIDTH+1)-1:0]  out,
    output logic 		                done 
);

    logic en_num;
    logic en_ctr;
    logic is_num_zero;
    logic sel_ctr;
    logic sel_num;

    datapath #(
        .WIDTH(WIDTH)
    ) path (
        // inputs
        .clk(clk),
        .rst(rst),
        .in(in),
        .en_num(en_num),
        .en_ctr(en_ctr),
        .sel_num(sel_num),
        .sel_ctr(sel_ctr),
        // outputs
        .out(out),
        .is_num_zero(is_num_zero)
    );

    fsm mach (
        // inputs
        .clk(clk),
        .rst(rst),
        .go(go),
        .is_num_zero(is_num_zero),
        // outputs
        .en_num(en_num),
        .en_ctr(en_ctr),
        .sel_num(sel_num),
        .sel_ctr(sel_ctr),
        .done(done)
    );
endmodule


//! Top-level for synthesis. Change the comments to synthesize each module.
module \asserted_bit_count
#(
    parameter int WIDTH = 32
)
(
    input logic 		                clk,
    input logic 		                rst,
    input logic 		                go,
    input logic [WIDTH-1:0] 	        in,
    output logic [$clog2(WIDTH+1)-1:0]  out,
    output logic 		                done 
);

    // asserted_bit_count_fsmd_1p #(.WIDTH(WIDTH)) top (.*);
    // asserted_bit_count_fsmd_2p #(.WIDTH(WIDTH)) top (.*);
    asserted_bit_count_fsm_plus_d #(.WIDTH(WIDTH)) top (.*);  
    
endmodule