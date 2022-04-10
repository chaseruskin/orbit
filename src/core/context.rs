use std::path;
use std::env;

pub struct Context {
    home_path: path::PathBuf,
    cache_path: path::PathBuf,
}

impl Context {
    pub fn new() -> Context {
        let home = std::env::temp_dir();
        let cache = home.join("cache");
        Context { 
            home_path: home,
            cache_path: cache, 
        }
    }

    /// Sets the home directory. By default this is `$HOME/.orbit`. If set by `var`,
    /// it must be an existing directory.
    pub fn home(mut self, key: &str) -> Context {
        self.home_path = if let Ok(s) = env::var(key) {
            std::path::PathBuf::from(s)
        } else {
            let hp = home::home_dir().expect("no home directory detected").join(".orbit");
            // create the directory if does not exist
            if path::Path::exists(&hp) == false {
                std::fs::create_dir(&hp).expect("failed to create .orbit directory");
            }
            hp
        };
        // do not allow a non-existent directory to be set for the home
        if path::Path::exists(&self.home_path) == false {
            panic!("the home directory does not exist");
        }
        // verify the environment variable is set
        env::set_var("ORBIT_HOME", &self.home_path);
        self
    }   

    /// Sets the cache directory. If it was set from `var`, it assumes the path
    /// exists. If setting by default (within HOME), it assumes HOME is already existing.
    pub fn cache(mut self, key: &str) -> Context {
        self.cache_path = if let Ok(s) = env::var(key) {
            let cp = std::path::PathBuf::from(s);
            // do not allow a nonexistent directory to be set for cache path
            if path::Path::exists(&cp) == false {
                panic!("the cache directory does not exist");
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
        self
    }

    /// Configures and reads data from the settings object to return a `Settings` struct
    /// in the `Context`. The settings file `s` must be directly under `$ORBIT_HOME`.
    pub fn settings(self, s: &str) -> Context {
        // create the settings file if does not exist
        let settings_path = self.home_path.join(s);
        if path::Path::exists(&settings_path) == false {
            std::fs::write(&settings_path, "").expect("failed to create settings file");
        }
        // :todo: read the data from the settings file and return `Settings` struct
        self
    }
}
