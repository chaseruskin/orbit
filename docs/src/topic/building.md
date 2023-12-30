# Building

The _build stage_ is the final step (2/2) in `orbit`'s run system. Building refers to the process of calling a plugin within `orbit` to perform a workflow. This step occurs after the planning step. 

## Plugin execution

A plugin starts its execution within the build directory. Therefore, all relative file paths inside the plugin's script will always be relative to the build directory. 