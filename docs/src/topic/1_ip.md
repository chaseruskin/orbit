# IP

Orbit refers to the packages it manages as _IP_. Orbit recognizes a directory to be an IP by finding the `Orbit.toml` manifest file at the IP's root.

Here is an example IP directory structure:
```
/gates
├─ /rtl
│   └─ and_gate.vhd
├─ /sim
│   ├─ test_vectors.txt
│   └─ and_gate_tb.vhd
└─ Orbit.toml 
```

## IP Levels

An IP can exist at 3 different levels:
1. __developing___: the IP is in-progress/mutable and its location on disk is known (DEV_PATH).
2. __installed__: the IP is immutable and its location on disk is abstracted away from the user (CACHE).
3. __available__: the IP is not stored on disk but has the ability to be pulled from a git remote. Only the IP's manifest is stored locally on disk through a _vendor_.

## Inside an IP

An IP is a HDL project recognized by Orbit. Therefore, an IP's files can be grouped into 3 sections.

- HDL source code files
- manifest file (`Orbit.toml`)
- Supportive files

Supportive files are the files needed within particular HDL workflows. This is a very generic term because there are a lot of different workflows, some require constraints files, python scripts, text files, configuration files, etc.

## Current Working IP (CWIP)

The current working IP (CWIP) is the IP project currently being developed. It is detected within the path from where Orbit was invoked. Some commands, such as `orbit plan` and `orbit build`, require you to call Orbit from within a working IP.