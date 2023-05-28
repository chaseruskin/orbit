use std::collections::HashSet;
use crate::core::v2::plugin::{Plugins, Plugin};
use crate::core::v2::protocol::Protocols;
use std::str::FromStr;
use crate::core::v2::manifest::FromFile;
use std::path::PathBuf;
use std::collections::HashMap;
use crate::util::anyerror::AnyError;
use std::error::Error;
use crate::util::filesystem;
use crate::util::filesystem::Standardize;
use crate::core::v2::protocol::Protocol;

use serde_derive::{Serialize, Deserialize};
use toml_edit::Document;

#[derive(Debug)]
pub struct ConfigDocument {
    document: Document,
}

impl FromStr for ConfigDocument {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // verify all keys are valid during deserializing
        let _ : Config = toml::from_str(s)?;
        Ok(Self {
            document: s.parse::<Document>().unwrap(),
        })
    }
}

const INCLUDE_KEY: &str = "include";
use toml_edit::Table;
use toml_edit::Item;
use toml_edit::Value;
use toml_edit::Array;
use toml_edit::Formatted;
use crate::util::anyerror::Fault;

impl ConfigDocument {
    pub fn print(&self) {
        println!("{}", self.document.to_string())
    }

    fn append_list(table: &mut Table, key: &str, item: &str) -> () {
        // verify the key/entry exists (make empty array)
        if table.contains_key(key) == false {
            table.insert(key, Item::Value(Value::Array(Array::new())));
        }
        table[key].as_array_mut().unwrap().push(item);
        // before neat formatting of an item on every line
        table[key].as_array_mut().unwrap().iter_mut().for_each(|f| {
            f.decor_mut().set_prefix("\n    ");
            f.decor_mut().set_suffix("");
        });
        table[key].as_array_mut().unwrap().set_trailing("\n");
    }

    /// Adds a new value to the `include` entry.
    /// 
    /// Automatically creates the new key if it does not exist.
    pub fn append_include(&mut self, item: &str) -> () {
        Self::append_list(&mut self.document, INCLUDE_KEY, item);
    } 

    /// Sets a value for the given entry in the toml document.
    /// 
    /// Creates parent table and/or key if does not exist.
    pub fn set(&mut self, table: &str, key: &str, value: &str) -> () {
        // create table if it does not exist
        if self.document.contains_key(table) == false {
            self.document.insert(table, Item::Table(Table::new()));
        }
        // create key if it does not exist
        let table = self.document.get_mut(table).unwrap().as_table_mut().unwrap();
        // insert/overwrite into the table
        table.insert(key, Item::Value(Value::String(Formatted::<String>::new(value.to_string()))));
    }

    /// Removes an entry from the toml document.
    /// 
    /// Errors if the entry does not exist.
    pub fn unset(&mut self, table: &str, key: &str) -> Result<(), Fault> {
        if self.document.contains_key(table) == false {
            return Err(AnyError(format!("key '{}.{}' does not exist in configuration", table, key)))?
        }
        // remnove the key if it does exist
        let toml_table = self.document.get_mut(table).unwrap().as_table_mut().unwrap();
        match toml_table.contains_key(key) {
            true => {
                toml_table.remove(key);
                Ok(())
            },
            false => {
                Err(AnyError(format!("key '{}.{}' does not exist in configuration", table, key)))?
            }
        }
    }

    /// Writes the `document` to the `path`.
    /// 
    /// Uses CONFIG_FILE as the filename to save to.
    pub fn write(&mut self, dest: &PathBuf) -> Result<(), Fault> {
        let contents = self.document.to_string();
        std::fs::write(&dest, contents)?;
        Ok(())
    }

}

impl FromFile for ConfigDocument {
    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        match Self::from_str(&contents) {
            Ok(r) => Ok(r),
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!("failed to parse {} file: {}", path.display(), e)))?
            }
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum Locality {
    Global,
    Local,
    Other,
}

#[derive(Debug, PartialEq)]
pub struct Configs {
    inner: Vec<(PathBuf, Config, Locality)>,
}

impl Configs {

    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    pub fn load(self, file: PathBuf, lvl: Locality) -> Result<Self, Box<dyn Error>> {
        // create a set to remember what paths are already loaded
        let mut set = HashSet::new();
        let mut configs = self.inner;

        // standardize the path
        let mut to_process = vec![(PathBuf::standardize(file), lvl)];
        let mut i = 0;
        while to_process.get(i).is_some() == true {
            {
                let (path, local) = to_process.get(i).unwrap();
                // load the entry file
                let cfg = Config::from_file(&path)?;
                set.insert(path.clone());
                configs.push((path.clone(), cfg, local.clone())); 
            }
            let base = configs.last().unwrap().0.parent().unwrap().to_path_buf();
            // access its neighboring files (check "include" key)
            for next in configs.last().unwrap().1.get_includes() {
                // avoid processing the same files
                let std_next = filesystem::resolve_rel_path2(&base, next);
                if set.contains(&std_next) == false {
                    to_process.push((std_next, Locality::Other));
                }
            }
            i += 1;
        }

        Ok(Self {
            inner: configs
        })
    }

    pub fn get_plugins(&self) -> HashMap<&str, &Plugin> {
        // iterate through all linked configs
        let mut map = HashMap::new();

        self.inner.iter().for_each(|(_path, cfg, _lvl)| {
            if let Some(plugs) = &cfg.plugin {
                plugs.iter().for_each(|p| {
                    match map.get(p.get_alias()) {
                        Some(_) => (),
                        None => { map.insert(p.get_alias().clone(), p); () }
                    }
                });
            }
        });
        map
    }

