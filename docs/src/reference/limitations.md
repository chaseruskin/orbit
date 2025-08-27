# Known Limitations

This section hopes to outline and bring attention to peculiar or limiting behavior when working with Orbit. Some limitations may be removed in future releases, while some may stick around due to Orbit's inherent architecture.

## Preprocessors 

Orbit does not implement any preprocessors for VHDL, Verilog, or SystemVerilog. When tokenizing and analyzing source code, it is loosely evaluated as WYSISYG, which typically means certain constructs are "all-inclusive".

Consider the following Verilog code snippet:
``` verilog

module foo;

// ...

`ifndef SYNTHESIS
    bar u_bar(
        .a(h),
        .b(j),
        .c(i)
    );
`else
    baz u_baz(
        .x(h),
        .y(j),
        .z(i)
    );
`endif

endmodule
```

In this case, Orbit tokenizes the Verilog code and identifies _both_ `bar` and `baz` modules are depedencies of `foo`, even though one may eliminated due to the ``` `ifndef``` macro.

This can be seen as not inherently problematic, because the `SYNTHESIS` macro may be defined or omitted at compile-time during the execution stage of a build process, long after Orbit gathered source files during the planning stage.

Next consider the following Verilog code snippet:
``` verilog
`define TOP foo

module `TOP;
// ...
endmodule 
```

Orbit thinks the literal identifier for the module is named ``` `TOP```, which obviously is not the case. In this instance, Orbit will not properly analyze this module and may not be able to connect this module with any modules that instantiate it.

Preprocessors are a giant feat that are largely unnecessary for Orbit to correctly function in 99% of given scenarios. As seen in the first example, Orbit will correctly handle tokenizing around macros, but will not evaluate them.
