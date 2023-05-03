//! The installation process:
//! 1. Optionally ask for an ip to install (default: current working ip)
//! --!-- Get the folder and change directories to the folder requiring installation --!--
//! * at the point in the process when the program is at the folder, it is assumed all sub-deps are also already installed
//! 2. Write results from computing the available units for the package
//! 3. Write results for information about accessing C,I,S,A
//! 3. Verify a .lock file is available (is this needed? - do dependents even read from this?)
//! 4. Move only relevant files to artifact directory (no .git/, etc.)
//! 5. Compute checksum on entire directory
//! 6. Zip contents and store in "store" for future re-installation
//! 7. Place artifact directory in "cache" for catalog lookup