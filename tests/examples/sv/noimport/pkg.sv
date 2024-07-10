package my_pkg; 
    function bit some_function(); 
        return( 1 );
    endfunction 
        
    typedef struct packed { 
        bit valid; 
        bit [ 11 : 0 ] field1; 
        bit [ 9 : 0 ] field2; 
    } my_t; 
    
    localparam MY_T_BITS = $bits( my_t ); 
endpackage
