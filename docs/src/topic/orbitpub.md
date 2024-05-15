# .orbitpub

A _.orbitpub_ is a file that lists user-defined file patterns to explicitly declare the set of HDL files that are public to external IPs that use this given IP as a dependency. Orbit recognizes .orbitpub files that match ".orbitpub" with case-sensitivity.

By default, if no .orbitpub file exists, then all HDL files are set to public visibility.

## Visibility

There are three levels of visibility for HDL files: _public_, _protected_, and _private_.

Public HDL files allow users to reference any design units declared in that file from across IPs.

Protected HDL files are not allowed to be referenced directly across IP, but may be referenced by a public file internally to its IP.

Private HDL files are not allowed to be referenced directly across IP, and are not exposed from references in any public files. Private HDL files for an IP are removed from building the HDL graph when that IP is used as a dependency.