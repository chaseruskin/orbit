module mydesign ( input  x, y, z,     // x is at position 1, y at 2, x at 3 and
                  output o);          // o is at position 4

    a a0(.x(0), .y(0), .z(1));

    my_pkg my_pkg;

endmodule
