/*
File: fsm.sv
Author: Chase Ruskin
Details:
    Finite state machine implementation for counting the number of bits asserted
    ('1') for a generic-width input.
*/

module fsm
(
    input logic clk,
    input logic rst,
    input logic go,
    input logic is_num_zero,
    output logic en_num,
    output logic sel_num,
    output logic sel_ctr,
    output logic en_ctr,
    output logic done
);  
    // states
    typedef enum { START, CHECK_ZERO, COMPUTE, COMPLETE } state_t;

    state_t state_r, next_state;

    always_ff @(posedge clk or posedge rst) begin
        if (rst == 1'b1) begin
            state_r <= START;
        end else begin
            state_r <= next_state;
        end
    end

    always_comb begin
        // defaults
        next_state = state_r;
        en_num = 1'b0;
        sel_num = 1'b0;
        en_ctr = 1'b0;
        done = 1'b0;
        sel_ctr = 1'b0;

        case (state_r)
            START: begin
                // allow the incoming input to store
                sel_num = 1'b0;
                en_num = 1'b1;
                // reset the internal count
                sel_ctr = 1'b0;
                en_ctr = 1'b1;

                if (go == 1'b1) begin next_state = CHECK_ZERO; end
            end
            CHECK_ZERO: begin
                if (is_num_zero == 1'b1) begin 
                    next_state = COMPLETE;
                end else begin
                    next_state = COMPUTE;
                end
            end
            COMPUTE: begin
                // allow the counter to be updated
                sel_ctr = 1'b1;
                en_ctr = 1'b1;

                // allow the number register to be updated
                sel_num = 1'b1;
                en_num = 1'b1;

                next_state = CHECK_ZERO;
            end
            COMPLETE: begin
                // allow the incoming input to store
                sel_num = 1'b0;
                en_num = 1'b1;
                
                done = 1'b1;

                if (go == 1'b1) begin 
                    // reset the internal count
                    sel_ctr = 1'b0;
                    en_ctr = 1'b1;
                    next_state = CHECK_ZERO; 
                end
            end
        endcase
    end
endmodule
