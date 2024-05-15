# Final Project: Full adder

In this tutorial, you will learn how to:

[1.](#specifying-multiple-dependencies-for-an-ip) Depend on multiple IPs for a single project  
[2.](#overcoming-hdl-problems-namespace-pollution) Use `orbit` to overcome namespace pollution  
[3.](#reusing-plugins-that-are-globally-configured) Build a project with a globally-configured plugin  

## Specifying multiple dependencies for an IP

After the quick detour back to the gates IP, we are ready to tackle our final challenge in this mini tutorial series: the full adder. Like our previous projects, navigate to a directory in your file system where you would like to store the project.
```
orbit new full-add
```

For the rest of this tutorial, we will be working relative to the project directory "full-add/" that currently stores our new full-add project. 

For this final project, we will need circuits described in both the gate IP and half-add IP. Let's quickly recall if version 1.0.0 of gates has the OR gate we will need.
```
$ orbit show gates:1.0.0 --units
```
```
Identifier                          Type          Visibility   
----------------------------------- ------------- ----------- 
and_gate                            entity        public 
nand_gate                           entity        public 
or_gate                             entity        public 
```

Yup! It's there, and we know we will need some half adders as well. Let's add both IPs to our manifest file.

filename: Orbit.toml
``` toml
[ip]
name = "full-add"
version = "0.1.0"

# See more keys and their definitions at https://cdotrus.github.io/orbit/reference/manifest.html

[dependencies]
gates = "1.0.0"
half-add = "0.1.0"

```

Okay, time to start coding!

## Overcoming HDL problems: Namespace pollution

Our full adder circuit will be constructed of 2 half adders and an OR gate. Let's collect some HDL code snippets to use for our full adder circuit.
```
$ orbit get half_add --ip half-add --instance
```
```
library work;

uX : entity work.half_add
  port map(
    a => a,
    b => b,
    c => c,
    s => s
  );
```

And let's get the code snippet for the OR gate as well.
```
orbit get or_gate --ip gates:1.0.0 --instance
```
```
library work;

uX : entity work.or_gate
  port map(
    a => a,
    b => b,
    y => y
  );
```

Let's combine these circuits together into our new file for our full adder implementation.

Filename: full_add.vhd
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

library work;

entity full_add is
  port(
    a, b, cin : in std_logic;
    cout, s : out std_logic
  );
end entity;

architecture rtl of full_add is
  
  signal c_ha0, s_ha0, c_ha1 : std_logic;

begin

  -- 1st layer: Peform half of the addition operation.
  u_ha0 : entity work.half_add
    port map(
      a => a,
      b => b,
      c => c_ha0,
      s => s_ha0
    );

  -- 2nd layer: Compute the final sum term.
  u_ha1 : entity work.half_add
    port map(
      a => s_ha0,
      b => cin,
      c => c_ha1,
      s => s
    );

  -- 3rd layer: Check both c terms from the half adders for the final cout term.
  u_or0 : entity work.or_gate
    port map(
      a => c_ha0,
      b => c_ha1,
      y => cout
    );

end architecture;
```

Our design heirarchy is getting more complex; we have full adders constructed of half adders and OR gates, half adders constructed of NAND gates, OR gates constructed of... uh-oh. More NAND gates. 

The NAND gate design unit used in the OR gates is different from NAND gates used in the half adders because they reside in different versions of the gates IP (essentially different IPs). So did we just define an NAND gate entity twice with the same identifier? Yes, and thanks to `orbit`, this situation is okay.

### Huh?

Typical EDA tools will complain and error out when primary design units share the same name. How would they know which one is being used where? Fortunately, `orbit` is one step ahead of these tools due to implementing an algorithm called _dynamic symbol transformation_.

Let's take a look at the design tree hierarchy. You may notice something interesting.
```
$ orbit tree --format long
```
```
full_add (full-add:0.1.0)
├─ or_gate (gates:1.0.0)
│  └─ nand_gate (gates:1.0.0)
└─ half_add (half-add:0.1.0)
   └─ nand_gate_8108cfb282 (gates:0.1.0)
```

The entities from gates version 0.1.0 and version 1.0.0 are allowed to co-exist in this design. To circumvent EDA tool problems during builds, `orbit` appends the beginning checksum digits from the IP of the unit in conflict to the design unit's identifier. Any design units that referenced the unit in conflict will also be updated to properly reference the new identifier for the unit in conflict. 

To us though, these slight identifier renamings remain hidden because they occur among indirect dependencies in relation to our current project. When deciding which design unit to rename, `orbit` will always choose to rename the unit that is used as an indirect dependency. This key choice allows us to keep using the original unit name when integrating design units into the current project.

### Okay, so what?

This may be a silly example, but there is a key takeaway here. Designs are constantly evolving. When creating the latest module, you never know what will come next. By allowing the state of a design to live on while also providing support for new growth, a user no longer spends their time trying to manage compatibility among the increasingly interconnected dependencies. Instead, there exists a freedom to continue to innovate.

## Reusing plugins that are globally-configured

To conclude this mini tutorial series, let's generate a bitstream for the Yilinx FPGA with our full adder implementation. 

First, let's verify our yilinx plugin is available to us after appending it to our global configuration file in the previous tutorial.
```
$ orbit plan --list
```
```
Plugins:
  yilinx          Generate bitstreams for Yilinx FPGAs
```
We can review more details about a particular plugin by specifying it with the "--plugin" command-line option while providing "--list" as well.
```
$ orbit plan --list --plugin yilinx
```
```
Name:    yilinx
Command: python "yilinx.py" 
Root:    /Users/chase/tutorials/gates/.orbit
Filesets:
    PIN-FILE        **/*.ydc

Generate bitstreams for Yilinx FPGAs
```

Let's plan with the intention to build using the yilinx plugin for our full adder.
```
$ orbit plan --plugin yilinx --top full_add
```

Opening the blueprint file shows we are indeed using different files for the different NAND gate design units, and the files are in a topologically-sorted order.

Filename: build/blueprint.tsv
``` text
VHDL-RTL	work	/Users/chase/.orbit/cache/gates-0.1.0-fe9ec9d99e/nand_gate.vhd
VHDL-RTL	work	/Users/chase/.orbit/cache/half-add-0.1.0-1c537df196/half_add.vhd
VHDL-RTL	work	/Users/chase/.orbit/cache/gates-1.0.0-4cb065a539/nand_gate.vhd
VHDL-RTL	work	/Users/chase/.orbit/cache/gates-1.0.0-4cb065a539/or_gate.vhd
VHDL-RTL	work	/Users/chase/tutorials/full-add/full_add.vhd

```

If a plugin is specified during the most recent planning step, it can be inferred to be used in the building step and therefore we can omit the "--plugin" command-line option for our convenience.
```
$ orbit build
```
```
YILINX: Synthesizing file /Users/chase/.orbit/cache/gates-0.1.0-fe9ec9d99e/nand_gate.vhd into work...
YILINX: Synthesizing file /Users/chase/.orbit/cache/half-add-0.1.0-1c537df196/half_add.vhd into work...
YILINX: Synthesizing file /Users/chase/.orbit/cache/gates-1.0.0-4cb065a539/nand_gate.vhd into work...
YILINX: Synthesizing file /Users/chase/.orbit/cache/gates-1.0.0-4cb065a539/or_gate.vhd into work...
YILINX: Synthesizing file /Users/chase/tutorials/full-add/full_add.vhd into work...
YILINX: Performing place-and-route...
YILINX: Generating bitstream...
YILINX: Bitstream saved at: build/full_add.bit
```

Great work! This marks the end to this tutorial series, but the beginning of your experience with `orbit`, an HDL package manager.