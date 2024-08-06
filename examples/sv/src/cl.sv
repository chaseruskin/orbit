virtual class Packet #(WIDTH = 2) extends hello implements aa;

    //data or class properties
    bit [3:0] command;
    bit [40:0] address;
    bit [4:0] master_id;
    integer time_requested;
    integer time_issued;
    integer status;
    typedef enum { ERR_OVERFLOW = 10, ERR_UNDERFLOW = 1123} PCKT_TYPE; 
    const integer buffer_size = 100;
    const integer header_size;

    // initialization 
    function new();
          command = 4'd0;
          address = 41'b0;
          master_id = 5'bx;
          header_size = 10;
    endfunction

    // methods
    // public access entry points 
    task clean();
        command = 0; address = 0; master_id = 5'bx;
    endtask

    task issue_request( int delay ); 
        // send request to bus
    endtask

    function integer current_status();
        current_status = status;
    endfunction
endclass


// interface class PutImp#(type PUT_T = logic); 

//     pure virtual function void put(PUT_T a);
//     endfunction

// endclass

// interface class Messaging #(type T = logic);
//   pure virtual task          put(T t);
//   pure virtual task          get(output T t);
//   pure virtual task          peek(output T t);
//   pure virtual function bit  try_peek(output T t);
//   pure virtual function bit  try_put(T t);
//   pure virtual function bit  try_get(output T t);
// endclass

module example_mod(
    `ifdef MY_VAR
        input wire clk,
        input wire rst
    `endif
);

endmodule