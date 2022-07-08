use toml_edit::{Document, ArrayOfTables, Item, Array, Value, Table, Formatted};
use std::path::PathBuf;
use crate::util::{anyerror::{AnyError, Fault}, filesystem::normalize_path};

pub trait FromToml {
    type Err;
    /// Parses a toml table into a struct.
    fn from_toml(table: &toml_edit::Table) -> Result<Self, Self::Err> where Self: Sized;

    /// Attempts to deserialize an entry in a table from a `String` to `T`.
    /// 
    /// An `Ok` result is safe to unwrap if `require` is true.
    /// 
    /// Errors if the entry is not a string or the entry failed to parse.
    fn get<T: std::str::FromStr>(table: &toml_edit::Table, s: &str) -> Result<Option<T>, FromTomlError>
    where <T as std::str::FromStr>::Err: std::error::Error {
        let result = match table.get(s) {
            Some(item) => {
                match item.as_str() {
                    Some(i) => i.parse::<T>(),
                    None => return Err(FromTomlError::ExpectingString(s.to_owned()))?
                }
            },
            None => return Ok(None),
        };
        match result {
            Ok(r) => Ok(Some(r)),
            Err(e) => Err(FromTomlError::BadParse(s.to_string(), e.to_string()))
        } 
    }

    /// Ensures a value is taken and the entry exists, otherwise it throws an error.
    fn require<T: std::str::FromStr>(table: &toml_edit::Table, s: &str) -> Result<T, FromTomlError>
    where <T as std::str::FromStr>::Err: std::error::Error {
        match Self::get(table, s)? {
            Some(value) => Ok(value),
            None => Err(FromTomlError::MissingEntry(s.to_owned()))?,
        }
    }

}

#[derive(Debug)]
pub enum FromTomlError {
    MissingEntry(String),
    ExpectingString(String),
    BadParse(String, String),
    ExpectingStringArray(String),
}

impl std::error::Error for FromTomlError {}

impl std::fmt::Display for FromTomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectingString(key) => write!(f, "key '{}' expects a toml string", key),
            Self::MissingEntry(key) => write!(f, "missing required key '{}'", key),
            Self::BadParse(key, value) => write!(f, "failed to parse value '{}' for key '{}'", value, key),
            Self::ExpectingStringArray(key) => write!(f, "key '{}' expects an array of strings", key),
        }
    }
}

pub struct Config {
    root: PathBuf,
    document: Document,
    includes: Vec<Box<Config>>,
    local: Option<Box<Config>>,
}

impl Config {
    /// Creates a new empty `Config` struct.
    pub fn new() -> Self {
        Self {
            root: PathBuf::new(),
            document: Document::new(),
            includes: Vec::new(),
            local: None,
        }
    }

    /// Initializes a configuration file from `file`.
    /// 
    /// Creates the file if it does not exist. Assumes the file is .toml file.
    pub fn from_path(file: &PathBuf) -> Result<Self, Fault> {
        if file.exists() == false {
            // create all missing intermediate directories as well 
            std::fs::create_dir_all(file.parent().unwrap())?;
            std::fs::File::create(&file)?;
        }
        let contents = std::fs::read_to_string(file)?;
        Ok(Self {
            root: file.parent().unwrap().to_path_buf(), 
            document: contents.parse::<Document>()?,
            includes: Vec::new(),
            local: None,
        })
    }

    /// Updates the configuration to see if there is a local project-based configuration
    /// to track.
    /// 
    /// Keeps `self.local` set to `None` if the file `file` does not exist.
    pub fn local(mut self, file: &PathBuf) -> Result<Self, Fault> {
        if file.exists() == true {
            self.local = Some(Box::new(Self::from_path(&file)?));
        }
        Ok(self)
    }

    /// References the .toml `Document` struct.
    pub fn get_doc(&self) -> &Document {
        &self.document
    }

    /// References the root directory where the file is located.
    pub fn get_root(&self) -> &PathBuf {
        &self.root
    }

    /// Adds Configurations from the `include` key.
    /// 
    /// Ignores configuration files that does not exist.
    pub fn include(mut self) -> Result<Self, Fault> {
        // detect the include entry
        if self.document.contains_key("include") == false {
            return Ok(self)
        }
        let config_paths: Vec<PathBuf> = self.document.get("include")
            .unwrap()
            .as_array()
            .unwrap()
            .into_iter()
            .filter_map(|f| f.as_str())
            .map(|f| PathBuf::from(f))
            // resolve paths with root (@TODO error on bad paths?)
            .map(|f| if f.is_relative() { self.root.join(f) } else { f })
            .collect();
        for file in &config_paths {
            self.includes.push(Box::new(Config::from_path(file)?));
        }
        Ok(self)
    }

