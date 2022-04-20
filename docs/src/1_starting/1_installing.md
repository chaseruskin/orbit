# Installing

There are two main methods for getting orbit running on your system: downloading a precompiled binary or using cargo.

## 1. Downloading a precompiled binary

1.  Visit Orbit's [releases](https://github.com/c-rus/orbit/releases) page on Github to find all official releases. 
2. Download the binary for your architecture and operating system.
3. Place the Orbit executable (`orbit` for unix and `orbit.exe` for windows) in a location recognized by the PATH environment variable.

There are multiple ways to accomplish step 3. The following outlines one way depending on the user's operating system. 

### Unix
1. Open a terminal to where Orbit was downloaded.
2. Unzip the prebuilt package.
```
$ unzip orbit-1.0.0-x86_64-macos.zip
```
3. Move the executable to a location already set in the PATH environment variable. 
```
$ mv ./orbit-1.0.0-x86_64-macos/bin/orbit /usr/local/bin/orbit
```

### Windows
1. Open a terminal (Powershell) to where Orbit was downloaded.
2. Unzip the prebuilt package.
```
$ expand-archive ".\orbit-1.0.0-x86_64-windows.zip"
```
3. Make a new directory to store this package.
```
new-item -path "$env:LOCALAPPDATA\Programs\orbit" -itemtype directory
```
4. Move the package to the new directory.
```
$ copy-item ".\orbit-1.0.0-x86_64-windows\*" -destination "$env:LOCALAPPDATA\Programs\orbit" -recurse
```
5. Edit the user-level PATH environment variable in ___Control Panel___ by adding __%LOCALAPPDATA%\Programs\orbit\bin__.

## 2. Installing with Cargo

To install with Cargo, enter the following command while replacing `<VERSION>` with the desired version tag (such as "1.0.0").
```
$ cargo install --git https://github.com/c-rus/orbit.git --tag <VERSION>
```

This will build the `orbit` binary and place it a path already set in the PATH environment variable.

## Checking Orbit is installed correctly

To verify orbit is working correctly on your system, open a new terminal and run:
```
$ orbit --version
```
This should print out your version of orbit you installed. Congratulations!