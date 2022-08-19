
- [ ] @idea: support `topics`/`tags` entry in Orbit.toml to provide another mechanism to filter ip search results in catalog (also have means to view all possible topics already defined in existing ip in catalog (so you know what to type in))
    ``` toml
    [ip]
    tags = ["cryptography", "symmetric"]
    # ...
    ```

- [ ] @idea: support `group` a virtual filepath-like categorization of an ip that can be filtered and have variable length on search
    ```toml
    [ip]
    group = "/a/b/c/d"
    # ...
    ```
    - specify `orbit search --group "**/symmetric"`
    - [ ] categories? groups? a tree-like attribute tied to an ip with variable depth defined by user displayed by search

### DEPENDENCY RESOLUTION STEPS

1. Build an Ip-level graph. If a dependency is missing, then it will be installed.


### Dynamic Symbol Resolution

- direct dependencies of the current ip cannot have duplicate identifier names

- all dependencies to follow (indirect) can follow to use DST

- as we build the ip graph, keep hashset of already used identifiers.

- if a ip dependency has a unit with an identifier already taken, then that ip must undergo DST. Transform all files into the sha markings. Since this file has new SHA markings, return to the immediate upper neighbors (each dependency that used this ip) and perform DST.


### Develop

- [ ] allow path as dependency 'version'

``` toml
[dependencies]
vendor.common.project1 = { path = "./dir/project1" , version = "0.1" }
```
