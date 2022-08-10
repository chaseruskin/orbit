use std::path;
use std::env;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::core::plugin::Plugin;
use crate::core::config::FromToml;
use crate::core::config::Config;
use crate::util::anyerror::AnyError;
use crate::util::anyerror::Fault;
use crate::core::template::Template;
use crate::util::filesystem;
use crate::util::filesystem::normalize_path;

use super::config::CONFIG_FILE;
use super::pkgid::PkgPart;
use super::vendor::VendorManifest;


const ORBIT_WIN_LITERAL_CMD: &str = "ORBIT_WIN_LITERAL_CMD";

pub struct Context {
    /// holds behind-the-scenes internal Orbit operations
    home_path: path::PathBuf,
    /// holds installed immutable tags of git repositories
    cache_path: path::PathBuf,
    /// the parent path to the current ip Orbit.toml file
    ip_path: Option<path::PathBuf>,
    /// holds in-development mutable ip projects
    dev_path: Option<path::PathBuf>,
    /// holds installed immutable git repositories to pull versions from into cache
    store_path: path::PathBuf, 
    /// temporary throwaway directory     
    build_dir: String,
    config: Config,
    plugins: HashMap<String, Plugin>, // @IDEA optionally move hashmap out of context and create it from fn to allow dynamic loading
    templates: HashMap<String, Template>,
    vendors: HashMap<PkgPart, VendorManifest>,
    pub force: bool,
}

impl Context {
    pub fn new() -> Context {
        let home = std::env::temp_dir();
        let cache = home.join("cache");
        let store = home.join("store");
        Context { 
            home_path: home,
            cache_path: cache,
            store_path: store,
            ip_path: None,
            dev_path: None,
            plugins: HashMap::new(),
            templates: HashMap::new(),
            config: Config::new(),
            build_dir: String::new(),
            force: false,
            vendors: HashMap::new(),
        }
    }

    pub fn retain_options(mut self, force: bool) -> Context {
        self.force = force;
        self
    }

    /// Sets the home directory. By default this is `$HOME/.orbit`. If set by `var`,
    /// it must be an existing directory.
    pub fn home(mut self, key: &str) -> Result<Context, ContextError> {
        self.home_path = if let Ok(s) = env::var(key) {
            std::path::PathBuf::from(s)
        } else {
            let hp = match home::home_dir() {
                Some(p) => p.join(".orbit"),
                None => return Err(ContextError(format!("failed to detect user's home directory; please set the ORBIT_HOME environment variable")))
            };
            // create the directory if does not exist
            if path::Path::exists(&hp) == false {
                std::fs::create_dir(&hp).expect("failed to create .orbit directory");
            }
            hp
        };
        // do not allow a non-existent directory to be set for the home
        if path::Path::exists(&self.home_path) == false {
            return Err(ContextError(format!("directory {} does not exist for ORBIT_HOME", self.home_path.display())))
        }
        // verify the environment variable is set
        env::set_var(key, &self.home_path);
        Ok(self)
    }

    /// Sets the store directory. If it was set from `var`, it assumes the path
    /// exists. If setting by default (within HOME), it assumes HOME is already existing.
    pub fn store(mut self, key: &str) -> Result<Context, Fault> {
        self.store_path = self.folder(key, "store")?;
        Ok(self)
    }

    /// Sets the cache directory. If it was set from `var`, it assumes the path
    /// exists. If setting by default (within HOME), it assumes HOME is already existing.
    pub fn cache(mut self, key: &str) -> Result<Context, Fault> {
        self.cache_path = self.folder(key, "cache")?;
        Ok(self)
    }

    /// Checks if windows literal command is enabled.
    pub fn enable_windows_bat_file_match() -> bool {
        if cfg!(target_os = "windows") {
            // by not finding the env var, the windows batch file match is enabled
            std::env::var(ORBIT_WIN_LITERAL_CMD).is_err()
        } else {
            false
        }
    }

