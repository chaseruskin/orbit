# Command line

The command line is the main way of interacting with `orbit`. Let's understand some terminology and rules for communicating to `orbit` through the command line.

## Syntax

Angular brackets (`< >`) denotes that a user input is required. The label within the angular brackets gives a hint to the user as to what type of value to enter here.

Square brackets (`[ ]`) denotes that the input is optional and is not required to get the command to successfully run.

## Jargon

Orbit uses _subcommands_, _arguments_, _options_, _flags_, and _switches_.

### __Subcommand__
Subcommands are special keywords to route to a particular action. Each subcommand inherits all supercommand's available options. They are the first positional argument following the call to `orbit`.
```
orbit get
```

### __Argument__
An argument is a value interpreted based on its position in the input. Arguments must be included when requested by Orbit within angular brackets (`< >`).
```
orbit new <path>
```
Arguments may be omitted if they are wrapped with square brackets (`[ ]`).
```
orbit search [<ip>]
```

### __Flag__
A flag is a simple boolean on-off conditional to alter a command's behavior, that is true when present and false otherwise. 
```
--help
```
Flags are options that do not take an argument and can be omitted.

### __Option__
An option is a type of flag that, when provided, is required to have an argument assigned to it. The argument may immediately proceed the option's flag separated by whitespace.
```
--plugin <name>
```
The argument may also be attached to the option's flag with an equal sign `=`.
```
--plugin=<name>
```
Options are able to be omitted.

### __Switch__
A switch is a shorthand flag denoted by a single dash and a single character.
```
-h
```
Multiple switches can be chained onto the same dash.
```
-ci
```

If a switch is associated with an option, it must be declared last on a chain with its argument to immediately follow separated by whitespace or an equal sign `=`.
```
-o <file>
```

Common flags and options may have a shorthand _switch_ associated with them. For example, `--help` can be alternatively passed with just `-h`.

### __Argument terminator__ 
The argument terminator is a special no-op flag `--` that tells the command-line interpreter to parse up until this symbol.

Some scenarios will allow you to pass arguments through `orbit` to an internally executed command. You can pass these arguments by using the _argument terminator_.

#### Examples

An example of using the argument terminator is brought up when calling a plugin through `orbit` during the building step.
```
$ orbit build --plugin yilinx -- --sram
```
In this command, `orbit` does not interpret the "--sram" flag, but instead passes it to the plugin named "yilinx" to handle.