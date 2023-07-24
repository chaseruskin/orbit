# Motion 0x02
Chase Ruskin  
2022/02/09

## The role of an HDL package manager

As we've previously discussed, a package manager is a development tool that helps speed up the development process by automating the difficulties of managing codebases. This is the first and most important goal for an HDL package manager to have. This requires a system in place where users can install, update, download, and uninstall projects across the entire codebase, ideally needing few commands and no hassle. A developer's main role is write code. By freeing the developer of these  management responsibilities, they can allocate more time and resources toward writing code.

The development cycle is the entire process one takes from outlining a project and its specification, creating that project, writing the code for the specified functionality, testing and verifying that functionality, and then publishing that project to be implemented onto real physical systems or to be reused in other projects as a dependency. Some stages are longer than others, and it is rarely an straightforward process.

Package managers are designed to speed up this development process in all areas requiring management. Ironically, management can be found in every stage of the development process: creating projects, bringing in dependencies, building projects across different tools, and sharing its code.

A package manager should be the single point-of-contact in the development cycle because it is intended to _oversee_ the hidden scattered strings behind keeping a project intact. It abstracts how it accomplishes its goals to the developer so they can remain focused on writing code. Being a single point-of-contact keeps the developer's life simple by requiring only one interface that gives access to unbeknownst superpowers in the realm of management.

## The development cycle through a management lens

The development process through a management lens:

- project creation
    - how are your projects structured? Is there a standard or pattern?
- dependency handling
    - how can you use files in multiple projects? What about versions?
- development
    - how can you get to developing faster? Is there a pattern in the code you write?
- project building
    - how can you access backend tools to be applied your project?
- project sharing
    - how can others retrieve my code to be used?
- configuration sharing
    - how can you ensure your team is working under the same environment?