# JSON Output

Some commands and processes serialize data into JSON as a mechanism for external tools and scripts to easily load data that may be of interest. All JSON output is sent to standard output (stdout).

## Cores

The `orbit get` command allows a user to receive various pieces of information related to a core, such its component declaration, defined architectures, or entity instantiation.

It also allows users to export the core's interface with the `--json` flag. This is convenient when you wish to pass this information in a more machine-readable format to another tool/program.

The serialized JSON data is also available during the execution phase of a build process through the appropriate environment variables. To know which variables contain this data, see [Environment Variables](./environment_variables.md).

The serialized JSON string data is unformatted.

### Schema

The following schema is implemented for the JSON output of `orbit get`:
``` json
{
  // The identifier of the unit
  "name": "string",
  // List of generics/parameters
  "generics": [
    {
      "name": "string",
      "mode": "string",
      "type": "string", // Null if blank
      "default": "string" // Null if blank
    }
  ],
  // List of ports
  "ports": [
    {
      "name": "string",
      "mode": "string",
      "type": "string", // Null if blank
      "default": "string" // Null if blank
    }
  ],
  // List of defined architectures (empty if Verilog or SystemVerilog)
  "architectures": [
    "string"
  ],
  // Hardware description language (choices: "vhdl", "verilog", "systemverilog")
  "language": "string",
  // Full path to the source file that declares this unit
  "source": "string"
}
```

## Hierarchies

The `orbit tree` command allows one to view how units are related to one another by displaying a dependency tree. 

It also allows users to export the dependency tree with the `--json` flag. This is convenient when you wish to pass this information in a more machine-readable format to another tool/program.

### Schema

The following schema is currently implemented for JSON output of `orbit tree`:
``` json
[
  {
    // the node's name
    "name": "string",
    // a list of the node names that use this current node
    "dependents": [
      "string",
    ],
    // a list of the node names this current node uses
    "dependencies": [
      "string"
    ]
  }
]
```

## Published projects

The `orbit publish` command allows users to automate the process of maintaining a centralized location where released projects can be found. When successfully ran, this command copies a project's manifest to a directory dedicated to storing manifests.

It also produces a JSON file of the various pieces of metadata related to the released project and places it alongide the copied manifest.

The serialized JSON data is available in a file called `Orbit.json` found in the same directory as where Orbit placed the `Orbit.toml` and `Orbit.lock` files during the publishing process. This file can be read by external tools and processes to collect various pieces of information to display to users in another way.

### Schema

The following schema is currently implemented for the JSON file `Orbit.json` produced by `orbit publish`:
``` json
{
  // Schema version
  "version": 1,
  // Serialized project manifest (Orbit.toml) data
  "manifest": {
    "project": {
      "name": "string",
      "uuid": "string",
      "version": "string",
      // ... See manifest reference for all properties
    }
  },
  // List of all local units
  "units": [
    {
      // The identifier of the unit
      "name": "string",
      // The primary unit type (examples: "entity", "module", "package")
      "type": "string",
      // Native hardware description language (choices: "vhdl", "verilog", "systemverilog")
      "language": "string",
      // Source visibility (choices: "public", "protected", "private")
      "visibility": "string",
      // Paths relative to the project's root directory for the source file that declare and implement this unit
      "sources": [
        "string"
      ],
      // List of unit names that the current unit requires
      "dependencies": [
        "string"
      ]
    }
  ]
}
```

The latest schema version is `1`. Schema versions increment by 1 whenever there are breaking changes to the schema and its defined properties.

## References

Some ideas about exporting JSON can be found at this [blog post](https://blog.kellybrazil.com/2021/12/03/tips-on-adding-json-output-to-your-cli-app/).