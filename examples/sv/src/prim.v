
//---------------------------------------------------------------------
// primitibe module (UDP_DFF) state table definition
// FUNCTION : POSITIVE EDGE TRIGGERED D FLIP-FLOP 
//                     WITH ACTIVE LOW ASYNCHRONOUS LOAD
//                     Q OUTPUT UDP.
//----------------------------------------------------------------------
primitive UDP_DFF (Q, AL, AD, SD, CLK);
  output Q;
  input AL, AD, SD, CLK;
  reg Q;
  table
  //AL    AD   SD   CLK  :  Qn  : Qn+1
     0     0    ?     ?  :  ?   :  0  ;   // Async Data
     0     1    ?     ?  :  ?   :  1  ;
     0     x    ?     ?  :  ?   :  x  ;    
    (?0)   0    ?     ?  :  ?   :  0  ;   // Update output for 
    (?0)   1    ?     ?  :  ?   :  1  ;   // Falling Edge
    (?0)   x    ?     ?  :  ?   :  x  ;
     0    (?0)  ?     ?  :  ?   :  0  ;   // Update output for 
     0    (?1)  ?     ?  :  ?   :  1  ;   // Changes in AD
     0    (?x)  ?     ?  :  ?   :  x  ;
    (?x)   0    ?     ?  :  0   :  0  ;   // Reducing Pessimism due to
    (?x)   1    ?     ?  :  1   :  1  ;   // transitions to X on AL
    (?x)   x    ?     ?  :  ?   :  x  ;                         // I can remove this line
    (?x)   ?    ?     ?  :  x   :  x  ;                         // I can remove this line
    (?x)   0    ?     ?  :  1   :  x  ;                         // I can remove this line
    (?x)   1    ?     ?  :  0   :  x  ;                         // I can remove this line
     x     x    ?     ?  :  ?   :  x  ;                         // I can remove this line
     x     0    ?     ?  :  1   :  x  ;                         // I can remove this line
     x     1    ?     ?  :  0   :  x  ;                         // I can remove this line
     1     ?    0   (01) :  ?   :  0  ;   // Sync Data
     1     ?    1   (01) :  ?   :  1  ;
     1     ?    x   (01) :  ?   :  x  ;
     1     ?    0   (x1) :  0   :  0  ;
     1     ?    1   (x1) :  1   :  1  ;
     1     ?    ?   (x1) :  x   :  x  ;                         // I can remove this line
     1     ?    x   (x1) :  ?   :  x  ;                         // I can remove this line
     1     ?    0   (x1) :  1   :  x  ;                         // I can remove this line
     1     ?    1   (x1) :  0   :  x  ;                         // I can remove this line
     x     1    1   (01) :  ?   :  1  ;
     x     0    0   (01) :  ?   :  0  ;
     x     ?    x   (01) :  ?   :  x  ;
     x     x    ?   (01) :  ?   :  x  ;
     x     1    0   (01) :  ?   :  x  ;
     x     0    1   (01) :  ?   :  x  ;
     x     0    0   (x1) :  0   :  0  ;
     x     1    1   (x1) :  1   :  1  ;
     x     ?    ?   (x1) :  x   :  x  ;
     x     ?    x   (x1) :  ?   :  x  ;
     x     x    ?   (x1) :  ?   :  x  ;
     x     ?    1   (x1) :  0   :  x  ;
    (?1)   ?    ?     ?  :  ?   :  -  ;   // Ignore Rising Edge on AL
     0     ?    ?     *  :  ?   :  -  ;   // Ignore Changes on CLK during Async
     ?     ?    ?   (?0) :  ?   :  -  ;   // Ignore CLK falling edge
     1     *    ?     ?  :  ?   :  -  ;   // Ignore changes on AD during Sync
     ?     ?    *     ?  :  ?   :  -  ;   // Ignore changes on SD during Sync
  endtable
endprimitive

