# Dynamic Symbol Transformation

> __Note:__ This is an advanced topic and is not necessary to understand in order to use Orbit.

This technique is related to _name mangling_ in programming languages. _Name mangling_ is a technique used to solve problems regarding the need to resolve unique names for programming entities. You can learn more about name mangling [here](https://en.wikipedia.org/wiki/Name_mangling).

## Problem Statement

Before we begin, it is important to understand the problem we are trying to solve. An issue inherent to VHDL and many other languages is _namespace pollution_, which is when many programming language variables/identifiers/units/classes are defined at the global level. To learn more about namespace pollution, [here](https://stackoverflow.com/questions/8862665/what-does-it-mean-global-namespace-would-be-polluted/13352212) is a StackOverflow post that explains it in relation to Javascript.

Namespace pollution can lead to _namespace clashes_. As you define more primary design units at the same scope, you are more likely to have two things that accidently have the same name. This is at the core the problem we are going to solve, because VHDL compilers and synthesizers are not built to gracefully handle clashes and will error out when a primary design unit at the same scope has multiple definitions.

In VHDL, a common example of a namespace clash is when different files define an entity by the same name, which may have different behaviors. Namespace clashes may start to appear when a higher-level IP requires the same entity from an IP but as different versions throughout its dependency tree.

## Solution - Dynamic Symbol Transformation

The proposed algorithm solves the namespace clashing problem by rewriting conflicts with a new unique identifier without losing information in the original identifier.

### Limitations
The user never must explicitly handle resolving duplicate identifiers due to the constraints set forth by Orbit when naming primary design units. The limitations are:
1. A primary design unit in the current IP may not have a duplicate name within its IP.
2. A primary design unit in the current IP may not have a duplicate name in a direct dependency of its IP.


## Example Demonstration

This section walks through a basic demonstration of the dynamic symbol transformation (DST) algorithm. First, it defines some terminology, and then walks through the algorithm's functionality.

### Symbols

Within the context of VHDL, let's consider a _symbol_ to be the identifier of a _primary design unit_. A primary design unit is a VHDL construct that exists at the global namespace. There are four primary design units:
- entity
- package
- configuration
- context

> __Note:__ VHDL does support the concept of _libraries_, which can add 1 level of nested scope to a primary design unit, but for this example we will assume the primary design units are defined within the same library/scope.

In the following code, the symbol `and_gate` corresponds to an entity.

``` vhdl
entity and_gate is
    port (
        a : in  bit;
        b : in  bit;
        c : out bit
    );
end entity;
```
file: _/lab1/and\_gate.vhd_

Remember that this identifier could appear again at the same namespace level as this exsiting entity in a different VHDL file.  

Now, imagine you are integrating VHDL code from various existing IPs. As you instantiate entities within larger entities, you realize there exists another entity named `and_gate` further down in the hierarchy, but this one has a different behavior than the previously defined `and_gate` circuit from the _/lab1_ directory.

``` vhdl
entity and_gate is
    port (
        a : in  bit_vector(3 downto 0);
        b : in  bit_vector(3 downto 0);
        c : out bit_vector(3 downto 0)
    );
end entity;
```
file: _/lab3/and\_gate.vhd_

Since the current IP requires both code segments, then traditionally your EDA tool would complain to you and be unable to resolve which `and_gate` to be used where. It then falls on the developer to rename one of the entities where it is defined and everywhere it is referenced, which introduces additional overhead in time and possibilities for errors. This problem is solved with DST.

### Walkthrough

Take this example dependency tree at the IP level:
```
final-project
├─ lab3
│  └─ lab2
|     └─ lab1
└─ lab2
```

Imagine the `final-project` IP has an entity called `top_level` which is the root of circuit hierarchy. From there, it reuses entities from the other IP.

Let's look at the VHDL entity hierarchy tree across the integrated IP:

```
top_level [final-project]
├─ and_gate [lab3]
│  └─ adder [lab2]
|     └─ and_gate [lab1]
└─ mux [lab2]
```

Notice lab1 and lab3 both have the `and_gate` entity, but their interfaces and functionality are internally different as previously mentioned. How can we allow both units in the hierarchy while resolving the namespace clash?

DST identifies namespace clashes within the current dependency graph and automatically resolve the conflicts to produce a clean unambiguous graph.

```
top_level [final-project]
├─ and_gate [lab3]
│  └─ adder [lab2]*
|     └─ and_gate_fbe4720d0 [lab1]*
└─ mux [lab2]
```

Let's take a closer look at what happened here. DST handled the namespace clash by _transforming_, or renaming, the entity related to lab1. The entity's identifier in lab1 was appended with the first 10 digits of the original lab1 IP's checksum. This transformation occurred at that IP's source code level (lab1), and in the source code for all dependents of that entity (lab2). Therefore, DST produced new dynamic variants of the lab1 and lab2 IPs that properly reference and associate with `and_gate_fbe4720d0`.

DST specifically chose not to rename the `and_gate` from lab3, or else the user would have to be burdened with trying to track and maintain the new unique identifier in the currently developed IP (final-project). As a result, DST has no additional overhead to the user and is kept abstracted away by Orbit. Direct dependencies are never chosen for DST.

## Emphasis

Dynamic symbol transformation lets Orbit avoid a major issue in package management: dependency hell. As projects and their dependencies become more complex, Orbit can continue to allow the user to integrate different verisons of the same package throughout the overall design. Conflicts in incompatible versions are avoided within the dependency graph. You can learn more about dependency hell [here](https://en.wikipedia.org/wiki/Dependency_hell).

## Further Reading

- https://stephencoakley.com/2019/04/24/how-rust-solved-dependency-hell