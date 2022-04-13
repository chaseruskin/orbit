# A multi-version approach to source-based package management.
### Chase Ruskin
### 2022-12-04

## Introduction

We will attempt to formulate the problem that arises during HDL development when working with multiple projects/components. Then we present the solution is to dynamically resolve symbol collisions.

## Terminology

- __Symbol__: an identifier from a project's public API. For example, in VHDL, primary design units can be thought of as the API, they are callable from the global space.

- __Hash__: a series of hexadecimal digits as the output of performing the SHA256 on the entire set of a project's IP. The hash can be used as a unique identifier for a project's current state.

## Problem

Consider we are currently working on "SUPER_IP" orbit package. An Orbit package has a manifest (Orbit.cfg) and any HDL files.

``` ini 
; /Orbit.cfg
[ip]
name    = SUPER_IP
version = 0.1.0
; ...
public  = [
    top.vhd,
]

[dependencies]
A_IP   = 2.0.0
MUX_IP = 1.0.0
```

Here is our public vhdl file `top.vhd` for this ip.

``` vhdl
-- /top.vhd
entity top is
-- ...
end entity top;

architecture rtl of top is
-- ...
begin

    u_mux : mux_2x1 generic map ( -- entity from MUX_IP 1.0.0 defined in manifest)
        n => 8
    ) port map (
        in1    => in1,
        in2    => in2,
        sel    => sel,
        output => output);

    u_a : ent_a port map ( -- entity from A_IP 2.0.0 defined in manifest
        x => x, 
        y => y); 

end architecture;
```

"A_IP" also uses "MUX_IP", or more generally, has a component in its design called `mux_2x1`. That is the root of the problem: a symbol collision. This `mux_2x1` is not the same as the one being used in `top.vhd` for "SUPER_IP".

``` ini
; cache/a3b8e4-a_ip-2.0.0/Orbit.cfg
[ip]
name    = A_IP
; ...
version = 2.0.0
public  = [
    ent_a.vhd,
]

[dependencies]
MUX_IP = 0.3.0
```

``` vhdl
-- cache/a3b8e4-a_ip-2.0.0/ent_a.vhd
entity ent_a is 
-- ...
end entity ent_a;

architecture rtl of ent_a is
    u_mux : mux_2x1 port map ( -- different mux_2x1 than used in SUPER_IP/top.vhd
        a    => in1,
        b    => in2,
        sel  => sel,
        z    => output);
end architecture;
```

"SUPER_IP"'s dependency "A_IP" also used a `mux_2x1`, but this is obviously a different entity than originally referenced in "SUPER_IP". Our intentions are clear though: we need both entities to exist to build the project, however, only one can have the name `mux_2x1`. No synthesis tool would support this. 

How can we allow both?

## Solution: Dynamic Symbol Collision Resolution (DSCR)

The `ent_a.vhd` file in "IP_A" is not easily accessible to the user because it is stored in the cache abstracted away. We also don't want the user to open and directly edit this file for multiple reasons. First, editing every symbol collision is tedious work to place on the user, especially when the project grows in complexityd drawing from many different IP. Second, the user could accidently modify some other piece of code and then break the build and any chance for reproducibility.

### Gameplan

Here is the "MUX_IP" orbit package in the cache:

- cache/a1e35f-mux_ip-1.0.0/...
- cache/fb59a1-mux_ip-0.3.0/...

In this instance, the `mux_2x1` symbol collision stems from trying to use the same IP, but as different versions. So, this could potentially be avoided when using minimum version selection. If "SUPER_IP" declared version `1` and "A_IP" declared `1.1.0`, then we would say "okay! these are the same symbols!" (chooses minimum selected version 1.1.0). However, if these entities came from different IP or if they specified their versions in a non-MVS fashion (like the problem statement in front of us), we need a different solution.

For version 1.0.0 of "MUX_IP", its source file in cache could look like:
``` vhdl
-- cache/a1e35f-mux_ip-1.0.0/mux_2x1.vhd
entity mux_2x1_a1e35f is
-- ...
end entity mux_2x1_a1e35f;
```

However, since a valid symbol must exist for `mux_2x1`, the cache for "MUX_IP" 1.0.0 will not use the hash in the symbol names because "SUPER_IP" (current IP being in development) wants `mux_2x1` to be from version 1.0.0.

