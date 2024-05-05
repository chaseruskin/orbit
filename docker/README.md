# `docker`

This folder contains Dockerfiles to automate the creation of docker images to be used with Orbit.

There are two types of images that are built: _stand-alone_ and _integrated_.

## Stand-alone images

Stand-alone images are Docker images with `orbit` ready-to-use on a base operating system with minimum runtime dependencies.

Operating System | `orbit` Tags
-- | --
ubuntu 18.04 | latest
ubuntu 22.04 | latest

## Integrated images

Integrated images are Docker images with `orbit` ready-to-use alongside other common hardware development tools for out-of-the-box functionality. Because these images contain a variety of different tools, which may vary by version, a development naming scheme "_adjective_"-"_musical noun_" is enforced.

Images | Operating System | Tools
-- | -- | --
`groovy-guitar` | ubuntu 22.04 | orbit, python, ghdl
`melodic-marimba` | ubuntu 18.04 | orbit, python, modelsim-intel
`quiet-quartet` | ubuntu 18.04 | orbit, python, quartus-prime-lite