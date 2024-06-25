use crate::commands::helps::new;
use crate::commands::orbit::AnyResult;
use crate::core::context::Context;
use crate::core::ip::Ip;
use crate::core::manifest::{Manifest, IP_MANIFEST_FILE};
use crate::core::pkgid::PkgPart;
use crate::error::{Error, Hint, LastError};
use crate::util::filesystem::{Standardize, ORBIT_IGNORE_FILE};
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use cliproc::{cli, proc, stage::*};
use cliproc::{Arg, Cli, Help, Subcommand};

#[derive(Debug, PartialEq)]
pub struct New {
    /// Specify where to create the new ip on the local machine.
    path: PathBuf,
    /// Optionally give the name for the ip, by default tries to be the parent folder's name.
    name: Option<PkgPart>,
    /// Create an ip directory with an `Orbit.toml` manifest file.
    is_ip: bool,
    // /// Overwrite any existing manifest at the given directory and do not error if the directory exists.
    // force: bool,
}

impl Subcommand<Context> for New {
    fn interpret(cli: &mut Cli<Memory>) -> cli::Result<Self> {
        cli.help(Help::with(new::HELP))?;
        Ok(Self {
            is_ip: cli.check(Arg::flag("ip"))?,
            name: cli.get(Arg::option("name"))?,
            path: cli.require(Arg::positional("path"))?,
        })
    }

    fn execute(self, c: &Context) -> proc::Result {
        // verify we are not already in an ip directory
        {
            // resolve any relative path
            let dest = PathBuf::standardize(self.path.clone());
            if let Some(p) = Context::find_ip_path(&dest) {
                return Err(Error::IpExistsAtPath(p))?;
            }
        }

        // verify the path does not exist
        if self.path.exists() == true {
            // @todo give user more helpful error message
            // 1. if the manifest already exists at this directory
            // 2. if no manifest already exists at this directory
            return Err(Error::PathAlreadyExists(
                self.path.clone(),
                Hint::InitNotNew,
            ))?;
        }

        let ip_name = Self::extract_name(self.name.as_ref(), &self.path)?;

        match self.create_ip(&ip_name, &c.get_target_dir()) {
            Ok(r) => Ok(r),
            Err(e) => Err(Error::FailedToCreateNewIp(LastError(e.to_string())))?,
        }
    }
}

impl New {
    /// Determines the final name to use for the ip based on the given `name` and falls back
    /// to the `path`'s file name if not `name` is given.
    pub fn extract_name<'a>(
        name: Option<&'a PkgPart>,
        path: &PathBuf,
    ) -> AnyResult<Cow<'a, PkgPart>> {
        match name {
            Some(n) => Ok(Cow::Borrowed(n)),
            // try to use the path's ending name as the ip name
            None => match path.file_name() {
                Some(fname) => {
                    let s = fname.to_string_lossy();
                    match PkgPart::from_str(s.as_ref()) {
                        Ok(r) => Ok(Cow::Owned(r)),
                        Err(e) => Err(Error::CannotAutoExtractNameFromPath(
                            s.to_string(),
                            LastError(e.to_string()),
                            Hint::IpNameSeparate,
                        ))?,
                    }
                }
                None => Err(Error::MissingFileSystemPathName(
                    path.clone(),
                    Hint::IpNameSeparate,
                ))?,
            },
        }
    }
}

impl New {
    /// Creates a new directory at the given `dest` with a new manifest file.
    fn create_ip(&self, ip: &PkgPart, target_dir: &str) -> AnyResult<()> {
        // create the directory
        std::fs::create_dir_all(&self.path)?;

        // create the file directly nested within the destination path
        let manifest_path = {
            let mut p = self.path.clone();
            p.push(IP_MANIFEST_FILE);
            p
        };

        let ignore_path = {
            let mut p = self.path.clone();
            p.push(ORBIT_IGNORE_FILE);
            p
        };

        let mut manifest = std::fs::File::create(&manifest_path)?;
        let mut ignore = std::fs::File::create(&ignore_path)?;
        manifest.write_all(Manifest::write_empty_manifest(&ip).as_bytes())?;
        ignore.write_all(Ip::write_default_ignore_file(target_dir).as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ut_extract_name() {
        let name = None;
        let path = PathBuf::from("gates");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("gates").unwrap()
        );

        let name = Some(PkgPart::from_str("sha256").unwrap());
        let path = PathBuf::from("gates");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("sha256").unwrap()
        );

        let name = None;
        let path = PathBuf::from("./a/long/path/to/project");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("project").unwrap()
        );

        let name = None;
        let path = PathBuf::from("./a/long/path/to/Project/");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("Project").unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn ut_extract_name_no_file_name() {
        let name = None;
        let path = PathBuf::from(".");
        assert_eq!(
            New::extract_name(name.as_ref(), &path).unwrap().as_ref(),
            &PkgPart::from_str("sha256").unwrap()
        );
    }
}

