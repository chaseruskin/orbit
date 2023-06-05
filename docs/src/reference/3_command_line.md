# Command line

The command line is the main way of interacting with Orbit. Let's understand some terminology and rules for communicating with Orbit through the command line.

## Syntax

`< >` denotes that a user input is required. The label within the angular brackets gives a hint to the user as to what type of value to enter here.

`[ ]` denotes that this input is optional and is not required to get the command to successfully run.

## Jargon

Orbit uses _subcommands_, _arguments_, _options_, _flags_, and _switches_.

### __Subcommand__
Special keyword to route to a particular action. Each subcommand inherits all supercommand's available options.
```
orbit <command>
```

### __Argument__
A value interpreted based on its position in the input. Arguments must be included when requested by Orbit.
```
orbit new <ip>
```

### __Option__
A flag that are required to have an argument assigned to them. The argument may immediately proceed the option's flag separated by whitespace or an equal sign `=`. Options can be omitted.
```
--output <file>

--code=<language>
```

Common flags and options may have a shorthand _switch_ associated with them. For example, `--help` can be alternatively passed with just `-h`.

### __Flag__
Simple boolean on-off conditional to alter a command's behavior. Flags are options that do not take an argument and can be omitted.
```
--help
```

### __Switch__
Shorthand flag denoted by a single dash and a single character. Switches can be chained onto the same dash.
```
-c -i
-ci
```
If a switch is associated with an option, it must be declared last on a chain with its argument separated by whitespace or an equal sign `=`.
```
-o <file>
```

### __Argument terminator__ 
Special no-op flag `--` that tells Orbit to parse up until this symbol.

Some scenarios will allow you to pass arguments through Orbit to an internally executed command. You can pass these arguments by using the _argument terminator_.

```
orbit build vivado -- synthesis --incremental
```
An example where an argument terminator is used is when invoking a plugin with Orbit. In this example,  `synthesis --incremental` is passed to a plugin recognized as "quartus" by Orbit.