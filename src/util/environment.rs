use crate::util::anyerror::Fault;
use std::collections::HashSet;
use std::hash::Hash;
use std::io::Write;
use std::io::Read;

#[derive(Eq)]
pub struct EnvVar { key: String, value: String }

impl PartialEq for EnvVar {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Hash for EnvVar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // only hash by the key name
        self.key.hash(state);
    }
}

impl EnvVar {
    pub fn new() -> Self {
        Self { key: String::new(), value: String::new() }
    }

    /// Sets the environment key.
    pub fn key(mut self, s: &str) -> Self {
        // normalize the key name upon entry
        self.key = s.to_ascii_uppercase().replace('-', "_");
        self
    }

    /// Sets the environment value.
    pub fn value(mut self, s: &str) -> Self {
        self.value = s.to_owned();
        self
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Debug for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}=\"{}\"", self.key, self.value)
    }
}

impl std::fmt::Display for EnvVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}

/// Sets environment variables from a '.env' file living at `root`.
/// 
/// Silently skips text lines that do not have proper delimiter `=` between key and value.
pub fn load_environment(root: &std::path::PathBuf) -> Result<Environment, Fault> {
    let mut envs = HashSet::new();
    // read the .env file
    let env_file = root.join(".env");
    if env_file.exists() == true {
        let mut file = std::fs::File::open(env_file).expect("failed to open .env file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("failed to read contents");
        // transform into environment variables
        for line in contents.split_terminator('\n') {
            let result = line.split_once('=');
            // set env variables
            if let Some((name, value)) = result {
                envs.insert(EnvVar::new().key(name).value(value));
            }
        }
    }
    Ok(Environment(envs))
}

/// Sets a set of environment variables, consuming the list.
pub fn set_environment(envs: Environment) -> () {
    envs.into_iter().for_each(|e| {
        std::env::set_var(e.key, e.value)
    });
}

/// Stores a list of `EnvVar` at root in a file named ".env".
pub fn save_environment(env: &Environment, root: &std::path::PathBuf) -> Result<(), Fault> {
    // create the file
    let mut env_file = std::fs::File::create(&root.join(".env")).expect("could not create .env file");
    // prepare the data into a single string for writing
    let contents = env.iter()
        .fold(String::new(), |x, y| x + &y.to_string() + &"\n");
    // write the data to the file
    env_file.write_all(contents.as_bytes()).expect("failed to write data to .env file");
    Ok(())
}

use std::collections::hash_set::Iter;
use std::collections::hash_set::IntoIter;

pub struct Environment(HashSet<EnvVar>);

impl Environment {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn insert(&mut self, var: EnvVar) -> bool {
        self.0.insert(var)
    }

    pub fn iter(&self) -> Iter<'_, EnvVar> {
        self.0.iter()
    }

    pub fn into_iter(self) -> IntoIter<EnvVar> {
        self.0.into_iter()
    }

    pub fn from_vec(vec: Vec<EnvVar>) -> Self {
        let mut inner = HashSet::new();
        vec.into_iter().for_each(|e| { inner.insert(e); () } );
        Self(inner)
    }

    pub fn get(&self, key: &str) -> Option<&EnvVar> {
        self.0.get(&EnvVar::new().key(key))
    }
}


pub const ORBIT_PLUGIN: &str = "ORBIT_PLUGIN";
pub const ORBIT_TOP: &str = "ORBIT_TOP";
pub const ORBIT_BENCH: &str = "ORBIT_BENCH";
pub const ORBIT_BUILD_DIR: &str = "ORBIT_BUILD_DIR";
pub const ORBIT_ENV_PREFIX: &str = "ORBIT_ENV_";