# Develop IP

## Creating a new IP from scratch
1. Create a new directory and import a template project:
```
$ orbit new <ip> --template <alias>
```

## Creating a new IP from an existing project

### Local project
1. Change directories to the project's root

2. Initialize an existing directory to be an IP:
```
$ orbit init <ip>
```

> __Note:__ Make sure that the ip is within the ORBIT_DEV_PATH if wanting to be able to detect it when using other Orbit commands.

### Remote project
1. Initialize a remote git repository as an IP on your local machine:
```
$ orbit init <ip> --git <url>
```

## Editing an IP

If the IP is recognized on the ORBIT_DEV_PATH, Orbit can open a text editor to the project for you. 

Open an IP found on the ORBIT_DEV_PATH:
```
$ orbit edit <ip> --editor <editor>
```

Only projects labeled under development are allowed to be opened for editing.

## Reviewing a design

View the HDL design hierarchy from within the current working IP:
```
$ orbit tree --root <entity>
```

## Planning a design

Orbit will collect the list of necessary files for you to build and execute a workflow. This command must be ran from within the current working IP.

Collect filesets defined for a plugin and write to a blueprint file:
```
$ orbit plan --plugin <alias>
```

A build directory is created and along with a blueprint.tsv file. The build directory changes frequently and is not to be edited by the user.

## Building a design

Orbit allows you to configure and run customized workflows through plugins.

Run a plugin and pass additional arguments to the plugin's command:
```
$ orbit b --plugin <alias> -- [args]...
```