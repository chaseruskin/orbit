# Orbit.toml

`Orbit.toml` is the manifest file that tells Orbit that a folder is an IP.

Here is a minimal example manifest:
``` toml
[ip]
vendor  = "ks-tech"
library = "rary"
name    = "gates"
version = "0.1.0"
```

## Available Keys

The following keys are accepted under the `ip` table:

- `vendor`: vendor identifier to link to IP index

- `library`: library identifier

- `name`: project identifier

- `version`: semantic version for current status

> __NOTE:__ The `vendor`, `library`, and `name` keys combined together to create the IP PKGID.

## Dependencies

The `dependencies` table lists the direct dependencies required for the current IP.

Here is an example dependencies section:
``` toml
# ...

[dependencies]
ks-tech.rary.mem = "1.2.0"
ks-tech.util.toolbox = "3"
xilinx.crypt.rsa = "1.3"
```

The fully qualified PKGID is entered as the key, while the minimum required version is entered as the value. The version can be partially or fully qualified.