    pub fn get_global(&self) -> (&PathBuf, &Config) {
        let cfg = &self.inner.iter().filter(|(_, _, l)| l == &Locality::Global).next().unwrap();
        (&cfg.0, &cfg.1)
    }
}

impl From<Configs> for Config {
    /// Transform the multi-layered configurations into a single level
    fn from(value: Configs) -> Self {
        let mut single = Config::new();

        value.inner.into_iter().for_each(|p| {
            single.append(p.1);
        });
        single
    }
}

pub const CONFIG_FILE: &str = "config.toml";

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Config {
    include: Option<Vec<PathBuf>>,
    env: Option<HashMap<String, String>>,
    plugin: Option<Plugins>,
    protocol: Option<Protocols>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            include: None,
            env: None,
            plugin: None,
            protocol: None,
        }
    }

    /// Adds `path` to the end of the list for the include attribute.
    /// 
    /// This function creates some vector if no vector originally exists.
    pub fn append_include(&mut self, path: &str) {
        match &self.include.is_some() {
            true => self.include.as_mut().unwrap().push(PathBuf::from(path)),
            false => self.include = Some(vec![PathBuf::from(path)]),
        }
    }

    /// Adds the new information to the existing configuration to combine data.
    pub fn append(&mut self, rhs: Self) {

        match &mut self.include {
            Some(v) => v.append(&mut rhs.include.unwrap_or(Vec::new())),
            None => self.include = rhs.include,
        }

        match &mut self.plugin {
            Some(v) => v.append(&mut rhs.plugin.unwrap_or(Vec::new())),
            None => self.plugin = rhs.plugin,
        }

        match &mut self.protocol {
            Some(v) => v.append(&mut rhs.protocol.unwrap_or(Vec::new())),
            None => self.protocol = rhs.protocol,
        }

        match &mut self.env {
            Some(v) => v.extend(rhs.env.unwrap_or(HashMap::new()).into_iter()),
            None => self.env = rhs.env,
        }
    }

    pub fn get_includes(&self) -> Vec<&PathBuf> {
        match &self.include {
            Some(i) => i.iter().collect(),
            None => Vec::new(),
        }
    }

    pub fn get_plugins(&self) -> HashMap<&str, &Plugin> {
        let mut map = HashMap::new();

        if let Some(plugs) = &self.plugin {
            plugs.iter().for_each(|p| {
                match map.get(p.get_alias()) {
                    Some(_) => (),
                    None => { map.insert(p.get_alias().clone(), p); () }
                }
            });
        }
        map
    }

    pub fn get_env(&self) -> &Option<HashMap<String, String>> {
        &self.env
    }

    pub fn get_protocols(&self) -> HashMap<&str, &Protocol> {
        let mut map = HashMap::new();

        if let Some(plugs) = &self.protocol {
            plugs.iter().for_each(|p| {
                match map.get(p.get_name()) {
                    Some(_) => (),
                    None => { map.insert(p.get_name().clone(), p); () }
                }
            });
        }
        map
    }
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

impl FromFile for Config {

    fn from_file(path: &PathBuf) -> Result<Self, Box<dyn Error>> {
        // verify the path exists
        if path.is_file() == false {
            return Err(AnyError(format!("failed to locate configuration file \"{}\"", path.display())))?
        }
        // open file
        let contents = std::fs::read_to_string(&path)?;
        // parse toml syntax
        match Self::from_str(&contents) {
            Ok(mut r) => {
                // set roots for plugins and protocols
                let base = PathBuf::standardize(path).parent().unwrap().to_path_buf();
                if let Some(protos) = &mut r.protocol {
                    protos.iter_mut().for_each(|p| {
                        p.set_root(base.clone());
                    });
                }
                if let Some(plugs) = &mut r.plugin {
                    plugs.iter_mut().for_each(|p| {
                        p.set_root(base.clone());
                    });
                }
                Ok(r)
            },
            // enter a blank lock file if failed (do not exit)
            Err(e) => {
                return Err(AnyError(format!("failed to parse \"{}\" file: {}", path.display(), e)))?
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const C_0: &str = r#"
# This is a blank configuration file.
"#;

    const C_1: &str = r#"
# orbit configuration file

# list of configuration files
include = [
    "path/to/other/config.toml",
]
    
[[plugin]]
alias = "quartus"
summary = "Complete toolflow for Intel Quartus Prime backend program"
command = "python"
args = ["./plugin/quartus.py"]
fileset.pin-plan = "*.board"
fileset.bdf-file = "*.bdf"

[env]
COURSE = "Reconfigurable Computing [EEL5721]"
QUARTUS_PATH = "C:/intelFPGA_lite/19.1/quartus/bin64/"
VCD_VIEWER = "dwfv"

[[protocol]]
name = "kstp"
command = "python"
args = ["./download.py"]
"#;

    #[test]
    fn parse_empty_config() {
        match Config::from_str(C_0) {
            Ok(r) => assert_eq!(r, Config::new()),
            Err(e) => { println!("{}", e); panic!("failed to parse") }
        }
    }

    #[test]
    fn parse_basic_config() {
        match Config::from_str(C_1) {
            Ok(r) => assert_ne!(r, Config::new()),
            Err(e) => { println!("{}", e); panic!("failed to parse") }
        }
    }

    #[test]
    fn linked_configs() {
        Configs::new().load(PathBuf::from("./tests/data/config1.toml"), Locality::Global).unwrap();
    }
}