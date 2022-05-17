use std::path;
use std::env;
use toml_edit::Document;
use std::collections::HashMap;
use crate::core::plugin::Plugin;
use crate::core::plugin::FromToml;

pub struct Context {
    home_path: path::PathBuf,
    cache_path: path::PathBuf,
    ip_path: Option<path::PathBuf>,
    dev_path: Option<path::PathBuf>,
    build_dir: String,
    config: Document,
    plugins: HashMap<String, Plugin>, // @IDEA optionally move hashmap out of context and create it from fn to allow dynamic loading
    pub force: bool,
}

impl Context {
    pub fn new() -> Context {
        let home = std::env::temp_dir();
        let cache = home.join("cache");
        Context { 
            home_path: home,
            cache_path: cache,
            ip_path: None,
            dev_path: None,
            plugins: HashMap::new(),
            config: Document::new(),
            build_dir: String::new(),
            force: false,
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

    /// Sets the cache directory. If it was set from `var`, it assumes the path
    /// exists. If setting by default (within HOME), it assumes HOME is already existing.
    pub fn cache(mut self, key: &str) -> Result<Context, ContextError> {
        self.cache_path = if let Ok(s) = env::var(key) {
            let cp = std::path::PathBuf::from(s);
            // do not allow a nonexistent directory to be set for cache path
            if path::Path::exists(&cp) == false {
                return Err(ContextError(format!("directory {} does not exist for ORBIT_CACHE", cp.display())))
            }
            cp
        // proceed with default
        } else {
            let cp = self.home_path.join("cache");
            // create the directory if does not exist
            if path::Path::exists(&cp) == false {
                std::fs::create_dir(&cp).expect("failed to create .orbit/cache directory");
            }
            cp
        };
        // verify the environment variable is set
        env::set_var(key, &self.cache_path);
        Ok(self)
    }

    /// Accesses the cache directory.
    pub fn get_cache_path(&self) -> &std::path::PathBuf {
        &self.cache_path
    }

    /// Configures and reads data from the settings object to return a `Settings` struct
    /// in the `Context`. The settings file `s` must be directly under `$ORBIT_HOME`.
    pub fn settings(mut self, s: &str) -> Result<Context, ContextError> {
        // create the settings file if does not exist
        let cfg_path = self.home_path.join(s);
        if path::Path::exists(&cfg_path) == false {
            std::fs::write(&cfg_path, "").expect("failed to create settings file");
        }
        let toml = std::fs::read_to_string(&cfg_path).expect("could not read string");
        let doc = toml.parse::<Document>();
        self.config = match doc {
            Ok(d) => d,
            Err(er) => return Err(ContextError(er.to_string()))
        };
        // @TODO also look within every path along current directory for a /.orbit/config.toml file to load

        // @TODO dynamically set from environment variables from configuration data

        // load plugins
        self = match self.plugins() {
            Ok(s) => s,
            Err(e) => return Err(ContextError(e.to_string())),
        };

        Ok(self)
    }

    /// Accesses the plugins in a map with `alias` as the keys.
    pub fn get_plugins(&self) -> &HashMap<String, Plugin> {
        &self.plugins
    }

    /// Iterates through an array of tables to define all plugins.
    fn plugins(mut self) -> Result<Context, Box<dyn std::error::Error>> {
        let cfg_path = self.home_path.join("config.toml");
        let arr_of_tbl = if let Some(arr) = self.config.get("plugin") {
            match arr {
                toml_edit::Item::ArrayOfTables(aot) => aot,
                _ => return Err(ContextError(format!("{} plugin expects to be array of tables", cfg_path.display())))?
            }
        } else {
            return Ok(self)
        };

        for tbl in arr_of_tbl {
            let plug = Plugin::from_toml(tbl)?
                .resolve_all_paths(&self.home_path); // resolve paths from the config file's parent directory
            self.plugins.insert(plug.alias().to_owned(), plug);
        }
        Ok(self)
    }

    /// Attempts to get the value of behind a key.
    /// 
    /// Dots are used to split among multiple `get` function calls. Panics if
    /// an invalid key path is passed.
    fn get_value_as_str<'a>(doc: &'a Document, key_path: &str) -> Option<&'a str> {
        let keys: Vec<&str> = key_path.split_terminator('.').collect();
        let mut keys_iter = keys.iter();
        let mut table = doc.get(keys_iter.next().expect("passed in empty key"))?;
        while let Some(key) = keys_iter.next() {
            table = table.get(key)?;
        }
        table.as_str()
    }

    /// Determines the orbit ip development path.
    /// 
    /// First checks if the environment already has ORBIT_PATH set, otherwise it
    /// will look for the value found in the config file. If no development path
    /// is set, it will use the current directory.
    pub fn development_path(mut self, s: &str) -> Result<Context, ContextError> {
        // an explicit environment variable takes precedence over config file data
        self.dev_path = Some(std::path::PathBuf::from(match std::env::var(s) {
            Ok(v) => v,
            Err(_) => {
                // use current directory if the key-value pair is not there
                let path = match Self::get_value_as_str(self.get_config(), "core.path") {
                    Some(p) => p.to_owned(),
                    None => std::env::current_dir().unwrap().display().to_string(),
                };
                std::env::set_var(s, &path);
                path
            }
        }));
        // verify the orbit path exists and is a directory
        if self.dev_path.as_ref().unwrap().exists() == false {
            return Err(ContextError(format!("orbit path '{}' does not exist", self.dev_path.as_ref().unwrap().display())));
        } else if self.dev_path.as_ref().unwrap().is_dir() == false {
            return Err(ContextError(format!("orbit path '{}' is not a directory", self.dev_path.as_ref().unwrap().display())));
        }
        Ok(self)
    }

    /// Access the Orbit development path.
    pub fn get_development_path(&self) -> Option<&path::PathBuf> {
        self.dev_path.as_ref()
    }

    /// Access the configuration data.
    pub fn get_config(&self) -> &Document {
        &self.config
    }

    /// Access the build directory data.
    pub fn get_build_dir(&self) -> &String {
        &self.build_dir
    }

    /// Access the ip directory detected from the current working directory.
    pub fn get_ip_path(&self) -> Option<&path::PathBuf> {
        self.ip_path.as_ref()
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
