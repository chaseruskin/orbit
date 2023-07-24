# Motion 0x04
Chase Ruskin  
2022/02/18

## Handling Dependencies

One of the most importan tasks for a package manager is to efficiently manage a project's dependencies. Effective code reuse is the key to growing and maintaining a well-organized codebase. 

Dependency solving is a difficult problem. It has been identified as an NP-complete problem [1] for expressive dependencies. However, we take an approach to modify the problem's definition so that we avoid the NP-complete complexity class. The approach outlined throughout this paper is adapted from orbit's prototype and go mod's MVS algorithm [2].

## Dependency solving

To establish orbit as a package manager, it must answer some fundamental questions about its operations. A criteria was curated for a variety of package managers in a paper found here https://arxiv.org/pdf/2011.07851.pdf and we will attempt to answer them for orbit.

### 1. Versioning scheme

Orbit will use semantic versioning to specify the versions of the packages it manages. This gives 3 levels of ordering: `major.minor.patch`. These versions are to appear as tags within the distributed vcs (git).

### 2. Distribution

Orbit will use VCS hosting websites such as `GitHub` and `GitLab` as the main distribution platforms. It will operate under a decentralized registry system, so users can optionally create their own registries for others to connect with and enable access collections of packages.

### 3. Granularity

The minimal unit that can versioned is the package. All source files and supportive files under the same package will belong to the same version at the time of a  pacakge's release.

### 4. Version Locking

Version locking is enabled through the MVS algorithm, in which a dependency file is
generated for every build that specifies which exact version was used in the build.
Orbit will read this information upon building the project from scratch to reproduce the environment it was previously built in.

### 5. Qualifiers

Currently there is no immediate intention for supporting specific dependencies based on a particular build configuration. Having not really though about this until writing, it can be beneficial in shortening an overall dependency tree by optionally only including a top-level package's testbench dependencies and ignoring all other package testbench dependencies.

A solution to support this could be to omit testbench files (or more generally, just their exclusive dependencies) during installation. Then, any dependency packages that were only used in the testbench files would be omitted from future dependency graphs, as users should only have access to the true HDL source code to build upon in external packages.

A problem with the above solution could be that testbenches are written in the same file as the source code, and therefore that file cannot be dropped, nor its dependencies. How likely is that though? Especially since there are largely unsynthesizable constructs used in testbenches.

### 6. Dependency range operators

This is an important question. Along with range modifiers, this determines how flexible developers can be with accepting particular versions for a package.

Currently, there is no operations allowed on versions, other than the proposed range modifiers approach. Future intentions may be to allow the top-level package to decide on what ranges are allowed, or excluded. All contraints defined in those dependencies would be ignored, because the top-level package is given entire authority. This implementation would not add complexity because it is not requiring the resolution to simultaneously solve all used package constraints, only the top-level package constraints. I do think if a user introduced constraints on particular packages then they could accidently break correctness. However, this would obviously be caught very early with a warning or error.

### 7. Range modifiers

Range modifiers are written within the source code itself, right where the developer
uses a piece from a package. Range modifiers include:
- `flexible patch -> "_v1_0"`: willing to accept all future patches
- `flexible minor -> "_v1"`: willing to accept all future minor improvements
- `flexible major -> ""`: willing to accept latest version

With modifiers set up like this, users will not have to edit manifest files to update versions. 

However, updates will not occur implicitly. This protects against low-fidelity builds. The developer must open the package and run a command to update any new acceptable versions. This is designed to prevent packages from unintentionally breaking when one of its dependencies release newer versions that may not work.

### 8. Resolution process

Our MVS algorithm takes the highest version among each subset of available versions.
We also allow multiple versions to coexist as separate nodes within the dependency tree. These behaviors lead to the following:

- __correctness__: a solution with respect to the dependency constraints is proposed
- __completeness__: a solution is always found
- __user preferences__: the user cannot offer customized optimization criteria among valid solutions, because there will only be one valid solution among MVS

### Approximate solutions

- __missing dependencies__: if a dependency version constraint cannot be satisified (cannot find the package in a registry or cache), Orbit will report an error