    /// Adds a new value to the `include` entry.
    /// 
    /// Automatically creates the new key if it does not exist.
    pub fn append_include(&mut self, item: &str) -> () {
        if self.document.contains_key("include") == false {
            self.document.insert("include", Item::Value(Value::Array(Array::new())));
        }
        self.document["include"].as_array_mut().unwrap().push(item);
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
        if table.contains_key(key) == false {
            table.insert(key, Item::Value(Value::String(Formatted::<String>::new(value.to_string()))));
        }
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
    pub fn write(&mut self) -> Result<(), Fault> {
        let contents = self.document.to_string();
        std::fs::write(self.get_root().join(CONFIG_FILE), contents)?;
        Ok(())
    }

    /// Gathers all values assigned to the `table.key` entry in configuration.
    /// 
    /// Errors if the entry exists, but is not a string.
    /// Returns `Vec::new()` if the entry does not exist anywhere.
    pub fn collect_as_str<'a>(&'a self, table: &str, key: &str) -> Result<Vec<&'a str>, Fault> {
        Ok(self.collect_as_item(Some(table), key, &Item::is_str, "string")?.into_iter().map(|f| f.0.as_str().unwrap()).collect())
    }

    /// Gathers all values assigned under a given `Array` entry in configuration.
    /// 
    /// The list is given with priority items first (base configurations), then
    /// extra included items to follow.
    /// 
    /// Errors if the entry exists, but is not an array.
    /// Returns `Vec::new()` if the entry does not exist anywhere.
    pub fn collect_as_array_of_tables<'a>(&'a self, key: &str) -> Result<Vec<(&ArrayOfTables, &PathBuf)>, Fault> {
        Ok(self.collect_as_item(None, key, &Item::is_array_of_tables, "array of tables")?.into_iter().map(|f| (f.0.as_array_of_tables().unwrap(), f.1)).collect())
    }

    /// Takes the last value.
    pub fn get_as_str(&self, table: &str, key: &str) -> Result<Option<&str>, Fault> {
        let mut values = self.collect_as_str(table, key)?;
        Ok(match values.len() {
            0 => None,
            _ => Some(values.remove(values.len()-1))
        })
    }

    /// Tries to visit a value at `table.key`.
    /// 
    /// If `table` is `None`, it will assume its a global-level key/item.
    /// 
    /// Returns `None` if unable to reach a value.
    fn access(&self, table: Option<&str>, key: &str) -> Option<&toml_edit::Item> {
        match table {
            Some(t) => self.get_doc().get(t)?.as_table()?.get(key),
            None => self.get_doc().get(key),
        }
    }

    /// Gathers all values assigned to the `table.key` entry in configuration that
    /// match with the `eval` fn.
    /// 
    /// The result is safe to unwrap as the evaluated struct. Returns `Vec::new()` if
    /// the entry does not exist anywhere.
    /// 
    /// Errors if the entry exists, but is not an item that evaluates true with `eval`.
    fn collect_as_item<'a>(&'a self, table: Option<&str>, key: &str, eval: &dyn Fn(&Item) -> bool, item_name: &str) -> Result<Vec<(&Item, &PathBuf)>, Fault> {
        let mut values: Vec<(&Item, &PathBuf)> = Vec::new();
        // collect all included (3rd-party) configuration data
        for inc in &self.includes {
            match inc.access(table, key) {
                Some(item) => {
                    // update the value as the list continues
                    if eval(item) {
                       values.push((item, inc.get_root()));
                    } else {
                        return Err(ConfigError::BadItem(format!("{}", normalize_path(self.get_root().join(CONFIG_FILE)).display()), 
                            item_name.to_owned(), format!("{}{}", { if table.is_some() { table.unwrap().to_string() + "." } else { "".to_string() } }, 
                            key.to_owned())))?
                    }
                }
                None => (),
            }
        }
        // access on current configuration
        match self.access(table, key) {
            Some(item) => {
                // update the value as the list continues
                if eval(item) {
                   values.push((item, self.get_root()));
                } else {
                    return Err(ConfigError::BadItem(format!("{}", normalize_path(self.get_root().join(CONFIG_FILE)).display()), 
                        item_name.to_owned(), format!("{}{}", { if table.is_some() { table.unwrap().to_string() + "." } else { "".to_string() } }, 
                        key.to_owned())))?
                }
            }
            None => (),
        }
        // access on local configuration
        if let Some(cfg) = &self.local {
            match cfg.access(table, key) {
                Some(item) => {
                    // update the value as the list continues
                    if eval(item) {
                       values.push((item, cfg.get_root()));
                    } else {
                        return Err(ConfigError::BadItem(format!("{}", normalize_path(self.get_root().join(CONFIG_FILE)).display()), 
                            item_name.to_owned(), format!("{}{}", { if table.is_some() { table.unwrap().to_string() + "." } else { "".to_string() } }, 
                            key.to_owned())))?
                    }
                }
                None => (),
            }
        }
        Ok(values)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    /// (file, item, key)
    BadItem(String, String, String),
}

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadItem(file, item, key) => write!(f, "configuration {}: expecting toml {} for key '{}'", file, item, key),
        }
    }
}

pub const CONFIG_FILE: &str = "config.toml";

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn access_includes() {
        let cfg = Config::from_path(&PathBuf::from("./test/data/config/config.toml"))
            .unwrap()
            .include().unwrap();

        // available in both configurations
        assert_eq!(cfg.collect_as_str("core", "editor").unwrap(), vec!["vim", "code"]);
        assert_eq!(cfg.get_as_str("core", "editor").unwrap(), Some("code"));
        // not available in any configuration
        assert_eq!(cfg.get_as_str("core", "user").unwrap(), None);
        // only seen in include's configuration
        assert_eq!(cfg.get_as_str("core", "build-dir").unwrap(), Some("build"));
    }

    #[test]
    fn collect_all_top_level_arrays() {
        let cfg = Config::from_path(&PathBuf::from("./test/data/config/config.toml"))
            .unwrap()
            .include().unwrap();

        // seen in both configuration files
        assert_eq!(cfg.collect_as_array_of_tables("plugin").unwrap().len(), 2);
        // only seen in include's configuration
        assert_eq!(cfg.collect_as_array_of_tables("template").unwrap().len(), 1);
    }

    #[test]
    fn collect_items() {
        let cfg = Config::from_path(&PathBuf::from("./test/data/config/config.toml"))
            .unwrap()
            .include().unwrap();

        let items: Vec<&str> = cfg.collect_as_item(Some("core"), "editor", &Item::is_str, "string")
            .unwrap().into_iter().map(|f| f.0.as_str().unwrap()).collect();
        assert_eq!(items, vec!["vim", "code"]);
    }
}