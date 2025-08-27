# JSON Output

Some commands and processes serialize data into JSON as a mechanism for external tools and scripts to easily load data that may be of interest.

## Design units

The `orbit get` command allows a user to receive various pieces of information related to a design unit, such its component declaration, defined architectures, or entity instantiation.

It also allows users to export the unit's interface with the `--json` flag. This is convenient when you wish to pass this information in a more machine-readable format to another tool/program.

The serialized JSON data is also available during the execution phase of a build process through the appropriate environment variables. To know which variables contain this data, see [Environment Variables](./environment_variables.md).

The serialized JSON string data is unformatted.

### Schema

The following schema is implemented for the json output of `orbit get`:
``` json
{
  // the name of the design unit
  "identifier": "string",
  // list of generics/parameters
  "generics": [
    {
        "identifier": "string",
        "mode": "string",
        "type": "string", // null if blank
        "default": "string" // null if blank
    }
  ],
  // list of ports
  "ports": [
    {
        "identifier": "string",
        "mode": "string",
        "type": "string", // null if blank
        "default": "string" // null if blank
    }
  ],
  // list of defined architectures (empty if Verilog or SystemVerilog)
  "architectures": [
      "string"
  ],
  // native language of the design unit (choices: "vhdl", "verilog", "systemverilog")
  "language": "string"
}
```

## Dependency trees

The `orbit tree` command allows one to view how cores are related to one another by displaying a dependency tree. 

It also allows users to export the dependency tree with the `--json` flag. This is convenient when you wish to pass this information in a more machine-readable format to another tool/program.

### Schema

The following schema is currently implemented for json output of `orbit tree`:
``` json
[
  {
    // the node's name
    "name": "string",
    // a list of the nodes that use this current node
    "targets": [
      "string",
    ],
    // a list of the nodes this current node uses
    "sources": [
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

The following schema is currently implemented for the json file `Orbit.json` produced by `orbit publish`:
``` json
{
  // schema version
  "version": integer,
  // serialized project manifest (Orbit.toml) data
  "manifest": {
    "project": {
      "name": "string",
      "uuid": "string",
      "version": "string",
      // ... see manifest reference for all properties
    }
  },
  // list of all local design units
  "units": [
    {
      // name of the design unit
      "identifier": "string",
      // the primary design unit type (examples: "entity", "module", "package")
      "type": "string",
      // native language (choices: "vhdl", "verilog", "systemverilog")
      "language": "string",
      // visibility of unit (choices: "public", "protected", "private")
      "visibility": "string",
      // list of files relative to the project that define this unit
      "sources": [
        "string"
      ],
      // list of unit identifiers that this unit requires
      "dependencies": [
        "string"
      ]
    }
  ]
}
```

The latest schema version is `1`. Schema versions increment by 1 whenever there are breaking changes to the schema and its defined properties.

## References

Some ideas about exporting json can be found at this [blog post](https://blog.kellybrazil.com/2021/12/03/tips-on-adding-json-output-to-your-cli-app/).