// /// Creates a new file.
// ///
// /// If pulling from a template, a source filepath must be defined with --from.
// /// If not using a template, then it will copy from the actually provided filepath from --from.
// /// If there is no source and no template, then it will create a new blank file at `dest`.
// fn new_file(&self, template: Option<&Template>, lut: &VariableTable, dest: &PathBuf) -> Result<(), Fault> {
//     // check if we are pulling from a template
//     if let Some(tplate) = template {
//         match &self.from {
//             Some(p) => {
//                 // verify path exists in template
//                 let src = PathBuf::from(tplate.path()).join(&p);

//                 if src.exists() == false {
//                     return Err(AnyError(format!("relative file path '{0}' does not exist in template '{1}'\n\nTry `orbit new --file --template {1} --list` to see available files", filesystem::normalize_path(p.to_path_buf()).display(), template.unwrap().alias())))?
//                 }
//                 // create all missing directories on destination side
//                 if let Some(parent) = dest.parent() {
//                     std::fs::create_dir_all(&parent)?;
//                 }
//                 // copy the file using import
//                 std::fs::copy(&src, &dest)?;

//                 // create template file
//                 let tfile = TemplateFile::new(&dest);
//                 // perform variable substitution
//                 tfile.substitute(&lut)?;
//                 return Ok(())
//             }
//              // issue error if no 'from' specified but 'template' was specified
//             None => {
//                 // print error with help message to view available files
//                 return Err(AnyError(format!("missing file to import from template '{1}' with option '{0}'\n\nTry `orbit new --file --template {1} --list` to see available files", "--from".yellow(), template.unwrap().alias())))?
//             }
//         }
//     }
//     // use from as a copy from relative path without a template
//     match &self.from {
//         // copy from file
//         Some(src) => {
//             std::fs::copy(&src, &dest)?;
//             // create template file to perform variable substitution
//             let tfile = TemplateFile::new(&dest);
//             tfile.substitute(&lut)?;
//         }
//         // create a new blank file
//         None => {
//             std::fs::OpenOptions::new()
//                 .write(true)
//                 .truncate(true)
//                 .create(true)
//                 .open(&dest)?;
//         }
//     }
//     Ok(())
// }

// fn run(&self, root: &std::path::PathBuf, template: Option<&Template>, lut: &mut VariableTable) -> Result<(), Fault> {
//     // create ip stemming from DEV_PATH with default /VENDOR/LIBRARY/NAME
//     let ip_path = if self.to.is_none() {
//         root.join(self.ip.as_ref().unwrap().get_vendor().as_ref().unwrap())
//             .join(self.ip.as_ref().unwrap().get_library().as_ref().unwrap())
//             .join(self.ip.as_ref().unwrap().get_name())
//     } else {
//         root.join(self.to.as_ref().unwrap())
//     };

//     // verify the from path works out
//     if let Some(src) = &self.from {
//         if src.exists() == false {
//             return Err(AnyError(format!("source path {} does not exist", filesystem::normalize_path(src.to_path_buf()).display())))?
//         }
//         if src.is_dir() == false {
//             return Err(AnyError(format!("source path {} is not a directory", filesystem::normalize_path(src.to_path_buf()).display())))?
//         }
//     }

//     // verify the ip would exist alone on this path (cannot nest IPs)
//     {
//         // go to the very tip existing parent of the path specified
//         let path_clone = {
//             let mut path_c = ip_path.clone();
//             while path_c.exists() == false {
//                 path_c.pop();
//             }
//             path_c
//         };
//         // verify there are no current IPs living on this path
//         if let Some(other_path) = Context::find_ip_path(&path_clone) {
//             return Err(AnyError(format!("ip already exists at path {}", other_path.display())))?
//         }
//     }

//     let ip = IpManifest::create(ip_path, &self.ip.as_ref().unwrap(), self.force, false)?;
//     let root = ip.get_root();

//     // import template if found
//     if let Some(t) = template {
//         t.import(&root, lut)?;
//     } else if let Some(src) = &self.from {
//         // act as if the from path is a template to allow for variable substitution
//         let tplate_base = filesystem::resolve_rel_path(&std::env::current_dir().unwrap(), src.to_str().unwrap());
//         let tplate = Template::from_path(tplate_base);
//         tplate.import(&root, lut)?;
//     }

//     // @TODO issue warning if the ip path is outside of the dev path or dev path is not set
//     println!("info: new ip created at {}", root.display());
//     Ok(())
// }

// orbit new --file --to sim/reg_tb.vhd --template wrasd --from extra/handshake_tb.vhd

// orbit new --file --template wrasd --list -> displays available files from template wrasd
// orbit new --list
// orbit new --list -> displays templates

// orbit new --ip ven.lib.project_c --to project_c --template wrasd

// using --file flag can only be called from an orbit ip
// the relative 'to' path is created from ip's root

// the relative 'from' path is joined with template base path

// 'from' can also be used with --ip to copy a directory (can be any directory and will write a new Orbit.toml file)
// 'from' and 'template' cannot be used at same time for `--ip` (similiar to how you cant use --plugin and --command on same build)
