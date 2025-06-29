# Packages and Cores

An important line of distinction is understanding the difference between a package and an IP core.

An _IP core_, commonly referred to as a _core_, is the smallest amount of hardware description code that Orbit considers at a time. A core is a reusable block of design logic composed of one or more design units. A core's name is defined by its root design unit, which is where Orbit starts from when collecting the rest of the required design units for a particular build. Design units are the highest levels of abstraction in hardware description languages, which include VHDL entities and SystemVerilog modules, and can exist in one or more design files.

All cores remain as their source files when installed by Orbit and are intended to be shared with multiple projects.

The _core root_ is a primary design unit that Orbit starts from and makes up the root of the core.

A _package_ is a collection of one or more cores. A package contains an _Orbit.toml_ file, describing metadata for the collection of cores, as well as an auto-generated _Orbit.lock_ file, recording the world state of the package. A package in this context is often synonymous with "project".

To disambiguate the concept of an "Orbit package" from an "HDL package", much of the documentation in this book and throughout Orbit will favor using the word "project" instead of "package" when discussing an "Orbit package".

Orbit operates at the package level for hardware description languages, hence being a _package manager_.

Cores within a package can be made public or private, see [IP Core Visibility](./scope.md) for details.

## Orbit package vs. HDL package

A number of different concepts are commonly referred to by the word "package". This section clarifies the differences between two distinct meanings in Orbit packaging, "Orbit package" and "HDL package".

### What's an Orbit package?

An Orbit package is a collection of source files you can install. Most of the time, this is synonymous with "project". When you type `orbit install pkg`, or add `pkg = 1.0.0` to the `[dependencies]` table in your `Orbit.toml`, `pkg` is the name of the Orbit package. Orbit packages are what Orbit manages for you.

### What's an HDL package?

An HDL package is a design unit within VHDL or SystemVerilog. These packages are language constructs embedded within their respective language, and are often used to store and share data, methods, and parameters across other design units.

In SystemVerilog, you can import packages using the `import` keyword, and in VHDL, you can reference packages using the `use` keyword.

### What are the links between Orbit packages and HDL packages?

An Orbit package contains one or more cores, and within each core exists one or more design units. In some cases, a core's set of design units may contain one or more HDL packages. Therefore, Orbit packages always encapsulate HDL packages and provide a means for other projects to reference these HDL packages.

## References

[Python Packaging User Guide - Distribution package vs. import package](https://packaging.python.org/en/latest/discussions/distribution-package-vs-import-package/)

[The Rust Book - Packages and Crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html)

[Python Packaging User Guide - Package Formats](https://packaging.python.org/en/latest/discussions/package-formats/)

[Wikipedia - Semiconductor intellectual property core](https://en.wikipedia.org/wiki/Semiconductor_intellectual_property_core)