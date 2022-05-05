# __orbit edit__

## __NAME__

edit - further develop an ip in a text editor

## __SYNOPSIS__

```
orbit edit [options] <ip>
```

## __DESCRIPTION__

This command will open an ip project in the configured text editor. To
determine the editor, it will first check if the EDITOR environment
variable is set. If not, it will search for the key 'core.editor' in the
config.toml file. Explicitly setting the '--editor' option will override
any previously determined value.
  
The ip's project directory must be located on the ORBIT_PATH. The ip's 
project path will be passed as an argument to the text editor command. For 
example, if EDITOR="code", then the command orbit will execute is: 
`code <ip-path>`.

## __OPTIONS__

`<ip>`  
      The PKGID to look up the ip under ORBIT_PATH.
  
`--editor <cmd>`  
      The command to open the requested text-editor.

## __EXAMPLES__

```
orbit edit ks-tech.rary.gates --editor=code
```