# __orbit get__

## __NAME__

get - fetch an hdl unit for code integration

## __SYNOPSIS__

```
orbit get [options] <unit>
```

## __DESCRIPTION__

Returns hdl code snippets for the provided design unit to be integrated into
the current design. The code snippets are returned in the native hdl
language of the identified design unit. Code snippets are designed to be
copy and pasted from the console to the current design for quick code 
integration.

If an ip is not provided with the `--ip` option, then it will search the local
ip for the requested design unit.

If the design unit is in VHDL with the `--instance` option being used without
the `--component` option, then it will return the direct instantiation code
style (VHDL-93 feature).

Copying unit instantiations into higher-level entities will not 
automatically track source code references across ips. In order to properly
establish source code reference tracking across ips, the local ip's manifest
must have an up to date `[dependencies]` table that lists all the ips from
which it references source code.

An identifier prefix or suffix can be attached to the signal declarations and
the instantiation's port connection signals by using `--signal-prefix` and 
`--signal-suffix` respectively. These optional texts are treated as normal
strings and are not checked for correct hdl coding syntax.

When no output options are specified, this command by default will display
the unit's declaration.

Exporting the unit's declaration information can be accomplished by using the
`--json` option. The valid json is produced with minimal formatting for
encouragement to be processed by other programs.

## __OPTIONS__

`<unit>`  
      Primary design unit identifier

`--ip <spec>`  
      Ip specification

`--json`  
      Export the unit's information as valid json

`--library, -l`  
      Display the unit's library declaration

`--component, -c`  
      Display the unit's declaration

`--signals, -s`  
      Display the constant and signal declarations

`--instance, -i`  
      Display the unit's instantiation

`--architecture, -a`  
      Display the unit's architectures

`--name <identifier>`  
      Set the instance's identifier

`--signal-prefix <str>`  
      Prepend information to the instance's signals

`--signal-suffix <str>`  
      Append information to the instance's signals

## __EXAMPLES__

```
orbit get and_gate --ip gates:1.0.0 --component
orbit get ram --ip mem:2 -csi
orbit get uart -si --name uart_inst0
orbit get or_gate --ip gates --json
```

