# Use IP
<!-- 
## Exploring the IP catalog

1. View all known IP and their current states:
```
$ orbit search [ip] -d -i -a
```

2. Discover the possible versions of an ip:
```
$ orbit probe <ip> --tags
```

## Installing IP to the cache

3. Install a version of an ip to be referenced:
```
$ orbit install --ip <ip> -v <version>
```

4. Discover what primary design units are available to usefor that ip:
```
$ orbit probe <ip> -v <version> --units
```

## Integrating IP as dependencies

5. Grab HDL code to use in the current project:
```
$ orbit get <ip>:<entity> -v <version> --component -signals --instance
```

> __Note:__ By default, `orbit get` will add the \<ip> to your `Orbit.toml` manifest file. To only peek at the HDL code, pass the `--peek` flag. -->