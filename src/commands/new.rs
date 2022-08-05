use colored::Colorize;

use crate::Command;
use crate::FromCli;
use crate::core::catalog::Catalog;
use crate::core::manifest::IpManifest;
use crate::core::template::TemplateFile;
use crate::core::variable::VariableTable;
use crate::interface::cli::Cli;
use crate::interface::arg::{Positional, Optional, Flag};
use crate::interface::errors::CliError;
use crate::core::pkgid::PkgId;
use crate::interface::arg::Arg;
use crate::core::context::Context;
use crate::util::anyerror::Fault;
use crate::util::environment::Environment;
use crate::util::filesystem;
use std::error::Error;
use std::path::PathBuf;
use crate::util::anyerror::AnyError;
use crate::core::template::Template;

#[derive(Debug, PartialEq)]
pub struct New {
    ip: Option<PkgId>,
    to: Option<PathBuf>,
    template: Option<String>,
    list: bool,
    file: bool,
    from: Option<PathBuf>,
}

impl FromCli for New {
    fn from_cli<'c>(cli: &'c mut Cli) -> Result<Self,  CliError<'c>> {
        cli.set_help(HELP);
        let command = Ok(New {
            to: cli.check_option(Optional::new("to").value("path"))?,
            list: cli.check_flag(Flag::new("list"))?,
            from: cli.check_option(Optional::new("from").value("path"))?,
            file: cli.check_flag(Flag::new("file"))?,
            template: cli.check_option(Optional::new("template").value("alias"))?,
            ip: cli.check_option(Optional::new("ip"))?,
        });
        command
    }
}

impl Command for New {
    type Err = Box<dyn Error>;
    fn exec(&self, c: &Context) -> Result<(), Self::Err> {
        // verify the template exists
        let template = if let Some(alias) = &self.template {
            match c.get_templates().get(alias) {
                Some(t) => Some(t),
                None => return Err(AnyError(format!("template '{}' does not exist", alias)))?
            }
        } else {
            None
        };

        // view templates
        if self.list == true {
            match template {
                Some(t) => {
                    t.display_files()
                },
                None => {
                    println!("{}", Template::list_templates(&c.get_templates().values().into_iter().collect::<Vec<&Template>>()))
                }
            }
            return Ok(())
        }

        // user must either provide an pkgid or use --file flag (not both)
        if self.ip.is_some() && self.file == true {
            return Err(AnyError(format!("cannot create new ip with '{}' and file with '{}' at the same time", "--ip".yellow(), "--file".yellow())))?
        }

        // load base-line variables
        let vars = VariableTable::new()
            .load_context(&c)?
            .load_environment(&Environment::new().from_config(c.get_config())?)?;

        // create a new file
        if self.file == true {
            c.goto_ip_path()?;
            // fail if no '--to' was specified
            if self.to.is_none() {
                return Err(AnyError(format!("creating a file requires a destination path with '{}'", "--to".yellow())))?
            }
            // filepath to place copied contents
            let dest = c.get_ip_path().unwrap().join(self.to.as_ref().unwrap());

            // fail is destination already exists and not forcing
            if dest.exists() == true && c.force == false {
                return Err(AnyError(format!("destination {} already exists; use '{}' to overwrite", filesystem::normalize_path(PathBuf::from(self.to.as_ref().unwrap())).display(), "--force".yellow())))?
            }
       
            let ip = IpManifest::from_path(&std::env::current_dir().unwrap())?;

            let vars = {
                // load variables for the current ip
                let mut vars = vars.load_pkgid(ip.get_pkgid())?;
                // add orbit.filename as variable
                vars.add("orbit.filename", dest.file_stem().unwrap().to_str().unwrap());
                vars
            };

            // load variables for the filename
            self.new_file(template, &vars, &dest)
        // create a new ip
        } else if let Some(ip) = &self.ip {
            // verify '--from' is not combined with '--template'
            if self.from.is_some() && self.template.is_some() {
                return Err(AnyError(format!("cannot copy path with '{}' and import template with '{}' at the same time", "--from".yellow(), "--template".yellow())))?
            }

            // extra validation for a new IP spec to contain all fields (V.L.N)
            if let Err(e) = ip.fully_qualified() {
                return Err(CliError::BadType(Arg::Positional(Positional::new("ip")), e.to_string()))?
            }
            let root = c.get_development_path().unwrap();
            // verify the pkgid is not taken
            {
                let catalog = Catalog::new()
                    .development(c.get_development_path().unwrap())?
                    .installations(c.get_cache_path())?
                    .available(c.get_vendors())?;
                if catalog.inner().contains_key(&ip) == true {
                    return Err(AnyError(format!("ip pkgid '{}' already taken", ip)))?
                }
            }
            // load variables for the new ip
            let mut vars = vars.load_pkgid(&ip)?;
            // only pass in necessary variables from context
            self.run(root, c.force, template, &mut vars)
        // what is default behavior? (currently undefined)
        } else {
            Err(AnyError(format!("nothing specified to create; use {} or {}\n\nFor more information try {}", "--ip".yellow(), "--file".yellow(), "--help".green())))?
        }
    }
}

