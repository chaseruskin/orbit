# .orbitpub

A _.orbitpub_ is a file that lists user-defined file patterns to explicitly declare the set of HDL files that are public to external IPs that use this given IP as a dependency. Orbit recognizes .orbitpub files that match ".orbitpub" with case-sensitivity.

If no .orbitpub file exists, then all HDL files are considered public and are available to be integrated into other IPs.