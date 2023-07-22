# First Project: Gates

## Setting Up Our Environment

Create a new directory to hold projects we will develop:
```
mkdir -p "~/ks-tech/hdl"
```

Set that path as your DEV_PATH within Orbit:
```
orbit config --set core.path="~/ks-tech/hdl"
```

Set your name:
```
orbit config --set core.user="Kepler [G-01]"
```

Set a text editor program:
```
orbit config --set core.editor="code"
```

Let's pull down our first profile to help us get up and running faster.

1. Clone the repository to your .orbit home folder:
```
git clone https://github.com/kepler-space-tech/orbit-profile.git "$(orbit env ORBIT_HOME)/profiles/ks-tech"
```

2. Link the profile's configuration file to your global configuration file:
```
orbit config --append include="$(orbit env ORBIT_HOME)/profiles/ks-tech/config.toml"
```

## Creating the Project

Lets create our first project! We, _KS-Tech_, want a HDL library called _rary_, that holds some _gate_ components for us. It's fitting for us to give it the title: `ks-tech.rary.gates`. This title is the _pkgid_ of the ip, split by \<vendor>.\<library>.\<name>.

Before we create the ip, let's view what templates available to import:
```
$ orbit new ks-tech.rary.gates --list
Templates:
    ks-tech         basic project structure with rtl and simulation layout

```

Let's use the template called `ks-tech` for this project:
```
$ orbit new ks-tech.rary.gates --path gates --template ks-tech
info: new ip created at C:/users/chase/ks-tech/hdl/gates
```

Notice the `--path` was a relative path so it was joined to the DEV_PATH we set earlier.

Verify the project is found in our IP catalog:

```
$ orbit search
Vendor         Library        Name                Status   
-------------- -------------- ------------------- --------   
ks-tech        rary           gates               D      

```

## Edit the Project

Open the newly created project with the configured text editor:
```
$ orbit edit gates
```

> __Note:__ We only specified `gates` instead of the full name because Orbit is able to deduce the target ip from only supplying the `gates` part.

The directory structure should resemble the following:
```
gates/
├─ README.md
├─ Orbit.toml
├─ rtl/
│  └─ gates.vhd
└─ sim/
   └─ gates_tb.vhd
```