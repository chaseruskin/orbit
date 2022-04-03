# Installing

There are two main methods for getting orbit running on your system: downloading a precompiled binary or using cargo.

## 1. Downloading a precompiled binary

1.  Visit to the [releases](https://github.com/c-rus/orbit/releases) page on Orbit's Github to find all official releases. 
2. Download the binary for your operating system.
3. Place orbit in a location recognized by the PATH environment variable.

There are multiple ways to accomplish step 3. The following outlines one way depending on the user's operating system. 

### Unix
1. Open a terminal to where Orbit was downloaded.
2. Move the executable to a location already set in the PATH environment variable:  
```
$ mv orbit /usr/local/bin/orbit
```

### Windows
1. Open a terminal (Powershell) to where Orbit was downloaded.
2. Make a new directory:  
```
$ mkdir "$env:LOCALAPPDATA\Programs\orbit\bin"
```
3. Move the executable to the new directory:
```
$ mv orbit.exe "$env:LOCALAPPDATA\Programs\orbit\bin\orbit.exe"
```
4. Edit the user-level PATH environment variable in ___Control Panel___ by adding __%LOCALAPPDATA%\Programs\orbit\bin__.

## 2. Installing with Cargo

> __Note__: This method is currently unavailable until Orbit is on [crates.io](https://crates.io).

## Checking Orbit is installed correctly

To verify orbit is working correctly on your system, open a new terminal and run:
```
$ orbit --version
```
This should print out your version of orbit you installed. Congratulations!