impl New {
    /// Creates a new file. 
    /// 
    /// If pulling from a template, a source filepath must be defined with --from.
    /// If not using a template, then it will copy from the actually provided filepath from --from.
    /// If there is no source and no template, then it will create a new blank file at `dest`.
    fn new_file(&self, template: Option<&Template>, lut: &VariableTable, dest: &PathBuf) -> Result<(), Fault> {
        // check if we are pulling from a template
        if let Some(tplate) = template {
            match &self.from {
                Some(p) => { 
                    // verify path exists in template
                    let src = PathBuf::from(tplate.path()).join(&p);

                    if src.exists() == false {
                        return Err(AnyError(format!("relative file path '{0}' does not exist in template '{1}'\n\nTry `orbit new --file --template {1} --list` to see available files", filesystem::normalize_path(p.to_path_buf()).display(), template.unwrap().alias())))?
                    }
                    // create all missing directories on destination side
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(&parent)?;
                    }
                    // copy the file using import
                    std::fs::copy(&src, &dest)?;

                    // create template file
                    let tfile = TemplateFile::new(&dest);
                    // perform variable substitution
                    tfile.substitute(&lut)?;
                    return Ok(())
                }
                 // issue error if no 'from' specified but 'template' was specified
                None => {
                    // print error with help message to view available files
                    return Err(AnyError(format!("missing file to import from template '{1}' with option '{0}'\n\nTry `orbit new --file --template {1} --list` to see available files", "--from".yellow(), template.unwrap().alias())))?
                }
            }
        }
        // use from as a copy from relative path without a template
        match &self.from {
            // copy from file
            Some(src) => {
                std::fs::copy(&src, &dest)?;
                // create template file to perform variable substitution
                let tfile = TemplateFile::new(&dest);
                tfile.substitute(&lut)?;
            }
            // create a new blank file
            None => {
                std::fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(&dest)?;
            }
        }
        Ok(())
    }

    fn run(&self, root: &std::path::PathBuf, force: bool, template: Option<&Template>, lut: &mut VariableTable) -> Result<(), Fault> {
        // create ip stemming from DEV_PATH with default /VENDOR/LIBRARY/NAME
        let ip_path = if self.to.is_none() {
            root.join(self.ip.as_ref().unwrap().get_vendor().as_ref().unwrap())
                .join(self.ip.as_ref().unwrap().get_library().as_ref().unwrap())
                .join(self.ip.as_ref().unwrap().get_name())
        } else {
            root.join(self.to.as_ref().unwrap())
        };

        // verify the from path works out
        if let Some(src) = &self.from {
            if src.exists() == false {
                return Err(AnyError(format!("source path {} does not exist", filesystem::normalize_path(src.to_path_buf()).display())))?
            }
            if src.is_dir() == false {
                return Err(AnyError(format!("source path {} is not a directory", filesystem::normalize_path(src.to_path_buf()).display())))?
            }
        }

        // verify the ip would exist alone on this path (cannot nest IPs)
        {
            // go to the very tip existing parent of the path specified
            let path_clone = { 
                let mut path_c = ip_path.clone();
                while path_c.exists() == false {
                    path_c.pop();
                }
                path_c
            };
            // verify there are no current IPs living on this path
            if let Some(other_path) = Context::find_ip_path(&path_clone) {
                return Err(AnyError(format!("ip already exists at path {}", other_path.display())))?
            }
        }

        let ip = IpManifest::create(ip_path, &self.ip.as_ref().unwrap(), force, false)?;
        let root = ip.get_root();

        // import template if found
        if let Some(t) = template {
            t.import(&root, lut)?;
        } else if let Some(src) = &self.from {
            // act as if the from path is a template to allow for variable substitution
            let tplate_base = filesystem::resolve_rel_path(&std::env::current_dir().unwrap(), src.to_str().unwrap().to_string());
            let tplate = Template::from_path(tplate_base);
            tplate.import(&root, lut)?;
        }

        // @TODO issue warning if the ip path is outside of the dev path or dev path is not set
        println!("info: new ip created at {}", root.display());
        Ok(())
    }
}

const HELP: &str = "\
Create a new orbit ip project.

Usage:
    orbit new [options]

Options:
    --ip <pkgid>        the V.L.N for the new project
    --template <alias>  specify a template to import
    --to <path>         set the destination path
    --file              create a file rather than an ip
    --from <path>       specify a source path to copy
    --list              view available templates
    --force             overwrite the existing destination

Use 'orbit help new' to read more about the command.
";

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
