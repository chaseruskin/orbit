## Commands

### `query`

SYNOPSIS: `orbit query [options] [<ip>]`

Ask for information about a particular ip. Could ask for: available versions, what states is it in? (develop, install, available), what public primary design units exist under it?

By default it queries the current directory ip (if exists).

OPTIONS:  
- `--units` : list available public primary design units
- `--versions` : list available versions to fetch/install
- `--range <range>` : specify a range for versions (must have versions flag set)
- `--changelog` : print the changelog for the ip
- `--state <state>` : specify which state to get data from (dev, inst, avail)
-----

### `env`

SYNOPSIS: `orbit env [options] <key>...`  

Print environment variables important to orbit when it runs.

-----

### `config`

SYNOPSIS: `orbit config [options]`

Modify the config.toml file. Has subcommands: `get, set, append, remove`. Can
also specify what file to config by adding `--file` option.

-----

### `fetch` / `get`

SYNOPSIS: `orbit fetch [options] <ip>::<unit>`

from GO: "add dependencies to current module and install them"

Grab the unit and place ip project data into Orbit.dep file for reprod. build. It will automatically update your `[dependencies]` section of the manifest, and Orbit.dep file.

- `--component, -c` : return vhdl component declaration code
- `--instance, -i` : return vhdl instantiation code
- `--signals, -s` : return vhdl signal connection code
- `--about` : return file's top comment header block
- `--crude` : return the exact syntax and formatting of entity declaration section

Idea: possibly have a second command such as `add` that will actually add it to `[dependencies]` section of manifest. So there would two levels/types of this command, one for strictly "browsing"/"trying"/"exploring" and one for taking action. This could also be accomplished with a flag.

- `--peek`, `-p` : do not add to `[dependencies]` section

Examples:
``` vhdl
$ orbit fetch gates::nor_gate -ics

component nor_gate is 
    generic(
        N: positive
    );
    port(
        a : in  std_logic_vector(N-1 downto 0);
        b : in  std_logic_vector(N-1 downto 0);
        c : out std_logic_vector(N-1 downto 0)
    );
end component;

constant N : positive;
signal a : std_logic_vector(N-1 downto 0);
signal b : std_logic_vector(N-1 downto 0);
signal c : std_logic_vector(N-1 downto 0);

u0 : nor_gate generic map(
        N => N
    ) port map(
        a => a,
        b => b,
        c => c
    );
```
-----


### `sync`

SYNOPSIS: `orbit sync`

Updates Orbit.dep based on what dependencies it found in the code.
Also checks with 

-----

### `search`

SYNOPSIS: `orbit search [options] [<ip>]`

Browse the IP catalog. By default it searches in 3 locations: the orbit development path, the cache directory, and the paths along each vendor's directory. 

- `--topic <topic>` : Filter by topics found in manifest

- `--install, -I` : Filter for ip found in cache available for use.

- `--develop, -D` : Filter for ip found on orbit development path.

- `--available, -A` : Filter for ip found in vendor registries.

- `--no-format` : Do not put in a pretty table format

example output:
```
Vendor       Library         Name             Dev   Inst    Aval      
------------ --------------- ---------------- ------ ------- ----
ks-tech      rary            gates            *      *       *
c-rus        util            toolbox                 *       
```

Examples:
``` 
orbit search ks-tech.. 
orbit search gates --topic simple --topic edu -A
```

-----

### `audit`

SYNOPSIS: `orbit audit [options]`

Verifies all checksums match between the lock file and the current machine's cache.

-----

### `read`

SYNOPSIS: `orbit read [options] [entity-path]`

Inspect an hdl file.

This command allows developers to read more into dependency code. If the requested unit is within the current development IP, it will open the actual file. If the requested file to fetch is from an immutable IP, it will open a temporary copy of the file to prevent a user from accidently modifying the contents and breaking reproducibility.

If the entity is coming from a mutable place, it will open the file in-place. If the file is coming from the cache, then it will create a clean copy in a temporary directory managed by Orbit. Any changes to the file will not affect the cache entry.

OPTIONS:
    `--editor <editor>`: specify the editor to open it with
    `--clean`          : empty the read directory
    `--version, -v <version>` : ip version to reference


Examples:
```
orbit read gates:nor_gate
```

-----

## Roadmap

Next command should be `query`. This will set up for future commands and force
refactor structure and new core code.


