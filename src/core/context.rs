use std::path;
use std::env;
use toml_edit::Document;

pub struct Context {
    home_path: path::PathBuf,
    cache_path: path::PathBuf,
    ip_path: Option<path::PathBuf>,
    build_dir: String,
    config: Document,
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
        // :todo: also look within every path along current directory for a /.orbit/config.toml file to load

        // :todo: dynamically set from environment variables from configuration data
        Ok(self)
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
