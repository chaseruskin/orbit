# Ip

Ips are the core component that Orbit operates on as a package manager. First, let's understand some key terms related to ip in the context of Orbit.

## Anatomy of an ip

Developers daily tasks involve interfacing with a collection of closely related files (source code, scripts, text files). This collection of closely related files is typically stored under a single directory, and we will call this collection of files a _project_. 

The core component a package manager operates on is called a _package_. A _package_ is a project _with additional information provided by the developer_. This "additional information" is called _metadata_, and it is written to a special file called a _manifest_. The manifest must be placed at the project's root directory. Without manifests, a package manager would not know which projects it should manage and what the project's current state is in relation to being a package.

In the context of being a package manager for digital hardware, Orbit calls a package an _ip_. An ip's manifest file is "Orbit.toml", with case-sensitivity.

Typically, developers work on one project at a time. The _working ip_ is the ip that is currently being developed at a given moment. Some Orbit commands only work when they are called within the working ip (`orbit lock`, `orbit build`).

## Types of files inside an ip

Since Orbit focuses on digital hardware projects, it automatically detects and manages HDL source code files. Any other files are considered _supportive files_. Therefore, there are three different types of files within a given ip:

- HDL source code files (automatically configured for management)
- Manifest (automatically detected for management)
- Supportive files (automatically ignored unless manually configured for management)

Supportive files can be filtered and manually configured for management for a particular target by defining filesets. A _fileset_ is glob-style pattern that collects matching files under a common name within the working ip. These matched files will appear in the target's generated blueprint file for future execution.

### Reserved names

File names that begin with ".orbit-" are reserved for internal use and are not allowed at the root directory of an ip. Files that are named with this pattern are used by Orbit to store additional metadata about an ip at different levels within the ip catalog.

## Ip names




<!-- Orbit refers to the packages it manages as _IP_. Orbit recognizes a directory to be an IP by finding the `Orbit.toml` manifest file at the IP's root.

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

The current working IP (CWIP) is the IP project currently being developed. It is detected within the path from where Orbit was invoked. Some commands, such as `orbit plan` and `orbit build`, require you to call Orbit from within a working IP. -->