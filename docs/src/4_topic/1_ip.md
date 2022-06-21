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

An IP can exist at three different levels:
- developing: the IP is in-progress mutable and its location on disk is known (DEV_PATH)
- installed: the IP is immutable and its location on disk is abstracted away from the user (CACHE)
- available: the IP is not stored on disk but has the ability to be pulled from a remote repository

IPs are indirectly made available through _vendors_. A vendor is a repository used soley to track the list of repositories where to find and download IP.

## Inside an IP

An IP is a HDL project recognized by Orbit. Therefore, an IP's files can be grouped into 3 sections.

- HDL source code files
- manifest file (`Orbit.toml`)
- Supportive files

Supportive files are the files needed within particular HDL workflows. This is a very generic term because there are a lot of different workflows, some require constraints files, python scripts, text files, configuration files, etc.