    /// Returns an existing filesystem path to be used under `key`.
    /// 
    /// Uses `key`'s value if already explicitly set and will set the environment
    /// variable accordingly.
    fn folder(&self, key: &str, folder: &str) -> Result<PathBuf, Fault> {
        // prioritize explicit variable setting
        let dir = if let Ok(s) = env::var(key) {
            let ep = PathBuf::from(s);
            // verify the path exists
            if ep.exists() == false {
                return Err(AnyError(format!("directory {} does not exist for {}", ep.display(), key)))?
            }
            // verify the path is a directory
            if ep.is_dir() == false {
                return Err(AnyError(format!("{} must be a filesystem directory", key)))?
            }
            ep
        // proceed with default
        } else {
            let ep = self.home_path.join(&folder);
            // create the directory if does not exist
            if ep.exists() == false {
                std::fs::create_dir(&ep).expect(&format!("failed to create .orbit/{} directory", folder));
            }
            ep
        };
        // set the environment variable
        env::set_var(key, &dir);
        Ok(dir)
    }

    /// Loads all vendor files.
    pub fn read_vendors(mut self) -> Result<Self, Fault> {
        // read off all the files in the vendor.index array
        let indices = self.config.collect_as_array_of_str("vendor", "index")?;
        for index in indices {
            let r_path = filesystem::resolve_rel_path(index.1, index.0);
            let vendor = VendorManifest::from_path(&PathBuf::from(r_path))?;
            self.vendors.insert(vendor.get_name().clone(), vendor);
        }
        Ok(self)
    }

    /// References the cache directory.
    pub fn get_cache_path(&self) -> &PathBuf {
        &self.cache_path
    }

    /// References the store directory.
    pub fn get_store_path(&self) -> &PathBuf {
        &self.store_path
    }

    /// References the list of linked vendors.
    pub fn get_vendors(&self) -> &HashMap<PkgPart, VendorManifest> {
        &self.vendors
    }

    /// Configures and reads data from the settings object to return a `Settings` struct
    /// in the `Context`. 
    /// 
    /// The settings file `name` must be directly under `$ORBIT_HOME`. It also
    /// checks for a local configuration as `name` under a .orbit/ directory if
    /// the command is invoked from within an ip directory.
    /// 
    /// Note: the `self.ip_path` must already be determined before invocation.
    pub fn settings(mut self, name: &str) -> Result<Context, Fault> {
        // initialize and load the global configuration
        let cfg = Config::from_path(&self.home_path.join(name))?
            .include()?;

        // if in ip, also look along current directory for a /.orbit/config.toml file to load (local configuration) 
        self.config = if let Some(ip_dir) = self.get_ip_path() {
            cfg.local(&ip_dir.join(".orbit").join(name))?
        } else {
            cfg
        };

        // @TODO dynamically set from environment variables from configuration data

        // load plugins and templates
        self.plugins()?.templates()
    }

