module bcd_enc #(
    parameter int LEN = 4,
    parameter int DIGITS = 2
) (
    input logic rst,
    input logic clk,
    input logic go,
    input logic[LEN-1:0] bin,
    output logic[(4*DIGITS)-1:0] bcd,
    output logic done,
    output logic ovfl
);

    typedef enum {S_WAIT, S_SHIFT, S_ADD} state;

    logic[$bits(bcd)+$bits(bin)-1:0] dabble_r, dabble_d;

    logic ovfl_r, ovfl_d;
    state state_r, state_d;
    logic done_r, done_d;

    localparam int CTR_LEN = $clog2(LEN);

    logic[CTR_LEN-1:0] ctr_r, ctr_d;

    // registers with positive async reset
    always_ff @(posedge rst, posedge clk) begin
        if(rst == 1'b1) begin
            ovfl_r <= '0;
            dabble_r <= '0;
            ctr_r <= '0;
            done_r <= '0;
            state_r <= S_WAIT;
        end else begin
            state_r <= state_d;
            ctr_r <= ctr_d;
            done_r <= done_d;
            ovfl_r <= ovfl_d;
            dabble_r <= dabble_d;
        end
    end

    // simple pass through
    assign ovfl = ovfl_r;
    assign done = done_r;
    assign bcd = dabble_r[LEN +: $bits(bcd)];

    // determine next state and output signals
    always_comb begin
        logic[3:0] bcd_digit;

        state_d = state_r;
        ctr_d = ctr_r;
        ovfl_d = ovfl_r;
        dabble_d = dabble_r;
        done_d = done_r;

        case(state_r)
            // Wait to capture inputs on the start of a request
            S_WAIT: begin
                if(go == 1'b1) begin
                    dabble_d = '0;
                    dabble_d[$bits(bin)-1:0] = bin;
                    ctr_d = '0;
                    ovfl_d = '0;
                    state_d = S_SHIFT;
                    done_d = '0;
                end
            end
            // perform the "double" (multiply by 2)
            S_SHIFT: begin
                dabble_d = dabble_r << 1;
                ovfl_d = ovfl_r | dabble_r[$bits(dabble_r)-1];
                ctr_d = ctr_r + 1;
                if(ctr_r == (CTR_LEN)'(LEN-1)) begin
                    state_d = S_WAIT;
                    done_d = 1'b1;
                end else begin
                    state_d = S_ADD;
                end
            end
            // perform the "dabble" (+3 when >= 5)
            S_ADD: begin
                for(int i = DIGITS-1; i >= 0; i--) begin
                    bcd_digit = dabble_r[(4*i)+LEN +: 4];
                    if(bcd_digit >= 5) begin
                        dabble_d[(4*i)+LEN +: 4] = bcd_digit + 3;
                    end
                end
                state_d = S_SHIFT;
            end
            default: begin
                state_d = S_WAIT;
            end
        endcase
    end

endmodule
