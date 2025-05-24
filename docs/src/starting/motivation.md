# Why Orbit Exists

If you work in the realm of computers, you may hear the phrase: "_Hardware is hard_." But have you ever thought about __why__?

There are many possible answers that may lead to this phrase's existence, such as hardware involves battling real-world physics, or that hardware requires knowledge across a wide range of skills, or that hardware has infinitely many ways to go wrong. While all of the aforementioned reasons validate this claim, there is one particular reason that should cease to exist in the context of digital hardware: __a slow and expensive development cycle__.

## Background

To describe hardware today, you typically use a hardware description language (HDL). The two most promiment HDLs used and supported by electronic design automation (EDA) tools are VHDL and Verilog. HDLs are not software programing languages, but rather a language to describe how a circuit is intended to be structured or function.

The core similarity between describing digital hardware and writing software programs is that the developer must apply the creative process by editing files called _source code_. Source code is used as an input to a back end process, whether it be compilation, simulation, or synthesis. Software and hardware encounter similar problems when working with source code, although software arguably has better tools that already do a good job in solving some of their problems. 

Let's talk about some observations about source code development in the layer above hardware: software. Software development today not only has nice languages to develop with, but the entire ecosystem surrounding that language is a pleasant experience. Take the Rust programming language as an example. Not only does Rust have moden language constructs that forces programmers to rethink how to write their program in a safer way regarding computer memory, but the infrastructure surrounding Rust is very accessible and easy to use, which involves the compiler, code management, and code testing. This is great for software development, but does the same hold true for hardware development?

## The problem

At the hardware layer, life is unfortunately not so simple. Not only are the promiment languages being used for things outside of their original intention, but the surrounding infrastructure around these languages is very lacking. Proprietary and expensive tools force developers to adapt to non-standardized practices, making it difficult to share code across systems or other tools, leading to _vendor lock-in_. Vendor lock-in fragments the community and makes it increasingly difficult to find portable and easily usable source code. Hardware does not have as big of a footprint in the open-source world in comparison to software's vibrant open-source community.

Without the right infrastructure and tools to easily share, reuse, and maintain source code, hardware experiences slow and expensive development cycles. At this point, you be wondering what are the right tools and infrastructure that hardware needs? To answer this question, you may have to shift your perspective about hardware. Thinking of hardware with the waterfall approach of being _built and done with_ is the wrong mindset. Hardware is rarely built once and forgotten about. Instead, one must think of hardware with a more modern approach in that it evolves and improves upon many iterations. This concept of hardware evolving over time requires special attention when creating the supportive infrastructure around it. While this infrastructure and tooling exists in the software development world, unfortunately it is largely missing for the common HDL languages such as VHDL and Verilog.

Without the right infrastructure and tools, many source code maintenance tasks related to development, also known as _technical debt_, become exponentially more time-consuming and increasingly difficult as time goes on and the code base grows in size and complexity. The technical debt falls upon the developer to handle manually because there is no automated systematic tool that can properly handle it instead. Large technical debt in turn leads to long and expensive hardware development cycles, and therefore it is a common goal to strive to minimize technical debt. The solution at this point is not to continue down the dark path ahead, but instead to take a look at how software managed to minimize this problem: package management.

## The approach

With the right tools, hardware development can experience faster development cycles and lower development costs. A tool that can make this experience true is a _package manager_. A package manager's role is to organize and automate the tasks related to source code management. A well-designed package manager can minimize technical debt, while a poorly-designed package manager can fail to minimize technical debt and instead shift it to different tasks. Software programming languages have become increasingly good at handling source code management through the creation of their own package managers, such as [go mod](https://go.dev/ref/mod) for Go, [Cargo](https://doc.rust-lang.org/cargo/) for Rust, and [pip](https://pip.pypa.io/en/stable/) for Python.

> Having well-designed and easily accessible tools is important to any form of development. Creativity's greatest limiting factor is the tools of which are available to turn an idea into a reality.

## Enter: Orbit

_Orbit_ is an attempt at bringing state of the art package management to HDLs. It automates the process of organizing, maintaining, and reusing HDL source code across projects. It is a tool that allows HDL projects to declare various dependencies and ensure that you'll always get a repeatable build. To accomplish this goal, Orbit does four things:

- Introduces two metadata files with various bits of ip information.
- Fetches and installs your ip's dependencies.
- Invokes any user-defined build tool to adapt to your workflow and EDA tools
- Introduces conventions to make working with HDL projects easier. 

To a large extent, Orbit normalizes the commands needed to build or test a given ip; this is one aspect of the aforementioned conventions. The same command can be used to build a project for any number of EDA tools. Furthermore, Orbit will automatically fetch any dependencies you have defined for your current project and arrange for them to be included in your build as needed.

One of the major goals behind Orbit is to provide the right infrastructure and ecosystem for anyone's hardware projects to live and evolve over time. By introducing Orbit into your workflow, we believe hardware development becomes a more enjoyable experience where more of your time is devoted to _actually_ creating cool things, not racking up technical debt and fighting to manage millions of files. After all, the saying should be "_Hardware is cool_", right?