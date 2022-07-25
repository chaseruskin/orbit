# Dynamic Symbol Transformation

> __Note:__ This is an advanced topic and is not necessary to understand in order to use Orbit.

This technique is related to _name mangling_ in programming languages. _Name mangling_ is a technique used to solve problems regarding the need to resolve unique names for programming entities. You can learn more about name mangling [here](https://en.wikipedia.org/wiki/Name_mangling).


## Symbols

Within the context of VHDL, let's consider a _symbol_ to be the identifier of a primary design unit. There are four primary design units:
- entity
- package
- configuration
- context

In the following code, the symbol `reg` corresponds to an entity. This identifier could appear again in a different VHDL file instantiating `reg`.
``` vhdl
library ieee;
use ieee.std_logic_1164.all;

entity reg is
    port (
        clk   : in  std_logic;
        rst_n : in  std_logic;
        en    : in  std_logic;
        d     : in  std_logic_vector(7 downto 0);
        q     : out std_logic_vector(7 downto 0)
    );
end entity reg;
```

Now imagine there are two projects that use this symbol `reg`, but the entities are not the same. If your current project required both of those projects as dependencies, then traditionally your tool would not be able to resolve which `reg` was used where. Now, this problem is solved using _dynamic symbol resolution_.

## Walkthrough

Take this example dependency tree:
```
Project A 1.0.0
├─ Project_B 1.0.0
│  └─ Project_C 2.0.0
└─ Project_C 3.0.0
```

Imagine `Project_C` has a primary design unit named `entity_c`, but has notable backward-incompatible differences between version 2.0.0 and version 3.0.0.

```
entity_a
├─ entity_c (from version 3.0.0)
└─ entity_b
   └─ entity_c (from version 2.0.0)
```

Both units are required in order to build the design, but when reading the VHDL files through a backend tool a naming issue will occur and only one can be used. How can we keep both?

_Dynamic symbol transformation_ will identify symbol collisions within the current build graph and automatically resolve the conflicts to produce a  cleanbuild tree.

```
entity_a
├─ entity_c (from version 3.0.0)
└─ entity_b
   └─ entity_c_fbe4720d0 (from version 2.0.0)
```

It takes the symbol conflict, and produces a new unique symbol to use to disambiguate primary design units across projects. The original identifier is appended with the first 10 digits of the project version's checksum.

Notice the symbol to be transformed is not the symbol used in the current project, so dynamic symbol transformation has no effect to the user and is kept abstracted away to Orbit. Direct dependencies are never chosen for dynamic symbol transformation.