- __conflicts__: to avoid conflicts, Orbit rewrites conflicting symbol names (such as `nor_gate` to `nor_gate_v1`) to enable multiple versions of a package to co-exist

## Multiple version support

To ease difficulty in our dependency solving, and to allow for more flexible builds, orbit supports multiple versions within the same dependency tree.

For example, an IP named `gates` can be used as different versions in the same project.

Consider the project `gates` with the following structure:

    /gates
        /nor_gate.vhd
        /IP.cfg

The `IP.cfg` is a mandatory manifest file for orbit to collect information about the given IP package. More discussion about `IP.cfg` is for a different time.

This would be a directory structure within the cache for `gates` if v1.0.0, v1.2.0, and v1.2.1 were installed:

    /gates 
        /v1.0.0
            /nor_gate.vhd
            /IP.cfg  
        /v1.2.0
            /nor_gate.vhd
            /IP.cfg
        /v1.2.1  
            /nor_gate.vhd
            /IP.cfg

Within each folder is the state of the code for the ip at its respective vcs version tags.

## How does this work in new projects?

A main concern you may have is, how would writing code differentiate between using
v1.0.0, v1.2.0, v1.2.1, when A has an entity say `nor_gate`. Trying to reference `nor_gate` under each context will run into errors regarding renaming/redefining the entity.

The solution is that for each installation in the cache, orbit actually performs a one-time analysis through the HDL files and alters the code with a _primary design unit modification_ (PDUM). Since primary design units are technically the API for your code,
modifying the identifiers will give unique units for each version.

So we have access to calling entities identified as `nor_gate_v1_0_0`, `nor_gate_v1_2_0`, and `nor_gate_v1_2_1`. This is the tighest bound we can place on a unit since we explicitly specify the 3 version numbers, and we will see shortly that this is not always necessary.

## Version bounds

Say for our design we wanted to loosen the version bound for using `nor_gate`. Following semantic versioning guidelines, we want the latest backwards-compatible version under v1, so anything under the v1 tag will suffice. When installing packages, orbit also maintains who is the latest v1 version and performs PDUM for the highest v1 installation. 

So our cache now looks like this:

    /gates 
        /v1.0.0
            /nor_gate.vhd
            /IP.cfg  
        /v1.2.0
            /nor_gate.vhd
            /IP.cfg
        /v1.2.1  
            /nor_gate.vhd
            /IP.cfg
        /v1                 * files copied from v1.2.1 code state
            /nor_gate.vhd
            /IP.cfg

We have access to `nor_gate_v1`, which in our case references the state of code from release v1.2.1.

This same logic extends to using a version bound that only includes the first 2 version numbers, such as v1.0 and v1.2.

## Rationale behind version bounds

Version bounds increase the flexibility of your current project to easily accept
future changes to defined dependencies. When you are ready for a project to use a
newer compatible version within given version bounds, you will be able to open the project under development and run a single command to update your cache and the project's dependency file to now point to the newer versions.

## Tradeoffs

The main tradeoff between allowing this flexibility is that the "locked" version it selected may not always be selected. So it is important to think carefully about what version bounds you intend to allow.

Here is an example scenario:

- X uses A with bound v1 from v1.2.1
- Y uses A with bound v1 from v1.4.0
- A's latest version is v1.8.2

You now have project Z that must incorporate X and Y. However, what version should be pointed to for using the v1 bound? Enter _Minimum Version Selection_ (MVS) [2].

## A quick glance at MVS

Our MVS will build the entire graph, collecting what versions are referenced for each version bound. So for A under project Z's build, we can only choose between v1.2.1 and v1.4.0 for determining A's bound for v1; v1.8.2 is excluded. 

MVS will then select v1.4.0 to use as A's bound for v1, because it is the minimum allowed version from all of A's choices (v1.2.1, v1.4.0, v1.8.2) which is also the maximum of all the constraints. To get a better grasp of MVS, I encourage you to read https://research.swtch.com/vgo-mvs.


### References

[1] [Dependency Solving Is Still Hard, but We Are Getting Better at It](https://arxiv.org/pdf/2011.07851.pdf)

[2] [Minimal Version Selection](https://research.swtch.com/vgo-mvs)