# Package Management

Orbit is designed for users to efficiently manage their projects and scale their codebase over time. Many of the processes regarding package management are automated and abstracted away from the user, enabling them to get their next design done faster. This section takes a closer look at how the _catalog_ works in the context of Orbit and package management. The catalog is a central concept to Orbit where existing projects are kept and maintained by Orbit.

![](./../images/management.svg)

## Catalog operations

The catalog is an abstract concept in Orbit where your existing projects are kept and maintained. Physically, the catalog is composed of a set of hidden directories in your file system found within the `$ORBIT_HOME` directory. Unless something has gone terribly wrong, users never have to worry about manually working within these directories; users interface with Orbit to request operations to be performed on the catalog. The two most fundamental catalog operations are: `install` and `remove`.

The `install` operation takes an existing project, which may be from a local file path or a repository URL, and performs a series of steps to place an immutable reference of that project in the catalog's _cache_. This operation is typically applied when you wish to make it possible for your current project to depend on that other project. If the project is coming from a source, such as a GitLab repository, then a _protocol_ is used to handle how to download the project's contents from the internet. Upon a successful download, a compressed backup of the project's contents is stored in the catalog's _archive_. If the immutable reference in the cache ever enters an invalid state, Orbit automatically reinstalls the immutable reference from the compressed backup of the project found in the archive.

The `remove` operation takes a project found in the catalog's cache, and deletes it from the catalog's cache and archive (if exists). This operation is typically applied when you have determined that a project is not needed anymore for any future projects. If the project is still a dependency of other projects in the catalog, then the catalog will automatically reinstall the missing project if required based on the current project's complete project dependency graph.

## Depending on projects

A project's _field of view_ represents the set of HDL source code files that can interact with one another. A newly created project's field of view starts out limited to the set of files found only within that project. This means instantiations of modules and references to packages must be from primary design units located only within the HDL source code files of the current project.

When a module, package, or any other design unit is needed from a different project, users express their intention to depend on another project within the current project by adding the needed project's name and version to the `[dependencies]` section of the current project's manifest. By doing so, the current project's field of view __expands__ to include the HDL source code files found directly within the dependent project. This means instantiations of modules and references to other primary design units in the current project's HDL source code can now include primary design units found in the dependent project.

Depending on projects in the catalog is only allowed for projects that have an immutable reference located in the catalog's cache. A project in the catalog only listed as _available_ or _downloaded_ does not have the necessary data to properly reference any HDL source code files from that particular project. Thankfully, Orbit automatically works through the required steps to get a project installed to the cache when the user requests an `install` operation, even if the project is not found in the catalog.

## Important concepts

- The _source_ is the location where the project's repository exists; this could be any cloud-based platform such as GitHub or GitLab where your repository is stored.

- The _catalog_ is an abstract concept in Orbit that maintains 3 levels: available projects (found in channels), downloaded projects (located in the archive), and installed projects (located in the cache).

- When a project is known as _available_ in the catalog, Orbit only has the manifest and other metadata about that project; there is no HDL source code for that project in the catalog. The project's manifest and metadata are tracked in a user-configured _channel_.

- When a project is known as _downloaded_ in the catalog, Orbit has a compressed version of the project's contents. In this state, the project is unusable as a dependency. The compressed version of the project is stored in the _archive_.

- When a project is known as _installed_ in the catalog, Orbit has an uncompressed version of the project's contents used for reference only. In this immutable state of the project's contents, the project is able to be used as a dependency. The immutable reference of the project is kept in the _cache_.

- Users interact with the catalog through Orbit. Two fundamental catalog operations are: `install` and `remove`. The `install` operation brings a project into a usable state in the catalog, while the `remove` operation takes a project out of a usable state in the catalog.

- A project's _field of view_ is the set of HDL source code files that can interact with one another. Each project has its own field of view, which may extend to other projects not found in the current project's field of view. By adding a project as a dependency in the current project's manifest, you effectively expand the current project's field of view to include the set of HDL source code files found within the project dependency.
