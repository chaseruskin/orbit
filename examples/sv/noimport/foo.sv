
module foo #( 
    parameter MY_T_BITS = my_pkg::MY_T_BITS 
) ( 
    input wire [ MY_T_BITS - 1 : 0 ] data; 
); 
    wire foo = my_pkg::some_function(); 

    my_pkg::my_t my_reg; 

    always @( data ) my_reg = data;
 
    wire [ 11 : 0 ] field1 = my_reg.field1; 

endmodule