# Gates: Revisited

In this tutorial, you will learn how to:

[1.](#updating-the-gates-ip) Edit an existing IP  
[2.](#extending-the-yilinx-plugin) Use environment variables and command-line arguments to create more robust plugins  
[3.](#rereleasing-the-gates-ip) Release the next version for an existing IP  

## Editing the gates IP

It seems we left out some logic gates when we last worked on the gates project, so let's implement them now. Navigate to the directory in your file system where you currently store the gates project.

For the rest of this tutorial, we will be working relative to the project directory "gates/" that currently stores the gates project.

Let's implement the OR gate while restricting our design to only NAND gates like before. 
```
$ orbit get nand_gate --signals --instance
```
```
library work;

signal a : std_logic;
signal b : std_logic;
signal x : std_logic;

uX : entity work.nand_gate
  port map(
    a => a,
    b => b,
    x => x
  );
```

An OR gate can be constructed using 3 NAND gates. Let's copy/paste our NAND gate instances into our new file "or_gate.vhd".

Filename: or_gate.vhd
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

library work;

entity or_gate is
  port(
    a, b : in std_logic;
    y : out std_logic
  );
end entity;

architecture rtl of or_gate is

  signal x1, x2 : std_logic;

begin
  -- 1st layer: This gate negates the first input 'a'.
  u1 : entity work.nand_gate
    port map(
      a => a,
      b => a,
      x => x1
    );

  -- 1st layer: This gate negates the second input 'b'.
  u2 : entity work.nand_gate
    port map(
      a => b,
      b => b,
      x => x2
    );
    
  -- 2nd layer: This gate produces the final output ('a' OR 'b').
  u3 : entity work.nand_gate
    port map(
      a => x1,
      b => x2,
      x => y
    );

end architecture;
```

Showing the list of possible design units for the current project should now include the OR gate entity.
```
$ orbit show --units
```
```
Identifier                          Type          Public   
----------------------------------- ------------- -------- 
and_gate                            entity        y 
nand_gate                           entity        y 
or_gate                             entity        y 
```

## Extending the Yilinx plugin


Next, we want to program our Yilinx FPGA with the OR gate design to test it on the board. However, there are some quick updates we first want to apply to the ".orbit/yilinx.py" script.
- We want a way to specify which I/O pins of the FPGA will be used during placement and routing
- We want a way to specify whether to program the FPGA bitstream to SRAM storage (volatile) or flash storage (nonvolatile).

After searching through Yilinx documentation for hours, you learn that the Yilinx design tool can accept .ydc files for FPGA pin assignments. Let's edit our yilinx plugin to collect any .ydc files our project may have during `orbit`'s planning step.

Filename: .orbit/config.toml
``` toml
[[plugin]]
name = "yilinx"
command = "python"
summary = "Generate bitstreams for Yilinx FPGAs"
args = ["yilinx.py"]
# Define the type of extra file(s) to collect during planning
fileset.pin-file = "*.ydc"
```

Now let's create our pin assignment file for our OR gate design.

Filename: pins.ydc
``` text
A1=a
A2=b
C7=y
```

Next, let's edit the Python script for the yilinx plugin to allow the Yilinx tool to use our .ydc file if we ever collect one into our blueprint file. We also want to accept command-line arguments to optionally program our FPGA using SRAM or flash storage.

Filename: .orbit/yilinx.py
``` Python
import sys, os

# Handle command-line arguments
PROG_SRAM = bool(sys.argv.count('--sram') > 0)
PROG_FLASH = bool(sys.argv.count('--flash') > 0)

# Get environment variables set by orbit for this particular build
BLUEPRINT = os.environ.get("ORBIT_BLUEPRINT")
BUILD_DIR = os.environ.get("ORBIT_BUILD_DIR")
TOP_LEVEL = os.environ.get("ORBIT_TOP")

synth_order = []
constraints_file = None

# Parse the blueprint file created by orbit
with open(BLUEPRINT) as blueprint:
    rules = blueprint.readlines()
    for r in rules:
        fileset, lib, path = r.strip().split('\t')
        if fileset == 'VHDL-RTL':
            synth_order += [(lib, path)]
        if fileset == 'PIN-FILE':
            constraints_file = path
    pass

# Run the Yilinx tool from synthesis to bistream generation
for (lib, path) in synth_order:
    print('YILINX:', 'Synthesizing file ' + str(path) + ' into ' + str(lib) + '...')

print('YILINX:','Performing place-and-route...')

# Read the Yilinx design constraints file to map pins to I/O top-level ports.
if constraints_file != None:
    with open(constraints_file, 'r') as ydc:
        mapping = [x.strip().split('=') for x in ydc.readlines()]
    for pin, port in mapping:
        print('YILINX:', 'Mapping pin ' + str(pin) + ' to port ' + str(port) + '...')
    pass

print('YILINX:', 'Generating bitstream...')

BIT_FILE = TOP_LEVEL + '.bit'
with open(BIT_FILE, 'w') as bitstream:
    for byte in [bin(b)[2:] for b in bytes(TOP_LEVEL, 'utf-8')]:
        bitstream.write(byte)

print('YILINX:','Bitstream saved at: '+ str(BUILD_DIR + '/' + BIT_FILE))

# Optionally allow the user to program the FPGA using flash or SRAM configuration
if PROG_FLASH == True and PROG_SRAM == False:
    print('YILINX:', 'Programming bitstream to flash...')
elif PROG_SRAM == True:
    print('YILINX:', 'Programming bitstream to SRAM...')

```

With all these changes, we can now go ahead and program our FPGA as we want!

First, let's plan the OR gate design. We can specify which entity is the top level by using the "--top" command-line option. The "--plugin" command-line option tells `orbit` what additional files to collect for a future build with that plugin.
```
$ orbit plan --plugin yilinx --top or_gate 
```

Let's take a look at the blueprint file `orbit` just created.

Filename: build/blueprint.tsv
``` text
PIN-FILE	work	/Users/chase/tutorials/gates/pins.ydc
VHDL-RTL	work	/Users/chase/tutorials/gates/nand_gate.vhd
VHDL-RTL	work	/Users/chase/tutorials/gates/or_gate.vhd

```

Inspecting our latest blueprint file reveals our .ydc file was successfully collected.

Finally, let's build our design. We can pass command-line arguments directly to the plugin by entering them after the "--" argument.
```
$ orbit build --plugin yilinx -- --flash
```
```
YILINX: Synthesizing file /Users/chase/tutorials/gates/nand_gate.vhd into work...
YILINX: Synthesizing file /Users/chase/tutorials/gates/or_gate.vhd into work...
YILINX: Performing place-and-route...
YILINX: Mapping pin A1 to port a...
YILINX: Mapping pin A2 to port b...
YILINX: Mapping pin C7 to port y...
YILINX: Generating bitstream...
YILINX: Bitstream saved at: build/or_gate.bit
YILINX: Programming bitstream to flash...
```

As expected, the bitstream is written and saved in our build directory.

Filename: build/or_gate.bit
``` text
1101111111001010111111100111110000111101001100101
```

Awesome! We added some pretty advanced settings to our yilinx plugin to make it more robust for future use. Let's configure this plugin to be used with any of our ongoing projects by editing the global configuration file through the command-line.
```
$ orbit config --global --append include="$(orbit env ORBIT_IP_PATH)/.orbit/config.toml"
```

Now when we call `orbit` from any directory, we can see our yilinx plugin is available to use.
```
$ orbit plan --list
```
```
Plugins:
  yilinx          Generate bitstreams for Yilinx FPGAs
```

## Rereleasing the gates IP

We made changes to the gates IP, and now we want to have the ability to use these new updates or continue using the old changes. To do this, we want to update the version number in the manifest file. Let's edit the Orbit.toml file's version field to contain version "1.0.0".

Filename: Orbit.toml
``` toml
[ip]
name = "gates"
version = "1.0.0"

# See more keys and their definitions at https://c-rus.github.io/orbit/reference/manifest.html

[dependencies]

```

Finally, let's release version 1.0.0 for the gates IP by installing it to our cache.
```
$ orbit install --path .
```

One last look at the catalog shows the latest version of gates we have installed is indeed 1.0.0. Nice work!
```
$ orbit search gates
```
```
Package                     Latest    Status   
--------------------------- --------- ---------- 
gates                       1.0.0     Installed
```