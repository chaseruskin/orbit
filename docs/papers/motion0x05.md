# Motion 0x05
Chase Ruskin  
2022/03/10

## Orbit- HDL package manager and development tool

### Feature: Dual language support

A package manager focusing on the 2 primary HDL languages: VHDL and Verilog.

<!-- 
## Feature: Handling monorepositories

There are a few examples of open-source monorepositories out there; maybe due to a lack of a good package manager/registry system.
-->
### Feature: Specifying constraints on registry URLs

You may want to protect your system from unknowningly using a dependency from an
untrusted developer/codebase. In this case, you can specify what URLs are allowed to be requested when downloading dependencies.

For example, suppose GitHub user "malware5" writes malicious code tucked away as a dependency in some package. You can block user malware5's code by writing to the _blocklist_ of URLs to deny during installation.
<!--
``` ini
;; in settings.cfg file
[security]
blocklist = (
    github.com/malware5,
)
```
-->
Orbit will error on trying to install a dependency from a blocked repository.

You can also write to the _trustlist_ to ensure only packages from these repositories are installed.
<!--
``` ini
;; in settings.cfg file
[security]
trustlist = (
    gitlab.com/chaseruskin,
    github.com/uf-ece/eel4712.git,
)
```
-->
### Feature: Development dependencies

Sometimes dependencies are not to be used except when working on the current IP.
These may include dependencies used for testing or for creating a toplevel wrapper to be programmed to an FPGA.

Development dependencies should be noted separate from regular dependencies and not
required during a dependency resolution when that package is an intermediate dependency.

Entities hidden from public API (testbenches, toplevels) are entiites in which their dependencies should be marked as development. This will help keep the dependency graph lean.

Could Orbit determine which files are a part of public API and which files are not based on where the files are located (/rtl, /tests, /src, etc)? This would prevent user's from having to explicitly specify what files are public/private inside a manifest.
<!--
``` ini
;; in ip.cfg file
[ip.dependencies]
foo = 1.2.0
bar = 3.0.0

[ip.dependencies.dev]
baz = 2.8.3
```

``` ini
;; in ip.dep file
[ven.lib.foo]
ver = 1.2.0
sum = "124ad0f"
dev = yes

[ven.lib.bar]
ver = 3.0.0
sum = "ae4352f"
dev = no
```
-->

### Feature: Editing IP in tandem

:todo:

### Feature: Custom templates and files

:todo:

### Feature: An API for writing/reading from manifest for indirect configuration

Allow developers indirect access to modifying the fields within the manifest file so they can read/write within scripts and extensions.

Read from a field.
```
$ orbit config --manifest ip.name
foo
```

Write to a field.
```
$ orbit config --manifest ip.name bar
```

### Feature: Auto-detect the next proper version number

With a partial parser implemented, the tool can detect the changes between the current status of the project and the previous version. This can produce a recommendation to the user on what to set the next version as, so packages can be consistently versioned appropriately.