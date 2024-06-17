# Exporting Interfaces

The `orbit get` command allows a user to receive various pieces of information related to a design unit, such its component declaration, defined architectures, or entity instantiation.

It also allows users to export the unit's interface with the `--json` flag. This is convenient when you wish to pass this information in a more machine-readable format to another tool/program.

## Schema

The following schema is implemented for the json output:
``` json
{
  "identifier": string
  "generics": [
    {
        "identifier": string
        "mode": string
        "type": string
        "default": string // null if blank
    }
  ]
  "ports": [
    {
        "identifier": string
        "mode": string
        "type": string
        "default": string // null if blank
    }
  ]
  "architectures": [
      string
  ],
  "language": string
}
```

## References

Some ideas about exporting json from `orbit get` can be found at this [blog post](https://blog.kellybrazil.com/2021/12/03/tips-on-adding-json-output-to-your-cli-app/).