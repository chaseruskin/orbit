# Command line

The command line is the main way of interacting with Orbit. Let's understand some terminology and rules for communicating with Orbit through the command line.

## Jargon

Orbit uses _subcommands_, _arguments_, _options_, and _flags_.

- __Subcommands__: Special keywords to route to a particular action. Each subcommand inherits all supercommand's available options.
```
orbit <command>
```

- __Argument__: A value interpreted based on its position in the input. Arguments must be included when requested by Orbit.
```
orbit new <ip>
```

- __Flags__: Simple boolean on-off switches to alter a command's behavior. Flags do not take an argument and can be omitted.
```
--help
```

- __Options__: Flags that are required to have an argument assigned to them. The argument may immediately proceed the option's flag separated by whitespace or an equal sign `=`. Options can be omitted.
```
--output <file>

--code=<language>
```

Common flags and options may have a shorthand _switch_ associated with them. For example, `--help` can be alternatively passed with just `-h`.

- __Switch__: Shorthand flag denoted by a single dash and a single character. Switches can be chained onto the same dash.
```
-c -i
-ci
```
If a switch is associated with an option, it must be declared last on a chain with its argument separated by whitespace or an equal sign `=`.

Some scenarios will allow you to pass arguments through Orbit to an internally executed command. You can pass these arguments by using the _argument terminator_.

- __Argument terminator__: Special no-op flag `--` that tells Orbit to parse up until the argument terminator.