    /// Accesses the plugins in a map with `alias` as the keys.
    pub fn get_plugins(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    /// Iterates through an array of tables to define all plugins.
    fn plugins(mut self) -> Result<Context, Fault> {
        let plugs = self.config.collect_as_array_of_tables("plugin")?;

        for (arr_tbl, root) in plugs {
            for tbl in arr_tbl {
                let plug = match Plugin::from_toml(tbl) {
                    Ok(r) => r.set_root(&root), // resolve paths from that config file's parent directory
                    Err(e) => return Err(AnyError(format!("configuration {}: plugin {}", normalize_path(root.join(CONFIG_FILE)).display(), e)))?
                };
                // will kick out previous values so last item in array has highest precedence
                self.plugins.insert(plug.alias().to_owned(), plug);
            }
        }
        Ok(self)
    }

    /// References the templates in a map with `alias` as the keys.
    pub fn get_templates(&self) -> &HashMap<String, Template> {
        &self.templates
    }

    /// Iterates through the array of tables to define all templates.
    fn templates(mut self) -> Result<Context, Fault> {
        let temps = self.config.collect_as_array_of_tables("template")?;

        for (arr_tbl, root) in temps {
            for tbl in arr_tbl {
                let template = match Template::from_toml(tbl) {
                    Ok(r) => r.resolve_root_path(&root),
                    Err(e) => return Err(AnyError(format!("configuration {}: template {}", normalize_path(root.join(CONFIG_FILE)).display(), e)))?
                };
                self.templates.insert(template.alias().to_owned(), template);
            }
        }
        Ok(self)
    }

    /// Determines the orbit ip development path.
    /// 
    /// First checks if the environment already has ORBIT_DEV_PATH set, otherwise it
    /// will look for the value found in the config file. If no development path
    /// is set, it will use the current directory.
    /// 
    /// Note: Stange behavior where `edit` with vscode captures current ENV variables
    /// into new window to prevent reading config for things like ORBIT_DEV_PATH.
    /// 
    /// If `verify` is set to `true`, then it will ensure the path is a directory and exists.
    pub fn development_path(mut self, s: &str, verify: bool) -> Result<Context, Fault> {
        // an explicit environment variable takes precedence over config file data
        self.dev_path = Some(std::path::PathBuf::from(match std::env::var(s) {
            Ok(v) => v,
            Err(_) => {
                // use current directory if the key-value pair is not there
                let path = match self.get_config().get_as_str("core", "path")? {
                    // normalize
                    Some(p) => crate::util::filesystem::normalize_path(PathBuf::from(p.to_owned())).to_str().unwrap().to_string(),
                    None => std::env::current_dir().unwrap().display().to_string(),
                };
                std::env::set_var(s, &path);
                path
            }
        }));

        if verify == true {
            // verify the orbit path exists and is a directory
            if self.dev_path.as_ref().unwrap().exists() == false {
                return Err(ContextError(format!("orbit dev path '{}' does not exist", self.dev_path.as_ref().unwrap().display())))?
            } else if self.dev_path.as_ref().unwrap().is_dir() == false {
                return Err(ContextError(format!("orbit dev path '{}' is not a directory", self.dev_path.as_ref().unwrap().display())))?
            }
        }
        Ok(self)
    }

    /// Access the Orbit development path.
    pub fn get_development_path(&self) -> Option<&path::PathBuf> {
        self.dev_path.as_ref()
    }

    /// Access the configuration data.
    pub fn get_config(&self) -> &Config {
        &self.config
    }

    /// Access the configuration data as mutable.
    pub fn get_config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Access the build directory data.
    pub fn get_build_dir(&self) -> &String {
        &self.build_dir
    }

    /// Access the ip directory detected from the current working directory.
    pub fn get_ip_path(&self) -> Option<&path::PathBuf> {
        self.ip_path.as_ref()
    }
    
    /// Access the home path.
    pub fn get_home_path(&self) -> &path::PathBuf {
        &self.home_path
    }

    /// Determines if the directory is within a current IP and sets the proper
    /// runtime environment variable.
    pub fn current_ip_dir(mut self, s: &str) -> Result<Context, ContextError> {
        self.ip_path = match Context::find_ip_path(&std::env::current_dir().expect("failed to get current directory")) {
            Some(cwd) => {
                env::set_var(s, &cwd);
                Some(cwd)
            }
            None => None,
        };
        Ok(self)
    }

    /// Changes current working directory to the detected IP path.
    /// 
    /// Returns an error if ip_path is `None`.
    pub fn goto_ip_path(&self) -> Result<(), ContextError> {
        match self.get_ip_path() {
            Some(cwd) => {
                // set the current working directory to here
                std::env::set_current_dir(&cwd).expect("could not change directories");
            }
            None => {
                // @IDEA also give information about reading about ip-dir sensitive commands as a topic?
                return Err(ContextError(format!("no orbit IP detected in current directory")));
            }
        }
        Ok(())
    }

    /// Finds the complete path to the current IP's directory.
    /// 
    /// This function will recursively backtrack down the current working directory
    /// until finding the first directory with a file named "Orbit.toml".
    pub fn find_ip_path(dir: &std::path::PathBuf) -> Option<path::PathBuf> {
        //let mut cwd = std::env::current_dir().expect("could not locate cwd");
        let mut cwd = dir.clone();
        // search for the manifest file
        loop {
            let mut entries = std::fs::read_dir(&cwd).expect("could not read cwd");
            let result = entries.find_map(|p| {
                match p {
                    Ok(file) => {
                        if file.file_name() == "Orbit.toml" {
                            Some(cwd.to_path_buf())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            });
            if let Some(r) = result {
                break Some(r)
            } else if cwd.pop() == false {
                break None
            }
        }
    }

    /// Sets the IP's build directory and the corresponding environment variable.
    pub fn build_dir(mut self, s: &str) -> Result<Context, ContextError> {
        self.build_dir = String::from("build");
        env::set_var(s, &self.build_dir);
        Ok(self)
    }
}

#[derive(Debug)]
pub struct ContextError(String);

impl std::error::Error for ContextError {}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
