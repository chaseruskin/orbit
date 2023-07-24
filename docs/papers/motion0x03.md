# Motion 0x03
Chase Ruskin  
2022/02/09

## Describing dependencies

There are some differences between the structure of a typical HDL file and software programming language files. These differences align nicely with the concept of graphs and dependencies.

1. To describe hardware, an entity (or module in verilog) is declared

2. As design complexity increases, entities commonly encapsulate and instantiate other "lower-level" entities

These entity properties within HDL can be illustrated with a directed acyclic graph.

There are two options for how to handle updating a project's external dependency list: __explicit__ and __implicit__. Regardless of implementation, for an entity to rely/depend on another entity it must be instantiated and wired accordingly within the source code.

- _Explicit_ means there exists a secondary source (perhaps some file) where developers must list what dependencies a current project requires, as well as maintain their source code to actually use that dependency.

- _Implicit_ means that a dependency list for the current project can be deduced from the source code itself. Developers must only maintain their code while the task of maintaining a dependency list is automated to the tool.

## Tradeoffs: Explicit vs. Implicit

### Explicit dependencies
- Pros
    - simpler for the tool by avoiding analyzing source code

- Cons
    - adds an unecessary extra step for developers to remember to update a location for dependencies as well as making sure their source code reflects the dependencies.
    - a change describing the same thing must occur in multiple locations

### Implicit dependencies
- Pros
    - faster integration with external dependencies
    - developers do not have to worry about remembering to update a dependencies list
    - allows a dependency list to follow a format not intended for humans to write
    - a change must only occur in one place (source code)
    - nicely extends to allowing a developer to visualize an entity's heirarchy tree itself because the tool extracted that data from source code
- Cons
    - requires more overhead within the tool to analyze source code

## Integrating dependencies

> __User story:__ _As a developer of a large codebase, I want to quickly integrate designs without worrying about miniscule details and remembering exact I/O._

Since adding a dependency commonly involves instantiating some external entity component, there is a command to automate adding a dependency.

A `get` command, specifying the external entity component, can be called to return the common code for instantiating that component directly within your source code.

Take an entity named _hw_comp_ described elsewhere to have 2 input ports _A_ and _B_ and 1 output port _C_. To add it into a new project's source code, `get` the instantiation code.
```
$ orbit get rary.hw_comp

uX : entity rary.hw_comp port map(
    A => w_A,
    B => w_B,
    C => w_C  
);
```

Some may argue this command is outside the scope of the package manager's role because it generates boilerplate code. To counter this claim, I would say `get` automates how to add a dependency to a project, which directly relates to the goal of streamlined and seamless integration. 

A main benefit from the `get` command is that a developer does not need to recall any previous entity's I/O mappings. For example, suppose developer A made one entity and now developer B must integrate it into some larger design. Developer B did not design this component, but this lack of entity I/O information is not a roadblock thanks to the `get` command.

I see `get` encouraging a more modular approach to separating large-scale systems into smaller, more resuable subsystems. It makes component integration _boring_; developers are given the answer to the question of how to instantiate particular designs.

## Implicit dependencies are a win

After only adding the `get` code and making any minor changes to the wired connections, the tool can then scan the source file next time a `build` occurs to update the project's dependency list. This creates a nice illusion to the developer that all IP is simply "one command away".

The common case in creating new hardware designs involves reusing existing generic sub-components within the larger design. We will take advantage of this common case to implement _implicit_ dependency management. It creates the most direct and minimally demanding route to define dependencies on the developer-end.

Go's package management system, go mod, uses implicit dependency management by inferring imports from the source code.

## Lazy linting: An approach to in-line dependency scanning

A role outside the scope for a package manager would be synthesizing/compiling. It is by no means intended to replace the available backend tools. However, in order for implicit dependency to work, the tool must be aware of valid HDL syntax. Rather than implementing full HDL static linting, we can cheat here by designing a laid-back _lazy linter_.

There are two main steps to compiling source code: tokenizing and parsing.
A lazy linter would fully implement the easier but necessary tokenization, and it would partially implement the grammar and parsing _just enough_ to produce an abstract syntax tree to achieve the `get` command's functionality.

By only defining _just enough_ grammar, it makes the parsing code less complex and faster. It also makes it easier to maintain as language specifications are updated because we only deal with a tiny subset of the grammar.