# Installing

There are two main methods for getting orbit running on your system: downloading a precompiled binary or using cargo.

> __Reminder:__ By installing and using Orbit, you accept usage under its GPL-3.0 license.

## 1. Using a precompiled binary

1.  Visit Orbit's [releases](https://github.com/c-rus/orbit/releases) page on Github to find all official releases. 
2. Download the binary for your architecture and operating system.
3. Install Orbit. Either run the provided `install` executable or follow the manual instructions for placing the Orbit executable (`orbit` for unix and `orbit.exe` for windows) in a location recognized by the PATH environment variable.

There are multiple solutions to accomplish step 3. The following outlines one way to manually install orbit depending on the user's operating system. 

### Unix
1. Open a terminal to where Orbit was downloaded.
2. Unzip the prebuilt package.
```
$ unzip orbit-CARGO_CRATE_VERSION-x86_64-macos.zip
```
3. Move the executable to a location already set in the PATH environment variable. 
```
$ mv ./orbit-CARGO_CRATE_VERSION-x86_64-macos/bin/orbit /usr/local/bin/orbit
```

### Windows
1. Open a terminal (Powershell) to where Orbit was downloaded.

2. Unzip the prebuilt package.
```
expand-archive "./orbit-CARGO_CRATE_VERSION-x86_64-windows.zip"
```

3. Make a new directory to store this package.
```
new-item -path "$env:LOCALAPPDATA/Programs/orbit" -itemtype directory
```

4. Move the package to the new directory.
```
copy-item "./orbit-CARGO_CRATE_VERSION-x86_64-windows/*" -destination "$env:LOCALAPPDATA/Programs/orbit" -recurse
```

5. Edit the user-level PATH environment variable in ___Control Panel___ by adding __%LOCALAPPDATA%\Programs\orbit\bin__.

## 2. Installing with Cargo

To install the latest version through Cargo:
```
cargo install --git https://github.com/c-rus/orbit.git --tag CARGO_CRATE_VERSION
```

This will build the `orbit` binary and place it a path already set in the PATH environment variable.

## Checking if Orbit is installed correctly

To verify orbit is working correctly on your system, open a new terminal and run:
```
orbit --version
```
This should print out your version of orbit you installed. Congratulations!