//---------------------------------------------------------------------
// primitibe module (UDP_DL) state table definition
// FUNCTION : POSITIVE EDGE TRIGGERED D LATCH 
//                     WITH ACTIVE LOW ASYNCHRONOUS LOAD
//                     Q OUTPUT UDP.
//----------------------------------------------------------------------
primitive UDP_DL (Q, AL, AD, SD, CLK);
  output Q;
  input AL, AD, SD, CLK;
  reg Q;
  table
  //AL    AD   SD   CLK  :  Qn  : Qn+1
     0     0    ?     ?  :  ?   :  0  ;
     0     1    ?     ?  :  ?   :  1  ;
     0     x    ?     ?  :  ?   :  x  ;
    (?1)   ?    ?     ?  :  ?   :  -  ;
    (?0)   0    ?     ?  :  ?   :  0  ;
    (?0)   1    ?     ?  :  ?   :  1  ;
    (?0)   x    ?     ?  :  ?   :  x  ;
     0    (?0)  ?     ?  :  ?   :  0  ;
     0    (?1)  ?     ?  :  ?   :  1  ;
     0    (?x)  ?     ?  :  ?   :  x  ;
    (?x)   0    ?     ?  :  0   :  0  ;
    (?x)   1    ?     ?  :  1   :  1  ;
    (?x)   x    ?     ?  :  ?   :  x  ;
    (?x)   ?    ?     ?  :  x   :  x  ;
    (?x)   0    ?     ?  :  1   :  x  ;
    (?x)   1    ?     ?  :  0   :  x  ;
     x     0    ?     b  :  0   :  0  ;
     x     1    ?     b  :  1   :  1  ;
     x     x    ?     b  :  ?   :  x  ;
     x     ?    ?     b  :  x   :  x  ;
     x     0    ?     b  :  1   :  x  ;
     x     1    ?     b  :  0   :  x  ;
     1     ?    0     1  :  ?   :  0  ;
     1     ?    1     1  :  ?   :  1  ;
     1     ?    x     1  :  ?   :  x  ;
     1     ?   (?0)   1  :  ?   :  0  ;
     1     ?   (?1)   1  :  ?   :  1  ;
     1     ?   (?x)   1  :  ?   :  x  ;
     1     ?    0  (?1)  :  ?   :  0  ;
     1     ?    1  (?1)  :  ?   :  1  ;
     1     ?    x  (?1)  :  ?   :  x  ;
     1     ?    0  (?x)  :  0   :  0  ;
     1     ?    1  (?x)  :  1   :  1  ;
     1     ?    ?  (?x)  :  x   :  x  ;
     1     ?    x  (?x)  :  ?   :  x  ;
     1     ?    0  (?x)  :  1   :  x  ;
     1     ?    1  (?x)  :  0   :  x  ;
     1     ?    0     x  :  0   :  0  ;
     1     ?    1     x  :  1   :  1  ;
     1     ?    x     x  :  ?   :  x  ;
     1     ?    ?     x  :  x   :  x  ;
     1     ?    1     x  :  0   :  x  ;
     1     ?    0     x  :  1   :  x  ;
     x     1    1     x  :  1   :  1  ;
     x     0    0     x  :  0   :  0  ;
     0     ?    ?     *  :  ?   :  -  ;
     ?     ?    ?  (?0)  :  ?   :  -  ;
     1     *    ?     ?  :  ?   :  -  ;
     0     ?    *     ?  :  ?   :  -  ;
     1     ?    *     0  :  ?   :  -  ;
  endtable
endprimitive

//---------------------------------------------------------------------
// primitibe module (UDP_MUX2) state table definition
// FUNCTION : 2-to-1 MULTIPLEXER
//                     SL = 0  --> Q = A
//                     SL = 1  --> Q = B
//----------------------------------------------------------------------
primitive UDP_MUX2 (Q, A, B, SL);
  output Q;
  input A, B, SL;
  table
    //  A   B   SL  :   Q
    0   0   ?   :   0 ;
    1   1   ?   :   1 ;
    0   ?   0   :   0 ;
    1   ?   0   :   1 ;
    ?   0   1   :   0 ;
    ?   1   1   :   1 ;
    x   ?   0   :   x ;
    ?   x   1   :   x ;
    1   0   x   :   x ;
    0   1   x   :   x ;
  endtable
endprimitive

//---------------------------------------------------------------------
// primitibe module (UDP_GBLAT) state table definition
// FUNCTION : POSITIVE EDGE TRIGGERED D LATCH 
//                     Q OUTPUT UDP.
//----------------------------------------------------------------------
primitive UDP_GBLAT (Q, D, G);
  output Q;
  input D, G;
  reg Q;
  table
  //D    G   :  Qn  : Qn+1
    ?    0   :  ?   :  -  ;
    0    1   :  ?   :  0  ;
    1    1   :  ?   :  1  ;
    x    1   :  ?   :  x  ;
    0    x   :  0   :  0  ;
    1    x   :  1   :  1  ;
    *    0   :  ?   :  -  ;
    0  (01)  :  ?   :  0  ;
    1  (01)  :  ?   :  1  ;
    x  (01)  :  ?   :  x  ;
    ?  (?0)  :  ?   :  -  ;
    0  (?x)  :  0   :  0  ;
    1  (?x)  :  1   :  1  ;
  (?1)   1   :  ?   :  1  ;
  (?0)   1   :  ?   :  0  ;
  (?x)   1   :  ?   :  x  ;
  endtable
endprimitive

//---------------------------------------------------------------------
// primitibe module (UDP_GBLAT) state table definition
// FUNCTION : POSITIVE EDGE TRIGGERED D LATCH
//                     Q OUTPUT UDP.
//----------------------------------------------------------------------
primitive UDP_GBLAT_T (Q, D, G);
  output Q;
  input D, G;
  reg Q;

  initial
    Q = 1'b1;
  table
  //D    G   :  Qn  : Qn+1
    ?    0   :  ?   :  -  ;
    0    1   :  ?   :  0  ;
    1    1   :  ?   :  1  ;
    x    1   :  ?   :  x  ;
    0    x   :  0   :  0  ;
    1    x   :  1   :  1  ;
    *    0   :  ?   :  -  ;
    0  (01)  :  ?   :  0  ;
    1  (01)  :  ?   :  1  ;
    x  (01)  :  ?   :  x  ;
    ?  (?0)  :  ?   :  -  ;
    0  (?x)  :  0   :  0  ;
    1  (?x)  :  1   :  1  ;
  (?1)   1   :  ?   :  1  ;
  (?0)   1   :  ?   :  0  ;
  (?x)   1   :  ?   :  x  ;
  endtable
endprimitive

//---------------------------------------------------------------------
// primitibe module (UDP_BUFF) state table definition
// FUNCTION : BUFF
//----------------------------------------------------------------------
primitive UDP_BUFF (Y, A);
  output Y;
  input  A;

  table
  //A     :  Y
    0  :  0;
    1  :  1;
    x  :  x;
  endtable
endprimitive