For version 0.3.0 MUX_IP, this is its source file in cache:
``` vhdl
-- cache/fb59a1-mux_ip-0.3.0/mux_2x1.vhd
entity mux_2x1_fb59a1 is
-- ...
end entity mux_2x1_fb59a1;
```

Now, because of the current state of "SUPER_IP", the "A_IP"'s cached file actually contains:

``` vhdl
-- cache/a3b8e4-a_ip-2.0.0/ent_a.vhd
entity ent_a is 
-- ...
end entity ent_a;

architecture rtl of ent_a is
    u_mux : mux_2x1_fb59a1 port map ( -- diff mux than used in SUPER_IP/top.vhd
        a    => in1,
        b    => in2,
        sel  => sel,
        z    => output);
end architecture;
```

The end result hierarchy tree looks like this:
``` bash
top                   # /top.vhd
\_ mux_2x1            # cache/a1e35f-mux_ip-1.0.0/mux_2x1.vhd
\_ ent_a              # cache/a3b8e4-a_ip-2.0.0/ent_a.vhd
    \_ mux_2x1_fb59a1 # cache/fb59a1-mux_ip-0.3.0/mux_2x1.vhd
```

If we were to change the state of SUPER_IP:
``` ini
; /Orbit.cfg
[ip]
; ...

[dependencies]
A_IP   = 2.0.0
MUX_IP = 0.3
```

Then "A_IP"'s cached file would be also updated:
``` vhdl
-- cache/a3b8e4-a_ip-2.0.0/ent_a.vhd
entity ent_a is 
-- ...
end entity ent_a;

architecture rtl of ent_a is
    u_mux : mux_2x1 port map ( -- same mux as used in SUPER_IP/top.vhd!
        a    => in1,
        b    => in2,
        sel  => sel,
        z    => output);
end architecture;
```

Because "SUPER_IP" and "A_IP" agreed on the same minimum selected version for "MUX_IP", they are the same symbol (version 0.3.0)!

The end result hierarchy tree now looks like this:
``` bash
top             # /top.vhd
\_ mux_2x1      # cache/fb59a1-mux_ip-0.3.0/mux_2x1.vhd
\_ ent_a        # cache/a3b8e4-a_ip-2.0.0/ent_a.vhd
    \_ mux_2x1  # cache/fb59a1-mux_ip-0.3.0/mux_2x1.vhd
```

## Reproducibility
Because we are source-based, the cache is always mutating in-place when the state of the current IP changes dependency requirements to best account for symbol collisions.

After resolving symbol collisions, Orbit stores the new checksum of the mutated cache in the current IP's `Orbit.lock` file. This ensures the next time it tries to run that the cache is mutated correctly by checking with the checksum. If the checksums are incorrect it must perform the symbol resolving again.

The `Orbit.lock` file also captures what _exact_ version was selected from MVS. This ensures it picks the right version even as new versions of the same package enter the universal set.

### Power of the hash:
The hash provides more information than just a version number. Less likely for namespace collisions because entities come from different projects with different set of files. More uniqueness to avoid another namespace collision again.


## Rough Draft

Resolve the symbols to avoid collisions:

- Source IP specifies using "mux_2x1" of version 1.1.0
- A dependent IP specifies using "mux_2x1" but of version 0.1.9
- updates symbol to "mux_2x1_fb59a1" <- hash of the IP

Output: 
- blueprint .tsv file with in-order dependency files
- dependency files are the resolved files in cache with correct symbols for the current IP
- never performs transform on the current IP's source files

What about upgrading builds?

- change the minimum version in the manifest for the particular IP. Run tests to ensure everything still works. IP upgraded!

Implications? 
- The cache has to potentially rename symbols any time the current IP manifest has different dependencies. This calls for a fast algorithm.

## Outcome

- no dependency hell
- reproducible builds via MVS
- update version in one location (manifest)
- current source files in-development require no different approach from user stand-point


## Current Limitations/Drawbacks

- what happens when a symbol collision occurs within the current IP itself?
(what if SUPER_IP directly needed both mux v1.0.0 and mux v0.3.0)

Potential Solution: to handle symbol collisions within the current IP itself, must explicitly grab hashed symbol (`$ orbit get` will detect if needs to return hash based on current manifest). One of the two mux entity calls must have the explicit full symbol hash transform name (`mux_2x1_fb59a1`) (whichever one is second introduced).