``` sample rust error code on terminal
error[E0599]: no variant or associated item named `Colon` found for enum `core::vhdl::VHDLToken` in the current scope
    --> src/core/vhdl.rs:2150:39
     |
673  | enum VHDLToken {
     | -------------- variant or associated item `Colon` not found here
...
2150 |                 Token::new(VHDLToken::Colon, Position(1, 18)),
     |                                       ^^^^^ variant or associated item not found in `core::vhdl::VHDLToken`
```

Returns as many errors as it finds.

``` zsh
error[E0432]: unresolved import `crate::core::manifest`
 --> src/commands/edit.rs:8:5
  |
8 | use crate::core::manifest;
  |     ^^^^^^^^^^^^^^^^^^^^^ no `manifest` in `core`
```

Colors are blue for sideline

component_instantiation_statement ::= instantiation_label : instantiated_unit
[ generic_map_aspect ] [ port_map_aspect ] ;


instantiated_unit ::=
[ component ] component_name
| entity entity_name [ ( architecture_identifier ) ] 
| configuration configuration_name



# .orbitignore

Orbit automatically ignores files listed in the .gitignore file. These files will be excluded from gathering filesets.

Sometimes you may want to check a file into version control, but exclude it from
development for a period of time. Enter the .orbitignore file. Place this file at the root of the ip project (next to Orbit.toml), and Orbit will also honor the ignore rules from .orbitignore. .orbitignore files follow the same syntax and grammar as a .gitignore file. To learn more about .gitignore files, visit: https://git-scm.com/docs/gitignore. 

When conflicts arise between .gitignore and .orbitignore, the .orbitignore file has precedence.


### `pack` command

$ orbit pack [options]

by default, it will create a vhdl package file according to the defined public api listed in the Orbit.toml file. It will take all entities listed there and write them to a single package file name the same name as the ip project. It will place the file at the location where the command was called. 

- --output <path> : location to place the file. By default it is where the command was called.

- --name <identifier> : name of the actual package file, by default it is the same name as the filename.

- --entity, -e <unit>

- --append : append to the existing package file found out output path

- --overwrite : create a new package file at output path

- --public :

- --swap : remove the entities it took and replace with the package file name it created

``` vhdl
library eel4721c;
use eel4712c.lab1; -- bring in package

architecture rtl of top is
-- ...
begin

    -- library.package.entity
    u0 : component lab1.adder;

    -- direct call to the entity
    u1 : entity eel4712c.adder;

-- ... 
end architecture;
```

``` toml
[ip]
# ...
public = [
    "lab1",  # needed only if component instantiations
    "adder", # needed for direct entity calls
]
```


## References

To properly determine links between code,
an issue arises when using packages. Anywhere in the source code a reference to the package could be used, thus highlighting that it is needed in the blueprint. 

There is a common pattern to search for though:

packages must be called from their residing library: i.e. there must be somewhere in the code that uses `library.pkg_name` as a group of tokens.

If we can find all references, then we can determine what files were needed by what units.

examples of references

- `use library.pkg_name;`
- `:= library.pkg_name.constant;`
- `entity library.ent_name;`

- `component library.pkg_name.comp_name;`


### Characters for ascii tree

from stack overflow:
∟
├──
└

from cargo book:
├
│
─
└


top_level_tb
└── top_level
    ├── bottom_a
    │   └── bottom_b
    │
    └── bottom_c



Could perform two passes:

- 1st pass over tokens checks for references, collects all library.unit patterns in the token stream. This helps identify what primary design units are dependent upon each other.

- 2nd pass over tokens to build entity structs, and find architecture entity instantiations, and link entities to architectures

For example:




NOTE: using component instantiations, the component must reside within the same library
as the entity that is instantiating it.

What does this mean?

This means that component instaniations will only be able to be used for relative components becuase we group the current package under the 'work' directory.


-- from CURRENT WORKING IP
```
orbit get nor_gate -csi
```

-- from external ip
```
orbit get my_ip::clk_div -csi
```

# General Command-line Flags

## --verbose

- build: shows the actual command begin processed

- search: shows warnings of invalid manifests being skipped in results

- plan: shows primary unit detections and entity edge connections, also show filesets being collected

- launch: print more info about release process


## --force

- install: will reinstall even if directory already exists


## --list

- build: view available plugins along with summaries

- help: view all topics and commands along with summaries


## --interactive

- plan: ask for user input when a choice is to be made (multiple testbenches)

This flag is default behavior (which is the opposite of force). Maybe they should have separate meaning (--no-interactive different than --force).

