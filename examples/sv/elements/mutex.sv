checker mutex (logic [31:0] sig, event clock, output bit failure); 
    assert property (@clock $onehot0(sig))
        failure = 1'b0; else failure = 1'b1;
endchecker : mutex
